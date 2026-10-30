//! Community events, as Teams schedules them from a community's Events view.
//! Any member may schedule one; every member sees the upcoming ones. Wall-clock
//! inputs are resolved once in the chosen time zone and stored in UTC.
use crate::{
    communities::{self, BANNERS},
    db,
    models::{community_event as event, meeting_link},
    rahti::{self, Error, Result, Validator},
};
use chrono::{DateTime, Duration, Months, NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};

/// How many upcoming occurrences the Events view lists.
const UPCOMING: u64 = 200;

#[derive(Deserialize)]
pub struct EventInput {
    pub title: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub timezone: String,
    pub all_day: bool,
    pub recurrence: String,
    pub occurrences: u32,
    pub location: String,
    pub online: bool,
    pub details: String,
    pub banner: String,
}

/// One occurrence, as the Events view shows it.
#[derive(Clone, Serialize)]
pub struct CommunityEvent {
    pub id: String,
    pub series_id: String,
    pub title: String,
    pub starts_at: String,
    pub ends_at: String,
    pub timezone: String,
    pub all_day: bool,
    pub recurrence: String,
    pub location: String,
    /// Where an online event is joined, `/meet/j/{id}`.
    pub join_url: Option<String>,
    pub details: String,
    pub banner: String,
    pub organizer_name: String,
    pub is_mine: bool,
}

fn invalid(field: &str, message: &str) -> Error {
    let mut validator = Validator::new();
    validator.check(field, false, message);
    validator.finish().unwrap_err().into()
}
fn parse_local(day: &str, clock: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(&format!("{day} {clock}"), "%Y-%m-%d %H:%M").ok()
}

async fn shown(rows: Vec<event::Model>, viewer: &str) -> Result<Vec<CommunityEvent>> {
    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        output.push(CommunityEvent {
            organizer_name: communities::author_name(&row.organizer_id).await?,
            is_mine: row.organizer_id == viewer,
            join_url: row.online.then(|| format!("/meet/j/{}", row.id)),
            starts_at: row.starts_at.to_rfc3339(),
            ends_at: row.ends_at.to_rfc3339(),
            id: row.id,
            series_id: row.series_id,
            title: row.title,
            timezone: row.timezone,
            all_day: row.all_day,
            recurrence: row.recurrence,
            location: row.location,
            details: row.details,
            banner: row.banner,
        });
    }
    Ok(output)
}

/// The community's events that have not ended yet, soonest first.
pub async fn upcoming(community: String) -> Result<Vec<CommunityEvent>> {
    communities::accessible(&community).await?;
    let viewer = communities::owner()?;
    let rows = event::Entity::find()
        .filter(event::Column::CommunityId.eq(&community))
        .filter(event::Column::EndsAt.gt(Utc::now()))
        .order_by_asc(event::Column::StartsAt)
        .limit(UPCOMING)
        .all(db::db())
        .await?;
    shown(rows, &viewer).await
}

/// Schedule an event, or a series of them, in a community the viewer belongs to.
pub async fn create(community: String, input: EventInput) -> Result<Vec<CommunityEvent>> {
    let community_row = communities::accessible(&community).await?;
    let organizer = communities::owner()?;
    let title = input.title.trim().to_string();
    let location = input.location.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120)
        .max_chars("location", &location, 300)
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
        )
        .check(
            "banner",
            BANNERS.contains(&input.banner.as_str()),
            "Choose one of the event banners.",
        );
    validator.finish()?;
    let zone: rahti::time::Tz = input
        .timezone
        .parse()
        .map_err(|_| invalid("timezone", "Choose a valid time zone."))?;
    let clock = |value: &str| {
        if input.all_day {
            "00:00".to_string()
        } else {
            value.to_string()
        }
    };
    let start = parse_local(&input.start_date, &clock(&input.start_time))
        .ok_or_else(|| invalid("start_date", "Choose a valid start date and time."))?;
    let mut end = parse_local(&input.end_date, &clock(&input.end_time))
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
    let series_id = communities::id();
    let created_at = Utc::now().timestamp_millis();
    let mut rows = Vec::new();
    for index in 0..count {
        let utc = |value: NaiveDateTime| -> Result<DateTime<Utc>> {
            let shifted = match input.recurrence.as_str() {
                "monthly" => value.checked_add_months(Months::new(index)),
                "weekly" => value.checked_add_signed(Duration::weeks(i64::from(index))),
                "daily" => value.checked_add_signed(Duration::days(i64::from(index))),
                _ => Some(value),
            }
            .ok_or_else(|| {
                invalid(
                    "end_date",
                    "This repeat schedule exceeds the supported dates.",
                )
            })?;
            shifted
                .and_local_timezone(zone)
                .single()
                .map(|value| value.with_timezone(&Utc))
                .ok_or_else(|| {
                    invalid(
                        "start_date",
                        "This time is ambiguous or does not exist due to daylight saving. Choose another time.",
                    )
                })
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
            id: communities::id(),
            community_id: community_row.id.clone(),
            series_id: series_id.clone(),
            organizer_id: organizer.clone(),
            title: title.clone(),
            starts_at,
            ends_at,
            timezone: input.timezone.clone(),
            all_day: input.all_day,
            recurrence: input.recurrence.clone(),
            location: location.clone(),
            online: input.online,
            details: input.details.clone(),
            banner: input.banner.clone(),
            created_at,
        });
    }
    let transaction = db::db().begin().await?;
    for row in &rows {
        let active: event::ActiveModel = row.clone().into();
        active.insert(&transaction).await?;
        if row.online {
            meeting_link::ActiveModel {
                id: Set(row.id.clone()),
                owner_id: Set(Some(organizer.clone())),
                title: Set(title.clone()),
                created_at: Set(Utc::now()),
                revoked_at: Set(None),
                provider_meeting_id: Set(None),
                media_region: Set(None),
            }
            .insert(&transaction)
            .await?;
        }
    }
    transaction.commit().await?;
    shown(rows, &organizer).await
}
