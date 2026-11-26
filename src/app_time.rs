//! Where SeaORM's chrono values meet `rahti::time`'s jiff calendar.
//!
//! Columns hold chrono `DateTime<Utc>` or Unix seconds; the application
//! calendar is jiff. Each conversion lives here once, so a page never has to
//! know which crate a value came from.

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use rahti::time::{TimeZone, Timestamp, ZonedDateTime};

/// A stored UTC instant as jiff sees it, clamped to jiff's range
/// (years -9999 to 9999), which no stored row reaches.
pub fn instant(value: DateTime<Utc>) -> Timestamp {
    let nanoseconds = value.timestamp_subsec_nanos() as i32;
    Timestamp::new(value.timestamp(), nanoseconds).unwrap_or(if value.timestamp() < 0 {
        Timestamp::MIN
    } else {
        Timestamp::MAX
    })
}

/// A stored UTC instant in the application timezone.
pub fn local(value: DateTime<Utc>) -> ZonedDateTime {
    rahti::time::to_app_time(instant(value))
}

/// Unix seconds in the application timezone; `None` outside jiff's range.
pub fn local_seconds(seconds: i64) -> Option<ZonedDateTime> {
    Timestamp::from_second(seconds)
        .ok()
        .map(rahti::time::to_app_time)
}

/// The UTC instant a wall-clock time names in `zone`. `None` when daylight
/// saving skips that time or repeats it, so the caller asks for another.
pub fn resolve(local: NaiveDateTime, zone: &TimeZone) -> Option<DateTime<Utc>> {
    let civil = rahti::time::DateTime::new(
        i16::try_from(local.year()).ok()?,
        local.month() as i8,
        local.day() as i8,
        local.hour() as i8,
        local.minute() as i8,
        local.second() as i8,
        local.nanosecond() as i32,
    )
    .ok()?;
    let instant = zone.to_ambiguous_timestamp(civil).unambiguous().ok()?;
    DateTime::from_timestamp(instant.as_second(), instant.subsec_nanosecond() as u32)
}
