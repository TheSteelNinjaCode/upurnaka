use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CalendarEvent::Table)
                    .col(string(CalendarEvent::Id).primary_key())
                    .col(string(CalendarEvent::OwnerId))
                    .col(string(CalendarEvent::Title))
                    .col(timestamp_with_time_zone(CalendarEvent::StartsAt))
                    .col(timestamp_with_time_zone(CalendarEvent::EndsAt))
                    .col(string(CalendarEvent::Timezone))
                    .col(boolean(CalendarEvent::AllDay))
                    .col(text(CalendarEvent::Attendees))
                    .col(text(CalendarEvent::Location))
                    .col(text(CalendarEvent::Details))
                    .col(string(CalendarEvent::SeriesId))
                    .col(string(CalendarEvent::Recurrence))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_owner_start")
                    .table(CalendarEvent::Table)
                    .col(CalendarEvent::OwnerId)
                    .col(CalendarEvent::StartsAt)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CalendarEvent::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum CalendarEvent {
    Table,
    Id,
    OwnerId,
    Title,
    StartsAt,
    EndsAt,
    Timezone,
    AllDay,
    Attendees,
    Location,
    Details,
    SeriesId,
    Recurrence,
}
