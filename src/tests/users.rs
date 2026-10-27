use super::auth::Client;
use super::*;
use crate::{auth::Role, models::account};
use rahti::serde_json::{Value, json};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};

async fn admin() -> Client {
    let mut client = Client::new().await;
    client.with_cookie(super::support::cookie_for(
        "test-admin",
        "Test Administrator",
        "admin-test@example.com",
    ));
    client
}
async fn call(client: &mut Client, name: &str, data: Value) -> (StatusCode, Value) {
    let (status, _, body) = client.request("/users", Some((name, data))).await;
    (
        status,
        rahti::serde_json::from_str(&body).unwrap_or_else(|_| json!({"raw":body})),
    )
}
async fn create(client: &mut Client, email: &str) -> Value {
    let (s, b) = call(
        client,
        "create_user",
        json!({"name":"Managed Person","email":email,
        "password":"a long unique password","confirmation":"a long unique password"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    b
}

#[tokio::test]
async fn only_the_database_admin_can_use_users_page_and_every_crud_rpc() {
    let mut regular = Client::new().await;
    regular.with_cookie(super::support::signed_cookie(json!({"id":"legacy-test-user","name":"Forged Admin", "email":"fixture@example.com","role":"admin"})));
    assert_eq!(
        regular.request("/users", None).await.0,
        StatusCode::FORBIDDEN
    );
    for (name, payload) in [
        ("list_users", json!({"search":"","page":0})),
        (
            "create_user",
            json!({"name":"Intruder","email":"intruder@example.com","password":"a long unique password","confirmation":"a long unique password","role":"admin"}),
        ),
        (
            "update_user",
            json!({"id":"test-admin","name":"Changed","email":"changed@example.com"}),
        ),
        ("set_user_active", json!({"id":"test-admin","active":false})),
        ("delete_user", json!({"id":"test-admin"})),
    ] {
        assert_eq!(
            call(&mut regular, name, payload.clone()).await.0,
            StatusCode::FORBIDDEN,
            "{name}"
        );
        let mut anonymous = Client::new().await;
        assert_eq!(
            call(&mut anonymous, name, payload).await.0,
            StatusCode::UNAUTHORIZED,
            "{name}"
        );
    }
    let mut admin = admin().await;
    let (s, _, body) = admin.request("/users", None).await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains("Manage accounts and access to your workspace."));
    assert!(!body.contains("password_hash"));
    assert!(!body.contains("otp_secret"));
    assert!(
        !regular
            .request("/chat", None)
            .await
            .2
            .contains("title=\"Users\"")
    );
    assert!(
        admin
            .request("/chat", None)
            .await
            .2
            .contains("title=\"Users\"")
    );
}

#[tokio::test]
async fn admin_crud_validates_and_protects_the_single_administrator() {
    let mut admin = admin().await;
    let created = create(&mut admin, "managed-crud@example.com").await;
    assert_eq!(created["role"], "user");
    assert_eq!(created["active"], true);
    assert_eq!(created["enrolled"], false);
    assert!(created.get("setup_key").is_none());
    let id = created["id"].as_str().unwrap();
    let (s,edited) = call(&mut admin,"update_user",json!({"id":id,"name":"Edited Person","email":" MANAGED-edited@example.com ","role":"admin"})).await;
    assert_eq!(s, StatusCode::OK, "{edited}");
    assert_eq!(edited["email"], "managed-edited@example.com");
    assert_eq!(edited["role"], "user");
    let (s, list) = call(
        &mut admin,
        "list_users",
        json!({"search":"managed-edited","page":0}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(list["total"], 1);
    assert_eq!(list["items"][0]["name"], "Edited Person");
    assert_eq!(
        call(
            &mut admin,
            "set_user_active",
            json!({"id":id,"active":false})
        )
        .await
        .1["active"],
        false
    );
    assert_eq!(
        call(
            &mut admin,
            "set_user_active",
            json!({"id":id,"active":true})
        )
        .await
        .1["active"],
        true
    );
    for name in [
        "list_users",
        "create_user",
        "update_user",
        "set_user_active",
        "delete_user",
    ] {
        assert_eq!(
            call(&mut admin, name, json!({})).await.0,
            StatusCode::BAD_REQUEST,
            "{name}"
        );
    }
    assert_eq!(
        call(
            &mut admin,
            "create_user",
            json!({"name":"Blank","email":"bad","password":"short","confirmation":"wrong"})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &mut admin,
            "update_user",
            json!({"id":id,"name":"","email":"bad"})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &mut admin,
            "update_user",
            json!({"id":id,"name":"Duplicate","email":"fixture@example.com"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    for (name, payload) in [
        (
            "update_user",
            json!({"id":"test-admin","name":"Changed","email":"changed@example.com"}),
        ),
        ("set_user_active", json!({"id":"test-admin","active":false})),
        ("delete_user", json!({"id":"test-admin"})),
    ] {
        assert_eq!(
            call(&mut admin, name, payload).await.0,
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        call(&mut admin, "delete_user", json!({"id":id})).await.0,
        StatusCode::OK
    );
    assert!(
        account::Entity::find_by_id(id)
            .one(crate::db::db())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        call(&mut admin, "delete_user", json!({"id":id})).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn inactive_deleted_and_revoked_sessions_cannot_access_the_workspace() {
    let mut admin = admin().await;
    let created = create(&mut admin, "access-toggle@example.com").await;
    let id = created["id"].as_str().unwrap();
    use sea_orm::sea_query::Expr;
    account::Entity::update_many()
        .col_expr(account::Column::Verified, Expr::value(true))
        .filter(account::Column::Id.eq(id))
        .exec(crate::db::db())
        .await
        .unwrap();
    let old_cookie = super::support::cookie_for(id, "Managed Person", "access-toggle@example.com");
    let mut user = Client::new().await;
    user.with_cookie(old_cookie.clone());
    assert_eq!(user.request("/chat", None).await.0, StatusCode::OK);
    call(
        &mut admin,
        "set_user_active",
        json!({"id":id,"active":false}),
    )
    .await;
    let (_, headers, _) = user.request("/chat", None).await;
    assert_eq!(headers["location"], "/signin");
    user.with_cookie(old_cookie.clone());
    assert_eq!(
        user.request("/chat", Some(("send_message", json!({"text":"Blocked"}))))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let mut signin = Client::new().await;
    assert_eq!(
        signin
            .call(
                "start_signin",
                json!({"email":"access-toggle@example.com","password":"a long unique password"})
            )
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    call(
        &mut admin,
        "set_user_active",
        json!({"id":id,"active":true}),
    )
    .await;
    user.with_cookie(old_cookie);
    assert_eq!(
        user.request("/chat", None).await.0,
        StatusCode::SEE_OTHER,
        "reactivation must not revive an old session"
    );
    assert_eq!(
        signin
            .call(
                "start_signin",
                json!({"email":"access-toggle@example.com","password":"a long unique password"})
            )
            .await
            .0,
        StatusCode::OK
    );
    let version = account::Entity::find_by_id(id)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap()
        .session_version;
    user.with_cookie(super::support::signed_cookie(json!({"id":id,"name":"Managed Person","email":"access-toggle@example.com","session_version":version})));
    assert_eq!(user.request("/chat", None).await.0, StatusCode::OK);
    let (_, _, body) = user
        .request(
            "/chat",
            Some((
                "create_chat_meeting",
                json!({"title":"Deleted user meeting"}),
            )),
        )
        .await;
    let meeting_id = rahti::serde_json::from_str::<Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    call(&mut admin, "delete_user", json!({"id":id})).await;
    assert_eq!(user.request("/chat", None).await.0, StatusCode::SEE_OTHER);
    assert!(crate::meetings::find(&meeting_id).await.unwrap().is_none());
}

#[tokio::test]
async fn public_registration_ignores_role_and_active_escalation_and_seed_is_idempotent() {
    let mut c = Client::new().await;
    let (s,b) = c.call("register_account",json!({"name":"Public User","email":"role-default@example.com","password":"a long unique password","confirmation":"a long unique password","role":"admin","active":false})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    let row = account::Entity::find()
        .filter(account::Column::Email.eq("role-default@example.com"))
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.role, "user");
    assert!(row.active);
    assert_eq!(row.admin_slot, None);
    let seeded = crate::users::seed_admin(
        "New name".into(),
        "admin-test@example.com".into(),
        "new seed password".into(),
    )
    .await
    .unwrap();
    assert_eq!(seeded.id, "test-admin");
    assert_eq!(seeded.name, "Test Administrator");
    assert_eq!(seeded.role, Role::Admin);
    let (a, b) = tokio::join!(
        crate::users::seed_admin(
            "Other".into(),
            "admin-second@example.com".into(),
            "a long unique password".into()
        ),
        crate::users::seed_admin(
            "Other".into(),
            "admin-third@example.com".into(),
            "a long unique password".into()
        )
    );
    assert!(a.is_err());
    assert!(b.is_err());
    assert_eq!(
        account::Entity::find()
            .filter(account::Column::Role.eq("admin"))
            .count(crate::db::db())
            .await
            .unwrap(),
        1
    );
    // Direct database writes cannot bypass the single-admin invariant either.
    let mut duplicate: account::ActiveModel = row.clone().into();
    duplicate.id = Set("illegal-second-admin".into());
    duplicate.email = Set("illegal-admin@example.com".into());
    duplicate.role = Set("admin".into());
    duplicate.admin_slot = Set(Some(1));
    assert!(duplicate.insert(crate::db::db()).await.is_err());
    let mut mismatch: account::ActiveModel = row.into();
    mismatch.id = Set("illegal-role".into());
    mismatch.email = Set("illegal-role@example.com".into());
    mismatch.role = Set("admin".into());
    assert!(mismatch.insert(crate::db::db()).await.is_err());
}

#[tokio::test]
async fn deactivation_cancels_pending_otp_and_managed_users_enroll_their_own_authenticator() {
    let mut admin = admin().await;
    let created = create(&mut admin, "managed-enrollment@example.com").await;
    let id = created["id"].as_str().unwrap();
    let mut user = Client::new().await;
    let (s, challenge) = user
        .call(
            "start_signin",
            json!({"email":"managed-enrollment@example.com","password":"a long unique password"}),
        )
        .await;
    assert_eq!(s, StatusCode::OK);
    assert!(challenge["setup_key"].is_string());
    let code = super::auth::otp(&challenge, "managed-enrollment@example.com");
    call(
        &mut admin,
        "set_user_active",
        json!({"id":id,"active":false}),
    )
    .await;
    assert_eq!(
        user.call(
            "verify_signin",
            json!({"token":challenge["token"],"code":code})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    call(
        &mut admin,
        "set_user_active",
        json!({"id":id,"active":true}),
    )
    .await;
    assert_eq!(
        user.call(
            "verify_signin",
            json!({"token":challenge["token"],"code":code})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (s, challenge) = user
        .call(
            "start_signin",
            json!({"email":"managed-enrollment@example.com","password":"a long unique password"}),
        )
        .await;
    assert_eq!(s, StatusCode::OK);
    let (s,b) = user.call("verify_signin",json!({"token":challenge["token"],"code":super::auth::otp(&challenge,"managed-enrollment@example.com")})).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["recovery_codes"].as_array().unwrap().len(), 8);
    assert_eq!(user.request("/chat", None).await.0, StatusCode::OK);
    assert_eq!(user.request("/users", None).await.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_browser_can_create_edit_deactivate_activate_search_and_delete() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    browser
        .goto(&format!("http://{address}/signin"))
        .await
        .unwrap();
    let cookie = format!(
        "{}; Path=/; SameSite=Lax",
        super::support::cookie_for("test-admin", "Test Administrator", "admin-test@example.com")
    );
    let _: String = browser
        .eval(&format!(
            "document.cookie = {}",
            rahti::serde_json::to_string(&cookie).unwrap()
        ))
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/users"))
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('tr[data-email=\"admin-test@example.com\"]') !== null")
        .await
        .unwrap();
    assert!(browser.eval::<bool>("Array.from(document.querySelectorAll('tr[data-email=\"admin-test@example.com\"] button')).every(b => b.getBoundingClientRect().height === 0)").await.unwrap());
    browser.click("main > div > div button").await.unwrap();
    browser
        .wait_for("document.querySelector('dialog[aria-label=\"User editor\"]').open")
        .await
        .unwrap();
    for (name, value) in [
        ("name", "Browser Managed"),
        ("email", "browser-managed@example.com"),
        ("password", "a long unique password"),
        ("confirmation", "a long unique password"),
    ] {
        browser
            .click(&format!("input[name='{name}']"))
            .await
            .unwrap();
        browser.type_text(value).await.unwrap();
    }
    browser
        .click("dialog[aria-label='User editor'] button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('tr[data-email=\"browser-managed@example.com\"]')?.textContent.includes('Browser Managed') && !document.querySelector('dialog[aria-label=\"User editor\"]').open").await.unwrap();
    browser
        .click("tr[data-email='browser-managed@example.com'] button:nth-of-type(1)")
        .await
        .unwrap();
    browser.click("input[name=name]").await.unwrap();
    let _: bool = browser
        .eval("document.querySelector('input[name=name]').select(); true")
        .await
        .unwrap();
    browser.type_text("Browser Edited").await.unwrap();
    browser
        .click("dialog[aria-label='User editor'] button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('tr[data-email=\"browser-managed@example.com\"]')?.textContent.includes('Browser Edited')").await.unwrap();
    browser
        .click("tr[data-email='browser-managed@example.com'] button:nth-of-type(2)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('tr[data-email=\"browser-managed@example.com\"] td:nth-child(3)').textContent.trim() === 'Inactive'").await.unwrap();
    browser
        .click("tr[data-email='browser-managed@example.com'] button:nth-of-type(2)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('tr[data-email=\"browser-managed@example.com\"] td:nth-child(3)').textContent.trim() === 'Active'").await.unwrap();
    browser.click("input[name=search]").await.unwrap();
    browser.type_text("browser edited").await.unwrap();
    browser
        .click("form:has(input[name=search]) button")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('tbody tr').length === 1")
        .await
        .unwrap();
    browser
        .click("tr[data-email='browser-managed@example.com'] button:nth-of-type(3)")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('dialog[aria-label=\"Delete user confirmation\"]').open")
        .await
        .unwrap();
    browser
        .click("[aria-label='Delete user confirmation'] [data-slot=alert-dialog-cancel]")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('dialog[aria-label=\"Delete user confirmation\"]').open")
        .await
        .unwrap();
    browser
        .click("tr[data-email='browser-managed@example.com'] button:nth-of-type(3)")
        .await
        .unwrap();
    browser
        .click("[aria-label='Delete user confirmation'] [data-slot=alert-dialog-action]")
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('tbody tr').length === 0 && document.querySelector('[role=status]').textContent.includes('User deleted')").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
