use crate::{
    components::auth_form::AuthForm,
    rahti::{Metadata, html},
};
use axum::response::{IntoResponse, Redirect, Response};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Set up upurnaka")
        .description("Create the administrator account for this upurnaka workspace")
}

/// First launch: create the administrator. Only while no administrator
/// exists; after that this page sends everyone to `/signin`.
pub async fn page() -> Response {
    if !crate::auth::needs_setup().await {
        return Redirect::to("/signin").into_response();
    }
    html! {
        <AuthForm mode="setup" next="/users" />
    }
    .into_response()
}
