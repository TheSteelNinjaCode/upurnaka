use super::*;

#[tokio::test]
async fn settings_renders_storage_options_and_download_guidance() {
    let (status, body) = get("/settings").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for text in [
        "Settings categories",
        "File storage",
        "Computer",
        "Dilnaka storage",
        "browser's download folder",
        "Audio settings",
        "Video settings",
        "Mute all notifications (except for calls and meetings)",
        "Test notification",
        "Position on the screen",
    ] {
        assert!(body.contains(text), "missing {text}");
    }
}

#[tokio::test]
async fn settings_requires_a_session() {
    setup().await;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let response = crate::routes::router()
        .oneshot(
            Request::builder()
                .uri("/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert!(
        response.headers()["location"]
            .to_str()
            .unwrap()
            .starts_with("/signin")
    );
}

#[tokio::test]
async fn settings_preferences_persist_apply_and_reset_in_the_browser() {
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
        .click("button[aria-label='Settings and more']")
        .await
        .unwrap();
    browser.click("a[href='/settings']").await.unwrap();
    browser
        .wait_for("location.pathname === '/settings' && !!document.querySelector('.settings-nav')")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Find in Settings']")
        .await
        .unwrap();
    browser.type_text("storage").await.unwrap();
    browser
        .wait_for("document.querySelector('.settings-main h2').textContent === 'Files and links'")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    browser
        .click(".storage-choice:first-child strong")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).storage === 'computer'").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=computer]').checked")
        .await
        .unwrap();
    browser
        .click(".storage-choice:last-child strong")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).storage === 'dilnaka'").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    browser.click(".settings-nav:nth-child(2)").await.unwrap();
    browser
        .click("input[name='theme'][value='light']")
        .await
        .unwrap();
    browser
        .wait_for("document.documentElement.dataset.appearance === 'light'")
        .await
        .unwrap();
    browser.click(".settings-nav:nth-child(3)").await.unwrap();
    browser
        .click("input[aria-label='Press Enter to send']")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).enterToSend === false").await.unwrap();
    browser
        .click("input[aria-label='Check spelling while typing']")
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .wait_for("document.documentElement.dataset.appearance === 'light'")
        .await
        .unwrap();
    browser.click("textarea[name='text']").await.unwrap();
    browser
        .wait_for("document.querySelector('textarea[name=text]').spellcheck === false")
        .await
        .unwrap();
    browser
        .type_text("A draft that should not send")
        .await
        .unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("document.querySelector('textarea[name=text]').value.includes('A draft that should not send')").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.click(".settings-intro button").await.unwrap();
    browser.click(".settings-primary").await.unwrap();
    browser.wait_for("document.documentElement.dataset.appearance === 'dark' && localStorage.getItem('upurnaka.settings.v1.legacy-test-user') === null").await.unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn settings_recovers_invalid_values_and_reports_failed_saves() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    // Older settings saved Computer implicitly alongside unrelated changes.
    let _: bool = browser.eval("localStorage.setItem('upurnaka.settings.v1.legacy-test-user', JSON.stringify({theme:'invalid', textSize:'huge', storage:'computer', spellcheck:'false'})); true").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser.wait_for("document.documentElement.dataset.appearance === 'dark' && document.documentElement.dataset.textSize === 'standard'").await.unwrap();
    browser.click(".settings-nav:last-child").await.unwrap();
    browser
        .wait_for("document.querySelector('input[name=storage][value=dilnaka]').checked")
        .await
        .unwrap();
    let _: bool = browser.eval("Storage.prototype.setItem = function() { throw new DOMException('Storage unavailable', 'QuotaExceededError'); }; true").await.unwrap();
    browser
        .click("input[name='storage'][value='computer']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('[role=status]').textContent.includes('Could not save') && document.querySelector('input[name=storage][value=dilnaka]').checked").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn devices_save_choices_test_privately_and_release_media() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    let _: bool = browser.eval(r##"
        window.deviceRequests = []; window.deviceTracks = []; window.deviceDenied = false;
        navigator.mediaDevices.enumerateDevices = async () => [
            {kind:'audioinput',deviceId:'mic-test',label:'Test microphone'},
            {kind:'audiooutput',deviceId:'speaker-test',label:'Test speaker'},
            {kind:'videoinput',deviceId:'camera-test',label:'Test camera'}
        ];
        window.deviceAudio = new AudioContext();
        navigator.mediaDevices.getUserMedia = async constraints => {
            window.deviceRequests.push(constraints);
            if (window.deviceDenied) throw new DOMException('Denied','NotAllowedError');
            let stream;
            if (constraints.video) {
                const canvas = document.createElement('canvas'); canvas.width = 320; canvas.height = 180;
                canvas.getContext('2d').fillRect(0,0,320,180);
                stream = canvas.captureStream(15);
            } else { stream = window.deviceAudio.createMediaStreamDestination().stream; }
            window.deviceTracks.push(...stream.getTracks());
            return stream;
        }; true
    "##).await.unwrap();
    browser
        .click(".settings-nav[data-category=devices]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.settings-main h2').textContent === 'Devices' && document.querySelector('select[aria-label=\"Camera device\"]').options.length === 2").await.unwrap();
    let requests: usize = browser.eval("window.deviceRequests.length").await.unwrap();
    assert_eq!(
        requests, 0,
        "opening Devices must not activate a camera or microphone"
    );
    let _: bool = browser.eval("const picker=document.querySelector('select[aria-label=\"Camera device\"]'); picker.value='camera-test'; picker.dispatchEvent(new Event('change',{bubbles:true})); true").await.unwrap();
    browser
        .click("input[aria-label='Mirror my video']")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Adjust brightness']")
        .await
        .unwrap();
    browser
        .click(".device-settings section:nth-of-type(2) button")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.device-camera-preview video').srcObject?.getVideoTracks()[0]?.readyState === 'live' && window.deviceRequests.some(request => request.video?.deviceId?.exact === 'camera-test')").await.unwrap();
    browser.wait_for("getComputedStyle(document.querySelector('.device-camera-preview video')).filter.includes('1.2')").await.unwrap();
    browser
        .click("button[aria-label='Test microphone']")
        .await
        .unwrap();
    browser.wait_for("window.deviceRequests.some(request => request.audio?.autoGainControl === true && request.audio?.noiseSuppression === true)").await.unwrap();
    browser.click(".settings-nav:first-child").await.unwrap();
    browser.wait_for("window.deviceTracks.length >= 2 && window.deviceTracks.every(track => track.readyState === 'ended')").await.unwrap();
    browser
        .click(".settings-nav[data-category=devices]")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.deviceDenied = true; true")
        .await
        .unwrap();
    browser
        .click(".device-settings section:nth-of-type(2) button")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.device-settings [role=alert]').textContent.includes('Access was denied')").await.unwrap();
    let _: bool = browser
        .eval("window.deviceAudio.close(); true")
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=devices]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('input[aria-label=\"Mirror my video\"]').checked && document.querySelector('input[aria-label=\"Adjust brightness\"]').checked").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn saved_devices_and_audio_processing_apply_to_new_meetings() {
    let (status, body) = rpc(
        "/meet",
        "create_meeting_link",
        r#"{"title":"Saved devices"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = rahti::serde_json::from_str::<rahti::serde_json::Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let router = crate::routes::router()
        .route("/js/vendor/amazon-chime-sdk.js", axum::routing::get(|| async {
            ([("content-type", "text/javascript")], include_str!("fixtures/chime-room.js"))
        }))
        .layer(axum::middleware::from_fn(|request: axum::extract::Request, next: axum::middleware::Next| async move {
            if request.headers().get("x-pp-function").and_then(|value| value.to_str().ok()) == Some("resume_meeting_audio") {
                return axum::response::Response::builder().header("content-type", "application/json").body(axum::body::Body::from("true")).unwrap();
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
    let _: bool = browser.eval("localStorage.setItem('upurnaka.settings.v1.legacy-test-user', JSON.stringify({cameraId:'second-camera',microphoneId:'mic-test',speakerId:'speaker-test',mirrorVideo:false,videoBrightness:true,softFocus:true,autoGainControl:false,noiseSuppression:false,signLanguageMode:true,mutedNotification:true,lockMediaOnScreenLock:true})); true").await.unwrap();
    browser
        .goto(&format!("http://{address}/meet/j/{id}"))
        .await
        .unwrap();
    let _: bool = browser.eval(r##"
        window.fixtureMicrophones = [{deviceId:'mic-test'}];
        window.IdleDetector = class extends EventTarget {
            constructor() { super(); window.meetingIdleDetector = this; this.screenState = 'unlocked'; }
            async start(options) { options.signal.addEventListener('abort', () => {window.idleProtectionAborted=true;}); }
        };
        const originalPermissionQuery = navigator.permissions.query.bind(navigator.permissions);
        navigator.permissions.query = descriptor => descriptor.name === 'idle-detection' ? Promise.resolve({state:'granted'}) : originalPermissionQuery(descriptor);
        window.meetingTestAudio = new AudioContext();
        navigator.mediaDevices.getUserMedia = async constraints => {
            window.meetingAudioConstraints = constraints;
            window.meetingTestStream = window.meetingTestAudio.createMediaStreamDestination().stream;
            return window.meetingTestStream;
        }; true
    "##).await.unwrap();
    browser.click("input[aria-label='Camera']").await.unwrap();
    browser
        .wait_for("window.testDeviceController?.lastInput === 'second-camera'")
        .await
        .unwrap();
    browser.wait_for("getComputedStyle(document.querySelector('[aria-label=\"Camera preview\"] video')).filter.includes('1.2')").await.unwrap();
    // One step: a re-render between enabling and clicking disables it again.
    let _: bool = browser
        .eval("(() => { const join = document.querySelector('.meeting-primary'); join.disabled = false; join.click(); return true; })()")
        .await
        .unwrap();
    browser
        .wait_for(
            "!!window.roomFixture?.audioInput && window.roomFixture.audioOutput === 'speaker-test'",
        )
        .await
        .unwrap();
    let correct_audio: bool = browser.eval("window.meetingAudioConstraints.audio.deviceId.exact === 'mic-test' && window.meetingAudioConstraints.audio.autoGainControl === false && window.meetingAudioConstraints.audio.noiseSuppression === false").await.unwrap();
    assert!(
        correct_audio,
        "new meetings must use the saved audio constraints"
    );
    browser
        .wait_for("!document.querySelector('.room-sign-language-notice').hidden")
        .await
        .unwrap();
    let _: bool = browser.eval("window.roomFixture.presence('signer',true,'Interpreter#fixture'); window.roomFixture.observer.videoTileDidUpdate({tileId:10,boundAttendeeId:'signer',boundExternalUserId:'Interpreter#fixture',paused:false}); true").await.unwrap();
    browser
        .click(".room-person-video[data-attendee-id=signer] button")
        .await
        .unwrap();
    browser
        .click("button[data-action='sign-language']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.room-person-video[data-attendee-id=signer]').getAttribute('data-featured') === 'true'").await.unwrap();
    browser
        .click("button[aria-label='Toggle meeting microphone']")
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.roomFixture.volume(0.7, true); true")
        .await
        .unwrap();
    browser
        .wait_for("!document.querySelector('.room-muted-notice').hidden")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Dismiss muted notification']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.room-muted-notice').hidden")
        .await
        .unwrap();
    browser
        .wait_for("!!window.meetingIdleDetector")
        .await
        .unwrap();
    let _: bool = browser.eval("window.meetingIdleDetector.screenState='locked'; window.meetingIdleDetector.dispatchEvent(new Event('change')); true").await.unwrap();
    browser.wait_for("window.meetingTestStream.getTracks().every(track => track.readyState === 'ended') && window.roomFixture.audioInput.getTracks().every(track => track.readyState === 'ended') && window.testDeviceController.stream.getTracks().every(track => track.readyState === 'ended') && !document.querySelector('.meeting-room').hidden").await.unwrap();
    let _: bool = browser.eval("window.meetingIdleDetector.screenState='unlocked'; window.meetingIdleDetector.dispatchEvent(new Event('change')); true").await.unwrap();
    let stays_muted: bool = browser.eval("window.roomFixture.muted && document.querySelector('button[aria-label=\"Toggle meeting microphone\"]').getAttribute('aria-pressed') === 'false'").await.unwrap();
    assert!(
        stays_muted,
        "unlocking must not restart capture automatically"
    );
    browser
        .click("button[aria-label='Toggle meeting microphone']")
        .await
        .unwrap();
    browser.wait_for("!window.roomFixture.muted && window.meetingTestStream.getAudioTracks()[0].readyState === 'live'").await.unwrap();
    browser
        .click("button[aria-label='Leave meeting']")
        .await
        .unwrap();
    browser
        .wait_for(
            "window.meetingTestStream.getTracks().every(track => track.readyState === 'ended')",
        )
        .await
        .unwrap();
    let _: bool = browser
        .eval("window.meetingTestAudio.close(); true")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn accessibility_matches_teams_and_saves_switches() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=accessibility]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.settings-main h2').textContent === 'Accessibility' && Array.from(document.querySelectorAll('.accessibility-setting')).filter(card => !card.hidden).length === 3").await.unwrap();
    let defaults: bool = browser.eval("!document.querySelector('input[aria-label=\"Sign language mode\"]').checked && document.querySelector('input[aria-label=\"Muted notification\"]').checked && !document.querySelector('input[aria-label=\"Turn on single pointer controls\"]').checked").await.unwrap();
    assert!(defaults);
    for label in [
        "Sign language mode",
        "Muted notification",
        "Turn on single pointer controls",
    ] {
        browser
            .click(&format!("input[aria-label='{label}']"))
            .await
            .unwrap();
    }
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).signLanguageMode === true && JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).mutedNotification === false && JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).singlePointerControls === true").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=accessibility]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('input[aria-label=\"Sign language mode\"]').checked && !document.querySelector('input[aria-label=\"Muted notification\"]').checked && document.querySelector('input[aria-label=\"Turn on single pointer controls\"]').checked").await.unwrap();
    browser.click(".settings-intro button").await.unwrap();
    browser.click(".settings-primary").await.unwrap();
    browser.wait_for("!document.querySelector('input[aria-label=\"Sign language mode\"]').checked && document.querySelector('input[aria-label=\"Muted notification\"]').checked && !document.querySelector('input[aria-label=\"Turn on single pointer controls\"]').checked").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn single_pointer_controls_reorder_chats_without_dragging() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    let _: bool = browser.eval("localStorage.setItem('upurnaka.settings.v1.legacy-test-user', JSON.stringify({singlePointerControls:true})); true").await.unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    for (index, name) in ["Alex", "Blair"].into_iter().enumerate() {
        browser
            .click("button[aria-label='New chat']")
            .await
            .unwrap();
        browser.type_text(name).await.unwrap();
        browser.press("Enter").await.unwrap();
        browser
            .click("textarea[aria-label='Type a message']")
            .await
            .unwrap();
        browser.type_text("Test thread").await.unwrap();
        browser.click("button[aria-label='Send']").await.unwrap();
        browser
            .wait_for(&format!(
                "document.querySelectorAll('.chat-thread-row').length === {}",
                index + 1
            ))
            .await
            .unwrap();
    }
    browser
        .click("button[aria-label='Move Blair up']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.chat-thread-row').textContent.includes('Blair') && document.querySelector('button[aria-label=\"Move Blair up\"]').disabled").await.unwrap();
    browser
        .click("button[aria-label='Move Blair down']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.chat-thread-row').textContent.includes('Alex')")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn privacy_controls_save_handle_permissions_and_open_profile() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=privacy]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.settings-main h2').textContent === 'Privacy' && document.querySelectorAll('.privacy-setting').length === 8").await.unwrap();
    let unavailable: bool = browser.eval("document.querySelector('select[aria-label=\"Who can call you\"]').disabled && document.querySelector('select[aria-label=\"Who can add you to groups\"]').disabled && document.querySelector('input[aria-label=\"Extended diagnostics data\"]').disabled && !document.querySelector('input[aria-label=\"Experiences that analyze your content\"]').checked").await.unwrap();
    assert!(
        unavailable,
        "unsupported privacy promises must not be presented as working controls"
    );
    browser
        .click(".privacy-settings [data-slot=dialog-trigger]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.privacy-contact-dialog').open && document.querySelector('.privacy-contact-dialog').textContent.includes('fixture@example.com')").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('.privacy-contact-dialog').open")
        .await
        .unwrap();
    let _: bool = browser.eval("window.privacyPermission='denied'; window.IdleDetector=class {static async requestPermission(){return window.privacyPermission;}}; true").await.unwrap();
    browser
        .click("input[aria-label='Optional connected experiences']")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Turn off my camera and mic when my screen locks']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.privacy-settings [role=alert]').textContent.includes('was not enabled') && !document.querySelector('input[aria-label=\"Turn off my camera and mic when my screen locks\"]').checked").await.unwrap();
    let _: bool = browser
        .eval("window.privacyPermission='granted'; true")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Turn off my camera and mic when my screen locks']")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).lockMediaOnScreenLock === true && JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).connectedExperiences === false").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=privacy]")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('input[aria-label=\"Optional connected experiences\"]').checked").await.unwrap();
    browser
        .click(".privacy-setting:nth-of-type(6) > a")
        .await
        .unwrap();
    browser.wait_for("location.pathname === '/people' && document.querySelector('.people-profile-dialog').open && document.querySelector('input[name=name]').value === 'Jefferson Abraham'").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn optional_connected_experiences_off_blocks_giphy_requests() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    let _: bool = browser.eval("localStorage.setItem('upurnaka.settings.v1.legacy-test-user', JSON.stringify({connectedExperiences:false})); true").await.unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    let _: bool = browser.eval("window.optionalGifRequests=0; const originalFetch=window.fetch.bind(window); window.fetch=(input,options)=>{if(String(input).includes('giphy.com')){window.optionalGifRequests++; return Promise.reject(new Error('Unexpected GIF request'));} return originalFetch(input,options);}; true").await.unwrap();
    browser.click("button[aria-label=Emoji]").await.unwrap();
    browser.wait_for("document.querySelector('.chat-emoji-content').textContent.includes('Optional connected experiences are off')").await.unwrap();
    let blocked: bool = browser.eval("window.optionalGifRequests === 0 && !Array.from(document.images).some(image => image.src.includes('giphy.com'))").await.unwrap();
    assert!(
        blocked,
        "GIF services must stay disconnected when the privacy setting is off"
    );
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn chat_settings_persist_and_control_the_chat_list_and_density() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=chats]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.settings-main h2').textContent === 'Chats' && document.querySelector('input[name=\"chat-density\"][value=comfy]').checked").await.unwrap();
    let receipts_unavailable: bool = browser.eval("document.querySelector('input[aria-label=\"Read receipts\"]').disabled && !document.querySelector('input[aria-label=\"Read receipts\"]').checked").await.unwrap();
    assert!(
        receipts_unavailable,
        "read receipts require recipient delivery and must not be simulated"
    );
    for label in [
        "Show message previews for chats",
        "Show time stamps",
        "Chat list filters",
    ] {
        browser
            .click(&format!("input[aria-label='{label}']"))
            .await
            .unwrap();
    }
    browser
        .click("input[name='chat-density'][value=compact]")
        .await
        .unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).chatDensity === 'compact'").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=chats]")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('input[name=\"chat-density\"][value=compact]').checked && !document.querySelector('input[aria-label=\"Show message previews for chats\"]').checked && !document.querySelector('input[aria-label=\"Show time stamps\"]').checked && !document.querySelector('input[aria-label=\"Chat list filters\"]').checked").await.unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .click("textarea[aria-label='Type a message']")
        .await
        .unwrap();
    browser.type_text("Hidden preview text").await.unwrap();
    browser.click("button[aria-label=Send]").await.unwrap();
    browser.wait_for("document.querySelector('.chat-message-row') && document.querySelector('.chat-message-list').dataset.density === 'compact'").await.unwrap();
    let hidden: bool = browser.eval("document.querySelector('.chat-list-filters').hidden && Array.from(document.querySelectorAll('.chat-list-preview')).every(element => element.hidden) && Array.from(document.querySelectorAll('.chat-list-time')).every(element => element.hidden)").await.unwrap();
    assert!(
        hidden,
        "chat list preferences must affect the actual chat UI"
    );
    let compact_padding: String = browser
        .eval("getComputedStyle(document.querySelector('.chat-message-row')).paddingTop")
        .await
        .unwrap();
    assert_eq!(compact_padding, "2px");
    browser
        .goto(&format!("http://{address}/settings"))
        .await
        .unwrap();
    browser
        .click(".settings-nav[data-category=chats]")
        .await
        .unwrap();
    for label in [
        "Show message previews for chats",
        "Show time stamps",
        "Chat list filters",
    ] {
        browser
            .click(&format!("input[aria-label='{label}']"))
            .await
            .unwrap();
    }
    browser
        .click("input[name='chat-density'][value=comfy]")
        .await
        .unwrap();
    browser
        .goto(&format!("http://{address}/chat"))
        .await
        .unwrap();
    browser
        .click("textarea[aria-label='Type a message']")
        .await
        .unwrap();
    browser.type_text("Visible preview text").await.unwrap();
    browser.click("button[aria-label=Send]").await.unwrap();
    browser.wait_for("document.querySelector('.chat-message-row') && document.querySelector('.chat-self-entry .chat-list-preview').textContent === 'Visible preview text'").await.unwrap();
    let visible: bool = browser.eval("!document.querySelector('.chat-list-filters').hidden && !document.querySelector('.chat-self-entry .chat-list-preview').hidden && !document.querySelector('.chat-self-entry .chat-list-time').hidden && document.querySelector('.chat-self-entry .chat-list-time').textContent.trim().length > 0 && document.querySelector('.chat-message-list').dataset.density === 'comfy'").await.unwrap();
    assert!(visible);
    let comfy_padding: String = browser
        .eval("getComputedStyle(document.querySelector('.chat-message-row')).paddingTop")
        .await
        .unwrap();
    assert_eq!(comfy_padding, "8px");
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn notification_settings_save_and_drive_the_test_notification() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = launch_browser().await else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/activity"))
        .await
        .unwrap();
    browser
        .click("a[aria-label='Notification settings']")
        .await
        .unwrap();
    browser.wait_for("location.search === '?category=notifications' && document.querySelector('.settings-main h2').textContent === 'Notifications and activity' && document.querySelector('input[name=\"notification-position\"][value=bottom-right]').checked").await.unwrap();
    let unavailable: bool = browser.eval("['Follow device Do not disturb', 'Meeting start notifications', 'Show meeting join bar', 'Meet now start notifications'].every(label => { const input = document.querySelector('input[aria-label=\"' + label + '\"]'); return input.disabled && !input.checked; }) && ['Missed activity emails', 'Chat message notifications', 'Mention notifications', 'Reaction notifications', 'Meeting chat notifications'].every(label => document.querySelector('select[aria-label=\"' + label + '\"]').disabled)").await.unwrap();
    assert!(
        unavailable,
        "notifications Upurnaka cannot deliver must not be presented as working controls"
    );
    // Record synthesized tones instead of playing them.
    let _: bool = browser.eval("window.playedTones = []; window.AudioContext = class { constructor(){ this.currentTime = 0; this.state = 'running'; this.destination = {}; } createGain(){ return {gain:{setValueAtTime(){}, exponentialRampToValueAtTime(){}}, connect(node){ return node; }}; } createOscillator(){ const tone = {type:'', frequency:{value:0}, connect(node){ return node; }, start(){ window.playedTones.push(tone.frequency.value); }, stop(){}}; return tone; } }; true").await.unwrap();
    let _: bool = browser.eval("const select = document.querySelector('select[aria-label=\"Notification sound\"]'); select.value = 'ping'; select.dispatchEvent(new Event('change',{bubbles:true})); true").await.unwrap();
    browser.wait_for("JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')).notifySoundName === 'ping'").await.unwrap();
    let _: bool = browser.eval("const dismiss = document.querySelector('select[aria-label=\"Dismiss notifications after\"]'); dismiss.value = '10'; dismiss.dispatchEvent(new Event('change',{bubbles:true})); true").await.unwrap();
    browser
        .click("button[aria-label='Play notification sound']")
        .await
        .unwrap();
    browser
        .wait_for("window.playedTones.join() === '1760'")
        .await
        .unwrap();
    browser
        .click("input[name='notification-position'][value=top-left]")
        .await
        .unwrap();
    browser
        .click("input[name='notification-size'][value=compact]")
        .await
        .unwrap();
    browser
        .click(".notification-display-heading .settings-secondary")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.notification-toast').hidden && document.querySelector('.notification-toast').classList.contains('at-top-left') && document.querySelector('.notification-toast').classList.contains('size-compact') && document.querySelector('.notification-toast').getBoundingClientRect().top < 40 && document.querySelector('.notification-toast').textContent.includes('test notification') && window.playedTones.length === 2").await.unwrap();
    browser
        .click("button[aria-label='Dismiss notification']")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.notification-toast').hidden")
        .await
        .unwrap();
    browser
        .click("input[aria-label='Mute all notifications']")
        .await
        .unwrap();
    browser
        .click(".notification-display-heading .settings-secondary")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.notification-toast').hidden && !document.querySelector('.notification-muted-note').hidden && window.playedTones.length === 2").await.unwrap();
    browser.wait_for("(s => s.notifyMuteAll === true && s.notifySoundName === 'ping' && s.notifyDismissAfter === '10' && s.notifyPosition === 'top-left' && s.notifySize === 'compact')(JSON.parse(localStorage.getItem('upurnaka.settings.v1.legacy-test-user')))").await.unwrap();
    browser
        .goto(&format!("http://{address}/settings?category=notifications"))
        .await
        .unwrap();
    browser.wait_for("document.querySelector('input[aria-label=\"Mute all notifications\"]').checked && document.querySelector('select[aria-label=\"Notification sound\"]').value === 'ping' && document.querySelector('input[name=\"notification-size\"][value=compact]').checked").await.unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
