//! Folders in the Chat file picker: a tree per person, and the folder each
//! chat file sits in. A file with no folder is at the top of "My files".
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatFolder::Table)
                    .col(string(ChatFolder::Id).primary_key())
                    .col(string(ChatFolder::OwnerId))
                    .col(string_null(ChatFolder::ParentId))
                    .col(string(ChatFolder::Name))
                    .col(big_integer(ChatFolder::CreatedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_chat_folder_owner_parent")
                    .table(ChatFolder::Table)
                    .col(ChatFolder::OwnerId)
                    .col(ChatFolder::ParentId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ChatFile::Table)
                    .add_column(string_null(ChatFile::FolderId))
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ChatFile::Table)
                    .drop_column(ChatFile::FolderId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ChatFolder::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum ChatFolder {
    Table,
    Id,
    OwnerId,
    ParentId,
    Name,
    CreatedAt,
}
#[derive(DeriveIden)]
enum ChatFile {
    Table,
    FolderId,
}
