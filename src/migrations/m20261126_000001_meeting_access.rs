//! Who may enter a meeting: open to anyone with the link, behind a passcode,
//! or limited to the accounts the organizer invited.
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Every link that already exists was shared as an open link, and stays one.
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .add_column(string(MeetingLink::Access).default("open"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .add_column(string_null(MeetingLink::Passcode))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(MeetingInvite::Table)
                    .col(string(MeetingInvite::MeetingId))
                    .col(string(MeetingInvite::UserId))
                    .col(timestamp_with_time_zone(MeetingInvite::CreatedAt))
                    .primary_key(
                        Index::create()
                            .col(MeetingInvite::MeetingId)
                            .col(MeetingInvite::UserId),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MeetingInvite::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .drop_column(MeetingLink::Passcode)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .drop_column(MeetingLink::Access)
                    .to_owned(),
            )
            .await
    }
}
#[derive(DeriveIden)]
enum MeetingLink {
    Table,
    Access,
    Passcode,
}
#[derive(DeriveIden)]
enum MeetingInvite {
    Table,
    MeetingId,
    UserId,
    CreatedAt,
}
