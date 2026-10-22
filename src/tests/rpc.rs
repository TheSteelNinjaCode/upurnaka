use super::*;

/// `pp.rpc("hello", {})` from the scaffolded page, end to end: the CSRF
/// pair accepted, the function found, the return value serialized.
#[tokio::test]
async fn the_hello_rpc_answers() {
    let (status, body) = rpc("/", "hello", "{}").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Hello from Rust."), "{body}");
}

/// A name the page does not declare is refused by name.
#[tokio::test]
async fn an_unknown_rpc_name_is_refused() {
    let (status, _) = rpc("/", "no_such_function", "{}").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
