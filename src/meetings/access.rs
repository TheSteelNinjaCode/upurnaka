//! Who may enter a meeting.
//!
//! Three modes, chosen per link by its organizer:
//!
//! - **open** — anyone holding the link asks to join and waits in the lobby.
//!   Every link made before this module existed is open.
//! - **passcode** — the link alone is not enough: the person also types the
//!   meeting's passcode (or follows a link that carries it as `?p=`), and then
//!   waits in the lobby. Guests are allowed.
//! - **invited** — only signed-in accounts on the organizer's list get in, and
//!   they skip the lobby. Everybody else is refused, link or no link.
//!
//! The organizer (or, for a link with no owner, the administrator) is never
//! subject to any of this. Every check runs on the server when a lobby ticket
//! is issued ([`admit`]); the page only uses [`page_access`] to decide what to
//! show first.
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use axum::http::StatusCode;
use chrono::Utc;
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, Condition, EntityTrait, ExprTrait, QueryFilter, QuerySelect, TransactionTrait,
    sea_query::{Expr, Func, LikeExpr},
};
use serde::Serialize;

use crate::{
    auth, db,
    models::{account, calendar_event, meeting_invite as invite, meeting_link, meeting_visit},
    rahti::{Error, Result, Validator},
};

pub const OPEN: &str = "open";
pub const PASSCODE: &str = "passcode";
pub const INVITED: &str = "invited";

/// The most people one meeting's invite list may hold.
const MAX_INVITES: usize = 300;
/// Wrong passcodes one meeting accepts in [`WINDOW`] seconds before it stops
/// checking for a while. Generated codes have 40 bits; this keeps guessing
/// hopeless without locking out someone who mistypes a few times.
const MAX_FAILURES: u32 = 20;
const WINDOW: i64 = 600;

/// One person on an invite list, as the organizer's picker shows them.
#[derive(Clone, Debug, Serialize)]
pub struct Invitee {
    pub id: String,
    pub name: String,
}

/// A meeting's access settings, as its organizer edits them.
#[derive(Debug, Serialize)]
pub struct AccessSettings {
    pub id: String,
    pub mode: String,
    /// Only ever sent to the organizer.
    pub passcode: Option<String>,
    pub invited: Vec<Invitee>,
    /// True when `invited` was suggested from the calendar event's attendees
    /// rather than saved: the organizer still has to save it.
    pub suggested: bool,
}

/// What the join page needs to know before it draws anything.
#[derive(Debug)]
pub struct PageAccess {
    pub mode: String,
    pub host: bool,
    /// For an invited meeting: whether the signed-in viewer is on the list.
    pub invited: bool,
}

fn now() -> i64 {
    rahti::time::Timestamp::now().as_second()
}

async fn live(id: &str) -> Result<meeting_link::Model> {
    auth::ready()?;
    meeting_link::Entity::find_by_id(id)
        .one(db::db())
        .await?
        .filter(|row| row.revoked_at.is_none())
        .ok_or_else(|| {
            Error::public(
                StatusCode::NOT_FOUND,
                "That meeting link is no longer available.",
            )
        })
}

async fn is_invited(meeting: &str, user: &str) -> Result<bool> {
    Ok(
        invite::Entity::find_by_id((meeting.to_string(), user.to_string()))
            .one(db::db())
            .await?
            .is_some(),
    )
}

/// Lower-cased and trimmed: a passcode read aloud should not fail on case.
fn normalize(code: &str) -> String {
    code.trim().to_lowercase()
}

fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn failures() -> &'static Mutex<HashMap<String, (i64, u32)>> {
    static FAILURES: OnceLock<Mutex<HashMap<String, (i64, u32)>>> = OnceLock::new();
    FAILURES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn too_many() -> Error {
    Error::public(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many wrong passcodes for this meeting. Wait a few minutes and try again.",
    )
}

/// Check a passcode against a meeting, counting the wrong ones.
fn verify_passcode(row: &meeting_link::Model, given: Option<&str>) -> Result<()> {
    let at = now();
    {
        let map = failures().lock().unwrap();
        if let Some((start, count)) = map.get(&row.id) {
            if at - start < WINDOW && *count >= MAX_FAILURES {
                return Err(too_many());
            }
        }
    }
    let expected = row.passcode.as_deref().map(normalize).unwrap_or_default();
    let given = given.map(normalize).unwrap_or_default();
    if given.is_empty() {
        return Err(Error::public(
            StatusCode::FORBIDDEN,
            "Enter the meeting passcode to join.",
        ));
    }
    if !expected.is_empty() && same(&expected, &given) {
        return Ok(());
    }
    let mut map = failures().lock().unwrap();
    if map.len() > 10_000 {
        map.retain(|_, (start, _)| at - *start < WINDOW);
    }
    let entry = map.entry(row.id.clone()).or_insert((at, 0));
    if at - entry.0 >= WINDOW {
        *entry = (at, 0);
    }
    entry.1 += 1;
    Err(Error::public(
        StatusCode::FORBIDDEN,
        "That passcode isn't right. Check it with the organizer.",
    ))
}

/// Decide whether this caller may ask to enter `row`. `Ok(true)` means they
/// skip the lobby; `Ok(false)` that they wait in it.
pub(super) async fn admit(
    row: &meeting_link::Model,
    host: bool,
    passcode: Option<&str>,
) -> Result<bool> {
    if host {
        return Ok(true);
    }
    match row.access.as_str() {
        PASSCODE => {
            verify_passcode(row, passcode)?;
            Ok(false)
        }
        INVITED => {
            let Some(user) = auth::current() else {
                return Err(Error::public(
                    StatusCode::UNAUTHORIZED,
                    "This meeting is for invited people only. Sign in to join.",
                ));
            };
            if !is_invited(&row.id, &user.id).await? {
                return Err(not_invited());
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn not_invited() -> Error {
    Error::public(
        StatusCode::FORBIDDEN,
        "You're not invited to this meeting. Ask the organizer to add you.",
    )
}

/// The passcode gate on the join page: `true` when the code is right.
pub async fn check_passcode(id: &str, passcode: &str) -> Result<bool> {
    let row = live(id).await?;
    if row.access != PASSCODE || super::lobby::is_host(id).await? {
        return Ok(true);
    }
    verify_passcode(&row, Some(passcode))?;
    Ok(true)
}

/// What the join page shows first for this viewer.
pub async fn page_access(id: &str) -> Result<PageAccess> {
    let row = live(id).await?;
    let host = super::lobby::is_host(id).await?;
    let invited = match auth::current() {
        Some(user) if row.access == INVITED => is_invited(id, &user.id).await?,
        _ => false,
    };
    Ok(PageAccess {
        mode: row.access,
        host,
        invited,
    })
}

async fn invitees(meeting: &str) -> Result<Vec<Invitee>> {
    let ids: Vec<String> = invite::Entity::find()
        .filter(invite::Column::MeetingId.eq(meeting))
        .all(db::db())
        .await?
        .into_iter()
        .map(|row| row.user_id)
        .collect();
    named(ids).await
}

async fn named(ids: Vec<String>) -> Result<Vec<Invitee>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut people: Vec<Invitee> = account::Entity::find()
        .filter(account::Column::Id.is_in(ids))
        .filter(account::Column::Active.eq(true))
        .all(db::db())
        .await?
        .into_iter()
        .map(|row| Invitee {
            id: row.id,
            name: row.name,
        })
        .collect();
    people.sort_by_key(|person| person.name.to_lowercase());
    Ok(people)
}

/// A calendar meeting's attendees that match an account, by email or by exact
/// name. Free-text attendees with no account cannot be invited.
async fn calendar_suggestions(meeting: &str, owner: Option<&str>) -> Result<Vec<Invitee>> {
    let Some(event) = calendar_event::Entity::find_by_id(meeting)
        .one(db::db())
        .await?
    else {
        return Ok(Vec::new());
    };
    let attendees: Vec<String> = rahti::serde_json::from_str(&event.attendees).unwrap_or_default();
    let wanted: HashSet<String> = attendees.iter().map(|a| a.trim().to_lowercase()).collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }
    let wanted: Vec<String> = wanted.into_iter().collect();
    let ids = account::Entity::find()
        .filter(account::Column::Active.eq(true))
        .filter(account::Column::Id.ne(owner.unwrap_or_default()))
        .filter(
            Condition::any()
                .add(
                    Expr::expr(Func::lower(Expr::col(account::Column::Email)))
                        .is_in(wanted.clone()),
                )
                .add(Expr::expr(Func::lower(Expr::col(account::Column::Name))).is_in(wanted)),
        )
        .limit(MAX_INVITES as u64)
        .all(db::db())
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    named(ids).await
}

/// The organizer's view of a meeting's access settings.
pub async fn settings(id: &str) -> Result<AccessSettings> {
    super::lobby::host(id).await?;
    let row = live(id).await?;
    let mut invited = invitees(id).await?;
    let mut suggested = false;
    if invited.is_empty() {
        invited = calendar_suggestions(id, row.owner_id.as_deref()).await?;
        suggested = !invited.is_empty();
    }
    Ok(AccessSettings {
        id: row.id,
        mode: row.access,
        passcode: row.passcode,
        invited,
        suggested,
    })
}

/// A fresh passcode: 8 characters with no look-alikes (no 0/o, 1/l/i).
pub fn mint_passcode() -> String {
    use rand::{TryRng, rngs::SysRng};
    const ALPHABET: &[u8] = b"23456789abcdefghjkmnpqrstuvwxyz";
    let mut bytes = [0u8; 8];
    SysRng
        .try_fill_bytes(&mut bytes)
        .expect("operating system randomness");
    bytes
        .iter()
        .map(|b| ALPHABET[*b as usize % ALPHABET.len()] as char)
        .collect()
}

/// Change who may enter a meeting.
///
/// `passcode` empty in passcode mode keeps the current code, or mints one if
/// there is none. Switching to invite-only, or taking someone off the list,
/// removes everyone in the meeting or its lobby who is no longer allowed.
pub async fn update(
    id: &str,
    mode: String,
    passcode: String,
    invited: Vec<String>,
) -> Result<AccessSettings> {
    let passcode = passcode.trim().to_string();
    let mut v = Validator::new();
    v.required("id", id)
        .max_chars("id", id, 64)
        .one_of("mode", &mode.as_str(), &[OPEN, PASSCODE, INVITED])
        .max_chars("passcode", &passcode, 32)
        .check(
            "passcode",
            passcode.is_empty()
                || (passcode.chars().count() >= 4
                    && passcode.chars().all(|c| c.is_ascii_alphanumeric())),
            "Use 4 to 32 letters and numbers.",
        )
        .check(
            "invited",
            invited.len() <= MAX_INVITES && invited.iter().all(|i| i.len() <= 100),
            "Invite up to 300 people.",
        );
    v.finish()?;
    super::lobby::host(id).await?;
    let row = live(id).await?;
    let owner = row.owner_id.clone();

    let wanted: HashSet<String> = invited
        .into_iter()
        .filter(|person| Some(person.as_str()) != owner.as_deref())
        .collect();
    let found = named(wanted.iter().cloned().collect()).await?;
    if found.len() != wanted.len() {
        let mut v = Validator::new();
        v.check(
            "invited",
            false,
            "Someone on this list no longer has an active account. Remove them and try again.",
        );
        v.finish()?;
    }

    let code = if mode == PASSCODE {
        Some(if !passcode.is_empty() {
            normalize(&passcode)
        } else {
            row.passcode.clone().unwrap_or_else(mint_passcode)
        })
    } else {
        // Kept, so switching back to passcode mode does not change the code
        // people were already sent.
        row.passcode.clone()
    };

    // Who could already get in, so only the people this change lets in are told.
    let already: HashSet<String> = if row.access == INVITED {
        invite::Entity::find()
            .filter(invite::Column::MeetingId.eq(id))
            .all(db::db())
            .await?
            .into_iter()
            .map(|row| row.user_id)
            .collect()
    } else {
        HashSet::new()
    };

    let transaction = db::db().begin().await?;
    meeting_link::Entity::update_many()
        .col_expr(meeting_link::Column::Access, Expr::value(mode.clone()))
        .col_expr(meeting_link::Column::Passcode, Expr::value(code))
        .filter(meeting_link::Column::Id.eq(id))
        .exec(&transaction)
        .await?;
    invite::Entity::delete_many()
        .filter(invite::Column::MeetingId.eq(id))
        .exec(&transaction)
        .await?;
    if !wanted.is_empty() {
        let created_at = Utc::now();
        invite::Entity::insert_many(wanted.iter().map(|user| invite::ActiveModel {
            meeting_id: Set(id.to_string()),
            user_id: Set(user.clone()),
            created_at: Set(created_at),
        }))
        .exec(&transaction)
        .await?;
    }
    transaction.commit().await?;
    failures().lock().unwrap().remove(id);

    if mode == INVITED {
        remove_uninvited(id, owner.as_deref(), &wanted).await?;
        let organizer = auth::current().map(|user| user.name).unwrap_or_default();
        crate::notifications::notify(
            wanted.difference(&already).cloned(),
            crate::notifications::Event {
                kind: crate::notifications::INVITE,
                meeting_id: id,
                actor: &organizer,
                title: format!("{organizer} invited you to {}", row.title),
                body: "Only invited people can join this meeting.".into(),
                link: format!("/meet/j/{id}"),
            },
        )
        .await;
    }
    settings(id).await
}

/// Take everybody who is not the organizer and not on `allowed` out of the
/// meeting and its lobby.
async fn remove_uninvited(id: &str, owner: Option<&str>, allowed: &HashSet<String>) -> Result<()> {
    let host = auth::current().map(|user| user.id);
    let visits = meeting_visit::Entity::find()
        .filter(meeting_visit::Column::MeetingId.eq(id))
        .filter(meeting_visit::Column::State.is_in(["waiting", "admitted", "issuing", "joined"]))
        .filter(meeting_visit::Column::LeaseUntil.gt(now()))
        .all(db::db())
        .await?;
    for visit in visits {
        if Some(visit.user_id.as_str()) == owner
            || Some(&visit.user_id) == host.as_ref()
            || allowed.contains(&visit.user_id)
        {
            continue;
        }
        meeting_visit::Entity::update_many()
            .col_expr(meeting_visit::Column::State, Expr::value("removed"))
            .filter(meeting_visit::Column::Id.eq(&visit.id))
            .exec(db::db())
            .await?;
        if let (Some(meeting), Some(attendee)) = (&visit.provider_meeting_id, &visit.attendee_id) {
            if super::dilnaka_meet::is_configured() {
                // Best effort: the visit is already refused here, and its
                // browser leaves on its next poll even if this fails.
                if let Err(error) = super::dilnaka_meet::delete_attendee(meeting, attendee).await {
                    eprintln!("meetings: could not remove attendee from {id}: {error:?}");
                }
            }
        }
    }
    Ok(())
}

/// People an organizer can add to an invite list: active, verified accounts
/// whose name contains `query`. Names only, as the directory search shows.
pub async fn search(query: &str) -> Result<Vec<Invitee>> {
    auth::ready()?;
    crate::rahti::authorization::authorize("directory.search")?;
    let me = auth::current().map(|user| user.id).unwrap_or_default();
    let query = query.trim().to_lowercase();
    let pattern = format!(
        "%{}%",
        query
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_")
    );
    let mut people: Vec<Invitee> = account::Entity::find()
        .filter(account::Column::Active.eq(true))
        .filter(account::Column::Verified.eq(true))
        .filter(account::Column::Id.ne(me))
        .filter(
            Expr::expr(Func::lower(Expr::col(account::Column::Name)))
                .like(LikeExpr::new(&pattern).escape('!')),
        )
        .limit(20)
        .all(db::db())
        .await?
        .into_iter()
        .map(|row| Invitee {
            id: row.id,
            name: row.name,
        })
        .collect();
    people.sort_by_key(|person| person.name.to_lowercase());
    people.truncate(20);
    Ok(people)
}
