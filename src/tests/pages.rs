use super::*;

/// The scaffolded home page, inside the root layout's document.
#[tokio::test]
async fn the_home_page_renders() {
    let (status, body) = get("/").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("<!DOCTYPE html>"), "{body}");
    assert!(body.contains("upurnaka"), "{body}");
}

/// A URL no `page.rs` claims: the 404 page, not a panic.
#[tokio::test]
async fn an_unknown_url_is_a_404() {
    let (status, _) = get("/this-page-does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
