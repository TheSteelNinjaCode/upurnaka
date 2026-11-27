//! Activity: who is told about what, and only the reader reads or clears it.

use super::auth::Client;
use super::*;
use crate::models::{account, meeting_visit as visit, notification};
use crate::notifications::{self, Event};
use rahti::serde_json::{Value, json};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr,
};

/// A verified account of the test's own, so other tests' events never land
/// in its counts.
async fn person(email: &str, name: &str) -> (String, String) {
    setup().await;
    let row = crate::auth::create_account(
        name.into(),
        email.into(),
        "a long unique password".into(),
        "a long unique password".into(),
        crate::auth::Role::User,
    )
    .await
    .unwrap();
    account::Entity::update_many()
        .col_expr(account::Column::Verified, Expr::value(true))
        .filter(account::Column::Id.eq(&row.id))
        .exec(crate::db::db())
        .await
        .unwrap();
    let cookie = cookie_for(&row.id, &row.name, &row.email);
    (row.id, cookie)
}
async fn signed_in(cookie: String) -> Client {
    let mut client = Client::new().await;
    client.with_cookie(cookie);
    client.request("/activity", None).await;
    client
}
async fn call(client: &mut Client, name: &str, payload: Value) -> (StatusCode, Value) {
    let (s, _, b) = client.request("/activity", Some((name, payload))).await;
    (
        s,
        rahti::serde_json::from_str(&b).unwrap_or_else(|_| json!({"raw":b})),
    )
}
async fn told(user: &str, kind: &str, meeting: &str) -> Vec<notification::Model> {
    notification::Entity::find()
        .filter(notification::Column::UserId.eq(user))
        .filter(notification::Column::Kind.eq(kind))
        .filter(notification::Column::MeetingId.eq(meeting))
        .all(crate::db::db())
        .await
        .unwrap()
}
fn event<'a>(meeting: &'a str, title: &str) -> Event<'a> {
    Event {
        kind: notifications::INVITE,
        meeting_id: meeting,
        actor: "Organizer",
        title: title.into(),
        body: "Body".into(),
        link: format!("/meet/j/{meeting}"),
    }
}
async fn mint(title: &str) -> String {
    let (status, body) = rpc(
        "/meet",
        "create_meeting_link",
        &json!({"title":title}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    rahti::serde_json::from_str::<Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
async fn joined(meeting: &str, user: &str, name: &str) -> String {
    let ticket = format!("ticket-{}", crate::meetings::mint_id());
    visit::ActiveModel {
        id: Set(ticket.clone()),
        meeting_id: Set(meeting.into()),
        user_id: Set(user.into()),
        name: Set(name.into()),
        state: Set("joined".into()),
        lease_until: Set(rahti::time::Timestamp::now().as_second() + 90),
        attendee_id: Set(None),
        provider_meeting_id: Set(None),
        muted_by_host: Set(false),
    }
    .insert(crate::db::db())
    .await
    .unwrap();
    ticket
}

#[tokio::test]
async fn the_reader_lists_counts_and_clears_only_their_own_activity() {
    let (me, cookie) = person("activity-reader@example.com", "Activity Reader").await;
    let (them, _) = person("activity-other@example.com", "Activity Other").await;
    notifications::notify([me.clone()], event("m-reader-1", "First for the reader")).await;
    notifications::notify(
        [me.clone(), me.clone()],
        event("m-reader-2", "Second for the reader"),
    )
    .await;
    notifications::notify([them.clone()], event("m-reader-3", "Not for the reader")).await;

    let mut client = signed_in(cookie).await;
    let (status, _, page) = client.request("/activity", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("First for the reader") && page.contains("Second for the reader"));
    assert!(
        !page.contains("Not for the reader"),
        "another person's activity leaked"
    );

    let (status, count) = call(&mut client, "activity_unread_count", json!({})).await;
    assert_eq!(
        (status, count),
        (StatusCode::OK, json!(2)),
        "a duplicate recipient is told once"
    );

    let theirs = told(&them, notifications::INVITE, "m-reader-3")
        .await
        .remove(0);
    let (status, _) = call(&mut client, "read_activity", json!({"id":theirs.id})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        told(&them, notifications::INVITE, "m-reader-3").await[0]
            .read_at
            .is_none()
    );
    for bad in [json!({}), json!({"id":5})] {
        assert_eq!(
            call(&mut client, "read_activity", bad).await.0,
            StatusCode::BAD_REQUEST
        );
    }

    let mine = told(&me, notifications::INVITE, "m-reader-1")
        .await
        .remove(0);
    let (status, left) = call(&mut client, "read_activity", json!({"id":mine.id})).await;
    assert_eq!((status, left), (StatusCode::OK, json!(1)));
    let (status, feed) = call(&mut client, "activity_feed", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed[0]["title"], "Second for the reader", "newest first");
    assert_eq!(feed[1]["read"], true);

    let (status, left) = call(&mut client, "read_all_activity", json!({})).await;
    assert_eq!((status, left), (StatusCode::OK, json!(0)));
    assert!(
        told(&them, notifications::INVITE, "m-reader-3").await[0]
            .read_at
            .is_none()
    );

    let mut anonymous = Client::new().await;
    assert_eq!(
        anonymous.request("/activity", None).await.0,
        StatusCode::SEE_OTHER
    );
    for name in [
        "activity_feed",
        "activity_unread_count",
        "read_all_activity",
    ] {
        assert_eq!(
            anonymous
                .request("/activity", Some((name, json!({}))))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn people_added_to_an_invite_list_are_told_once() {
    let id = mint("Invite-only review").await;
    let save = |invited: Value| {
        let id = id.clone();
        async move {
            let payload = json!({"id":id,"mode":"invited","passcode":"","invited":invited});
            let (status, body) = rpc("/meet", "save_meeting_access", &payload.to_string()).await;
            assert_eq!(status, StatusCode::OK, "{body}");
        }
    };
    save(json!(["other-user"])).await;
    save(json!(["other-user", "test-admin"])).await;

    let other = told("other-user", notifications::INVITE, &id).await;
    assert_eq!(
        other.len(),
        1,
        "saving the list again does not repeat the invite"
    );
    assert_eq!(
        other[0].title,
        "Jefferson Abraham invited you to Invite-only review"
    );
    assert_eq!(other[0].link, format!("/meet/j/{id}"));
    assert_eq!(
        told("test-admin", notifications::INVITE, &id).await.len(),
        1
    );
    assert!(
        told("legacy-test-user", notifications::INVITE, &id)
            .await
            .is_empty(),
        "never the organizer"
    );
}

#[tokio::test]
async fn scheduled_attendees_with_accounts_are_told() {
    let input = json!({"title":"Planning with accounts","attendees":"other@example.com, Test Administrator, Somebody Unknown","start_date":"2026-12-02","start_time":"15:00","end_date":"2026-12-02","end_time":"15:30","timezone":"America/Managua","all_day":false,"recurrence":"none","occurrences":1,"location":"","details":""});
    let (status, body) = rpc(
        "/calendar",
        "schedule_meeting",
        &json!({"input":input}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let id = rahti::serde_json::from_str::<Value>(&body).unwrap()[0]["event_id"]
        .as_str()
        .unwrap()
        .to_string();

    let other = told("other-user", notifications::SCHEDULED, &id).await;
    assert_eq!(other.len(), 1, "matched by email");
    assert_eq!(
        other[0].title,
        "Jefferson Abraham invited you to Planning with accounts"
    );
    assert_eq!(other[0].body, "Wed, Dec 2 at 15:00 (America/Managua)");
    assert_eq!(
        told("test-admin", notifications::SCHEDULED, &id)
            .await
            .len(),
        1,
        "matched by name"
    );
}

#[tokio::test]
async fn a_lobby_tells_an_absent_organizer_once_and_a_present_one_never() {
    let id = mint("Lobby while away").await;
    let page = format!("/meet/j/{id}");
    for name in ["Early Guest", "Second Guest"] {
        let mut guest = Client::new().await;
        let (status, _, body) = guest
            .request(
                &page,
                Some(("request_meeting_entry", json!({"id":id,"name":name}))),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let lobby = told("legacy-test-user", notifications::LOBBY, &id).await;
    assert_eq!(lobby.len(), 1, "one unread lobby notice, not one per guest");
    assert_eq!(lobby[0].title, "Early Guest is waiting in the lobby");

    notification::Entity::update_many()
        .col_expr(notification::Column::ReadAt, Expr::value(1_i64))
        .filter(notification::Column::Id.eq(&lobby[0].id))
        .exec(crate::db::db())
        .await
        .unwrap();
    joined(&id, "legacy-test-user", "Jefferson Abraham").await;
    let mut guest = Client::new().await;
    guest
        .request(
            &page,
            Some((
                "request_meeting_entry",
                json!({"id":id,"name":"Late Guest"}),
            )),
        )
        .await;
    assert_eq!(
        told("legacy-test-user", notifications::LOBBY, &id)
            .await
            .len(),
        1
    );
}

#[tokio::test]
async fn a_meeting_chat_mention_reaches_only_the_named_people_of_that_meeting() {
    let id = mint("Mention room").await;
    let payload = json!({"id":id,"mode":"open","passcode":"","invited":["other-user"]});
    assert_eq!(
        rpc("/meet", "save_meeting_access", &payload.to_string())
            .await
            .0,
        StatusCode::OK
    );
    let ticket = joined(&id, "guest:speaker", "Speaker").await;
    let page = format!("/meet/j/{id}");
    for text in [
        "@Other Username is not anyone here",
        "Thanks @other user, and @Test Administrator too",
    ] {
        let payload = json!({"id":id,"ticket":ticket,"text":text});
        let (status, body) = rpc(&page, "send_meeting_chat", &payload.to_string()).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let other = told("other-user", notifications::MENTION, &id).await;
    assert_eq!(other.len(), 1, "a longer name is not a mention");
    assert_eq!(other[0].title, "Speaker mentioned you in Mention room");
    assert_eq!(
        other[0].body,
        "Thanks @other user, and @Test Administrator too"
    );
    assert!(
        told("test-admin", notifications::MENTION, &id)
            .await
            .is_empty(),
        "someone with no part in the meeting is not told"
    );
}

#[tokio::test]
async fn the_rail_badge_counts_unread_and_reading_lowers_it() {
    let (me, cookie) = person("activity-browser@example.com", "Activity Browser").await;
    notifications::notify([me.clone()], event("m-browser-1", "Browser first")).await;
    notifications::notify([me.clone()], event("m-browser-2", "Browser second")).await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    browser
        .goto(&format!("http://{address}/signin"))
        .await
        .unwrap();
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
    browser.wait_for("document.querySelector('.activity-badge')?.textContent.trim() === '2' && !document.querySelector('.activity-badge').hidden").await.unwrap();

    browser
        .goto(&format!("http://{address}/activity"))
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('.activity-row').length === 2")
        .await
        .unwrap();
    browser.click(".activity-row").await.unwrap();
    browser.wait_for("document.querySelector('.activity-detail h2')?.textContent === 'Browser second' && document.querySelector('.activity-badge').textContent.trim() === '1'").await.unwrap();
    browser
        .click("button[aria-label='Mark all as read']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.activity-badge').hidden")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
