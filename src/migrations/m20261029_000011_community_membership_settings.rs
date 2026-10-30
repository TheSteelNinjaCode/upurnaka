use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Community::Table)
                .add_column(boolean(Community::MembersCanShare).default(true))
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Community::Table)
                .add_column(boolean(Community::RequireApproval).default(false))
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(CommunityMember::Table)
                .add_column(string(CommunityMember::Nickname).default(""))
                .to_owned(),
        )
        .await
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(CommunityMember::Table)
                .drop_column(CommunityMember::Nickname)
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Community::Table)
                .drop_column(Community::RequireApproval)
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Community::Table)
                .drop_column(Community::MembersCanShare)
                .to_owned(),
        )
        .await
    }
}
#[derive(DeriveIden)]
enum Community {
    Table,
    MembersCanShare,
    RequireApproval,
}
#[derive(DeriveIden)]
enum CommunityMember {
    Table,
    Nickname,
}
