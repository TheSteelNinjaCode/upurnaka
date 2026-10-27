use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sea_orm::EntityTrait;

pub async fn get() -> crate::rahti::Result<Response> {
    let user = crate::auth::current().ok_or_else(|| {
        crate::rahti::Error::new(StatusCode::UNAUTHORIZED, "Authentication required")
    })?;
    let row = crate::models::account::Entity::find_by_id(user.id)
        .one(crate::db::db())
        .await?;
    Ok(match row.and_then(|row| row.picture) {
        Some(bytes) => (
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            bytes,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    })
}
