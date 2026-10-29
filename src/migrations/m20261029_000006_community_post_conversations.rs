use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(CommunityPostReply::Table)
                .col(string(CommunityPostReply::Id).primary_key())
                .col(string(CommunityPostReply::CommunityId))
                .col(string(CommunityPostReply::PostId))
                .col(string(CommunityPostReply::AuthorId))
                .col(text(CommunityPostReply::Body))
                .col(big_integer(CommunityPostReply::CreatedAt))
                .to_owned(),
        )
        .await?;
        m.create_index(
            Index::create()
                .name("idx_community_post_reply_post")
                .table(CommunityPostReply::Table)
                .col(CommunityPostReply::CommunityId)
                .col(CommunityPostReply::PostId)
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(CommunityPostReaction::Table)
                .col(string(CommunityPostReaction::Id).primary_key())
                .col(string(CommunityPostReaction::CommunityId))
                .col(string(CommunityPostReaction::PostId))
                .col(string(CommunityPostReaction::UserId))
                .col(string(CommunityPostReaction::Emoji))
                .col(big_integer(CommunityPostReaction::CreatedAt))
                .to_owned(),
        )
        .await?;
        // One reaction of each kind per person, even when requests race.
        m.create_index(
            Index::create()
                .name("idx_community_post_reaction_unique")
                .table(CommunityPostReaction::Table)
                .col(CommunityPostReaction::PostId)
                .col(CommunityPostReaction::UserId)
                .col(CommunityPostReaction::Emoji)
                .unique()
                .to_owned(),
        )
        .await
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(CommunityPostReaction::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(CommunityPostReply::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum CommunityPostReply {
    Table,
    Id,
    CommunityId,
    PostId,
    AuthorId,
    Body,
    CreatedAt,
}
#[derive(DeriveIden)]
enum CommunityPostReaction {
    Table,
    Id,
    CommunityId,
    PostId,
    UserId,
    Emoji,
    CreatedAt,
}
