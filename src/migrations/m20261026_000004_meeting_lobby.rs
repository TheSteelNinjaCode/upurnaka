use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MeetingVisit::Table)
                    .col(string(MeetingVisit::Id).primary_key())
                    .col(string(MeetingVisit::MeetingId))
                    .col(string(MeetingVisit::UserId))
                    .col(string(MeetingVisit::Name))
                    .col(string(MeetingVisit::State))
                    .col(big_integer(MeetingVisit::LeaseUntil))
                    .col(string_null(MeetingVisit::AttendeeId))
                    .col(string_null(MeetingVisit::ProviderMeetingId))
                    .col(boolean(MeetingVisit::MutedByHost).default(false))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_visit_meeting_lease")
                    .table(MeetingVisit::Table)
                    .col(MeetingVisit::MeetingId)
                    .col(MeetingVisit::LeaseUntil)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(MeetingControl::Table)
                    .col(string(MeetingControl::MeetingId).primary_key())
                    .col(string_null(MeetingControl::Spotlight))
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MeetingControl::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MeetingVisit::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]

enum MeetingVisit {
    Table,
    Id,
    MeetingId,
    UserId,
    Name,
    State,
    LeaseUntil,
    AttendeeId,
    ProviderMeetingId,
    MutedByHost,
}
#[derive(DeriveIden)]

enum MeetingControl {
    Table,
    MeetingId,
    Spotlight,
}
