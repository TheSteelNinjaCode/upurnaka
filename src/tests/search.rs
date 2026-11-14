use super::*;
use rahti::serde_json::{Value, json};

#[tokio::test]
async fn workspace_file_filters_match_teams_and_apply_calendar_ranges() {
    use crate::models::chat_file;
    use sea_orm::{ActiveModelTrait, Set};
    setup().await;
    for (id, owner, name, kind, at) in [
        (
            "teams-file-pdf",
            "legacy-test-user",
            "teams-file-report.pdf",
            "application/pdf",
            crate::rahti::time::Utc::now().timestamp(),
        ),
        (
            "teams-file-excel",
            "legacy-test-user",
            "teams-file-budget.xlsx",
            "application/octet-stream",
            1,
        ),
        (
            "teams-file-other",
            "other-user",
            "teams-file-private.pdf",
            "application/pdf",
            crate::rahti::time::Utc::now().timestamp(),
        ),
    ] {
        chat_file::ActiveModel {
            id: Set(id.into()),
            owner_id: Set(owner.into()),
            name: Set(name.into()),
            content_type: Set(kind.into()),
            bytes: Set(10),
            file_id: Set("test-storage-object".into()),
            file_key: Set("private-storage-key".into()),
            created_at: Set(at),
            folder_id: Set(None),
        }
        .insert(crate::db::db())
        .await
        .unwrap();
    }
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("teams-file").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.click("#global-search-tab-files").await.unwrap();
    browser.wait_for("document.querySelectorAll('.global-search-card').length === 2 && document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false'").await.unwrap();
    browser
        .click("button[aria-label='Filter files by type']")
        .await
        .unwrap();
    let labels: Vec<String> = browser.eval("Array.from(document.querySelectorAll('.search-file-type-option')).map(row=>row.querySelector('span.flex-1').textContent)").await.unwrap();
    assert_eq!(
        labels,
        [
            "Excel",
            "Images",
            "Loop",
            "OneNote",
            "PDF",
            "PowerPoint",
            "Text file",
            "Word"
        ]
    );
    assert_eq!(
        browser
            .eval::<String>("getComputedStyle(document.querySelector('.file-kind-badge')).fill")
            .await
            .unwrap(),
        "rgb(16, 124, 65)"
    );
    browser
        .click(".search-file-type-option:nth-child(5)")
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('.global-search-card').length === 1 && document.querySelector('.global-search-card').textContent.includes('teams-file-report.pdf') && document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false'").await.unwrap();
    browser
        .click("button[aria-label='Filter files by sender']")
        .await
        .unwrap();
    browser
        .wait_for("document.activeElement.getAttribute('aria-label') === 'Find file sender'")
        .await
        .unwrap();
    browser.type_text("fixture@example.com").await.unwrap();
    browser.click(".file-owner-option").await.unwrap();
    browser.wait_for("!document.querySelector('.file-owner-picker').matches(':popover-open') && document.querySelector('button[aria-label=\"Filter files by sender\"]').textContent.includes('You')").await.unwrap();
    browser
        .click("button[aria-label='Filter files by date']")
        .await
        .unwrap();
    assert_eq!(browser.eval::<Vec<String>>("Array.from(document.querySelectorAll('.file-date-preset')).map(button=>button.textContent)").await.unwrap(),["Today","Yesterday","This week","This month"]);
    assert!(browser.eval::<bool>("document.querySelector('button[aria-label=\"Apply file date\"]').disabled && document.querySelector('input[aria-label=\"File date from\"]').value === ''").await.unwrap());
    browser
        .click(".file-date-preset:nth-of-type(2)")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.file-date-picker').matches(':popover-open') && document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false' && document.querySelectorAll('.global-search-card').length === 0").await.unwrap();
    browser
        .click("button[aria-label='Filter files by date']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Clear file date']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false' && document.querySelectorAll('.global-search-card').length === 1").await.unwrap();
    browser
        .click("button[aria-label='Filter files by date']")
        .await
        .unwrap();
    browser.click(".file-date-preset").await.unwrap();
    browser.wait_for("!document.querySelector('.file-date-picker').matches(':popover-open') && document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false'").await.unwrap();
    browser
        .click("button[aria-label='Filter files by date']")
        .await
        .unwrap();
    let previous: String = browser
        .eval("document.querySelector('input[aria-label=\"File date to\"]').value")
        .await
        .unwrap();
    browser
        .click("input[aria-label='File date to']")
        .await
        .unwrap();
    browser.press("ArrowUp").await.unwrap();
    browser.press("Tab").await.unwrap();
    assert_ne!(
        browser
            .eval::<String>("document.querySelector('input[aria-label=\"File date to\"]').value")
            .await
            .unwrap(),
        previous
    );
    browser
        .click("button[aria-label='Apply file date']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.file-date-picker').matches(':popover-open') && document.querySelector('#global-search-panel').getAttribute('aria-busy') === 'false' && document.querySelectorAll('.global-search-card').length === 1").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn message_search_calendar_presets_and_custom_ranges_are_validated() {
    use crate::rahti::time;
    use chrono::{Datelike, Duration};
    for preset in ["today", "yesterday", "week", "month"] {
        let (status, body) = rpc(
            "/chat",
            "message_search_dates",
            &json!({"preset":preset}).to_string(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let range: Value = rahti::serde_json::from_str(&body).unwrap();
        let today = time::today();
        let first = match preset {
            "yesterday" => today - Duration::days(1),
            "week" => today - Duration::days(today.weekday().num_days_from_monday().into()),
            "month" => today.with_day(1).unwrap(),
            _ => today,
        };
        let last = if preset == "yesterday" { first } else { today };
        assert_eq!(
            range["start"],
            time::day_bounds_utc(first).unwrap().0.timestamp()
        );
        assert_eq!(
            range["end"],
            time::day_bounds_utc(last).unwrap().1.timestamp()
        );
    }
    let (status, body) = rpc(
        "/chat",
        "message_search_dates",
        r#"{"preset":"custom","from":"2026-03-08","to":"2026-03-09"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let range: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(range["from"], "2026-03-08");
    assert_eq!(range["to"], "2026-03-09");
    assert_eq!(
        range["end"],
        time::day_bounds_utc(time::NaiveDate::from_ymd_opt(2026, 3, 9).unwrap())
            .unwrap()
            .1
            .timestamp()
    );
    for payload in [
        json!({"preset":"custom","from":"invalid","to":"2026-03-09"}),
        json!({"preset":"custom","from":"2026-03-10","to":"2026-03-09"}),
        json!({"preset":"custom"}),
        json!({"preset":"last-century"}),
    ] {
        assert_eq!(
            rpc("/chat", "message_search_dates", &payload.to_string())
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let mut anonymous = super::auth::Client::new().await;
    assert_eq!(
        anonymous
            .request(
                "/signin",
                Some(("message_search_dates", json!({"preset":"today"})))
            )
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn workspace_search_tabs_show_matching_filters_and_keep_the_active_tab() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
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
    browser
        .type_text("filter message @Jefferson Abraham")
        .await
        .unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('textarea').value === '' && document.querySelector('[aria-label=Messages]').textContent.includes('filter message @Jefferson Abraham')").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.type_text("filter message").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.click("#global-search-tab-messages").await.unwrap();
    browser.wait_for("document.querySelectorAll('.global-search-card').length === 2 && !document.querySelector('.global-search-filters').hidden").await.unwrap();
    let matches: bool = browser.eval("(() => {const header=document.querySelector('.global-search-header').getBoundingClientRect(); const row=document.querySelector('.global-search-filters').getBoundingClientRect();const tab=document.querySelector('#global-search-tab-messages');const style=getComputedStyle(tab);return header.height===60 && row.height===41 && row.top===header.bottom && style.borderColor==='rgb(139, 143, 240)' && style.backgroundColor==='rgb(44, 45, 87)';})()").await.unwrap();
    assert!(
        matches,
        "search tabs and filter strip must match the Teams layout"
    );
    browser
        .click("button[aria-label='Filter messages by sender']")
        .await
        .unwrap();
    browser
        .wait_for("document.activeElement.getAttribute('aria-label') === 'Find message sender'")
        .await
        .unwrap();
    browser.type_text("fixture@example.com").await.unwrap();
    browser.click(".message-sender-option").await.unwrap();
    browser
        .click("button[aria-label='Filter messages by date']")
        .await
        .unwrap();
    browser.click(".message-date-preset").await.unwrap();
    browser.wait_for("!document.querySelector('.message-date-picker').matches(':popover-open') && document.querySelector('button[aria-label=\"Filter messages by date\"]').textContent.includes('Today')").await.unwrap();
    browser
        .click("button[aria-label='Filter messages by date']")
        .await
        .unwrap();
    // Exercise the native date field with trusted keyboard input. Its min/max
    // constraints prevent choosing an inverted range; the RPC also checks it.
    let previous_date: String = browser
        .eval("document.querySelector('input[aria-label=\"Message date to\"]').value")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Message date to']")
        .await
        .unwrap();
    browser.press("ArrowUp").await.unwrap();
    browser.press("Tab").await.unwrap();
    assert_ne!(
        browser
            .eval::<String>("document.querySelector('input[aria-label=\"Message date to\"]').value")
            .await
            .unwrap(),
        previous_date,
        "native date entry must change the selected date"
    );
    browser.wait_for("document.querySelector('input[aria-label=\"Message date from\"]').max === document.querySelector('input[aria-label=\"Message date to\"]').value && !document.querySelector('button[aria-label=\"Apply message date\"]').disabled").await.unwrap();
    browser
        .click("button[aria-label='Apply message date']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.message-date-picker').matches(':popover-open') && document.querySelectorAll('.global-search-card').length === 2").await.unwrap();
    browser
        .click("button[aria-label='Filter messages by date']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Clear message date']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.message-date-picker').matches(':popover-open') && document.querySelector('button[aria-label=\"Filter messages by date\"]').textContent.trim() === 'Date'").await.unwrap();
    browser
        .click("button[aria-label='More message filters']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.message-more-dialog').open && document.querySelector('button[aria-label=\"Apply message filters\"]').disabled").await.unwrap();
    browser
        .click("input[aria-label='Has attachment']")
        .await
        .unwrap();
    assert_eq!(
        browser
            .eval::<usize>("document.querySelectorAll('.global-search-card').length")
            .await
            .unwrap(),
        2,
        "draft filters must wait for Apply"
    );
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.message-more-dialog').open && !document.querySelector('.global-search-view').hidden").await.unwrap();
    browser
        .click("button[aria-label='More message filters']")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Has attachment']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Apply message filters']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('.global-search-card').length === 0")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More message filters']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Clear message filters']")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Mentions me']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Apply message filters']")
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('.global-search-card').length === 1 && document.querySelector('.global-search-card').textContent.includes('@Jefferson Abraham') && !document.querySelector('.message-more-dialog').open").await.unwrap();
    browser.click("input[aria-label=Search]").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('#global-search-tab-messages').getAttribute('aria-selected') === 'true' && document.querySelectorAll('.global-search-card').length === 1").await.unwrap();
    browser.click("#global-search-tab-files").await.unwrap();
    browser
        .click("button[aria-label='Filter files by type']")
        .await
        .unwrap();
    browser
        .click("#dropdown-menu-search-file-type [role=menuitem]:nth-child(5)")
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
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/communities"))
        .await
        .unwrap();
    let _: bool = browser.eval("(Array.from(document.querySelectorAll('main button')).find(button => button.getAttribute('aria-label') === 'Create community') || Array.from(document.querySelectorAll('main button')).find(button => button.textContent.trim() === 'Create your own')).click(); true").await.unwrap();
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
    browser.wait_for("document.querySelector('.global-search-view').hidden && document.activeElement.textContent.includes('Searchable weekend community') && document.activeElement.getAttribute('aria-current') === 'page' && document.querySelector('.community-identity-title h2').textContent === 'Searchable weekend community'").await.unwrap();
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
    for (id, name) in [
        ("search-loop", "search-notes.loop"),
        ("search-onenote", "search-notes.ONE"),
    ] {
        chat_file::ActiveModel {
            id: Set(id.into()),
            owner_id: Set(owner.id.clone()),
            name: Set(name.into()),
            content_type: Set("application/octet-stream".into()),
            bytes: Set(10),
            file_id: Set("test-storage-note".into()),
            file_key: Set("private-storage-key".into()),
            created_at: Set(crate::rahti::time::Utc::now().timestamp()),
            folder_id: Set(None),
        }
        .insert(crate::db::db())
        .await
        .unwrap();
    }
    for (kind, date, expected) in [
        ("pdf", "day", 1),
        ("pdf", "any", 1),
        ("text", "any", 2),
        ("text", "day", 0),
        ("images", "any", 0),
        ("loop", "today", 1),
        ("onenote", "today", 1),
        ("pdf", "today", 1),
        ("pdf", "yesterday", 0),
        ("pdf", "week", 1),
        ("pdf", "month", 1),
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
        json!({"query":"search","file_date":"custom","file_from":"invalid","file_to":"2026-10-28"}),
        json!({"query":"search","file_date":"custom","file_from":"2026-10-29","file_to":"2026-10-28"}),
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
    let Some(mut browser) = launch_browser().await else {
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
