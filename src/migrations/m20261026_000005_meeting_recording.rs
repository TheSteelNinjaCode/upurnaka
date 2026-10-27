//! Meeting recordings: one row per recording an organizer started, from the
//! first piece the browser sends to the Dilnaka Storage file it ends up as.
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MeetingRecording::Table)
                    .col(string(MeetingRecording::Id).primary_key())
                    .col(string(MeetingRecording::MeetingId))
                    .col(string(MeetingRecording::StartedBy))
                    .col(string(MeetingRecording::State))
                    .col(string(MeetingRecording::ContentType))
                    .col(integer(MeetingRecording::Parts).default(0))
                    .col(big_integer(MeetingRecording::Bytes).default(0))
                    .col(big_integer(MeetingRecording::StartedAt))
                    .col(big_integer(MeetingRecording::UpdatedAt))
                    .col(big_integer_null(MeetingRecording::EndedAt))
                    .col(string_null(MeetingRecording::FileId))
                    .col(string_null(MeetingRecording::FileKey))
                    .col(string_null(MeetingRecording::Failure))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_recording_meeting_state")
                    .table(MeetingRecording::Table)
                    .col(MeetingRecording::MeetingId)
                    .col(MeetingRecording::State)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MeetingRecording::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum MeetingRecording {
    Table,
    Id,
    MeetingId,
    StartedBy,
    State,
    ContentType,
    Parts,
    Bytes,
    StartedAt,
    UpdatedAt,
    EndedAt,
    FileId,
    FileKey,
    Failure,
}
