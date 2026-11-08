//! Meeting links: where they are kept, and the seam a real meeting provider
//! plugs into.
//!
//! Meet does three things with a link — mint one, list the ones this account
//! owns, and resolve one somebody followed — and all three are here rather
//! than in `page.rs`, because two pages need them: `/meet` lists and mints,
//! `/meet/j/{id}` resolves.
//!
//! ## Two lifetimes
//!
//! A meeting *link* and a *conference* are not the same thing, and keeping
//! them apart is most of what this module does.
//!
//! The link is this application's: a join code, a title, a row, reusable for
//! as long as the organizer wants it. The conference is Dilnaka Meet's (see
//! [`dilnaka_meet`]): it exists while people are in it and Dilnaka ends it
//! after the last one leaves. So a link spends most of its life
//! with no conference behind it, and [`join`] is where one is found or made.
//! `provider_meeting_id` on the row is a cache of which conference is live,
//! never the authority for whether the link is.
//!
//! With no `DILNAKA_MEETINGS_KEY` configured there is no provider, and a link
//! is still a link: it mints, lists, resolves and revokes, and the lobby says
//! so rather than pretending.
//!
//! ## Where rows live
//!
//! The database, when the process has one. A deployment with no
//! `DATABASE_URL` keeps them in memory for the life of the process instead,
//! which is enough to develop against — a browser reload still shows the list
//! — and is deliberately not enough to ship on: a restart loses them. The
//! moment a connection string is configured and `db::migrate()` has run, the
//! same calls write rows.

pub mod chat;
pub mod dilnaka_meet;
pub mod lobby;
pub mod recording;

use std::sync::{Mutex, OnceLock};

use serde::Serialize;

use crate::db;
use crate::models::meeting_link;
use crate::rahti::time::{self, DateTime, Utc};
use crate::rahti::{Error, Result};

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

/// Everything one browser needs to connect to a conference.
///
/// The two fields are `amazon-chime-sdk-js`'s own arguments, in its own
/// shape: `new MeetingSessionConfiguration(meeting, attendee)`.
#[derive(Debug, Serialize)]
pub struct JoinInfo {
    pub meeting: dilnaka_meet::MeetingResponse,
    pub attendee: dilnaka_meet::AttendeeResponse,
}

/// Mint a link and keep it.
///
/// No conference yet. Dilnaka Meet bills and reaps by conference, and a link that
/// nobody has opened should cost nothing and expire from nothing — so the
/// provider is not called until somebody actually arrives.
pub async fn create(title: String) -> Result<MeetingLink> {
    let owner = crate::auth::owner_id()?;
    let now = Utc::now();

    // The identifier is the server's to hand out. A browser that picked its
    // own could collide with somebody else's meeting, and a guessable one is
    // a meeting strangers can walk into.
    let id = mint_id();

    if db::is_connected() {
        use sea_orm::ActiveModelTrait;
        use sea_orm::ActiveValue::Set;

        let saved = meeting_link::ActiveModel {
            id: Set(id),
            owner_id: Set(Some(owner)),
            title: Set(title),
            created_at: Set(now),
            revoked_at: Set(None),
            provider_meeting_id: Set(None),
            media_region: Set(None),
        }
        .insert(db::db())
        .await?;

        return Ok(shown(saved));
    }

    let row = meeting_link::Model {
        id,
        owner_id: Some(owner),
        title,
        created_at: now,
        revoked_at: None,
        provider_meeting_id: None,
        media_region: None,
    };
    memory().lock().unwrap().push(row.clone());
    Ok(shown(row))
}

/// Every link this account still owns, newest first.
pub async fn list() -> Result<Vec<MeetingLink>> {
    let owner = crate::auth::owner_id()?;
    if db::is_connected() {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

        let rows = meeting_link::Entity::find()
            .filter(meeting_link::Column::OwnerId.eq(&owner))
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
        .filter(|row| row.revoked_at.is_none() && row.owner_id.as_deref() == Some(owner.as_str()))
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
pub async fn find(id: &str) -> Result<Option<MeetingLink>> {
    Ok(row(id).await?.map(shown))
}

/// Rename a live link without changing its URL, creation time, or conference.
pub async fn rename(id: &str, title: String) -> Result<MeetingLink> {
    let unavailable = || {
        Error::public(
            axum::http::StatusCode::NOT_FOUND,
            "That meeting link is no longer available.",
        )
    };
    let existing = row(id).await?.ok_or_else(unavailable)?;
    crate::auth::authorize_meeting(existing.owner_id.as_deref())?;
    if db::is_connected() {
        use sea_orm::ActiveValue::Set;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let updated = meeting_link::Entity::update_many()
            .set(meeting_link::ActiveModel {
                title: Set(title),
                ..Default::default()
            })
            .filter(meeting_link::Column::Id.eq(id))
            .filter(meeting_link::Column::RevokedAt.is_null())
            .exec(db::db())
            .await?;
        if updated.rows_affected == 0 {
            return Err(unavailable());
        }
        return find(id).await?.ok_or_else(unavailable);
    }
    let mut rows = memory().lock().unwrap();
    let row = rows
        .iter_mut()
        .find(|row| row.id == id && row.revoked_at.is_none())
        .ok_or_else(unavailable)?;
    row.title = title;
    Ok(shown(row.clone()))
}

/// The stored row behind a live join code.
async fn row(id: &str) -> Result<Option<meeting_link::Model>> {
    if db::is_connected() {
        use sea_orm::EntityTrait;

        let found = meeting_link::Entity::find_by_id(id.to_string())
            .one(db::db())
            .await?;

        return Ok(found.filter(|row| row.revoked_at.is_none()));
    }

    Ok(memory()
        .lock()
        .unwrap()
        .iter()
        .find(|row| row.id == id && row.revoked_at.is_none())
        .cloned())
}

/// Connect somebody to the conference behind a link, making one if there is
/// none live.
///
/// This is the only place the provider is reached for a join, and the only
/// place a join token is minted. The token is this caller's alone: it is
/// returned to the browser that asked and is never stored, logged or reused.
pub async fn join(id: &str, display_name: &str) -> Result<JoinInfo> {
    let Some(row) = row(id).await? else {
        return Err(Error::public(
            axum::http::StatusCode::NOT_FOUND,
            "That meeting link is no longer available. Ask the organizer for a new one.",
        ));
    };

    let meeting = conference_for(&row).await?;
    let attendee =
        dilnaka_meet::create_attendee(&meeting.meeting_id, &attendee_id(display_name)).await?;

    Ok(JoinInfo { meeting, attendee })
}

/// The live conference behind a link: the one it remembers, or a new one.
///
/// Three outcomes, and the middle one is the common one. The row names a
/// conference and Dilnaka still has it — reuse. The row names one that has
/// ended, or names none at all — create, and remember which. Both
/// writes are best-effort against the row, because the conference is already
/// real by then and failing the join over a cache update would be worse than
/// looking it up again next time.
async fn conference_for(row: &meeting_link::Model) -> Result<dilnaka_meet::MeetingResponse> {
    if let Some(existing) = &row.provider_meeting_id {
        if let Some(meeting) = dilnaka_meet::meeting_or_none(existing).await? {
            return Ok(meeting);
        }
    }

    let meeting = dilnaka_meet::create_meeting(&row.id).await?;
    remember_conference(
        &row.id,
        Some(&meeting.meeting_id),
        Some(&meeting.media_region),
    )
    .await;
    Ok(meeting)
}

/// Record which conference a link is currently pointing at.
///
/// Deliberately infallible from the caller's side: this is a cache of the
/// provider's state, and the provider is the authority. A failed write costs
/// one extra `GetMeeting` later.
async fn remember_conference(id: &str, meeting_id: Option<&str>, media_region: Option<&str>) {
    if db::is_connected() {
        use sea_orm::ActiveModelTrait;
        use sea_orm::ActiveValue::{Set, Unchanged};

        let update = meeting_link::ActiveModel {
            id: Unchanged(id.to_string()),
            provider_meeting_id: Set(meeting_id.map(str::to_string)),
            media_region: Set(media_region.map(str::to_string)),
            ..Default::default()
        }
        .update(db::db())
        .await;

        if let Err(error) = update {
            eprintln!("meetings: could not record the conference for {id}: {error}");
        }
        return;
    }

    if let Some(row) = memory().lock().unwrap().iter_mut().find(|row| row.id == id) {
        row.provider_meeting_id = meeting_id.map(str::to_string);
        row.media_region = media_region.map(str::to_string);
    }
}

/// Revoke a link. `false` when there was nothing to revoke.
///
/// Ends the conference as well as the link. Somebody who was handed this URL
/// is in the meeting *because of it*, so taking the link away and leaving
/// them connected would revoke nothing that matters.
pub async fn revoke(id: &str) -> Result<bool> {
    let now = Utc::now();

    let Some(row) = row(id).await? else {
        return Ok(false);
    };
    crate::auth::authorize_meeting(row.owner_id.as_deref())?;

    if let Some(meeting_id) = &row.provider_meeting_id {
        if dilnaka_meet::is_configured() {
            dilnaka_meet::delete_meeting(meeting_id).await?;
        }
    }

    if db::is_connected() {
        use sea_orm::ActiveModelTrait;
        use sea_orm::ActiveValue::{Set, Unchanged};

        meeting_link::ActiveModel {
            id: Unchanged(row.id),
            revoked_at: Set(Some(now)),
            provider_meeting_id: Set(None),
            ..Default::default()
        }
        .update(db::db())
        .await?;
        chat::clear(id).await?;

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
    row.provider_meeting_id = None;
    Ok(true)
}

/// A Dilnaka Meet `ExternalUserId` for one person arriving at one meeting.
///
/// Dilnaka Meet allows 2–64 characters and treats this as opaque, so it carries the
/// display name for the benefit of anyone reading an attendee list, plus a
/// nonce: two people called "Jefferson Abraham" in one meeting are two
/// attendees, and the same person in two tabs is as well.
fn attendee_id(display_name: &str) -> String {
    let name: String = display_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .take(40)
        .collect();
    let name = name.trim();
    let name = if name.is_empty() { "Guest" } else { name };

    format!("{name}#{}", &mint_id()[..8])
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
pub(crate) fn mint_id() -> String {
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
