use super::*;

#[tokio::test]
async fn chat_formatting_and_importance_are_validated() {
    use rahti::serde_json::{Value, json};
    let payload = json!({"text":"Hello", "important":true, "formatting":[{"kind":"paragraph","runs":[{"text":"Hello","bold":true,"link":"https://example.com"}]}]});
    let (status, body) = rpc("/chat", "send_message", &payload.to_string()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let message: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(message["important"], true);
    assert_eq!(message["formatting"][0]["runs"][0]["bold"], true);
    for bad in [
        json!({"text":"Hello","formatting":[{"kind":"paragraph","runs":[{"text":"Hello","link":"javascript:alert(1)"}]}]}),
        json!({"text":"Hello","formatting":[{"kind":"paragraph","runs":[{"text":"Different"}]}]}),
        json!({"text":"Hello","formatting":[{"kind":"paragraph","runs":[{"text":"Hello","color":"red;display:none"}]}]}),
        json!({"text":"Hello","formatting":[{"kind":"html","runs":[{"text":"Hello"}]}]}),
    ] {
        assert_eq!(
            rpc("/chat", "send_message", &bad.to_string()).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        rpc(
            "/chat",
            "send_message",
            r#"{"text":"Hello","important":{}}"#
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, body) = rpc("/chat", "send_message", r#"{"text":"Plain"}"#).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let message: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(message["important"], false);
    assert!(message["formatting"].is_null());
}

#[tokio::test]
async fn calendar_meet_now_creates_validated_links_and_requires_a_session() {
    let (status, body) = rpc(
        "/calendar",
        "create_calendar_meeting_link",
        r#"{"title":" Calendar immediate meeting "}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let value: rahti::serde_json::Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(value["title"], "Calendar immediate meeting");
    let row = crate::meetings::find(value["id"].as_str().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.title, "Calendar immediate meeting");
    for data in [
        r#"{"title":" "}"#.to_string(),
        rahti::serde_json::json!({"title":"x".repeat(121)}).to_string(),
    ] {
        assert_eq!(
            rpc("/calendar", "create_calendar_meeting_link", &data)
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        rpc("/calendar", "create_calendar_meeting_link", "{}")
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let mut anonymous = super::auth::Client::new().await;
    assert_eq!(
        anonymous
            .request(
                "/calendar",
                Some((
                    "create_calendar_meeting_link",
                    rahti::serde_json::json!({"title":"Anonymous"})
                ))
            )
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn chat_meet_now_creates_a_persistent_validated_link() {
    let (status, body) = rpc(
        "/chat",
        "create_chat_meeting",
        r#"{"title":"  Meet now from Chat  "}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("Meet now from Chat"));
    assert!(body.contains("/meet/j/"));
    let (_, listing) = get("/meet").await;
    assert!(listing.contains("Meet now from Chat"));
    for payload in [
        r#"{"title":"   "}"#.to_string(),
        format!(r#"{{"title":"{}"}}"#, "x".repeat(121)),
    ] {
        let (status, _) = rpc("/chat", "create_chat_meeting", &payload).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (status, _) = rpc("/chat", "create_chat_meeting", r#"{"name":"Wrong field"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

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

// Meet's own round trips — minting a link, listing it, resolving it — are in
// `meetings.rs`, which covers the store and the join route with them.

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

/// A name the page does not declare is refused by name.
#[tokio::test]
async fn an_unknown_rpc_name_is_refused() {
    let (status, _) = rpc("/chat", "no_such_function", "{}").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
