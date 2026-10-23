//! Meeting links, end to end: minted on `/meet`, kept, listed on the next
//! page load, and resolved at `/meet/j/{id}`.

use super::*;

/// Pull a string field out of an RPC's JSON body.
///
/// Small on purpose — the suite asserts on two or three fields of one shape,
/// and a parser dependency to read them would be more machinery than the
/// thing it reads.
fn field(body: &str, name: &str) -> String {
    let key = format!("\"{name}\":\"");
    let start = body
        .find(&key)
        .unwrap_or_else(|| panic!("no {name} in {body}"))
        + key.len();
    let rest = &body[start..];
    rest[..rest.find('"').expect("the end of the value")].to_string()
}

/// Mint a link and hand back its join code.
async fn mint(title: &str) -> String {
    let payload = format!("{{\"title\":\"{title}\"}}");
    let (status, body) = rpc("/meet", "create_meeting_link", &payload).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    field(&body, "id")
}

/// Meet offers the three ways in, and the title box the first one opens.
#[tokio::test]
async fn the_meet_page_offers_the_three_ways_in() {
    let (status, body) = get("/meet").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Create a meeting link"), "{body}");
    assert!(body.contains("Schedule a meeting"), "{body}");
    assert!(body.contains("Join with a meeting ID"), "{body}");
    assert!(body.contains("Give your meeting a title"), "{body}");
    assert!(body.contains("Create and copy link"), "{body}");
}

/// The identifier is the server's to hand out, and the link it builds points
/// at a route this application serves rather than a hostname nobody owns.
#[tokio::test]
async fn a_meeting_link_is_minted_by_the_server() {
    let (status, body) = rpc("/meet", "create_meeting_link", r#"{"title":"Weekly sync"}"#).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Weekly sync"), "{body}");

    let id = field(&body, "id");
    assert!(!id.is_empty(), "{body}");
    assert!(
        body.contains(&format!("\"url\":\"/meet/j/{id}\"")),
        "the link does not point at the join route: {body}"
    );
}

/// Minting writes a row, so the next page load lists it — the link survives
/// the browser going away, which is the whole point of saving one.
#[tokio::test]
async fn a_minted_link_is_still_listed_after_a_reload() {
    let id = mint("Persisted stand-up").await;

    let (status, body) = get("/meet").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Persisted stand-up"), "{body}");
    assert!(body.contains(&id), "the page does not list {id}");
    assert!(body.contains("just now"), "{body}");
}

/// Newest first, the way the cards are read.
#[tokio::test]
async fn the_list_puts_the_newest_link_first() {
    let older = mint("Older link in order").await;
    let newer = mint("Newer link in order").await;

    let (_, body) = get("/meet").await;

    let older_at = body.find(&older).expect("the older link");
    let newer_at = body.find(&newer).expect("the newer link");
    assert!(
        newer_at < older_at,
        "the older link was listed before the newer one"
    );
}

/// An empty title is a validation failure naming the field, not a card with
/// no name on it.
#[tokio::test]
async fn an_empty_meeting_title_is_refused() {
    let (status, body) = rpc("/meet", "create_meeting_link", r#"{"title":"   "}"#).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("title"), "{body}");
}

/// So is one nobody could read on a card.
#[tokio::test]
async fn an_overlong_meeting_title_is_refused() {
    let payload = format!("{{\"title\":\"{}\"}}", "a".repeat(200));
    let (status, body) = rpc("/meet", "create_meeting_link", &payload).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("title"), "{body}");
}

/// A payload whose shape the function does not declare stays a 400, not a
/// 422: nothing was validated, because nothing deserialized.
#[tokio::test]
async fn a_malformed_meeting_link_payload_is_a_400() {
    let (status, _) = rpc("/meet", "create_meeting_link", r#"{"name":"wrong key"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Following a minted link lands on its pre-join screen.
#[tokio::test]
async fn a_minted_link_resolves_to_its_join_page() {
    let id = mint("Resolvable review").await;

    let (status, body) = get(&format!("/meet/j/{id}")).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("Resolvable review"), "{body}");
    assert!(body.contains("Join now"), "{body}");
    assert!(body.contains(&id), "{body}");
}

/// A code nobody minted is a 404 rendered in the shell, not a panic and not
/// a blank page.
#[tokio::test]
async fn an_unknown_meeting_code_is_a_404() {
    let (status, body) = get("/meet/j/nosuchmeeting00").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("no longer available"), "{body}");
}

/// Revoking takes the card off the page and stops the link resolving — the
/// two halves of deleting one.
#[tokio::test]
async fn a_revoked_link_stops_resolving_and_stops_being_listed() {
    let id = mint("Revoked all-hands").await;

    let payload = format!("{{\"id\":\"{id}\"}}");
    let (status, _) = rpc("/meet", "delete_meeting_link", &payload).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = get(&format!("/meet/j/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, body) = get("/meet").await;
    assert!(
        !body.contains("Revoked all-hands"),
        "the revoked link is still listed"
    );
}

/// The pre-join screen says plainly when no provider is wired, rather than
/// offering a call there is nothing behind. The suite runs with no
/// `CHIME_MEDIA_REGION`, so this is the state it sees.
#[tokio::test]
async fn the_join_page_says_when_no_provider_is_configured() {
    let id = mint("Unprovisioned sync").await;

    let (status, body) = get(&format!("/meet/j/{id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("No meeting provider is configured"), "{body}");
    assert!(body.contains("CHIME_MEDIA_REGION"), "{body}");
}

/// The join RPC is a second front door onto a meeting, so it checks the code
/// the way the page does: an unknown one is a 404 before any provider call.
#[tokio::test]
async fn joining_an_unknown_code_is_refused() {
    let payload = r#"{"id":"nosuchmeeting00","display_name":"Ada"}"#;
    let (status, _) = rpc("/meet/j/nosuchmeeting00", "join_meeting", payload).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// A seat needs a name on it.
#[tokio::test]
async fn joining_without_a_name_is_refused() {
    let id = mint("Nameless attendee").await;

    let payload = format!("{{\"id\":\"{id}\",\"display_name\":\"  \"}}");
    let (status, body) = rpc(&format!("/meet/j/{id}"), "join_meeting", &payload).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("display_name"), "{body}");
}

/// Asking to join with no provider configured is a server error naming the
/// misconfiguration in the log — not a panic, and not a browser that waits
/// forever on a call nobody is going to answer.
///
/// "No provider" is guaranteed rather than assumed: `chime::settings`
/// returns `None` under `cfg(test)`, so this holds on a machine whose `.env`
/// has a real media region in it, and the suite never calls AWS.
#[tokio::test]
async fn joining_without_a_provider_is_a_server_error() {
    let id = mint("Provider-less huddle").await;

    let payload = format!("{{\"id\":\"{id}\",\"display_name\":\"Ada Lovelace\"}}");
    let (status, _) = rpc(&format!("/meet/j/{id}"), "join_meeting", &payload).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

/// Revoking without an id is a 422 naming it.
#[tokio::test]
async fn revoking_nothing_is_refused() {
    let (status, body) = rpc("/meet", "delete_meeting_link", r#"{"id":""}"#).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("id"), "{body}");
}
