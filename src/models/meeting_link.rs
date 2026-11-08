//! The `meeting_link` table.
//!
//! One row per link Meet has handed out. The primary key is the join code
//! itself rather than a counter: it is what the URL carries, what the browser
//! sends back, and what a meeting provider will be asked about later, so an
//! extra surrogate key would only be a second name for the same thing.
//!
//! `revoked_at` rather than a delete. A link that was shared has been seen by
//! people this account does not control, and the honest answer to one of them
//! following it afterwards is "this meeting link is no longer available" —
//! which needs the row to still be there.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "meeting_link")]
pub struct Model {
    /// The join code, as it appears in `/meet/j/{id}`.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: Option<String>,
    /// What the organizer called the meeting.
    pub title: String,
    /// When the link was minted, UTC — the instant, never the wall clock.
    pub created_at: DateTimeUtc,
    /// When the organizer revoked it, UTC. `None` while the link still works.
    pub revoked_at: Option<DateTimeUtc>,
    /// The Dilnaka Meet meeting currently behind this link, if there is one.
    ///
    /// Nullable, and expected to be null most of the time. A Chime meeting
    /// ends when the last attendee leaves; the link outlives it and gets a
    /// fresh conference the next time somebody joins. This column is a cache
    /// of which one is live, never the authority for whether the link is.
    pub provider_meeting_id: Option<String>,
    /// The media region that conference was created in.
    pub media_region: Option<String>,
}

/// Nothing points at this table yet. The empty enum is how that is written.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
