use sea_orm::entity::prelude::*;
// `sender_name` is the name the sender joined with, kept as it was when they wrote.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meeting_message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub meeting_id: String,
    pub sender_id: String,
    pub sender_name: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    /// Milliseconds since the epoch, UTC.
    pub sent_at: i64,
    /// The Dilnaka Storage object of a shared file. Read through a
    /// short-lived link minted per participant, never stored as a URL.
    pub file_object: Option<String>,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub file_bytes: Option<i64>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
