//! Account administration. The database, never the cookie or payload, supplies authority.
use crate::{
    auth::{self, PublicUser, Role},
    db,
    models::{account, meeting_link},
    rahti::{Error, Result, Validator},
};
use axum::http::StatusCode;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, ExprTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
    sea_query::{Expr, Func},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct UserSummary {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    pub enrolled: bool,
}
impl From<account::Model> for UserSummary {
    fn from(row: account::Model) -> Self {
        Self {
            id: row.id,
            name: row.name,
            email: row.email,
            role: row.role,
            active: row.active,
            enrolled: row.verified,
        }
    }
}
#[derive(Serialize)]
pub struct UserList {
    pub items: Vec<UserSummary>,
    pub total: u64,
    pub page: u64,
}

pub async fn require_admin() -> Result<PublicUser> {
    auth::ready()?;
    let session = rahti::auth::user::<PublicUser>()
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Authentication required"))?;
    let row = account::Entity::find_by_id(&session.id)
        .one(db::db())
        .await?
        .filter(|row| row.active && row.verified && row.session_version == session.session_version)
        .ok_or_else(|| {
            Error::new(
                StatusCode::UNAUTHORIZED,
                "Your account is unavailable. Please sign in again.",
            )
        })?;
    let live = PublicUser::from(&row);
    rahti::authorization::authorize_resource("users.manage", &live)?;
    Ok(live)
}

pub async fn list(search: String, page: u64) -> Result<UserList> {
    require_admin().await?;
    let mut v = Validator::new();
    v.max_chars("search", &search, 254)
        .max_value("page", &page, 1_000_000_u64);
    v.finish()?;
    let search = search.trim().to_lowercase();
    let query = account::Entity::find().filter(
        Condition::any()
            .add(account::Column::Email.contains(&search))
            .add(Func::lower(Expr::col(account::Column::Name)).like(format!("%{search}%"))),
    );
    let total = query.clone().count(db::db()).await?;
    let page = std::cmp::min(page, total.saturating_sub(1) / 25);
    let items = query
        .order_by_asc(account::Column::Email)
        .offset(page * 25)
        .limit(25)
        .all(db::db())
        .await?
        .into_iter()
        .map(UserSummary::from)
        .collect();
    Ok(UserList { items, total, page })
}

pub async fn create(
    name: String,
    email: String,
    password: String,
    confirmation: String,
) -> Result<UserSummary> {
    require_admin().await?;
    // No OTP key is returned to the administrator. The user enrolls on their first sign-in.
    Ok(
        auth::create_account(name, email, password, confirmation, Role::User)
            .await?
            .into(),
    )
}

fn mutable_user(id: &str) -> sea_orm::Select<account::Entity> {
    account::Entity::find_by_id(id).filter(account::Column::Role.eq("user"))
}
fn missing() -> Error {
    Error::new(
        StatusCode::NOT_FOUND,
        "User not found, or this account is the protected administrator.",
    )
}
fn duplicate(error: sea_orm::DbErr) -> Error {
    if error
        .sql_err()
        .is_some_and(|e| matches!(e, sea_orm::SqlErr::UniqueConstraintViolation(_)))
    {
        Error::new(
            StatusCode::CONFLICT,
            "That email address is already registered.",
        )
    } else {
        Error::from(error)
    }
}

pub async fn update(id: String, name: String, email: String) -> Result<UserSummary> {
    require_admin().await?;
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    let mut v = Validator::new();
    v.required("id", &id)
        .max_chars("id", &id, 100)
        .required("name", &name)
        .max_chars("name", &name, 80)
        .required("email", &email)
        .email("email", &email)
        .max_chars("email", &email, 254);
    v.finish()?;
    let result = account::Entity::update_many()
        .col_expr(account::Column::Name, Expr::value(name))
        .col_expr(account::Column::Email, Expr::value(email))
        .col_expr(
            account::Column::SessionVersion,
            Expr::col(account::Column::SessionVersion).add(1),
        )
        .col_expr(account::Column::ChallengeHash, Expr::value(""))
        .filter(account::Column::Id.eq(&id))
        .filter(account::Column::Role.eq("user"))
        .exec(db::db())
        .await
        .map_err(duplicate)?;
    if result.rows_affected != 1 {
        return Err(missing());
    }
    Ok(mutable_user(&id)
        .one(db::db())
        .await?
        .ok_or_else(missing)?
        .into())
}

pub async fn set_active(id: String, active: bool) -> Result<UserSummary> {
    require_admin().await?;
    let result = account::Entity::update_many()
        .col_expr(account::Column::Active, Expr::value(active))
        .col_expr(
            account::Column::SessionVersion,
            Expr::col(account::Column::SessionVersion).add(1),
        )
        .col_expr(account::Column::ChallengeHash, Expr::value(""))
        .filter(account::Column::Id.eq(&id))
        .filter(account::Column::Role.eq("user"))
        .exec(db::db())
        .await?;
    if result.rows_affected != 1 {
        return Err(missing());
    }
    Ok(mutable_user(&id)
        .one(db::db())
        .await?
        .ok_or_else(missing)?
        .into())
}

pub async fn delete(id: String) -> Result<bool> {
    require_admin().await?;
    let transaction = db::db().begin().await?;
    let result = account::Entity::delete_many()
        .filter(account::Column::Id.eq(&id))
        .filter(account::Column::Role.eq("user"))
        .exec(&transaction)
        .await?;
    if result.rows_affected != 1 {
        return Err(missing());
    }
    meeting_link::Entity::update_many()
        .col_expr(
            meeting_link::Column::RevokedAt,
            Expr::value(Some(chrono::Utc::now())),
        )
        .filter(meeting_link::Column::OwnerId.eq(&id))
        .filter(meeting_link::Column::RevokedAt.is_null())
        .exec(&transaction)
        .await?;
    transaction.commit().await?;
    Ok(true)
}

/// Seed-only entry point. Never exposed as an RPC and never promotes an existing user.
pub async fn seed_admin(name: String, email: String, password: String) -> Result<PublicUser> {
    auth::ready()?;
    let email = email.trim().to_lowercase();
    if let Some(admin) = account::Entity::find()
        .filter(account::Column::Role.eq("admin"))
        .one(db::db())
        .await?
    {
        if admin.email == email {
            return Ok(PublicUser::from(&admin));
        }
        return Err(Error::new(
            StatusCode::CONFLICT,
            "The single administrator has already been seeded.",
        ));
    }
    Ok(PublicUser::from(
        &auth::create_account(name, email, password.clone(), password, Role::Admin).await?,
    ))
}
