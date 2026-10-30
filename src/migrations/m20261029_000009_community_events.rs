use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(CommunityEvent::Table)
                .col(string(CommunityEvent::Id).primary_key())
                .col(string(CommunityEvent::CommunityId))
                .col(string(CommunityEvent::SeriesId))
                .col(string(CommunityEvent::OrganizerId))
                .col(string(CommunityEvent::Title))
                .col(timestamp_with_time_zone(CommunityEvent::StartsAt))
                .col(timestamp_with_time_zone(CommunityEvent::EndsAt))
                .col(string(CommunityEvent::Timezone))
                .col(boolean(CommunityEvent::AllDay))
                .col(string(CommunityEvent::Recurrence))
                .col(text(CommunityEvent::Location))
                .col(boolean(CommunityEvent::Online))
                .col(text(CommunityEvent::Details))
                .col(string(CommunityEvent::Banner))
                .col(big_integer(CommunityEvent::CreatedAt))
                .to_owned(),
        )
        .await?;
        m.create_index(
            Index::create()
                .name("idx_community_event_community_start")
                .table(CommunityEvent::Table)
                .col(CommunityEvent::CommunityId)
                .col(CommunityEvent::StartsAt)
                .to_owned(),
        )
        .await
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(CommunityEvent::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum CommunityEvent {
    Table,
    Id,
    CommunityId,
    SeriesId,
    OrganizerId,
    Title,
    StartsAt,
    EndsAt,
    Timezone,
    AllDay,
    Recurrence,
    Location,
    Online,
    Details,
    Banner,
    CreatedAt,
}
