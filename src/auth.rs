//! Password + authenticator authentication. A session is issued only after the second factor.
use crate::{
    db,
    models::account,
    rahti::{self, Error, Result, Validator},
};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{TryRng, rngs::SysRng};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, PaginatorTrait,
    QueryFilter, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, Builder, Secret, Totp};

pub fn settings() -> rahti::auth::AuthSettings {
    rahti::auth::AuthSettings {
        all_private: true,
        public_routes: vec!["/meet/j/[id]".into()],
        auth_routes: vec!["/signin".into(), "/register".into(), "/setup".into()],
        signin: "/signin".into(),
        after_signin: "/chat".into(),
        after_signout: "/signin".into(),
        sliding: false,
        ..rahti::auth::AuthSettings::from_env()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub has_picture: bool,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub session_version: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    #[default]
    User,
}

impl From<&account::Model> for PublicUser {
    fn from(row: &account::Model) -> Self {
        Self {
            id: row.id.clone(),
            name: row.name.clone(),
            email: row.email.clone(),
            has_picture: row.picture.is_some(),
            role: if row.role == "admin" {
                Role::Admin
            } else {
                Role::User
            },
            session_version: row.session_version,
        }
    }
}

tokio::task_local! { pub(crate) static REQUEST_USER: Option<PublicUser>; }

impl PublicUser {
    pub fn initials(&self) -> String {
        self.name
            .split_whitespace()
            .take(2)
            .filter_map(|s| s.chars().next())
            .collect::<String>()
            .to_uppercase()
    }
}

pub fn current() -> Option<PublicUser> {
    REQUEST_USER
        .try_with(Clone::clone)
        .unwrap_or_else(|_| rahti::auth::user())
}

pub fn authorization() -> rahti::authorization::Gate {
    let mut gate = rahti::authorization::Gate::new();
    // The collaboration directory exposes display names, not account management data.
    gate.define::<PublicUser, _>("directory.search", |user| !user.id.is_empty());
    gate.define_resource::<PublicUser, PublicUser, _>("users.manage", |session, live| {
        session.id == live.id
            && live.role == Role::Admin
            && session.session_version == live.session_version
    });
    gate.define_resource::<PublicUser, String, _>("meetings.manage", |user, owner| {
        user.id == *owner
    });
    gate.define_resource::<PublicUser, String, _>("profile.update", |user, owner| {
        user.id == *owner
    });
    gate.define_resource::<PublicUser, String, _>("calendar.manage", |user, owner| {
        user.id == *owner
    });
    gate.define_resource::<PublicUser, String, _>("communities.manage", |user, owner| {
        user.id == *owner
    });
    gate
}

pub(crate) fn owner_id() -> Result<String> {
    if let Some(user) = current() {
        return Ok(user.id);
    }
    // Store-level tests run outside HTTP. Router tests still exercise the real session guard.
    #[cfg(test)]
    {
        Ok("legacy-test-user".into())
    }
    #[cfg(not(test))]
    {
        Err(Error::new(
            StatusCode::UNAUTHORIZED,
            "Authentication required",
        ))
    }
}

pub(crate) fn authorize_meeting(owner: Option<&str>) -> Result<()> {
    let owner = owner.unwrap_or("").to_string();
    if owner != owner_id()? {
        return Err(Error::new(StatusCode::FORBIDDEN, "Permission denied"));
    }
    if current().is_some() {
        rahti::authorization::authorize_resource("meetings.manage", &owner)?;
    }
    Ok(())
}

#[derive(Serialize)]
pub struct Challenge {
    pub token: String,
    pub setup_key: Option<String>,
    pub qr: Option<String>,
}

#[derive(Serialize)]
pub struct SignedIn {
    pub recovery_codes: Vec<String>,
}

fn unavailable() -> Error {
    Error::public(
        StatusCode::SERVICE_UNAVAILABLE,
        "Accounts are unavailable. Please try again later.",
    )
}
fn invalid() -> Error {
    Error::new(
        StatusCode::BAD_REQUEST,
        "Sign-in failed. Check your details or start again.",
    )
}
fn now() -> i64 {
    rahti::time::Timestamp::now().as_second()
}
fn digest(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(value.as_bytes()))
}
fn random_token() -> String {
    let mut b = [0; 32];
    SysRng
        .try_fill_bytes(&mut b)
        .expect("operating system randomness");
    URL_SAFE_NO_PAD.encode(b)
}
pub(crate) fn ready() -> Result<()> {
    if !db::is_connected() {
        return Err(unavailable());
    }
    // A per-process development signing key cannot decrypt persistent authenticator secrets
    // after a restart. Refuse account operations until a stable key is configured.
    #[cfg(not(test))]
    if std::env::var("AUTH_SECRET")
        .ok()
        .filter(|key| key.len() >= 16)
        .is_none()
    {
        return Err(Error::public(
            StatusCode::SERVICE_UNAVAILABLE,
            "Account security needs a persistent AUTH_SECRET. Configure it and restart the application.",
        ));
    }
    Ok(())
}

// Domain-separated encryption key; keep AUTH_SECRET stable and back it up with the database.
fn cipher() -> Aes256Gcm {
    let key = Sha256::digest(format!(
        "upurnaka:otp:v1:{}",
        rahti::auth::settings().secret
    ));
    Aes256Gcm::new_from_slice(&key).expect("32-byte key")
}
fn encrypt(secret: &[u8]) -> Result<String> {
    let mut nonce = [0; 12];
    SysRng
        .try_fill_bytes(&mut nonce)
        .map_err(|_| unavailable())?;
    let encrypted = cipher()
        .encrypt(&Nonce::from(nonce), secret)
        .map_err(|_| unavailable())?;
    Ok(URL_SAFE_NO_PAD.encode([nonce.to_vec(), encrypted].concat()))
}
fn decrypt(value: &str) -> Result<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| unavailable())?;
    if bytes.len() < 28 {
        return Err(unavailable());
    }
    let nonce: [u8; 12] = bytes[..12].try_into().map_err(|_| unavailable())?;
    cipher()
        .decrypt(&Nonce::from(nonce), &bytes[12..])
        .map_err(|_| unavailable())
}
pub(crate) fn authenticator(secret: Vec<u8>, email: String) -> Result<Totp> {
    Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_digits(6)
        .with_skew(0)
        .with_step_duration(30)
        .with_secret(secret)
        .with_issuer(Some("upurnaka"))
        .with_account_name(email)
        .build()
        .map_err(|_| unavailable())
}
async fn hash_password(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|_| unavailable())
    })
    .await
    .map_err(|_| unavailable())?
}
async fn password_matches(password: String, hash: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
    })
    .await
    .map_err(|_| unavailable())
}
fn validate(email: &str, password: &str) -> Result<()> {
    let mut v = Validator::new();
    v.required("email", email)
        .email("email", email)
        .max_chars("email", email, 254)
        .required("password", password)
        .max_chars("password", password, 128);
    v.finish()?;
    Ok(())
}

pub async fn register(
    name: String,
    email: String,
    password: String,
    confirmation: String,
) -> Result<Challenge> {
    // Nobody joins a workspace that has no administrator yet: the first
    // account is created through `/setup`, and it is the admin.
    if !administrator_exists().await? {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "upurnaka isn't set up yet. Create the administrator account first.",
        ));
    }
    let row = create_account(name, email, password, confirmation, Role::User).await?;
    issue_challenge(row).await
}

/// Whether the single administrator account exists yet.
pub async fn administrator_exists() -> Result<bool> {
    ready()?;
    Ok(account::Entity::find()
        .filter(account::Column::Role.eq("admin"))
        .count(db::db())
        .await?
        > 0)
}

/// True only when accounts are available and no administrator exists: a
/// fresh installation, whose sign-in and registration pages send people to
/// `/setup`. An unavailable database is not a first launch.
pub async fn needs_setup() -> bool {
    matches!(administrator_exists().await, Ok(false))
}

fn already_set_up() -> Error {
    Error::new(
        StatusCode::CONFLICT,
        "upurnaka is already set up. Sign in instead.",
    )
}

/// First launch: create the administrator from `/setup`, then continue to
/// enrolling an authenticator exactly as a registration does. Refused once an
/// administrator exists, so the page cannot create a second one or promote
/// anybody.
pub async fn create_first_admin(
    name: String,
    email: String,
    password: String,
    confirmation: String,
) -> Result<Challenge> {
    // One setup at a time in this process; across processes the database's
    // single admin slot is the authority, and the loser is told it is set up.
    static SETUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _guard = SETUP.lock().await;
    if administrator_exists().await? {
        return Err(already_set_up());
    }
    let row = match create_account(name, email, password, confirmation, Role::Admin).await {
        Ok(row) => row,
        Err(error) if error.status() == StatusCode::CONFLICT => {
            return Err(if administrator_exists().await? {
                already_set_up()
            } else {
                error
            });
        }
        Err(error) => return Err(error),
    };
    issue_challenge(row).await
}

async fn issue_challenge(row: account::Model) -> Result<Challenge> {
    let token = random_token();
    account::Entity::update_many()
        .col_expr(account::Column::ChallengeHash, Expr::value(digest(&token)))
        .col_expr(account::Column::ChallengeExpires, Expr::value(now() + 600))
        .filter(account::Column::Id.eq(&row.id))
        .exec(db::db())
        .await?;
    challenge(row, token)
}

pub(crate) async fn create_account(
    name: String,
    email: String,
    password: String,
    confirmation: String,
    role: Role,
) -> Result<account::Model> {
    let email = email.trim().to_lowercase();
    let name = name.trim().to_string();
    validate(&email, &password)?;
    let mut v = Validator::new();
    v.required("name", &name)
        .max_chars("name", &name, 80)
        .min_chars("password", &password, 12)
        .same("confirmation", &confirmation, "password", &password);
    v.finish()?;
    ready()?;
    let secret = Secret::generate();
    let row = account::ActiveModel {
        id: Set(random_token()), email: Set(email), name: Set(name),
        picture: Set(None),
        password_hash: Set(hash_password(password).await?), otp_secret: Set(encrypt(secret.as_bytes())?),
        verified: Set(false), last_step: Set(-1), recovery_hashes: Set(String::new()),
        challenge_hash: Set(String::new()), challenge_expires: Set(0),
        attempts: Set(0), locked_until: Set(0),
        role: Set(if role == Role::Admin { "admin".into() } else { "user".into() }),
        active: Set(true), session_version: Set(0),
        admin_slot: Set(if role == Role::Admin { Some(1) } else { None }),
    }.insert(db::db()).await.map_err(|e| {
        if e.sql_err().is_some_and(|e| matches!(e, sea_orm::SqlErr::UniqueConstraintViolation(_))) {
            Error::new(StatusCode::CONFLICT, "Unable to register this email. If you already registered, sign in to continue setup.")
        } else { Error::from(e) }
    })?;
    Ok(row)
}

fn challenge(row: account::Model, token: String) -> Result<Challenge> {
    if row.verified {
        return Ok(Challenge {
            token,
            setup_key: None,
            qr: None,
        });
    }
    let totp = authenticator(decrypt(&row.otp_secret)?, row.email)?;
    Ok(Challenge {
        token,
        setup_key: Some(totp.secret().to_base32()),
        qr: Some(format!(
            "data:image/png;base64,{}",
            totp.to_qr_base64().map_err(|_| unavailable())?
        )),
    })
}

pub async fn begin_signin(email: String, password: String) -> Result<Challenge> {
    let email = email.trim().to_lowercase();
    validate(&email, &password)?;
    ready()?;
    let row = account::Entity::find()
        .filter(account::Column::Email.eq(email))
        .one(db::db())
        .await?;
    // Unknown addresses still pay the same Argon2 verification cost.
    static DUMMY: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();
    let dummy = DUMMY
        .get_or_try_init(|| hash_password(random_token()))
        .await?;
    let matches = password_matches(
        password,
        row.as_ref()
            .map(|u| &u.password_hash)
            .unwrap_or(dummy)
            .clone(),
    )
    .await?;
    let Some(row) = row else {
        return Err(invalid());
    };
    if !row.active || row.locked_until > now() {
        return Err(invalid());
    }
    if !matches {
        fail(&row).await?;
        return Err(invalid());
    }
    let token = random_token();
    account::Entity::update_many()
        .col_expr(account::Column::ChallengeHash, Expr::value(digest(&token)))
        .col_expr(account::Column::ChallengeExpires, Expr::value(now() + 600))
        .filter(account::Column::Id.eq(&row.id))
        .exec(db::db())
        .await?;
    challenge(row, token)
}

// Atomic increments prevent parallel guesses from losing the failure count. Lock for 15 minutes
// after five failures; successful password checks never reset the second-factor budget.
async fn fail(row: &account::Model) -> Result<()> {
    account::Entity::update_many()
        .col_expr(
            account::Column::Attempts,
            Expr::col(account::Column::Attempts).add(1),
        )
        .filter(account::Column::Id.eq(&row.id))
        .exec(db::db())
        .await?;
    account::Entity::update_many()
        .col_expr(account::Column::LockedUntil, Expr::value(now() + 900))
        .col_expr(account::Column::ChallengeHash, Expr::value(""))
        .col_expr(account::Column::Attempts, Expr::value(0))
        .filter(account::Column::Id.eq(&row.id))
        .filter(account::Column::Attempts.gte(5))
        .exec(db::db())
        .await?;
    Ok(())
}

pub async fn finish_signin(token: String, code: String) -> Result<SignedIn> {
    let mut v = Validator::new();
    v.required("token", &token)
        .max_chars("token", &token, 100)
        .required("code", &code)
        .max_chars("code", &code, 100);
    v.finish()?;
    ready()?;
    let row = account::Entity::find()
        .filter(account::Column::ChallengeHash.eq(digest(&token)))
        .filter(account::Column::ChallengeExpires.gt(now()))
        .one(db::db())
        .await?
        .ok_or_else(invalid)?;
    if !row.active || row.locked_until > now() {
        return Err(invalid());
    }
    let totp = authenticator(decrypt(&row.otp_secret)?, row.email.clone())?;
    let code = code.trim();
    let step = now() / 30;
    let accepted = (step - 1..=step + 1)
        .find(|s| *s > row.last_step && totp.check(code, (*s * 30) as u64).is_some());
    let recovery = digest(&code.to_ascii_uppercase());
    let mut hashes: Vec<String> = row.recovery_hashes.lines().map(str::to_string).collect();
    let recovered = row.verified && accepted.is_none() && hashes.contains(&recovery);
    if accepted.is_none() && !recovered {
        fail(&row).await?;
        return Err(invalid());
    }
    let mut recovery_codes = Vec::new();
    if !row.verified {
        recovery_codes = (0..8)
            .map(|_| {
                let mut bytes = [0; 10];
                SysRng
                    .try_fill_bytes(&mut bytes)
                    .expect("operating system randomness");
                bytes.iter().map(|b| format!("{b:02X}")).collect::<String>()
            })
            .collect();
        hashes = recovery_codes.iter().map(|c| digest(c)).collect();
    } else if recovered {
        hashes.retain(|h| h != &recovery);
    }
    // Consume the challenge and factor in the same conditional write: one winner under a race.
    let result = account::Entity::update_many()
        .col_expr(account::Column::Verified, Expr::value(true))
        .col_expr(
            account::Column::LastStep,
            Expr::value(accepted.unwrap_or(row.last_step)),
        )
        .col_expr(
            account::Column::RecoveryHashes,
            Expr::value(hashes.join("\n")),
        )
        .col_expr(account::Column::ChallengeHash, Expr::value(""))
        .col_expr(account::Column::Attempts, Expr::value(0))
        .filter(account::Column::Id.eq(&row.id))
        .filter(account::Column::ChallengeHash.eq(digest(&token)))
        .filter(account::Column::Active.eq(true))
        .filter(account::Column::SessionVersion.eq(row.session_version))
        .filter(account::Column::LastStep.eq(row.last_step))
        .filter(account::Column::LockedUntil.lte(now()))
        .filter(account::Column::ChallengeExpires.gt(now()))
        .exec(db::db())
        .await?;
    if result.rows_affected != 1 {
        return Err(invalid());
    }
    rahti::auth::sign_in(&PublicUser::from(&row));
    Ok(SignedIn { recovery_codes })
}

pub fn safe_next(value: &str) -> String {
    if value.starts_with('/')
        && !value.starts_with("//")
        && !value.contains(['\\', '\r', '\n'])
        && !value.chars().any(char::is_control)
        && !value.starts_with("/signin")
        && !value.starts_with("/register")
        && !value.starts_with("/setup")
    {
        value.to_string()
    } else {
        "/chat".into()
    }
}
