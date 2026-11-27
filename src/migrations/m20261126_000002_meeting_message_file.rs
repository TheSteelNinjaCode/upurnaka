//! A meeting chat message may carry one file, shared from the sender's files.
//!
//! The message keeps its own copy of what the chat shows (name, type, size)
//! and the storage object, so it survives the sender renaming or deleting
//! the file in My files afterwards.
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for column in [
            string_null(MeetingMessage::FileObject),
            string_null(MeetingMessage::FileName),
            string_null(MeetingMessage::FileType),
            big_integer_null(MeetingMessage::FileBytes),
        ] {
            let mut column = column;
            manager
                .alter_table(
                    Table::alter()
                        .table(MeetingMessage::Table)
                        .add_column(&mut column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for column in [
            MeetingMessage::FileBytes,
            MeetingMessage::FileType,
            MeetingMessage::FileName,
            MeetingMessage::FileObject,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(MeetingMessage::Table)
                        .drop_column(column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
#[derive(DeriveIden)]
enum MeetingMessage {
    Table,
    FileObject,
    FileName,
    FileType,
    FileBytes,
}
