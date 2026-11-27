//! The Files page: My files managed in a table, and this person's recordings.

use super::{auth::Client, *};
use rahti::serde_json::{Value, json};

async fn person(email: &str) -> (Client, String) {
    use crate::models::account;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
    setup().await;
    let row = crate::auth::create_account(
        "Files Tester".into(),
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
    let mut client = Client::new().await;
    client.with_cookie(super::support::cookie_for(&row.id, &row.name, &row.email));
    client.request("/files", None).await;
    (client, row.id)
}

async fn call(client: &mut Client, rpc: &str, body: Value) -> (StatusCode, Value) {
    let (status, _, text) = client.request("/files", Some((rpc, body))).await;
    let value = rahti::serde_json::from_str(&text).unwrap_or(Value::String(text));
    (status, value)
}

async fn upload(client: &mut Client, name: &str) -> String {
    let (status, body) = client
        .upload(
            "/files",
            "files_upload",
            json!({}),
            ("file", "text/plain", b"file contents"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let stored: Value = rahti::serde_json::from_str(&body).unwrap();
    let id = stored["id"].as_str().unwrap().to_string();
    let (status, _) = call(client, "files_rename", json!({"id": id, "name": name})).await;
    assert_eq!(status, StatusCode::OK);
    id
}

#[tokio::test]
async fn files_page_renders_in_the_rail_for_a_signed_in_person() {
    let (mut client, _) = person("files-page@example.com").await;
    let (status, _, body) = client.request("/files", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("My files"), "{body}");
    assert!(body.contains("Recordings"));
    assert!(body.contains("href=\"/files\""), "the rail links to Files");
}

#[tokio::test]
async fn files_can_be_renamed_moved_and_deleted_only_by_their_owner() {
    let (mut owner, _) = person("files-owner@example.com").await;
    let (mut other, _) = person("files-other@example.com").await;

    let (status, folder) = call(
        &mut owner,
        "files_new_folder",
        json!({"parent": null, "name": "Reports"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{folder}");
    let folder = folder["id"].as_str().unwrap().to_string();
    let first = upload(&mut owner, "notes.txt").await;
    let second = upload(&mut owner, "draft.txt").await;

    // Rename: a bad name is a 422 with a field message; a malformed call is a 400.
    let (status, body) = call(
        &mut owner,
        "files_rename",
        json!({"id": first, "name": "a/b"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["errors"]["name"].is_array(), "{body}");
    let (status, _) = call(&mut owner, "files_rename", json!({"id": first})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(
        &mut other,
        "files_rename",
        json!({"id": first, "name": "stolen.txt"}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "someone else's file was renamed"
    );

    // Folder names stay unique among siblings.
    let (status, _) = call(
        &mut owner,
        "files_new_folder",
        json!({"parent": null, "name": "Archive"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = call(
        &mut owner,
        "files_rename_folder",
        json!({"id": folder, "name": "archive"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    // Move both into the folder; the top level no longer lists them.
    let (status, _) = call(
        &mut other,
        "files_move",
        json!({"ids": [first], "folder": null}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &mut owner,
        "files_move",
        json!({"ids": [first, second], "folder": folder}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, top) = call(&mut owner, "files_list", json!({"folder": null})).await;
    assert_eq!(
        top["listing"]["files"].as_array().unwrap().len(),
        0,
        "{top}"
    );
    let (_, inside) = call(&mut owner, "files_list", json!({"folder": folder})).await;
    let names: Vec<&str> = inside["listing"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"notes.txt") && names.contains(&"draft.txt"),
        "{inside}"
    );
    assert_eq!(inside["usage"]["files"], 2);
    let (_, places) = call(&mut owner, "files_places", json!({})).await;
    assert!(places.to_string().contains("Reports"), "{places}");

    // Delete one file, then the folder with the rest inside it.
    let (status, _) = call(&mut other, "files_delete", json!({"ids": [first]})).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "someone else's file was deleted"
    );
    let (status, count) = call(&mut owner, "files_delete", json!({"ids": [first]})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(count, 1);
    let (status, _) = call(&mut other, "files_delete_folder", json!({"id": folder})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, count) = call(&mut owner, "files_delete_folder", json!({"id": folder})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(count, 1, "the file inside went with its folder");
    let (_, after) = call(&mut owner, "files_list", json!({"folder": null})).await;
    assert_eq!(after["usage"]["files"], 0, "{after}");
    assert!(!after.to_string().contains("Reports"));
}

#[tokio::test]
async fn recordings_are_listed_downloaded_and_deleted_by_whoever_started_them() {
    use crate::models::meeting_recording as recording;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
    let (mut owner, owner_id) = person("files-recorder@example.com").await;
    let (mut other, _) = person("files-viewer@example.com").await;
    let now = rahti::time::Timestamp::now().as_second();
    for (id, state) in [
        ("files-rec-stored", "stored"),
        ("files-rec-live", "recording"),
    ] {
        recording::ActiveModel {
            id: Set(id.into()),
            meeting_id: Set("files-meeting".into()),
            started_by: Set(owner_id.clone()),
            state: Set(state.into()),
            content_type: Set("video/webm".into()),
            parts: Set(1),
            bytes: Set(2048),
            started_at: Set(now - 90),
            updated_at: Set(now),
            ended_at: Set(Some(now - 30)),
            file_id: Set((state == "stored").then(|| "files-object".into())),
            file_key: Set((state == "stored").then(|| "recordings/files-object".into())),
            failure: Set(None),
        }
        .insert(crate::db::db())
        .await
        .unwrap();
    }

    let (status, listed) = call(&mut owner, "files_recordings", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed.as_array().unwrap().len(), 2, "{listed}");
    assert_eq!(listed[0]["seconds"], 60);
    let (_, foreign) = call(&mut other, "files_recordings", json!({})).await;
    assert_eq!(foreign.as_array().unwrap().len(), 0, "recordings leaked");

    let (status, headers, _) = owner
        .request("/files/recordings/files-rec-stored", None)
        .await;
    assert_eq!(status, StatusCode::FOUND);
    assert!(
        headers["location"]
            .to_str()
            .unwrap()
            .contains("files-object")
    );
    let (status, _, _) = other
        .request("/files/recordings/files-rec-stored", None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = call(
        &mut owner,
        "files_delete_recording",
        json!({"id": "files-rec-live"}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a running recording was deleted"
    );
    let (status, _) = call(
        &mut other,
        "files_delete_recording",
        json!({"id": "files-rec-stored"}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &mut owner,
        "files_delete_recording",
        json!({"id": "files-rec-stored"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        recording::Entity::find_by_id("files-rec-stored")
            .one(crate::db::db())
            .await
            .unwrap()
            .is_none()
    );
}

async fn insert_recording(id: &str, owner: &str, state: &str) {
    use crate::models::meeting_recording as recording;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    let now = rahti::time::Timestamp::now().as_second();
    recording::ActiveModel {
        id: Set(id.into()),
        meeting_id: Set("files-bulk-meeting".into()),
        started_by: Set(owner.into()),
        state: Set(state.into()),
        content_type: Set("video/webm".into()),
        parts: Set(1),
        bytes: Set(4096),
        started_at: Set(now - 90),
        updated_at: Set(now),
        ended_at: Set(Some(now - 30)),
        file_id: Set((state == "stored").then(|| format!("object-{id}"))),
        file_key: Set((state == "stored").then(|| format!("recordings/object-{id}"))),
        failure: Set(None),
    }
    .insert(crate::db::db())
    .await
    .unwrap();
}

async fn recording_exists(id: &str) -> bool {
    use crate::models::meeting_recording as recording;
    use sea_orm::EntityTrait;
    recording::Entity::find_by_id(id)
        .one(crate::db::db())
        .await
        .unwrap()
        .is_some()
}

#[tokio::test]
async fn selected_recordings_are_deleted_together_and_only_by_their_owner() {
    let (mut owner, owner_id) = person("files-bulk-owner@example.com").await;
    let (mut other, other_id) = person("files-bulk-other@example.com").await;
    insert_recording("bulk-stored", &owner_id, "stored").await;
    insert_recording("bulk-failed", &owner_id, "failed").await;
    insert_recording("bulk-live", &owner_id, "recording").await;
    insert_recording("bulk-foreign", &other_id, "stored").await;

    // Someone else's recording in the list: nothing is deleted.
    let (status, _) = call(
        &mut owner,
        "files_delete_recordings",
        json!({"ids": ["bulk-stored", "bulk-foreign"]}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(recording_exists("bulk-stored").await);
    assert!(recording_exists("bulk-foreign").await);

    for bad in [json!({"ids": []}), json!({"ids": [""]})] {
        let (status, _) = call(&mut owner, "files_delete_recordings", bad).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (status, _) = call(
        &mut owner,
        "files_delete_recordings",
        json!({"ids": "bulk-stored"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // A running recording is skipped; the finished and failed ones go.
    let (status, count) = call(
        &mut owner,
        "files_delete_recordings",
        json!({"ids": ["bulk-stored", "bulk-failed", "bulk-live", "bulk-stored"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{count}");
    assert_eq!(count, 2);
    assert!(!recording_exists("bulk-stored").await);
    assert!(!recording_exists("bulk-failed").await);
    assert!(
        recording_exists("bulk-live").await,
        "a running recording was deleted"
    );
    let (_, count) = call(
        &mut other,
        "files_delete_recordings",
        json!({"ids": ["bulk-foreign"]}),
    )
    .await;
    assert_eq!(count, 1);
}

#[tokio::test]
async fn select_all_recordings_then_delete_in_the_browser() {
    let (_, owner_id) = person("files-bulk-browser@example.com").await;
    for (id, state) in [
        ("browser-rec-a", "stored"),
        ("browser-rec-b", "failed"),
        ("browser-rec-live", "uploading"),
    ] {
        insert_recording(id, &owner_id, state).await;
    }
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    browser
        .goto(&format!("http://{address}/signin"))
        .await
        .unwrap();
    let cookie = format!(
        "{}; Path=/; SameSite=Lax",
        super::support::cookie_for(&owner_id, "Files Tester", "files-bulk-browser@example.com")
    );
    let _: String = browser
        .eval(&format!(
            "document.cookie = {}",
            rahti::serde_json::to_string(&cookie).unwrap()
        ))
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/files"))
        .await
        .unwrap();
    browser
        .wait_for("Array.from(document.querySelectorAll('button[role=tab]')).some(b => b.textContent.trim() === 'Recordings')")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { Array.from(document.querySelectorAll('button[role=tab]')).find(b => b.textContent.trim() === 'Recordings').click(); return true; })()")
        .await
        .unwrap();
    // The row still uploading cannot be picked.
    browser
        .wait_for("document.querySelectorAll('section[aria-label=Recordings] tbody input[type=checkbox]').length === 3 && Array.from(document.querySelectorAll('section[aria-label=Recordings] tbody input[type=checkbox]')).filter(box => box.disabled).length === 1")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Select all recordings']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.recording-selection').hidden && document.querySelector('.recording-selection').textContent.includes('2 selected')")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { Array.from(document.querySelectorAll('.recording-selection button')).find(b => b.textContent.trim() === 'Delete').click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("Array.from(document.querySelectorAll('[role=alertdialog]')).some(d => d.textContent.includes('Delete 2 recordings?'))")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { Array.from(document.querySelectorAll('[role=alertdialog] button')).find(b => b.textContent.trim() === 'Delete permanently').click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("document.body.textContent.includes('2 recordings deleted.') && document.querySelectorAll('section[aria-label=Recordings] tbody tr').length === 1 && document.querySelector('.recording-selection').hidden")
        .await
        .unwrap();
    assert!(!recording_exists("browser-rec-a").await);
    assert!(!recording_exists("browser-rec-b").await);
    assert!(recording_exists("browser-rec-live").await);
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
