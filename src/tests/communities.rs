use super::*;
use base64::Engine;
use rahti::serde_json;
use rahti::serde_json::{Value, json};

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
    assert_eq!(owner.request("/communities", Some(("publish_community_post", json!({"id":id,"channel":channel["id"],"body":"Saved school announcement", "announcement":true})))).await.0, StatusCode::OK);
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
    browser.click("#community-post-body").await.unwrap();
    browser
        .type_text("Our first saved class project.")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.community-composer button[type=submit]').disabled")
        .await
        .unwrap();
    browser
        .click(".community-composer button[type=submit]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.community-post p')?.textContent === 'Our first saved class project.'").await.unwrap();
    browser.click(".community-cover-edit").await.unwrap();
    browser
        .click(".community-cover-picker button:nth-child(2)")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.community-cover').dataset.cover === 'lavender'")
        .await
        .unwrap();
    browser.goto(&url).await.unwrap();
    browser
        .wait_for("document.querySelector('.community-cover').dataset.cover === 'lavender'")
        .await
        .unwrap();
    let _: bool = browser.eval("Array.from(document.querySelectorAll('.community-channel')).find(button => button.textContent === 'Class projects').click(); true").await.unwrap();
    browser.wait_for("document.querySelector('.community-post p')?.textContent === 'Our first saved class project.'").await.unwrap();
    browser
        .click("button[role=tab][aria-controls=community-files]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('#community-files').hidden && document.querySelector('#community-posts').hidden").await.unwrap();
    browser
        .click("button[role=tab][aria-controls=community-photos]")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('#community-photos').hidden")
        .await
        .unwrap();
    assert!(browser.eval::<bool>("document.querySelector('.community-detail').getBoundingClientRect().right <= innerWidth + 1").await.unwrap());
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
