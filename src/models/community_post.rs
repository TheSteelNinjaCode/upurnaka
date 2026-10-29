use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "community_post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub community_id: String,
    pub channel_id: String,
    pub body: String,
    pub announcement: bool,
    pub created_at: i64,
    pub author_id: String,
    pub subject: String,
    pub headline: String,
    pub subheading: String,
    pub accent: String,
    /// The formatted message as JSON `Block`s; empty for plain-text posts.
    pub content: String,
    pub edited_at: Option<i64>,
    pub deleted_at: Option<i64>,
    pub pinned_at: Option<i64>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
