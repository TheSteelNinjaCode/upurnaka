use super::{auth::Client, *};
use rahti::serde_json::{Value, json};

async fn chatter(email: &str) -> Client {
    use crate::models::account;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
    setup().await;
    let row = crate::auth::create_account(
        "Chat Tester".into(),
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
    client.request("/chat", None).await;
    client
}

#[tokio::test]
async fn chat_files_upload_attach_and_download_only_for_their_owner() {
    let mut owner = chatter("chat-file-owner@example.com").await;
    let mut other = chatter("chat-file-other@example.com").await;

    let (status, body) = owner
        .upload(
            "/chat",
            "upload_chat_file",
            json!({}),
            ("file", "text/plain", b""),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("file"));
    assert_eq!(
        owner
            .request("/chat", Some(("upload_chat_file", json!({}))))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );

    let (status, body) = owner
        .upload(
            "/chat",
            "upload_chat_file",
            json!({}),
            ("file", "text/plain", b"quarterly notes"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let stored: Value = rahti::serde_json::from_str(&body).unwrap();
    let id = stored["id"].as_str().unwrap().to_string();
    assert_eq!(stored["bytes"], 15);
    assert_eq!(stored["content_type"], "text/plain");

    let (_, _, listed) = owner
        .request("/chat", Some(("chat_cloud_files", json!({"view":"files"}))))
        .await;
    assert!(listed.contains(&id), "{listed}");
    let (_, _, foreign) = other
        .request("/chat", Some(("chat_cloud_files", json!({"view":"files"}))))
        .await;
    assert!(
        !foreign.contains(&id),
        "another person's cloud files leaked"
    );

    let (status, _, body) = owner
        .request(
            "/chat",
            Some((
                "send_message",
                json!({"text":"", "gif_id":null, "file_ids":[id]}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let message: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(message["attachments"][0]["id"], id.as_str());

    let (status, _, _) = other
        .request(
            "/chat",
            Some((
                "send_message",
                json!({"text":"mine now", "gif_id":null, "file_ids":[id]}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, body) = owner
        .request(
            "/chat",
            Some((
                "send_message",
                json!({"text":"too many", "gif_id":null, "file_ids":vec![id.clone(); 11]}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("file_ids"));

    let (status, headers, _) = owner.request(&format!("/chat/files/{id}"), None).await;
    assert_eq!(status, StatusCode::FOUND);
    assert!(
        headers["location"]
            .to_str()
            .unwrap()
            .starts_with("https://storage.test/test-file-15")
    );
    let (status, _, _) = other.request(&format!("/chat/files/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cloud_picker_browses_folders_and_views_of_its_owner_only() {
    let mut owner = chatter("chat-folder-owner@example.com").await;
    let mut other = chatter("chat-folder-other@example.com").await;
    let browse = |view: &str, folder: Option<&str>| {
        Some(("chat_cloud_files", json!({"view": view, "folder": folder})))
    };

    let (status, _, _) = owner.request("/chat", browse("trash", None)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _, body) = owner
        .request(
            "/chat",
            Some(("create_chat_folder", json!({"parent":null, "name":"  "}))),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (status, _, body) = owner
        .request(
            "/chat",
            Some((
                "create_chat_folder",
                json!({"parent":null, "name":"Documentos"}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let folder: Value = rahti::serde_json::from_str(&body).unwrap();
    let folder_id = folder["id"].as_str().unwrap().to_string();
    let (status, _, body) = owner
        .request(
            "/chat",
            Some((
                "create_chat_folder",
                json!({"parent":null, "name":"documentos"}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("already here"), "{body}");
    let (status, _, body) = owner
        .request(
            "/chat",
            Some((
                "create_chat_folder",
                json!({"parent":folder_id, "name":"Facturas"}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let nested: Value = rahti::serde_json::from_str(&body).unwrap();

    let (status, body) = owner
        .upload(
            "/chat",
            "upload_chat_file",
            json!({"folder": folder_id}),
            ("file", "image/png", b"not really a png"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let photo: Value = rahti::serde_json::from_str(&body).unwrap();
    let (status, body) = other
        .upload(
            "/chat",
            "upload_chat_file",
            json!({"folder": folder_id}),
            ("file", "text/plain", b"into a stranger's folder"),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let (status, _, body) = owner.request("/chat", browse("files", None)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let top: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(top["folders"][0]["name"], "Documentos");
    assert_eq!(top["folders"][0]["bytes"], 16);
    assert_eq!(top["folders"][0]["items"], 2);
    assert!(
        top["files"].as_array().unwrap().is_empty(),
        "a file in a folder showed at the top: {body}"
    );

    let (_, _, body) = owner
        .request("/chat", browse("files", Some(&folder_id)))
        .await;
    let inside: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(inside["path"][0]["name"], "Documentos");
    assert_eq!(inside["folders"][0]["id"], nested["id"]);
    assert_eq!(inside["files"][0]["id"], photo["id"]);

    let (_, _, body) = owner.request("/chat", browse("photos", None)).await;
    assert!(body.contains(photo["id"].as_str().unwrap()), "{body}");
    let (_, _, body) = owner.request("/chat", browse("shared", None)).await;
    let shared: Value = rahti::serde_json::from_str(&body).unwrap();
    assert!(shared["files"].as_array().unwrap().is_empty());

    let (status, _, _) = other
        .request("/chat", browse("files", Some(&folder_id)))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, _, body) = other.request("/chat", browse("files", None)).await;
    assert!(
        !body.contains("Documentos"),
        "another person's folders leaked"
    );
}

#[tokio::test]
async fn composer_uploads_attaches_cloud_files_and_sends_them() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .click("button[aria-label='Attach a file']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.chat-attach-menu').matches(':popover-open') && document.querySelector('.chat-attach-menu').textContent.includes('Attach cloud files') && document.querySelector('.chat-attach-menu').textContent.includes('Upload from this device')").await.unwrap();
    browser.press("Escape").await.unwrap();
    let _: bool = browser.eval("const transfer = new DataTransfer(); transfer.items.add(new File(['agenda for the week'], 'Agenda.txt', {type:'text/plain'})); transfer.items.add(new File([''], 'empty.txt', {type:'text/plain'})); const input = document.querySelector('.chat-file-input'); input.files = transfer.files; input.dispatchEvent(new Event('change', {bubbles:true})); true").await.unwrap();
    browser.wait_for("document.querySelectorAll('.chat-draft-file').length === 1 && document.querySelector('.chat-draft-file').textContent.includes('Agenda.txt') && document.querySelector('.chat-draft-file').textContent.includes('19 B') && !document.querySelector('button[aria-label=Send]').disabled").await.unwrap();
    browser.click("button[aria-label=Send]").await.unwrap();
    browser.wait_for("document.querySelector('.chat-message-file')?.textContent.includes('Agenda.txt') && document.querySelector('.chat-message-file').getAttribute('href').startsWith('/chat/files/') && document.querySelectorAll('.chat-draft-file').length === 0").await.unwrap();

    browser
        .click("button[aria-label='Attach a file']")
        .await
        .unwrap();
    browser
        .click(".chat-attach-menu [role=menuitem]:first-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.chat-cloud-dialog').open && document.querySelector('.chat-cloud-files button[aria-label=\"Select Agenda.txt\"]') && document.querySelector('.chat-cloud-attach').disabled").await.unwrap();
    browser
        .click(".chat-cloud-files button[aria-label='Select Agenda.txt']")
        .await
        .unwrap();
    browser.click(".chat-cloud-attach").await.unwrap();
    browser.wait_for("!document.querySelector('.chat-cloud-dialog').open && document.querySelector('.chat-draft-file')?.textContent.includes('Agenda.txt')").await.unwrap();
    browser
        .click("button[aria-label='Remove Agenda.txt']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('.chat-draft-file').length === 0 && document.querySelector('button[aria-label=Send]').disabled")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
