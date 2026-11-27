//! Activity notifications: the meeting events a person should hear about.
//!
//! Each event is written here by the code that made it happen, after its own
//! write has committed: being invited to an invite-only meeting, being on a
//! scheduled meeting's attendee list, someone waiting in the lobby while the
//! organizer is away, an `@mention` in a meeting chat, and a recording that
//! was stored or failed. [`notify`] never fails its caller: a notification
//! that cannot be written is logged and dropped, and the invite, message or
//! recording it describes stands.
//!
//! The Activity page lists them and the rail shows how many are unread; both
//! read through the signed-in account only.
use crate::{
    db,
    models::notification as row,
    rahti::{Error, Result, Validator},
};
use axum::http::StatusCode;
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, sea_query::Expr,
};
use serde::Serialize;

/// The most the Activity page lists.
const FEED: u64 = 100;
/// The most one event tells, so a long invite list is one bounded insert.
const RECIPIENTS: usize = 300;

pub const INVITE: &str = "invite";
pub const SCHEDULED: &str = "scheduled";
pub const LOBBY: &str = "lobby";
pub const MENTION: &str = "mention";
pub const RECORDING: &str = "recording";

fn now() -> i64 {
    rahti::time::Timestamp::now().as_millisecond()
}

/// What happened, before it is addressed to anyone.
pub struct Event<'a> {
    pub kind: &'a str,
    pub meeting_id: &'a str,
    pub actor: &'a str,
    pub title: String,
    pub body: String,
    pub link: String,
}

/// Tell each of `users` about `event`. Duplicates and empty ids are skipped.
pub async fn notify(users: impl IntoIterator<Item = String>, event: Event<'_>) {
    let mut seen = std::collections::HashSet::new();
    let users: Vec<String> = users
        .into_iter()
        .filter(|user| !user.is_empty() && !user.starts_with("guest:"))
        .filter(|user| seen.insert(user.clone()))
        .take(RECIPIENTS)
        .collect();
    if users.is_empty() {
        return;
    }
    let at = now();
    let title: String = event.title.chars().take(200).collect();
    let body: String = event.body.chars().take(500).collect();
    let rows = users.into_iter().map(|user| row::ActiveModel {
        id: sea_orm::ActiveValue::Set(crate::meetings::mint_id()),
        user_id: sea_orm::ActiveValue::Set(user),
        kind: sea_orm::ActiveValue::Set(event.kind.into()),
        meeting_id: sea_orm::ActiveValue::Set(event.meeting_id.into()),
        actor: sea_orm::ActiveValue::Set(event.actor.chars().take(80).collect()),
        title: sea_orm::ActiveValue::Set(title.clone()),
        body: sea_orm::ActiveValue::Set(body.clone()),
        link: sea_orm::ActiveValue::Set(event.link.clone()),
        created_at: sea_orm::ActiveValue::Set(at),
        read_at: sea_orm::ActiveValue::Set(None),
    });
    if let Err(error) = row::Entity::insert_many(rows).exec(db::db()).await {
        eprintln!("notifications: {} was not recorded: {error}", event.kind);
    }
}

/// Whether `user` already has an unread `kind` notification for a meeting,
/// so a repeating event (a lobby filling up) is told once until it is seen.
pub async fn has_unread(user: &str, kind: &str, meeting_id: &str) -> bool {
    row::Entity::find()
        .filter(row::Column::UserId.eq(user))
        .filter(row::Column::Kind.eq(kind))
        .filter(row::Column::MeetingId.eq(meeting_id))
        .filter(row::Column::ReadAt.is_null())
        .count(db::db())
        .await
        .map(|count| count > 0)
        // Unsure, so say nothing rather than repeat.
        .unwrap_or(true)
}

fn reader() -> Result<String> {
    crate::auth::ready()?;
    crate::auth::current()
        .map(|user| user.id)
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Authentication required"))
}

/// One notification, as the Activity page shows it.
#[derive(Clone, Serialize)]
pub struct Item {
    pub id: String,
    pub kind: String,
    pub actor: String,
    pub title: String,
    pub body: String,
    pub link: String,
    /// Milliseconds since the epoch, UTC.
    pub at: i64,
    pub read: bool,
}

impl From<row::Model> for Item {
    fn from(row: row::Model) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            actor: row.actor,
            title: row.title,
            body: row.body,
            link: row.link,
            at: row.created_at,
            read: row.read_at.is_some(),
        }
    }
}

/// The signed-in person's newest notifications.
pub async fn feed() -> Result<Vec<Item>> {
    let user = reader()?;
    let rows = row::Entity::find()
        .filter(row::Column::UserId.eq(user))
        .order_by_desc(row::Column::CreatedAt)
        .order_by_desc(row::Column::Id)
        .limit(FEED)
        .all(db::db())
        .await?;
    Ok(rows.into_iter().map(Item::from).collect())
}

/// How many of the signed-in person's notifications are unread.
pub async fn unread() -> Result<u64> {
    let user = reader()?;
    Ok(row::Entity::find()
        .filter(row::Column::UserId.eq(user))
        .filter(row::Column::ReadAt.is_null())
        .count(db::db())
        .await?)
}

/// Mark one of the signed-in person's notifications read. Someone else's id
/// is not found, the same as an id that never existed.
pub async fn mark_read(id: String) -> Result<u64> {
    let mut validator = Validator::new();
    validator.required("id", &id).max_chars("id", &id, 64);
    validator.finish()?;
    let user = reader()?;
    let found = row::Entity::find_by_id(&id)
        .filter(row::Column::UserId.eq(&user))
        .count(db::db())
        .await?;
    if found == 0 {
        return Err(Error::new(StatusCode::NOT_FOUND, "Notification not found."));
    }
    row::Entity::update_many()
        .col_expr(row::Column::ReadAt, Expr::value(now()))
        .filter(row::Column::Id.eq(&id))
        .filter(row::Column::UserId.eq(&user))
        .filter(row::Column::ReadAt.is_null())
        .exec(db::db())
        .await?;
    unread().await
}

/// Mark every one of the signed-in person's notifications read.
pub async fn mark_all_read() -> Result<u64> {
    let user = reader()?;
    row::Entity::update_many()
        .col_expr(row::Column::ReadAt, Expr::value(now()))
        .filter(row::Column::UserId.eq(&user))
        .filter(row::Column::ReadAt.is_null())
        .exec(db::db())
        .await?;
    Ok(0)
}
