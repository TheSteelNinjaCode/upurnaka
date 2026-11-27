//! Meeting links, end to end: minted on `/meet`, kept, listed on the next
//! page load, and resolved at `/meet/j/{id}`.

use super::*;

#[tokio::test]
async fn meet_popovers_anchor_dismiss_and_open_the_calendar_form() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
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
/// Dilnaka Meet and screen-capture permissions are not needed by this test.
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
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("meeting_chat_history") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(r#"[{"id":"saved-1","name":"Sammy","text":"From the last call","at":1793000000000}]"#)).unwrap();
            }
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("send_meeting_chat") {
                let bytes = axum::body::to_bytes(request.into_body(), 1 << 16).await.unwrap();
                let input: rahti::serde_json::Value = rahti::serde_json::from_slice(&bytes).unwrap();
                let answer = if input["file"] == "pick-1" {
                    r#"{"id":"sent-2","name":"Administrator","text":"","at":1793000002000,"file":{"name":"plan.pdf","content_type":"application/pdf","bytes":2048}}"#
                } else {
                    r#"{"id":"sent-1","name":"Administrator","text":"Hello from the meeting","at":1793000001000,"file":null}"#
                };
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(answer)).unwrap();
            }
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("meeting_chat_my_files") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(r#"{"view":"files","folder":null,"path":[],"folders":[{"id":"folder-1","name":"Projects","bytes":0,"items":0,"modified":"","modified_at":0}],"files":[{"id":"pick-1","name":"plan.pdf","content_type":"application/pdf","bytes":2048,"uploaded":"","uploaded_at":0}]}"#)).unwrap();
            }
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("meeting_chat_file_url") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(r#""https://storage.test/plan.pdf""#)).unwrap();
            }
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("join_meeting") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(r#"{"meeting":{"MeetingId":"fixture"},"attendee":{"AttendeeId":"self","ExternalUserId":"Jefferson Abraham#fixture"}}"#)).unwrap();
            }
            next.run(request).await
        }));
    let address = rahti::browser::serve(router).await;
    let Some(mut browser) = launch_browser().await else {
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
    // One step: a re-render between enabling and clicking disables it again.
    let _: bool = browser
        .eval("(() => { const join = document.querySelector('.meeting-primary'); join.disabled = false; join.click(); return true; })()")
        .await
        .unwrap();
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
    // A message arriving with the panel closed says so on the Chat button.
    let _: bool = browser
        .eval(
            r#"(() => {
        const say = id => window.roomFixture.message({
            throttled: false,
            data: new Uint8Array(1),
            senderAttendeeId: "guest",
            senderExternalUserId: "Sammy#12345678",
            json: () => ({ kind: "chat", id, text: "Are you there?" }),
        });
        say("live-1"); say("live-2");
        return true;
    })()"#,
        )
        .await
        .unwrap();
    browser.wait_for("(() => { const badge = document.querySelector('.room-chat-unread'); return !badge.hidden && badge.textContent.trim() === '2' && document.querySelector('button[aria-label=\"Meeting chat, 2 unread\"]') !== null && [...document.querySelectorAll('[role=status]')].some(status => !status.hidden && status.textContent.includes('Sammy: Are you there?')); })()").await.unwrap();
    browser
        .click("button[aria-label=\"Meeting chat, 2 unread\"]")
        .await
        .unwrap();
    // Opening the panel clears the count and shows what the last call said.
    browser.wait_for("document.querySelector('.room-chat-unread').hidden && document.querySelector('.room-chat-messages').textContent.includes('From the last call') && document.querySelector('.room-chat-messages').textContent.includes('Are you there?')").await.unwrap();
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
    // Share a file from My files, then download it from the chat.
    browser
        .click("button[aria-label='Attach a file']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-chat-attach-menu').hidden")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { [...document.querySelectorAll('.room-chat-attach-menu button')].find(b => b.textContent.includes('Choose from My files')).click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-chat-picker').hidden && document.querySelector('.room-chat-messages').hidden && document.querySelector('.room-picker-folder')?.textContent.includes('Projects') && document.querySelector('.room-picker-file')?.textContent.includes('plan.pdf')")
        .await
        .unwrap();
    browser.click(".room-picker-file").await.unwrap();
    browser
        .wait_for("document.querySelector('.room-chat-picker').hidden && !document.querySelector('.room-chat-pending').hidden && document.querySelector('.room-chat-pending').textContent.includes('plan.pdf') && !document.querySelector(\"button[aria-label='Send meeting message']\").disabled")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Send meeting message']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.room-chat-pending').hidden && [...document.querySelectorAll('.room-chat-file')].some(card => !card.hidden && card.textContent.includes('plan.pdf') && card.textContent.includes('2.0 KB'))")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { window.openedFiles = []; window.open = url => { window.openedFiles.push(url); return null; }; [...document.querySelectorAll('.room-chat-file')].find(card => !card.hidden).click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("window.openedFiles.length === 1 && window.openedFiles[0] === 'https://storage.test/plan.pdf'")
        .await
        .unwrap();
    // Device settings mid-call: microphone, speaker and camera change in place.
    let _: bool = browser
        .eval(
            r#"(() => {
        const device = (kind, deviceId, label) => ({ kind, deviceId, label, groupId: deviceId });
        globalThis.fixtureMicrophones = [{ deviceId: "array" }, { deviceId: "headset" }];
        navigator.mediaDevices.enumerateDevices = async () => [
            device("audioinput", "array", "Microphone Array"),
            device("audioinput", "headset", "Headset Microphone"),
            device("audiooutput", "soundwire", "SoundWire Speakers"),
            device("audiooutput", "usb", "USB Speakers"),
            device("videoinput", "test-camera", "Built-in camera"),
            device("videoinput", "second-camera", "USB camera"),
        ];
        navigator.mediaDevices.getUserMedia = async (constraints) => {
            const stream = new AudioContext().createMediaStreamDestination().stream;
            (globalThis.requestedMicrophones ||= []).push(constraints.audio?.deviceId?.exact);
            return stream;
        };
        navigator.mediaDevices.dispatchEvent(new Event("devicechange"));
        return true;
    })()"#,
        )
        .await
        .unwrap();
    browser
        .click("button[aria-label='Audio settings']")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-device-settings').hidden && document.querySelector('.room-side-panel h2').textContent.includes('Device settings') && document.querySelector('#room-camera').options.length === 3")
        .await
        .unwrap();
    let _: bool = browser
        .eval(
            r##"(() => {
        const pick = (id, value) => { const select = document.querySelector(id); select.value = value; select.dispatchEvent(new Event("change", { bubbles: true })); };
        pick("#room-microphone", "headset");
        pick("#room-speaker", "usb");
        return true;
    })()"##,
        )
        .await
        .unwrap();
    browser
        .wait_for("roomFixture.audioOutput === 'usb' && globalThis.requestedMicrophones?.at(-1) === 'headset' && !!roomFixture.audioInput")
        .await
        .unwrap();
    let _: bool = browser
        .eval(
            r##"(() => {
        const select = document.querySelector("#room-camera");
        select.value = "second-camera";
        select.dispatchEvent(new Event("change", { bubbles: true }));
        return true;
    })()"##,
        )
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('#room-camera').disabled")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { const toggle = document.querySelector(\"button[aria-label='Toggle meeting camera']\"); if (toggle.getAttribute('aria-pressed') !== 'true') toggle.click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("testDeviceController.lastInput === 'second-camera'")
        .await
        .unwrap();
    // Leave the camera as the rest of this test expects it: off.
    browser
        .wait_for("!document.querySelector(\"button[aria-label='Toggle meeting camera']\").disabled")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Toggle meeting camera']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector(\"button[aria-label='Toggle meeting camera']\").getAttribute('aria-pressed') === 'false' && !document.querySelector(\"button[aria-label='Toggle meeting camera']\").disabled")
        .await
        .unwrap();
    // The More menu leads to the same panel.
    browser
        .click("button[aria-label='Close meeting panel']")
        .await
        .unwrap();
    browser
        .click("button[aria-label='More meeting options']")
        .await
        .unwrap();
    let _: bool = browser
        .eval("(() => { [...document.querySelectorAll('.room-more-menu button')].find(b => b.textContent.includes('Device settings')).click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-device-settings').hidden && document.querySelector('.room-more-menu').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Close meeting panel']")
        .await
        .unwrap();
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
    // A phone browser has no screen capture: it says so, without a raw error.
    let _: bool = browser
        .eval("window.realShare = navigator.mediaDevices.getDisplayMedia; navigator.mediaDevices.getDisplayMedia = undefined; true")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Share screen']")
        .await
        .unwrap();
    browser.wait_for("[...document.querySelectorAll('[role=status]')].some(status => !status.hidden && status.textContent.includes('Join from a computer to share')) && !document.body.textContent.includes('is not a function') && document.querySelector('button[aria-label=\"Share screen\"]').getAttribute('aria-pressed') !== 'true'").await.unwrap();
    let _: bool = browser
        .eval("navigator.mediaDevices.getDisplayMedia = window.realShare; true")
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

/// A visit that is in the call, or waiting to be let in.
async fn visit(id: &str, name: &str, state: &str) -> String {
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    let ticket = format!("ticket-{}", crate::meetings::mint_id());
    crate::models::meeting_visit::ActiveModel {
        id: Set(ticket.clone()),
        meeting_id: Set(id.into()),
        user_id: Set(format!("guest:{ticket}")),
        name: Set(name.into()),
        state: Set(state.into()),
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
async fn meeting_chat_is_still_there_on_the_next_join() {
    let id = mint("Kept chat").await;
    let page = format!("/meet/j/{id}");
    let first = visit(&id, "Sammy", "joined").await;
    let payload = format!(r#"{{"id":"{id}","ticket":"{first}","text":"  See you tomorrow  "}}"#);
    let (status, body) = rpc(&page, "send_meeting_chat", &payload).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""name":"Sammy""#), "{body}");

    let (status, _) = rpc(
        &page,
        "leave_meeting_entry",
        &format!(r#"{{"id":"{id}","ticket":"{first}"}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let next = visit(&id, "Sammy", "joined").await;
    let (status, body) = rpc(
        &page,
        "meeting_chat_history",
        &format!(r#"{{"id":"{id}","ticket":"{next}"}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""text":"See you tomorrow""#), "{body}");
}

#[tokio::test]
async fn meeting_chat_is_only_for_people_in_the_call() {
    let id = mint("Private chat").await;
    let page = format!("/meet/j/{id}");
    let inside = visit(&id, "Sammy", "joined").await;
    let waiting = visit(&id, "Stranger", "waiting").await;
    let elsewhere = visit(&mint("Other room").await, "Stranger", "joined").await;
    let (status, _) = rpc(
        &page,
        "send_meeting_chat",
        &format!(r#"{{"id":"{id}","ticket":"{inside}","text":"Secret"}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    for ticket in [waiting.as_str(), elsewhere.as_str(), "no-such-ticket"] {
        let (status, body) = rpc(
            &page,
            "meeting_chat_history",
            &format!(r#"{{"id":"{id}","ticket":"{ticket}"}}"#),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
        assert!(!body.contains("Secret"), "{body}");
        let (status, _) = rpc(
            &page,
            "send_meeting_chat",
            &format!(r#"{{"id":"{id}","ticket":"{ticket}","text":"Forged"}}"#),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn meeting_chat_refuses_bad_messages_and_ends_with_the_link() {
    let id = mint("Short chat").await;
    let page = format!("/meet/j/{id}");
    let ticket = visit(&id, "Sammy", "joined").await;
    let send = |text: String| format!(r#"{{"id":"{id}","ticket":"{ticket}","text":"{text}"}}"#);

    let (status, _) = rpc(&page, "send_meeting_chat", &send("   ".into())).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = rpc(&page, "send_meeting_chat", &send("a".repeat(501))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = rpc(&page, "send_meeting_chat", r#"{"id":1}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = rpc(&page, "send_meeting_chat", &send("Goodbye".into())).await;
    assert_eq!(status, StatusCode::OK);
    crate::meetings::revoke(&id).await.unwrap();
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
    let left = crate::models::meeting_message::Entity::find()
        .filter(crate::models::meeting_message::Column::MeetingId.eq(&id))
        .count(crate::db::db())
        .await
        .unwrap();
    assert_eq!(left, 0, "a revoked link keeps no conversation");
}

#[tokio::test]
async fn pre_join_audio_selects_name_the_default_device() {
    let id = mint("Audio device names").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        eprintln!("Skipping audio device test: Chrome or Edge is not installed.");
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    let shown = |label: &str, text: &str| {
        format!(
            "(() => {{ const select = document.querySelector(\"select[aria-label='{label}']\"); return select.selectedIndex >= 0 && select.selectedOptions[0].textContent.trim() === '{text}'; }})()"
        )
    };

    // Before the browser lists anything, the fallback option is still chosen.
    browser
        .wait_for(&shown("Microphone device", "Default microphone"))
        .await
        .unwrap();
    browser
        .wait_for(&shown("Speakers", "Default speakers"))
        .await
        .unwrap();

    // Chromium's list, once the page may read labels.
    let _: bool = browser
        .eval(
            r#"(() => {
        const device = (kind, deviceId, label) => ({ kind, deviceId, label });
        navigator.mediaDevices.enumerateDevices = async () => [
            device("audioinput", "default", "Default - Microphone Array"),
            device("audioinput", "communications", "Communications - Microphone Array"),
            device("audioinput", "array", "Microphone Array"),
            device("audioinput", "headset", "Headset Microphone"),
            device("audiooutput", "default", "Default - SoundWire Speakers"),
            device("audiooutput", "soundwire", "SoundWire Speakers"),
        ];
        navigator.mediaDevices.dispatchEvent(new Event("devicechange"));
        return true;
    })()"#,
        )
        .await
        .unwrap();
    browser
        .wait_for(&shown("Microphone device", "Microphone Array"))
        .await
        .unwrap();
    browser
        .wait_for(&shown("Speakers", "SoundWire Speakers"))
        .await
        .unwrap();
    let options: usize = browser
        .eval("document.querySelector(\"select[aria-label='Microphone device']\").options.length")
        .await
        .unwrap();
    assert_eq!(options, 3, "the default device is listed once");

    // A device that was chosen and then unplugged falls back to the default.
    let _: bool = browser
        .eval(
            r#"(() => {
        const select = document.querySelector("select[aria-label='Microphone device']");
        select.value = "headset";
        select.dispatchEvent(new Event("change", { bubbles: true }));
        return true;
    })()"#,
        )
        .await
        .unwrap();
    browser
        .wait_for(&shown("Microphone device", "Headset Microphone"))
        .await
        .unwrap();
    let _: bool = browser.eval(r#"(() => {
        const list = navigator.mediaDevices.enumerateDevices;
        navigator.mediaDevices.enumerateDevices = async () => (await list()).filter((device) => device.deviceId !== "headset");
        navigator.mediaDevices.dispatchEvent(new Event("devicechange"));
        return true;
    })()"#).await.unwrap();
    browser
        .wait_for(&shown("Microphone device", "Microphone Array"))
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn meeting_link_actions_stand_out_from_their_card_in_both_appearances() {
    let id = mint("Visible actions").await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
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
    let Some(mut browser) = launch_browser().await else {
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
/// `DILNAKA_MEETINGS_KEY`, so this is the state it sees.
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
    let Some(mut browser) = launch_browser().await else {
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
    let Some(mut browser) = launch_browser().await else {
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
            const input=document.querySelector('input[type=file][aria-label="Upload background image"]'); input.files=transfer.files;
            input.dispatchEvent(new Event('change',{bubbles:true}));
            return true;
        })()
    "##).await.unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"My background.png\"]')?.getAttribute('aria-pressed') === 'true'").await.unwrap();
    let _: bool = browser.eval(r##"
        (() => {
            const transfer=new DataTransfer(); transfer.items.add(new File(['not an image'],'bad.png',{type:'image/png'}));
            const input=document.querySelector('input[type=file][aria-label="Upload background image"]'); input.files=transfer.files;
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
/// "No provider" is guaranteed rather than assumed: `dilnaka_meet::settings`
/// returns `None` under `cfg(test)`, so this holds on a machine whose `.env`
/// has a real Meet key in it, and the suite never calls Dilnaka.
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

/// A visit by a signed-in account, in the call.
async fn account_visit(id: &str, user: &str, name: &str) -> String {
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    let ticket = format!("ticket-{}", crate::meetings::mint_id());
    crate::models::meeting_visit::ActiveModel {
        id: Set(ticket.clone()),
        meeting_id: Set(id.into()),
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
async fn files_shared_in_meeting_chat_reach_every_participant() {
    use rahti::serde_json::{Value, json};
    let id = mint("Shared files").await;
    let page = format!("/meet/j/{id}");
    let mine = account_visit(&id, "legacy-test-user", "Jefferson").await;
    let guest = visit(&id, "Guest", "joined").await;

    // Uploading from the chat puts the file in the sender's My files.
    let mut sender = super::auth::Client::new().await;
    sender.with_cookie(test_cookie());
    sender.request(&page, None).await;
    let (status, body) = sender
        .upload(
            &page,
            "upload_meeting_chat_file",
            json!({"id": id, "ticket": mine}),
            ("file", "application/pdf", b"%PDF-1.4 agenda"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let stored: Value = rahti::serde_json::from_str(&body).unwrap();
    let file = stored["id"].as_str().unwrap().to_string();

    // My files is offered for choosing.
    let (status, body) = rpc(
        &page,
        "meeting_chat_my_files",
        &json!({"id": id, "ticket": mine, "folder": null}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(&file), "{body}");

    // A message may be the file alone.
    let (status, body) = rpc(
        &page,
        "send_meeting_chat",
        &json!({"id": id, "ticket": mine, "text": "", "file": file}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let sent: Value = rahti::serde_json::from_str(&body).unwrap();
    assert_eq!(sent["file"]["content_type"], "application/pdf");
    assert_eq!(sent["file"]["bytes"], 15);
    let message = sent["id"].as_str().unwrap().to_string();

    // A guest in the call sees it and gets a link to it.
    let (status, body) = rpc(
        &page,
        "meeting_chat_history",
        &json!({"id": id, "ticket": guest}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""bytes":15"#), "{body}");
    let (status, body) = rpc(
        &page,
        "meeting_chat_file_url",
        &json!({"id": id, "ticket": guest, "message": message}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("https://storage.test/"), "{body}");

    // Removing the file from My files leaves the chat's copy in place.
    assert!(
        crate::meetings::chat::shares_object(stored_object(&file).await.as_str())
            .await
            .unwrap()
    );
    let (status, _) = rpc(
        &page,
        "meeting_chat_file_url",
        &json!({"id": id, "ticket": guest, "message": message}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// The storage object behind one of My files.
async fn stored_object(file: &str) -> String {
    use sea_orm::EntityTrait;
    crate::models::chat_file::Entity::find_by_id(file)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap()
        .file_id
}

#[tokio::test]
async fn meeting_chat_files_are_refused_to_guests_strangers_and_other_peoples_files() {
    use rahti::serde_json::json;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    let id = mint("Guarded files").await;
    let page = format!("/meet/j/{id}");
    let mine = account_visit(&id, "legacy-test-user", "Jefferson").await;
    let guest = visit(&id, "Guest", "joined").await;
    let waiting = visit(&id, "Stranger", "waiting").await;

    // Guests have no My files to share from.
    for name in ["meeting_chat_my_files", "send_meeting_chat"] {
        let (status, body) = rpc(
            &page,
            name,
            &json!({"id": id, "ticket": guest, "text": "", "file": "anything", "folder": null})
                .to_string(),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{name}: {body}");
    }

    // Somebody else's file cannot be shared.
    crate::models::chat_file::ActiveModel {
        id: Set(format!("foreign-{id}")),
        owner_id: Set("other-user".into()),
        name: Set("theirs.txt".into()),
        content_type: Set("text/plain".into()),
        bytes: Set(4),
        file_id: Set("object-theirs".into()),
        file_key: Set("chat/other-user/theirs.txt".into()),
        created_at: Set(rahti::time::Timestamp::now().as_second()),
        folder_id: Set(None),
    }
    .insert(crate::db::db())
    .await
    .unwrap();
    let (status, _) = rpc(
        &page,
        "send_meeting_chat",
        &json!({"id": id, "ticket": mine, "text": "", "file": format!("foreign-{id}")}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Neither text nor a file is not a message.
    let (status, _) = rpc(
        &page,
        "send_meeting_chat",
        &json!({"id": id, "ticket": mine, "text": "  ", "file": null}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Someone waiting in the lobby gets no link, and a text message has no file.
    let (_, body) = rpc(
        &page,
        "send_meeting_chat",
        &json!({"id": id, "ticket": mine, "text": "just words"}).to_string(),
    )
    .await;
    let sent: rahti::serde_json::Value = rahti::serde_json::from_str(&body).unwrap();
    let message = sent["id"].as_str().unwrap();
    let (status, _) = rpc(
        &page,
        "meeting_chat_file_url",
        &json!({"id": id, "ticket": waiting, "message": message}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = rpc(
        &page,
        "meeting_chat_file_url",
        &json!({"id": id, "ticket": mine, "message": message}).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
