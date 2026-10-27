use sea_orm::entity::prelude::*;
// `file_id` names a Dilnaka Storage object; a URL to read it is minted per request.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "chat_file")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: String,
    /// The name the browser reported, shown to people and used for downloads.
    pub name: String,
    pub content_type: String,
    pub bytes: i64,
    pub file_id: String,
    pub file_key: String,
    pub created_at: i64,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
