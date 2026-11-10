use super::*;
use crate::models::account;
use axum::{
    body::Body,
    http::{Request, header::SET_COOKIE},
};
use http_body_util::BodyExt;
use rahti::serde_json::{self, Value, json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use tower::ServiceExt;

#[derive(Clone, Default)]
pub(super) struct Client {
    cookies: std::collections::BTreeMap<String, String>,
}
impl Client {
    pub(super) async fn request(
        &mut self,
        path: &str,
        rpc: Option<(&str, Value)>,
    ) -> (StatusCode, axum::http::HeaderMap, String) {
        let rpc = rpc.map(|(name, data)| {
            (
                name,
                "application/json".to_string(),
                data.to_string().into_bytes(),
            )
        });
        self.send(path, rpc).await
    }
    /// An rpc carrying one file, on the multipart wire `pp.rpc` uses: every
    /// ordinary field first, the file last.
    pub(super) async fn upload(
        &mut self,
        path: &str,
        name: &str,
        fields: Value,
        file: (&str, &str, &[u8]),
    ) -> (StatusCode, String) {
        let (field, content_type, bytes) = file;
        let boundary = "upurnaka-test-boundary";
        let mut body = Vec::new();
        for (key, value) in fields.as_object().unwrap() {
            let value = value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            body.extend(
                format!(
                    "--{boundary}\r\ncontent-disposition: form-data; name=\"{key}\"\r\n\r\n{value}\r\n"
                )
                .into_bytes(),
            );
        }
        body.extend(
            format!(
                "--{boundary}\r\ncontent-disposition: form-data; name=\"{field}\"; filename=\"piece\"\r\ncontent-type: {content_type}\r\n\r\n"
            )
            .into_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());
        let content_type = format!("multipart/form-data; boundary={boundary}");
        let (status, _, body) = self.send(path, Some((name, content_type, body))).await;
        (status, body)
    }
    async fn send(
        &mut self,
        path: &str,
        rpc: Option<(&str, String, Vec<u8>)>,
    ) -> (StatusCode, axum::http::HeaderMap, String) {
        setup().await;
        let mut req = Request::builder()
            .uri(path)
            .header("host", "127.0.0.1:3000")
            .header(
                "cookie",
                self.cookies
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("; "),
            );
        let body = if let Some((name, content_type, data)) = rpc {
            let csrf = self
                .cookies
                .iter()
                .find(|(key, _)| key.contains("csrf"))
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            req = req
                .method("POST")
                .header("x-pp-function", name)
                .header("x-csrf-token", csrf)
                .header("content-type", content_type);
            Body::from(data)
        } else {
            Body::empty()
        };
        let response = crate::routes::router()
            .oneshot(req.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        for header in headers.get_all(SET_COOKIE) {
            let pair = header.to_str().unwrap().split(';').next().unwrap();
            let (key, value) = pair.split_once('=').unwrap();
            if value.is_empty() {
                self.cookies.remove(key);
            } else {
                self.cookies.insert(key.into(), value.into());
            }
        }
        let body = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        (status, headers, body)
    }
    pub(super) async fn new() -> Self {
        let mut c = Self::default();
        c.request("/signin", None).await;
        c
    }
    pub(super) async fn call(&mut self, name: &str, data: Value) -> (StatusCode, Value) {
        let (s, _, b) = self.request("/signin", Some((name, data))).await;
        (
            s,
            serde_json::from_str(&b).unwrap_or_else(|_| json!({"raw": b})),
        )
    }
    pub(super) async fn register(&mut self, email: &str) -> Value {
        let (s, body) = self
            .call(
                "register_account",
                json!({"name":"Ada Lovelace", "email": email,
            "password":"a long unique password", "confirmation":"a long unique password"}),
            )
            .await;
        assert_eq!(s, StatusCode::OK, "{body}");
        body
    }
    async fn start(&mut self, email: &str) -> Value {
        let (s, body) = self
            .call(
                "start_signin",
                json!({"email":email, "password":"a long unique password"}),
            )
            .await;
        assert_eq!(s, StatusCode::OK, "{body}");
        body
    }
}
impl Client {
    pub(super) fn with_cookie(&mut self, cookie: String) {
        let (key, value) = cookie.split_once('=').unwrap();
        self.cookies.insert(key.into(), value.into());
    }
}
pub(super) fn otp(setup: &Value, email: &str) -> String {
    let secret = totp_rs::Secret::try_from_base32(setup["setup_key"].as_str().unwrap())
        .unwrap()
        .as_bytes()
        .to_vec();
    crate::auth::authenticator(secret, email.into())
        .unwrap()
        .generate_current()
        .to_string()
}
async fn account(email: &str) -> account::Model {
    account::Entity::find()
        .filter(account::Column::Email.eq(email))
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn registration_requires_otp_and_recovery_codes_are_single_use() {
    let mut c = Client::new().await;
    let email = "roundtrip-auth@example.com";
    let setup = c.register(" ROUNDTRIP-auth@example.com ").await;
    assert!(
        setup["qr"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
    assert!(!c.cookies.contains_key(&rahti::auth::settings().cookie));
    let row = account(email).await;
    assert!(!row.verified);
    assert!(row.password_hash.starts_with("$argon2id$"));
    assert!(
        !row.otp_secret
            .contains(setup["setup_key"].as_str().unwrap())
    );
    assert_eq!(c.request("/chat", None).await.0, StatusCode::SEE_OTHER);
    let (s, _) = c
        .call(
            "verify_signin",
            json!({"token":setup["token"], "code":"0000000"}),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let code = otp(&setup, email);
    let (s, verified) = c
        .call(
            "verify_signin",
            json!({"token":setup["token"], "code":code}),
        )
        .await;
    assert_eq!(s, StatusCode::OK, "{verified}");
    assert_eq!(verified["recovery_codes"].as_array().unwrap().len(), 8);
    let recovery = verified["recovery_codes"][0].as_str().unwrap();
    assert!(!account(email).await.recovery_hashes.contains(recovery));
    let (s, headers, body) = c.request("/account", None).await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains("Ada Lovelace"));
    assert!(body.contains(email));
    assert!(!body.contains("$argon2id$"));
    assert_eq!(headers["cache-control"], "no-store");
    let (s, replay) = c
        .call(
            "verify_signin",
            json!({"token":setup["token"], "code":code}),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "{replay}");
    let (s, headers, _) = c
        .request("/account", Some(("sign_out_account", json!({}))))
        .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        headers
            .get_all(SET_COOKIE)
            .iter()
            .any(|v| v.to_str().unwrap().contains("Max-Age=0"))
    );
    assert_eq!(c.request("/chat", None).await.0, StatusCode::SEE_OTHER);
    let login = c.start(email).await;
    assert!(login["setup_key"].is_null());
    assert!(login["qr"].is_null());
    assert!(!c.cookies.contains_key(&rahti::auth::settings().cookie));
    // A TOTP already used to enroll cannot be used again in another challenge.
    let (s, _) = c
        .call(
            "verify_signin",
            json!({"token":login["token"], "code":code}),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, b) = c
        .call(
            "verify_signin",
            json!({"token":login["token"], "code":recovery.to_lowercase()}),
        )
        .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["recovery_codes"], json!([]));
    assert_eq!(account(email).await.recovery_hashes.lines().count(), 7);
    let login = c.start(email).await;
    let (s, _) = c
        .call(
            "verify_signin",
            json!({"token":login["token"], "code":recovery}),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn pending_enrollment_can_resume_and_expired_challenges_fail() {
    let email = "pending-auth@example.com";
    let mut c = Client::new().await;
    let setup = c.register(email).await;
    let resume = c.start(email).await;
    assert_eq!(setup["setup_key"], resume["setup_key"]);
    let (s, _) = c
        .call(
            "verify_signin",
            json!({"token":setup["token"],"code":otp(&setup, email)}),
        )
        .await;
    assert_eq!(
        s,
        StatusCode::BAD_REQUEST,
        "old challenges must be replaced"
    );
    account::Entity::update_many()
        .col_expr(account::Column::ChallengeExpires, Expr::value(0))
        .filter(account::Column::Email.eq(email))
        .exec(crate::db::db())
        .await
        .unwrap();
    let (s, _) = c
        .call(
            "verify_signin",
            json!({"token":resume["token"],"code":otp(&resume, email)}),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let resume = c.start(email).await;
    assert_eq!(
        c.call(
            "verify_signin",
            json!({"token":resume["token"],"code":otp(&resume, email)})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn accounts_from_the_previous_crypto_versions_can_still_sign_in() {
    let mut client = Client::new().await;
    let email = "legacy-crypto@example.com";
    client.register(email).await;
    // Synthetic fixtures generated by Argon2 0.5.3 and AES-GCM 0.10.3 using
    // the test-only signing key. No live account credentials are used here.
    let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$huepRidAXG3KCEoGMRcLHg$mX24QGfCrloqhZZnH+r2ysXOZO4kJpOXzXk4k+JY1g0";
    let encrypted_secret = "mkZ3kz3KWkkAfdrolFvEcswoiqeaPT-N45aIPRcyI2JYZI_irhV-TG3KK0ssgCsN";
    account::Entity::update_many()
        .col_expr(account::Column::PasswordHash, Expr::value(password_hash))
        .col_expr(account::Column::OtpSecret, Expr::value(encrypted_secret))
        .filter(account::Column::Email.eq(email))
        .exec(crate::db::db())
        .await
        .unwrap();
    let challenge = client.start(email).await;
    let (status, result) = client
        .call(
            "verify_signin",
            json!({
                "token":challenge["token"], "code":otp(&challenge, email)
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(client.request("/account", None).await.0, StatusCode::OK);
}

#[tokio::test]
async fn bad_passwords_and_bad_codes_share_a_persistent_attempt_limit() {
    let email = "locked-auth@example.com";
    let mut c = Client::new().await;
    let setup = c.register(email).await;
    let (unknown_s, unknown) = c
        .call(
            "start_signin",
            json!({"email":"unknown-auth@example.com","password":"wrong password"}),
        )
        .await;
    for _ in 0..3 {
        let (s, b) = c
            .call(
                "start_signin",
                json!({"email":email,"password":"wrong password"}),
            )
            .await;
        assert_eq!(s, unknown_s);
        assert_eq!(b["error"], unknown["error"]);
    }
    // Starting a new valid-password challenge must not reset the failed OTP budget.
    let login = c.start(email).await;
    for _ in 0..2 {
        assert_eq!(
            c.call(
                "verify_signin",
                json!({"token":login["token"],"code":"bad-code"})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert!(account(email).await.locked_until > rahti::time::Utc::now().timestamp());
    assert_eq!(
        c.call(
            "start_signin",
            json!({"email":email,"password":"a long unique password"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        c.call(
            "verify_signin",
            json!({"token":setup["token"],"code":otp(&setup, email)})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    account::Entity::update_many()
        .col_expr(account::Column::LockedUntil, Expr::value(0))
        .filter(account::Column::Email.eq(email))
        .exec(crate::db::db())
        .await
        .unwrap();
    assert!(!c.start(email).await["setup_key"].is_null());
}

#[tokio::test]
async fn auth_validates_input_and_all_private_endpoints_require_sessions() {
    let mut c = Client::new().await;
    for name in ["register_account", "start_signin", "verify_signin"] {
        assert_eq!(c.call(name, json!({})).await.0, StatusCode::BAD_REQUEST);
    }
    let (s, b) = c
        .call(
            "register_account",
            json!({"name":"", "email":"invalid", "password":"short", "confirmation":"different"}),
        )
        .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY, "{b}");
    for path in [
        "/",
        "/chat",
        "/meet",
        "/people",
        "/calendar",
        "/activity",
        "/communities",
        "/apps",
        "/account",
        "/files",
    ] {
        let (s, headers, _) = c.request(path, None).await;
        assert_eq!(s, StatusCode::SEE_OTHER, "{path}");
        assert!(
            headers["location"]
                .to_str()
                .unwrap()
                .starts_with("/signin?next=")
        );
    }
    for (path, name) in [
        ("/meet", "create_meeting_link"),
        ("/chat", "send_message"),
        ("/signin", "sign_out_account"),
        ("/signin", "save_usage_survey"),
    ] {
        assert_eq!(
            c.request(path, Some((name, json!({})))).await.0,
            StatusCode::UNAUTHORIZED,
            "{path}/{name}"
        );
    }
    assert_eq!(c.request("/register", None).await.0, StatusCode::OK);
    c.register("duplicate-auth@example.com").await;
    let (s, _) = c.call("register_account", json!({"name":"Other", "email":" DUPLICATE-auth@example.com ", "password":"a long unique password", "confirmation":"a long unique password"})).await;
    assert_eq!(s, StatusCode::CONFLICT);
    for bad in [
        "//evil.test",
        "https://evil.test",
        "/\\evil.test",
        "/signin",
        "/register",
        "/\tevil.test",
    ] {
        assert_eq!(crate::auth::safe_next(bad), "/chat");
    }
    assert_eq!(
        crate::auth::safe_next("/calendar?day=1#meeting"),
        "/calendar?day=1#meeting"
    );
}

#[tokio::test]
async fn meeting_lists_and_mutations_are_scoped_to_the_owner() {
    setup().await;
    let (s, b) = rpc(
        "/meet",
        "create_meeting_link",
        r#"{"title":"Owner-only meeting"}"#,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let id = serde_json::from_str::<Value>(&b).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut other = Client::new().await;
    let cookie = super::support::cookie_for("other-user", "Other User", "other@example.com");
    let (key, value) = cookie.split_once('=').unwrap();
    other.cookies.insert(key.into(), value.into());
    let (_, _, body) = other.request("/meet", None).await;
    assert!(!body.contains("Owner-only meeting"));
    for (name, data) in [
        ("rename_meeting_link", json!({"id":id,"title":"Stolen"})),
        ("delete_meeting_link", json!({"id":id})),
    ] {
        assert_eq!(
            other.request("/meet", Some((name, data))).await.0,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        crate::meetings::find(&id).await.unwrap().unwrap().title,
        "Owner-only meeting"
    );
}

#[tokio::test]
async fn a_challenge_has_only_one_winner_under_parallel_verification() {
    let mut first = Client::new().await;
    let setup = first.register("parallel-auth@example.com").await;
    let mut second = first.clone();
    let payload = json!({"token":setup["token"], "code":otp(&setup, "parallel-auth@example.com")});
    let (a, b) = tokio::join!(
        first.call("verify_signin", payload.clone()),
        second.call("verify_signin", payload)
    );
    assert!(
        (a.0 == StatusCode::OK && b.0 == StatusCode::BAD_REQUEST)
            || (b.0 == StatusCode::OK && a.0 == StatusCode::BAD_REQUEST)
    );
}

#[tokio::test]
async fn a_browser_can_register_enroll_save_codes_and_sign_out() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    browser
        .goto(&format!("http://{address}/register?next=/account"))
        .await
        .unwrap();
    for (name, value) in [
        ("name", "Browser User"),
        ("email", "browser-auth@example.com"),
        ("password", "a long unique password"),
        ("confirmation", "a long unique password"),
    ] {
        browser
            .click(&format!("input[name='{name}']"))
            .await
            .unwrap();
        browser.type_text(value).await.unwrap();
    }
    browser.click("form button[type=submit]").await.unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('h1')).some(h => h.textContent.includes('Set up your authenticator') && h.getBoundingClientRect().height > 0)").await.unwrap();
    let key: String = browser
        .eval("document.querySelector('code').textContent")
        .await
        .unwrap();
    let code = otp(&json!({"setup_key":key}), "browser-auth@example.com");
    browser
        .wait_for("document.activeElement === document.querySelector('input[name=code]')")
        .await
        .unwrap();
    browser.type_text(&code).await.unwrap();
    browser
        .click("form:has(input[name=code]) button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('h1')).some(h => h.textContent.includes('Save your recovery codes') && h.getBoundingClientRect().height > 0)").await.unwrap();
    assert_eq!(
        browser
            .eval::<usize>("document.querySelectorAll('code').length")
            .await
            .unwrap(),
        9
    );
    let recovery: String = browser.eval("Array.from(document.querySelectorAll('code')).find(c => c.textContent.length === 20).textContent").await.unwrap();
    browser.click("input[type=checkbox]").await.unwrap();
    browser
        .click("section > div:nth-of-type(3) button")
        .await
        .unwrap();
    browser.wait_for("location.pathname === '/account' && document.querySelector('main').textContent.includes('Browser User')").await.unwrap();
    browser
        .click("button[aria-label='Your profile']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Your profile\"]').getAttribute('aria-expanded') === 'true'").await.unwrap();
    browser
        .click("[role=menuitem][data-variant=destructive]")
        .await
        .unwrap();
    browser
        .wait_for("location.pathname === '/signin'")
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .wait_for("location.pathname === '/signin'")
        .await
        .unwrap();
    for (name, value) in [
        ("email", "browser-auth@example.com"),
        ("password", "a long unique password"),
    ] {
        browser
            .click(&format!("input[name='{name}']"))
            .await
            .unwrap();
        browser.type_text(value).await.unwrap();
    }
    browser.click("form button[type=submit]").await.unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('h1')).some(h => h.textContent.includes('Verify it’s you') && h.getBoundingClientRect().height > 0)").await.unwrap();
    browser
        .wait_for("document.activeElement === document.querySelector('input[name=code]')")
        .await
        .unwrap();
    browser
        .click("section > div:nth-of-type(2) > div > button:first-child")
        .await
        .unwrap();
    browser
        .wait_for("document.activeElement === document.querySelector('input[name=code]') && document.activeElement.inputMode === 'text'")
        .await
        .unwrap();
    browser.type_text(&recovery).await.unwrap();
    browser
        .click("form:has(input[name=code]) button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("location.pathname === '/chat' && document.querySelector('header button[aria-label=\"Your profile\"]').title === 'Browser User'").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
