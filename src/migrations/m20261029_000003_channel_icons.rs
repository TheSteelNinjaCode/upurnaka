use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CommunityChannel::Table)
                    .add_column(string(CommunityChannel::Icon).default("💬"))
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CommunityChannel::Table)
                    .drop_column(CommunityChannel::Icon)
                    .to_owned(),
            )
            .await
    }
}
#[derive(DeriveIden)]
enum CommunityChannel {
    Table,
    Icon,
}
