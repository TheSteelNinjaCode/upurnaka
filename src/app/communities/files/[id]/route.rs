//! Download one community file: membership is checked here, then the browser
//! is sent to a short-lived storage link.

use axum::extract::Path;
use axum::http::header;
use axum::response::{IntoResponse, Response};

pub async fn get(Path(id): Path<String>) -> crate::rahti::Result<Response> {
    let url = crate::communities::file_link(&id).await?;
    Ok((
        axum::http::StatusCode::FOUND,
        [
            (header::LOCATION, url),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
    )
        .into_response())
}
