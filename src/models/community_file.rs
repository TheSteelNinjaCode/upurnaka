use sea_orm::entity::prelude::*;
// A file shared to a community. It belongs to the post its upload created;
// `file_id` names a Dilnaka Storage object read through a short-lived link.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "community_file")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub community_id: String,
    pub post_id: String,
    pub uploader_id: String,
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
