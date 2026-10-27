use sea_orm::entity::prelude::*;
// `file_id` names a Dilnaka Storage object; a URL to read it is minted per request.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meeting_recording")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub meeting_id: String,
    pub started_by: String,
    /// `recording`, `uploading`, `stored` or `failed`.
    pub state: String,
    pub content_type: String,
    /// How many pieces have arrived, which is also the next piece's number.
    pub parts: i32,
    pub bytes: i64,
    pub started_at: i64,
    pub updated_at: i64,
    pub ended_at: Option<i64>,
    pub file_id: Option<String>,
    pub file_key: Option<String>,
    /// Why an upload failed, for the operator. Never sent to a browser.
    pub failure: Option<String>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
