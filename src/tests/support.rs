//! Shared infrastructure for the application test areas.
//!
//! No server starts and no port binds: `oneshot` drives the same
//! `axum::Router` the binary serves.

use axum::body::Body;
use axum::http::Request;
pub(super) use axum::http::StatusCode;
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Everything the suite installs once, before the first request.
///
/// Called from the request helpers rather than from each test, so a test
/// cannot forget it — and guarded, so the first request performs it and the
/// rest wait.
///
/// Installs the same auth, authorization, and response policies as startup,
/// with a signing key used only by this test binary.
pub(super) async fn setup() {
    static READY: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

    READY
        .get_or_init(|| async {
            crate::browser_policy::configure();
            let mut settings = crate::auth::settings();
            settings.secret = "upurnaka-test-only-signing-key-32-bytes".into();
            rahti::auth::configure(settings);
            rahti::authorization::configure(crate::auth::authorization());
            rahti::request_pipeline::configure(crate::middleware::pipeline());
            // A real database for the whole test binary: SQLite in a
            // file, with this application's own migrations run against it.
            // A file rather than `:memory:`, deleted on the way in —
            // docs/conventions/testing.md explains why both halves matter.
            let path = std::env::temp_dir().join("upurnaka-tests.db");
            let _ = std::fs::remove_file(&path);

            let url = format!(
                "sqlite://{}?mode=rwc",
                path.display().to_string().replace('\\', "/")
            );
            crate::db::connect_to(&url).await.expect("a test database");
            crate::db::migrate().await.expect("the migrations");
        })
        .await;
}

/// GET a URL through the real router.
pub(super) async fn get(url: &str) -> (StatusCode, String) {
    setup().await;

    let res = crate::routes::router()
        .oneshot(
            Request::builder()
                .uri(url)
                .header("cookie", test_cookie())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("the router answered");

    let status = res.status();
    let bytes = res.into_body().collect().await.expect("a body").to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// Call an `#[rpc]` the way the browser does: load its page, keep the CSRF
/// cookie the response hands out, then POST back to the same URL with the
/// function's name in `x-pp-function` and the token sent both ways.
pub(super) async fn rpc(url: &str, name: &str, payload: &str) -> (StatusCode, String) {
    setup().await;

    let router = crate::routes::router();

    let visit = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(url)
                .header("host", "127.0.0.1:3000")
                .header("cookie", test_cookie())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("the page answered");

    let cookie = visit
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .expect("the page hands out a csrf token")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let res = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(url)
                .header("host", "127.0.0.1:3000")
                .header("x-pp-function", name)
                .header("content-type", "application/json")
                .header("cookie", format!("{}; {}", test_cookie(), cookie))
                .header("x-csrf-token", cookie.split_once('=').unwrap().1)
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .expect("the rpc answered");

    let status = res.status();
    let bytes = res.into_body().collect().await.expect("a body").to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// Existing application tests operate as one signed-in fixture. Authentication tests use the
/// public registration/sign-in endpoints instead, and never use this signing helper.
pub(super) fn test_cookie() -> String {
    cookie_for(
        "legacy-test-user",
        "Jefferson Abraham",
        "fixture@example.com",
    )
}

pub(super) fn cookie_for(id: &str, name: &str, email: &str) -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use hmac::{Hmac, KeyInit, Mac};
    let settings = rahti::auth::settings();
    let payload = rahti::serde_json::json!({"user": {"id": id, "name": name, "email": email},
        "exp": rahti::time::Utc::now().timestamp() + 3600});
    let body = URL_SAFE_NO_PAD.encode(payload.to_string());
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(settings.secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    format!(
        "{}={}.{}",
        settings.cookie,
        body,
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    )
}

pub(super) async fn authenticate_browser(
    browser: &mut rahti::browser::Browser,
    address: std::net::SocketAddr,
) {
    browser
        .goto(&format!("http://{address}/signin"))
        .await
        .unwrap();
    let cookie = format!("{}; Path=/; SameSite=Lax", test_cookie());
    let _: String = browser
        .eval(&format!(
            "document.cookie = {}",
            rahti::serde_json::to_string(&cookie).unwrap()
        ))
        .await
        .unwrap();
}
