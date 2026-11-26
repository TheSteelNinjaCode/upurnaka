//! Admission precedes provider credentials; organizer authority comes from the link owner.
use crate::{
    auth, db,
    models::{meeting_control as control, meeting_link, meeting_visit as visit},
    rahti::{Error, Result, Validator},
};
use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    sea_query::{Expr, OnConflict},
};
use serde::Serialize;
fn now() -> i64 {
    rahti::time::Timestamp::now().as_second()
}
pub(super) fn denied() -> Error {
    Error::new(StatusCode::FORBIDDEN, "Meeting access denied.")
}
async fn link(id: &str) -> Result<meeting_link::Model> {
    auth::ready()?;
    meeting_link::Entity::find_by_id(id)
        .one(db::db())
        .await?
        .filter(|row| row.revoked_at.is_none())
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Meeting no longer available."))
}
pub async fn is_host(id: &str) -> Result<bool> {
    let row = link(id).await?;
    if auth::current().is_none() {
        return Ok(false);
    }
    if let Some(owner) = row.owner_id {
        return Ok(auth::current().is_some_and(|user| owner == user.id));
    }
    // Links made before account ownership existed are moderated by the live database admin.
    match crate::users::require_admin().await {
        Ok(_) => Ok(true),
        Err(error)
            if error.status() == StatusCode::FORBIDDEN
                || error.status() == StatusCode::UNAUTHORIZED =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}
pub(super) async fn host(id: &str) -> Result<()> {
    let row = link(id).await?;
    if row.owner_id.is_none() {
        crate::users::require_admin().await?;
        return Ok(());
    }
    crate::rahti::authorization::authorize_resource(
        "meetings.manage",
        &row.owner_id.unwrap_or_default(),
    )?;
    Ok(())
}
pub(super) async fn mine(id: &str, ticket: &str) -> Result<visit::Model> {
    link(id).await?;
    let row = visit::Entity::find_by_id(ticket)
        .filter(visit::Column::MeetingId.eq(id))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .one(db::db())
        .await?
        .ok_or_else(denied)?;
    // Anonymous entries use a random bearer ticket, held only in this browser's
    // component state. Account entries remain bound to their signed-in owner.
    if !row.user_id.starts_with("guest:")
        && auth::current().is_none_or(|user| user.id != row.user_id)
    {
        return Err(denied());
    }
    Ok(row)
}
#[derive(Serialize)]
pub struct Admission {
    pub id: String,
    pub state: String,
    pub host: bool,
}
#[derive(Serialize)]
pub struct Waiting {
    pub id: String,
    pub name: String,
}
#[derive(Serialize)]
pub struct RoomState {
    pub state: String,
    pub waiting: Vec<Waiting>,
    pub spotlight: Option<String>,
    pub muted_by_host: bool,
    /// Whether the organizer is recording, so every browser can say so.
    pub recording: bool,
}

pub async fn request(id: String, name: String) -> Result<Admission> {
    let mut v = Validator::new();
    v.required("id", &id)
        .max_chars("id", &id, 64)
        .required("name", &name)
        .max_chars("name", &name, 64);
    v.finish()?;
    let row = link(&id).await?;
    let user_id = auth::current().map(|user| user.id);
    let host = if let Some(owner) = row.owner_id {
        user_id.as_deref() == Some(owner.as_str())
    } else {
        is_host(&id).await?
    };
    visit::Entity::delete_many()
        .filter(visit::Column::LeaseUntil.lt(now() - 86_400))
        .exec(db::db())
        .await?;
    if visit::Entity::find()
        .filter(visit::Column::MeetingId.eq(&id))
        .filter(visit::Column::UserId.eq(user_id.as_deref().unwrap_or("")))
        .filter(visit::Column::State.is_in(["waiting", "admitted", "issuing", "joined"]))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .count(db::db())
        .await?
        >= 8
    {
        return Err(Error::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many open meeting requests. Close an older tab and try again.",
        ));
    }
    // Also bound anonymous requests per meeting; a guest cannot evade this
    // bound by creating a new identity for every request.
    if !host
        && visit::Entity::find()
            .filter(visit::Column::MeetingId.eq(&id))
            .filter(visit::Column::State.eq("waiting"))
            .filter(visit::Column::LeaseUntil.gt(now()))
            .count(db::db())
            .await?
            >= 100
    {
        return Err(Error::new(
            StatusCode::TOO_MANY_REQUESTS,
            "The lobby is full. Try again shortly.",
        ));
    }
    let state = if host { "admitted" } else { "waiting" };
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use rand::{TryRng, rngs::SysRng};
    let mut random = [0; 32];
    SysRng.try_fill_bytes(&mut random).map_err(|_| {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not create meeting entry",
        )
    })?;
    let ticket = URL_SAFE_NO_PAD.encode(random);
    visit::ActiveModel {
        id: Set(ticket.clone()),
        meeting_id: Set(id),
        user_id: Set(user_id.unwrap_or_else(|| format!("guest:{ticket}"))),
        name: Set(name.trim().into()),
        state: Set(state.into()),
        lease_until: Set(now() + 90),
        attendee_id: Set(None),
        provider_meeting_id: Set(None),
        muted_by_host: Set(false),
    }
    .insert(db::db())
    .await?;
    Ok(Admission {
        id: ticket,
        state: state.into(),
        host,
    })
}
pub async fn snapshot(id: String, ticket: String) -> Result<RoomState> {
    let row = mine(&id, &ticket).await?;
    if ["waiting", "admitted", "joined", "issuing"].contains(&row.state.as_str()) {
        visit::Entity::update_many()
            .col_expr(visit::Column::LeaseUntil, Expr::value(now() + 90))
            .filter(visit::Column::Id.eq(&ticket))
            .exec(db::db())
            .await?;
    }
    let waiting = if is_host(&id).await? {
        visit::Entity::find()
            .filter(visit::Column::MeetingId.eq(&id))
            .filter(visit::Column::State.eq("waiting"))
            .filter(visit::Column::LeaseUntil.gt(now()))
            .limit(50)
            .all(db::db())
            .await?
            .into_iter()
            .map(|v| Waiting {
                id: v.id,
                name: v.name,
            })
            .collect()
    } else {
        Vec::new()
    };
    let spotlight = control::Entity::find_by_id(&id)
        .one(db::db())
        .await?
        .and_then(|r| r.spotlight);
    // An ended participant must not leave an empty spotlight on everybody's screen.
    let spotlight = if let Some(target) = spotlight {
        if visit::Entity::find()
            .filter(visit::Column::MeetingId.eq(&id))
            .filter(visit::Column::AttendeeId.eq(&target))
            .filter(visit::Column::State.eq("joined"))
            .filter(visit::Column::LeaseUntil.gt(now()))
            .one(db::db())
            .await?
            .is_some()
        {
            Some(target)
        } else {
            None
        }
    } else {
        None
    };
    Ok(RoomState {
        state: row.state,
        waiting,
        spotlight,
        muted_by_host: row.muted_by_host,
        recording: super::recording::active(&id).await?,
    })
}
pub async fn decide(id: String, ticket: String, admit: bool) -> Result<bool> {
    host(&id).await?;
    let user = auth::current().ok_or_else(denied)?;
    if visit::Entity::find()
        .filter(visit::Column::MeetingId.eq(&id))
        .filter(visit::Column::UserId.eq(user.id))
        .filter(visit::Column::State.eq("joined"))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .one(db::db())
        .await?
        .is_none()
    {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "Join the meeting before admitting guests.",
        ));
    }
    let changed = visit::Entity::update_many()
        .col_expr(
            visit::Column::State,
            Expr::value(if admit { "admitted" } else { "denied" }),
        )
        .filter(visit::Column::Id.eq(ticket))
        .filter(visit::Column::MeetingId.eq(id))
        .filter(visit::Column::State.eq("waiting"))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .exec(db::db())
        .await?;
    if changed.rows_affected == 0 {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This lobby request is no longer waiting.",
        ));
    }
    Ok(true)
}
pub async fn finish(id: String, ticket: String) -> Result<bool> {
    let row = mine(&id, &ticket).await?;
    visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("left"))
        .filter(visit::Column::Id.eq(&row.id))
        .exec(db::db())
        .await?;
    Ok(true)
}
pub async fn claim(id: &str, ticket: &str) -> Result<visit::Model> {
    let row = mine(id, ticket).await?;
    let changed = visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("issuing"))
        .filter(visit::Column::Id.eq(ticket))
        .filter(visit::Column::State.eq("admitted"))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .exec(db::db())
        .await?;
    if changed.rows_affected != 1 {
        return Err(denied());
    }
    Ok(row)
}
pub async fn record(ticket: &str, provider: &str, attendee: &str) -> Result<()> {
    let changed = visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("joined"))
        .col_expr(visit::Column::ProviderMeetingId, Expr::value(provider))
        .col_expr(visit::Column::AttendeeId, Expr::value(attendee))
        .filter(visit::Column::Id.eq(ticket))
        .filter(visit::Column::State.eq("issuing"))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .exec(db::db())
        .await?;
    if changed.rows_affected != 1 {
        return Err(denied());
    }
    Ok(())
}
pub async fn release(ticket: &str) {
    let _ = visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("admitted"))
        .filter(visit::Column::Id.eq(ticket))
        .filter(visit::Column::State.eq("issuing"))
        .exec(db::db())
        .await;
}
pub async fn spotlight(id: String, attendee: Option<String>) -> Result<bool> {
    host(&id).await?;
    if let Some(target) = &attendee {
        target_visit(&id, target).await?;
    }
    control::Entity::insert(control::ActiveModel {
        meeting_id: Set(id),
        spotlight: Set(attendee),
    })
    .on_conflict(
        OnConflict::column(control::Column::MeetingId)
            .update_column(control::Column::Spotlight)
            .to_owned(),
    )
    .exec(db::db())
    .await?;
    Ok(true)
}
async fn target_visit(id: &str, attendee: &str) -> Result<visit::Model> {
    visit::Entity::find()
        .filter(visit::Column::MeetingId.eq(id))
        .filter(visit::Column::AttendeeId.eq(attendee))
        .filter(visit::Column::State.eq("joined"))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .one(db::db())
        .await?
        .ok_or_else(denied)
}
pub async fn mute(id: String, attendee: String) -> Result<bool> {
    host(&id).await?;
    let row = target_visit(&id, &attendee).await?;
    super::dilnaka_meet::set_attendee_audio(
        row.provider_meeting_id.as_deref().ok_or_else(denied)?,
        &attendee,
        false,
    )
    .await?;
    visit::Entity::update_many()
        .col_expr(visit::Column::MutedByHost, Expr::value(true))
        .filter(visit::Column::Id.eq(row.id))
        .exec(db::db())
        .await?;
    Ok(true)
}
/// Mute everybody in the meeting except the organizer asking. Each person is
/// muted at the provider, so a client that ignores the request still cannot
/// send audio, and told through the room state so their own controls agree.
/// Returns how many people were muted.
pub async fn mute_all(id: String) -> Result<u64> {
    host(&id).await?;
    let me = auth::current().ok_or_else(denied)?;
    let targets = visit::Entity::find()
        .filter(visit::Column::MeetingId.eq(&id))
        .filter(visit::Column::State.eq("joined"))
        .filter(visit::Column::LeaseUntil.gt(now()))
        .filter(visit::Column::UserId.ne(me.id))
        .all(db::db())
        .await?;
    let mut muted = Vec::new();
    for row in targets {
        let (Some(provider), Some(attendee)) = (
            row.provider_meeting_id.as_deref(),
            row.attendee_id.as_deref(),
        ) else {
            continue;
        };
        super::dilnaka_meet::set_attendee_audio(provider, attendee, false).await?;
        muted.push(row.id);
    }
    if !muted.is_empty() {
        visit::Entity::update_many()
            .col_expr(visit::Column::MutedByHost, Expr::value(true))
            .filter(visit::Column::Id.is_in(muted.clone()))
            .exec(db::db())
            .await?;
    }
    Ok(muted.len() as u64)
}
pub async fn resume_audio(id: String, ticket: String) -> Result<bool> {
    let row = mine(&id, &ticket).await?;
    if row.state != "joined" {
        return Err(denied());
    }
    if row.muted_by_host {
        super::dilnaka_meet::set_attendee_audio(
            row.provider_meeting_id.as_deref().ok_or_else(denied)?,
            row.attendee_id.as_deref().ok_or_else(denied)?,
            true,
        )
        .await?;
        visit::Entity::update_many()
            .col_expr(visit::Column::MutedByHost, Expr::value(false))
            .filter(visit::Column::Id.eq(row.id))
            .exec(db::db())
            .await?;
    }
    Ok(true)
}
