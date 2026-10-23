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

use crate::components::app_shell::AppShell;
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
        <AppShell active="meet">
            <main
                class="scroll-thin relative min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="mx-auto w-full max-w-[920px] px-14 py-9">
                    <a
                        href="/meet"
                        class="flex items-center gap-2 text-[13px] text-link hover:underline"
                    >
                        <Icon name="chevron-left" class="h-4 w-4" />
                        "Back to Meet"
                    </a>
                    <h1 class="mt-5 truncate text-[22px] font-semibold tracking-tight text-ink">
                        @{&title}
                    </h1>
                    <p class="mt-1 text-[13px] text-muted">"Meeting ID "@{&code}</p>
                    <div
                        class="relative mt-7 flex aspect-video w-full flex-col items-center justify-center gap-4 overflow-hidden rounded-xl border border-edge bg-sunken"
                    >
                        <span
                            class="flex flex-col items-center gap-4"
                            hidden={camera && cameraLive}
                        >
                            <Avatar initials="JA" size="xl" presence="online" />
                            <span class="text-[13px] text-muted">
                                {camera ? "Starting your camera…" : "Your camera is turned off"}
                            </span>
                        </span>
                        <video
                            pp-ref={localVideo}
                            class="absolute inset-0 h-full w-full -scale-x-100 rounded-xl object-cover"
                            hidden={!camera || !cameraLive}
                            muted
                            playsinline
                        >
                        </video>
                    </div>
                    <div class="mt-4 grid grid-cols-4 gap-3" hidden={remoteCount === 0}>
                        <template pp-for="slot in remoteSlots">
                            <div
                                key={slot.index}
                                class="relative aspect-video overflow-hidden rounded-lg border border-edge bg-sunken"
                                hidden={!slot.active}
                            >
                                <video
                                    pp-ref={slot.ref}
                                    class="h-full w-full object-cover"
                                    playsinline
                                >
                                </video>
                            </div>
                        </template>
                    </div>
                    <audio pp-ref={meetingAudio} class="hidden"></audio>
                    <div class="mt-5 flex items-center justify-center gap-3">
                        <button
                            type="button"
                            class={microphone ? CONTROL_ON : CONTROL_OFF}
                            aria-pressed={microphone}
                            onclick={toggleMicrophone()}
                        >
                            <span class="flex" hidden={!microphone}>
                                <Icon name="mic" class="h-[18px] w-[18px]" />
                            </span>
                            <span class="flex" hidden={microphone}>
                                <Icon name="mic-off" class="h-[18px] w-[18px]" />
                            </span>
                            {microphone ? "Mic on" : "Mic off"}
                        </button>
                        <button
                            type="button"
                            class={camera ? CONTROL_ON : CONTROL_OFF}
                            aria-pressed={camera}
                            onclick={toggleCamera()}
                        >
                            <span class="flex" hidden={!camera}>
                                <Icon name="video" class="h-[18px] w-[18px]" />
                            </span>
                            <span class="flex" hidden={camera}>
                                <Icon name="video-off" class="h-[18px] w-[18px]" />
                            </span>
                            {camera ? "Camera on" : "Camera off"}
                        </button>
                    </div>
                    <div class="mt-7 flex flex-col items-center gap-4" hidden={session !== null}>
                        <label class="flex items-center gap-3 text-[13px] text-muted">
                            "Your name"
                            <input
                                name="display_name"
                                type="text"
                                class="h-9 w-[260px] rounded border border-edge bg-raised px-3 text-[13px] text-ink placeholder:text-subtle focus:border-brand focus:outline-none"
                                autocomplete="name"
                                value={displayName}
                                oninput={setDisplayName(target.value)}
                            />
                        </label>
                        <div class="flex items-center gap-3">
                            <button
                                type="button"
                                class="h-10 rounded-md bg-brand px-8 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                disabled={!providerReady || joining || displayName.trim().length === 0}
                                onclick={joinNow()}
                            >
                                {joining ? "Joining…" : "Join now"}
                            </button>
                            <button
                                type="button"
                                class="flex h-10 items-center gap-2 rounded-md border border-edge bg-raised px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                onclick={share()}
                            >
                                <Icon name="copy" class="h-4 w-4" />
                                "Copy link"
                            </button>
                        </div>
                        <p class="text-xs text-busy" hidden={!error}>{error}</p>
                        <p class="text-xs text-subtle" hidden={providerReady}>
                            "No meeting provider is configured, so this link has no call behind it. Set CHIME_MEDIA_REGION to connect one."
                        </p>
                    </div>
                    <div
                        class="mt-7 rounded-lg border border-edge bg-raised px-6 py-5"
                        hidden={session === null}
                    >
                        <p class="text-center text-[15px] font-semibold text-ink">
                            "Connected to the meeting"
                        </p>
                        <dl class="mx-auto mt-5 grid max-w-[520px] grid-cols-2 gap-y-2 text-[13px]">
                            <dt class="text-muted">"Conference"</dt>
                            <dd class="truncate text-right text-ink">{conferenceId}</dd>
                            <dt class="text-muted">"Media region"</dt>
                            <dd class="truncate text-right text-ink">{mediaRegion}</dd>
                            <dt class="text-muted">"You are"</dt>
                            <dd class="truncate text-right text-ink">{attendeeName}</dd>
                        </dl>
                        <p class="mt-5 text-center text-xs text-subtle">
                            "Chime has the conference and this seat in it. Audio and video still need amazon-chime-sdk-js loaded in the page."
                        </p>
                        <p class="mt-5 text-center">
                            <button
                                type="button"
                                class="h-9 rounded-md border border-edge bg-surface px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                onclick={leave()}
                            >
                                "Leave"
                            </button>
                        </p>
                    </div>
                </div>
                <div
                    class="fixed bottom-4 left-4 z-50 flex items-center gap-3 rounded-md border border-edge bg-raised px-4 py-3 text-[13px] text-ink shadow-2xl shadow-black/70"
                    role="status"
                    hidden={!toast}
                >
                    <Icon name="check-circle" class="h-[18px] w-[18px] text-online" />
                    {toast}
                </div>
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
                    const REMOTE_SLOTS = 4;

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
                    // Filled in when the SDK loads, so the stop handler can
                    // read the enum without importing the module again.
                    const statusCodes = pp.ref(null);

                    const [microphone, setMicrophone] = pp.state(true);
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
                                setCameraLive(true);
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
                        setCameraLive(false);
                        setRemoteTiles([false, false, false, false]);
                        if (!live) return;

                        try {
                            live.av.stopLocalVideoTile();
                            await live.av.stopVideoInput();
                            await live.av.stopAudioInput();
                            live.av.stop();
                        } catch (failure) {
                            // Leaving a call that is already gone is not a
                            // failure anybody needs to hear about.
                        }
                    });

                    async function startCamera(av) {
                        const cameras = await av.listVideoInputDevices();
                        if (cameras.length === 0) {
                            setError("No camera was found.");
                            return;
                        }
                        await av.startVideoInput(cameras[0].deviceId);
                        av.startLocalVideoTile();
                    }

                    // Open the audio and video session Chime just gave us a
                    // seat in.
                    async function connect(info) {
                        const sdk = await import(SDK_URL);
                        statusCodes.current = sdk.MeetingSessionStatusCode;

                        const logger = new sdk.ConsoleLogger("upurnaka", sdk.LogLevel.WARN);
                        const devices = new sdk.DefaultDeviceController(logger);
                        const configuration = new sdk.MeetingSessionConfiguration(
                            info.meeting,
                            info.attendee,
                        );
                        const meetingSession = new sdk.DefaultMeetingSession(
                            configuration,
                            logger,
                            devices,
                        );
                        const av = meetingSession.audioVideo;

                        av.addObserver({
                            videoTileDidUpdate: onTileUpdate,
                            videoTileWasRemoved: onTileRemoved,
                            audioVideoDidStop: onAudioVideoStop,
                        });

                        const microphones = await av.listAudioInputDevices();
                        if (microphones.length > 0) {
                            await av.startAudioInput(microphones[0].deviceId);
                        }
                        if (meetingAudio.current) {
                            await av.bindAudioElement(meetingAudio.current);
                        }

                        call.current = { session: meetingSession, av };
                        av.start();

                        if (!microphone) av.realtimeMuteLocalAudio();
                        if (camera) await startCamera(av);
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
                        const live = call.current;
                        if (!live) return;
                        try {
                            if (next) {
                                await startCamera(live.av);
                            } else {
                                live.av.stopLocalVideoTile();
                                await live.av.stopVideoInput();
                                setCameraLive(false);
                            }
                        } catch (failure) {
                            setError(failure.message || "The camera could not be changed.");
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
                        };
                    }, []);
                "##</script>
            </main>
        </AppShell>
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
