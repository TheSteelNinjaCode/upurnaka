//! Create the `todo` table.
//!
//! The file name is the version: `src/migrations/mod.rs` is generated from
//! this directory in filename order, and that is the order migrations are
//! applied. So a new migration is named for the moment it was written, and
//! adding the file is the whole of adding the migration.
//!
//! `up` makes the change and `down` undoes it. Write both — a `down` you
//! never run costs a minute, and the one time you need it you need it badly.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Todo::Table)
                    .if_not_exists()
                    .col(pk_auto(Todo::Id))
                    .col(string(Todo::Text))
                    .col(boolean(Todo::Done).default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Todo::Table).to_owned())
            .await
    }
}

/// The column names, as the query builder needs them. Kept beside the
/// migration rather than shared with the entity on purpose: this describes the
/// table as it was at this version, and a later migration that renames a
/// column must not change what an earlier one did.
#[derive(DeriveIden)]
enum Todo {
    Table,
    Id,
    Text,
    Done,
}
