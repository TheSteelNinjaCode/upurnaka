//! Explicit administrator bootstrap, invoked only by `--seed-admin`.
use crate::{auth::Role, db, models::account, rahti::Result, users};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::io::Write;

pub async fn admin_from_env() -> Result<()> {
    crate::auth::ready()?;
    if let Some(admin) = account::Entity::find()
        .filter(account::Column::Role.eq("admin"))
        .one(db::db())
        .await?
    {
        println!(
            "Administrator already seeded: {}. Credentials were not changed.",
            admin.email
        );
        return Ok(());
    }
    let email = std::env::var("SEED_ADMIN_EMAIL").unwrap_or_else(|_| "admin@upurnaka.local".into());
    let name = std::env::var("SEED_ADMIN_NAME").unwrap_or_else(|_| "Administrator".into());
    let configured = std::env::var("SEED_ADMIN_PASSWORD").ok();
    let password = configured.clone().unwrap_or_else(|| {
        use base64::Engine;
        use rand::{TryRng, rngs::SysRng};
        let mut bytes = [0; 24];
        SysRng
            .try_fill_bytes(&mut bytes)
            .expect("operating system randomness");
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    });
    // Persist a generated password before inserting the account, so an I/O failure cannot
    // leave an inaccessible admin. Never print it, or overwrite a prior credentials file.
    let path = std::env::current_dir()?.join(".rahti/admin-credentials.txt");
    if configured.is_none() {
        std::fs::create_dir_all(path.parent().expect("credentials directory"))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        writeln!(
            file,
            "Email: {email}\nPassword: {password}\n\nSign in and enroll your own authenticator app."
        )?;
        file.sync_all()?;
    }
    match users::seed_admin(name, email, password).await {
        Ok(user) => {
            debug_assert_eq!(user.role, Role::Admin);
            println!(
                "Administrator seeded: {}. Sign in to enroll an authenticator.",
                user.email
            );
            if configured.is_none() {
                println!("Initial credentials saved to {}", path.display());
            }
            Ok(())
        }
        Err(error) => {
            if configured.is_none() {
                let _ = std::fs::remove_file(&path);
            }
            Err(error)
        }
    }
}
