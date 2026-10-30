use sea_orm::entity::prelude::*;
// One occurrence of a community event. A repeating event is one row per
// occurrence sharing `series_id`; an online occurrence has a meeting link
// whose join code is its `id`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "community_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub community_id: String,
    pub series_id: String,
    pub organizer_id: String,
    pub title: String,
    pub starts_at: DateTimeUtc,
    pub ends_at: DateTimeUtc,
    pub timezone: String,
    pub all_day: bool,
    pub recurrence: String,
    pub location: String,
    pub online: bool,
    pub details: String,
    /// The details' chat_format lines as JSON, or empty when unformatted.
    pub formatting: String,
    pub banner: String,
    pub created_at: i64,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
