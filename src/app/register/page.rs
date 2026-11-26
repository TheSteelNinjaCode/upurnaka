use crate::{
    components::auth_form::AuthForm,
    rahti::{Metadata, html},
};
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    pub next: Option<String>,
}
pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Create account")
        .description("Create your upurnaka account")
}
/// Nobody registers before the administrator exists: that comes first, on `/setup`.
pub async fn page(Query(params): Query<Params>) -> Response {
    if crate::auth::needs_setup().await {
        return Redirect::to("/setup").into_response();
    }
    let next = crate::auth::safe_next(params.next.as_deref().unwrap_or("/chat"));
    html! {
        <AuthForm mode="register" next=@{next.as_str()} />
    }
    .into_response()
}
