//! The `meeting_invite` table: the accounts an invite-only meeting lets in.
//!
//! One row per (meeting, account). The list is only consulted while the
//! link's `access` is `"invited"`; switching a meeting back to open keeps it,
//! so switching again does not make the organizer pick everybody twice.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meeting_invite")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub meeting_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
