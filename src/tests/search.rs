use super::*;
use rahti::serde_json::{Value, json};

#[tokio::test]
async fn workspace_search_tabs_show_matching_filters_and_keep_the_active_tab() {
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
        .click("textarea[aria-label='Type a message']")
        .await
        .unwrap();
    browser.type_text("filter message ordinary").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('textarea').value === '' && document.querySelector('[aria-label=Messages]').textContent.includes('filter message ordinary')").await.unwrap();
    browser
        .click("button[aria-label='More actions']")
        .await
        .unwrap();
    browser
        .click(".chat-compose-menu [role=menuitem]:nth-child(2)")
        .await
        .unwrap();
    browser
        .click(".chat-compose-menu [role=menuitem]:nth-child(4)")
        .await
        .unwrap();
    browser
        .click("textarea[aria-label='Type a message']")
        .await
        .unwrap();
    browser.type_text("filter message important").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('textarea').value === '' && document.querySelector('[aria-label=Messages]').textContent.includes('filter message important')").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("filter message").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.click("#global-search-tab-messages").await.unwrap();
    browser.wait_for("document.querySelectorAll('.global-search-card').length === 2 && !document.querySelector('.global-search-filters').hidden").await.unwrap();
    let matches: bool = browser.eval("(() => {const header=document.querySelector('.global-search-header').getBoundingClientRect(); const row=document.querySelector('.global-search-filters').getBoundingClientRect();const tab=document.querySelector('#global-search-tab-messages');const style=getComputedStyle(tab);return header.height===60 && row.height===41 && row.top===header.bottom && style.borderColor==='rgb(222, 221, 234)' && style.backgroundColor==='rgb(62, 61, 108)';})()").await.unwrap();
    assert!(
        matches,
        "search tabs and filter strip must match the Teams layout"
    );
    browser
        .click("button[aria-label='Filter messages by sender']")
        .await
        .unwrap();
    browser
        .click("#dropdown-menu-search-from [role=menuitem]:nth-child(2)")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Filter messages by date']")
        .await
        .unwrap();
    browser
        .click("#dropdown-menu-search-message-date [role=menuitem]:nth-child(2)")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More message filters']")
        .await
        .unwrap();
    browser
        .click("#popover-search-message-more label:nth-of-type(1) input")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('.global-search-card').length === 0")
        .await
        .unwrap();
    // The More panel has two labelled checkboxes; reset before selecting Important.
    let _: bool = browser.eval("Array.from(document.querySelectorAll('#popover-search-message-more button')).find(button=>button.textContent.trim()==='Reset message filters').click(); true").await.unwrap();
    browser
        .click("#popover-search-message-more label:nth-of-type(2) input")
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('.global-search-card').length === 1 && document.querySelector('.global-search-card').textContent.includes('filter message important')").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('#popover-search-message-more').matches(':popover-open') && !document.querySelector('.global-search-view').hidden").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('#global-search-tab-messages').getAttribute('aria-selected') === 'true' && document.querySelectorAll('.global-search-card').length === 1").await.unwrap();
    browser.click("#global-search-tab-files").await.unwrap();
    browser
        .click("button[aria-label='Filter files by type']")
        .await
        .unwrap();
    browser
        .click("#dropdown-menu-search-file-type [role=menuitem]:nth-child(3)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false' && document.querySelector('button[aria-label=\"Filter files by type\"]').textContent.includes('PDF')").await.unwrap();
    browser
        .click("#global-search-tab-communities")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.global-search-filters').hidden")
        .await
        .unwrap();
    browser.click("#global-search-tab-messages").await.unwrap();
    browser.wait_for("!document.querySelector('.global-search-filters').hidden && document.querySelectorAll('.global-search-card').length === 1").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn workspace_search_finds_current_communities_and_focuses_the_result() {
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
    browser.click("main button").await.unwrap();
    browser.click("#community-name").await.unwrap();
    browser
        .type_text("Searchable weekend community")
        .await
        .unwrap();
    browser.press("Enter").await.unwrap();
    let _: bool = browser.eval("Array.from(document.querySelectorAll('dialog button')).find(button=>button.textContent.trim()==='Hobbies').click(); true").await.unwrap();
    browser.wait_for("!document.querySelector('dialog[open]') && document.querySelector('main').textContent.includes('Searchable weekend community')").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("weekend").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .click("#global-search-tab-communities")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.global-search-card')?.textContent.includes('Searchable weekend community')").await.unwrap();
    browser.click(".global-search-card").await.unwrap();
    browser.wait_for("document.querySelector('.global-search-view').hidden && document.activeElement.textContent.includes('Searchable weekend community') && document.activeElement.classList.contains('outline-brand-ink')").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn workspace_search_limits_directory_fields_and_scopes_files() {
    use crate::models::{account, chat_file};
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
    setup().await;
    let owner = crate::auth::create_account(
        "Search Directory Example".into(),
        "workspace-search-owner@example.com".into(),
        "a long unique password".into(),
        "a long unique password".into(),
        crate::auth::Role::User,
    )
    .await
    .unwrap();
    account::Entity::update_many()
        .col_expr(account::Column::Verified, Expr::value(true))
        .filter(account::Column::Id.eq(&owner.id))
        .exec(crate::db::db())
        .await
        .unwrap();
    for (id, file_owner, name) in [
        ("search-own-report", owner.id.as_str(), "search-report.txt"),
        ("search-own-percent", owner.id.as_str(), "search-100%.txt"),
        ("search-other-report", "other-user", "search-report.txt"),
    ] {
        chat_file::ActiveModel {
            id: Set(id.into()),
            owner_id: Set(file_owner.into()),
            name: Set(name.into()),
            content_type: Set("text/plain".into()),
            bytes: Set(10),
            file_id: Set("test-storage-object".into()),
            file_key: Set("private-storage-key".into()),
            created_at: Set(1),
            folder_id: Set(None),
        }
        .insert(crate::db::db())
        .await
        .unwrap();
    }
    let mut client = super::auth::Client::new().await;
    client.with_cookie(super::support::cookie_for(
        &owner.id,
        &owner.name,
        &owner.email,
    ));
    client.request("/chat", None).await;
    let (status, _, body) = client
        .request(
            "/chat",
            Some(("search_workspace", json!({"query":"SEARCH"}))),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let response: Value = rahti::serde_json::from_str(&body).unwrap();
    assert!(
        response["people"]
            .as_array()
            .unwrap()
            .iter()
            .any(|person| person["id"] == owner.id && person["is_self"] == true)
    );
    assert_eq!(response["files"].as_array().unwrap().len(), 2);
    assert!(
        response["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["id"] != "search-other-report")
    );
    for secret in [
        "email",
        "password_hash",
        "otp_secret",
        "file_key",
        "file_id",
        "role",
        "recovery_hashes",
    ] {
        assert!(
            !body.contains(&format!("\"{secret}\"")),
            "{secret} escaped into search results"
        );
    }
    let (status, _, body) = client
        .request("/chat", Some(("search_workspace", json!({"query":"%"}))))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let response: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(response["files"].as_array().unwrap().len(), 1);
    assert_eq!(response["files"][0]["id"], "search-own-percent");
    chat_file::ActiveModel {
        id: Set("search-recent-pdf".into()),
        owner_id: Set(owner.id.clone()),
        name: Set("search-recent.PDF".into()),
        content_type: Set("application/pdf".into()),
        bytes: Set(10),
        file_id: Set("test-storage-pdf".into()),
        file_key: Set("private-storage-key".into()),
        created_at: Set(crate::rahti::time::Utc::now().timestamp()),
        folder_id: Set(None),
    }
    .insert(crate::db::db())
    .await
    .unwrap();
    for (kind, date, expected) in [
        ("pdf", "day", 1),
        ("pdf", "any", 1),
        ("text", "any", 2),
        ("text", "day", 0),
        ("images", "any", 0),
    ] {
        let (status, _, body) = client
            .request(
                "/chat",
                Some((
                    "search_workspace",
                    json!({"query":"search","file_type":kind,"file_date":date}),
                )),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let response: Value = rahti::serde_json::from_str(&body).unwrap();
        assert_eq!(
            response["files"].as_array().unwrap().len(),
            expected,
            "{kind}/{date}"
        );
    }
    for payload in [
        json!({"query":"search","file_type":"executable"}),
        json!({"query":"search","file_date":"forever"}),
    ] {
        assert_eq!(
            client
                .request("/chat", Some(("search_workspace", payload)))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    for query in [" ".to_string(), "a".repeat(121)] {
        assert_eq!(
            client
                .request("/chat", Some(("search_workspace", json!({"query":query}))))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        client
            .request("/chat", Some(("search_workspace", json!({"query":{}}))))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let mut anonymous = super::auth::Client::new().await;
    assert_eq!(
        anonymous
            .request(
                "/signin",
                Some(("search_workspace", json!({"query":"Search"})))
            )
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn workspace_search_switches_tabs_and_returns_to_preserved_chat() {
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
        .click("textarea[aria-label='Type a message']")
        .await
        .unwrap();
    browser.type_text("hello from global search").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('[aria-label=Messages]').textContent.includes('hello from global search') && document.querySelector('textarea').value === ''").await.unwrap();
    browser.type_text("Unsent search draft").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("Jefferson").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("!document.querySelector('.global-search-view').hidden && document.querySelector('.shell-page').hidden && document.querySelector('.global-search-card')?.textContent.includes('Jefferson Abraham')").await.unwrap();
    let layout: bool = browser.eval("(() => {const card=document.querySelector('.global-search-card').getBoundingClientRect();return card.width <= 760 && card.height >= 72 && document.querySelectorAll('.global-search-tab').length === 4 && document.querySelector('[role=tab][aria-selected=true]').textContent === 'People';})()").await.unwrap();
    assert!(layout);
    browser.click(".global-search-card").await.unwrap();
    browser.wait_for("document.querySelector('.global-search-view').hidden && document.querySelector('textarea').value === 'Unsent search draft'").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("hello").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.click("#global-search-tab-people").await.unwrap();
    browser.press("ArrowRight").await.unwrap();
    browser.wait_for("document.querySelector('#global-search-tab-messages').getAttribute('aria-selected') === 'true' && document.querySelector('.global-search-card')?.textContent.includes('hello from global search')").await.unwrap();
    browser.click(".global-search-card").await.unwrap();
    browser.wait_for("document.querySelector('.global-search-view').hidden && document.querySelector('[aria-label=\"Find messages in this conversation\"]').value === 'hello from global search' && document.querySelector('textarea').value === 'Unsent search draft'").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("NobodyMatchesThisSearch").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false' && document.querySelector('.global-search-view h1').textContent === 'No people found' && !document.querySelector('.global-search-view h1').parentElement.hidden").await.unwrap();
    browser
        .click("button[aria-label='Clear search']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.global-search-view').hidden && document.querySelector('input[aria-label=Search]').value === '' && document.querySelector('textarea').value === 'Unsent search draft'").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
