use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Account::Table)
                    .add_column(string(Account::Role).default("user"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Account::Table)
                    .add_column(boolean(Account::Active).default(true))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Account::Table)
                    .add_column(big_integer(Account::SessionVersion).default(0))
                    .to_owned(),
            )
            .await?;
        // Null slots allow many users; the unique value 1 permits exactly one admin.
        // The check also prevents a role/slot mismatch, including under concurrent seeds.
        manager.alter_table(Table::alter().table(Account::Table)
            .add_column(integer_null(Account::AdminSlot).check(Expr::cust(
                "(role = 'user' AND admin_slot IS NULL) OR (role = 'admin' AND admin_slot IS NOT NULL AND admin_slot = 1)"
            ))).to_owned()).await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_single_admin")
                    .table(Account::Table)
                    .col(Account::AdminSlot)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_single_admin")
                    .table(Account::Table)
                    .to_owned(),
            )
            .await?;
        for column in [
            Account::AdminSlot,
            Account::SessionVersion,
            Account::Active,
            Account::Role,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Account::Table)
                        .drop_column(column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Role,
    Active,
    SessionVersion,
    AdminSlot,
}
