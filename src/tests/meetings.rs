//! Meeting links, end to end: minted on `/meet`, kept, listed on the next
//! page load, and resolved at `/meet/j/{id}`.

use super::*;

#[tokio::test]
async fn meet_popovers_anchor_dismiss_and_open_the_calendar_form() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    browser
        .click("button[aria-controls='dropdown-menu-meeting-schedule']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#dropdown-menu-meeting-schedule').matches(':popover-open') && document.activeElement.textContent.includes('Schedule in Upurnaka')").await.unwrap();
    browser
        .click("button[aria-controls='popover-meeting-join']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#popover-meeting-join').matches(':popover-open') && !document.querySelector('#dropdown-menu-meeting-schedule').matches(':popover-open') && document.activeElement.id === 'meeting-join-id'").await.unwrap();
    // The panel zooms in as it opens; measure where it settles.
    browser
        .wait_for("document.querySelector('#popover-meeting-join').getAnimations().every(animation => animation.playState === 'finished')")
        .await
        .unwrap();
    let anchored: bool = browser.eval("(() => { const button = document.querySelector('button[aria-controls=\"popover-meeting-join\"]').getBoundingClientRect(); const panel = document.querySelector('#popover-meeting-join').getBoundingClientRect(); return Math.abs(panel.top - button.bottom - 4) < 2 && Math.abs(panel.right - button.right) < 2; })()").await.unwrap();
    assert!(anchored, "join panel must be anchored under its own action");
    assert!(
        browser
            .eval::<bool>(
                "document.querySelector('#popover-meeting-join button[type=submit]').disabled"
            )
            .await
            .unwrap()
    );
    browser.type_text("test-meeting").await.unwrap();
    browser
        .wait_for("!document.querySelector('#popover-meeting-join button[type=submit]').disabled")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("!document.querySelector('#popover-meeting-join').matches(':popover-open') && document.activeElement.getAttribute('aria-controls') === 'popover-meeting-join'").await.unwrap();
    browser
        .click("button[aria-controls='popover-meeting-join']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Close meeting ID dialog']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('#popover-meeting-join').matches(':popover-open')")
        .await
        .unwrap();
    browser
        .click("button[aria-controls='dropdown-menu-meeting-schedule']")
        .await
        .unwrap();
    browser.click("h1").await.unwrap();
    browser
        .wait_for(
            "!document.querySelector('#dropdown-menu-meeting-schedule').matches(':popover-open')",
        )
        .await
        .unwrap();
    browser
        .click("button[aria-controls='dropdown-menu-meeting-schedule']")
        .await
        .unwrap();
    browser
        .click("#dropdown-menu-meeting-schedule a[href='/calendar?new=1']")
        .await
        .unwrap();
    browser.wait_for("location.pathname === '/calendar' && Array.from(document.querySelectorAll('h2')).some(el => el.textContent === 'New meeting' && el.getBoundingClientRect().height > 0)").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// Exercise the complete join/leave UI with a deterministic SDK transport.
/// AWS and screen-capture permissions are not needed by this test.
#[tokio::test]
async fn joining_opens_the_meeting_room_and_leave_returns_to_setup() {
    let id = mint("Room transition").await;
    let router = crate::routes::router()
        .route("/js/vendor/amazon-chime-sdk.js", axum::routing::get(|| async {
            ([("content-type", "text/javascript")], include_str!("fixtures/chime-room.js"))
        }))
        .layer(axum::middleware::from_fn(|request: axum::extract::Request, next: axum::middleware::Next| async move {
            // This UI test uses a synthetic join response rather than a real
            // admitted provider seat. The full moderation tests cover this RPC.
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("resume_meeting_audio") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from("true")).unwrap();
            }
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("join_meeting") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(r#"{"meeting":{"MeetingId":"fixture"},"attendee":{"AttendeeId":"self","ExternalUserId":"Jefferson Abraham#fixture"}}"#)).unwrap();
            }
            next.run(request).await
        }));
    let address = rahti::browser::serve(router).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.meeting-primary')?.textContent.includes('Join now')")
        .await
        .unwrap();
    // Unit-test startup intentionally has no provider. Supply its successful
    // boundary response above and enable the button for this UI test only.
    let _: bool = browser
        .eval("document.querySelector('.meeting-primary').disabled=false; true")
        .await
        .unwrap();
    browser.click(".meeting-primary").await.unwrap();
    browser.wait_for("!document.querySelector('.meeting-room').hidden && document.querySelector('.room-waiting').textContent.includes('Waiting for others to join')").await.unwrap();
    let setup_hidden: bool = browser
        .eval("document.querySelector('.meeting-body').hidden")
        .await
        .unwrap();
    assert!(setup_hidden, "joining must replace the setup screen");
    browser
        .click("button[aria-label='Participants']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.room-side-panel h2').textContent === 'People (1)'")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.roomFixture.presence('guest',true,'Ada Lovelace#fixture'); true")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-side-panel h2').textContent === 'People (2)' && document.querySelector('.room-person-off').textContent.includes('Ada Lovelace')").await.unwrap();
    browser
        .click("button[aria-label='Raise hand']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-raised-hand').hidden")
        .await
        .unwrap();
    // A remote hand is attendee state, independent of whether a camera tile exists.
    let _: bool = browser.eval(r##"
        window.raiseGuestHand = raised => window.roomFixture.message({
            data: new Uint8Array([1]), senderAttendeeId:'guest', senderExternalUserId:'Ada Lovelace#fixture',
            json: () => ({kind:'hand', raised})
        });
        window.raiseGuestHand(true); true
    "##).await.unwrap();
    browser.wait_for("!document.querySelector('.room-person-off[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    let _: bool = browser.eval("window.roomFixture.observer.videoTileDidUpdate({tileId:10,boundAttendeeId:'guest',boundExternalUserId:'Ada Lovelace#fixture',paused:false}); true").await.unwrap();
    browser.wait_for("!document.querySelector('.room-person-video[data-attendee-id=guest]').hidden && document.querySelector('.room-person-off[data-attendee-id=guest]').hidden && !document.querySelector('.room-person-video[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    let visible_above_video: bool = browser.eval(r##"
        (() => {
            const tile = document.querySelector('.room-person-video[data-attendee-id=guest]');
            const badge = tile.querySelector('.room-hand-indicator');
            const rect = badge.getBoundingClientRect();
            window.handTestVideo = tile.querySelector('video');
            window.handTestStream = window.handTestVideo.srcObject;
            return rect.width > 0 && rect.height > 0 && badge.querySelector('svg') !== null &&
                badge.contains(document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2));
        })()
    "##).await.unwrap();
    assert!(
        visible_above_video,
        "the hand icon must be visible above the playing video"
    );
    // A playing camera must not cover the raised-hand frame at the tile edges.
    assert!(browser.eval::<bool>(r##"(() => {
        const tile = document.querySelector('.room-person-video[data-attendee-id=guest]');
        const frame = getComputedStyle(tile, '::after');
        const camera = tile.querySelector('video').getBoundingClientRect();
        const card = tile.getBoundingClientRect();
        return frame.content === '\"\"' && frame.position === 'absolute' &&
            Number(frame.zIndex) > 0 && frame.boxShadow.includes('inset') &&
            frame.pointerEvents === 'none' && frame.borderRadius === getComputedStyle(tile).borderRadius &&
            getComputedStyle(tile).overflow === 'hidden' &&
            camera.left >= card.left && camera.right <= card.right + 1;
    })()"##).await.unwrap(), "the rounded hand frame must paint over the camera without blocking controls");
    let _: bool = browser
        .eval("window.raiseGuestHand(false); true")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-person-video[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    let preserved: bool = browser.eval("document.querySelector('.room-person-video[data-attendee-id=guest] video') === window.handTestVideo && window.handTestVideo.srcObject === window.handTestStream && window.handTestStream.active").await.unwrap();
    assert!(
        preserved,
        "hand updates must preserve the SDK's video element and stream"
    );
    // Screen-sharing tiles must not acquire the camera owner's hand badge.
    let _: bool = browser.eval("window.raiseGuestHand(true); window.roomFixture.observer.videoTileDidUpdate({tileId:11,boundAttendeeId:'guest#content',boundExternalUserId:'Ada Lovelace#fixture',isContent:true,paused:false}); true").await.unwrap();
    browser.wait_for("!document.querySelector('.room-content-tile').hidden && !document.querySelector('.room-person-video[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    assert!(
        browser
            .eval::<bool>(
                "document.querySelector('.room-content-tile .room-hand-indicator').hidden"
            )
            .await
            .unwrap()
    );
    // Pausing, resuming, and removing the camera must retain the raised hand on the visible card.
    let _: bool = browser.eval("window.roomFixture.observer.videoTileDidUpdate({tileId:10,boundAttendeeId:'guest',boundExternalUserId:'Ada Lovelace#fixture',paused:true}); true").await.unwrap();
    browser.wait_for("!document.querySelector('.room-person-off[data-attendee-id=guest]').hidden && !document.querySelector('.room-person-off[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    let _: bool = browser.eval("window.roomFixture.observer.videoTileDidUpdate({tileId:10,boundAttendeeId:'guest',boundExternalUserId:'Ada Lovelace#fixture',paused:false}); true").await.unwrap();
    browser.wait_for("!document.querySelector('.room-person-video[data-attendee-id=guest]').hidden && !document.querySelector('.room-person-video[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    let _: bool = browser
        .eval("window.roomFixture.observer.videoTileWasRemoved(10); true")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.room-person-off[data-attendee-id=guest]').hidden && !document.querySelector('.room-person-off[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    let _: bool = browser
        .eval("window.raiseGuestHand(false); true")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-person-off[data-attendee-id=guest] .room-hand-indicator').hidden").await.unwrap();
    browser
        .click("button[aria-label='Meeting chat']")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Message everyone']")
        .await
        .unwrap();
    browser.type_text("Hello from the meeting").await.unwrap();
    browser
        .click("button[aria-label='Send meeting message']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-chat-messages').textContent.includes('Hello from the meeting')").await.unwrap();
    browser
        .click("button[aria-label='Send a reaction']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-reactions').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-reactions').hidden && !document.querySelector('.room-more-menu').hidden").await.unwrap();
    browser
        .click("button[aria-label='Send a reaction']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.room-reactions').hidden && document.querySelector('.room-more-menu').hidden").await.unwrap();
    browser
        .click("button[aria-label='Meeting chat']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-reactions').hidden && document.querySelector('.room-more-menu').hidden").await.unwrap();
    browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    browser.click(".room-stage").await.unwrap();
    browser
        .wait_for("document.querySelector('.room-more-menu').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser.wait_for("document.querySelector('.room-more-menu').hidden && document.activeElement.getAttribute('aria-label') === 'More meeting options'").await.unwrap();
    browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    browser
        .click(".room-more-menu button:first-child")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.meeting-background-panel').hidden && document.querySelector('.room-more-menu').hidden").await.unwrap();
    browser.wait_for(r##"(() => {
        const panel = document.querySelector('.meeting-background-panel').getBoundingClientRect();
        const toolbar = document.querySelector('.meeting-room > header').getBoundingClientRect();
        return panel.top >= toolbar.bottom && Array.from(document.querySelectorAll('.meeting-room > header > div:first-of-type + div > button')).every(button => {
            const rect = button.getBoundingClientRect();
            return button.contains(document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2));
        });
    })()"##).await.unwrap();
    browser
        .click("button[aria-label='Send a reaction']")
        .await
        .unwrap();
    browser.wait_for(r##"(() => {
        const menu = document.querySelector('.room-reactions');
        const rect = menu.getBoundingClientRect();
        const panel = document.querySelector('.meeting-background-panel').getBoundingClientRect();
        return !menu.hidden && rect.left >= 8 && rect.right <= panel.left - 7 && rect.bottom <= innerHeight &&
            Array.from(menu.querySelectorAll('button')).every(button => {
                const r = button.getBoundingClientRect();
                return button.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2));
            });
    })()"##).await.unwrap();
    browser
        .click("button[aria-label='Send laugh']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-reactions').hidden && !document.querySelector('.room-reaction-toast').hidden").await.unwrap();
    browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-more-menu').getBoundingClientRect().right <= document.querySelector('.meeting-background-panel').getBoundingClientRect().left - 7").await.unwrap();
    browser
        .click("button[aria-label='Close background settings']")
        .await
        .unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .click("button[aria-label='Send a reaction']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Send a reaction']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.room-reactions').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Send a reaction']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Send thumbs up']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-reaction-toast').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Toggle meeting microphone']")
        .await
        .unwrap();
    let muted: bool = browser.eval("window.roomFixture.muted").await.unwrap();
    assert!(muted);
    browser
        .click("button[aria-label='Toggle meeting camera']")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.room-local-video').hidden && document.querySelector('.room-local-video').srcObject !== null").await.unwrap();
    assert!(browser.eval::<bool>("getComputedStyle(document.querySelector('.room-self-tile'), '::after').boxShadow.includes('inset') && getComputedStyle(document.querySelector('.room-self-tile'), '::after').zIndex === '2'").await.unwrap(), "your camera must also retain its raised-hand outline");
    assert!(browser.eval::<bool>("!document.querySelector('.room-raised-hand').hidden && document.querySelector('.room-raised-hand').parentElement === document.querySelector('.room-local-video').parentElement").await.unwrap(), "your hand badge must stay with your camera tile");
    browser
        .click("button[aria-label='Toggle meeting camera']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-local-video').hidden && !document.querySelector('.room-raised-hand').hidden").await.unwrap();
    browser
        .click("button[aria-label='Raise hand']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.room-raised-hand').hidden")
        .await
        .unwrap();
    // Only sustained local, unmuted audio starts automatic lowering. Advance
    // the wall clock while keeping the real interval and browser bindings live.
    let _: bool = browser.eval("window.originalHandClock = Date.now; window.handClockOffset = 0; Date.now = () => window.originalHandClock() + window.handClockOffset; true").await.unwrap();
    browser
        .click("button[aria-label='Raise hand']")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.roomFixture.volume(0.8, true); window.handClockOffset += 3000; true")
        .await
        .unwrap();
    browser.pause(std::time::Duration::from_millis(1100)).await;
    assert!(browser.eval::<bool>("document.querySelector('.room-hand-auto-lower').hidden && !document.querySelector('.room-raised-hand').hidden").await.unwrap());
    browser
        .click("button[aria-label='Toggle meeting microphone']")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.roomFixture.volume(0, false); window.handClockOffset += 60000; true")
        .await
        .unwrap();
    browser.pause(std::time::Duration::from_millis(1100)).await;
    assert!(browser.eval::<bool>("document.querySelector('.room-hand-auto-lower').hidden && !document.querySelector('.room-raised-hand').hidden").await.unwrap(), "a silent raised hand has no expiry");
    browser
        .click("button[aria-label='Toggle meeting camera']")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.roomFixture.volume(0.6, false); window.handClockOffset += 3000; true")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.room-hand-auto-lower').hidden && document.querySelector('.room-hand-auto-lower').textContent.includes('5 seconds')").await.unwrap();
    browser
        .click(".room-hand-auto-lower > button")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.handClockOffset += 10000; window.roomFixture.volume(0.7, false); true")
        .await
        .unwrap();
    browser.pause(std::time::Duration::from_millis(1100)).await;
    assert!(browser.eval::<bool>("document.querySelector('.room-hand-auto-lower').hidden && !document.querySelector('.room-raised-hand').hidden").await.unwrap(), "Keep it raised cancels automatic lowering for this raise");
    browser
        .click("button[aria-label='Toggle meeting camera']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Raise hand']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Raise hand']")
        .await
        .unwrap();
    let _: bool = browser.eval("window.roomFixture.volume(0.6, false); window.roomFixture.volume(0, null); window.handClockOffset += 3000; true").await.unwrap();
    browser.pause(std::time::Duration::from_millis(1100)).await;
    assert!(
        browser
            .eval::<bool>("document.querySelector('.room-hand-auto-lower').hidden")
            .await
            .unwrap(),
        "a brief noise spike must not trigger the warning"
    );
    let _: bool = browser
        .eval("window.roomFixture.volume(0.6, false); window.handClockOffset += 3000; true")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-hand-auto-lower').hidden")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.roomFixture.observer.audioVideoDidStartConnecting(true); true")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.room-hand-auto-lower').hidden")
        .await
        .unwrap();
    let _: bool = browser.eval("window.roomFixture.observer.audioVideoDidStart(); window.handClockOffset += 6000; true").await.unwrap();
    browser.pause(std::time::Duration::from_millis(1100)).await;
    assert!(browser.eval::<bool>("document.querySelector('.room-hand-auto-lower').hidden && !document.querySelector('.room-raised-hand').hidden").await.unwrap(), "reconnecting cancels a stale countdown");
    let _: bool = browser
        .eval("window.roomFixture.volume(0.6, false); window.handClockOffset += 3000; true")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-hand-auto-lower').hidden")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.handClockOffset += 6000; true")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-raised-hand').hidden && document.querySelector('.room-hand-auto-lower').hidden && document.querySelector('button[aria-label=\"Raise hand\"]').getAttribute('aria-pressed') === 'false'").await.unwrap();
    assert!(browser.eval::<bool>("window.roomFixture.sent.filter(message => message.kind === 'hand').at(-1).raised === false").await.unwrap(), "automatic lowering must notify the other attendees");
    let _: bool = browser
        .eval("Date.now = window.originalHandClock; true")
        .await
        .unwrap();
    let _: bool = browser
        .eval(
            r##"
        (() => {
            navigator.mediaDevices.getDisplayMedia=async () => {
                const canvas=document.createElement('canvas'); canvas.width=640; canvas.height=360;
                return canvas.captureStream(15);
            };
            return true;
        })()
    "##,
        )
        .await
        .unwrap();
    browser
        .click("button[aria-label='Share screen']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Share screen\"]').getAttribute('aria-pressed') === 'true'").await.unwrap();
    browser
        .click("button[aria-label='Share screen']")
        .await
        .unwrap();
    let share_stopped: bool = browser
        .eval("window.roomFixture.shared.getTracks().every(track => track.readyState === 'ended')")
        .await
        .unwrap();
    assert!(
        share_stopped,
        "stopping a screen share must release its tracks"
    );
    browser
        .click("button[aria-label='Leave meeting']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.meeting-room').hidden && !document.querySelector('.meeting-body').hidden").await.unwrap();
    let stopped: bool = browser.eval("window.roomFixture.stopped && window.roomFixture.volume === null && window.roomFixture.controller.stream.getTracks().every(track => track.readyState === 'ended')").await.unwrap();
    assert!(stopped, "leaving must stop the transport and camera");
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

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

#[tokio::test]
async fn renaming_a_meeting_link_persists_without_changing_the_link() {
    let id = mint("Original rename title").await;
    let before = crate::meetings::find(&id).await.unwrap().unwrap();
    let payload = format!(r#"{{"id":"{id}","title":"  Updated planning name  "}}"#);
    let (status, body) = rpc("/meet", "rename_meeting_link", &payload).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let after = crate::meetings::find(&id).await.unwrap().unwrap();
    assert_eq!(after.title, "Updated planning name");
    assert_eq!(after.url, before.url);
    assert_eq!(after.created_at, before.created_at);
    let (_, listing) = get("/meet").await;
    assert!(listing.contains("Updated planning name"));
    let (status, joined) = get(&after.url).await;
    assert_eq!(status, StatusCode::OK);
    assert!(joined.contains("Updated planning name"));
}

#[tokio::test]
async fn renaming_rejects_invalid_payloads_and_unavailable_links() {
    let id = mint("Rename validation").await;
    for title in ["   ".to_string(), "x".repeat(121)] {
        let (status, body) = rpc(
            "/meet",
            "rename_meeting_link",
            &format!(r#"{{"id":"{id}","title":"{title}"}}"#),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert!(body.contains("title"));
    }
    let (status, _) = rpc("/meet", "rename_meeting_link", r#"{"title":"Missing id"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = rpc(
        "/meet",
        "rename_meeting_link",
        r#"{"id":"never-created","title":"No link"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    crate::meetings::revoke(&id).await.unwrap();
    let (status, _) = rpc(
        "/meet",
        "rename_meeting_link",
        &format!(r#"{{"id":"{id}","title":"Revoked"}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn meeting_link_actions_stand_out_from_their_card_in_both_appearances() {
    let id = mint("Visible actions").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    let card = format!("document.querySelector(\"[data-meeting-id='{id}']\")");
    let contrast = format!(
        "(() => {{ const card = {card}; const fill = (element) => getComputedStyle(element).backgroundColor; const join = card.querySelector('a'); const share = [...card.querySelectorAll('button')].find((button) => button.textContent.includes('Share link')); return fill(join) === fill(share) && fill(join) !== fill(card) && join.offsetHeight === share.offsetHeight && join.offsetWidth === share.offsetWidth; }})()"
    );
    browser.wait_for(&contrast).await.unwrap();
    browser
        .wait_for(&format!(
            "(document.documentElement.dataset.appearance = 'light', {contrast})"
        ))
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn meeting_name_editor_selects_saves_and_cancels() {
    let id = mint("Name editor original").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    let card = format!("[data-meeting-id='{id}']");
    let edit = format!("{card} button[aria-label='Edit meeting name']");
    browser.click(&edit).await.unwrap();
    browser.wait_for("document.querySelector('.meeting-rename-dialog').open && document.activeElement.getAttribute('aria-label') === 'Meeting name' && document.activeElement.selectionStart === 0 && document.activeElement.selectionEnd === document.activeElement.value.length").await.unwrap();
    browser.type_text("Name editor saved").await.unwrap();
    browser.click(".meeting-rename-save").await.unwrap();
    browser.wait_for(&format!("!document.querySelector('.meeting-rename-dialog').open && document.querySelector(\"{card}\").textContent.includes('Name editor saved')")).await.unwrap();
    browser
        .goto(&format!("http://{address}/meet"))
        .await
        .unwrap();
    browser
        .wait_for(&format!(
            "document.querySelector(\"{card}\")?.textContent.includes('Name editor saved')"
        ))
        .await
        .unwrap();
    browser.click(&edit).await.unwrap();
    browser.type_text("Cancelled edit").await.unwrap();
    browser.click(".meeting-rename-cancel").await.unwrap();
    browser
        .wait_for("!document.querySelector('.meeting-rename-dialog').open")
        .await
        .unwrap();
    assert_eq!(
        crate::meetings::find(&id).await.unwrap().unwrap().title,
        "Name editor saved"
    );
    browser.click(&edit).await.unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.meeting-rename-dialog').open")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
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
    assert!(
        body.contains("This meeting is unavailable right now"),
        "{body}"
    );
}

/// The meeting opens on its own screen, and its setup controls work before
/// asking for camera or microphone access.
#[tokio::test]
async fn pre_join_is_standalone_and_audio_choices_update() {
    let id = mint("Standalone pre-join").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        eprintln!("Skipping pre-join browser test: Chrome or Edge is not installed.");
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.meeting-camera-off')?.textContent.includes('Your camera is turned off')").await.unwrap();
    let dashboard_present: bool = browser.eval("document.querySelector('nav') !== null || document.querySelector('[aria-label=\"Search\"]') !== null").await.unwrap();
    assert!(
        !dashboard_present,
        "pre-join must not render the dashboard shell"
    );
    browser.wait_for("!document.querySelector('.meeting-titlebar') && document.querySelector('.meeting-join').scrollHeight <= document.querySelector('.meeting-join').clientHeight + 1 && document.querySelector('.meeting-footer').getBoundingClientRect().bottom <= innerHeight").await.unwrap();
    browser.click("input[value='none']").await.unwrap();
    browser
        .wait_for("document.querySelector('.meeting-audio-devices').hidden")
        .await
        .unwrap();
    browser.click("input[value='computer']").await.unwrap();
    browser
        .wait_for("!document.querySelector('.meeting-audio-devices').hidden")
        .await
        .unwrap();
    browser.click(".meeting-account button").await.unwrap();
    browser
        .wait_for("!document.querySelector('.meeting-name-editor').hidden")
        .await
        .unwrap();
    browser.click(".meeting-account button").await.unwrap();
    browser
        .wait_for("document.querySelector('.meeting-name-editor').hidden")
        .await
        .unwrap();
    browser.click(".meeting-background-button").await.unwrap();
    browser
        .wait_for("!document.querySelector('.meeting-background-panel').hidden")
        .await
        .unwrap();
    browser.wait_for("innerWidth <= 1100 || document.querySelector('.meeting-body').getBoundingClientRect().right <= document.querySelector('.meeting-background-panel').getBoundingClientRect().left + 1").await.unwrap();
    browser
        .click("button[aria-label='Light studio']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Light studio\"]').getAttribute('aria-pressed') === 'true'").await.unwrap();
    browser
        .click("button[aria-label='Close background settings']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.meeting-background-panel').hidden")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

/// Exercise real person segmentation on a generated video stream. This
/// checks worker, WASM, model and outgoing video without using a webcam.
#[tokio::test]
async fn meeting_background_replacement_processes_video_locally() {
    let id = mint("Background processing").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    let replaced: bool = browser.eval(r##"
        (async () => {
            const sdk = await import('/js/vendor/amazon-chime-sdk.js');
            const base = location.origin + '/js/vendor/background/';
            const spec = {
                paths: { worker: base + 'worker.js', wasm: base + '_cwt-wasm.wasm', simd: base + '_cwt-wasm-simd.wasm' },
                model: { path: base + 'selfie_segmentation_landscape.tflite', input: { width:256, height:144, channels:3, range:[0,1] }, output: { width:256, height:144, channels:1, range:[0,1] } }
            };
            const source = document.createElement('canvas'); source.width=640; source.height=360;
            const context = source.getContext('2d');
            context.fillStyle='#00bb00'; context.fillRect(0,0,640,360);
            const replacement = document.createElement('canvas'); replacement.width=640; replacement.height=360;
            const paint = replacement.getContext('2d'); paint.fillStyle='#ee2222'; paint.fillRect(0,0,640,360);
            const imageBlob = await new Promise(resolve => replacement.toBlob(resolve));
            const logger = new sdk.ConsoleLogger('background-test', sdk.LogLevel.ERROR);
            const processor = await sdk.BackgroundReplacementVideoFrameProcessor.create(spec, { imageBlob, logger });
            const stream = source.captureStream(15);
            const transform = new sdk.DefaultVideoTransformDevice(logger, stream, [processor]);
            const tick = setInterval(() => {context.fillRect(0,0,640,360);}, 60);
            const video = document.createElement('video'); video.muted=true;
            let result=false;
            try {
                video.srcObject=await transform.transformStream(stream);
                await video.play();
                const output = document.createElement('canvas'); output.width=640; output.height=360;
                const pixels=output.getContext('2d');
                for (let index=0; index<30; index++) {
                    await new Promise(resolve => video.requestVideoFrameCallback(resolve));
                    pixels.drawImage(video,0,0,640,360);
                    const color=pixels.getImageData(320,180,1,1).data;
                    if(color[0]>150 && color[1]<100) { result=true; break; }
                }
            } finally {
                clearInterval(tick); video.pause(); video.srcObject=null;
                await transform.stop(); stream.getTracks().forEach(track => track.stop());
            }
            return result;
        })()
    "##).await.unwrap();
    assert!(
        replaced,
        "the outgoing video must contain the replacement background"
    );
    // Feed that generated source through the application's actual camera
    // controls too, so switching effects exercises its device lifecycle.
    let _: bool = browser.eval(r##"
        (() => {
            const canvas=document.createElement('canvas'); canvas.width=640; canvas.height=360;
            const context=canvas.getContext('2d'); context.fillStyle='#00bb00';
            const tick=setInterval(() => context.fillRect(0,0,640,360),60);
            navigator.mediaDevices.enumerateDevices=async () => [{kind:'videoinput',deviceId:'test-camera',groupId:'test',label:'Generated camera'}];
            navigator.mediaDevices.getUserMedia=async () => canvas.captureStream(15);
            window.stopGeneratedCamera=() => clearInterval(tick);
            return true;
        })()
    "##).await.unwrap();
    browser.click(".meeting-background-button").await.unwrap();
    browser
        .click("button[aria-label='Light studio']")
        .await
        .unwrap();
    browser.click("input[aria-label='Camera']").await.unwrap();
    browser.wait_for("document.querySelector('input[aria-label=\"Camera\"]').checked && !document.querySelector('input[aria-label=\"Camera\"]').disabled && document.querySelector('.meeting-camera-off').hidden").await.unwrap();
    browser
        .click(".meeting-background-option:nth-child(2)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.meeting-background-option:nth-child(2)').getAttribute('aria-pressed') === 'true' && !document.querySelector('input[aria-label=\"Camera\"]').disabled").await.unwrap();
    browser
        .click(".meeting-background-option:first-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.meeting-background-option:first-child').getAttribute('aria-pressed') === 'true'").await.unwrap();
    browser.click("input[aria-label='Camera']").await.unwrap();
    browser
        .wait_for("!document.querySelector('input[aria-label=\"Camera\"]').checked")
        .await
        .unwrap();
    let _: bool = browser.eval(r##"
        (async () => {
            const canvas=document.createElement('canvas'); canvas.width=32; canvas.height=32;
            const blob=await new Promise(resolve => canvas.toBlob(resolve));
            const transfer=new DataTransfer(); transfer.items.add(new File([blob],'My background.png',{type:'image/png'}));
            const input=document.querySelector('input[type=file]'); input.files=transfer.files;
            input.dispatchEvent(new Event('change',{bubbles:true}));
            return true;
        })()
    "##).await.unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"My background.png\"]')?.getAttribute('aria-pressed') === 'true'").await.unwrap();
    let _: bool = browser.eval(r##"
        (() => {
            const transfer=new DataTransfer(); transfer.items.add(new File(['not an image'],'bad.png',{type:'image/png'}));
            const input=document.querySelector('input[type=file]'); input.files=transfer.files;
            input.dispatchEvent(new Event('change',{bubbles:true}));
            return true;
        })()
    "##).await.unwrap();
    browser
        .wait_for("!document.querySelector('.meeting-background-error').hidden")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.stopGeneratedCamera(); true")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
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
