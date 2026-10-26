//! Where a meeting link lands: the pre-join screen for one join code.
//!
//! `/meet/j/{id}`. This is the half of a meeting link that makes it a link at
//! all — somebody who was sent one opens it here, sets their camera and
//! microphone the way they want them, and joins.
//!
//! "Join now" calls [`join_meeting`], which reaches AWS Chime SDK Meetings
//! through `crate::meetings::join`: a conference is found or created for this
//! link and a seat in it is minted for this person. What comes back is a
//! `MeetingSessionConfiguration`'s two arguments, ready for
//! `amazon-chime-sdk-js` to open audio and video with.
//!
//! The media client itself is not loaded here yet — see the note in the
//! connected panel below. Everything up to it is live, so a join either
//! produces real Chime credentials or a real failure.
//!
//! With no `CHIME_MEDIA_REGION` configured there is no provider, and the
//! button says so instead of calling one.

use axum::extract::Path;
use axum::http::StatusCode;

use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::meetings::{self, JoinInfo, chime};
use crate::rahti::{Error, Html, Json, Metadata, Page, Result, Validator, html, rpc};

pub async fn page(Path(id): Path<String>) -> Result<Page<Html>> {
    // A revoked link and a code nobody ever minted read the same from here,
    // and deliberately: which of the two a stranger found is a fact about
    // this account they have no claim on.
    let Some(link) = meetings::find(&id).await? else {
        return Err(Error::public(
            StatusCode::NOT_FOUND,
            "That meeting link is no longer available. Ask the organizer for a new one.",
        ));
    };

    let title = link.title.clone();
    let code = link.id.clone();

    // Decided on the server, because it is a deployment fact rather than
    // something the browser could discover: with no provider configured the
    // button must not pretend there is a call to join.
    let provider_ready = chime::is_configured();

    let content = html! {
        <main class="meeting-join" onkeydown={handleBackgroundKey(event)}>
            <div class={backgroundsOpen ? "meeting-body with-backgrounds" : "meeting-body"}>
                <div class="meeting-account">
                    <Avatar initials="JA" size="lg" presence="none" />
                    <div class="meeting-account-name">
                        <strong>{displayName || "Your name"}</strong>
                        <span>"Joining as a guest"</span>
                    </div>
                    <button
                        type="button"
                        class="meeting-text-button"
                        onclick={setEditingName(!editingName)}
                    >
                        {editingName ? "Done" : "Change"}
                    </button>
                    <label class="meeting-name-editor" hidden={!editingName}>
                        "Your name"
                        <input
                            name="display_name"
                            aria-label="Your name"
                            autocomplete="name"
                            maxlength="64"
                            value={displayName}
                            oninput={setDisplayName(target.value)}
                        />
                    </label>
                </div>
                <div class="meeting-heading">
                    <span class="meeting-logo">
                        <Icon name="teams" class="h-9 w-9" />
                    </span>
                    <h1>@{&title}</h1>
                </div>
                <div class="meeting-setup">
                    <section class="meeting-camera" aria-label="Camera preview">
                        <div class="meeting-preview">
                            <video
                                pp-ref={localVideo}
                                class="meeting-local-video"
                                onloadedmetadata={setCameraLive(true)}
                                onemptied={setCameraLive(false)}
                                muted
                                playsinline
                            >
                            </video>
                            <div class="meeting-camera-off" hidden={cameraLive}>
                                <Icon name="video-off" class="h-8 w-8" />
                                <strong>
                                    {camera ? "Starting your camera…" : "Your camera is turned off"}
                                </strong>
                            </div>
                        </div>
                        <div class="meeting-camera-toolbar">
                            <Icon name="video-off" class="h-6 w-6" />
                            <button
                                type="button"
                                role="switch"
                                aria-label="Camera"
                                aria-checked={camera}
                                disabled={applyingBackground || joining}
                                class={camera ? "meeting-toggle is-on" : "meeting-toggle"}
                                onclick={toggleCamera()}
                            >
                                <span></span>
                            </button>
                            <span class="meeting-toolbar-divider"></span>
                            <button
                                type="button"
                                class="meeting-background-button"
                                aria-expanded={backgroundsOpen}
                                onclick={setBackgroundsOpen(!backgroundsOpen)}
                            >
                                <Icon name="image" class="h-5 w-5" />
                                "Background filters"
                            </button>
                        </div>
                    </section>
                    <section class="meeting-audio" aria-label="Audio options">
                        <div
                            class={audioMode === "computer" ? "meeting-audio-card selected" : "meeting-audio-card"}
                        >
                            <label class="meeting-audio-choice">
                                <input
                                    type="radio"
                                    name="audio-mode"
                                    value="computer"
                                    checked={audioMode === "computer"}
                                    onchange={setAudioMode("computer")}
                                    disabled={session !== null}
                                />
                                <strong>"Computer audio"</strong>
                            </label>
                            <div class="meeting-audio-devices" hidden={audioMode !== "computer"}>
                                <div class="meeting-device-row">
                                    <Icon name="mic" class="h-6 w-6" />
                                    <select
                                        aria-label="Microphone"
                                        value={microphoneDevice}
                                        onfocus={refreshDevices()}
                                        onchange={setMicrophoneDevice(target.value)}
                                        disabled={session !== null}
                                    >
                                        <option value="">"Default microphone"</option>
                                        <template pp-for="device in audioInputs">
                                            <option key={device.deviceId} value={device.deviceId}>
                                                {device.label || "Microphone"}
                                            </option>
                                        </template>
                                    </select>
                                    <button
                                        type="button"
                                        role="switch"
                                        aria-label="Microphone"
                                        aria-checked={microphone}
                                        class={microphone ? "meeting-toggle is-on" : "meeting-toggle"}
                                        onclick={toggleMicrophone()}
                                    >
                                        <span></span>
                                    </button>
                                </div>
                                <div class="meeting-device-row">
                                    <svg
                                        aria-hidden="true"
                                        class="meeting-speaker-icon"
                                        width="24"
                                        height="24"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="1.7"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    >
                                        <path d="M11 5 6 9H3v6l5 4V5Z" />
                                        <path d="M15 8a6 6 0 0 1 0 8M18 5a10 10 0 0 1 0 14" />
                                    </svg>
                                    <select
                                        aria-label="Speakers"
                                        value={speakerDevice}
                                        onfocus={refreshDevices()}
                                        onchange={setSpeakerDevice(target.value)}
                                        disabled={session !== null}
                                    >
                                        <option value="">"Default speakers"</option>
                                        <template pp-for="device in audioOutputs">
                                            <option key={device.deviceId} value={device.deviceId}>
                                                {device.label || "Speaker"}
                                            </option>
                                        </template>
                                    </select>
                                </div>
                                <button
                                    type="button"
                                    class="meeting-text-button meeting-test-audio"
                                    onclick={testSpeaker()}
                                >
                                    "Test speaker"
                                </button>
                            </div>
                        </div>
                        <label
                            class="meeting-audio-card meeting-audio-choice meeting-unavailable"
                            title="Phone audio is not available"
                        >
                            <input type="radio" name="audio-mode" disabled />
                            "Phone audio"
                        </label>
                        <label class="meeting-audio-card meeting-audio-choice">
                            <input
                                type="radio"
                                name="audio-mode"
                                value="none"
                                checked={audioMode === "none"}
                                onchange={setAudioMode("none")}
                                disabled={session !== null}
                            />
                            "Don't use audio"
                        </label>
                    </section>
                </div>
                <div class="meeting-remote-grid" hidden={remoteCount === 0}>
                    <template pp-for="slot in remoteSlots">
                        <div key={slot.index} class="meeting-remote-tile" hidden={!slot.active}>
                            <video pp-ref={slot.ref} class="h-full w-full object-cover" playsinline>
                            </video>
                        </div>
                    </template>
                </div>
                <audio pp-ref={meetingAudio} class="hidden"></audio>
                <div class="meeting-actions" hidden={session !== null}>
                    <a href="/meet" class="meeting-cancel">"Cancel"</a>
                    <button
                        type="button"
                        class="meeting-primary"
                        disabled={!providerReady || joining || applyingBackground || displayName.trim().length === 0}
                        onclick={joinNow()}
                    >
                        {joining ? "Joining…" : "Join now"}
                    </button>
                </div>
                <div class="meeting-connected" hidden={session === null}>
                    <span>"You're in the meeting"</span>
                    <button type="button" class="meeting-cancel" onclick={leave()}>"Leave"</button>
                </div>
                <p class="meeting-error" role="alert" hidden={!error}>{error}</p>
                <p class="meeting-provider-note" hidden={providerReady}>
                    "This meeting is unavailable right now. Please contact the organizer."
                </p>
                <footer class="meeting-footer">
                    <button type="button" class="meeting-text-button" onclick={share()}>
                        "Copy meeting link"
                    </button>
                    <span>"Meeting ID: "@{&code}</span>
                </footer>
            </div>
            <aside
                class="meeting-background-panel"
                aria-label="Background settings"
                hidden={!backgroundsOpen}
            >
                <header class="meeting-background-header">
                    <h2>"Background settings"</h2>
                    <button
                        type="button"
                        aria-label="Close background settings"
                        onclick={setBackgroundsOpen(false)}
                    >
                        <Icon name="close" class="h-5 w-5" />
                    </button>
                </header>
                <div class="meeting-background-content">
                    <div class="meeting-background-grid">
                        <button
                            type="button"
                            class={backgroundId === "none" ? "meeting-background-option chosen" : "meeting-background-option"}
                            aria-pressed={backgroundId === "none"}
                            disabled={applyingBackground || joining}
                            onclick={selectBackground("none")}
                        >
                            <span class="meeting-background-symbol">"⊘"</span>
                            <span>"None"</span>
                        </button>
                        <button
                            type="button"
                            class={backgroundId === "blur" ? "meeting-background-option chosen" : "meeting-background-option"}
                            aria-pressed={backgroundId === "blur"}
                            disabled={applyingBackground || joining}
                            onclick={selectBackground("blur")}
                        >
                            <span class="meeting-background-symbol">"≋"</span>
                            <span>"Standard blur"</span>
                        </button>
                        <label class="meeting-background-option meeting-background-upload">
                            <Icon name="plus" class="h-6 w-6" />
                            <span>"Add new"</span>
                            <input
                                type="file"
                                accept="image/jpeg,image/png,image/webp"
                                aria-label="Upload background image"
                                disabled={applyingBackground || joining}
                                onchange={uploadBackground(event)}
                            />
                        </label>
                        <template pp-for="background in backgrounds">
                            <button
                                key={background.id}
                                type="button"
                                class={backgroundId === background.id ? "meeting-background-option meeting-background-image chosen" : "meeting-background-option meeting-background-image"}
                                aria-label={background.name}
                                aria-pressed={backgroundId === background.id}
                                disabled={applyingBackground || joining}
                                onclick={selectBackground(background.id)}
                            >
                                <img src={background.url} alt="" />
                                <span>{background.name}</span>
                            </button>
                        </template>
                    </div>
                    <p class="meeting-background-hint" role="status">
                        {applyingBackground ? "Applying background…" : camera ? "Your selected background is applied to your camera and meeting video." : "Turn on your camera to preview your background."}
                    </p>
                    <p class="meeting-background-error" role="alert" hidden={!backgroundError}>
                        {backgroundError}
                    </p>
                    <p class="meeting-background-hint">
                        "Custom images stay in this tab. Camera processing happens on your device."
                    </p>
                </div>
            </aside>
            <div class="meeting-toast" role="status" hidden={!toast}>{toast}</div>
            <style>r##"
                .meeting-join {height:100dvh;overflow:auto;background:#1f1f1f;color:#f5f5f5;font-family:var(--font-ui);}
                .meeting-body {max-width:1460px;padding:0 40px 24px;margin:0 auto;}
                .meeting-account {position:relative;display:flex;align-items:center;gap:16px;width:fit-content;max-width:100%;margin:0 auto;padding:20px 24px;background:#292929;border-radius:0 0 6px 6px;box-shadow:0 3px 6px #0003;}
                .meeting-account-name {display:flex;flex-direction:column;gap:2px;font-size:15px;min-width:160px;}
                .meeting-account-name>span {font-size:13px;color:#c7c7c7;}
                .meeting-text-button {color:#a6a3ff;background:transparent;border:0;cursor:pointer;font:inherit;}
                .meeting-text-button:hover {text-decoration:underline;}
                .meeting-name-editor {position:absolute;top:100%;left:0;right:0;z-index:2;background:#292929;padding:16px;box-shadow:0 6px 12px #0004;display:grid;gap:8px;font-size:13px;}
                .meeting-name-editor input {background:#1f1f1f;border:1px solid #777;border-radius:3px;padding:9px;color:white;}
                .meeting-heading {display:flex;align-items:center;flex-direction:column;gap:24px;text-align:center;margin:70px 0 52px;}
                .meeting-logo {display:grid;place-items:center;width:52px;height:52px;border-radius:12px;background:linear-gradient(145deg,#aaa1ff,#5147b9);color:white;box-shadow:8px 5px 0 -3px #7061d8;}
                .meeting-heading h1 {font-size:26px;font-weight:600;line-height:1.3;overflow-wrap:anywhere;}
                .meeting-setup {display:grid;grid-template-columns:1.13fr 1fr;gap:24px;}
                .meeting-camera {display:flex;flex-direction:column;border:1px solid #525252;border-radius:5px;overflow:hidden;background:#242424;min-width:0;}
                .meeting-preview {position:relative;flex:1;min-height:270px;aspect-ratio:16/9;display:grid;place-items:center;}
                .meeting-local-video {position:absolute;inset:0;width:100%;height:100%;object-fit:cover;transform:scaleX(-1);}
                .meeting-camera-off {position:relative;display:flex;flex-direction:column;align-items:center;gap:18px;color:#b5b5b5;font-size:17px;}
                .meeting-camera-toolbar {display:flex;align-items:center;gap:18px;min-height:64px;padding:12px 24px;border-top:1px solid #525252;color:#c9c9c9;}
                .meeting-toggle {border:1px solid #aaa;border-radius:20px;background:transparent;width:48px;height:25px;padding:3px;flex-shrink:0;cursor:pointer;}
                .meeting-toggle>span {display:block;width:17px;height:17px;border-radius:50%;background:#aaa;transition:transform .15s;}
                .meeting-toggle.is-on {background:#9790fa;border-color:#9790fa;}
                .meeting-toggle.is-on>span {background:#242424;transform:translateX(22px);}
                .meeting-toolbar-divider {height:30px;border-left:1px solid #525252;margin:0 2px 0 8px;}
                .meeting-unavailable {color:#888;cursor:not-allowed;}
                button.meeting-unavailable {display:flex;align-items:center;gap:10px;font-size:14px;}
                .meeting-audio {display:flex;flex-direction:column;gap:8px;min-width:0;}
                .meeting-audio-card {background:#292929;border:1px solid #525252;border-radius:5px;overflow:hidden;}
                .meeting-audio-choice {display:flex;align-items:center;gap:16px;padding:23px 26px;font-size:17px;cursor:pointer;min-height:66px;}
                .meeting-audio-choice input {width:24px;height:24px;accent-color:#9790fa;flex-shrink:0;}
                .meeting-audio-devices {border-top:1px solid #525252;background:#202020;padding:18px 26px;}
                .meeting-device-row {display:flex;align-items:center;gap:16px;min-height:47px;color:#ddd;}
                .meeting-device-row select {width:100%;min-width:0;background:transparent;color:#ddd;border:0;font:inherit;font-size:16px;text-overflow:ellipsis;padding:6px 0;}
                .meeting-device-row option {background:#292929;}
                .meeting-speaker-icon {font-size:21px;width:24px;flex-shrink:0;}
                .meeting-test-audio {display:block;margin:16px 0 2px auto;font-size:15px;}
                .meeting-actions {display:flex;justify-content:flex-end;gap:12px;margin-top:28px;}
                .meeting-primary,.meeting-cancel {display:inline-flex;align-items:center;justify-content:center;min-width:118px;min-height:40px;border-radius:4px;font-size:16px;font-weight:600;padding:8px 22px;}
                .meeting-primary {background:#5b53b3;color:white;cursor:pointer;}
                .meeting-primary:hover {background:#7067cc;}
                .meeting-primary:disabled {opacity:.45;cursor:not-allowed;}
                .meeting-cancel {border:1px solid #777;background:#242424;color:white;}
                .meeting-cancel:hover {background:#333;}
                .meeting-footer {display:flex;align-items:center;justify-content:center;gap:24px;margin-top:70px;font-size:12px;color:#999;}
                .meeting-error {margin-top:16px;color:#ffacb5;text-align:right;font-size:14px;}
                .meeting-provider-note {margin-top:16px;color:#bbb;text-align:right;font-size:13px;}
                .meeting-connected {display:flex;align-items:center;justify-content:flex-end;gap:20px;margin-top:24px;}
                .meeting-remote-grid {display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin-top:20px;}
                .meeting-remote-tile {aspect-ratio:16/9;overflow:hidden;border-radius:4px;background:#242424;}
                .meeting-toast {position:fixed;bottom:24px;left:24px;padding:14px 20px;background:#333;border:1px solid #666;border-radius:5px;font-size:14px;}
                .meeting-join :focus-visible {outline:2px solid #b7b1ff;outline-offset:4px;}
                .meeting-join [hidden] {display:none!important;}
                .meeting-background-button {display:flex;align-items:center;gap:10px;font-size:14px;color:#ddd;cursor:pointer;text-align:left;}
                .meeting-background-button:hover {color:white;}
                .meeting-background-button[aria-expanded="true"] {color:#b7b1ff;}
                .meeting-background-panel {position:fixed;right:0;top:0;bottom:0;width:340px;background:#202020;border-left:1px solid #383838;display:flex;flex-direction:column;z-index:5;}
                .meeting-background-header {display:flex;align-items:center;justify-content:space-between;padding:24px 20px;gap:12px;}
                .meeting-background-header h2 {font-size:18px;font-weight:600;}
                .meeting-background-header button {padding:6px;cursor:pointer;color:#bbb;}
                .meeting-background-content {min-height:0;overflow-y:auto;padding:8px 20px 24px;}
                .meeting-background-grid {display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;}
                .meeting-background-option {position:relative;min-width:0;height:70px;background:#333;border:2px solid transparent;border-radius:3px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;cursor:pointer;overflow:hidden;font-size:11px;color:#eee;}
                .meeting-background-option:hover {border-color:#aaa;}
                .meeting-background-option.chosen {border-color:#a39bff;box-shadow:0 0 0 1px #a39bff;}
                .meeting-background-option.chosen:after {content:"✓";position:absolute;right:3px;top:1px;line-height:18px;width:18px;text-align:center;background:#514a9f;border-radius:50%;color:white;}
                .meeting-background-option:disabled {cursor:wait;opacity:.6;}
                .meeting-background-symbol {font-size:25px;line-height:28px;}
                .meeting-background-image img {position:absolute;inset:0;width:100%;height:100%;object-fit:cover;}
                .meeting-background-image>span {position:absolute;bottom:0;left:0;right:0;background:#0009;padding:3px 2px;text-align:center;}
                .meeting-background-upload input {position:absolute;inset:0;width:100%;height:100%;opacity:0;cursor:pointer;}
                .meeting-background-upload:focus-within {outline:2px solid #b7b1ff;outline-offset:3px;}
                .meeting-background-hint {font-size:12px;line-height:1.6;color:#aaa;margin-top:18px;}
                .meeting-background-error {font-size:13px;line-height:1.5;color:#ffacb5;margin-top:14px;}
                @media(min-width:1101px) {.meeting-body.with-backgrounds {width:calc(100% - 340px);margin-left:0;padding-left:24px;padding-right:24px;}.meeting-body.with-backgrounds .meeting-setup {gap:16px;}.meeting-body.with-backgrounds .meeting-camera-toolbar{gap:10px;padding-left:12px;padding-right:12px;}.meeting-body.with-backgrounds .meeting-audio-choice,.meeting-body.with-backgrounds .meeting-audio-devices{padding-left:16px;padding-right:16px;}.meeting-body.with-backgrounds .meeting-device-row{gap:8px;}.meeting-body.with-backgrounds .meeting-device-row select{font-size:14px;}}
                @media(max-width:1100px) {.meeting-background-panel{width:min(340px,92vw);box-shadow:-12px 0 30px #0006;}}
                @media(min-width:851px) {
                    .meeting-body {height:100%;display:flex;flex-direction:column;gap:clamp(10px,1.8dvh,20px);padding:0 clamp(24px,4vw,64px) 16px;}
                    .meeting-account {flex-shrink:0;padding:clamp(12px,1.8dvh,20px) 24px;}
                    .meeting-heading {flex-shrink:0;margin:0;padding-top:clamp(4px,2dvh,26px);gap:clamp(10px,1.5dvh,20px);}
                    .meeting-logo {width:clamp(40px,5dvh,52px);height:clamp(40px,5dvh,52px);}
                    .meeting-heading h1 {font-size:clamp(22px,2.7dvh,26px);}
                    .meeting-setup {flex:1;min-height:0;max-height:min(52dvh,520px);}
                    .meeting-preview {min-height:0;aspect-ratio:auto;}
                    .meeting-camera-toolbar {min-height:56px;padding:10px 20px;}
                    .meeting-audio-card:first-child {flex:1;display:flex;flex-direction:column;}
                    .meeting-audio-choice {flex-shrink:0;min-height:56px;padding:16px 24px;font-size:16px;}
                    .meeting-audio-devices {flex:1;padding:12px 24px;}
                    .meeting-device-row {min-height:42px;}
                    .meeting-test-audio {margin-top:10px;}
                    .meeting-actions,.meeting-connected {flex-shrink:0;margin-top:0;}
                    .meeting-footer {flex-shrink:0;margin-top:auto;}
                    .meeting-error,.meeting-provider-note {flex-shrink:0;margin-top:0;}
                }
                @media(max-width:850px) {.meeting-body{padding:0 20px 24px;}.meeting-heading{margin:44px 0 32px;}.meeting-setup{grid-template-columns:1fr;}.meeting-preview{min-height:220px;}.meeting-heading h1{font-size:23px;}.meeting-footer{margin-top:36px;flex-wrap:wrap;}}
                @media(max-width:480px) {.meeting-account{padding:14px;gap:10px;}.meeting-account-name{min-width:0;font-size:13px;}.meeting-camera-toolbar{padding:12px;gap:10px;}.meeting-audio-choice{padding:20px;}.meeting-audio-devices{padding:14px 18px;}.meeting-footer{gap:10px;}.meeting-heading{margin-top:36px;}}
            "##</style>
            <script>r##"
                    const CONTROL_ON = "flex h-10 items-center gap-2 rounded-md border border-edge bg-raised px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover";
                    const CONTROL_OFF = "flex h-10 items-center gap-2 rounded-md border border-busy bg-busy/15 px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-busy/25";

                    const url = @{Json(&link.url)};
                    const code = @{Json(&code)};
                    const providerReady = @{Json(&provider_ready)};

                    // The SDK is a megabyte, so it is imported inside the join
                    // rather than on the page, and only the elements it is
                    // handed are ever touched by it: re-renders leave their
                    // srcObject alone because nothing below re-creates them.
                    const SDK_URL = "/js/vendor/amazon-chime-sdk.js";

                    const localVideo = pp.ref(null);
                    const meetingAudio = pp.ref(null);
                    const remote0 = pp.ref(null);
                    const remote1 = pp.ref(null);
                    const remote2 = pp.ref(null);
                    const remote3 = pp.ref(null);
                    const remoteRefs = [remote0, remote1, remote2, remote3];

                    // The live call. A ref, not state: a re-render must never
                    // rebuild a meeting session, and nothing in the markup
                    // reads it.
                    const call = pp.ref(null);
                    const tileSlots = pp.ref(new Map());
                    // The SDK, and the device controller the preview and the meeting
                    // share. Shared on purpose: a camera somebody turned on before
                    // joining keeps running through the join rather than being torn
                    // down and started again.
                    const devices = pp.ref(null);
                    const speakerTest = pp.ref(null);
                    const backgroundUploads = pp.ref([]);
                    const cameraLifetime = pp.ref(0);
                    const disposed = pp.ref(false);
                    // Filled in when the SDK loads, so the stop handler can
                    // read the enum without importing the module again.
                    const statusCodes = pp.ref(null);

                    const [microphone, setMicrophone] = pp.state(true);
                    const [backgroundsOpen, setBackgroundsOpen] = pp.state(false);
                    const [backgroundId, setBackgroundId] = pp.state("none");
                    const [applyingBackground, setApplyingBackground] = pp.state(false);
                    const [backgroundError, setBackgroundError] = pp.state("");
                    const [backgrounds, setBackgrounds] = pp.state([
                        { id: "studio", name: "Light studio", url: "/images/backgrounds/studio.svg" },
                        { id: "loft", name: "Modern loft", url: "/images/backgrounds/loft.svg" },
                        { id: "library", name: "Reading room", url: "/images/backgrounds/library.svg" },
                        { id: "lavender", name: "Lavender room", url: "/images/backgrounds/lavender.svg" },
                        { id: "coast", name: "Coastal calm", url: "/images/backgrounds/coast.svg" },
                        { id: "mountains", name: "Mountains", url: "/images/backgrounds/mountains.svg" },
                        { id: "garden", name: "Green hills", url: "/images/backgrounds/garden.svg" },
                        { id: "sunset", name: "Sunset", url: "/images/backgrounds/sunset.svg" },
                    ]);
                    const [editingName, setEditingName] = pp.state(false);
                    const [audioMode, setAudioMode] = pp.state("computer");
                    const [microphoneDevice, setMicrophoneDevice] = pp.state("");
                    const [speakerDevice, setSpeakerDevice] = pp.state("");
                    const [audioInputs, setAudioInputs] = pp.state([]);
                    const [audioOutputs, setAudioOutputs] = pp.state([]);
                    const [camera, setCamera] = pp.state(false);
                    const [cameraLive, setCameraLive] = pp.state(false);
                    const [remoteTiles, setRemoteTiles] = pp.state([false, false, false, false]);
                    const [displayName, setDisplayName] = pp.state("Jefferson Abraham");
                    const [joining, setJoining] = pp.state(false);
                    const [session, setSession] = pp.state(null);
                    const [error, setError] = pp.state("");
                    const [toast, setToast] = pp.state("");

                    const remoteCount = remoteTiles.filter(Boolean).length;
                    const remoteSlots = remoteRefs.map((ref, index) => ({
                        index,
                        ref,
                        active: remoteTiles[index],
                    }));

                    // Read off the session for the panel, because a binding
                    // renders text and `session.Meeting.MeetingId` on a null
                    // session would throw during the render that hides it.
                    const conferenceId = session ? session.meeting.MeetingId : "";
                    const mediaRegion = session ? session.meeting.MediaRegion : "";
                    const attendeeName = session ? session.attendee.ExternalUserId : "";

                    pp.effect(() => {
                        if (!toast) return;
                        const timer = setTimeout(() => setToast(""), 4000);
                        return () => clearTimeout(timer);
                    }, [toast]);

                    async function share() {
                        const absolute = location.origin + url;
                        try {
                            await navigator.clipboard.writeText(absolute);
                            setToast("Link copied to clipboard");
                        } catch (failure) {
                            setToast("Copy the link from the address bar: " + absolute);
                        }
                    }

                    async function refreshDevices() {
                        try {
                            const available = await navigator.mediaDevices.enumerateDevices();
                            setAudioInputs(available.filter(device => device.kind === "audioinput" && device.deviceId));
                            setAudioOutputs(available.filter(device => device.kind === "audiooutput" && device.deviceId));
                        } catch (failure) {
                            setError("Audio devices could not be listed. Check your browser permissions.");
                        }
                    }

                    function backgroundEffect(id) {
                        if (id === "none" || id === "blur") return { id };
                        const found = backgrounds.find(item => item.id === id);
                        if (!found) throw new Error("Choose a background from the list.");
                        return found;
                    }

                    function handleBackgroundKey(event) {
                        if (event.key === "Escape") setBackgroundsOpen(false);
                    }

                    async function selectBackground(id, custom) {
                        if (applyingBackground || joining) return;
                        setBackgroundError("");
                        const effect = custom || backgroundEffect(id);
                        const previous = backgroundEffect(backgroundId);
                        if (!camera) {
                            setBackgroundId(id);
                            return;
                        }
                        setApplyingBackground(true);
                        try {
                            await closeCamera();
                            await openCamera(effect);
                            if (disposed.current) return;
                            if (call.current) call.current.av.startLocalVideoTile();
                            else startPreview();
                            setBackgroundId(id);
                        } catch (failure) {
                            setBackgroundError(failure.message || "This background could not be applied.");
                            try {
                                if (!disposed.current) {
                                    await closeCamera();
                                    await openCamera(previous);
                                    if (call.current) call.current.av.startLocalVideoTile();
                                    else startPreview();
                                }
                            } catch (recoveryFailure) {
                                setCamera(false);
                            }
                        } finally {
                            setApplyingBackground(false);
                        }
                    }

                    async function uploadBackground(event) {
                        const input = event.currentTarget;
                        const file = input.files && input.files[0];
                        input.value = "";
                        if (!file) return;
                        setBackgroundError("");
                        if (!["image/jpeg", "image/png", "image/webp"].includes(file.type) || file.size > 8 * 1024 * 1024) {
                            setBackgroundError("Choose a JPG, PNG, or WebP image smaller than 8 MB.");
                            return;
                        }
                        let bitmap;
                        try {
                            bitmap = await createImageBitmap(file);
                            if (bitmap.width > 8192 || bitmap.height > 8192) throw new Error("Choose an image smaller than 8192 pixels on each side.");
                            if (disposed.current) return;
                            const url = URL.createObjectURL(file);
                            backgroundUploads.current.push(url);
                            const custom = { id: "custom-" + backgroundUploads.current.length, name: file.name, url };
                            setBackgrounds(items => [...items, custom]);
                            await selectBackground(custom.id, custom);
                        } catch (failure) {
                            setBackgroundError(failure.message || "This image could not be read.");
                        } finally {
                            if (bitmap) bitmap.close();
                        }
                    }

                    function backgroundSpec() {
                        const base = location.origin + "/js/vendor/background/";
                        return {
                            paths: { worker: base + "worker.js", wasm: base + "_cwt-wasm.wasm", simd: base + "_cwt-wasm-simd.wasm" },
                            model: {
                                path: base + "selfie_segmentation_landscape.tflite",
                                input: { width: 256, height: 144, channels: 3, range: [0, 1] },
                                output: { width: 256, height: 144, channels: 1, range: [0, 1] },
                            },
                        };
                    }

                    async function filteredCamera(kit, cameraId, effect) {
                        if (effect.id === "none") return cameraId;
                        const processorType = effect.id === "blur" ? kit.sdk.BackgroundBlurVideoFrameProcessor : kit.sdk.BackgroundReplacementVideoFrameProcessor;
                        const options = { logger: kit.logger, filterCPUUtilization: 50 };
                        if (!await processorType.isSupported(backgroundSpec(), options)) {
                            throw new Error("Background effects are not supported by this browser. Choose None to use your camera.");
                        }
                        if (effect.id !== "blur") {
                            const response = await fetch(effect.url);
                            if (!response.ok) throw new Error("The background image could not be loaded.");
                            options.imageBlob = await response.blob();
                        }
                        const processor = await processorType.create(backgroundSpec(), options);
                        if (disposed.current) {
                            await processor.destroy();
                            throw new Error("Meeting setup was closed.");
                        }
                        try {
                            const transform = new kit.sdk.DefaultVideoTransformDevice(kit.logger, cameraId, [processor]);
                            kit.transform = transform;
                            return transform;
                        } catch (failure) {
                            await processor.destroy();
                            throw failure;
                        }
                    }

                    async function testSpeaker() {
                        setError("");
                        try {
                            if (speakerTest.current) await speakerTest.current.close();
                            const context = new AudioContext();
                            speakerTest.current = context;
                            if (speakerDevice && context.setSinkId) await context.setSinkId(speakerDevice);
                            await context.resume();
                            const tone = context.createOscillator();
                            const volume = context.createGain();
                            tone.frequency.value = 440;
                            volume.gain.setValueAtTime(0, context.currentTime);
                            volume.gain.linearRampToValueAtTime(0.12, context.currentTime + 0.04);
                            volume.gain.linearRampToValueAtTime(0, context.currentTime + 0.6);
                            tone.connect(volume);
                            volume.connect(context.destination);
                            tone.onended = pp.event(() => {
                                context.close();
                                if (speakerTest.current === context) speakerTest.current = null;
                            });
                            tone.start();
                            tone.stop(context.currentTime + 0.65);
                        } catch (failure) {
                            if (speakerTest.current) await speakerTest.current.close();
                            speakerTest.current = null;
                            setError("The speaker test could not play. Check your audio output.");
                        }
                    }

                    // Chime hands a tile update for every camera in the
                    // meeting, local and remote. Wrapped in pp.event because
                    // the SDK keeps these for the life of the call and they
                    // have to read the state of the render they run in, not
                    // the one that registered them.
                    const onTileUpdate = pp.event((tile) => {
                        const live = call.current;
                        if (!live || !tile.boundAttendeeId) return;

                        if (tile.localTile) {
                            if (localVideo.current) {
                                live.av.bindVideoElement(tile.tileId, localVideo.current);
                            }
                            return;
                        }

                        let slot = tileSlots.current.get(tile.tileId);
                        if (slot === undefined) {
                            slot = remoteTiles.findIndex(
                                (taken, index) =>
                                    !taken && !hasSlot(index),
                            );
                            // Four tiles on screen is what this layout holds.
                            // A fifth camera is in the meeting and audible,
                            // it simply is not shown.
                            if (slot < 0) return;
                            tileSlots.current.set(tile.tileId, slot);
                        }

                        const element = remoteRefs[slot].current;
                        if (element) live.av.bindVideoElement(tile.tileId, element);
                        setRemoteTiles((current) => withSlot(current, slot, true));
                    });

                    const onTileRemoved = pp.event((tileId) => {
                        const slot = tileSlots.current.get(tileId);
                        if (slot === undefined) return;
                        tileSlots.current.delete(tileId);
                        setRemoteTiles((current) => withSlot(current, slot, false));
                    });

                    // Chime ended the call from its side — the meeting was
                    // deleted, or the connection could not be recovered.
                    // Chime ended the session from its side. "Left" is this
                    // browser's own `leave()` coming back around, and the
                    // reader does not need to be told what they just did.
                    const onAudioVideoStop = pp.event((sessionStatus) => {
                        const code = sessionStatus.statusCode();
                        const codes = statusCodes.current;
                        if (codes && code === codes.Left) return;

                        teardown();
                        setSession(null);
                        setError(
                            codes && code === codes.MeetingEnded
                                ? "The meeting ended."
                                : "The connection to the meeting was lost.",
                        );
                    });

                    function hasSlot(index) {
                        for (const taken of tileSlots.current.values()) {
                            if (taken === index) return true;
                        }
                        return false;
                    }

                    function withSlot(slots, index, value) {
                        const next = slots.slice();
                        next[index] = value;
                        return next;
                    }

                    // Everything the SDK holds: microphone, camera, the
                    // peer connection. Best effort, because this also runs
                    // when the component is going away and a throw there
                    // would leave the rest unreleased.
                    const teardown = pp.event(async () => {
                        const live = call.current;
                        call.current = null;
                        tileSlots.current = new Map();
                        setRemoteTiles([false, false, false, false]);

                        // The camera goes off with the call rather than staying on
                        // over a pre-join screen nobody is looking at — and the
                        // button goes back with it, or it would read "on" above a
                        // picture that is no longer being taken.
                        setCamera(false);

                        try {
                            if (live) live.av.stopLocalVideoTile();
                            await closeCamera();
                            if (live) {
                                await live.av.stopAudioInput();
                                live.av.stop();
                            }
                        } catch (failure) {
                            // Leaving a call that is already gone is not a
                            // failure anybody needs to hear about.
                        }
                    });

                    // The SDK and the device controller, loaded once. Reached
                    // before a meeting exists, because the camera preview is
                    // a device, not a meeting.
                    async function ensureDevices() {
                        if (devices.current) return devices.current;

                        const sdk = await import(SDK_URL);
                        statusCodes.current = sdk.MeetingSessionStatusCode;
                        const logger = new sdk.ConsoleLogger("upurnaka", sdk.LogLevel.WARN);

                        devices.current = {
                            sdk,
                            logger,
                            controller: new sdk.DefaultDeviceController(logger),
                            videoInput: false,
                            previewing: false,
                        };
                        return devices.current;
                    }

                    // Open the camera. Safe to call when it is already open.
                    async function openCamera(effect = backgroundEffect(backgroundId)) {
                        const lifetime = cameraLifetime.current;
                        const kit = await ensureDevices();
                        if (disposed.current) return kit;
                        if (kit.videoInput) return kit;

                        const cameras = await kit.controller.listVideoInputDevices();
                        if (cameras.length === 0) throw new Error("No camera was found.");

                        const input = await filteredCamera(kit, cameras[0].deviceId, effect);
                        if (disposed.current || lifetime !== cameraLifetime.current) {
                            if (kit.transform) await kit.transform.stop();
                            kit.transform = null;
                            return kit;
                        }
                        let stream;
                        try {
                            stream = await kit.controller.startVideoInput(input);
                        } catch (failure) {
                            if (kit.transform) await kit.transform.stop();
                            kit.transform = null;
                            throw failure;
                        }
                        if (disposed.current || lifetime !== cameraLifetime.current) {
                            await kit.controller.stopVideoInput();
                            if (kit.transform) await kit.transform.stop();
                            kit.transform = null;
                            return kit;
                        }

                        // A camera that enumerates and then hands back no live track is one
                        // the browser refused or another application is holding. Chime
                        // reports most of those by throwing; this covers the rest, so the
                        // button cannot sit on "Camera on" above a black rectangle.
                        const track = stream && stream.getVideoTracks()[0];
                        if (!track || track.readyState !== "live") {
                            try {
                                await kit.controller.stopVideoInput();
                            } catch (failure) {
                                // Nothing was started; nothing to stop.
                            }
                            throw new Error(
                                "The camera could not be started. Check this site's camera permission.",
                            );
                        }

                        kit.videoInput = true;
                        return kit;
                    }

                    async function closeCamera() {
                        cameraLifetime.current += 1;
                        const kit = devices.current;
                        stopPreview();
                        setCameraLive(false);
                        if (!kit) return;
                        try {
                            if (kit.videoInput) await kit.controller.stopVideoInput();
                        } finally {
                            kit.videoInput = false;
                            if (kit.transform) await kit.transform.stop();
                            kit.transform = null;
                        }
                    }

                    // The picture before the meeting: Teams shows you yourself
                    // while you decide. There is no conference yet, so this is
                    // the device controller's own preview rather than a tile.
                    function startPreview() {
                        const kit = devices.current;
                        if (!kit || kit.previewing || !localVideo.current) return;
                        kit.controller.startVideoPreviewForVideoInput(localVideo.current);
                        kit.previewing = true;
                    }

                    // Also what hands the element back before a tile binds it.
                    function stopPreview() {
                        const kit = devices.current;
                        if (!kit || !kit.previewing) return;
                        if (localVideo.current) {
                            kit.controller.stopVideoPreviewForVideoInput(localVideo.current);
                        }
                        kit.previewing = false;
                    }

                    // Open the audio and video session Chime just gave us a
                    // seat in.
                    async function connect(info) {
                        const kit = await ensureDevices();
                        const configuration = new kit.sdk.MeetingSessionConfiguration(
                            info.meeting,
                            info.attendee,
                        );
                        const meetingSession = new kit.sdk.DefaultMeetingSession(
                            configuration,
                            kit.logger,
                            kit.controller,
                        );
                        const av = meetingSession.audioVideo;

                        av.addObserver({
                            videoTileDidUpdate: onTileUpdate,
                            videoTileWasRemoved: onTileRemoved,
                            audioVideoDidStop: onAudioVideoStop,
                        });

                        if (audioMode === "computer") {
                            const microphones = await av.listAudioInputDevices();
                            if (microphones.length > 0) {
                                await av.startAudioInput(microphoneDevice || microphones[0].deviceId);
                            }
                            if (speakerDevice) await av.chooseAudioOutput(speakerDevice);
                            if (meetingAudio.current) {
                                await av.bindAudioElement(meetingAudio.current);
                            }
                        }

                        call.current = { session: meetingSession, av };
                        av.start();

                        if (!microphone || audioMode === "none") av.realtimeMuteLocalAudio();

                        if (camera) {
                            // The preview was holding the video element; the
                            // meeting's own tile takes it over.
                            stopPreview();
                            await openCamera();
                            av.startLocalVideoTile();
                        }
                    }

                    // The server finds or creates the Chime conference and
                    // mints this person's seat. The join token in the reply
                    // is a credential for this browser and this attendee —
                    // it stays in this state and goes nowhere else.
                    async function joinNow() {
                        if (joining || applyingBackground) return;
                        setJoining(true);
                        setError("");
                        try {
                            const info = await pp.rpc("join_meeting", {
                                id: code,
                                display_name: displayName,
                            });
                            await connect(info);
                            setSession(info);
                        } catch (failure) {
                            await teardown();
                            setError(failure.message || "The meeting could not be joined.");
                        } finally {
                            setJoining(false);
                        }
                    }

                    async function toggleMicrophone() {
                        const next = !microphone;
                        setMicrophone(next);
                        const live = call.current;
                        if (!live) return;
                        if (next) {
                            live.av.realtimeUnmuteLocalAudio();
                        } else {
                            live.av.realtimeMuteLocalAudio();
                        }
                    }

                    async function toggleCamera() {
                        if (applyingBackground || joining) return;
                        setApplyingBackground(true);
                        const next = !camera;
                        setCamera(next);
                        setError("");

                        try {
                            if (!next) {
                                if (call.current) call.current.av.stopLocalVideoTile();
                                await closeCamera();
                                return;
                            }

                            await openCamera();

                            if (call.current) {
                                stopPreview();
                                call.current.av.startLocalVideoTile();
                            } else {
                                startPreview();
                            }
                        } catch (failure) {
                            // A refused permission or a camera another application
                            // already holds. Put the button back rather than leaving
                            // it on over a picture that is never going to arrive.
                            await closeCamera();
                            setCamera(false);
                            setCameraLive(false);
                            setError(failure.message || "The camera could not be started.");
                        } finally {
                            setApplyingBackground(false);
                        }
                    }

                    async function leave() {
                        await teardown();
                        setSession(null);
                    }

                    // Navigating away mid-call must release the microphone
                    // and camera, or the browser keeps showing this tab as
                    // recording.
                    pp.effect(() => {
                        return () => {
                            disposed.current = true;
                            teardown();
                            if (speakerTest.current) speakerTest.current.close();
                            backgroundUploads.current.forEach(url => URL.revokeObjectURL(url));
                        };
                    }, []);
                "##</script>
        </main>
    };

    Ok(Page::new(
        content,
        Metadata::new()
            .title(&title)
            .description(format!("Join {title} on upurnaka.")),
    ))
}

/// Take a seat in the conference behind a meeting link.
///
/// The join code is checked the same way the page checked it: a revoked or
/// unknown one is a 404 here too, because an RPC a browser can call is a
/// second front door and guarding only the page would leave it open.
#[rpc]
pub async fn join_meeting(id: String, display_name: String) -> Result<JoinInfo> {
    let id = id.trim().to_string();
    let display_name = display_name.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("id", &id)
        .max_chars("id", &id, 64)
        .required("display_name", &display_name)
        .max_chars("display_name", &display_name, 64);
    validator.finish()?;

    meetings::join(&id, &display_name).await
}
