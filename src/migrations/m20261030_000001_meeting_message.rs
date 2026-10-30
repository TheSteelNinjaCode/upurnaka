//! Meeting chat: what participants wrote, kept with the meeting link.
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MeetingMessage::Table)
                    .col(string(MeetingMessage::Id).primary_key())
                    .col(string(MeetingMessage::MeetingId))
                    .col(string(MeetingMessage::SenderId))
                    .col(string(MeetingMessage::SenderName))
                    .col(text(MeetingMessage::Body))
                    .col(big_integer(MeetingMessage::SentAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_meeting_message_meeting_sent")
                    .table(MeetingMessage::Table)
                    .col(MeetingMessage::MeetingId)
                    .col(MeetingMessage::SentAt)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MeetingMessage::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum MeetingMessage {
    Table,
    Id,
    MeetingId,
    SenderId,
    SenderName,
    Body,
    SentAt,
}
