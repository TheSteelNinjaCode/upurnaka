use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meeting_control")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub meeting_id: String,
    pub spotlight: Option<String>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
