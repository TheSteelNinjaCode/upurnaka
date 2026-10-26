use super::*;

/// A context provider must not shrink-wrap the page or remove it from the
/// shell's flex layout. Check the hydrated geometry, not just the markup.
#[tokio::test]
async fn shell_pages_fill_the_content_area() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
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
    browser.wait_for("!document.querySelector('.chat-meet-now').hidden && document.activeElement.getAttribute('aria-label') === 'Meeting name'").await.unwrap();
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
    browser.wait_for("!document.querySelector('.chat-meet-now').hidden && document.activeElement.getAttribute('aria-label') === 'Meeting name' && document.activeElement.selectionEnd === document.activeElement.value.length").await.unwrap();
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
        .wait_for("document.querySelector('.chat-meet-now').hidden")
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
        .click("button[aria-label='Keyboard shortcuts']")
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
    let _: bool = browser.eval("document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'.',code:'Period',ctrlKey:true,bubbles:true,cancelable:true})); true").await.unwrap();
    browser
        .wait_for("document.querySelector('.shortcuts-dialog').open")
        .await
        .unwrap();
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
