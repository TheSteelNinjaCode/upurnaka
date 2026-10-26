use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .col(string(Account::Id).primary_key())
                    .col(string(Account::Email).unique_key())
                    .col(string(Account::Name))
                    .col(text(Account::PasswordHash))
                    .col(text(Account::OtpSecret))
                    .col(boolean(Account::Verified))
                    .col(big_integer(Account::LastStep))
                    .col(text(Account::RecoveryHashes))
                    .col(string(Account::ChallengeHash))
                    .col(big_integer(Account::ChallengeExpires))
                    .col(integer(Account::Attempts))
                    .col(big_integer(Account::LockedUntil))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
    Email,
    Name,
    PasswordHash,
    OtpSecret,
    Verified,
    LastStep,
    RecoveryHashes,
    ChallengeHash,
    ChallengeExpires,
    Attempts,
    LockedUntil,
}
