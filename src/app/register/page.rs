use crate::{
    components::auth_form::AuthForm,
    rahti::{Html, Metadata, html},
};
use axum::extract::Query;
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
pub async fn page(Query(params): Query<Params>) -> Html {
    let next = crate::auth::safe_next(params.next.as_deref().unwrap_or("/chat"));
    html! {
        <AuthForm register=@{true} next=@{next.as_str()} />
    }
}
