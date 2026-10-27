use sea_orm::entity::prelude::*;

// Deliberately no Serialize: only PublicUser leaves auth. Debug redacts credentials.
#[derive(Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub email: String,
    pub name: String,
    pub picture: Option<Vec<u8>>,
    pub password_hash: String,
    pub otp_secret: String,
    pub verified: bool,
    pub last_step: i64,
    pub recovery_hashes: String,
    pub challenge_hash: String,
    pub challenge_expires: i64,
    pub attempts: i32,
    pub locked_until: i64,
    pub role: String,
    pub active: bool,
    pub session_version: i64,
    pub admin_slot: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}
