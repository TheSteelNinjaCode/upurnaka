use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Old unowned links remain joinable by URL, but belong to nobody's account list.
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .add_column(string_null(MeetingLink::OwnerId))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_meeting_owner")
                    .table(MeetingLink::Table)
                    .col(MeetingLink::OwnerId)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_meeting_owner")
                    .table(MeetingLink::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(MeetingLink::Table)
                    .drop_column(MeetingLink::OwnerId)
                    .to_owned(),
            )
            .await
    }
}
#[derive(DeriveIden)]
enum MeetingLink {
    Table,
    OwnerId,
}
