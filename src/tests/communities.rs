use super::*;
use base64::Engine;
use rahti::serde_json;
use rahti::serde_json::{Value, json};

#[test]
fn community_emoji_catalog_has_local_artwork_and_supported_glyphs() {
    let mut glyphs = std::collections::HashSet::new();
    let mut categories = std::collections::HashSet::new();
    for emoji in crate::emoji::catalog() {
        assert!(glyphs.insert(emoji.glyph.replace('\u{fe0f}', "")));
        assert!(emoji.image.starts_with("/images/fluent-emoji/"));
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("public")
            .join(emoji.image.trim_start_matches('/'));
        assert!(
            std::fs::read(path)
                .unwrap()
                .starts_with(b"\x89PNG\r\n\x1a\n")
        );
        categories.insert(emoji.category.as_str());
        assert!(crate::emoji::supported(&emoji.glyph));
    }
    assert_eq!(categories.len(), 9);
    for icon in [
        "💬", "📢", "🎓", "📚", "💡", "🗓️", "🎨", "⚽", "🌱", "💻", "🎵", "🚀", "✔", "✔️",
    ] {
        assert!(crate::emoji::supported(icon), "Missing {icon}");
    }
    for icon in ["", "\u{fe0f}", "text", "💬📚", "<script>"] {
        assert!(!crate::emoji::supported(icon));
    }
}

#[tokio::test]
async fn community_emoji_picker_search_categories_keyboard_and_recents() {
    let (status, body) = rpc(
        "/communities",
        "create_community",
        r#"{"name":"Emoji picker workspace","category":"School"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let community: Value = serde_json::from_str(&body).unwrap();
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    let url = format!(
        "http://{address}/communities?community={}",
        community["id"].as_str().unwrap()
    );
    browser.goto(&url).await.unwrap();
    let initial_channels = browser
        .eval::<usize>("document.querySelectorAll('.community-channel').length")
        .await
        .unwrap();
    browser.click(".community-add-channel").await.unwrap();
    browser
        .click(".community-channel-icon-trigger")
        .await
        .unwrap();
    browser
        .wait_for("document.activeElement.getAttribute('aria-label') === 'Search emoji'")
        .await
        .unwrap();
    assert!(browser.eval::<bool>("document.querySelectorAll('#popover-channel-icon .channel-emoji-categories [role=tab]').length === 10 && document.querySelectorAll('#popover-channel-icon .community-channel-icon-grid button').length === 24").await.unwrap());
    browser
        .click("button[aria-label='Animals and nature']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#popover-channel-icon .channel-emoji-heading').textContent === 'Animals and nature' && document.querySelector('button[data-emoji=\"🐶\"]') !== null").await.unwrap();
    browser.press("ArrowRight").await.unwrap();
    browser
        .wait_for(
            "document.querySelector('#popover-channel-icon .channel-emoji-heading').textContent === 'Food and drink'",
        )
        .await
        .unwrap();
    browser
        .click("input[aria-label='Search emoji']")
        .await
        .unwrap();
    browser.type_text("no-such-emoji-result").await.unwrap();
    browser.wait_for("!document.querySelector('#popover-channel-icon .channel-emoji-empty').hidden && !document.querySelector('#popover-channel-icon .community-channel-icon-grid button')").await.unwrap();
    browser
        .eval::<bool>("document.activeElement.select();true")
        .await
        .unwrap();
    browser.type_text("🚀").await.unwrap();
    browser
        .wait_for("document.querySelector('button[data-emoji=\"🚀\"]') !== null")
        .await
        .unwrap();
    browser.press("ArrowDown").await.unwrap();
    let focused = browser
        .wait_for("document.activeElement.dataset.emoji === '🚀'")
        .await;
    assert!(focused.is_ok(), "{focused:?}; {:?}; {}", browser.errors(), browser.eval::<String>("JSON.stringify({active:document.activeElement.outerHTML,first:document.querySelector('#popover-channel-icon .community-channel-icon-grid button')?.outerHTML})").await.unwrap());
    browser.press("Enter").await.unwrap();
    browser.wait_for("!document.querySelector('#popover-channel-icon').matches(':popover-open') && document.querySelector('.community-channel-icon-trigger').textContent.includes('🚀')").await.unwrap();
    browser
        .click(".community-channel-icon-trigger")
        .await
        .unwrap();
    browser
        .wait_for(
            "document.querySelector('#popover-channel-icon .community-channel-icon-grid button').dataset.emoji === '🚀'",
        )
        .await
        .unwrap();
    browser
        .click("input[aria-label='Search emoji']")
        .await
        .unwrap();
    browser.type_text("telescope").await.unwrap();
    browser
        .wait_for("document.querySelector('button[data-emoji=\"🔭\"]') !== null")
        .await
        .unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('.community-channel-icon-trigger').textContent.includes('🔭') && document.querySelector('.community-channel-dialog').open").await.unwrap();
    assert_eq!(
        browser
            .eval::<usize>("document.querySelectorAll('.community-channel').length")
            .await
            .unwrap(),
        initial_channels
    );
    browser.goto(&url).await.unwrap();
    browser.click(".community-add-channel").await.unwrap();
    browser
        .click(".community-channel-icon-trigger")
        .await
        .unwrap();
    browser
        .wait_for(
            "document.querySelector('#popover-channel-icon .community-channel-icon-grid button').dataset.emoji === '🔭'",
        )
        .await
        .unwrap();
    assert!(browser.eval::<bool>("Array.from(document.querySelectorAll('#popover-channel-icon .community-channel-icon-grid button')).filter(b=>b.dataset.emoji==='🔭').length===1 && document.querySelectorAll('#popover-channel-icon .community-channel-icon-grid button').length===24").await.unwrap());
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

async fn community_call(
    client: &mut super::auth::Client,
    name: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let (status, _, body) = client.request("/communities", Some((name, payload))).await;
    (
        status,
        serde_json::from_str(&body).unwrap_or_else(|_| json!({"raw":body})),
    )
}

#[tokio::test]
async fn post_reactions_toggle_and_replies_persist_for_members_only() {
    setup().await;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let mut member = super::auth::Client::new().await;
    member.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    member.request("/communities", None).await;
    let mut outsider = super::auth::Client::new().await;
    outsider.with_cookie(super::support::cookie_for(
        "test-admin",
        "Test Administrator",
        "admin-test@example.com",
    ));
    outsider.request("/communities", None).await;
    let (_, created) = community_call(
        &mut owner,
        "create_community",
        json!({"name":"Conversation school","category":"School"}),
    )
    .await;
    let id = created["id"].as_str().unwrap();
    let (_, link) = community_call(
        &mut owner,
        "community_join_link",
        json!({"id":id,"reset":false}),
    )
    .await;
    let token = link.as_str().unwrap().split("join=").nth(1).unwrap();
    let (status, _) = community_call(
        &mut member,
        "join_community",
        json!({"id":id,"token":token}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, post) = community_call(
        &mut owner,
        "publish_community_post",
        json!({"id":id,"channel":"","body":"Talk to me","announcement":false}),
    )
    .await;
    let post_id = post["id"].as_str().unwrap();
    assert_eq!(post["reactions"], json!([]));
    assert_eq!(post["replies"], json!([]));

    let react = json!({"id":id,"post":post_id,"emoji":"👍"});
    let (status, reactions) =
        community_call(&mut owner, "react_to_community_post", react.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reactions, json!([{"emoji":"👍","count":1,"mine":true}]));
    let (_, reactions) =
        community_call(&mut member, "react_to_community_post", react.clone()).await;
    assert_eq!(reactions, json!([{"emoji":"👍","count":2,"mine":true}]));
    let (_, reactions) = community_call(
        &mut member,
        "react_to_community_post",
        json!({"id":id,"post":post_id,"emoji":"❤️"}),
    )
    .await;
    assert_eq!(
        reactions,
        json!([{"emoji":"👍","count":2,"mine":true},{"emoji":"❤️","count":1,"mine":true}])
    );
    // Reacting again takes the reaction back.
    let (_, reactions) = community_call(&mut owner, "react_to_community_post", react).await;
    assert_eq!(
        reactions,
        json!([{"emoji":"👍","count":1,"mine":false},{"emoji":"❤️","count":1,"mine":false}])
    );

    let (status, reply) = community_call(
        &mut member,
        "reply_to_community_post",
        json!({"id":id,"post":post_id,"body":"  Replying here  "}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(reply["body"], "Replying here");
    assert_eq!(reply["author_name"], "Other User");
    assert_eq!(reply["author_is_owner"], false);
    let (_, reply) = community_call(
        &mut owner,
        "reply_to_community_post",
        json!({"id":id,"post":post_id,"body":"Owner answer"}),
    )
    .await;
    assert_eq!(reply["author_is_owner"], true);

    let (_, _, page) = member
        .request(&format!("/communities?community={id}"), None)
        .await;
    for value in ["Replying here", "Owner answer", "❤️"] {
        assert!(page.contains(value), "Missing {value}");
    }
    let replying = page.find("Replying here").unwrap();
    assert!(
        replying < page.find("Owner answer").unwrap(),
        "Replies are oldest first"
    );

    for (name, payload) in [
        (
            "react_to_community_post",
            json!({"id":id,"post":post_id,"emoji":"not an emoji"}),
        ),
        (
            "reply_to_community_post",
            json!({"id":id,"post":post_id,"body":"   "}),
        ),
        (
            "reply_to_community_post",
            json!({"id":id,"post":post_id,"body":"x".repeat(2001)}),
        ),
    ] {
        assert_eq!(
            community_call(&mut owner, name, payload).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    for (name, payload) in [
        (
            "react_to_community_post",
            json!({"id":id,"post":"missing","emoji":"👍"}),
        ),
        (
            "reply_to_community_post",
            json!({"id":id,"post":"missing","body":"Lost"}),
        ),
    ] {
        assert_eq!(
            community_call(&mut owner, name, payload).await.0,
            StatusCode::NOT_FOUND
        );
    }
    for (name, payload) in [
        (
            "react_to_community_post",
            json!({"id":id,"post":post_id,"emoji":"👍"}),
        ),
        (
            "reply_to_community_post",
            json!({"id":id,"post":post_id,"body":"Sneaky"}),
        ),
    ] {
        assert_eq!(
            community_call(&mut outsider, name, payload.clone()).await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            super::auth::Client::new()
                .await
                .request("/communities", Some((name, payload)))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            community_call(&mut owner, name, json!({"id":id})).await.0,
            StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test]
async fn post_edit_delete_and_pin_respect_authors_and_owners() {
    setup().await;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let mut member = super::auth::Client::new().await;
    member.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    member.request("/communities", None).await;
    let (_, created) = community_call(
        &mut owner,
        "create_community",
        json!({"name":"Moderated school","category":"School"}),
    )
    .await;
    let id = created["id"].as_str().unwrap();
    let (_, link) = community_call(
        &mut owner,
        "community_join_link",
        json!({"id":id,"reset":false}),
    )
    .await;
    let token = link.as_str().unwrap().split("join=").nth(1).unwrap();
    community_call(
        &mut member,
        "join_community",
        json!({"id":id,"token":token}),
    )
    .await;
    let (_, announcement) = community_call(
        &mut owner,
        "publish_community_post",
        json!({"id":id,"channel":"","body":"Old","announcement":true,"headline":"Old headline"}),
    )
    .await;
    let owner_post = announcement["id"].as_str().unwrap();
    assert_eq!(announcement["is_mine"], true);
    let (_, member_post) = community_call(
        &mut member,
        "publish_community_post",
        json!({"id":id,"channel":"","body":"Member words","announcement":false}),
    )
    .await;
    let member_post = member_post["id"].as_str().unwrap();

    // Only the author edits, and an announcement stays one.
    for (client, post) in [(&mut member, owner_post), (&mut owner, member_post)] {
        assert_eq!(
            community_call(
                client,
                "edit_community_post",
                json!({"id":id,"post":post,"body":"Hijacked"})
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        community_call(
            &mut owner,
            "edit_community_post",
            json!({"id":id,"post":owner_post,"body":"No headline"})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let (status, edited) = community_call(
        &mut owner,
        "edit_community_post",
        json!({"id":id,"post":owner_post,"body":"New","headline":"New headline","accent":"teal","formatting":[{"kind":"paragraph","runs":[{"text":"New","italic":true}]}]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{edited}");
    assert_eq!(edited["edited"], true);
    assert_eq!(edited["announcement"], true);
    assert_eq!(edited["headline"], "New headline");
    assert_eq!(edited["formatting"][0]["runs"][0]["italic"], true);

    // Pinning is owner-only.
    assert_eq!(
        community_call(
            &mut member,
            "pin_community_post",
            json!({"id":id,"post":member_post,"pinned":true})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, pinned) = community_call(
        &mut owner,
        "pin_community_post",
        json!({"id":id,"post":member_post,"pinned":true}),
    )
    .await;
    assert_eq!(pinned["pinned"], true);
    assert_eq!(pinned["is_mine"], false);

    // A member cannot delete the owner's post; the owner may delete theirs.
    assert_eq!(
        community_call(
            &mut member,
            "delete_community_post",
            json!({"id":id,"post":owner_post})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, deleted) = community_call(
        &mut owner,
        "delete_community_post",
        json!({"id":id,"post":member_post}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(deleted["deleted"], true);
    assert_eq!(deleted["body"], "");
    assert_eq!(deleted["pinned"], false);
    let (_, _, page) = member
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(
        !page.contains("Member words"),
        "Deleted content is withheld"
    );
    for (name, payload) in [
        (
            "react_to_community_post",
            json!({"id":id,"post":member_post,"emoji":"👍"}),
        ),
        (
            "reply_to_community_post",
            json!({"id":id,"post":member_post,"body":"Late"}),
        ),
        (
            "edit_community_post",
            json!({"id":id,"post":member_post,"body":"Revive"}),
        ),
    ] {
        assert_eq!(
            community_call(&mut member, name, payload).await.0,
            StatusCode::NOT_FOUND
        );
    }
    let (_, restored) = community_call(
        &mut member,
        "delete_community_post",
        json!({"id":id,"post":member_post,"restore":true}),
    )
    .await;
    assert_eq!(restored["deleted"], false);
    assert_eq!(restored["body"], "Member words");
    for name in [
        "edit_community_post",
        "delete_community_post",
        "pin_community_post",
    ] {
        assert_eq!(
            community_call(&mut owner, name, json!({"id":id})).await.0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            super::auth::Client::new()
                .await
                .request(
                    "/communities",
                    Some((
                        name,
                        json!({"id":id,"post":member_post,"body":"x","pinned":true})
                    ))
                )
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn community_files_upload_as_posts_and_download_only_for_members() {
    setup().await;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let mut member = super::auth::Client::new().await;
    member.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    member.request("/communities", None).await;
    let mut outsider = super::auth::Client::new().await;
    outsider.with_cookie(super::support::cookie_for(
        "test-admin",
        "Test Administrator",
        "admin-test@example.com",
    ));
    outsider.request("/communities", None).await;
    let (_, created) = community_call(
        &mut owner,
        "create_community",
        json!({"name":"File sharing school","category":"School"}),
    )
    .await;
    let id = created["id"].as_str().unwrap();
    let (_, link) = community_call(
        &mut owner,
        "community_join_link",
        json!({"id":id,"reset":false}),
    )
    .await;
    let token = link.as_str().unwrap().split("join=").nth(1).unwrap();
    community_call(
        &mut member,
        "join_community",
        json!({"id":id,"token":token}),
    )
    .await;

    let fields = json!({"id":id,"channel":""});
    let (status, body) = member
        .upload(
            "/communities",
            "upload_community_file",
            fields.clone(),
            ("file", "text/plain", b"homework list"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let post: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(post["author_name"], "Other User");
    assert_eq!(post["body"], "");
    let file = &post["files"][0];
    assert_eq!(file["bytes"], 13);
    assert_eq!(file["content_type"], "text/plain");
    assert_eq!(file["uploader_name"], "Other User");
    let file_id = file["id"].as_str().unwrap().to_string();
    let post_id = post["id"].as_str().unwrap().to_string();

    let (_, _, page) = owner
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(page.contains(&file_id), "The owner sees the shared file");
    let (status, headers, _) = owner
        .request(&format!("/communities/files/{file_id}"), None)
        .await;
    assert_eq!(status, StatusCode::FOUND);
    assert!(
        headers["location"]
            .to_str()
            .unwrap()
            .starts_with("https://storage.test/")
    );
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(
        outsider
            .request(&format!("/communities/files/{file_id}"), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        super::auth::Client::new()
            .await
            .request(&format!("/communities/files/{file_id}"), None)
            .await
            .0,
        StatusCode::SEE_OTHER
    );
    let (status, _) = outsider
        .upload(
            "/communities",
            "upload_community_file",
            fields.clone(),
            ("file", "text/plain", b"intruder"),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, body) = member
        .upload(
            "/communities",
            "upload_community_file",
            fields.clone(),
            ("file", "text/plain", b""),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("file"));
    let (status, _) = member
        .upload(
            "/communities",
            "upload_community_file",
            json!({"id":id,"channel":"missing"}),
            ("file", "text/plain", b"lost"),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        community_call(
            &mut member,
            "upload_community_file",
            json!({"id":id,"channel":""})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );

    // "Attach cloud files" shares from the uploader's Dilnaka Storage.
    member.request("/chat", None).await;
    let (status, body) = member
        .upload(
            "/chat",
            "upload_chat_file",
            json!({}),
            ("file", "application/pdf", b"%PDF-1.4 syllabus"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let cloud: Value = serde_json::from_str(&body).unwrap();
    let cloud_id = cloud["id"].as_str().unwrap();
    let (status, listing) = community_call(
        &mut member,
        "community_cloud_files",
        json!({"view":"recent"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        listing["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["id"] == cloud_id)
    );
    let (status, posts) = community_call(
        &mut member,
        "share_community_cloud_files",
        json!({"id":id,"channel":"","files":[cloud_id]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{posts}");
    let shared = &posts[0]["files"][0];
    assert_eq!(shared["content_type"], "application/pdf");
    assert_eq!(shared["bytes"], 17);
    assert_eq!(shared["uploader_name"], "Other User");
    let (status, headers, _) = owner
        .request(
            &format!("/communities/files/{}", shared["id"].as_str().unwrap()),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FOUND);
    assert!(
        headers["location"]
            .to_str()
            .unwrap()
            .starts_with("https://storage.test/")
    );
    // Nobody shares another person's storage, or into a community they are not in.
    assert_eq!(
        community_call(
            &mut owner,
            "share_community_cloud_files",
            json!({"id":id,"channel":"","files":[cloud_id]}),
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        community_call(
            &mut outsider,
            "share_community_cloud_files",
            json!({"id":id,"channel":"","files":[cloud_id]}),
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        community_call(
            &mut member,
            "share_community_cloud_files",
            json!({"id":id,"channel":"","files":[]}),
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        community_call(
            &mut member,
            "share_community_cloud_files",
            json!({"id":id,"channel":""}),
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );

    // Deleting the post withdraws its file.
    community_call(
        &mut owner,
        "delete_community_post",
        json!({"id":id,"post":post_id}),
    )
    .await;
    let (_, _, page) = member
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(!page.contains(&file_id), "A deleted post hides its file");
    assert_eq!(
        member
            .request(&format!("/communities/files/{file_id}"), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn community_actions_enforce_roles_invites_links_and_owner_transfer() {
    setup().await;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let mut other = super::auth::Client::new().await;
    other.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    other.request("/communities", None).await;
    let (status, created) = community_call(
        &mut owner,
        "create_community",
        json!({"name":"Action lifecycle school","category":"School"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = created["id"].as_str().unwrap();
    let (status, link) = community_call(
        &mut owner,
        "community_join_link",
        json!({"id":id,"reset":false}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = link.as_str().unwrap().split("join=").nth(1).unwrap();
    assert_eq!(
        community_call(
            &mut other,
            "join_community",
            json!({"id":id,"token":"invalid"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        community_call(&mut other, "community_members", json!({"id":id}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        community_call(
            &mut other,
            "edit_community",
            json!({"id":id,"name":"Stolen","description":"","guidelines":"","remove_picture":false})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        community_call(
            &mut owner,
            "invite_community_member",
            json!({"id":id,"email":"other@example.com"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, _, pending) = other.request("/communities", None).await;
    assert!(pending.contains("Action lifecycle school"));
    assert_eq!(
        community_call(
            &mut other,
            "publish_community_post",
            json!({"id":id,"channel":"","body":"Before accepting","announcement":false})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (status, joined) = community_call(&mut other, "join_community", json!({"id":id})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(joined["is_owner"], false);
    assert_eq!(joined["member_count"], 2);
    let (_, post) = community_call(
        &mut other,
        "publish_community_post",
        json!({"id":id,"channel":"","body":"Member post author","announcement":false}),
    )
    .await;
    assert_eq!(post["author_name"], "Other User");
    for (name, args) in [
        ("community_join_link", json!({"id":id,"reset":true})),
        ("delete_community", json!({"id":id})),
        (
            "create_community_channel",
            json!({"id":id,"name":"Unauthorized channel"}),
        ),
        (
            "invite_community_member",
            json!({"id":id,"email":"fixture@example.com"}),
        ),
    ] {
        assert_eq!(
            community_call(&mut other, name, args).await.0,
            StatusCode::NOT_FOUND,
            "{name}"
        );
    }
    assert_eq!(
        community_call(
            &mut other,
            "set_community_notifications",
            json!({"id":id,"preference":"off"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, _, fresh) = other
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(fresh.contains("Action lifecycle school"));
    let (_, refreshed) = community_call(&mut other, "join_community", json!({"id":id})).await;
    assert_eq!(refreshed["notifications"], "off");
    let (_,edited)=community_call(&mut owner,"edit_community",json!({"id":id,"name":"Renamed school workspace","description":"Saved menu edit","guidelines":"Updated rules","remove_picture":false})).await;
    assert_eq!(edited["name"], "Renamed school workspace");
    assert_eq!(edited["posts"][0]["author_name"], "Other User");
    assert_eq!(
        community_call(&mut owner, "leave_community", json!({"id":id}))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        community_call(&mut other, "leave_community", json!({"id":id}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        community_call(
            &mut other,
            "publish_community_post",
            json!({"id":id,"channel":"","body":"After leaving","announcement":false})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, new_link) = community_call(
        &mut owner,
        "community_join_link",
        json!({"id":id,"reset":true}),
    )
    .await;
    assert_ne!(link, new_link);
    assert_eq!(
        community_call(&mut other, "join_community", json!({"id":id,"token":token}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let new_token = new_link.as_str().unwrap().split("join=").nth(1).unwrap();
    assert_eq!(
        community_call(
            &mut other,
            "join_community",
            json!({"id":id,"token":new_token})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        community_call(
            &mut owner,
            "transfer_community_ownership",
            json!({"id":id,"target":"other-user"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        community_call(&mut owner, "delete_community", json!({"id":id}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        community_call(&mut owner, "leave_community", json!({"id":id}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        community_call(&mut other, "leave_community", json!({"id":id}))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        community_call(&mut other, "delete_community", json!({"id":id}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        community_call(
            &mut owner,
            "join_community",
            json!({"id":id,"token":new_token})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        community_call(
            &mut other,
            "publish_community_post",
            json!({"id":id,"channel":"","body":"Deleted community","announcement":false})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn community_reports_are_saved_and_visible_only_to_admins() {
    setup().await;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let (_, created) = community_call(
        &mut owner,
        "create_community",
        json!({"name":"Reporting workspace","category":"School"}),
    )
    .await;
    let id = &created["id"];
    let (status, receipt) = community_call(
        &mut owner,
        "report_community",
        json!({"id":id,"details":"Review the reported community concern."}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        owner.request("/communities/reports", None).await.0,
        StatusCode::FORBIDDEN
    );
    let mut admin = super::auth::Client::new().await;
    admin.with_cookie(super::support::cookie_for(
        "test-admin",
        "Test Administrator",
        "admin-test@example.com",
    ));
    let (status, _, body) = admin.request("/communities/reports", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(receipt.as_str().unwrap()));
    assert!(body.contains("Review the reported community concern."));
    let mut anonymous = super::auth::Client::new().await;
    for name in [
        "edit_community",
        "community_members",
        "community_join_link",
        "invite_community_member",
        "join_community",
        "set_community_notifications",
        "transfer_community_ownership",
        "remove_community_member",
        "leave_community",
        "delete_community",
        "report_community",
    ] {
        assert_eq!(
            community_call(&mut anonymous, name, json!({})).await.0,
            StatusCode::UNAUTHORIZED,
            "{name}"
        );
        assert_eq!(
            community_call(&mut owner, name, json!({})).await.0,
            StatusCode::BAD_REQUEST,
            "{name}"
        );
    }
}

#[tokio::test]
async fn community_menu_matches_teams_and_edits_persist_in_browser() {
    let (status, body) = rpc(
        "/communities",
        "create_community",
        r#"{"name":"Menu browser school","category":"School"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let created: Value = serde_json::from_str(&body).unwrap();
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    let url = format!(
        "http://{address}/communities?community={}",
        created["id"].as_str().unwrap()
    );
    browser.goto(&url).await.unwrap();
    browser.wait_for("document.querySelector('.community-identity-title h2').textContent === 'Menu browser school'").await.unwrap();
    assert!(browser.eval::<bool>("Array.from(document.querySelectorAll('.community-channel')).every(button=>button.textContent.trim()!=='General')").await.unwrap());
    browser
        .click("button[aria-label='More options for Menu browser school']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#community-actions-menu').matches(':popover-open') && document.activeElement.textContent.includes('Share join link')").await.unwrap();
    let labels:Vec<String>=browser.eval("Array.from(document.querySelectorAll('#community-actions-menu button')).map(button=>button.textContent.trim())").await.unwrap();
    assert_eq!(
        labels,
        vec![
            "Share join link",
            "Invite members",
            "Notifications",
            "Manage community",
            "Edit community",
            "Report a concern",
            "Leave community",
            "Delete community"
        ]
    );
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('#community-actions-menu').matches(':popover-open')")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More options for Menu browser school']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#community-actions-menu').matches(':popover-open') && document.activeElement.textContent.includes('Share join link')").await.unwrap();
    let _:bool=browser.eval("document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'End',bubbles:true}));true").await.unwrap();
    browser
        .wait_for("document.activeElement.textContent.trim() === 'Delete community'")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .click("button[aria-label='More options for Menu browser school']")
        .await
        .unwrap();
    browser
        .click("#community-actions-menu button:nth-of-type(5)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.community-create-dialog[open]') && document.querySelector('.community-create-dialog h2').textContent === 'Edit community'").await.unwrap();
    browser.click("#community-name").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select();true")
        .await
        .unwrap();
    browser.type_text("Saved menu rename").await.unwrap();
    browser
        .click(".community-create-form button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.community-create-dialog[open]') && document.querySelector('.community-identity-title h2').textContent === 'Saved menu rename'").await.unwrap();
    browser.goto(&url).await.unwrap();
    browser.wait_for("document.querySelector('.community-identity-title h2').textContent === 'Saved menu rename'").await.unwrap();
    browser
        .click("button[aria-label='More options for Saved menu rename']")
        .await
        .unwrap();
    browser
        .click("#community-actions-menu button:nth-of-type(3)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.community-action-dialog[open]') && !document.querySelector('#community-notification-preference').hidden").await.unwrap();
    browser
        .click("#community-notification-preference")
        .await
        .unwrap();
    browser.press("ArrowDown").await.unwrap();
    browser.press("ArrowDown").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .click(".community-action-dialog form:not([hidden]) button[type=submit]")
        .await
        .unwrap();
    browser
        .wait_for(
            "document.querySelector('.community-action-notice').textContent.includes('saved')",
        )
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-action-dialog[open]')")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More options for Saved menu rename']")
        .await
        .unwrap();
    browser
        .click("#community-actions-menu button:nth-of-type(8)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.community-action-dialog[open]') && document.querySelector('.community-action-dialog h2').textContent === 'Delete community'").await.unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-nav-item[aria-current=page]').textContent.includes('Saved menu rename')").await.unwrap(),"Opening delete must not delete anything");
    browser.press("Escape").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

fn community_test_png() -> Vec<u8> {
    let mut output = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(8, 4)
        .write_to(&mut output, image::ImageFormat::Png)
        .unwrap();
    output.into_inner()
}

#[tokio::test]
async fn channel_creation_limit_is_atomic_and_icons_and_validation_persist() {
    setup().await;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let (_, created) = community_call(
        &mut owner,
        "create_community",
        json!({"name":"Channel limit workspace","category":"School"}),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();
    for name in ["", "   ", &"x".repeat(61)] {
        assert_eq!(
            community_call(
                &mut owner,
                "create_community_channel",
                json!({"id":id,"name":name})
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        community_call(
            &mut owner,
            "create_community_channel",
            json!({"id":id,"name":"Bad icon","icon":"<script>"})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let (status, first) = community_call(
        &mut owner,
        "create_community_channel",
        json!({"id":id,"name":"Homework","icon":"📚"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["icon"], "📚");
    assert_eq!(
        community_call(
            &mut owner,
            "create_community_channel",
            json!({"id":id,"name":"  HOMEWORK  "})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut requests = Vec::new();
    for number in 0..8 {
        let mut client = owner.clone();
        let id = id.clone();
        requests.push(tokio::spawn(async move {
            community_call(
                &mut client,
                "create_community_channel",
                json!({"id":id,"name":format!("Topic {number}")}),
            )
            .await
        }));
    }
    let mut accepted = 0;
    for request in requests {
        let (status, body) = request.await.unwrap();
        if status == StatusCode::OK {
            accepted += 1;
            assert_eq!(body["icon"], "💬");
        } else {
            assert_eq!(status, StatusCode::CONFLICT, "{body}");
        }
    }
    assert_eq!(
        accepted, 4,
        "Only the remaining four slots may be created concurrently"
    );
    let (status, body) = community_call(
        &mut owner,
        "create_community_channel",
        json!({"id":id,"name":"Sixth channel"}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.to_string().contains("5 channels"));
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let saved = crate::models::community_channel::Entity::find()
        .filter(crate::models::community_channel::Column::CommunityId.eq(&id))
        .all(crate::db::db())
        .await
        .unwrap();
    assert_eq!(saved.len(), 5);
    assert!(saved.iter().any(|c| c.name == "Homework" && c.icon == "📚"));
}

#[tokio::test]
async fn canceling_channel_dialog_discards_drafts_and_errors_keep_it_open() {
    let (_, body) = rpc(
        "/communities",
        "create_community",
        r#"{"name":"Channel dialog school","category":"School"}"#,
    )
    .await;
    let created: Value = serde_json::from_str(&body).unwrap();
    let id = created["id"].as_str().unwrap();
    for name in ["Existing", "Second", "Third", "Fourth"] {
        assert_eq!(
            rpc(
                "/communities",
                "create_community_channel",
                &json!({"id":id,"name":name}).to_string()
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/communities?community={id}"))
        .await
        .unwrap();
    browser.click(".community-add-channel").await.unwrap();
    browser.wait_for("document.querySelector('.community-channel-dialog[open]') && document.activeElement.id === 'channel-name'").await.unwrap();
    browser.type_text("Canceled draft").await.unwrap();
    browser
        .click(".community-channel-footer button[type=button]")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-channel-dialog[open]')")
        .await
        .unwrap();
    assert!(
        browser
            .eval::<bool>("document.querySelectorAll('.community-channel').length === 4")
            .await
            .unwrap()
    );
    browser.click(".community-add-channel").await.unwrap();
    browser.wait_for("document.querySelector('.community-channel-dialog[open]') && document.querySelector('#channel-name').value === '' && document.querySelector('.community-channel-form button[type=submit]').disabled").await.unwrap();
    browser.type_text("Existing").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-channel-form button[type=submit]').disabled")
        .await
        .unwrap();
    browser
        .click(".community-channel-form button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('#community-channel-error').hidden && document.querySelector('#community-channel-error').textContent.includes('already exists')").await.unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-channel-dialog').open && document.querySelector('#channel-name').value === 'Existing'").await.unwrap());
    browser.click("#channel-name").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select();true")
        .await
        .unwrap();
    browser.type_text("Fifth").await.unwrap();
    browser
        .click(".community-channel-form button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.community-channel-dialog[open]') && document.querySelectorAll('.community-channel').length === 5").await.unwrap();
    browser.click(".community-add-channel").await.unwrap();
    browser.wait_for("document.querySelector('.community-channel-dialog[open]') && document.querySelector('.community-channel-form button[type=submit]').disabled && document.querySelector('#community-channel-error').textContent.includes('5 channels')").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-channel-dialog[open]')")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn canceling_the_community_image_picker_keeps_the_creation_dialog_open() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/communities"))
        .await
        .unwrap();
    let _: bool = browser.eval("Array.from(document.querySelectorAll('main button')).find(button => button.textContent.trim().endsWith('School')).click(); true").await.unwrap();
    browser.wait_for("document.querySelector('.community-create-dialog[open]') && document.querySelector('#community-name').value === 'School'").await.unwrap();
    let _: bool = browser
        .eval("document.querySelector('#community-description').focus(); true")
        .await
        .unwrap();
    browser
        .type_text("Keep this community draft.")
        .await
        .unwrap();
    // The native file picker sends a non-cancelable, bubbling `cancel` on
    // its input when Cancel, the window close button, or Escape dismisses it.
    let retained: bool = browser.eval("(() => { document.querySelector('#community-picture').dispatchEvent(new Event('cancel', {bubbles:true})); const dialog = document.querySelector('.community-create-dialog'); return dialog.open && dialog.dataset.state === 'open' && document.querySelector('#community-name').value === 'School' && document.querySelector('#community-description').value === 'Keep this community draft.'; })()").await.unwrap();
    assert!(
        retained,
        "canceling the image picker must keep the parent dialog and its draft"
    );
    // Escape on the creation dialog itself must still dismiss it normally.
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-create-dialog[open]')")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn community_template_edits_and_creates_in_one_step() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/communities"))
        .await
        .unwrap();
    for template in ["Business", "Life", "School"] {
        let _: bool = browser.eval(&format!("Array.from(document.querySelectorAll('main button')).find(button => button.textContent.trim().endsWith('{template}')).click(); true")).await.unwrap();
        browser.wait_for(&format!("document.querySelector('.community-create-dialog[open]') && document.querySelector('#community-name').value === '{template}'")).await.unwrap();
        browser.wait_for("document.querySelector('.community-create-dialog').getAnimations().every(animation => animation.playState === 'finished')").await.unwrap();
        let _: bool = browser.eval("document.querySelector('button[aria-label=\"Close community creation\"]').click(); true").await.unwrap();
        browser
            .wait_for("!document.querySelector('.community-create-dialog[open]')")
            .await
            .unwrap();
    }
    // Template buttons have compiled event bindings, so select by their text.
    let _: bool = browser.eval("Array.from(document.querySelectorAll('main button')).find(button => button.textContent.trim().endsWith('School')).click(); true").await.unwrap();
    browser.wait_for("document.querySelector('.community-create-dialog[open]') && document.querySelector('#community-name').value === 'School'").await.unwrap();
    browser.wait_for("document.querySelector('.community-create-dialog').getAnimations().every(animation => animation.playState === 'finished')").await.unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-create-dialog').textContent.includes('Create your school community')").await.unwrap());
    let image = base64::engine::general_purpose::STANDARD.encode(community_test_png());
    let _: bool = browser.eval(&format!(r#"(() => {{ const input = document.querySelector('#community-picture'); const data = new DataTransfer(); data.items.add(new File([Uint8Array.from(atob('{image}'), c => c.charCodeAt(0))], 'community.png', {{type:'image/png'}})); input.files = data.files; input.dispatchEvent(new Event('change', {{bubbles:true}})); return true; }})()"#)).await.unwrap();
    browser.wait_for("!document.querySelector('.community-avatar-photo').hidden && document.querySelector('.community-avatar-photo').naturalWidth === 8").await.unwrap();
    // A bad replacement must leave the existing valid preview in place.
    let _: bool = browser.eval("(() => { const input = document.querySelector('#community-picture'); const data = new DataTransfer(); data.items.add(new File(['invalid'], 'picture.txt', {type:'text/plain'})); input.files = data.files; input.dispatchEvent(new Event('change', {bubbles:true})); return true; })()").await.unwrap();
    browser.wait_for("document.querySelector('main').textContent.includes('Choose a PNG or JPEG picture up to 2 MB.') && document.querySelector('.community-avatar-photo').naturalWidth === 8").await.unwrap();
    let _: bool = browser
        .eval("document.querySelector('#community-description').focus(); true")
        .await
        .unwrap();
    browser.type_text("A place for our school.").await.unwrap();
    browser
        .click("button[aria-label='Edit community guidelines']")
        .await
        .unwrap();
    let focused = browser
        .wait_for("document.activeElement.id === 'community-guidelines'")
        .await;
    assert!(focused.is_ok(),"{:?}; {}",focused,browser.eval::<String>("JSON.stringify({active:document.activeElement.outerHTML,expanded:document.querySelector('button[aria-label=\"Done editing community guidelines\"]')?.outerHTML,field:document.querySelector('#community-guidelines').outerHTML,dialog:document.querySelector('.community-create-dialog').open})").await.unwrap());
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser
        .type_text("Respect each other and share ideas.")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Done editing community guidelines']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.community-guidelines-preview').textContent === 'Respect each other and share ideas.'").await.unwrap();
    let fits: bool = browser.eval("(() => { const box = document.querySelector('.community-create-dialog').getBoundingClientRect(); return box.top >= 0 && box.left >= 0 && box.bottom <= innerHeight && box.right <= innerWidth; })()").await.unwrap();
    assert!(fits, "creation dialog fits the viewport");
    let _: bool = browser.eval(r##"document.querySelector(".community-create-form button[type=submit]").click(); true"##).await.unwrap();
    browser.wait_for("!document.querySelector('.community-create-dialog[open]') && document.querySelector('main').textContent.includes('A place for our school.')").await.unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('.community-saved-picture')).some(image => !image.hidden && image.naturalWidth === 256)").await.unwrap();
    let url: String = browser.eval("location.href").await.unwrap();
    browser.goto(&url).await.unwrap();
    browser.wait_for("document.querySelector('.community-nav-item[aria-current=page]')?.textContent.includes('School') && document.querySelector('.community-saved-picture').naturalWidth === 256").await.unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-details-panel').textContent.includes('Respect each other and share ideas.')").await.unwrap());
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn community_picture_upload_normalizes_images_and_rejects_invalid_files() {
    setup().await;
    let mut client = super::auth::Client::new().await;
    client.with_cookie(test_cookie());
    client.request("/communities", None).await;
    let fields = serde_json::json!({"name":"School", "category":"School"});
    let (status, body) = client
        .upload(
            "/communities",
            "create_community",
            fields.clone(),
            ("picture", "image/png", &community_test_png()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let community: serde_json::Value = serde_json::from_str(&body).unwrap();
    let picture = community["picture"]
        .as_str()
        .unwrap()
        .strip_prefix("data:image/png;base64,")
        .unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(picture)
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    assert_eq!((image.width(), image.height()), (256, 256));
    for (mime, data) in [
        ("image/png", b"not a PNG".to_vec()),
        ("image/svg+xml", b"<svg></svg>".to_vec()),
        ("image/png", vec![0; 2 * 1024 * 1024 + 1]),
    ] {
        let (status, body) = client
            .upload(
                "/communities",
                "create_community",
                fields.clone(),
                ("picture", mime, &data),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(body.contains("picture"));
    }
}

#[tokio::test]
async fn community_banner_upload_is_cropped_kept_and_replaced_by_presets() {
    setup().await;
    let mut client = super::auth::Client::new().await;
    client.with_cookie(test_cookie());
    client.request("/communities", None).await;
    let (_, _, body) = client
        .request(
            "/communities",
            Some((
                "create_community",
                serde_json::json!({"name":"Banner school", "category":"School"}),
            )),
        )
        .await;
    let id = serde_json::from_str::<serde_json::Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let fields = serde_json::json!({"id":id, "cover":"custom"});
    let (status, body) = client
        .upload(
            "/communities",
            "change_community_cover",
            fields.clone(),
            ("image", "image/png", &community_test_png()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let banner: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(banner["cover"], "custom");
    let encoded = banner["cover_image"]
        .as_str()
        .unwrap()
        .strip_prefix("data:image/jpeg;base64,")
        .unwrap();
    let image = image::load_from_memory(
        &base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap(),
    )
    .unwrap();
    assert_eq!((image.width(), image.height()), (1600, 400));
    // Choosing the uploaded tile again without a new file keeps the image.
    let (status, _, body) = client
        .request(
            "/communities",
            Some(("change_community_cover", fields.clone())),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).unwrap()["cover_image"],
        banner["cover_image"]
    );
    let (status, body) = client
        .upload(
            "/communities",
            "change_community_cover",
            fields,
            ("image", "image/svg+xml", b"<svg></svg>"),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("image"));
    let (_, _, page) = client
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(page.contains("data:image/jpeg;base64,"));
    client
        .request(
            "/communities",
            Some((
                "change_community_cover",
                serde_json::json!({"id":id, "cover":"stars"}),
            )),
        )
        .await;
    let (_, _, page) = client
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(!page.contains("data:image/jpeg;base64,"));
}

#[tokio::test]
async fn community_details_round_trip_and_reject_invalid_payloads() {
    let payload = serde_json::json!({"name":"School", "category":"School", "description":"Our school", "guidelines":"Be kind", "avatar":"🎓"});
    let (status, body) = rpc("/communities", "create_community", &payload.to_string()).await;
    assert_eq!(status, StatusCode::OK);
    let community: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(community["description"], "Our school");
    assert_eq!(community["guidelines"], "Be kind");
    assert_eq!(community["avatar"], "🎓");
    let mut invalid = payload.clone();
    invalid["description"] = serde_json::Value::String("x".repeat(1001));
    let (status, body) = rpc("/communities", "create_community", &invalid.to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("description"));
    let (status, _) = rpc(
        "/communities",
        "create_community",
        r#"{"name":3,"category":"School"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn communities_channels_posts_and_covers_survive_reload_and_are_private() {
    use serde_json::json;
    let mut owner = super::auth::Client::new().await;
    owner.with_cookie(test_cookie());
    owner.request("/communities", None).await;
    let (status, _, body) = owner.request("/communities", Some(("create_community", json!({"name":"Persistent school workspace", "category":"School", "description":"Saved description", "guidelines":"Saved guidelines"})))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let community: serde_json::Value = serde_json::from_str(&body).unwrap();
    let id = community["id"].as_str().unwrap();
    let (status, _, body) = owner
        .request(
            "/communities",
            Some((
                "create_community_channel",
                json!({"id":id,"name":"Homework"}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let channel: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        owner
            .request(
                "/communities",
                Some((
                    "create_community_channel",
                    json!({"id":id,"name":"Homework"})
                ))
            )
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(owner.request("/communities", Some(("publish_community_post", json!({"id":id,"channel":channel["id"],"body":"Saved school announcement", "announcement":true, "headline":"Saved headline", "accent":"teal"})))).await.0, StatusCode::OK);
    assert_eq!(
        owner
            .request(
                "/communities",
                Some((
                    "change_community_cover",
                    json!({"id":id,"cover":"lavender"})
                ))
            )
            .await
            .0,
        StatusCode::OK
    );
    let mut fresh = super::auth::Client::new().await;
    fresh.with_cookie(test_cookie());
    let (status, _, body) = fresh
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    for value in [
        "Persistent school workspace",
        "Saved description",
        "Saved guidelines",
        "Homework",
        "Saved school announcement",
        "lavender",
    ] {
        assert!(body.contains(value), "Missing {value}");
    }
    let mut other = super::auth::Client::new().await;
    other.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    let (_, _, body) = other
        .request(&format!("/communities?community={id}"), None)
        .await;
    assert!(!body.contains("Persistent school workspace"));
    for (name, payload) in [
        (
            "create_community_channel",
            json!({"id":id,"name":"Unauthorized"}),
        ),
        (
            "publish_community_post",
            json!({"id":id,"channel":"","body":"Unauthorized","announcement":false}),
        ),
        ("change_community_cover", json!({"id":id,"cover":"sunset"})),
    ] {
        assert_eq!(
            other.request("/communities", Some((name, payload))).await.0,
            StatusCode::NOT_FOUND
        );
    }
    let mut anonymous = super::auth::Client::new().await;
    for name in [
        "create_community",
        "create_community_channel",
        "publish_community_post",
        "change_community_cover",
    ] {
        assert_eq!(
            anonymous
                .request("/communities", Some((name, json!({}))))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            owner
                .request("/communities", Some((name, json!({}))))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        owner
            .request(
                "/communities",
                Some((
                    "publish_community_post",
                    json!({"id":id,"channel":"missing","body":"Wrong channel","announcement":false})
                ))
            )
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        owner
            .request(
                "/communities",
                Some((
                    "publish_community_post",
                    json!({"id":id,"channel":"","body":"  ","announcement":false})
                ))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        owner
            .request(
                "/communities",
                Some(("change_community_cover", json!({"id":id,"cover":"invalid"})))
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for (payload, field) in [
        (
            json!({"id":id,"channel":"","body":"No headline","announcement":true}),
            "headline",
        ),
        (
            json!({"id":id,"channel":"","body":"Bad color","announcement":true,"headline":"Hi","accent":"neon"}),
            "accent",
        ),
        (
            json!({"id":id,"channel":"","body":"Plain","announcement":false,"headline":"Stray"}),
            "headline",
        ),
        (
            json!({"id":id,"channel":"","body":"Text","announcement":false,"formatting":[{"kind":"paragraph","runs":[{"text":"Different"}]}]}),
            "formatting",
        ),
        (
            json!({"id":id,"channel":"","body":"Text","announcement":false,"formatting":[{"kind":"paragraph","runs":[{"text":"Text","link":"javascript:alert(1)"}]}]}),
            "formatting",
        ),
    ] {
        let (status, _, body) = owner
            .request("/communities", Some(("publish_community_post", payload)))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(body.contains(field), "{body}");
    }
    let (status, _, body) = owner
        .request(
            "/communities",
            Some((
                "publish_community_post",
                json!({"id":id,"channel":"","body":"","announcement":false,"subject":"Subject only"}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _, body) = owner
        .request(
            "/communities",
            Some((
                "publish_community_post",
                json!({"id":id,"channel":"","body":"Bold move","announcement":false,"formatting":[{"kind":"bullet","runs":[{"text":"Bold","bold":true},{"text":" move"}]}]}),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, _, page) = owner
        .request(&format!("/communities?community={id}"), None)
        .await;
    for value in ["Subject only", "Saved headline", "teal", "bullet"] {
        assert!(page.contains(value), "Missing {value}");
    }
    let (status, _, body) = owner
        .request(
            "/communities",
            Some(("change_community_cover", json!({"id":id,"cover":"custom"}))),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("Upload an image for the banner."), "{body}");
    let (status, _, body) = owner
        .request(
            "/communities",
            Some(("change_community_cover", json!({"id":id,"cover":"compass"}))),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let banner: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(banner["cover"], "compass");
    assert!(banner["cover_image"].is_null());
}

#[tokio::test]
async fn community_workspace_saves_channels_posts_and_cover_in_the_browser() {
    let (status, body) = rpc(
        "/communities",
        "create_community",
        r#"{"name":"Browser school workspace","category":"School"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let community: serde_json::Value = serde_json::from_str(&body).unwrap();
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    let url = format!(
        "http://{address}/communities?community={}",
        community["id"].as_str().unwrap()
    );
    browser.goto(&url).await.unwrap();
    browser.wait_for("document.querySelector('.community-identity-title h2').textContent === 'Browser school workspace'").await.unwrap();
    browser.click(".community-add-channel").await.unwrap();
    browser.wait_for("document.querySelector('.community-channel-dialog[open]') && document.activeElement.id === 'channel-name'").await.unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-channel-dialog h2').textContent === 'Create a channel in \"Browser school workspace\"' && document.querySelector('.community-channel-form button[type=submit]').disabled").await.unwrap());
    browser
        .click(".community-channel-icon-trigger")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('#popover-channel-icon').matches(':popover-open')")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Search emoji']")
        .await
        .unwrap();
    browser.type_text("books").await.unwrap();
    browser.wait_for("document.querySelector('.channel-emoji-heading').textContent === 'Search results' && document.querySelector('button[data-emoji=\"📚\"]') !== null").await.unwrap();
    browser
        .click("button[aria-label='Use 📚 icon']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('#popover-channel-icon').matches(':popover-open') && document.querySelector('.community-channel-icon-trigger').textContent.includes('📚')").await.unwrap();
    browser.click("#channel-name").await.unwrap();
    browser.type_text("Class projects").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-channel-form button[type=submit]').disabled")
        .await
        .unwrap();
    browser
        .click(".community-channel-form button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.community-identity-title h2').textContent === 'Class projects'").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-channel-dialog[open]')")
        .await
        .unwrap();
    browser
        .click(".community-compose-mode[data-primary]")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-composer-expanded').hidden && document.querySelector('.community-rich-host .ProseMirror') !== null")
        .await
        .unwrap();
    assert!(
        browser
            .eval::<bool>(
                "document.querySelector('.community-composer-footer button[type=submit]').disabled"
            )
            .await
            .unwrap()
    );
    browser.click("input[aria-label=Subject]").await.unwrap();
    browser.type_text("Projects").await.unwrap();
    browser
        .click(".community-rich-host .ProseMirror")
        .await
        .unwrap();
    browser.type_text("Our first ").await.unwrap();
    browser
        .click(".community-format-toolbar button[aria-label=Bold]")
        .await
        .unwrap();
    browser.type_text("saved").await.unwrap();
    browser
        .click(".community-format-toolbar button[aria-label=Bold]")
        .await
        .unwrap();
    browser.type_text(" class project.").await.unwrap();
    browser
        .wait_for(
            "!document.querySelector('.community-composer-footer button[type=submit]').disabled",
        )
        .await
        .unwrap();
    browser
        .click(".community-composer-footer button[type=submit]")
        .await
        .unwrap();
    let posted = browser.wait_for("document.querySelector('.community-post .community-rich-message')?.innerText.trim() === 'Our first saved class project.' && document.querySelector('.community-post .community-rich-message .font-bold')?.innerText === 'saved' && document.querySelector('.community-post-subject').textContent.trim() === 'Projects' && document.querySelector('.community-composer').dataset.mode === 'collapsed'").await;
    assert!(posted.is_ok(),"{:?}; {}",posted,browser.eval::<String>("JSON.stringify({error:document.querySelector('.community-compose-error').textContent,html:document.querySelector('.community-post .community-rich-message')?.innerHTML,hidden:document.querySelector('.community-post .community-rich-message')?.hidden,mode:document.querySelector('.community-composer').dataset.mode,posts:Array.from(document.querySelectorAll('.community-post')).map(p=>p.innerText)})").await.unwrap());
    // Hovering a post shows the Teams toolbar: quick reactions, more
    // reactions, edit and more options.
    let toolbar: [f64; 2] = browser
        .eval("(() => { const r = document.querySelector('.community-post').getBoundingClientRect(); return [r.x + r.width / 2, r.y + 40]; })()")
        .await
        .unwrap();
    assert!(browser.eval::<bool>("getComputedStyle(document.querySelector('.community-post .community-post-actions')).opacity === '0'").await.unwrap());
    browser.move_mouse(toolbar[0], toolbar[1]).await.unwrap();
    browser
        .wait_for("getComputedStyle(document.querySelector('.community-post .community-post-actions')).opacity === '1' && document.querySelectorAll('.community-post .community-quick-reaction img[src*=fluent-emoji]').length === 4")
        .await
        .unwrap();
    browser
        .click(".community-post .community-post-actions button[aria-label='React with Like']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-post .community-reaction[aria-pressed=true]')?.getAttribute('aria-label') === 'Like: 1, including you'")
        .await
        .unwrap();
    // A picker opened from the toolbar keeps the toolbar shown beside it,
    // even once the pointer leaves the post.
    browser
        .click(".community-post .community-post-actions button[aria-label=\"More reactions\"]")
        .await
        .unwrap();
    browser.move_mouse(2.0, 2.0).await.unwrap();
    browser
        .wait_for("document.querySelector('#community-reaction-picker').matches(':popover-open') && document.querySelector('.community-post .community-post-actions button[aria-label=\"More reactions\"]').getAttribute('aria-expanded') === 'true' && getComputedStyle(document.querySelector('.community-post .community-post-actions')).opacity === '1'")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('#community-reaction-picker').matches(':popover-open') && document.querySelector('.community-post .community-post-actions button[aria-label=\"More reactions\"]').getAttribute('aria-expanded') === 'false'")
        .await
        .unwrap();
    browser
        .click(".community-post .community-reaction-add")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('#community-reaction-picker').matches(':popover-open') && document.activeElement.placeholder === 'Find something fun'")
        .await
        .unwrap();
    browser.type_text("party").await.unwrap();
    browser
        .wait_for("document.querySelector('#community-reaction-picker button[aria-label=\\'React with 🥳\\']') !== null")
        .await
        .unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .wait_for("!document.querySelector('#community-reaction-picker').matches(':popover-open') && Array.from(document.querySelectorAll('.community-post .community-reaction')).map(chip => chip.getAttribute('aria-label')).join('|') === 'Like: 1, including you|🥳: 1, including you'")
        .await
        .unwrap();
    browser
        .click(".community-post .community-reply-open")
        .await
        .unwrap();
    browser
        .wait_for("document.activeElement === document.querySelector('.community-post .community-reply-form textarea')")
        .await
        .unwrap();
    browser.type_text("Count me in").await.unwrap();
    browser.press("Enter").await.unwrap();
    browser
        .wait_for("document.querySelector('.community-post .community-reply-body')?.textContent === 'Count me in' && document.querySelector('.community-post .community-reply-form').hidden && document.querySelector('.community-post .community-conversation-toggle').getAttribute('aria-expanded') === 'true'")
        .await
        .unwrap();
    browser
        .click(".community-compose-mode:not([data-primary])")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-announcement-banner').hidden && document.activeElement.getAttribute('aria-label') === 'Headline'")
        .await
        .unwrap();
    browser.type_text("Science fair").await.unwrap();
    browser.click(".community-accent-button").await.unwrap();
    browser
        .click(".community-accent-palette button[value=green]")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-announcement-banner').dataset.accent === 'green' && document.querySelector('.community-accent-palette').hidden")
        .await
        .unwrap();
    browser
        .click(".community-composer-footer button[type=submit]")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-post .community-post-banner h3')?.textContent === 'Science fair' && document.querySelector('.community-post .community-post-banner').dataset.accent === 'green'")
        .await
        .unwrap();
    // Edit the earlier post from its toolbar; the composer reopens with its
    // subject and formatting and saves in place.
    let projects = "Array.from(document.querySelectorAll('.community-post')).find(post => post.querySelector('.community-post-subject')?.textContent.trim().startsWith('Projects'))";
    let _: bool = browser
        .eval(&format!("{projects}.querySelector('.community-post-actions button[aria-label=Edit]').click(), true"))
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-composer-editing').hidden && document.querySelector('input[aria-label=Subject]').value === 'Projects' && document.querySelector('.community-rich-host .ProseMirror strong')?.textContent === 'saved' && document.querySelector('.community-composer-footer button[type=submit]').textContent.trim() === 'Save'")
        .await
        .unwrap();
    browser.click("input[aria-label=Subject]").await.unwrap();
    let _: bool = browser
        .eval("(() => { const input = document.querySelector('input[aria-label=Subject]'); input.setSelectionRange(input.value.length, input.value.length); return true; })()")
        .await
        .unwrap();
    browser.type_text(" update").await.unwrap();
    browser
        .click(".community-composer-footer button[type=submit]")
        .await
        .unwrap();
    browser
        .wait_for(&format!("{projects}?.querySelector('.community-post-subject').textContent.trim() === 'Projects update' && !{projects}.querySelector('.community-post-edited').hidden && {projects}.querySelector('.community-rich-message .font-bold')?.innerText === 'saved' && document.querySelector('.community-composer').dataset.mode === 'collapsed'"))
        .await
        .unwrap();
    // Pin from the more-options menu: it moves to the top for everyone.
    let _: bool = browser
        .eval(&format!("{projects}.querySelector('.community-post-actions button[aria-label=\\'More options\\']').click(), true"))
        .await
        .unwrap();
    browser
        .wait_for(&format!("document.querySelector('#community-post-menu').matches(':popover-open') && document.activeElement.textContent.trim() === 'Copy link' && {projects}.querySelector('.community-post-actions button[aria-label=\"More options\"]').getAttribute('aria-expanded') === 'true' && getComputedStyle({projects}.querySelector('.community-post-actions')).opacity === '1'"))
        .await
        .unwrap();
    assert_eq!(
        browser.eval::<String>("Array.from(document.querySelectorAll('#community-post-menu [role=menuitem]')).filter(item => !item.hidden).map(item => item.innerText.replace(/\\s+/g, ' ').trim()).join('|')").await.unwrap(),
        "Copy link|Delete Del|Pin for everyone P|Open conversation in new window"
    );
    browser.press("p").await.unwrap();
    browser
        .wait_for("!document.querySelector('.community-post .community-post-pinned').hidden && document.querySelector('.community-post .community-post-subject').textContent.trim() === 'Projects update'")
        .await
        .unwrap();
    // Delete leaves a placeholder with Undo, which restores the post.
    let _: bool = browser
        .eval(&format!("{projects}.querySelector('.community-post-actions button[aria-label=\\'More options\\']').click(), true"))
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('#community-post-menu').matches(':popover-open')")
        .await
        .unwrap();
    // The driver has no Delete key; send the menu the same keydown.
    let _: bool = browser
        .eval("document.querySelector('#community-post-menu').dispatchEvent(new KeyboardEvent('keydown', {key: 'Delete', bubbles: true}))")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-post[data-deleted=true] .community-post-deleted')?.innerText.startsWith('This post has been deleted.')")
        .await
        .unwrap();
    browser
        .click(".community-post[data-deleted=true] .community-post-deleted button")
        .await
        .unwrap();
    browser
        .wait_for(&format!(
            "{projects} && !document.querySelector('.community-post[data-deleted=true]')"
        ))
        .await
        .unwrap();
    // A copied link opens the post with its conversation expanded.
    let link: String = browser
        .eval(&format!("location.origin + '/communities?community=' + new URLSearchParams(location.search).get('community') + '&post=' + {projects}.dataset.post"))
        .await
        .unwrap();
    browser.goto(&link).await.unwrap();
    browser
        .wait_for("document.querySelector('.community-post[data-focused=true] .community-post-subject')?.textContent.trim() === 'Projects update' && document.querySelector('.community-post[data-focused=true] .community-conversation-toggle').getAttribute('aria-expanded') === 'true'")
        .await
        .unwrap();
    browser.goto(&url).await.unwrap();
    browser.click(".community-cover-edit").await.unwrap();
    browser
        .wait_for("document.querySelector('.community-banner-dialog[open]') && document.querySelectorAll('.community-banner-choice:not([hidden])').length === 11")
        .await
        .unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-banner-choice[value=mint]').getAttribute('aria-checked') === 'true' && document.querySelector('.community-banner-footer > button:last-child').disabled").await.unwrap());
    browser
        .click(".community-banner-choice[value=lavender]")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-banner-choice[value=lavender]').getAttribute('aria-checked') === 'true' && !document.querySelector('.community-banner-footer > button:last-child').disabled")
        .await
        .unwrap();
    browser
        .click(".community-banner-footer > button:last-child")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-cover').dataset.cover === 'lavender' && document.querySelector('.community-cover-image').getAttribute('src') === '/images/banners/lavender.svg' && !document.querySelector('.community-banner-dialog').open")
        .await
        .unwrap();
    browser.goto(&url).await.unwrap();
    browser
        .wait_for("document.querySelector('.community-cover').dataset.cover === 'lavender'")
        .await
        .unwrap();
    let _: bool = browser.eval("Array.from(document.querySelectorAll('.community-channel')).find(button => button.textContent.trim().endsWith('Class projects')).click(); true").await.unwrap();
    assert!(browser.eval::<bool>("Array.from(document.querySelectorAll('.community-channel')).some(button => button.textContent.includes('📚') && button.textContent.includes('Class projects'))").await.unwrap());
    browser.wait_for("Array.from(document.querySelectorAll('.community-post .community-rich-message')).some(message => message.innerText.trim() === 'Our first saved class project.')").await.unwrap();
    browser
        .click("button[role=tab][aria-controls=community-files]")
        .await
        .unwrap();
    // Like Teams, the Files tab has no banner and starts from an upload prompt.
    browser.wait_for("!document.querySelector('#community-files').hidden && document.querySelector('#community-posts').hidden && document.querySelector('.community-cover').hidden && !document.querySelector('.community-files-empty').hidden && document.querySelector('.community-files-empty h3').textContent === 'Share files in this community'").await.unwrap();
    let _: bool = browser
        .eval("(() => { const input = document.querySelector('#community-files input[type=file]'); const data = new DataTransfer(); data.items.add(new File(['week one plan'], 'Plan.txt', {type:'text/plain'})); data.items.add(new File(['%PDF-1.4'], 'Rubric.pdf', {type:'application/pdf'})); input.files = data.files; input.dispatchEvent(new Event('change', {bubbles:true})); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelectorAll('.community-files-list tbody tr').length === 2 && !document.querySelector('.community-files-list').hidden && document.querySelector('.community-files-status').textContent.trim() === '2 files uploaded and posted.'")
        .await
        .unwrap();
    assert_eq!(
        browser.eval::<String>("Array.from(document.querySelectorAll('.community-files-list tbody tr')).map(row => row.querySelector('.community-file-icon').dataset.kind + ':' + row.querySelector('.community-file-name').textContent.trim().replace(/\\s+/g, ' ') + ':' + row.cells[3].textContent.trim()).join('|')").await.unwrap(),
        "pdf:PDFRubric.pdf:8 B|text:TXTPlan.txt:13 B"
    );
    assert!(browser.eval::<bool>("document.querySelector('.community-file-name').getAttribute('href').startsWith('/communities/files/')").await.unwrap());
    browser
        .click("button[role=tab][aria-controls=community-posts]")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-cover').hidden && document.querySelector('.community-post .community-file-card strong')?.textContent === 'Rubric.pdf'")
        .await
        .unwrap();
    browser
        .click("button[role=tab][aria-controls=community-files]")
        .await
        .unwrap();
    browser
        .click("button[role=tab][aria-controls=community-photos]")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('#community-photos').hidden && document.querySelector('.community-cover').hidden && !document.querySelector('.community-photos-empty').hidden && document.querySelector('.community-photos-empty h3').textContent === 'No photos shared in the community'")
        .await
        .unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-detail').getBoundingClientRect().right <= innerWidth + 1").await.unwrap());
    // Like Teams, the posts pane scrolls on its own: the banner leaves, while
    // the header and the collapsed composer stay pinned.
    browser
        .click("button[role=tab][aria-controls=community-posts]")
        .await
        .unwrap();
    let pinned = browser.eval::<String>("(async () => { const pane = document.querySelector('.community-detail'), main = document.querySelector('main'); pane.style.height = '420px'; pane.scrollTop = pane.scrollHeight; await new Promise(r => setTimeout(r, 100)); const top = pane.getBoundingClientRect().top, header = document.querySelector('.community-detail-header').getBoundingClientRect(), composer = document.querySelector('.community-composer').getBoundingClientRect(); const result = JSON.stringify({scrolled: pane.scrollTop > 0, header: Math.round(header.top - top), composer: Math.round(composer.top - header.bottom), mainScrolls: main.scrollHeight > main.clientHeight + 1}); pane.style.height = ''; pane.scrollTop = 0; return result; })()").await.unwrap();
    assert_eq!(
        pinned,
        r#"{"scrolled":true,"header":0,"composer":12,"mainScrolls":false}"#
    );
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
