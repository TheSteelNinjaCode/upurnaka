use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(CommunityFile::Table)
                .col(string(CommunityFile::Id).primary_key())
                .col(string(CommunityFile::CommunityId))
                .col(string(CommunityFile::PostId))
                .col(string(CommunityFile::UploaderId))
                .col(string(CommunityFile::Name))
                .col(string(CommunityFile::ContentType))
                .col(big_integer(CommunityFile::Bytes))
                .col(string(CommunityFile::FileId))
                .col(string(CommunityFile::FileKey))
                .col(big_integer(CommunityFile::CreatedAt))
                .to_owned(),
        )
        .await?;
        m.create_index(
            Index::create()
                .name("idx_community_file_community")
                .table(CommunityFile::Table)
                .col(CommunityFile::CommunityId)
                .col(CommunityFile::PostId)
                .to_owned(),
        )
        .await
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(CommunityFile::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum CommunityFile {
    Table,
    Id,
    CommunityId,
    PostId,
    UploaderId,
    Name,
    ContentType,
    Bytes,
    FileId,
    FileKey,
    CreatedAt,
}
