use super::*;
use rahti::serde_json::json;
use std::sync::{LazyLock, Mutex};

// Per-account test configuration keeps live credentials out of tests and avoids
// changing process environment while other router/browser tests run.
static KEYS: LazyLock<Mutex<std::collections::HashMap<String, String>>> =
    LazyLock::new(Default::default);
pub(crate) fn key_for(id: &str) -> Option<String> {
    KEYS.lock().unwrap().get(id).cloned()
}

#[tokio::test]
async fn gif_messages_accept_only_giphy_identifiers() {
    for id in [
        "",
        "../other",
        "https://evil.test/x.gif",
        "abc?x=1",
        "abc_def",
    ] {
        let (status, body) = rpc(
            "/chat",
            "send_message",
            &json!({"text":"", "gif_id":id}).to_string(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(body.contains("gif_id"));
    }
    let (status, body) = rpc(
        "/chat",
        "send_message",
        &json!({"text":"", "gif_id":"l0MYt5jPR6QX5pnqM"}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("https://media.giphy.com/media/l0MYt5jPR6QX5pnqM/giphy.gif"));
    assert_eq!(
        rpc("/chat", "send_message", r#"{"text":"","gif_id":null}"#)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        rpc("/chat", "send_message", r#"{"text":"","gif_id":true}"#)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn giphy_picker_trends_searches_paginates_and_sends_a_gif() {
    setup().await;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
    let row = crate::auth::create_account(
        "GIF Tester".into(),
        "giphy-browser@example.com".into(),
        "a long unique password".into(),
        "a long unique password".into(),
        crate::auth::Role::User,
    )
    .await
    .unwrap();
    crate::models::account::Entity::update_many()
        .col_expr(crate::models::account::Column::Verified, Expr::value(true))
        .filter(crate::models::account::Column::Id.eq(&row.id))
        .exec(crate::db::db())
        .await
        .unwrap();
    KEYS.lock()
        .unwrap()
        .insert(row.id.clone(), "public-test-key".into());
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    browser
        .goto(&format!("http://{address}/signin"))
        .await
        .unwrap();
    let cookie = super::support::cookie_for(&row.id, &row.name, &row.email);
    let _: String = browser
        .eval(&format!(
            "document.cookie = {}",
            json!(format!("{cookie}; Path=/; SameSite=Lax"))
        ))
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    // Mock only the external provider. The page registry, event handlers, RPC,
    // form state, authenticated server validation and message rendering are real.
    let _: bool = browser.eval(r#"
        window.giphyRequests = [];
        window.giphyMode = 'success';
        const originalFetch = window.fetch.bind(window);
        window.fetch = async (input, options) => {
            const url = new URL(String(input), location.origin);
            if (url.hostname !== 'api.giphy.com') return originalFetch(input, options);
            window.giphyRequests.push(url.href);
            if (window.giphyMode === 'failure') return {ok:false, status:429};
            const query = url.searchParams.get('q') || '';
            const offset = Number(url.searchParams.get('offset'));
            const item = id => ({id, title:query ? query + ' GIF' : 'Trending GIF', images:{fixed_height_small:{url:'https://media.giphy.com/media/' + id + '/giphy.gif'}}});
            const data = query === 'empty' ? [] : [item(offset ? 'SecondGif' : 'FirstGif')];
            if (query === 'slow') await new Promise(resolve => setTimeout(resolve, 1200));
            return {ok:true, json:async () => ({data,pagination:{total_count:2}})};
        };
        // Render a tiny fixture instead of downloading remote media in the suite.
        const originalAttribute = Element.prototype.setAttribute;
        Element.prototype.setAttribute = function(name, value) {
            if (this.tagName === 'IMG' && name === 'src' && String(value).startsWith('https://media.giphy.com/')) value = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
            return originalAttribute.call(this,name,value);
        };
        true
    "#).await.unwrap();
    browser.click("button[aria-label=Emoji]").await.unwrap();
    let ready = browser.wait_for("document.querySelectorAll('.chat-gif-grid button').length === 1 && !document.querySelector('.chat-giphy-attribution').hidden && document.querySelector('.chat-giphy-attribution img').naturalWidth > 0").await;
    if let Err(error) = ready {
        let state: String = browser.eval("JSON.stringify({pickerPresent:!!document.querySelector('.chat-emoji-picker'), attributionWidth:document.querySelector('.chat-giphy-attribution img')?.naturalWidth, gifs:document.querySelectorAll('.chat-gif-grid button').length})").await.unwrap();
        panic!(
            "{error}; picker state: {state}; browser errors: {:?}",
            browser.errors()
        );
    }
    let trending: bool = browser.eval("window.giphyRequests.some(value => {const u = new URL(value); return u.pathname.endsWith('/trending') && u.searchParams.get('rating') === 'g' && u.searchParams.get('api_key') === 'public-test-key';})").await.unwrap();
    assert!(trending);
    browser
        .click(".chat-emoji-tabs label:nth-of-type(3)")
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('.chat-gif-grid button').length === 1 && !document.querySelector('.chat-gif-more').hidden").await.unwrap();
    browser.click(".chat-gif-more").await.unwrap();
    browser.wait_for("document.querySelectorAll('.chat-gif-grid button').length === 2 && document.querySelector('.chat-gif-more').hidden").await.unwrap();
    browser.click(".chat-emoji-search input").await.unwrap();
    browser.type_text("happy & excited").await.unwrap();
    browser.wait_for("document.querySelector('.chat-gif-grid button img')?.getAttribute('alt') === 'happy & excited GIF'").await.unwrap();
    let query: bool = browser.eval("window.giphyRequests.some(value => {const u = new URL(value); return u.pathname.endsWith('/search') && u.searchParams.get('q') === 'happy & excited' && u.searchParams.get('offset') === '0';})").await.unwrap();
    assert!(
        query,
        "search must encode the user's query once and reset pagination"
    );
    browser.click(".chat-gif-grid button").await.unwrap();
    browser.wait_for("!document.querySelector('.chat-draft-gif').hidden && !document.querySelector('button[aria-label=Send]').disabled && !document.querySelector('.chat-emoji-picker').matches(':popover-open')").await.unwrap();
    browser
        .click("button[aria-label='New chat']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.chat-draft-gif').hidden")
        .await
        .unwrap();
    browser
        .click("aside .chat-self-entry button")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.chat-draft-gif').hidden")
        .await
        .unwrap();
    browser.click("button[aria-label=Send]").await.unwrap();
    browser.wait_for("document.querySelectorAll('.chat-message-gif').length === 1 && document.querySelector('.chat-draft-gif').hidden").await.unwrap();
    browser.click("button[aria-label=Emoji]").await.unwrap();
    // The picker focuses its search field as it opens; work in it after that.
    browser
        .wait_for("document.querySelector('.chat-emoji-picker').matches(':popover-open') && document.activeElement === document.querySelector('.chat-emoji-search input')")
        .await
        .unwrap();
    browser.click(".chat-emoji-search input").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser.type_text("empty").await.unwrap();
    browser.wait_for("document.querySelectorAll('.chat-gif-grid button').length === 0 && Array.from(document.querySelectorAll('.chat-emoji-empty')).some(el => !el.hidden && el.textContent.includes('No GIFs found'))").await.unwrap();
    let _: bool = browser
        .eval("window.giphyMode = 'failure'; document.activeElement.select(); true")
        .await
        .unwrap();
    browser.type_text("error").await.unwrap();
    browser.wait_for("document.querySelector('.chat-emoji-content').textContent.includes('GIPHY is busy') && !document.querySelector('.chat-gif-retry').hidden").await.unwrap();
    let _: bool = browser
        .eval("window.giphyMode = 'success'; true")
        .await
        .unwrap();
    browser.click(".chat-gif-retry").await.unwrap();
    browser
        .wait_for("document.querySelectorAll('.chat-gif-grid button').length === 1")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
    KEYS.lock().unwrap().remove(&row.id);
}
