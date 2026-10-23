//! Record which conference is currently behind a meeting link.
//!
//! Both columns are nullable, and null is the ordinary state: a link that
//! nobody is in right now has no Chime meeting, because Chime cleans one up
//! after the last attendee leaves. The link is the durable thing.
//!
//! One `ALTER TABLE` per column, rather than one statement adding both:
//! SQLite takes a single alter option per statement and the suite runs on
//! SQLite, so the two-column form panics in the query builder before it ever
//! reaches a database.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .add_column(string_null(MeetingLink::ProviderMeetingId))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .add_column(string_null(MeetingLink::MediaRegion))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .drop_column(MeetingLink::MediaRegion)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .drop_column(MeetingLink::ProviderMeetingId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum MeetingLink {
    Table,
    ProviderMeetingId,
    MediaRegion,
}
