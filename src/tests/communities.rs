use super::*;
use rahti::serde_json;

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
    browser
        .click("button[aria-label='Edit community image']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-image-picker').hidden")
        .await
        .unwrap();
    let _: bool = browser
        .eval("document.querySelector('.community-image-choice:last-child').click(); true")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-image-picker').hidden")
        .await
        .unwrap();
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
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
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
