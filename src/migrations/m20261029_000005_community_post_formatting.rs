use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite alters one column per statement.
        for column in [
            string(CommunityPost::Subject).default(""),
            string(CommunityPost::Headline).default(""),
            string(CommunityPost::Subheading).default(""),
            string(CommunityPost::Accent).default(""),
            text(CommunityPost::Content).default(""),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(CommunityPost::Table)
                        .add_column(column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for column in [
            CommunityPost::Subject,
            CommunityPost::Headline,
            CommunityPost::Subheading,
            CommunityPost::Accent,
            CommunityPost::Content,
        ] {
            manager
                .alter_table(
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
    Subject,
    Headline,
    Subheading,
    Accent,
    Content,
}
