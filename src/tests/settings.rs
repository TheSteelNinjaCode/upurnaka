use super::*;

#[tokio::test]
async fn settings_renders_storage_options_and_download_guidance() {
    let (status, body) = get("/settings").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for text in [
        "Settings categories",
        "File storage",
        "Computer",
        "Dilnaka storage",
        "browser's download folder",
    ] {
        assert!(body.contains(text), "missing {text}");
    }
}

#[tokio::test]
async fn settings_requires_a_session() {
    setup().await;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let response = crate::routes::router()
        .oneshot(
            Request::builder()
                .uri("/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert!(
        response.headers()["location"]
            .to_str()
            .unwrap()
            .starts_with("/signin")
    );
}

#[tokio::test]
async fn settings_preferences_persist_apply_and_reset_in_the_browser() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .click("button[aria-label='Settings and more']")
        .await
        .unwrap();
    browser.click("a[href='/settings']").await.unwrap();
    browser
        .wait_for("location.pathname === '/settings' && !!document.querySelector('.settings-nav')")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Find in Settings']")
        .await
        .unwrap();
    browser.type_text("storage").await.unwrap();
    browser
        .wait_for("document.querySelector('.settings-main h2').textContent === 'Files and links'")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    browser
        .click(".storage-choice:first-child strong")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).storage === 'computer'").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=computer]').checked")
        .await
        .unwrap();
    browser
        .click(".storage-choice:last-child strong")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).storage === 'dilnaka'").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    browser.click(".settings-nav:nth-child(2)").await.unwrap();
    browser
        .click("input[name='theme'][value='light']")
        .await
        .unwrap();
    browser
        .wait_for("document.documentElement.dataset.appearance === 'light'")
        .await
        .unwrap();
    browser.click(".settings-nav:nth-child(3)").await.unwrap();
    browser
        .click("input[aria-label='Press Enter to send']")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).enterToSend === false").await.unwrap();
    browser
        .click("input[aria-label='Check spelling while typing']")
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .wait_for("document.documentElement.dataset.appearance === 'light'")
        .await
        .unwrap();
    browser.click("textarea[name='text']").await.unwrap();
    browser
        .wait_for("document.querySelector('textarea[name=text]').spellcheck === false")
        .await
        .unwrap();
    browser
        .type_text("A draft that should not send")
        .await
        .unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('textarea[name=text]').value.includes('A draft that should not send')").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.click(".settings-intro button").await.unwrap();
    browser.click(".settings-primary").await.unwrap();
    browser.wait_for("document.documentElement.dataset.appearance === 'dark' && localStorage.getItem('upurnaka.settings.v1.legacy-test-user') === null").await.unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn settings_recovers_invalid_values_and_reports_failed_saves() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    // Older settings saved Computer implicitly alongside unrelated changes.
    let _: bool = browser.eval("localStorage.setItem('upurnaka.settings.v1.legacy-test-user', JSON.stringify({theme:'invalid', textSize:'huge', storage:'computer', spellcheck:'false'})); true").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.wait_for("document.documentElement.dataset.appearance === 'dark' && document.documentElement.dataset.textSize === 'standard'").await.unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    let _: bool = browser.eval("Storage.prototype.setItem = function() { throw new DOMException('Storage unavailable', 'QuotaExceededError'); }; true").await.unwrap();
    browser
        .click("input[name='storage'][value='computer']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('[role=status]').textContent.includes('Could not save') && document.querySelector('input[name=storage][value=dilnaka]').checked").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
