//! Create the `meeting_link` table.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MeetingLink::Table)
                    .if_not_exists()
                    .col(string(MeetingLink::Id).primary_key())
                    .col(string(MeetingLink::Title))
                    .col(timestamp_with_time_zone(MeetingLink::CreatedAt))
                    .col(timestamp_with_time_zone_null(MeetingLink::RevokedAt))
                    .to_owned(),
            )
            .await?;

        // The list is read newest first and nothing else, so that is the
        // index worth having.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_meeting_link_created_at")
                    .table(MeetingLink::Table)
                    .col(MeetingLink::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MeetingLink::Table).to_owned())
            .await
    }
}

/// The columns as this version of the table has them.
#[derive(DeriveIden)]
enum MeetingLink {
    Table,
    Id,
    Title,
    CreatedAt,
    RevokedAt,
}
