//! Activity: what happened that a person should know about, kept until they
//! have seen it and after.
use sea_orm_migration::{prelude::*, schema::*};
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notification::Table)
                    .col(string(Notification::Id).primary_key())
                    .col(string(Notification::UserId))
                    .col(string(Notification::Kind))
                    .col(string(Notification::MeetingId))
                    .col(string(Notification::Actor))
                    .col(string(Notification::Title))
                    .col(text(Notification::Body))
                    .col(string(Notification::Link))
                    .col(big_integer(Notification::CreatedAt))
                    .col(big_integer_null(Notification::ReadAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_notification_user_created")
                    .table(Notification::Table)
                    .col(Notification::UserId)
                    .col(Notification::CreatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_notification_user_read")
                    .table(Notification::Table)
                    .col(Notification::UserId)
                    .col(Notification::ReadAt)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Notification::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum Notification {
    Table,
    Id,
    UserId,
    Kind,
    MeetingId,
    Actor,
    Title,
    Body,
    Link,
    CreatedAt,
    ReadAt,
}
