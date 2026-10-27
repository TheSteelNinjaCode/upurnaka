use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "calendar_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub starts_at: DateTimeUtc,
    pub ends_at: DateTimeUtc,
    pub timezone: String,
    pub all_day: bool,
    pub attendees: String,
    pub location: String,
    pub details: String,
    pub series_id: String,
    pub recurrence: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
