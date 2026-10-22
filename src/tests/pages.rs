use super::*;

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
}

/// Every rail destination answers, and names itself in its title.
#[tokio::test]
async fn every_rail_destination_answers() {
    let sections = [
        ("/chat", "Chat"),
        ("/meet", "Meet"),
        ("/people", "People"),
        ("/copilot", "Copilot"),
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
