use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for column in [
            CommunityPost::EditedAt,
            CommunityPost::DeletedAt,
            CommunityPost::PinnedAt,
        ] {
            m.alter_table(
                Table::alter()
                    .table(CommunityPost::Table)
                    .add_column(big_integer_null(column))
                    .to_owned(),
            )
            .await?;
        }
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for column in [
            CommunityPost::EditedAt,
            CommunityPost::DeletedAt,
            CommunityPost::PinnedAt,
        ] {
            m.alter_table(
                Table::alter()
                    .table(CommunityPost::Table)
                    .drop_column(column)
                    .to_owned(),
            )
            .await?;
        }
        Ok(())
    }
}
#[derive(DeriveIden, Clone, Copy)]
enum CommunityPost {
    Table,
    EditedAt,
    DeletedAt,
    PinnedAt,
}
