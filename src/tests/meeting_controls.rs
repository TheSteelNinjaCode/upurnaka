use super::auth::Client;
use super::*;
use crate::models::meeting_visit as visit;
use rahti::serde_json::{Value, json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use std::time::Duration;

async fn guest() -> Client {
    let mut client = Client::new().await;
    client.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    client
}
async fn owner() -> Client {
    let mut client = Client::new().await;
    client.with_cookie(super::support::test_cookie());
    client
}
async fn meeting() -> String {
    let (s, b) = rpc(
        "/meet",
        "create_meeting_link",
        r#"{"title":"Moderated meeting"}"#,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    rahti::serde_json::from_str::<Value>(&b).unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
async fn call(client: &mut Client, id: &str, name: &str, payload: Value) -> (StatusCode, Value) {
    let (s, _, b) = client
        .request(&format!("/meet/j/{id}"), Some((name, payload)))
        .await;
    (
        s,
        rahti::serde_json::from_str(&b).unwrap_or_else(|_| json!({"raw":b})),
    )
}
async fn entry(client: &mut Client, id: &str, name: &str) -> Value {
    let (s, b) = call(
        client,
        id,
        "request_meeting_entry",
        json!({"id":id,"name":name}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    b
}

async fn joined(ticket: &Value) {
    visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("joined"))
        .filter(visit::Column::Id.eq(ticket.as_str().unwrap()))
        .exec(crate::db::db())
        .await
        .unwrap();
}

#[tokio::test]
async fn anonymous_guests_use_expiring_tickets_without_an_account() {
    let id = meeting().await;
    let path = format!("/meet/j/{id}");
    let mut anon = Client::new().await;
    let (status, headers, body) = anon.request(&path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Continue in this browser"));
    assert!(
        headers["cache-control"]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    assert!(!headers.get_all("set-cookie").iter().any(|h| {
        h.to_str()
            .unwrap()
            .starts_with(&format!("{}=", rahti::auth::settings().cookie))
    }));
    assert_eq!(
        anon.request("/calendar", None).await.0,
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        anon.request("/meet/j/unknown-guest-link", None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &mut anon,
            &id,
            "request_meeting_entry",
            json!({"id":id,"name":"  "})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let guest = entry(&mut anon, &id, "Anonymous visitor").await;
    assert_eq!(guest["state"], "waiting");
    assert_eq!(guest["host"], false);
    let ticket = &guest["id"];
    assert_eq!(ticket.as_str().unwrap().len(), 43);
    assert_eq!(
        call(
            &mut anon,
            &id,
            "join_meeting",
            json!({"id":id,"display_name":"Guest","admission":ticket})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut anon,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":"guessed-ticket"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut host = owner().await;
    let organizer = entry(&mut host, &id, "Organizer").await;
    assert_eq!(
        call(
            &mut anon,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":organizer["id"]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut host,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":ticket,"admit":true})
        )
        .await
        .0,
        StatusCode::FORBIDDEN,
        "Organizer must join first"
    );
    joined(&organizer["id"]).await;
    assert_eq!(
        call(
            &mut host,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":ticket,"admit":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, state) = call(
        &mut anon,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":ticket}),
    )
    .await;
    assert_eq!(state["state"], "admitted");
    assert_eq!(state["waiting"], json!([]));
    let other = meeting().await;
    assert_eq!(
        call(
            &mut anon,
            &other,
            "meeting_room_state",
            json!({"id":other,"ticket":ticket})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut anon,
            &id,
            "leave_meeting_entry",
            json!({"id":id,"ticket":ticket})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &mut anon,
            &id,
            "join_meeting",
            json!({"id":id,"display_name":"Guest","admission":ticket})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let expired = entry(&mut anon, &id, "Expired visitor").await;
    visit::Entity::update_many()
        .col_expr(visit::Column::LeaseUntil, Expr::value(0))
        .filter(visit::Column::Id.eq(expired["id"].as_str().unwrap()))
        .exec(crate::db::db())
        .await
        .unwrap();
    assert_eq!(
        call(
            &mut anon,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":expired["id"]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let revoked = entry(&mut anon, &id, "Visitor before revocation").await;
    assert_eq!(
        rpc(
            "/meet",
            "delete_meeting_link",
            &json!({"id":id}).to_string()
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(anon.request(&path, None).await.0, StatusCode::NOT_FOUND);
    assert_eq!(
        call(
            &mut anon,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":revoked["id"]})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn lobby_admission_is_required_before_credentials_and_only_owner_can_decide() {
    let id = meeting().await;
    let mut host = owner().await;
    let mut guest = guest().await;
    let host_entry = entry(&mut host, &id, "Organizer").await;
    assert_eq!(host_entry["state"], "admitted");
    assert_eq!(host_entry["host"], true);
    joined(&host_entry["id"]).await;
    let guest_entry = entry(&mut guest, &id, "Waiting Guest").await;
    assert_eq!(guest_entry["state"], "waiting");
    assert_eq!(guest_entry["host"], false);
    assert!(guest_entry.get("attendee").is_none());
    let token = guest_entry["id"].as_str().unwrap();
    assert_eq!(
        call(
            &mut guest,
            &id,
            "join_meeting",
            json!({"id":id,"display_name":"Bypass","admission":token})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut guest,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":token,"admit":true})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut guest,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":host_entry["id"]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, snapshot) = call(
        &mut host,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":host_entry["id"]}),
    )
    .await;
    assert!(
        snapshot["waiting"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["id"] == token)
    );
    let (_, snapshot) = call(
        &mut guest,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":token}),
    )
    .await;
    assert_eq!(snapshot["waiting"], json!([]));
    assert_eq!(
        call(
            &mut host,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":token,"admit":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &mut host,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":token,"admit":true})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    // Approval gets past admission; the suite deliberately has no AWS provider.
    assert_eq!(
        call(
            &mut guest,
            &id,
            "join_meeting",
            json!({"id":id,"display_name":"Guest","admission":token})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let (_, snapshot) = call(
        &mut guest,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":token}),
    )
    .await;
    assert_eq!(
        snapshot["state"], "admitted",
        "provider failures must allow a safe retry"
    );
    let other = meeting().await;
    assert_eq!(
        call(
            &mut guest,
            &other,
            "join_meeting",
            json!({"id":other,"display_name":"Guest","admission":token})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let denied = entry(&mut guest, &id, "Denied Guest").await;
    assert_eq!(
        call(
            &mut host,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":denied["id"],"admit":false})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &mut guest,
            &id,
            "join_meeting",
            json!({"id":id,"display_name":"Denied","admission":denied["id"]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    visit::Entity::update_many()
        .col_expr(visit::Column::LeaseUntil, Expr::value(0))
        .filter(visit::Column::Id.eq(token))
        .exec(crate::db::db())
        .await
        .unwrap();
    assert_eq!(
        call(
            &mut guest,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":token})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn spotlight_and_host_mute_are_scoped_and_unmute_belongs_to_the_participant() {
    let id = meeting().await;
    let mut host = owner().await;
    let mut guest = guest().await;
    let host_entry = entry(&mut host, &id, "Organizer").await;
    let guest_entry = entry(&mut guest, &id, "Guest").await;
    let ticket = guest_entry["id"].as_str().unwrap();
    // A synthetic provider seat, without making billable AWS calls.
    visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("joined"))
        .col_expr(
            visit::Column::ProviderMeetingId,
            Expr::value("fake-provider"),
        )
        .col_expr(visit::Column::AttendeeId, Expr::value("guest-seat"))
        .filter(visit::Column::Id.eq(ticket))
        .exec(crate::db::db())
        .await
        .unwrap();
    for name in ["mute_participant", "spotlight_participant"] {
        assert_eq!(
            call(
                &mut guest,
                &id,
                name,
                json!({"id":id,"attendee":"guest-seat"})
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            call(
                &mut host,
                &id,
                name,
                json!({"id":id,"attendee":"another-meeting-attendee"})
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        call(
            &mut host,
            &id,
            "spotlight_participant",
            json!({"id":id,"attendee":"guest-seat"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &mut host,
            &id,
            "mute_participant",
            json!({"id":id,"attendee":"guest-seat"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, state) = call(
        &mut guest,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":ticket}),
    )
    .await;
    assert_eq!(state["spotlight"], "guest-seat");
    assert_eq!(state["muted_by_host"], true);
    assert_eq!(
        call(
            &mut host,
            &id,
            "resume_meeting_audio",
            json!({"id":id,"ticket":ticket})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut guest,
            &id,
            "resume_meeting_audio",
            json!({"id":id,"ticket":ticket})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &mut guest,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":ticket})
        )
        .await
        .1["muted_by_host"],
        false
    );
    assert_eq!(
        call(
            &mut host,
            &id,
            "spotlight_participant",
            json!({"id":id,"attendee":null})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &mut host,
            &id,
            "spotlight_participant",
            json!({"id":id,"attendee":"guest-seat"})
        )
        .await
        .0,
        StatusCode::OK
    );
    call(
        &mut guest,
        &id,
        "leave_meeting_entry",
        json!({"id":id,"ticket":ticket}),
    )
    .await;
    assert!(
        call(
            &mut host,
            &id,
            "meeting_room_state",
            json!({"id":id,"ticket":host_entry["id"]})
        )
        .await
        .1["spotlight"]
            .is_null()
    );
    assert_eq!(
        call(
            &mut guest,
            &id,
            "join_meeting",
            json!({"id":id,"display_name":"Reuse","admission":ticket})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn provider_permission_denial_preserves_mute_state_and_explains_the_failure() {
    let id = meeting().await;
    let mut host = owner().await;
    let mut guest = guest().await;
    let guest_entry = entry(&mut guest, &id, "Guest").await;
    let ticket = guest_entry["id"].as_str().unwrap();
    visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("joined"))
        .col_expr(
            visit::Column::ProviderMeetingId,
            Expr::value("denied-moderation-provider"),
        )
        .col_expr(visit::Column::AttendeeId, Expr::value("denied-seat"))
        .filter(visit::Column::Id.eq(ticket))
        .exec(crate::db::db())
        .await
        .unwrap();
    let (status, body) = call(
        &mut host,
        &id,
        "mute_participant",
        json!({"id":id,"attendee":"denied-seat"}),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("participant-mute permission")
    );
    assert!(
        !visit::Entity::find_by_id(ticket)
            .one(crate::db::db())
            .await
            .unwrap()
            .unwrap()
            .muted_by_host
    );

    // A failed self-unmute must retain the previously confirmed host mute.
    visit::Entity::update_many()
        .col_expr(visit::Column::MutedByHost, Expr::value(true))
        .filter(visit::Column::Id.eq(ticket))
        .exec(crate::db::db())
        .await
        .unwrap();
    let (status, body) = call(
        &mut guest,
        &id,
        "resume_meeting_audio",
        json!({"id":id,"ticket":ticket}),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("participant-mute permission")
    );
    assert!(
        visit::Entity::find_by_id(ticket)
            .one(crate::db::db())
            .await
            .unwrap()
            .unwrap()
            .muted_by_host
    );
}

#[tokio::test]
async fn meeting_control_rpcs_validate_shapes_and_guard_moderator_calls() {
    let id = meeting().await;
    let mut host = owner().await;
    let mut anon = Client::new().await;
    for name in [
        "request_meeting_entry",
        "meeting_room_state",
        "decide_meeting_entry",
        "leave_meeting_entry",
        "spotlight_participant",
        "mute_participant",
        "resume_meeting_audio",
    ] {
        assert_eq!(
            call(&mut host, &id, name, json!({})).await.0,
            StatusCode::BAD_REQUEST,
            "{name}"
        );
        assert_eq!(
            call(&mut anon, &id, name, json!({})).await.0,
            if [
                "decide_meeting_entry",
                "spotlight_participant",
                "mute_participant"
            ]
            .contains(&name)
            {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_REQUEST
            },
            "{name}"
        );
    }
    assert_eq!(
        call(
            &mut host,
            &id,
            "request_meeting_entry",
            json!({"id":id,"name":"  "})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn only_the_seeded_admin_moderates_legacy_unowned_links() {
    let id = meeting().await;
    crate::models::meeting_link::Entity::update_many()
        .col_expr(
            crate::models::meeting_link::Column::OwnerId,
            Expr::value(Option::<String>::None),
        )
        .filter(crate::models::meeting_link::Column::Id.eq(&id))
        .exec(crate::db::db())
        .await
        .unwrap();
    let mut admin = Client::new().await;
    admin.with_cookie(super::support::cookie_for(
        "test-admin",
        "Test Administrator",
        "admin-test@example.com",
    ));
    let admitted = entry(&mut admin, &id, "Admin").await;
    assert_eq!(admitted["host"], true);
    joined(&admitted["id"]).await;
    let mut regular = owner().await;
    let waiting = entry(&mut regular, &id, "Regular").await;
    assert_eq!(waiting["state"], "waiting");
    assert_eq!(
        call(
            &mut regular,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":waiting["id"],"admit":true})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &mut admin,
            &id,
            "decide_meeting_entry",
            json!({"id":id,"ticket":waiting["id"],"admit":true})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn cancelled_entries_do_not_use_up_the_open_tab_limit() {
    let id = meeting().await;
    let mut host = owner().await;
    for _ in 0..10 {
        let row = entry(&mut host, &id, "Organizer").await;
        assert_eq!(
            call(
                &mut host,
                &id,
                "leave_meeting_entry",
                json!({"id":id,"ticket":row["id"]})
            )
            .await
            .0,
            StatusCode::OK
        );
    }
}

fn browser_router() -> axum::Router {
    crate::routes::router()
        .route("/js/vendor/amazon-chime-sdk.js",axum::routing::get(|| async {
            ([("content-type","text/javascript")],include_str!("fixtures/chime-room.js"))
        }))
        .layer(axum::middleware::from_fn(|request:axum::extract::Request,next:axum::middleware::Next| async move {
            if request.headers().get("x-pp-function").and_then(|v|v.to_str().ok()) == Some("join_meeting") {
                // Replace only the external provider boundary. Real admission/mutation RPCs
                // and separate router tests still enforce organizer and ticket authority.
                let bytes = axum::body::to_bytes(request.into_body(),4096).await.unwrap();
                let input:Value = rahti::serde_json::from_slice(&bytes).unwrap();
                let ticket = input["admission"].as_str().unwrap();
                let row = visit::Entity::find_by_id(ticket).one(crate::db::db()).await.unwrap().unwrap();
                let attendee = format!("seat-{ticket}");
                visit::Entity::update_many().col_expr(visit::Column::State,Expr::value("joined"))
                    .col_expr(visit::Column::ProviderMeetingId,Expr::value("fixture"))
                    .col_expr(visit::Column::AttendeeId,Expr::value(&attendee))
                    .filter(visit::Column::Id.eq(ticket)).exec(crate::db::db()).await.unwrap();
                return axum::Json(json!({"meeting":{"MeetingId":"fixture"},"attendee":{"AttendeeId":attendee,"ExternalUserId":format!("{}#fixture",row.name)}})).into_response();
            }
            next.run(request).await
        }))
}
use axum::response::IntoResponse;

/// The open tile menu touches the button that opened it and stays on the stage.
async fn assert_tile_menu_beside(browser: &mut rahti::browser::Browser, trigger: &str) {
    browser
        .wait_for(&format!(
            r#"(() => {{
                const menu = document.querySelector(".room-tile-menu");
                if (menu.hidden) return false;
                const box = menu.getBoundingClientRect();
                const button = document.querySelector("{}").getBoundingClientRect();
                const stage = document.querySelector(".room-stage").getBoundingClientRect();
                const tall = Math.min(Math.abs(box.bottom - button.top), Math.abs(box.top - button.bottom)) <= 12;
                const wide = box.right >= button.left && box.left <= button.right;
                return tall && wide && box.left >= stage.left && box.right <= stage.right && box.top >= stage.top && box.bottom <= stage.bottom;
            }})()"#,
            trigger.replace('"', "\\\"")
        ))
        .await
        .unwrap();
}

async fn browser_join(browser: &mut rahti::browser::Browser) {
    browser
        .wait_for("document.querySelector('.meeting-primary') !== null")
        .await
        .unwrap();
    // One step: a re-render between enabling and clicking disables it again.
    let _: bool = browser
        .eval("(() => { const join = document.querySelector('.meeting-primary'); join.disabled = false; join.click(); return true; })()")
        .await
        .unwrap();
}

#[tokio::test]
async fn browsers_can_use_lobby_and_host_controls_without_replacing_video_elements() {
    let id = meeting().await;
    let address = rahti::browser::serve(browser_router()).await;
    let Some(mut host_browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    let Some(mut guest_browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut host_browser, address).await;
    host_browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    browser_join(&mut host_browser).await;
    host_browser
        .wait_for("!document.querySelector('.meeting-room').hidden")
        .await
        .unwrap();
    let _: bool = host_browser
        .eval("window.lobbySounds = []; window.UpurnakaNotificationSound.play = name => window.lobbySounds.push(name); true")
        .await
        .unwrap();
    guest_browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    guest_browser.wait_for("!document.querySelector('.meeting-launch').hidden && document.querySelector('.meeting-body').hidden").await.unwrap();
    let _: bool = guest_browser.eval("(() => { const event = new Event('beforeinstallprompt'); event.prompt = async () => {}; event.userChoice = Promise.resolve({outcome:'accepted'}); window.dispatchEvent(event); return true; })()").await.unwrap();
    guest_browser.click(".guest-install").await.unwrap();
    guest_browser.wait_for("!document.querySelector('.meeting-launch p[role=status]').hidden && document.querySelector('.meeting-launch p[role=status]').textContent.length > 0").await.unwrap();
    guest_browser.click(".guest-browser").await.unwrap();
    guest_browser.wait_for("!document.querySelector('.meeting-body').hidden && !document.querySelector('.meeting-guest-name').hidden && document.querySelector('.meeting-primary').disabled").await.unwrap();
    guest_browser.click("#guest-display-name").await.unwrap();
    guest_browser.type_text("Other User").await.unwrap();
    guest_browser
        .wait_for("document.querySelector('.meeting-account strong').textContent === 'Other User'")
        .await
        .unwrap();
    browser_join(&mut guest_browser).await;
    guest_browser.wait_for("!document.querySelector('.meeting-lobby-wait').hidden && document.querySelector('.meeting-room').hidden").await.unwrap();
    host_browser.wait_for("!document.querySelector('.room-lobby-notice').hidden && document.querySelector('button[aria-label=Participants]').textContent.includes('1') && window.lobbySounds.join() === 'default'").await.unwrap();
    host_browser
        .click(".room-lobby-notice button:nth-of-type(1)")
        .await
        .unwrap();
    guest_browser.wait_for("document.querySelector('.meeting-lobby-wait').hidden && document.querySelector('[role=alert]').textContent.includes('declined')").await.unwrap();
    browser_join(&mut guest_browser).await;
    host_browser
        .wait_for("!document.querySelector('.room-lobby-notice').hidden")
        .await
        .unwrap();
    host_browser
        .click(".room-lobby-notice button:nth-of-type(2)")
        .await
        .unwrap();
    guest_browser.wait_for("!document.querySelector('.meeting-room').hidden && document.querySelector('.meeting-lobby-wait').hidden").await.unwrap();
    let row = visit::Entity::find()
        .filter(visit::Column::MeetingId.eq(&id))
        .filter(visit::Column::UserId.starts_with("guest:"))
        .filter(visit::Column::State.eq("joined"))
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    let guest_id = row.attendee_id.unwrap();
    let _:bool = host_browser.eval(&format!(r##"
        window.roomFixture.presence({guest_id:?},true,'Other User#fixture');
        window.roomFixture.observer.videoTileDidUpdate({{tileId:10,boundAttendeeId:{guest_id:?},boundExternalUserId:'Other User#fixture',paused:false}});
        true
    "##)).await.unwrap();
    host_browser.wait_for("document.querySelector('.room-person-video:not([hidden]) video')?.srcObject !== null && document.querySelector('button[aria-label=Participants]').textContent.includes('2')").await.unwrap();
    let _:bool = host_browser.eval("window.testTileVideo = document.querySelector('.room-person-video:not([hidden]) video'); true").await.unwrap();
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    assert_tile_menu_beside(&mut host_browser, ".room-person-video:not([hidden]) button").await;
    // The same button closes it, and stays closed.
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    host_browser.pause(Duration::from_millis(300)).await;
    assert!(
        host_browser
            .eval::<bool>("document.querySelector('.room-tile-menu').hidden")
            .await
            .unwrap(),
        "a second click on the options button must close the menu"
    );
    // Another tile's button moves it there instead.
    host_browser
        .click("button[aria-label='Options for your video']")
        .await
        .unwrap();
    assert_tile_menu_beside(
        &mut host_browser,
        "button[aria-label='Options for your video']",
    )
    .await;
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    host_browser
        .wait_for("document.querySelector('.room-tile-menu').textContent.includes('Other User')")
        .await
        .unwrap();
    assert_tile_menu_beside(&mut host_browser, ".room-person-video:not([hidden]) button").await;
    host_browser
        .click(".room-tile-menu [data-action=pin]")
        .await
        .unwrap();
    host_browser
        .wait_for("document.querySelector('.room-gallery').getAttribute('data-focus') === 'true'")
        .await
        .unwrap();
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    host_browser
        .click(".room-tile-menu [data-action=fit]")
        .await
        .unwrap();
    host_browser.wait_for("document.querySelector('.room-person-video:not([hidden]) video').classList.contains('object-cover')").await.unwrap();
    assert!(host_browser.eval::<bool>("document.querySelector('.room-person-video:not([hidden]) video') === window.testTileVideo && window.testTileVideo.srcObject.active").await.unwrap());
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    host_browser
        .click(".room-tile-menu [data-action=spotlight]")
        .await
        .unwrap();
    guest_browser
        .wait_for(
            "document.querySelector('.room-self-tile').getAttribute('data-featured') === 'true'",
        )
        .await
        .unwrap();
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    host_browser
        .click(".room-tile-menu [data-action=mute]")
        .await
        .unwrap();
    guest_browser.wait_for("window.roomFixture.muted && document.querySelector('button[aria-label=\"Toggle meeting microphone\"]').getAttribute('aria-pressed') === 'false'").await.unwrap();
    guest_browser
        .click("button[aria-label='Toggle meeting microphone']")
        .await
        .unwrap();
    guest_browser
        .wait_for("!window.roomFixture.muted")
        .await
        .unwrap();
    // Raised-hand toast and gold frame are independent of video and spotlight layout.
    let _:bool = host_browser.eval(&format!(r##"
        window.roomFixture.message({{data:new Uint8Array([1]),senderAttendeeId:{guest_id:?},senderExternalUserId:'Other User#fixture',json:()=>({{kind:'hand',raised:true}})}}); true
    "##)).await.unwrap();
    host_browser.wait_for("!document.querySelector('.room-hand-notice').hidden && document.querySelector('.room-person-video:not([hidden])').getAttribute('data-raised') === 'true'").await.unwrap();
    host_browser
        .click("button[aria-label='Dismiss raised hand notification']")
        .await
        .unwrap();
    // Stop spotlight before checking the default screen-share layout.
    host_browser
        .click(".room-person-video:not([hidden]) button")
        .await
        .unwrap();
    host_browser
        .click(".room-tile-menu [data-action=spotlight]")
        .await
        .unwrap();
    host_browser
        .wait_for("document.querySelector('.room-gallery').getAttribute('data-focus') === 'false'")
        .await
        .unwrap();
    let _:bool = host_browser.eval(&format!(r##"
        window.roomFixture.observer.videoTileDidUpdate({{tileId:11,boundAttendeeId:{guest_id:?}+'#content',boundExternalUserId:'Other User#fixture',isContent:true,paused:true}}); true
    "##)).await.unwrap();
    host_browser.wait_for("!document.querySelector('.room-content-tile').hidden && document.querySelector('.room-content-tile').textContent.includes('paused or unavailable')").await.unwrap();
    assert!(host_browser.eval::<bool>("(() => { const main = document.querySelector('.room-content-tile').getBoundingClientRect(); const tile = document.querySelector('.room-person-video:not([hidden])').getBoundingClientRect(); return main.width > tile.width && main.right <= tile.left; })()").await.unwrap(), "{}", host_browser.eval::<String>("JSON.stringify({width:innerWidth, focus:document.querySelector('.room-gallery').getAttribute('data-focus'), columns:getComputedStyle(document.querySelector('.room-gallery')).gridTemplateColumns, content:document.querySelector('.room-content-tile').getBoundingClientRect().toJSON(), camera:document.querySelector('.room-person-video:not([hidden])').getBoundingClientRect().toJSON()})").await.unwrap());
    host_browser
        .click("button[aria-label='Options for your video']")
        .await
        .unwrap();
    host_browser
        .click(".room-tile-menu [data-action=hide-self]")
        .await
        .unwrap();
    host_browser
        .wait_for("document.querySelector('.room-self-tile').hidden")
        .await
        .unwrap();
    host_browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    host_browser
        .click(".room-more-menu button:last-child")
        .await
        .unwrap();
    host_browser
        .wait_for("!document.querySelector('.room-self-tile').hidden")
        .await
        .unwrap();
    host_browser
        .click("button[aria-label='Toggle meeting camera']")
        .await
        .unwrap();
    host_browser.wait_for("!document.querySelector('.room-local-video').hidden && window.roomFixture.controller.lastInput === 'test-camera'").await.unwrap();
    let _: bool = host_browser
        .eval("window.previousCameraStream = window.roomFixture.controller.stream; true")
        .await
        .unwrap();
    host_browser
        .click("button[aria-label='Options for your video']")
        .await
        .unwrap();
    assert_tile_menu_beside(
        &mut host_browser,
        "button[aria-label='Options for your video']",
    )
    .await;
    host_browser
        .click(".room-tile-menu [data-action=switch-camera]")
        .await
        .unwrap();
    host_browser.wait_for("window.roomFixture.controller.lastInput === 'second-camera' && document.querySelector('.room-local-video').srcObject === window.roomFixture.controller.stream && window.previousCameraStream.getTracks().every(track => track.readyState === 'ended')").await.unwrap();
    host_browser
        .click("button[aria-label='Leave meeting']")
        .await
        .unwrap();
    guest_browser
        .click("button[aria-label='Leave meeting']")
        .await
        .unwrap();
    assert!(
        host_browser.errors().is_empty(),
        "{:?}",
        host_browser.errors()
    );
    assert!(
        guest_browser.errors().is_empty(),
        "{:?}",
        guest_browser.errors()
    );
}
