use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Community::Table)
                    .col(string(Community::Id).primary_key())
                    .col(string(Community::OwnerId))
                    .col(string(Community::Name))
                    .col(string(Community::Category))
                    .col(text(Community::Description))
                    .col(text(Community::Guidelines))
                    .col(string(Community::Avatar))
                    .col(text_null(Community::Picture))
                    .col(string(Community::Cover))
                    .col(big_integer(Community::CreatedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_community_owner")
                    .table(Community::Table)
                    .col(Community::OwnerId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(CommunityChannel::Table)
                    .col(string(CommunityChannel::Id).primary_key())
                    .col(string(CommunityChannel::CommunityId))
                    .col(string(CommunityChannel::Name))
                    .col(big_integer(CommunityChannel::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(CommunityChannel::Table, CommunityChannel::CommunityId)
                            .to(Community::Table, Community::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_community_channel_name")
                    .table(CommunityChannel::Table)
                    .col(CommunityChannel::CommunityId)
                    .col(CommunityChannel::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(CommunityPost::Table)
                    .col(string(CommunityPost::Id).primary_key())
                    .col(string(CommunityPost::CommunityId))
                    .col(string(CommunityPost::ChannelId))
                    .col(text(CommunityPost::Body))
                    .col(boolean(CommunityPost::Announcement))
                    .col(big_integer(CommunityPost::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(CommunityPost::Table, CommunityPost::CommunityId)
                            .to(Community::Table, Community::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_community_posts")
                    .table(CommunityPost::Table)
                    .col(CommunityPost::CommunityId)
                    .col(CommunityPost::ChannelId)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CommunityPost::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CommunityChannel::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Community::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum Community {
    Table,
    Id,
    OwnerId,
    Name,
    Category,
    Description,
    Guidelines,
    Avatar,
    Picture,
    Cover,
    CreatedAt,
}
#[derive(DeriveIden)]
enum CommunityChannel {
    Table,
    Id,
    CommunityId,
    Name,
    CreatedAt,
}
#[derive(DeriveIden)]
enum CommunityPost {
    Table,
    Id,
    CommunityId,
    ChannelId,
    Body,
    Announcement,
    CreatedAt,
}
