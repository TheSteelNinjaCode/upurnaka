use sea_orm::entity::prelude::*;
// Contains provider identifiers, but never a join token.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meeting_visit")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub meeting_id: String,
    pub user_id: String,
    pub name: String,
    pub state: String,
    pub lease_until: i64,
    pub attendee_id: Option<String>,
    pub provider_meeting_id: Option<String>,
    pub muted_by_host: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
