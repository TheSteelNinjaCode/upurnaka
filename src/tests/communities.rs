use super::*;
use base64::Engine;
use rahti::serde_json;

fn community_test_png() -> Vec<u8> {
    let mut output = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(8, 4)
        .write_to(&mut output, image::ImageFormat::Png)
        .unwrap();
    output.into_inner()
}

#[tokio::test]
async fn community_template_edits_and_creates_in_one_step() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/communities"))
        .await
        .unwrap();
    for template in ["Business", "Life", "School"] {
        let _: bool = browser.eval(&format!("Array.from(document.querySelectorAll('main button')).find(button => button.textContent.trim().endsWith('{template}')).click(); true")).await.unwrap();
        browser.wait_for(&format!("document.querySelector('.community-create-dialog[open]') && document.querySelector('#community-name').value === '{template}'")).await.unwrap();
        browser.wait_for("document.querySelector('.community-create-dialog').getAnimations().every(animation => animation.playState === 'finished')").await.unwrap();
        let _: bool = browser.eval("document.querySelector('button[aria-label=\"Close community creation\"]').click(); true").await.unwrap();
        browser
            .wait_for("!document.querySelector('.community-create-dialog[open]')")
            .await
            .unwrap();
    }
    // Template buttons have compiled event bindings, so select by their text.
    let _: bool = browser.eval("Array.from(document.querySelectorAll('main button')).find(button => button.textContent.trim().endsWith('School')).click(); true").await.unwrap();
    browser.wait_for("document.querySelector('.community-create-dialog[open]') && document.querySelector('#community-name').value === 'School'").await.unwrap();
    browser.wait_for("document.querySelector('.community-create-dialog').getAnimations().every(animation => animation.playState === 'finished')").await.unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-create-dialog').textContent.includes('Create your school community')").await.unwrap());
    let image = base64::engine::general_purpose::STANDARD.encode(community_test_png());
    let _: bool = browser.eval(&format!(r#"(() => {{ const input = document.querySelector('#community-picture'); const data = new DataTransfer(); data.items.add(new File([Uint8Array.from(atob('{image}'), c => c.charCodeAt(0))], 'community.png', {{type:'image/png'}})); input.files = data.files; input.dispatchEvent(new Event('change', {{bubbles:true}})); return true; }})()"#)).await.unwrap();
    browser.wait_for("!document.querySelector('.community-avatar-photo').hidden && document.querySelector('.community-avatar-photo').naturalWidth === 8").await.unwrap();
    // A bad replacement must leave the existing valid preview in place.
    let _: bool = browser.eval("(() => { const input = document.querySelector('#community-picture'); const data = new DataTransfer(); data.items.add(new File(['invalid'], 'picture.txt', {type:'text/plain'})); input.files = data.files; input.dispatchEvent(new Event('change', {bubbles:true})); return true; })()").await.unwrap();
    browser.wait_for("document.querySelector('main').textContent.includes('Choose a PNG or JPEG picture up to 2 MB.') && document.querySelector('.community-avatar-photo').naturalWidth === 8").await.unwrap();
    let _: bool = browser
        .eval("document.querySelector('#community-description').focus(); true")
        .await
        .unwrap();
    browser.type_text("A place for our school.").await.unwrap();
    let _: bool = browser.eval(r##"document.querySelector("button[aria-label='Edit community guidelines']").click(); true"##).await.unwrap();
    browser
        .wait_for("document.activeElement.id === 'community-guidelines'")
        .await
        .unwrap();
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser
        .type_text("Respect each other and share ideas.")
        .await
        .unwrap();
    let _: bool = browser.eval(r##"document.querySelector("button[aria-label='Done editing community guidelines']").click(); true"##).await.unwrap();
    browser.wait_for("document.querySelector('.community-guidelines-preview').textContent === 'Respect each other and share ideas.'").await.unwrap();
    let fits: bool = browser.eval("(() => { const box = document.querySelector('.community-create-dialog').getBoundingClientRect(); return box.top >= 0 && box.left >= 0 && box.bottom <= innerHeight && box.right <= innerWidth; })()").await.unwrap();
    assert!(fits, "creation dialog fits the viewport");
    let _: bool = browser.eval(r##"document.querySelector(".community-create-form button[type=submit]").click(); true"##).await.unwrap();
    browser.wait_for("!document.querySelector('.community-create-dialog[open]') && document.querySelector('main').textContent.includes('A place for our school.')").await.unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('main [data-slot=card] img')).some(image => !image.hidden && image.naturalWidth === 256)").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn community_picture_upload_normalizes_images_and_rejects_invalid_files() {
    setup().await;
    let mut client = super::auth::Client::new().await;
    client.with_cookie(test_cookie());
    client.request("/communities", None).await;
    let fields = serde_json::json!({"name":"School", "category":"School"});
    let (status, body) = client
        .upload(
            "/communities",
            "create_community",
            fields.clone(),
            ("picture", "image/png", &community_test_png()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let community: serde_json::Value = serde_json::from_str(&body).unwrap();
    let picture = community["picture"]
        .as_str()
        .unwrap()
        .strip_prefix("data:image/png;base64,")
        .unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(picture)
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    assert_eq!((image.width(), image.height()), (256, 256));
    for (mime, data) in [
        ("image/png", b"not a PNG".to_vec()),
        ("image/svg+xml", b"<svg></svg>".to_vec()),
        ("image/png", vec![0; 2 * 1024 * 1024 + 1]),
    ] {
        let (status, body) = client
            .upload(
                "/communities",
                "create_community",
                fields.clone(),
                ("picture", mime, &data),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(body.contains("picture"));
    }
}

#[tokio::test]
async fn community_details_round_trip_and_reject_invalid_payloads() {
    let payload = serde_json::json!({"name":"School", "category":"School", "description":"Our school", "guidelines":"Be kind", "avatar":"🎓"});
    let (status, body) = rpc("/communities", "create_community", &payload.to_string()).await;
    assert_eq!(status, StatusCode::OK);
    let community: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(community["description"], "Our school");
    assert_eq!(community["guidelines"], "Be kind");
    assert_eq!(community["avatar"], "🎓");
    let mut invalid = payload.clone();
    invalid["description"] = serde_json::Value::String("x".repeat(1001));
    let (status, body) = rpc("/communities", "create_community", &invalid.to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("description"));
    let (status, _) = rpc(
        "/communities",
        "create_community",
        r#"{"name":3,"category":"School"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
