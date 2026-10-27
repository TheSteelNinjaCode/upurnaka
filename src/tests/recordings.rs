//! Meeting recordings: organizer-only, pieced together in order, and handed
//! to storage after the browser says stop. The suite's storage is the
//! stand-in in `meetings::recording`, so nothing here reaches Dilnaka.

use super::auth::Client;
use super::*;
use crate::models::{meeting_recording as recording, meeting_visit as visit};
use rahti::serde_json::{Value, json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};

async fn signed_in(cookie: String) -> Client {
    let mut client = Client::new().await;
    client.with_cookie(cookie);
    client
}
async fn meeting() -> String {
    let (s, b) = rpc(
        "/meet",
        "create_meeting_link",
        r#"{"title":"Recorded meeting"}"#,
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
/// A seat in the call, without making billable provider calls.
async fn seated(client: &mut Client, id: &str, name: &str, seat: &str) -> String {
    let (s, entry) = call(
        client,
        id,
        "request_meeting_entry",
        json!({"id":id,"name":name}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{entry}");
    let ticket = entry["id"].as_str().unwrap().to_string();
    visit::Entity::update_many()
        .col_expr(visit::Column::State, Expr::value("joined"))
        .col_expr(visit::Column::AttendeeId, Expr::value(seat))
        .filter(visit::Column::Id.eq(&ticket))
        .exec(crate::db::db())
        .await
        .unwrap();
    ticket
}
async fn piece(
    client: &mut Client,
    id: &str,
    recording: &str,
    part: i32,
    content_type: &str,
    bytes: &[u8],
) -> StatusCode {
    let reply = client
        .upload(
            &format!("/meet/j/{id}"),
            "append_meeting_recording",
            json!({"id":id,"recording":recording,"part":part}),
            ("chunk", content_type, bytes),
        )
        .await;
    reply.0
}
async fn row(id: &str) -> recording::Model {
    recording::Entity::find_by_id(id)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn the_organizer_records_in_ordered_pieces_and_the_file_is_stored() {
    let id = meeting().await;
    let mut host = signed_in(support::test_cookie()).await;
    let mut guest = signed_in(support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ))
    .await;
    let host_ticket = seated(&mut host, &id, "Organizer", "host-seat").await;
    let guest_ticket = seated(&mut guest, &id, "Guest", "guest-seat").await;

    let start = json!({"id":id,"ticket":guest_ticket,"content_type":"video/webm"});
    let (s, _) = call(&mut guest, &id, "start_meeting_recording", start).await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    let start = json!({"id":id,"ticket":host_ticket,"content_type":"video/webm;codecs=vp8,opus"});
    let (s, started) = call(&mut host, &id, "start_meeting_recording", start.clone()).await;
    assert_eq!(s, StatusCode::OK, "{started}");
    let recording_id = started["id"].as_str().unwrap().to_string();
    assert_eq!(row(&recording_id).await.content_type, "video/webm");

    // Everybody in the room is told.
    let (s, state) = call(
        &mut guest,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":guest_ticket}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{state}");
    assert_eq!(state["recording"], true);
    let (s, _) = call(&mut host, &id, "start_meeting_recording", start).await;
    assert_eq!(s, StatusCode::CONFLICT);

    // In order, once each, of the type the recording started with.
    let webm = "video/webm;codecs=vp8,opus";
    assert_eq!(
        piece(&mut host, &id, &recording_id, 1, webm, b"late").await,
        StatusCode::CONFLICT
    );
    assert_eq!(
        piece(&mut host, &id, &recording_id, 0, webm, b"abc").await,
        StatusCode::OK
    );
    assert_eq!(
        piece(&mut host, &id, &recording_id, 0, webm, b"abc").await,
        StatusCode::OK
    );
    assert_eq!(
        piece(&mut host, &id, &recording_id, 1, "video/webm", b"defg").await,
        StatusCode::OK
    );
    assert_eq!(
        piece(&mut host, &id, &recording_id, 2, "text/plain", b"nope").await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        piece(&mut guest, &id, &recording_id, 2, webm, b"nope").await,
        StatusCode::FORBIDDEN
    );
    let counted = row(&recording_id).await;
    assert_eq!((counted.parts, counted.bytes), (2, 7));

    let (s, _) = call(
        &mut guest,
        &id,
        "finish_meeting_recording",
        json!({"id":id,"recording":recording_id}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = call(
        &mut host,
        &id,
        "finish_meeting_recording",
        json!({"id":id,"recording":recording_id}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        piece(&mut host, &id, &recording_id, 2, webm, b"after").await,
        StatusCode::CONFLICT
    );

    // The upload runs detached from the request that asked for it.
    let mut stored = row(&recording_id).await;
    for _ in 0..100 {
        if stored.state != "uploading" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        stored = row(&recording_id).await;
    }
    assert_eq!(stored.state, "stored", "{:?}", stored.failure);
    // The pieces joined into one seven-byte file.
    assert_eq!(stored.file_id.as_deref(), Some("test-file-7"));
    let key = stored.file_key.unwrap();
    assert!(
        key.starts_with(&format!("recordings/{id}/meeting-{id}-")),
        "{key}"
    );
    assert!(key.ends_with(".webm"), "{key}");
    assert!(
        !std::env::temp_dir()
            .join("upurnaka-recordings")
            .join(&recording_id)
            .exists()
    );

    let (s, state) = call(
        &mut guest,
        &id,
        "meeting_room_state",
        json!({"id":id,"ticket":guest_ticket}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(state["recording"], false);

    let (s, _) = call(&mut guest, &id, "meeting_recordings", json!({"id":id})).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, listed) = call(&mut host, &id, "meeting_recordings", json!({"id":id})).await;
    assert_eq!(s, StatusCode::OK, "{listed}");
    assert_eq!(listed[0]["id"], recording_id.as_str());
    assert_eq!(listed[0]["state"], "stored");
    assert_eq!(listed[0]["bytes"], 7);

    let access = json!({"id":id,"recording":recording_id});
    let (s, _) = call(&mut guest, &id, "meeting_recording_url", access.clone()).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, url) = call(&mut host, &id, "meeting_recording_url", access).await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        url.as_str()
            .unwrap()
            .starts_with("https://storage.test/test-file-7")
    );
}

#[tokio::test]
async fn recording_refuses_malformed_and_unsupported_requests() {
    let id = meeting().await;
    let mut host = signed_in(support::test_cookie()).await;
    let ticket = seated(&mut host, &id, "Organizer", "host-seat").await;
    let (s, _) = call(
        &mut host,
        &id,
        "start_meeting_recording",
        json!({"id":id,"content_type":"video/webm"}),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, body) = call(
        &mut host,
        &id,
        "start_meeting_recording",
        json!({"id":id,"ticket":ticket,"content_type":"image/png"}),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (s, _) = call(
        &mut host,
        &id,
        "meeting_recording_url",
        json!({"id":id,"recording":"missing"}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Without a session there is no recording at all.
    let mut anonymous = Client::new().await;
    let (s, _) = call(&mut anonymous, &id, "meeting_recordings", json!({"id":id})).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}
