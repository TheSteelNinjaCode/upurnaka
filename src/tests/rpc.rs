use super::*;

/// The composer's round trip: text in, a stamped message out.
#[tokio::test]
async fn the_composer_sends_a_message() {
    let (status, body) = rpc("/chat", "send_message", r#"{"text":"A note to myself"}"#).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("A note to myself"), "{body}");
    assert!(body.contains("Jefferson Abraham"), "{body}");
}

/// An empty message is a validation failure naming the field, not a blank line
/// in the thread.
#[tokio::test]
async fn an_empty_message_is_refused() {
    let (status, body) = rpc("/chat", "send_message", r#"{"text":"   "}"#).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("text"), "{body}");
}

/// A payload whose shape the function does not declare stays a 400.
#[tokio::test]
async fn a_malformed_message_payload_is_a_400() {
    let (status, _) = rpc("/chat", "send_message", r#"{"body":"wrong key"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Meet hands out the identifier, so the browser cannot invent one.
#[tokio::test]
async fn a_meeting_link_is_minted_by_the_server() {
    let (status, body) = rpc("/meet", "create_meeting_link", r#"{"title":"Weekly sync"}"#).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("https://meet.upurnaka.test/j/"), "{body}");
    assert!(body.contains("Weekly sync"), "{body}");
}

/// The calendar parses the form's strings before the grid is allowed to show
/// the meeting.
#[tokio::test]
async fn a_meeting_lands_on_the_day_it_names() {
    let payload = r#"{"title":"Design review","date":"2026-10-22","start":"09:30","minutes":"90"}"#;
    let (status, body) = rpc("/calendar", "create_meeting", payload).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""day":"2026-10-22""#), "{body}");
    assert!(body.contains(r#""start":9.5"#), "{body}");
    assert!(body.contains(r#""length":1.5"#), "{body}");
}

/// A time of day that does not exist is refused by field.
#[tokio::test]
async fn a_meeting_outside_the_day_is_refused() {
    let payload = r#"{"title":"Design review","date":"2026-10-22","start":"33:00","minutes":"60"}"#;
    let (status, body) = rpc("/calendar", "create_meeting", payload).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("start"), "{body}");
}

/// A length nobody offered is refused, however the request was made.
#[tokio::test]
async fn a_meeting_length_outside_the_offered_set_is_refused() {
    let payload = r#"{"title":"Design review","date":"2026-10-22","start":"09:00","minutes":"7"}"#;
    let (status, body) = rpc("/calendar", "create_meeting", payload).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("minutes"), "{body}");
}

/// Invitations are checked in Rust, so the dialog can show a field message.
#[tokio::test]
async fn an_invitation_needs_a_real_address() {
    let (ok, body) = rpc(
        "/people",
        "invite_to_teams",
        r#"{"email":"ada@example.com"}"#,
    )
    .await;
    assert_eq!(ok, StatusCode::OK);
    assert!(body.contains("\"initials\":\"AE\""), "{body}");

    let (bad, body) = rpc(
        "/people",
        "invite_to_teams",
        r#"{"email":"not-an-address"}"#,
    )
    .await;
    assert_eq!(bad, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("email"), "{body}");
}

/// A community takes a category from the list the dialog offers, and nothing
/// else — the page RPC lives on `/communities`.
#[tokio::test]
async fn a_community_takes_only_an_offered_category() {
    let (ok, body) = rpc(
        "/communities",
        "create_community",
        r#"{"name":"Saturday football","category":"Hobbies"}"#,
    )
    .await;
    assert_eq!(ok, StatusCode::OK);
    assert!(body.contains("Saturday football"), "{body}");

    let (bad, body) = rpc(
        "/communities",
        "create_community",
        r#"{"name":"Saturday football","category":"Something else"}"#,
    )
    .await;
    assert_eq!(bad, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("category"), "{body}");
}

/// The survey is a component RPC, so it answers on whichever page the shell is
/// rendering — here, Chat.
#[tokio::test]
async fn the_usage_survey_answers_from_any_page() {
    let payload = r#"{"primary_use":"Personal use","category":"Family and friends"}"#;
    let (status, body) = rpc("/chat", "save_usage_survey", payload).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Family and friends"), "{body}");
}

/// An unanswered survey question is a 422, not a saved blank.
#[tokio::test]
async fn an_unanswered_survey_is_refused() {
    let payload = r#"{"primary_use":"","category":"Family and friends"}"#;
    let (status, body) = rpc("/chat", "save_usage_survey", payload).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("primary_use"), "{body}");
}

/// A name the page does not declare is refused by name.
#[tokio::test]
async fn an_unknown_rpc_name_is_refused() {
    let (status, _) = rpc("/chat", "no_such_function", "{}").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
