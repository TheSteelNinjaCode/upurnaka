//! Private calendar events. Wall-clock inputs are resolved once and stored in UTC.
use crate::{
    auth, db,
    models::{calendar_event as event, meeting_link},
    rahti::{self, Error, Result, Validator},
};
use chrono::{DateTime, Duration, Months, NaiveDate, NaiveDateTime, Timelike, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ScheduleInput {
    pub title: String,
    pub attendees: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub timezone: String,
    pub all_day: bool,
    pub recurrence: String,
    pub occurrences: u32,
    pub location: String,
    pub details: String,
}

#[derive(Clone, Serialize)]
pub struct ScheduledMeeting {
    pub id: String,
    pub event_id: String,
    pub title: String,
    pub day: String,
    pub start: f64,
    pub length: f64,
    pub all_day: bool,
    pub attendees: Vec<String>,
    pub location: String,
    pub details: String,
    pub starts_at: String,
    pub ends_at: String,
    pub timezone: String,
    pub recurrence: String,
}

fn owner() -> Result<String> {
    auth::ready()?;
    let user = auth::current().ok_or_else(|| {
        Error::new(
            axum::http::StatusCode::UNAUTHORIZED,
            "Authentication required",
        )
    })?;
    rahti::authorization::authorize_resource("calendar.manage", &user.id)?;
    Ok(user.id)
}
fn invalid(field: &str, message: &str) -> Error {
    let mut validator = Validator::new();
    validator.check(field, false, message);
    validator.finish().unwrap_err().into()
}
fn id() -> String {
    use rand::TryRng;
    let mut bytes = [0u8; 16];
    rand::rngs::SysRng
        .try_fill_bytes(&mut bytes)
        .expect("system randomness");
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}
fn parse_local(day: &str, clock: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(&format!("{day} {clock}"), "%Y-%m-%d %H:%M").ok()
}
pub async fn schedule(input: ScheduleInput) -> Result<Vec<ScheduledMeeting>> {
    let owner_id = owner()?;
    let title = input.title.trim().to_string();
    let attendees: Vec<String> = input
        .attendees
        .split([',', ';', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();
    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120)
        .max_chars("attendees", &input.attendees, 3000)
        .check(
            "attendees",
            attendees.len() <= 20
                && attendees.iter().all(|value| {
                    value.chars().count() <= 120 && !value.chars().any(char::is_control)
                }),
            "Add up to 20 names or email addresses, separated by commas.",
        )
        .max_chars("location", &input.location, 300)
        .max_chars("details", &input.details, 8000)
        .one_of(
            "recurrence",
            &input.recurrence.as_str(),
            &["none", "daily", "weekly", "monthly"],
        )
        .check(
            "occurrences",
            (1..=52).contains(&input.occurrences),
            "Choose between 1 and 52 occurrences.",
        );
    validator.finish()?;
    let mut validator = Validator::new();
    for attendee in &attendees {
        if attendee.contains('@') {
            validator.email("attendees", attendee);
        }
    }
    validator.finish()?;
    let zone: rahti::time::Tz = input
        .timezone
        .parse()
        .map_err(|_| invalid("timezone", "Choose a valid time zone."))?;
    let start = parse_local(
        &input.start_date,
        if input.all_day {
            "00:00"
        } else {
            &input.start_time
        },
    )
    .ok_or_else(|| invalid("start_date", "Choose a valid start date and time."))?;
    let mut end = parse_local(
        &input.end_date,
        if input.all_day {
            "00:00"
        } else {
            &input.end_time
        },
    )
    .ok_or_else(|| invalid("end_date", "Choose a valid end date and time."))?;
    if input.all_day {
        end = end
            .checked_add_signed(Duration::days(1))
            .ok_or_else(|| invalid("end_date", "Choose a valid end date."))?;
    }
    if end <= start || end - start > Duration::days(31) {
        return Err(invalid(
            "end_date",
            "End must be after start, with a duration of at most 31 days.",
        ));
    }
    let count = if input.recurrence == "none" {
        1
    } else {
        input.occurrences
    };
    let series_id = id();
    let mut rows = Vec::new();
    for index in 0..count {
        let shift = |value: NaiveDateTime| -> Option<NaiveDateTime> {
            match input.recurrence.as_str() {
                "monthly" => value.checked_add_months(Months::new(index)),
                "weekly" => value.checked_add_signed(Duration::weeks(i64::from(index))),
                "daily" => value.checked_add_signed(Duration::days(i64::from(index))),
                _ => Some(value),
            }
        };
        let utc = |value| -> Result<DateTime<Utc>> {
            let local = shift(value).ok_or_else(|| {
                invalid(
                    "end_date",
                    "This repeat schedule exceeds the supported dates.",
                )
            })?;
            local.and_local_timezone(zone).single().map(|value| value.with_timezone(&Utc)).ok_or_else(|| invalid("timezone", "This time is ambiguous or does not exist due to daylight saving. Choose another time."))
        };
        let starts_at = utc(start)?;
        let ends_at = utc(end)?;
        if ends_at <= starts_at {
            return Err(invalid(
                "end_date",
                "End must be after start for every occurrence.",
            ));
        }
        rows.push(event::Model {
            id: id(),
            owner_id: owner_id.clone(),
            title: title.clone(),
            starts_at,
            ends_at,
            timezone: input.timezone.clone(),
            all_day: input.all_day,
            attendees: rahti::serde_json::to_string(&attendees)?,
            location: input.location.trim().into(),
            details: input.details.clone(),
            series_id: series_id.clone(),
            recurrence: input.recurrence.clone(),
        });
    }
    let transaction = db::db().begin().await?;
    for row in &rows {
        let active: event::ActiveModel = row.clone().into();
        active.insert(&transaction).await?;
        meeting_link::ActiveModel {
            id: Set(row.id.clone()),
            owner_id: Set(Some(owner_id.clone())),
            title: Set(title.clone()),
            created_at: Set(Utc::now()),
            revoked_at: Set(None),
            provider_meeting_id: Set(None),
            media_region: Set(None),
        }
        .insert(&transaction)
        .await?;
    }
    transaction.commit().await?;
    project(rows)
}

pub async fn list() -> Result<Vec<ScheduledMeeting>> {
    let owner_id = owner()?;
    project(
        event::Entity::find()
            .filter(event::Column::OwnerId.eq(owner_id))
            .order_by_asc(event::Column::StartsAt)
            .all(db::db())
            .await?,
    )
}
fn project(rows: Vec<event::Model>) -> Result<Vec<ScheduledMeeting>> {
    let mut output = Vec::new();
    for row in rows {
        let start = rahti::time::to_app_time(row.starts_at);
        let end = rahti::time::to_app_time(row.ends_at);
        let mut day: NaiveDate = start.date_naive();
        while day <= end.date_naive() {
            let a = if day == start.date_naive() {
                f64::from(start.hour()) + f64::from(start.minute()) / 60.0
            } else {
                0.0
            };
            let b = if day == end.date_naive() {
                f64::from(end.hour()) + f64::from(end.minute()) / 60.0
            } else {
                24.0
            };
            if b > a {
                output.push(ScheduledMeeting {
                    id: format!("{}@{day}", row.id),
                    event_id: row.id.clone(),
                    title: row.title.clone(),
                    day: day.to_string(),
                    start: a,
                    length: b - a,
                    all_day: row.all_day,
                    attendees: rahti::serde_json::from_str(&row.attendees)?,
                    location: row.location.clone(),
                    details: row.details.clone(),
                    starts_at: row.starts_at.to_rfc3339(),
                    ends_at: row.ends_at.to_rfc3339(),
                    timezone: row.timezone.clone(),
                    recurrence: row.recurrence.clone(),
                });
            }
            let Some(next) = day.succ_opt() else { break };
            day = next;
        }
    }
    Ok(output)
}
