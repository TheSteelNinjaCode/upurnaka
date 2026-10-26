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
        <main class="meeting-join">
            <div class="meeting-body">
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
                                class={camera ? "meeting-toggle is-on" : "meeting-toggle"}
                                onclick={toggleCamera()}
                            >
                                <span></span>
                            </button>
                            <span class="meeting-toolbar-divider"></span>
                            <button
                                type="button"
                                class="meeting-unavailable"
                                disabled
                                title="Background effects are not available yet"
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
                        disabled={!providerReady || joining || displayName.trim().length === 0}
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
                    // Filled in when the SDK loads, so the stop handler can
                    // read the enum without importing the module again.
                    const statusCodes = pp.ref(null);

                    const [microphone, setMicrophone] = pp.state(true);
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
                    async function openCamera() {
                        const kit = await ensureDevices();
                        if (kit.videoInput) return kit;

                        const cameras = await kit.controller.listVideoInputDevices();
                        if (cameras.length === 0) throw new Error("No camera was found.");

                        const stream = await kit.controller.startVideoInput(cameras[0].deviceId);

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
                        const kit = devices.current;
                        stopPreview();
                        setCameraLive(false);
                        if (!kit || !kit.videoInput) return;
                        await kit.controller.stopVideoInput();
                        kit.videoInput = false;
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
                        if (joining) return;
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
                            setCamera(false);
                            setCameraLive(false);
                            setError(failure.message || "The camera could not be started.");
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
                            teardown();
                            if (speakerTest.current) speakerTest.current.close();
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
