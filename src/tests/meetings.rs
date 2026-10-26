//! Meeting links, end to end: minted on `/meet`, kept, listed on the next
//! page load, and resolved at `/meet/j/{id}`.

use super::*;

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
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("join_meeting") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from(r#"{"meeting":{"MeetingId":"fixture"},"attendee":{"AttendeeId":"self","ExternalUserId":"Jefferson Abraham#fixture"}}"#)).unwrap();
            }
            next.run(request).await
        }));
    let address = rahti::browser::serve(router).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
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
    browser
        .click("button[aria-label='Toggle meeting camera']")
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
    let stopped: bool = browser.eval("window.roomFixture.stopped && window.roomFixture.controller.stream.getTracks().every(track => track.readyState === 'ended')").await.unwrap();
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
    browser.click("button[aria-label='Camera']").await.unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Camera\"]').getAttribute('aria-checked') === 'true' && !document.querySelector('button[aria-label=\"Camera\"]').disabled && document.querySelector('.meeting-camera-off').hidden").await.unwrap();
    browser
        .click(".meeting-background-option:nth-child(2)")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.meeting-background-option:nth-child(2)').getAttribute('aria-pressed') === 'true' && !document.querySelector('button[aria-label=\"Camera\"]').disabled").await.unwrap();
    browser
        .click(".meeting-background-option:first-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.meeting-background-option:first-child').getAttribute('aria-pressed') === 'true'").await.unwrap();
    browser.click("button[aria-label='Camera']").await.unwrap();
    browser.wait_for("document.querySelector('button[aria-label=\"Camera\"]').getAttribute('aria-checked') === 'false'").await.unwrap();
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
