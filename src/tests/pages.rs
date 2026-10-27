use super::*;

#[tokio::test]
async fn opening_a_header_menu_closes_meet_popovers() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    for header in ["Settings and more", "Your profile"] {
        for (trigger, panel) in [
            (
                "button[aria-controls='popover-meeting-join']",
                "#popover-meeting-join",
            ),
            (
                "button[aria-controls='dropdown-menu-meeting-schedule']",
                "#dropdown-menu-meeting-schedule",
            ),
            (
                "button[aria-controls='popover-meeting-compose']",
                "#popover-meeting-compose input[name=title]",
            ),
        ] {
            browser.click(trigger).await.unwrap();
            browser
                .wait_for(&format!(
                    "document.querySelector({panel:?}).getBoundingClientRect().height > 0"
                ))
                .await
                .unwrap();
            browser
                .click(&format!("button[aria-label='{header}']"))
                .await
                .unwrap();
            browser.wait_for(&format!("document.querySelector({panel:?}).getBoundingClientRect().height === 0 && document.querySelector('button[aria-label=\"{header}\"]').getAttribute('aria-expanded') === 'true'")).await.unwrap();
            browser
                .click(&format!("button[aria-label='{header}']"))
                .await
                .unwrap();
            assert!(
                browser
                    .eval::<bool>(&format!(
                        "document.querySelector({panel:?}).getBoundingClientRect().height === 0"
                    ))
                    .await
                    .unwrap(),
                "closing the header must not reopen the page popover"
            );
        }
    }
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn shell_dropdowns_stay_above_meet_actions() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    browser
        .click("button[aria-label='Settings and more']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Settings and more\"]').getAttribute('aria-expanded') === 'true'").await.unwrap();
    let unobscured: bool = browser.eval("(() => { const action = document.querySelector('button[aria-controls=\"popover-meeting-join\"]').getBoundingClientRect(); const menu = Array.from(document.querySelectorAll('[role=menuitem]')).find(el => el.textContent.includes('Tell us how you use Teams')); const rect = menu.getBoundingClientRect(); const x = Math.max(rect.left, action.left) + 5; const y = Math.max(rect.top, action.top) + 5; return x < Math.min(rect.right, action.right) && y < Math.min(rect.bottom, action.bottom) && menu.contains(document.elementFromPoint(x, y)); })()").await.unwrap();
    assert!(
        unobscured,
        "header dropdown must cover the overlapping Meet action"
    );
    browser
        .click("button[aria-controls='popover-meeting-join']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Settings and more\"]').getAttribute('aria-expanded') === 'false'").await.unwrap();
    assert!(
        browser
            .eval::<bool>(
                "!document.querySelector('#popover-meeting-join').matches(':popover-open')"
            )
            .await
            .unwrap(),
        "the dismissal click must not reach the page action"
    );
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// A context provider must not shrink-wrap the page or remove it from the
/// shell's flex layout. Check the hydrated geometry, not just the markup.
#[tokio::test]
async fn shell_pages_fill_the_content_area() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    for path in ["/meet", "/chat"] {
        browser
            .goto(&format!("http://{address}{path}"))
            .await
            .unwrap();
        browser
            .wait_for("document.querySelector('.shortcuts-row') !== null")
            .await
            .unwrap();
        let fills: bool = browser.eval("(() => { const page = document.querySelector('main').getBoundingClientRect(); return Math.abs(page.right - innerWidth) <= 1 && Math.abs(page.bottom - innerHeight) <= 1 && Math.abs(page.top - 48) <= 1; })()").await.unwrap();
        assert!(
            fills,
            "{path} must fill the width and height beside the rail"
        );
    }
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn chat_search_and_meet_now_controls_work() {
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
        .click("button[aria-label='Search chats']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.chat-list-search').hidden && document.activeElement.getAttribute('aria-label') === 'Filter by person or chat name'").await.unwrap();
    browser.type_text("No matching person").await.unwrap();
    browser
        .wait_for("document.querySelector('.chat-self-entry').hidden")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("document.querySelector('.chat-list-search').hidden && !document.querySelector('.chat-self-entry').hidden").await.unwrap();
    browser
        .click("button[aria-label='Meet now']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.chat-meet-now').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'Meeting name'").await.unwrap();
    browser
        .type_text("Meeting from chat controls")
        .await
        .unwrap();
    browser.click(".chat-meeting-share-button").await.unwrap();
    browser.wait_for("document.querySelector('input[aria-label=\"Meeting link\"]').value.includes('/meet/j/') && !document.querySelector('.chat-meeting-share-button').disabled").await.unwrap();
    let shared: String = browser
        .eval("document.querySelector('input[aria-label=\"Meeting link\"]').value")
        .await
        .unwrap();
    let id = shared.rsplit('/').next().unwrap();
    assert_eq!(
        crate::meetings::find(id).await.unwrap().unwrap().title,
        "Meeting from chat controls"
    );
    // Observe tab creation without opening an uncontrolled window in the suite.
    let _: bool = browser.eval(r#"
        window.open = (url, target) => {
            window.chatOpenedURL = url;
            window.chatOpenedTarget = target;
            window.chatOpenedTab = { opener: window, closed:false, location:{ href:'about:blank', replace(url) { window.chatOpenedURL=url; } }, close() { this.closed=true; } };
            return window.chatOpenedTab;
        }; true
    "#).await.unwrap();
    browser.click(".chat-meeting-start").await.unwrap();
    browser
        .wait_for(
            "window.chatOpenedTarget === '_blank' && window.chatOpenedURL.includes('/meet/j/')",
        )
        .await
        .unwrap();
    let reused: bool = browser
        .eval(&format!("window.chatOpenedURL === '/meet/j/{id}'"))
        .await
        .unwrap();
    assert!(
        reused,
        "Start must reuse the link already offered for sharing"
    );
    browser.press("Escape").await.unwrap();
    browser
        .click("button[aria-label='Meet now']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.chat-meet-now').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'Meeting name' && document.activeElement.selectionEnd === document.activeElement.value.length").await.unwrap();
    browser.type_text("Direct meeting from chat").await.unwrap();
    browser.wait_for("document.querySelector('.chat-meeting-share').hidden && !document.querySelector('.chat-meeting-start').disabled").await.unwrap();
    browser.click(".chat-meeting-start").await.unwrap();
    browser
        .wait_for("!document.querySelector('.chat-meeting-start').disabled")
        .await
        .unwrap();
    browser.wait_for("window.chatOpenedURL !== 'about:blank' && window.chatOpenedURL.includes('/meet/j/') && window.chatOpenedTab.opener === null && !document.querySelector('.chat-meeting-start').disabled").await.unwrap();
    let direct: String = browser.eval("window.chatOpenedURL").await.unwrap();
    assert_eq!(
        crate::meetings::find(direct.rsplit('/').next().unwrap())
            .await
            .unwrap()
            .unwrap()
            .title,
        "Direct meeting from chat"
    );
    browser
        .wait_for("document.activeElement.getAttribute('aria-label') === 'Meeting name'")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.chat-meet-now').matches(':popover-open')")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// Mount the real runtime under the framework's Trusted Types CSP. Server
/// rendering alone cannot catch a script sink that prevents hydration.
#[tokio::test]
async fn chat_mounts_and_updates_under_trusted_types() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        eprintln!("Skipping chat browser test: Chrome or Edge is not installed.");
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();

    let favorites = "Array.from(document.querySelectorAll('button')).find(button => button.textContent.trim() === 'Favorites')";
    browser
        .wait_for(&format!(
            "{favorites}?.getAttribute('aria-expanded') === 'true'"
        ))
        .await
        .unwrap();
    let selector = "aside button[aria-expanded='true']";
    browser.click(selector).await.unwrap();
    browser
        .wait_for(&format!("{favorites}?.getAttribute('aria-expanded') === 'false' && {favorites}?.nextElementSibling.hidden"))
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// The root is where the desktop client opens: Chat.
#[tokio::test]
async fn the_root_redirects_to_chat() {
    let (status, _) = get("/").await;
    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
}

/// Chat, inside the root layout's document, with the shell around it.
#[tokio::test]
async fn the_chat_page_renders_inside_the_shell() {
    let (status, body) = get("/chat").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("<!DOCTYPE html>"), "{body}");
    assert!(body.contains("This is your space"), "{body}");
    assert!(body.contains("Type a message"), "{body}");
    assert!(
        body.contains("lucide-message-circle"),
        "shell icons use rahti-icons"
    );
    assert!(
        body.contains("data-slot=\"avatar\""),
        "avatars use Rahti UI"
    );
    assert!(
        body.contains("data-slot=\"avatar-fallback\""),
        "initials remain available"
    );
}

/// Every rail destination answers, and names itself in its title.
#[tokio::test]
async fn every_rail_destination_answers() {
    let sections = [
        ("/chat", "Chat"),
        ("/meet", "Meet"),
        ("/people", "People"),
        ("/communities", "Communities"),
        ("/calendar", "Calendar"),
        ("/activity", "Activity"),
        ("/apps", "Apps"),
    ];

    for (url, title) in sections {
        let (status, body) = get(url).await;

        assert_eq!(status, StatusCode::OK, "{url}");
        assert!(
            body.contains(&format!(">{title} | upurnaka</title>")),
            "{url} did not title itself {title}"
        );
    }
}

/// The rail is chrome every section shares, so it is on every page, and the
/// section being shown is the one marked current.
#[tokio::test]
async fn the_rail_marks_the_section_being_shown() {
    let (_, body) = get("/calendar").await;

    assert!(body.contains("href=\"/chat\""), "{body}");
    assert!(body.contains("href=\"/activity\""), "{body}");
    assert!(
        body.contains("href=\"/calendar\" class=\"group relative flex h-11 w-11 items-center justify-center rounded-md bg-raised text-brand-ink\""),
        "the calendar rail item is not the selected one"
    );
}

/// The usage survey from the title bar menu ships with the shell, so it is on
/// every page rather than only the one it was written beside.
#[tokio::test]
async fn the_usage_survey_ships_with_the_shell() {
    let (_, body) = get("/meet").await;

    assert!(
        body.contains("What is your primary use for Teams?"),
        "{body}"
    );
    assert!(body.contains("Choose a category"), "{body}");
}

/// A URL no `page.rs` claims: the 404 page, not a panic.
#[tokio::test]
async fn an_unknown_url_is_a_404() {
    let (status, _) = get("/this-page-does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// The shell opens a native modal, while filtering and collapsing stay in
/// the catalog's component so typing cannot disturb the page underneath.
#[tokio::test]
async fn keyboard_shortcuts_dialog_filters_and_handles_focus() {
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
        .wait_for("document.querySelector('.shortcuts-row') !== null")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Settings and more']")
        .await
        .unwrap();
    browser
        .click("[role=menuitem][aria-label='Keyboard shortcuts']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.shortcuts-dialog').open && document.querySelector('.shortcuts-dialog').contains(document.activeElement)").await.unwrap();
    browser
        .wait_for("document.querySelectorAll('.shortcuts-section').length === 7")
        .await
        .unwrap();
    browser.click(".shortcuts-section-toggle").await.unwrap();
    browser.wait_for("document.querySelector('.shortcuts-section-toggle').getAttribute('aria-expanded') === 'false'").await.unwrap();
    browser
        .click("input[aria-label='Filter shortcuts']")
        .await
        .unwrap();
    browser.type_text("zoom").await.unwrap();
    browser.wait_for("document.querySelectorAll('.shortcuts-row').length === 6 && Array.from(document.querySelectorAll('.shortcuts-section-toggle')).every(button => button.getAttribute('aria-expanded') === 'true')").await.unwrap();
    browser.type_text(" no matching shortcut").await.unwrap();
    browser.wait_for("!document.querySelector('.shortcuts-empty').hidden && document.querySelectorAll('.shortcuts-row').length === 0").await.unwrap();
    // Closing the native modal restores focus to chrome.
    browser.click(".shortcuts-title button").await.unwrap();
    browser.wait_for("!document.querySelector('.shortcuts-dialog').open && document.activeElement.getAttribute('aria-label') === 'Settings and more'").await.unwrap();
    // This browser helper cannot hold Ctrl; dispatch the modifier chord explicitly.
    let _: bool = browser.eval("document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {key:'.', ctrlKey:true, bubbles:true, cancelable:true})); true").await.unwrap();
    browser
        .wait_for("document.querySelector('.shortcuts-dialog').open")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.shortcuts-dialog').open && document.activeElement.getAttribute('aria-label') === 'Settings and more'").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// Shortcuts work from body focus and continue to use current page state.
#[tokio::test]
async fn shortcuts_control_chat_from_body_and_composer() {
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
        .wait_for("document.querySelector('.shortcuts-row') !== null")
        .await
        .unwrap();
    // The bundled browser driver cannot send modifier chords. These exercise
    // the browser's actual document handlers with different focus targets.
    let _: bool = browser
        .eval("document.activeElement?.blur(); document.activeElement === document.body")
        .await
        .unwrap();
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'.',code:'Period',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser
        .wait_for("document.querySelector('.shortcuts-dialog').open")
        .await
        .unwrap();
    // Let the native modal finish its opening and focus steps before sending Escape.
    let _: bool = browser.eval("new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve(true))))").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.shortcuts-dialog').open && document.activeElement.getAttribute('aria-label') === 'Settings and more'")
        .await
        .unwrap();
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'r',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser
        .wait_for("document.activeElement.getAttribute('aria-label') === 'Type a message'")
        .await
        .unwrap();
    // Input.insertText supplies multiline text; Shift+Enter is also checked
    // with trusted input in the interactive browser (the driver omits modifiers).
    browser.type_text("First line\nSecond line").await.unwrap();
    let _: bool = browser.eval("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'x',ctrlKey:true,shiftKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser.wait_for("document.querySelector('textarea').rows === 5 && document.querySelector('textarea').value === 'First line\\nSecond line'").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .wait_for("document.querySelector('textarea').value.endsWith('\\n')")
        .await
        .unwrap();
    let _: bool = browser.eval("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser.wait_for("document.querySelector('textarea').value === '' && document.querySelector('[aria-label=Messages]').textContent.includes('Second line')").await.unwrap();
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'f',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser.wait_for("document.activeElement.getAttribute('aria-label') === 'Find messages in this conversation'").await.unwrap();
    browser.type_text("absent text").await.unwrap();
    browser.wait_for("document.querySelector('.chat-conversation-search').textContent.includes('0 matching messages')").await.unwrap();
    browser.press("Escape").await.unwrap();
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'f',ctrlKey:true,shiftKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser
        .wait_for(
            "document.activeElement.getAttribute('aria-label') === 'Filter by person or chat name'",
        )
        .await
        .unwrap();
    browser.type_text("no match").await.unwrap();
    browser
        .wait_for("document.querySelector('.chat-self-entry').hidden")
        .await
        .unwrap();
    let _: bool = browser.eval("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'z',ctrlKey:true,altKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser.wait_for("!document.querySelector('.chat-self-entry').hidden && document.querySelector('[aria-label=\"Filter by person or chat name\"]').value === ''").await.unwrap();
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'e',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser
        .wait_for("document.activeElement.getAttribute('aria-label') === 'Search'")
        .await
        .unwrap();
    browser.type_text("calendar").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .wait_for("location.pathname === '/calendar'")
        .await
        .unwrap();
    // SPA navigation must clean up the old chat handler and retain the shell.
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'.',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser
        .wait_for("document.querySelector('.shortcuts-dialog').open")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// The new conversation view keeps the self-chat intact, supports recipient
/// chips and group names, and turns a first message into a session-local thread.
#[tokio::test]
async fn new_chat_composes_a_group_and_preserves_other_drafts() {
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
    browser.type_text("Saved self draft").await.unwrap();
    browser
        .click("button[aria-label='New chat']")
        .await
        .unwrap();
    browser.wait_for("document.activeElement.getAttribute('aria-label') === 'Recipients' && !document.querySelector('.new-chat-empty').hidden && document.querySelector('textarea').value === ''").await.unwrap();
    browser.type_text("not-an-email@").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("!document.querySelector('.new-chat-recipient-error').hidden && document.querySelectorAll('.new-chat-chip').length === 0").await.unwrap();
    browser
        .click("textarea[aria-label='Type a message']")
        .await
        .unwrap();
    browser.type_text("First group draft").await.unwrap();
    let blocked: bool = browser
        .eval("document.querySelector('button[aria-label=Send]').disabled")
        .await
        .unwrap();
    assert!(
        blocked,
        "a new conversation requires recipients before sending"
    );
    browser
        .click("aside .chat-self-entry button")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('textarea').value === 'Saved self draft' && document.querySelector('.new-chat-header').hidden").await.unwrap();
    browser
        .click("button[aria-label='New message']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('textarea').value === 'First group draft' && document.activeElement.getAttribute('aria-label') === 'Recipients'").await.unwrap();
    // Replace the invalid recipient left in the draft with a valid one.
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser.type_text("alex@example.com").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelectorAll('.new-chat-chip').length === 1 && document.querySelector('[aria-label=Recipients]').value === ''").await.unwrap();
    browser.type_text("Avery").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .wait_for("document.querySelectorAll('.new-chat-chip').length === 2")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Remove Avery']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('.new-chat-chip').length === 1")
        .await
        .unwrap();
    browser.type_text("Avery").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .click("button[aria-label='Show group name']")
        .await
        .unwrap();
    browser.wait_for("document.activeElement.getAttribute('aria-label') === 'Group name' && !document.querySelector('.new-chat-group').hidden").await.unwrap();
    browser.type_text("Project team").await.unwrap();
    browser
        .click("button[aria-label='Hide group name']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.new-chat-group').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Show group name']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('[aria-label=\"Group name\"]').value === 'Project team'")
        .await
        .unwrap();
    browser.click("button[aria-label=Send]").await.unwrap();
    browser.wait_for("document.querySelector('.new-chat-header').hidden && document.querySelector('main h2').textContent.includes('Project team') && document.querySelector('[aria-label=Messages]').textContent.includes('First group draft')").await.unwrap();
    browser
        .click("aside .chat-self-entry button")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('textarea').value === 'Saved self draft' && !document.querySelector('[aria-label=Messages]').textContent.includes('First group draft')").await.unwrap();
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'n',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser.wait_for("!document.querySelector('.new-chat-header').hidden && document.activeElement.getAttribute('aria-label') === 'Recipients'").await.unwrap();
    let fills: bool = browser.eval("Math.abs(document.querySelector('main').getBoundingClientRect().right-innerWidth) <= 1 && Math.abs(document.querySelector('main').getBoundingClientRect().bottom-innerHeight) <= 1").await.unwrap();
    assert!(fills, "new chat must retain the full-height shell layout");
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn calendar_meet_now_shares_and_starts_the_same_meeting() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/calendar"))
        .await
        .unwrap();
    browser
        .click("button[aria-label='Join with an ID']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Meet now']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.calendar-meet-now').matches(':popover-open') && !document.querySelector('.calendar-join-dialog').matches(':popover-open') && document.activeElement.name === 'title' && document.activeElement.selectionEnd === document.activeElement.value.length").await.unwrap();
    browser.type_text("Shared Calendar meeting").await.unwrap();
    let _: bool = browser.eval("Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text => {window.calendarCopiedLink=text;}}}); true").await.unwrap();
    browser.click(".calendar-meet-share-button").await.unwrap();
    browser.wait_for("!document.querySelector('.calendar-meet-share').hidden && document.querySelector('.calendar-meet-share [role=status]').textContent.includes('copied') && !document.querySelector('.calendar-meet-start').disabled").await.unwrap();
    let link: String = browser.eval("window.calendarCopiedLink").await.unwrap();
    let id = link.rsplit('/').next().unwrap();
    assert_eq!(
        crate::meetings::find(id).await.unwrap().unwrap().title,
        "Shared Calendar meeting"
    );
    let _: bool = browser.eval("window.calendarOriginalOpen = window.open.bind(window); window.open = (...args) => {window.calendarMeetWindow = window.calendarOriginalOpen(...args); return window.calendarMeetWindow;}; true").await.unwrap();
    browser.click(".calendar-meet-start").await.unwrap();
    browser.wait_for(&format!("window.calendarMeetWindow?.location.pathname === '/meet/j/{id}' && window.calendarMeetWindow.document.body.textContent.includes('Shared Calendar meeting')")).await.unwrap();
    let detached: bool = browser.eval("window.calendarMeetWindow.opener === null && location.pathname === '/calendar' && !document.querySelector('.calendar-meet-now').matches(':popover-open')").await.unwrap();
    assert!(
        detached,
        "Start reuses the shared meeting and leaves Calendar open"
    );
    browser
        .wait_for(
            "Array.from(window.calendarMeetWindow.document.images).every(image => image.complete)",
        )
        .await
        .unwrap();
    let _: bool = browser.eval("window.calendarMeetWindow.close(); window.open = () => null; navigator.clipboard.writeText = async () => {throw new Error('Clipboard unavailable');}; true").await.unwrap();
    browser
        .click("button[aria-label='Meet now']")
        .await
        .unwrap();
    browser.type_text("Blocked Calendar meeting").await.unwrap();
    browser.click(".calendar-meet-share-button").await.unwrap();
    browser.wait_for("document.querySelector('.calendar-meet-share [role=status]').textContent.includes('Select the link') && !document.querySelector('.calendar-meet-start').disabled").await.unwrap();
    browser.click(".calendar-meet-start").await.unwrap();
    browser.wait_for("document.querySelector('.calendar-meet-share [role=status]').textContent.includes('Use Open meeting') && !document.querySelector('.calendar-meet-start').disabled").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.calendar-meet-now').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'Meet now'").await.unwrap();
    browser
        .click("button[aria-label='Meet now']")
        .await
        .unwrap();
    browser.click("main h1").await.unwrap();
    browser
        .wait_for("!document.querySelector('.calendar-meet-now').matches(':popover-open')")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn calendar_joins_a_meeting_by_id_and_handles_dismissal_and_blocked_windows() {
    setup().await;
    let meeting = crate::meetings::create("Calendar join test".into())
        .await
        .unwrap();
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/calendar"))
        .await
        .unwrap();
    browser
        .click("button[aria-label='Join with an ID']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.calendar-join-dialog').matches(':popover-open') && document.activeElement.name === 'meeting' && document.querySelector('.calendar-join-dialog button[type=submit]').disabled").await.unwrap();
    let fits: bool = browser.eval("(() => {const box = document.querySelector('.calendar-join-dialog').getBoundingClientRect(); return box.left >= 0 && box.right <= innerWidth && box.top >= 0 && box.bottom <= innerHeight;})()").await.unwrap();
    assert!(fits, "the join popover must fit in the calendar viewport");
    browser.type_text("bad/id").await.unwrap();
    browser
        .click(".calendar-join-dialog button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.calendar-join-dialog [role=alert]').hidden && document.activeElement.name === 'meeting'").await.unwrap();
    let _: bool = browser.eval("document.activeElement.select(); window.originalJoinOpen = window.open.bind(window); window.open = (...args) => {window.calendarJoinWindow = window.originalJoinOpen(...args); return window.calendarJoinWindow;}; true").await.unwrap();
    browser.type_text(&meeting.id).await.unwrap();
    browser
        .click(".calendar-join-dialog button[type=submit]")
        .await
        .unwrap();
    browser.wait_for(&format!("window.calendarJoinWindow?.location.pathname === '/meet/j/{}' && window.calendarJoinWindow.document.body.textContent.includes('Calendar join test')", meeting.id)).await.unwrap();
    let detached: bool = browser.eval("window.calendarJoinWindow.opener === null && !document.querySelector('.calendar-join-dialog').matches(':popover-open') && location.pathname === '/calendar'").await.unwrap();
    assert!(
        detached,
        "joining opens a separate, detached pre-join window and preserves Calendar"
    );
    let _: bool = browser
        .eval("window.calendarJoinWindow.close(); true")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Join with an ID']")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.calendar-join-dialog').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'Join with an ID'").await.unwrap();
    browser
        .click("button[aria-label='Join with an ID']")
        .await
        .unwrap();
    browser.click("main h1").await.unwrap();
    browser
        .wait_for("!document.querySelector('.calendar-join-dialog').matches(':popover-open')")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.open = () => null; true")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Join with an ID']")
        .await
        .unwrap();
    browser
        .click(".calendar-join-dialog button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.calendar-join-dialog a').hidden && document.querySelector('.calendar-join-dialog [role=alert]').textContent.includes('Allow pop-ups')").await.unwrap();
    let fallback: String = browser
        .eval("document.querySelector('.calendar-join-dialog a').getAttribute('href')")
        .await
        .unwrap();
    assert_eq!(fallback, format!("/meet/j/{}", meeting.id));
    browser
        .click("button[aria-label='Close meeting ID dialog']")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn emoji_picker_searches_inserts_at_the_cursor_and_dismisses() {
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
    browser.click("textarea").await.unwrap();
    browser.type_text("Hello world").await.unwrap();
    let _: bool = browser
        .eval("document.querySelector('textarea').setSelectionRange(6,11); true")
        .await
        .unwrap();
    browser.click("button[aria-label=Emoji]").await.unwrap();
    browser.wait_for("document.querySelector('.chat-emoji-picker').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'Find something fun' && document.querySelectorAll('.chat-emoji-grid button').length === 12").await.unwrap();
    let fits: bool = browser.eval("(() => {const box = document.querySelector('.chat-emoji-picker').getBoundingClientRect(); const trigger = document.querySelector('button[aria-label=Emoji]').getBoundingClientRect(); return box.left >= 0 && box.right <= innerWidth && box.top >= 0 && box.bottom <= trigger.top;})()").await.unwrap();
    assert!(
        fits,
        "the picker must fit above the composer inside the viewport"
    );
    browser.type_text("coffee").await.unwrap();
    browser
        .wait_for("document.querySelectorAll('.chat-emoji-grid button').length === 1")
        .await
        .unwrap();
    browser.click(".chat-emoji-grid button").await.unwrap();
    browser.wait_for("document.querySelector('textarea').value === 'Hello ☕' && document.activeElement.tagName === 'TEXTAREA' && document.activeElement.selectionStart === 7 && !document.querySelector('.chat-emoji-picker').matches(':popover-open')").await.unwrap();
    browser.type_text(" time").await.unwrap();
    browser.click("button[aria-label=Emoji]").await.unwrap();
    browser
        .click(".chat-emoji-tabs label:nth-of-type(2)")
        .await
        .unwrap();
    browser.click(".chat-emoji-search input").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser.type_text("zzzznoemoji").await.unwrap();
    browser.wait_for("document.querySelectorAll('.chat-emoji-grid button').length === 0 && !document.querySelector('.chat-emoji-empty').hidden").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser.press("Backspace").await.unwrap();
    browser
        .click("button[title='Food and drink']")
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('.chat-emoji-grid button').length === 12 && document.querySelector('.chat-emoji-grid').textContent.includes('🍕')").await.unwrap();
    browser
        .click(".chat-emoji-tabs label:nth-of-type(2)")
        .await
        .unwrap();
    // The media tabs are one radio group: from the chosen tab, the arrow keys
    // move the choice.
    let _: bool = browser
        .eval("document.querySelector('.chat-emoji-tabs input:checked').focus(); true")
        .await
        .unwrap();
    browser.press("ArrowRight").await.unwrap();
    browser.wait_for("document.querySelector('.chat-emoji-tabs label:nth-of-type(3) input').checked && !document.querySelector('.chat-giphy-attribution').hidden").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.chat-emoji-picker').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'Emoji'").await.unwrap();
    browser.click("button[aria-label=Emoji]").await.unwrap();
    browser.click("main h2").await.unwrap();
    browser
        .wait_for("!document.querySelector('.chat-emoji-picker').matches(':popover-open')")
        .await
        .unwrap();
    browser.click("button[aria-label=Send]").await.unwrap();
    browser.wait_for("document.querySelector('[aria-label=Messages]').textContent.includes('Hello ☕ time') && document.querySelector('textarea').value === ''").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn new_message_menu_opens_a_window_and_discards_only_its_draft() {
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
    browser.click("textarea").await.unwrap();
    browser.type_text("Keep my self draft").await.unwrap();
    browser
        .click("button[aria-label='New chat']")
        .await
        .unwrap();
    browser.type_text("alex@example.com").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .click("button[aria-label='Show group name']")
        .await
        .unwrap();
    browser.type_text("Window team").await.unwrap();
    browser.click("textarea").await.unwrap();
    browser.type_text("Draft for another window").await.unwrap();
    browser
        .click("button[aria-label='New message options']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.new-chat-menu').matches(':popover-open') && document.activeElement.textContent.includes('Open in new window')").await.unwrap();
    browser.press("ArrowDown").await.unwrap();
    browser
        .wait_for("document.activeElement.textContent.includes('Discard')")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('.new-chat-menu').matches(':popover-open') && document.activeElement.getAttribute('aria-label') === 'New message options'").await.unwrap();
    browser
        .click("button[aria-label='New message options']")
        .await
        .unwrap();
    browser.click(".new-chat-empty").await.unwrap();
    browser
        .wait_for("!document.querySelector('.new-chat-menu').matches(':popover-open')")
        .await
        .unwrap();
    // Keep a handle to the real second window so its restored state can be checked.
    let _: bool = browser.eval("window.originalOpen = window.open.bind(window); window.open = (...args) => { window.draftPopup = window.originalOpen(...args); return window.draftPopup; }; true").await.unwrap();
    browser
        .click("button[aria-label='New message options']")
        .await
        .unwrap();
    browser
        .click(".new-chat-menu [role=menuitem]:first-child")
        .await
        .unwrap();
    browser.wait_for("window.draftPopup && window.draftPopup.location.pathname === '/chat' && window.draftPopup.document.querySelector('textarea')?.value === 'Draft for another window' && window.draftPopup.document.querySelectorAll('.new-chat-chip').length === 1 && window.draftPopup.document.querySelector('[aria-label=\"Group name\"]')?.value === 'Window team'").await.unwrap();
    let detached: bool = browser.eval("window.draftPopup.opener === null && window.draftPopup.sessionStorage.getItem('upurnaka.chat.compose') === null").await.unwrap();
    assert!(
        detached,
        "the restored window must detach its opener and consume its transfer data"
    );
    let _: bool = browser
        .eval("window.draftPopup.close(); true")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('textarea').value === 'Draft for another window'")
        .await
        .unwrap();
    // A blocked popup must leave the source draft available.
    let _: bool = browser
        .eval("window.open = () => null; true")
        .await
        .unwrap();
    browser
        .click("button[aria-label='New message options']")
        .await
        .unwrap();
    browser
        .click(".new-chat-menu [role=menuitem]:first-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('main').textContent.includes('Allow pop-ups') && document.querySelector('textarea').value === 'Draft for another window'").await.unwrap();
    browser
        .click("button[aria-label='New message options']")
        .await
        .unwrap();
    browser
        .click(".new-chat-menu [role=menuitem]:last-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.new-chat-header').hidden && document.querySelector('textarea').value === 'Keep my self draft' && document.querySelector('button[aria-label=\"New message\"]').parentElement.hidden").await.unwrap();
    browser
        .click("button[aria-label='New chat']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('textarea').value === '' && document.querySelectorAll('.new-chat-chip').length === 0 && document.querySelector('[aria-label=Recipients]').value === '' && document.querySelector('[aria-label=\"Group name\"]').value === ''").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
