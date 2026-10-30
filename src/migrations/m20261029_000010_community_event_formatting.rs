use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(CommunityEvent::Table)
                .add_column(text(CommunityEvent::Formatting).default(""))
                .to_owned(),
        )
        .await
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(CommunityEvent::Table)
                .drop_column(CommunityEvent::Formatting)
                .to_owned(),
        )
        .await
    }
}
#[derive(DeriveIden)]
enum CommunityEvent {
    Table,
    Formatting,
}
