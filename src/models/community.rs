use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "community")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub guidelines: String,
    pub avatar: String,
    pub picture: Option<String>,
    pub cover: String,
    pub cover_image: Option<String>,
    pub created_at: i64,
    pub join_token: String,
    pub deleted_at: Option<i64>,
    /// Members, not only owners, may copy the join link.
    pub members_can_share: bool,
    /// Joining by link asks an owner, who approves or declines the request.
    pub require_approval: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
