use sea_orm::entity::prelude::*;
// One row per person told. `title` and `body` are written when the event
// happens, so renaming a meeting later does not rewrite what was said.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notification")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// The account this is for.
    pub user_id: String,
    /// `invite`, `scheduled`, `lobby`, `mention` or `recording`.
    pub kind: String,
    pub meeting_id: String,
    /// The name of whoever caused it; empty when nobody did.
    pub actor: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    /// Where opening it goes: a path inside this application.
    pub link: String,
    /// Milliseconds since the epoch, UTC.
    pub created_at: i64,
    pub read_at: Option<i64>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
