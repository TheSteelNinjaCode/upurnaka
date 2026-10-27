//! Chat attachments: one row per file a person uploaded from Chat, naming the
//! Dilnaka Storage object it became.
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatFile::Table)
                    .col(string(ChatFile::Id).primary_key())
                    .col(string(ChatFile::OwnerId))
                    .col(string(ChatFile::Name))
                    .col(string(ChatFile::ContentType))
                    .col(big_integer(ChatFile::Bytes))
                    .col(string(ChatFile::FileId))
                    .col(string(ChatFile::FileKey))
                    .col(big_integer(ChatFile::CreatedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_chat_file_owner_created")
                    .table(ChatFile::Table)
                    .col(ChatFile::OwnerId)
                    .col(ChatFile::CreatedAt)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatFile::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum ChatFile {
    Table,
    Id,
    OwnerId,
    Name,
    ContentType,
    Bytes,
    FileId,
    FileKey,
    CreatedAt,
}
