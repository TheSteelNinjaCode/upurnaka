use super::{auth::Client, *};
use crate::models::account;
use rahti::serde_json::json;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};

async fn person(email: &str) -> (Client, String, String) {
    setup().await;
    let row = crate::auth::create_account(
        "Profile Tester".into(),
        email.into(),
        "a long unique password".into(),
        "a long unique password".into(),
        crate::auth::Role::User,
    )
    .await
    .unwrap();
    account::Entity::update_many()
        .col_expr(account::Column::Verified, Expr::value(true))
        .filter(account::Column::Id.eq(&row.id))
        .exec(crate::db::db())
        .await
        .unwrap();
    let cookie = super::support::cookie_for(&row.id, &row.name, &row.email);
    let mut client = Client::new().await;
    client.with_cookie(cookie.clone());
    client.request("/people", None).await;
    (client, row.id, cookie)
}

fn png() -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(8, 4)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

#[tokio::test]
async fn profile_updates_validate_persist_and_affect_only_the_signed_in_person() {
    let (mut client, id, cookie) = person("profile-rpc@example.com").await;
    for data in [json!({}), json!({"name":true})] {
        assert_eq!(
            client
                .request("/people", Some(("update_profile_name", data)))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    for name in [" ".to_string(), "x".repeat(81), "Bad\nName".to_string()] {
        assert_eq!(
            client
                .request(
                    "/people",
                    Some(("update_profile_name", json!({"name":name})))
                )
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        client
            .request(
                "/people",
                Some((
                    "update_profile_name",
                    json!({"name":"  Updated Profile  ","id":"other-user"})
                ))
            )
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        account::Entity::find_by_id(&id)
            .one(crate::db::db())
            .await
            .unwrap()
            .unwrap()
            .name,
        "Updated Profile"
    );
    assert_eq!(
        account::Entity::find_by_id("other-user")
            .one(crate::db::db())
            .await
            .unwrap()
            .unwrap()
            .name,
        "Other User"
    );
    for (mime, data) in [
        ("image/svg+xml", b"<svg></svg>".to_vec()),
        ("image/png", b"fake PNG".to_vec()),
        ("image/png", vec![0; 2 * 1024 * 1024 + 1]),
    ] {
        let (status, body) = client
            .upload(
                "/people",
                "update_profile_picture",
                json!({}),
                ("picture", mime, &data),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(body.contains("picture"));
    }
    assert_eq!(
        client
            .request("/people", Some(("update_profile_picture", json!({}))))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (status, body) = client
        .upload(
            "/people",
            "update_profile_picture",
            json!({}),
            ("picture", "image/png", &png()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let row = account::Entity::find_by_id(&id)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    let image = image::load_from_memory(row.picture.as_ref().unwrap()).unwrap();
    assert_eq!((image.width(), image.height()), (256, 256));
    assert!(
        account::Entity::find_by_id("other-user")
            .one(crate::db::db())
            .await
            .unwrap()
            .unwrap()
            .picture
            .is_none()
    );
    use tower::ServiceExt;
    let response = crate::routes::router()
        .oneshot(
            axum::http::Request::builder()
                .uri("/profile-picture")
                .header("cookie", cookie)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/png");
    assert_eq!(response.headers()["cache-control"], "no-store");
    let mut anonymous = Client::new().await;
    assert_eq!(
        anonymous.request("/profile-picture", None).await.0,
        StatusCode::SEE_OTHER
    );
    for (name, data) in [
        ("update_profile_name", json!({"name":"Anonymous"})),
        ("update_profile_picture", json!({})),
        ("remove_profile_picture", json!({})),
    ] {
        assert_eq!(
            anonymous.request("/people", Some((name, data))).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn people_profile_menu_edits_name_and_picture_and_restores_focus() {
    let (_, _, cookie) = person("profile-browser@example.com").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    browser
        .goto(&format!("http://{address}/signin"))
        .await
        .unwrap();
    let _: String = browser
        .eval(&format!(
            "document.cookie = {}",
            json!(format!("{cookie}; Path=/; SameSite=Lax"))
        ))
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/people"))
        .await
        .unwrap();
    browser
        .click("aside button[aria-label='More options']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.people-profile-menu').matches(':popover-open') && document.activeElement.textContent.includes('Edit name')").await.unwrap();
    browser.press("ArrowDown").await.unwrap();
    browser
        .wait_for("document.activeElement.textContent.includes('Change profile picture')")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("document.activeElement.getAttribute('aria-label') === 'More options' && !document.querySelector('.people-profile-menu').matches(':popover-open')").await.unwrap();
    browser
        .click("aside button[aria-label='More options']")
        .await
        .unwrap();
    browser
        .click(".people-profile-menu button:first-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.people-profile-dialog').open && document.activeElement.name === 'name'").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser.type_text("Browser Profile").await.unwrap();
    browser
        .click(".people-profile-dialog button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.people-profile-dialog').open && document.querySelector('button[aria-label=\"Your profile\"]').title === 'Browser Profile' && document.querySelector('aside').textContent.includes('Browser Profile')").await.unwrap();
    browser
        .click("aside button[aria-label='More options']")
        .await
        .unwrap();
    browser
        .click(".people-profile-menu button:last-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.people-profile-dialog').open && document.activeElement.textContent.includes('Upload') && document.querySelector('.people-profile-dialog button[type=submit]').disabled").await.unwrap();
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(png());
    let _: bool = browser.eval(&format!("const transfer = new DataTransfer(); transfer.items.add(new File([Uint8Array.from(atob({}), c => c.charCodeAt(0))], 'portrait.png', {{type:'image/png'}})); const input = document.querySelector('input[name=picture]'); input.files = transfer.files; input.dispatchEvent(new Event('change', {{bubbles:true}})); true",json!(encoded))).await.unwrap();
    browser.wait_for("document.querySelector('.picture-editor-preview img')?.src.startsWith('data:image/png') && !document.querySelector('.people-profile-dialog button[type=submit]').disabled").await.unwrap();
    browser
        .click(".people-profile-dialog button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.people-profile-dialog').open && Array.from(document.querySelectorAll('img[src=\"/profile-picture\"]')).length >= 3 && Array.from(document.querySelectorAll('img[src=\"/profile-picture\"]')).every(img => img.complete && img.naturalWidth === 256)").await.unwrap();
    browser
        .click("aside button[aria-label='More options']")
        .await
        .unwrap();
    browser
        .click(".people-profile-menu button:last-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.picture-editor').open && document.querySelector('.people-profile-dialog button[type=submit]').disabled").await.unwrap();
    browser
        .click(".picture-editor-actions button:nth-child(2)")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.picture-editor-preview span').hidden && !document.querySelector('.people-profile-dialog button[type=submit]').disabled").await.unwrap();
    // Closing discards the pending removal.
    browser.press("Escape").await.unwrap();
    browser
        .click("aside button[aria-label='More options']")
        .await
        .unwrap();
    browser
        .click(".people-profile-menu button:last-child")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.picture-editor-preview img').hidden && document.querySelector('.people-profile-dialog button[type=submit]').disabled").await.unwrap();
    browser
        .click(".picture-editor-actions button:nth-child(2)")
        .await
        .unwrap();
    browser
        .click(".people-profile-dialog button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.people-profile-dialog').open && document.querySelectorAll('img[src=\"/profile-picture\"]').length === 0").await.unwrap();
    browser
        .click("aside button[aria-label='More options']")
        .await
        .unwrap();
    browser
        .click(".people-profile-menu button:first-child")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.people-profile-dialog').open && document.activeElement.getAttribute('aria-label') === 'More options'").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
