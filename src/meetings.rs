//! Meeting links: where they are kept, and the seam a real meeting provider
//! plugs into.
//!
//! Meet does three things with a link — mint one, list the ones this account
//! owns, and resolve one somebody followed — and all three are here rather
//! than in `page.rs`, because two pages need them: `/meet` lists and mints,
//! `/meet/j/{id}` resolves.
//!
//! ## The provider seam
//!
//! [`provision`] is the one function that knows how a meeting comes into
//! existence. Today it mints an identifier and nothing else: there is no call
//! to join yet, only a URL that resolves to a lobby. When a real provider
//! arrives — AWS Chime SDK Meetings is the obvious candidate — it is that
//! function that calls it and returns what the provider hands back, and
//! [`Provisioned`] that grows the fields to carry it. Validation, storage,
//! the list, the join route and the revoke are already in place around it and
//! do not change.
//!
//! ## Where rows live
//!
//! The database, when the process has one. A deployment with no
//! `DATABASE_URL` keeps them in memory for the life of the process instead,
//! which is enough to develop against — a browser reload still shows the list
//! — and is deliberately not enough to ship on: a restart loses them. The
//! moment a connection string is configured and `db::migrate()` has run, the
//! same calls write rows.

use std::sync::{Mutex, OnceLock};

use serde::Serialize;

use crate::db;
use crate::models::meeting_link;
use crate::rahti::time::{self, DateTime, Utc};

/// A meeting link, in the shape the browser is given it.
///
/// `url` is a path, not an absolute URL. The server does not know which host
/// the reader typed — a development machine, a LAN address and the public
/// name are all the same application — so the page builds the absolute form
/// with `location.origin` at the moment somebody asks to copy it.
#[derive(Clone, Debug, Serialize)]
pub struct MeetingLink {
    /// The join code.
    pub id: String,
    /// What the organizer called the meeting.
    pub title: String,
    /// Where the link points, as a path: `/meet/j/{id}`.
    pub url: String,
    /// "just now", "4 minutes ago", "Oct 22, 2026" — what the card shows.
    pub created: String,
    /// The instant, for anything that needs to compare rather than read.
    pub created_at: DateTime<Utc>,
}

/// What a provider hands back when a meeting is created.
///
/// One field today. A provider that returns a conference id, a region, a
/// media placement or a set of attendee credentials adds them here, and
/// [`create`] decides which of them are worth a column.
pub struct Provisioned {
    /// The identifier the join URL carries.
    pub id: String,
}

/// Bring a meeting into existence.
///
/// The identifier is the server's to hand out. A browser that picked its own
/// could collide with somebody else's meeting, and a guessable one is a
/// meeting strangers can walk into, so both halves of that — the round trip
/// and the entropy — are the point even while nothing is dialled yet.
async fn provision(_title: &str) -> crate::rahti::Result<Provisioned> {
    Ok(Provisioned { id: mint_id() })
}

/// Mint a link and keep it.
pub async fn create(title: String) -> crate::rahti::Result<MeetingLink> {
    let provisioned = provision(&title).await?;
    let now = Utc::now();

    if db::is_connected() {
        use sea_orm::ActiveModelTrait;
        use sea_orm::ActiveValue::Set;

        let saved = meeting_link::ActiveModel {
            id: Set(provisioned.id),
            title: Set(title),
            created_at: Set(now),
            revoked_at: Set(None),
        }
        .insert(db::db())
        .await?;

        return Ok(shown(saved));
    }

    let row = meeting_link::Model {
        id: provisioned.id,
        title,
        created_at: now,
        revoked_at: None,
    };
    memory().lock().unwrap().push(row.clone());
    Ok(shown(row))
}

/// Every link this account still owns, newest first.
pub async fn list() -> crate::rahti::Result<Vec<MeetingLink>> {
    if db::is_connected() {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

        let rows = meeting_link::Entity::find()
            .filter(meeting_link::Column::RevokedAt.is_null())
            .order_by_desc(meeting_link::Column::CreatedAt)
            .all(db::db())
            .await?;

        return Ok(rows.into_iter().map(shown).collect());
    }

    let mut rows: Vec<_> = memory()
        .lock()
        .unwrap()
        .iter()
        .filter(|row| row.revoked_at.is_none())
        .cloned()
        .collect();
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(rows.into_iter().map(shown).collect())
}

/// The link behind a join code, if it exists and still works.
///
/// A revoked link reads the same as one that never existed. Telling a
/// stranger which of the two they found is a fact about this account they
/// have no claim on.
pub async fn find(id: &str) -> crate::rahti::Result<Option<MeetingLink>> {
    if db::is_connected() {
        use sea_orm::EntityTrait;

        let row = meeting_link::Entity::find_by_id(id.to_string())
            .one(db::db())
            .await?;

        return Ok(row.filter(|row| row.revoked_at.is_none()).map(shown));
    }

    Ok(memory()
        .lock()
        .unwrap()
        .iter()
        .find(|row| row.id == id && row.revoked_at.is_none())
        .cloned()
        .map(shown))
}

/// Revoke a link. `false` when there was nothing to revoke.
pub async fn revoke(id: &str) -> crate::rahti::Result<bool> {
    let now = Utc::now();

    if db::is_connected() {
        use sea_orm::ActiveValue::Set;
        use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

        let found = meeting_link::Entity::find_by_id(id.to_string())
            .one(db::db())
            .await?
            .filter(|row| row.revoked_at.is_none());

        let Some(row) = found else {
            return Ok(false);
        };

        let mut active = row.into_active_model();
        active.revoked_at = Set(Some(now));
        active.update(db::db()).await?;
        return Ok(true);
    }

    let mut rows = memory().lock().unwrap();
    let found = rows
        .iter_mut()
        .find(|row| row.id == id && row.revoked_at.is_none());

    let Some(row) = found else {
        return Ok(false);
    };
    row.revoked_at = Some(now);
    Ok(true)
}

/// A stored row, as the browser is shown it.
fn shown(row: meeting_link::Model) -> MeetingLink {
    MeetingLink {
        url: format!("/meet/j/{}", row.id),
        created: created_label(row.created_at),
        id: row.id,
        title: row.title,
        created_at: row.created_at,
    }
}

/// How long ago, in the words the card uses.
///
/// Read in the application calendar once the answer stops being relative: an
/// instant only has a date after a timezone is chosen, and the organizer's is
/// `APP_TIMEZONE`.
fn created_label(created_at: DateTime<Utc>) -> String {
    let seconds = (Utc::now() - created_at).num_seconds().max(0);

    match seconds {
        0..60 => "just now".to_string(),
        60..120 => "a minute ago".to_string(),
        120..3600 => format!("{} minutes ago", seconds / 60),
        3600..7200 => "an hour ago".to_string(),
        7200..86400 => format!("{} hours ago", seconds / 3600),
        86400..172800 => "yesterday".to_string(),
        _ => time::to_app_time(created_at)
            .format("%b %-d, %Y")
            .to_string(),
    }
}

/// A join code: short enough to read over a call, wide enough that nobody
/// finds a meeting by trying.
///
/// Crockford-style base32, from the standard library's own randomness —
/// `RandomState` is seeded per instance from the operating system, which is
/// what keeps two codes minted in the same nanosecond apart.
fn mint_id() -> String {
    use std::hash::{BuildHasher, Hasher, RandomState};

    const ALPHABET: &[u8] = b"0123456789abcdefghjkmnpqrstvwxyz";

    let mut bits = 0u128;
    for round in 0..2u64 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(round);
        hasher.write_i64(Utc::now().timestamp_nanos_opt().unwrap_or_default());
        bits = (bits << 64) | u128::from(hasher.finish());
    }

    (0..16)
        .map(|position| ALPHABET[((bits >> (position * 5)) & 0x1f) as usize] as char)
        .collect()
}

/// Where links live when the process has no database.
fn memory() -> &'static Mutex<Vec<meeting_link::Model>> {
    static ROWS: OnceLock<Mutex<Vec<meeting_link::Model>>> = OnceLock::new();
    ROWS.get_or_init(|| Mutex::new(Vec::new()))
}
