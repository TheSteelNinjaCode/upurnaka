use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use rahti::request_pipeline::{Next, RequestPipeline};
use sea_orm::EntityTrait;

async fn private_responses(request: Request, next: Next) -> Response {
    let mut response = check_account(request, next).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
}

async fn check_account(request: Request, next: Next) -> Response {
    let Some(session) = rahti::auth::user::<crate::auth::PublicUser>() else {
        return crate::auth::REQUEST_USER
            .scope(None, next.run(request))
            .await;
    };
    if !crate::db::is_connected() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(rahti::serde_json::json!({"error":"Accounts are temporarily unavailable."})),
        )
            .into_response();
    }
    let row = match crate::models::account::Entity::find_by_id(&session.id)
        .one(crate::db::db())
        .await
    {
        Ok(Some(row))
            if row.active && row.verified && row.session_version == session.session_version =>
        {
            row
        }
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(
                    rahti::serde_json::json!({"error":"Accounts are temporarily unavailable."}),
                ),
            )
                .into_response();
        }
        _ => {
            rahti::auth::sign_out();
            return if request.headers().contains_key("x-pp-function") {
                (StatusCode::UNAUTHORIZED, axum::Json(rahti::serde_json::json!({"error":"Your account is unavailable. Please sign in again."}))).into_response()
            } else {
                Redirect::to("/signin").into_response()
            };
        }
    };
    crate::auth::REQUEST_USER
        .scope(Some(crate::auth::PublicUser::from(&row)), next.run(request))
        .await
}

pub fn pipeline() -> RequestPipeline {
    RequestPipeline::new().with(private_responses)
}
