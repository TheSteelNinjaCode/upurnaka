//! Who may enter a meeting: open, passcode, and invite-only links.

use super::auth::Client;
use super::*;
use crate::models::meeting_visit as visit;
use rahti::serde_json::{Value, json};
use sea_orm::EntityTrait;

async fn as_user(cookie: String) -> Client {
    let mut client = Client::new().await;
    client.with_cookie(cookie);
    // The CSRF token is bound to the session, so take a fresh one with it.
    client.request("/chat", None).await;
    client
}
// The cookies are signed with the key `setup` installs, so it runs first.
async fn owner() -> Client {
    setup().await;
    as_user(test_cookie()).await
}
async fn other() -> Client {
    setup().await;
    as_user(cookie_for("other-user", "Other User", "other@example.com")).await
}
async fn call(client: &mut Client, path: &str, name: &str, payload: Value) -> (StatusCode, Value) {
    let (s, _, b) = client.request(path, Some((name, payload))).await;
    (
        s,
        rahti::serde_json::from_str(&b).unwrap_or_else(|_| json!({"raw":b})),
    )
}
async fn meeting(title: &str) -> String {
    let (s, b) = call(
        &mut owner().await,
        "/meet",
        "create_meeting_link",
        json!({"title":title}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["access"], "open");
    b["id"].as_str().unwrap().into()
}
async fn set_access(id: &str, mode: &str, passcode: &str, invited: &[&str]) -> Value {
    let (s, b) = call(
        &mut owner().await,
        "/meet",
        "save_meeting_access",
        json!({"id":id,"mode":mode,"passcode":passcode,"invited":invited}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    b
}
async fn enter(client: &mut Client, id: &str, passcode: Option<&str>) -> (StatusCode, Value) {
    call(
        client,
        &format!("/meet/j/{id}"),
        "request_meeting_entry",
        json!({"id":id,"name":"Visitor","passcode":passcode}),
    )
    .await
}

#[tokio::test]
async fn a_passcode_meeting_refuses_the_link_alone() {
    let id = meeting("Passcode meeting").await;
    let saved = set_access(&id, "passcode", "Blue42Sky", &[]).await;
    assert_eq!(saved["mode"], "passcode");
    assert_eq!(saved["passcode"], "blue42sky");

    let mut guest = Client::new().await;
    let (status, _, body) = guest.request(&format!("/meet/j/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Enter the meeting passcode"));
    assert!(
        !body.contains("blue42sky"),
        "the page never carries the code"
    );

    assert_eq!(enter(&mut guest, &id, None).await.0, StatusCode::FORBIDDEN);
    assert_eq!(
        enter(&mut guest, &id, Some("wrong-code")).await.0,
        StatusCode::FORBIDDEN
    );
    let (status, entry) = enter(&mut guest, &id, Some(" BLUE42SKY ")).await;
    assert_eq!(status, StatusCode::OK, "{entry}");
    assert_eq!(
        entry["state"], "waiting",
        "passcode guests still wait in the lobby"
    );

    let (status, ok) = call(
        &mut guest,
        &format!("/meet/j/{id}"),
        "check_meeting_passcode",
        json!({"id":id,"passcode":"blue42sky"}),
    )
    .await;
    assert_eq!((status, ok), (StatusCode::OK, json!(true)));

    // The organizer needs no passcode and is admitted directly.
    let (status, host) = enter(&mut owner().await, &id, None).await;
    assert_eq!(status, StatusCode::OK, "{host}");
    assert_eq!(host["state"], "admitted");

    // The organizer's list carries the code so Share link can include it.
    let (_, _, page) = owner().await.request("/meet", None).await;
    assert!(page.contains("blue42sky"));
}

#[tokio::test]
async fn repeated_wrong_passcodes_are_throttled() {
    let id = meeting("Guessing target").await;
    set_access(&id, "passcode", "", &[]).await;
    let mut guest = Client::new().await;
    let mut last = StatusCode::OK;
    for _ in 0..25 {
        last = enter(&mut guest, &id, Some("nope1234")).await.0;
    }
    assert_eq!(last, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn an_invite_only_meeting_admits_only_invited_accounts() {
    let id = meeting("Invite only").await;
    let saved = set_access(&id, "invited", "", &["other-user"]).await;
    assert_eq!(saved["invited"][0]["id"], "other-user");
    assert_eq!(saved["invited"][0]["name"], "Other User");

    // Anonymous visitors are sent to sign in, and cannot ask for a ticket.
    let mut anon = Client::new().await;
    let (status, headers, _) = anon.request(&format!("/meet/j/{id}"), None).await;
    assert!(status.is_redirection(), "{status}");
    assert_eq!(
        headers["location"].to_str().unwrap(),
        format!("/signin?next=/meet/j/{id}")
    );

    // The invited account skips the lobby.
    let mut invited = other().await;
    let (status, _, _) = invited.request(&format!("/meet/j/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, entry) = enter(&mut invited, &id, None).await;
    assert_eq!(status, StatusCode::OK, "{entry}");
    assert_eq!(entry["state"], "admitted");

    // Off the list, the link is not enough.
    set_access(&id, "invited", "", &[]).await;
    let (status, _, body) = other().await.request(&format!("/meet/j/{id}"), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("not invited"), "{body}");
    assert_eq!(
        enter(&mut other().await, &id, None).await.0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn limiting_a_meeting_removes_people_who_are_no_longer_allowed() {
    let id = meeting("Tightened meeting").await;
    let mut guest = Client::new().await;
    let (_, waiting) = enter(&mut guest, &id, None).await;
    let ticket = waiting["id"].as_str().unwrap().to_string();
    let mut invited = other().await;
    let (_, kept) = enter(&mut invited, &id, None).await;
    let kept = kept["id"].as_str().unwrap().to_string();

    set_access(&id, "invited", "", &["other-user"]).await;

    let row = visit::Entity::find_by_id(&ticket)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.state, "removed");
    let (status, state) = call(
        &mut guest,
        &format!("/meet/j/{id}"),
        "meeting_room_state",
        json!({"id":id,"ticket":ticket}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{state}");
    assert_eq!(state["state"], "removed");
    let row = visit::Entity::find_by_id(&kept)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.state, "waiting", "invited people stay");
}

#[tokio::test]
async fn only_the_organizer_changes_access_and_bad_input_is_refused() {
    let id = meeting("Guarded settings").await;
    let mut stranger = other().await;
    assert_eq!(
        call(&mut stranger, "/meet", "meeting_access", json!({"id":id}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut stranger,
            "/meet",
            "save_meeting_access",
            json!({"id":id,"mode":"open","passcode":"","invited":[]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut host = owner().await;
    let (status, body) = call(
        &mut host,
        "/meet",
        "save_meeting_access",
        json!({"id":id,"mode":"secret","passcode":"","invited":[]}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(
        call(
            &mut host,
            "/meet",
            "save_meeting_access",
            json!({"id":id,"mode":"passcode","passcode":"ab","invited":[]})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &mut host,
            "/meet",
            "save_meeting_access",
            json!({"id":id,"mode":"invited","passcode":"","invited":["no-such-account"]})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &mut host,
            "/meet",
            "save_meeting_access",
            json!({"id":id,"mode":"invited","passcode":"","invited":"other-user"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    // An empty passcode in passcode mode is minted by the server.
    let saved = set_access(&id, "passcode", "", &[]).await;
    assert_eq!(saved["passcode"].as_str().unwrap().len(), 8);
}

#[tokio::test]
async fn a_calendar_meeting_suggests_its_attendees_as_invitees() {
    let (status, body) = rpc(
        "/calendar",
        "schedule_meeting",
        &json!({"input":{"title":"Planning with Other","attendees":"other@example.com, Somebody Unknown","start_date":"2026-12-01","start_time":"10:00","end_date":"2026-12-01","end_time":"10:30","timezone":"UTC","all_day":false,"recurrence":"none","occurrences":1,"location":"","details":""}}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let scheduled: Value = rahti::serde_json::from_str(&body).unwrap();
    let id = scheduled[0]["event_id"].as_str().unwrap();
    let (status, settings) = call(
        &mut owner().await,
        "/meet",
        "meeting_access",
        json!({"id":id}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{settings}");
    assert_eq!(settings["suggested"], true);
    assert_eq!(
        settings["invited"],
        json!([{"id":"other-user","name":"Other User"}])
    );
}

#[tokio::test]
async fn organizers_can_search_people_to_invite() {
    let (status, people) = call(
        &mut owner().await,
        "/meet",
        "search_meeting_invitees",
        json!({"query":"other"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{people}");
    assert!(
        people
            .as_array()
            .unwrap()
            .iter()
            .any(|person| person["id"] == "other-user")
    );
}

#[tokio::test]
async fn organizers_set_access_in_the_browser_and_guests_meet_the_passcode_gate() {
    let id = meeting("Browser access").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    let card = format!("[data-meeting-id='{id}']");
    browser
        .wait_for(&format!("document.querySelector(\"{card} .meeting-access-summary\").textContent.includes('Anyone with the link')"))
        .await
        .unwrap();

    // Passcode: choosing the mode proposes a code; saving applies it.
    browser
        .click(&format!("{card} .meeting-access-open"))
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.meeting-access-dialog').open && document.querySelector('input[name=mode][value=open]').checked")
        .await
        .unwrap();
    browser
        .click("input[name=mode][value=passcode]")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('#meeting-access-passcode').value.length === 8 && document.querySelector('#meeting-access-passcode').offsetParent !== null")
        .await
        .unwrap();
    browser.click(".meeting-access-save").await.unwrap();
    browser
        .wait_for(&format!("!document.querySelector('.meeting-access-dialog').open && document.querySelector(\"{card} .meeting-access-summary\").textContent.includes('Passcode required')"))
        .await
        .unwrap();
    let row = crate::models::meeting_link::Entity::find_by_id(&id)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.access, "passcode");
    let code = row.passcode.unwrap();

    // A guest must get through the gate before the pre-join screen.
    let Some(mut guest) = launch_browser().await else {
        return;
    };
    guest
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    guest
        .wait_for("!document.querySelector('.meeting-passcode').hidden && document.querySelector('.meeting-launch').hidden")
        .await
        .unwrap();
    let _: bool = guest
        .eval("(() => { document.querySelector('#meeting-passcode').focus(); return true; })()")
        .await
        .unwrap();
    // Fired until the page's script hears it: the button is enabled by its
    // state, so a keystroke made before the script was live cannot be lost.
    guest
        .wait_for("(() => { const field = document.querySelector('#meeting-passcode'); field.value = 'wrongcode'; field.dispatchEvent(new Event('input', { bubbles: true })); return !document.querySelector('.meeting-passcode button[type=submit]').disabled; })()")
        .await
        .unwrap();
    guest
        .click(".meeting-passcode button[type=submit]")
        .await
        .unwrap();
    guest
        .wait_for("document.querySelector('.meeting-passcode p[aria-live]').textContent.includes(\"isn't right\")")
        .await
        .unwrap();
    // A link carrying the code goes straight past the gate, and the code
    // leaves the address bar.
    guest
        .goto(&format!(
            "http://{address}/meet/j/{id}?p={}",
            code.to_uppercase()
        ))
        .await
        .unwrap();
    guest
        .wait_for("document.querySelector('.meeting-passcode').hidden && !document.querySelector('.meeting-launch').hidden && location.search === ''")
        .await
        .unwrap();
    assert!(guest.errors().is_empty(), "{:?}", guest.errors());

    // Invite-only: search, add, save.
    browser
        .click(&format!("{card} .meeting-access-open"))
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.meeting-access-dialog').open && document.querySelector('input[name=mode][value=passcode]').checked")
        .await
        .unwrap();
    browser
        .click("input[name=mode][value=invited]")
        .await
        .unwrap();
    let _: bool = browser
        .eval(
            "(() => { document.querySelector('#meeting-access-search').focus(); return true; })()",
        )
        .await
        .unwrap();
    browser.type_text("other").await.unwrap();
    browser
        .wait_for("Array.from(document.querySelectorAll('.meeting-invite-results button')).some(b => b.textContent.includes('Other User'))")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { Array.from(document.querySelectorAll('.meeting-invite-results button')).find(b => b.textContent.includes('Other User')).click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.meeting-invited').textContent.includes('Other User')")
        .await
        .unwrap();
    browser.click(".meeting-access-save").await.unwrap();
    browser
        .wait_for(&format!("!document.querySelector('.meeting-access-dialog').open && document.querySelector(\"{card} .meeting-access-summary\").textContent.includes('Invited people only')"))
        .await
        .unwrap();
    let invites = crate::models::meeting_invite::Entity::find()
        .all(crate::db::db())
        .await
        .unwrap();
    assert!(
        invites
            .iter()
            .any(|row| row.meeting_id == id && row.user_id == "other-user")
    );
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
