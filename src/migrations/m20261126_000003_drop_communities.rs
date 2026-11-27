//! Communities are gone from the product, so their tables go too.
//!
//! Children are dropped before the tables they reference. Rolling back
//! replays the original community migrations, so the schema returns empty:
//! the rows dropped here cannot be restored.
use super::{
    m20261029_000001_communities as m1, m20261029_000002_community_actions as m2,
    m20261029_000003_channel_icons as m3, m20261029_000004_community_banner_image as m4,
    m20261029_000005_community_post_formatting as m5,
    m20261029_000006_community_post_conversations as m6,
    m20261029_000007_community_post_state as m7, m20261029_000008_community_files as m8,
    m20261029_000009_community_events as m9, m20261029_000010_community_event_formatting as m10,
    m20261029_000011_community_membership_settings as m11,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const TABLES: [&str; 9] = [
    "community_post_reaction",
    "community_post_reply",
    "community_file",
    "community_report",
    "community_event",
    "community_member",
    "community_post",
    "community_channel",
    "community",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in TABLES {
            manager
                .drop_table(
                    Table::drop()
                        .table(Alias::new(table))
                        .if_exists()
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        m1::Migration.up(manager).await?;
        m2::Migration.up(manager).await?;
        m3::Migration.up(manager).await?;
        m4::Migration.up(manager).await?;
        m5::Migration.up(manager).await?;
        m6::Migration.up(manager).await?;
        m7::Migration.up(manager).await?;
        m8::Migration.up(manager).await?;
        m9::Migration.up(manager).await?;
        m10::Migration.up(manager).await?;
        m11::Migration.up(manager).await?;
        Ok(())
    }
}
