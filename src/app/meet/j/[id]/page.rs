//! Where a meeting link lands: the pre-join screen for one join code.
//!
//! `/meet/j/{id}`. This is the half of a meeting link that makes it a link at
//! all — somebody who was sent one opens it here, sets their camera and
//! microphone the way they want them, and joins.
//!
//! "Join now" calls [`join_meeting`], which reaches Dilnaka Meet through `crate::meetings::join`: a conference is found or created for this
//! link and a seat in it is minted for this person. What comes back is a
//! `MeetingSessionConfiguration`'s two arguments, ready for
//! `amazon-chime-sdk-js` to open audio and video with.
//!
//! The media client itself is not loaded here yet — see the note in the
//! connected panel below. Everything up to it is live, so a join either
//! produces real Dilnaka Meet credentials or a real failure.
//!
//! With no `DILNAKA_MEETINGS_KEY` configured there is no provider, and the
//! button says so instead of calling one.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::components::aria::{bound, hidden, labelled, named, plain, submit};
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::components::notification_sound::NotificationSound;
use crate::components::rahti_ui::attrs::{Attrs, ui_expr};
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger,
};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::label::Label;
use crate::components::rahti_ui::radio_group::{RadioGroup, RadioGroupItem};
use crate::components::rahti_ui::select::{Select, SelectItem};
use crate::components::rahti_ui::spinner::Spinner;
use crate::components::rahti_ui::switch::Switch;
use crate::meetings::{self, JoinInfo, dilnaka_meet, recording};
use crate::rahti::{Error, Json, Metadata, Page, Result, RpcUpload, Validator, html, rpc};

/// A tool in the meeting toolbar: an icon over its label, underlined while on.
const ROOM_TOOL: &str = "h-[60px]! min-w-14 flex-col gap-[5px] px-2.5! py-1.5 text-xs font-normal \
                         text-ink aria-pressed:bg-hover aria-disabled:opacity-40 \
                         aria-pressed:shadow-[inset_0_-2px_var(--color-brand-ink)] \
                         max-[1000px]:min-w-[45px] max-[1000px]:p-1.5! max-[1000px]:text-[11px] \
                         max-[650px]:h-[52px]! max-[650px]:min-w-10 \
                         [&_svg]:size-6! max-[650px]:[&_svg]:size-5!";

/// Leaving the meeting: the toolbar tool in the danger colour.
const ROOM_LEAVE: &str = "ml-3.5 text-danger-ink! hover:bg-destructive/40! max-[1000px]:ml-1";

/// A device picker on the pre-join screen.
const DEVICE_SELECT: &str = "max-w-full min-w-0 text-ellipsis border-transparent bg-transparent text-base shadow-none \
                             dark:bg-transparent \
                             min-[1101px]:group-data-[backgrounds=true]/body:text-sm";

/// A background tile in the background panel, outlined while chosen.
const BACKGROUND_OPTION: &str = "meeting-background-option relative h-[70px]! min-w-0 flex-col \
                                 gap-[3px] overflow-hidden rounded-[3px]! border-2 \
                                 border-transparent bg-raised p-0! text-xs font-normal \
                                 aria-pressed:border-brand-ink";

/// A row in a menu the room draws itself: the reactions, More, a tile's options.
const MENU_ROW: &str = "w-full justify-start! px-4! py-2.5 text-left text-[13px] font-normal";

/// The picker dialog's Cancel, an outline button.
const PICKER_CANCEL: &str = "inline-flex h-9 min-w-24 items-center justify-center rounded-md border border-edge bg-transparent px-4 text-sm font-medium hover:bg-hover";

/// An action that reads as a link.
const LINK: &str = "h-auto! px-0! text-link";

/// `?p=` carries a passcode-protected meeting's passcode, as a shared
/// link may.
#[derive(Deserialize)]
pub struct JoinQuery {
    pub p: Option<String>,
}

pub async fn page(Path(id): Path<String>, Query(query): Query<JoinQuery>) -> Result<Response> {
    let anonymous = crate::auth::current().is_none();
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();
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

    // Who may enter is decided again, on the server, when a lobby ticket is
    // asked for; this only picks what to show first.
    let access = if crate::db::is_connected() {
        Some(meetings::access::page_access(&code).await?)
    } else {
        None
    };
    let mode = access
        .as_ref()
        .map_or(meetings::access::OPEN, |a| a.mode.as_str());
    let host = access.as_ref().is_some_and(|a| a.host);
    if mode == meetings::access::INVITED && !host {
        if anonymous {
            return Ok(Redirect::to(&format!("/signin?next=/meet/j/{code}")).into_response());
        }
        if !access.as_ref().is_some_and(|a| a.invited) {
            return Err(meetings::access::not_invited());
        }
    }
    let needs_passcode = mode == meetings::access::PASSCODE && !host;
    let initial_passcode: String = if needs_passcode {
        query.p.unwrap_or_default().chars().take(64).collect()
    } else {
        String::new()
    };

    // Decided on the server, because it is a deployment fact rather than
    // something the browser could discover: with no provider configured the
    // button must not pretend there is a call to join.
    let provider_ready = dilnaka_meet::is_configured();
    let recording_ready = recording::is_configured();

    let room_leave = format!("{ROOM_TOOL} {ROOM_LEAVE}");
    let upload_background = button_variants(
        ButtonVariant::Ghost,
        ButtonSize::Default,
        &format!(
            "{BACKGROUND_OPTION} focus-within:outline-2 focus-within:outline-offset-3 focus-within:outline-ring"
        ),
    );
    let content = html! {
        <main
            class="meeting-join h-dvh overflow-auto bg-pane text-ink **:focus-visible:outline-2 **:focus-visible:outline-offset-4 **:focus-visible:outline-ring"
            onkeydown={handleBackgroundKey(event)}
        >
            <section
                class="meeting-passcode mx-auto flex min-h-dvh max-w-[480px] flex-col items-center justify-center gap-6 px-6 py-12 text-center"
                hidden={!passcodeGate}
            >
                <div
                    class="flex size-20 items-center justify-center rounded-2xl bg-brand/20 text-brand-ink"
                >
                    <Icon name="shield" class="size-10" />
                </div>
                <div>
                    <p class="mb-3 text-sm text-muted-foreground">"UPURNAKA MEETINGS"</p>
                    <h1 class="text-2xl font-semibold">"Enter the meeting passcode"</h1>
                    <p class="mt-3 text-muted-foreground">@{title.as_str()}</p>
                </div>
                <form
                    class="grid w-full gap-3 rounded-xl border border-edge bg-raised p-7 text-left shadow-lg"
                    onsubmit={submitPasscode(event)}
                    novalidate
                >
                    <Label for="meeting-passcode">"Passcode"</Label>
                    <Input
                        id="meeting-passcode"
                        name="passcode"
                        attrs=@{Attrs::new()
                            .set("autocomplete", "off")
                            .set("maxlength", "64")
                            .set("autocapitalize", "off")
                            .set("spellcheck", "false")}
                        value={passcode}
                        oninput={setPasscode(target.value)}
                        disabled={checkingPasscode}
                    />
                    <p class="text-sm text-danger-ink" aria-live="polite" hidden={!passcodeError}>
                        {passcodeError}
                    </p>
                    <Button
                        class="w-full"
                        attrs=@{submit()}
                        disabled={checkingPasscode || !passcode.trim()}
                    >
                        {checkingPasscode ? "Checking…" : "Continue"}
                    </Button>
                    <p class="text-sm text-muted-foreground">
                        "The organizer protected this meeting with a passcode. It's in the invitation."
                    </p>
                </form>
            </section>
            <section
                class="meeting-launch mx-auto flex min-h-dvh max-w-[480px] flex-col items-center justify-center gap-6 px-6 py-12 text-center"
                hidden={passcodeGate || !showLauncher}
            >
                <div
                    class="flex size-20 items-center justify-center rounded-2xl bg-brand/20 text-brand-ink"
                >
                    <Icon name="video" class="size-10" />
                </div>
                <div>
                    <p class="mb-3 text-sm text-muted-foreground">"UPURNAKA MEETINGS"</p>
                    <h1 class="text-2xl font-semibold">"Join your meeting"</h1>
                    <p class="mt-3 text-muted-foreground">@{title.as_str()}</p>
                </div>
                <div
                    class="grid w-full gap-3 rounded-xl border border-edge bg-raised p-7 shadow-lg"
                >
                    <Button class="guest-browser w-full" onclick={setShowLauncher(false)}>
                        "Continue in this browser"
                    </Button>
                    <Button variant="outline" class="guest-install w-full" onclick={installApp()}>
                        "Get the Upurnaka app"
                    </Button>
                    <p class="text-sm text-muted-foreground">
                        "No account needed. Enter your name, then wait for the organizer to let you in."
                    </p>
                    <p class="text-sm text-muted-foreground" role="status" hidden={!installHelp}>
                        {installHelp}
                    </p>
                </div>
                <a href="/signin" class="text-sm text-link">"Sign in with an account"</a>
            </section>
            <div
                class="meeting-body group/body mx-auto max-w-[1460px] px-5 pb-6 min-[851px]:flex min-[851px]:h-full min-[851px]:flex-col min-[851px]:gap-[clamp(10px,1.8dvh,20px)] min-[851px]:px-[clamp(24px,4vw,64px)] min-[851px]:pb-4 min-[1101px]:data-[backgrounds=true]:ml-0 min-[1101px]:data-[backgrounds=true]:w-[calc(100%-340px)] min-[1101px]:data-[backgrounds=true]:px-6"
                data-backgrounds={backgroundsOpen}
                hidden={passcodeGate || showLauncher || session !== null}
            >
                <div
                    class="meeting-account relative mx-auto flex w-fit max-w-full items-center gap-4 rounded-b-md bg-raised px-6 py-5 shadow-md shadow-black/20 max-[480px]:gap-2.5 max-[480px]:p-3.5 min-[851px]:shrink-0 min-[851px]:py-[clamp(12px,1.8dvh,20px)]"
                    hidden={anonymous}
                >
                    <Avatar initials=@{initials.as_str()} size="lg" presence="none" />
                    <div
                        class="flex min-w-40 flex-col gap-0.5 text-[15px] max-[480px]:min-w-0 max-[480px]:text-[13px]"
                    >
                        <strong>{displayName || "Your name"}</strong>
                        <span class="text-[13px] text-muted-foreground">
                            @{if anonymous { "Joining as a guest" } else { "Joining with your account" }}
                        </span>
                    </div>
                    <Button variant="link" class=@{LINK} onclick={setEditingName(!editingName)}>
                        {editingName ? "Done" : "Change"}
                    </Button>
                    <div
                        class="meeting-name-editor absolute inset-x-0 top-full z-2 grid gap-2 bg-raised p-4 text-[13px] shadow-lg shadow-black/30"
                        hidden={!editingName}
                    >
                        <Label for="meeting-display-name" class="font-normal">"Your name"</Label>
                        <Input
                            id="meeting-display-name"
                            name="display_name"
                            attrs=@{Attrs::new().set("autocomplete", "name").set("maxlength", "64")}
                            value={displayName}
                            oninput={setDisplayName(target.value)}
                        />
                    </div>
                </div>
                <div
                    class="mb-8 mt-11 flex flex-col items-center gap-6 text-center max-[480px]:mt-9 min-[851px]:m-0 min-[851px]:shrink-0 min-[851px]:gap-[clamp(10px,1.5dvh,20px)] min-[851px]:pt-[clamp(4px,2dvh,26px)]"
                >
                    <span
                        class="grid h-[52px] w-[52px] place-items-center rounded-xl bg-linear-145 from-brand-ink to-brand-press text-white shadow-[8px_5px_0_-3px_var(--color-brand)] min-[851px]:h-[clamp(40px,5dvh,52px)] min-[851px]:w-[clamp(40px,5dvh,52px)]"
                    >
                        <Icon name="teams" class="h-9 w-9" />
                    </span>
                    <h1
                        class="text-[23px] font-semibold leading-[1.3] wrap-anywhere min-[851px]:text-[clamp(22px,2.7dvh,26px)]"
                    >
                        @{&title}
                    </h1>
                    <div
                        class="meeting-guest-name grid w-full max-w-sm gap-2 text-left"
                        hidden={!anonymous}
                    >
                        <Label for="guest-display-name">"Your name"</Label>
                        <Input
                            id="guest-display-name"
                            name="guest_name"
                            attrs=@{Attrs::new().set("autocomplete", "name").set("maxlength", "64").set("placeholder", "Enter your name")}
                            value={displayName}
                            oninput={setDisplayName(target.value)}
                            disabled={lobbyWaiting || joining}
                        />
                        <p class="text-center text-xs text-muted-foreground">
                            "Joining as a guest. The organizer will let you in."
                        </p>
                    </div>
                </div>
                <div
                    class="grid grid-cols-1 gap-6 min-[851px]:max-h-[min(52dvh,520px)] min-[851px]:min-h-0 min-[851px]:flex-1 min-[851px]:grid-cols-[1.13fr_1fr] min-[1101px]:group-data-[backgrounds=true]/body:gap-4"
                >
                    <section
                        class="flex min-w-0 flex-col overflow-hidden rounded-md border border-edge bg-surface"
                        aria-label="Camera preview"
                    >
                        <div
                            class="relative grid aspect-video min-h-[220px] flex-1 place-items-center min-[851px]:aspect-auto min-[851px]:min-h-0"
                        >
                            <video
                                pp-ref={localVideo}
                                class="absolute inset-0 h-full w-full object-cover"
                                pp-style={selfVideoStyle}
                                onloadedmetadata={setCameraLive(true)}
                                onemptied={setCameraLive(false)}
                                muted
                                playsinline
                            >
                            </video>
                            <div
                                class="meeting-camera-off relative flex flex-col items-center gap-[18px] text-[17px] text-muted-foreground"
                                hidden={cameraLive}
                            >
                                <Icon name="video-off" class="h-8 w-8" />
                                <strong>
                                    {camera ? "Starting your camera…" : "Your camera is turned off"}
                                </strong>
                            </div>
                        </div>
                        <div
                            class="flex min-h-16 items-center gap-[18px] border-t border-edge px-6 py-3 text-muted-foreground max-[480px]:gap-2.5 max-[480px]:p-3 min-[851px]:min-h-14 min-[851px]:px-5 min-[851px]:py-2.5 min-[1101px]:group-data-[backgrounds=true]/body:gap-2.5 min-[1101px]:group-data-[backgrounds=true]/body:px-3"
                        >
                            <Icon name="video-off" class="h-6 w-6" />
                            <Switch
                                attrs=@{named("Camera")}
                                checked={camera}
                                disabled={applyingBackground || joining}
                                onchange={toggleCamera()}
                            />
                            <span class="ml-2 mr-0.5 h-[30px] border-l border-edge"></span>
                            <Button
                                variant="ghost"
                                class="meeting-background-button gap-2.5 px-2! text-sm font-normal text-muted-foreground aria-expanded:text-brand-ink"
                                attrs=@{bound("aria-expanded", ui_expr!(aria-expanded={backgroundsOpen}))}
                                onclick={setBackgroundsOpen(!backgroundsOpen)}
                            >
                                <Icon name="image" class="size-5" />
                                "Background filters"
                            </Button>
                        </div>
                    </section>
                    <RadioGroup class="flex min-w-0 flex-col gap-2" attrs=@{named("Audio options")}>
                        <div
                            class="overflow-hidden rounded-md border border-edge bg-raised min-[851px]:flex min-[851px]:flex-1 min-[851px]:flex-col"
                        >
                            <label class={AUDIO_CHOICE}>
                                <RadioGroupItem
                                    name="audio-mode"
                                    value="computer"
                                    class="size-6"
                                    checked={audioMode === "computer"}
                                    onchange={setAudioMode("computer")}
                                    disabled={session !== null}
                                />
                                <strong>"Computer audio"</strong>
                            </label>
                            <div
                                class="meeting-audio-devices border-t border-edge bg-surface px-[26px] py-[18px] max-[480px]:px-[18px] max-[480px]:py-3.5 min-[851px]:flex-1 min-[851px]:px-6 min-[851px]:py-3 min-[1101px]:group-data-[backgrounds=true]/body:px-4"
                                hidden={audioMode !== "computer"}
                            >
                                <div class={DEVICE_ROW}>
                                    <Icon name="mic" class="h-6 w-6 shrink-0" />
                                    <Select
                                        class=@{DEVICE_SELECT}
                                        attrs=@{named("Microphone device")}
                                        value={availableDevice(microphoneDevice, audioInputs)}
                                        onfocus={refreshDevices()}
                                        onchange={setMicrophoneDevice(target.value)}
                                        disabled={session !== null}
                                    >
                                        <SelectItem value="">
                                            {defaultDeviceName(audioInputs, "Default microphone")}
                                        </SelectItem>
                                        <template pp-for="device in selectableDevices(audioInputs)">
                                            <SelectItem
                                                key={device.deviceId}
                                                value={device.deviceId}
                                            >
                                                {device.label || "Microphone"}
                                            </SelectItem>
                                        </template>
                                    </Select>
                                    <Switch
                                        class="ml-auto"
                                        attrs=@{named("Microphone")}
                                        checked={microphone}
                                        onchange={toggleMicrophone()}
                                    />
                                </div>
                                <div class={DEVICE_ROW}>
                                    <Icon name="speaker" class="h-6 w-6 shrink-0" />
                                    <Select
                                        class=@{DEVICE_SELECT}
                                        attrs=@{named("Speakers")}
                                        value={availableDevice(speakerDevice, audioOutputs)}
                                        onfocus={refreshDevices()}
                                        onchange={setSpeakerDevice(target.value)}
                                        disabled={session !== null}
                                    >
                                        <SelectItem value="">
                                            {defaultDeviceName(audioOutputs, "Default speakers")}
                                        </SelectItem>
                                        <template
                                            pp-for="device in selectableDevices(audioOutputs)"
                                        >
                                            <SelectItem
                                                key={device.deviceId}
                                                value={device.deviceId}
                                            >
                                                {device.label || "Speaker"}
                                            </SelectItem>
                                        </template>
                                    </Select>
                                </div>
                                <Button
                                    variant="link"
                                    class="mb-0.5 ml-auto mt-4 flex! h-auto! w-fit px-0! text-[15px] text-link min-[851px]:mt-2.5"
                                    onclick={testSpeaker()}
                                >
                                    "Test speaker"
                                </Button>
                            </div>
                        </div>
                        <label
                            class={twMerge(AUDIO_CARD + " " + AUDIO_CHOICE + " cursor-not-allowed text-subtle")}
                            title="Phone audio is not available"
                        >
                            <RadioGroupItem
                                name="audio-mode"
                                value="phone"
                                class="size-6"
                                disabled
                            />
                            "Phone audio"
                        </label>
                        <label class={AUDIO_CARD + " " + AUDIO_CHOICE}>
                            <RadioGroupItem
                                name="audio-mode"
                                value="none"
                                class="size-6"
                                checked={audioMode === "none"}
                                onchange={setAudioMode("none")}
                                disabled={session !== null}
                            />
                            "Don't use audio"
                        </label>
                    </RadioGroup>
                </div>
                <audio pp-ref={meetingAudio} class="hidden"></audio>
                <div
                    class="mt-7 flex shrink-0 justify-end gap-3 min-[851px]:mt-0"
                    hidden={session !== null}
                >
                    <Button variant="outline" onclick={cancelSetup()}>"Cancel"</Button>
                    <Button
                        size="lg"
                        class="meeting-primary min-w-[118px] px-[22px]! text-base font-semibold"
                        disabled={!providerReady || joining || lobbyWaiting || applyingBackground || displayName.trim().length === 0}
                        onclick={joinNow()}
                    >
                        {lobbyWaiting ? "Waiting for admission…" : joining ? "Joining…" : "Join now"}
                    </Button>
                </div>
                <div
                    class="meeting-lobby-wait mt-4 rounded-lg border border-edge bg-raised p-4 text-sm"
                    hidden={!lobbyWaiting}
                    role="status"
                >
                    <strong>"Waiting in the lobby"</strong>
                    <p class="mt-2 text-muted-foreground">
                        "The organizer will let you in. Your microphone and camera are not being sent to the meeting."
                    </p>
                    <Button
                        variant="link"
                        class="mt-3 h-auto! px-0! text-link"
                        onclick={cancelLobby()}
                    >
                        "Cancel request"
                    </Button>
                </div>
                <p
                    class="mt-4 shrink-0 text-right text-sm text-danger-ink min-[851px]:mt-0"
                    role="alert"
                    hidden={!error}
                >
                    {error}
                </p>
                <p
                    class="mt-4 shrink-0 text-right text-[13px] text-muted-foreground min-[851px]:mt-0"
                    hidden={providerReady}
                >
                    "This meeting is unavailable right now. Please contact the organizer."
                </p>
                <footer
                    class="meeting-footer mt-9 flex shrink-0 flex-wrap items-center justify-center gap-6 text-xs text-subtle max-[480px]:gap-2.5 min-[851px]:mt-auto min-[851px]:flex-nowrap"
                >
                    <Button
                        variant="link"
                        size="sm"
                        class="h-auto! px-0! text-xs text-link"
                        onclick={share()}
                    >
                        "Copy meeting link"
                    </Button>
                    <span>"Meeting ID: "@{&code}</span>
                </footer>
            </div>
            <section
                class="meeting-room absolute inset-0 flex flex-col overflow-hidden bg-pane"
                aria-label="Meeting room"
                hidden={session === null}
                onpointerdown={dismissRoomMenusOutside(event)}
            >
                <header
                    pp-ref={roomToolbar}
                    class="relative z-3 flex h-[72px] shrink-0 items-center justify-between gap-6 border-b border-line bg-raised px-5 py-1 max-[1000px]:gap-2 max-[1000px]:px-2.5 max-[650px]:h-auto max-[650px]:min-h-16 max-[650px]:items-start"
                >
                    <div
                        class="flex shrink-0 items-center gap-3 text-sm tabular-nums text-muted-foreground max-[1000px]:gap-1.5 max-[1000px]:text-xs max-[650px]:pt-[18px]"
                    >
                        <Icon name="shield" class="h-5 w-5" />
                        <span>{elapsedLabel}</span>
                        <span
                            class="room-recording-badge flex items-center gap-1.5 rounded-full bg-destructive/25 px-2.5 py-1 text-xs font-semibold text-danger-ink"
                            role="status"
                            hidden={!recordingLive}
                        >
                            <span class="h-2 w-2 animate-pulse rounded-full bg-danger-ink"></span>
                            "Recording"
                        </span>
                    </div>
                    <div
                        class="flex items-center gap-1.5 max-[1000px]:gap-0 max-[650px]:flex-wrap max-[650px]:justify-end"
                    >
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{bound("aria-label", ui_expr!(aria-label={recordingId ? "Stop recording" : "Start recording"}))
                                .merge(&bound("aria-pressed", ui_expr!(aria-pressed={!!recordingId})))
                                .merge(&bound("title", ui_expr!(title={recordTitle})))}
                            disabled={!recordingAllowed || recordingBusy}
                            onclick={toggleRecording()}
                        >
                            <span class={recordingId ? "text-danger-ink" : ""}>
                                <Icon
                                    name="record"
                                    class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5"
                                />
                            </span>
                            <span>
                                {recordingBusy ? "Wait…" : recordingId ? "Stop" : "Record"}
                            </span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{bound("aria-label", ui_expr!(aria-label={chatUnread > 0 ? "Meeting chat, " + chatUnread + " unread" : "Meeting chat"})).merge(&bound("aria-pressed", ui_expr!(aria-pressed={roomPanel === "chat"})))}
                            onclick={toggleRoomPanel("chat")}
                        >
                            <span class="relative">
                                <Icon name="chat" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                                <span
                                    hidden={chatUnread === 0}
                                    class="room-chat-unread absolute -right-2.5 -top-1.5 min-w-4 rounded-full bg-destructive px-1 text-center text-[10px] font-semibold leading-4 text-white"
                                >
                                    {chatUnread > 99 ? "99+" : chatUnread}
                                </span>
                            </span>
                            <span>"Chat"</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Participants").merge(&bound("aria-pressed", ui_expr!(aria-pressed={roomPanel === "people"})))}
                            onclick={toggleRoomPanel("people")}
                        >
                            <Icon name="people" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                            <span>"People "{people.length}</span>
                            <span
                                hidden={lobbyRequests.length === 0}
                                class="rounded-full bg-brand px-1.5 text-[10px] text-white"
                            >
                                {lobbyRequests.length}
                            </span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Raise hand").merge(&bound("aria-pressed", ui_expr!(aria-pressed={handRaised})))}
                            disabled={connectionStatus !== "connected"}
                            onclick={raiseHand()}
                        >
                            <Icon name="hand" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                            <span>{handRaised ? "Lower" : "Raise"}</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Send a reaction")
                                .merge(&bound("aria-expanded", ui_expr!(aria-expanded={reactionsOpen})))}
                            disabled={connectionStatus !== "connected"}
                            onclick={toggleRoomMenu("reactions")}
                        >
                            <span class="contents" pp-ref={reactionTriggerMark}>
                                <Icon
                                    name="emoji"
                                    class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5"
                                />
                            </span>
                            <span>"React"</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Change meeting view").merge(&bound("aria-pressed", ui_expr!(aria-pressed={focusView})))}
                            onclick={setFocusView(!focusView)}
                        >
                            <Icon name="grid" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                            <span>"View"</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("More meeting options")
                                .merge(&bound("aria-expanded", ui_expr!(aria-expanded={moreOpen})))}
                            onclick={toggleRoomMenu("more")}
                        >
                            <span class="contents" pp-ref={moreTriggerMark}>
                                <Icon name="more" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                            </span>
                            <span>"More"</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Toggle meeting camera").merge(&bound("aria-pressed", ui_expr!(aria-pressed={camera})))}
                            disabled={applyingBackground || joining}
                            onclick={toggleCamera()}
                        >
                            <span hidden={!camera}>
                                <Icon
                                    name="video"
                                    class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5"
                                />
                            </span>
                            <span hidden={camera}>
                                <Icon
                                    name="video-off"
                                    class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5"
                                />
                            </span>
                            <span>"Camera"</span>
                        </Button>
                        <div class="room-device-anchor relative flex">
                            <Button
                                variant="ghost"
                                class="room-device-chevron h-[60px]! w-4! min-w-0! px-0! -ml-1 text-muted-foreground hover:text-ink max-[650px]:h-[52px]!"
                                attrs=@{labelled("Camera settings").merge(&bound("aria-expanded", ui_expr!(aria-expanded={deviceMenu === "camera"}))).set("aria-haspopup", "menu")}
                                disabled={!session}
                                onclick={toggleDeviceMenu("camera")}
                            >
                                <Icon name="chevron-down" class="size-3.5" />
                            </Button>
                            <div
                                class="room-device-menu absolute right-0 top-full z-5 mt-1 grid w-80 max-w-[calc(100vw-16px)] grid-cols-[minmax(0,1fr)] gap-0.5 rounded-md border bg-popover p-1.5 shadow-xl shadow-black/40 room-camera-menu"
                                role="menu"
                                aria-label="Camera"
                                hidden={deviceMenu !== "camera"}
                            >
                                <p
                                    class="px-3 pb-1 pt-2 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground"
                                >
                                    "Camera"
                                </p>
                                <button
                                    type="button"
                                    role="menuitemradio"
                                    class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                    aria-checked={!availableDevice(cameraDevice, videoInputs)}
                                    value=""
                                    hidden={selectableDevices(videoInputs).length > 0}
                                    disabled={changingDevice}
                                    onclick={changeCamera(event.currentTarget.value)}
                                >
                                    <span
                                        class="w-4 shrink-0 text-brand-ink"
                                        hidden={!!availableDevice(cameraDevice, videoInputs)}
                                    >
                                        <Icon name="check" class="size-4" />
                                    </span>
                                    <span
                                        class="w-4 shrink-0"
                                        hidden={!availableDevice(cameraDevice, videoInputs)}
                                    >
                                    </span>
                                    <span class="min-w-0 flex-1 truncate">
                                        {defaultDeviceName(videoInputs, "Default camera")}
                                    </span>
                                </button>
                                <template pp-for="device in selectableDevices(videoInputs)">
                                    <button
                                        key={"camera|" + device.deviceId + "|" + (effectiveDevice(cameraDevice, videoInputs) === device.deviceId)}
                                        type="button"
                                        role="menuitemradio"
                                        class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                        aria-checked={effectiveDevice(cameraDevice, videoInputs) === device.deviceId}
                                        value={device.deviceId}
                                        title={device.label}
                                        disabled={changingDevice}
                                        onclick={changeCamera(event.currentTarget.value)}
                                    >
                                        <span
                                            class="w-4 shrink-0 text-brand-ink"
                                            hidden={effectiveDevice(cameraDevice, videoInputs) !== device.deviceId}
                                        >
                                            <Icon name="check" class="size-4" />
                                        </span>
                                        <span
                                            class="w-4 shrink-0"
                                            hidden={effectiveDevice(cameraDevice, videoInputs) === device.deviceId}
                                        >
                                        </span>
                                        <span class="min-w-0 flex-1 truncate">
                                            {device.label || "Camera"}
                                        </span>
                                    </button>
                                </template>
                                <div class="my-1 border-t border-line"></div>
                                <button
                                    type="button"
                                    role="menuitem"
                                    class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                    onclick={openRoomBackgrounds()}
                                >
                                    <Icon name="sparkle" class="size-4" />
                                    "Background effects"
                                </button>
                                <div class="my-1 border-t border-line"></div>
                                <button
                                    type="button"
                                    role="menuitem"
                                    class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                    onclick={openDeviceSettings()}
                                >
                                    <Icon name="settings" class="size-4" />
                                    "More device settings"
                                </button>
                            </div>
                        </div>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Toggle meeting microphone").merge(&bound("aria-pressed", ui_expr!(aria-pressed={microphone})))}
                            disabled={audioMode === "none"}
                            onclick={toggleMicrophone()}
                        >
                            <span hidden={!microphone}>
                                <Icon name="mic" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                            </span>
                            <span hidden={microphone}>
                                <Icon
                                    name="mic-off"
                                    class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5"
                                />
                            </span>
                            <span>"Mic"</span>
                        </Button>
                        <div class="room-device-anchor relative flex">
                            <Button
                                variant="ghost"
                                class="room-device-chevron h-[60px]! w-4! min-w-0! px-0! -ml-1 text-muted-foreground hover:text-ink max-[650px]:h-[52px]!"
                                attrs=@{labelled("Audio settings").merge(&bound("aria-expanded", ui_expr!(aria-expanded={deviceMenu === "audio"}))).set("aria-haspopup", "menu")}
                                disabled={!session}
                                onclick={toggleDeviceMenu("audio")}
                            >
                                <Icon name="chevron-down" class="size-3.5" />
                            </Button>
                            <div
                                class="room-device-menu absolute right-0 top-full z-5 mt-1 grid w-80 max-w-[calc(100vw-16px)] grid-cols-[minmax(0,1fr)] gap-0.5 rounded-md border bg-popover p-1.5 shadow-xl shadow-black/40 room-audio-menu"
                                role="menu"
                                aria-label="Audio"
                                hidden={deviceMenu !== "audio"}
                            >
                                <div class="grid gap-2 p-2" hidden={audioMode !== "none"}>
                                    <p class="text-[13px] text-muted-foreground">
                                        "You joined without computer audio."
                                    </p>
                                    <Button
                                        class="w-full"
                                        disabled={changingDevice}
                                        onclick={useComputerAudio()}
                                    >
                                        "Use computer audio"
                                    </Button>
                                </div>
                                <div class="grid min-w-0 gap-0.5" hidden={audioMode === "none"}>
                                    <p
                                        class="px-3 pb-1 pt-2 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Microphone"
                                    </p>
                                    <button
                                        type="button"
                                        role="menuitemradio"
                                        class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                        aria-checked={!availableDevice(microphoneDevice, audioInputs)}
                                        value=""
                                        hidden={selectableDevices(audioInputs).length > 0}
                                        disabled={changingDevice}
                                        onclick={changeMicrophone(event.currentTarget.value)}
                                    >
                                        <span
                                            class="w-4 shrink-0 text-brand-ink"
                                            hidden={!!availableDevice(microphoneDevice, audioInputs)}
                                        >
                                            <Icon name="check" class="size-4" />
                                        </span>
                                        <span
                                            class="w-4 shrink-0"
                                            hidden={!availableDevice(microphoneDevice, audioInputs)}
                                        >
                                        </span>
                                        <span class="min-w-0 flex-1 truncate">
                                            {defaultDeviceName(audioInputs, "Default microphone")}
                                        </span>
                                    </button>
                                    <template pp-for="device in selectableDevices(audioInputs)">
                                        <button
                                            key={"mic|" + device.deviceId + "|" + (effectiveDevice(microphoneDevice, audioInputs) === device.deviceId)}
                                            type="button"
                                            role="menuitemradio"
                                            class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                            aria-checked={effectiveDevice(microphoneDevice, audioInputs) === device.deviceId}
                                            value={device.deviceId}
                                            title={device.label}
                                            disabled={changingDevice}
                                            onclick={changeMicrophone(event.currentTarget.value)}
                                        >
                                            <span
                                                class="w-4 shrink-0 text-brand-ink"
                                                hidden={effectiveDevice(microphoneDevice, audioInputs) !== device.deviceId}
                                            >
                                                <Icon name="check" class="size-4" />
                                            </span>
                                            <span
                                                class="w-4 shrink-0"
                                                hidden={effectiveDevice(microphoneDevice, audioInputs) === device.deviceId}
                                            >
                                            </span>
                                            <span class="min-w-0 flex-1 truncate">
                                                {device.label || "Microphone"}
                                            </span>
                                        </button>
                                    </template>
                                    <p
                                        class="px-3 pb-1 pt-2 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Speaker"
                                    </p>
                                    <button
                                        type="button"
                                        role="menuitemradio"
                                        class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                        aria-checked={!availableDevice(speakerDevice, audioOutputs)}
                                        value=""
                                        hidden={selectableDevices(audioOutputs).length > 0}
                                        disabled={changingDevice}
                                        onclick={changeSpeaker(event.currentTarget.value)}
                                    >
                                        <span
                                            class="w-4 shrink-0 text-brand-ink"
                                            hidden={!!availableDevice(speakerDevice, audioOutputs)}
                                        >
                                            <Icon name="check" class="size-4" />
                                        </span>
                                        <span
                                            class="w-4 shrink-0"
                                            hidden={!availableDevice(speakerDevice, audioOutputs)}
                                        >
                                        </span>
                                        <span class="min-w-0 flex-1 truncate">
                                            {defaultDeviceName(audioOutputs, "Default speakers")}
                                        </span>
                                    </button>
                                    <template pp-for="device in selectableDevices(audioOutputs)">
                                        <button
                                            key={"speaker|" + device.deviceId + "|" + (effectiveDevice(speakerDevice, audioOutputs) === device.deviceId)}
                                            type="button"
                                            role="menuitemradio"
                                            class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                            aria-checked={effectiveDevice(speakerDevice, audioOutputs) === device.deviceId}
                                            value={device.deviceId}
                                            title={device.label}
                                            disabled={changingDevice}
                                            onclick={changeSpeaker(event.currentTarget.value)}
                                        >
                                            <span
                                                class="w-4 shrink-0 text-brand-ink"
                                                hidden={effectiveDevice(speakerDevice, audioOutputs) !== device.deviceId}
                                            >
                                                <Icon name="check" class="size-4" />
                                            </span>
                                            <span
                                                class="w-4 shrink-0"
                                                hidden={effectiveDevice(speakerDevice, audioOutputs) === device.deviceId}
                                            >
                                            </span>
                                            <span class="min-w-0 flex-1 truncate">
                                                {device.label || "Speaker"}
                                            </span>
                                        </button>
                                    </template>
                                    <button
                                        type="button"
                                        role="menuitem"
                                        class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50 text-link"
                                        onclick={testSpeaker()}
                                    >
                                        <Icon name="speaker" class="size-4" />
                                        "Test speaker"
                                    </button>
                                </div>
                                <div class="my-1 border-t border-line"></div>
                                <button
                                    type="button"
                                    role="menuitem"
                                    class="flex w-full min-w-0 items-center gap-2.5 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover disabled:opacity-50"
                                    onclick={openDeviceSettings()}
                                >
                                    <Icon name="settings" class="size-4" />
                                    "More device settings"
                                </button>
                            </div>
                        </div>
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Share screen").merge(&bound("aria-pressed", ui_expr!(aria-pressed={sharingScreen}))).merge(&bound("aria-disabled", ui_expr!(aria-disabled={!canShareScreen()})))}
                            disabled={connectionStatus !== "connected" || startingShare}
                            onclick={toggleScreenShare()}
                        >
                            <Icon
                                name="share-screen"
                                class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5"
                            />
                            <span>{sharingScreen ? "Stop share" : "Share"}</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{room_leave.as_str()}
                            attrs=@{named("Leave meeting")}
                            onclick={leave()}
                        >
                            <Icon name="leave" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
                            <span>"Leave"</span>
                        </Button>
                    </div>
                    <div
                        class="room-reactions fixed z-5 flex w-max gap-1 overflow-auto rounded-md border bg-popover p-2 shadow-xl shadow-black/40"
                        style={roomMenuStyle}
                        pp-ref={reactionMenu}
                        hidden={!reactionsOpen}
                    >
                        <Button
                            variant="ghost"
                            size="icon-lg"
                            class="size-12! text-[26px]"
                            attrs=@{labelled("Send thumbs up")}
                            onclick={sendReaction("👍")}
                        >
                            "👍"
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon-lg"
                            class="size-12! text-[26px]"
                            attrs=@{labelled("Send heart")}
                            onclick={sendReaction("❤️")}
                        >
                            "❤️"
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon-lg"
                            class="size-12! text-[26px]"
                            attrs=@{labelled("Send applause")}
                            onclick={sendReaction("👏")}
                        >
                            "👏"
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon-lg"
                            class="size-12! text-[26px]"
                            attrs=@{labelled("Send laugh")}
                            onclick={sendReaction("😄")}
                        >
                            "😄"
                        </Button>
                    </div>
                    <div
                        class="room-more-menu fixed z-5 grid w-max gap-1 overflow-auto rounded-md border bg-popover p-2 shadow-xl shadow-black/40"
                        style={roomMenuStyle}
                        pp-ref={moreMenu}
                        hidden={!moreOpen}
                    >
                        <Button variant="ghost" class=@{MENU_ROW} onclick={openRoomBackgrounds()}>
                            "Background settings"
                        </Button>
                        <Button variant="ghost" class=@{MENU_ROW} onclick={openDeviceSettings()}>
                            "Device settings"
                        </Button>
                        <Button variant="ghost" class=@{MENU_ROW} onclick={copyRoomLink()}>
                            "Copy meeting link"
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{MENU_ROW}
                            hidden={!isOrganizer}
                            onclick={openRecordings()}
                        >
                            "Recordings"
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{MENU_ROW}
                            hidden={!isOrganizer || people.length < 2}
                            disabled={mutingAll || unmutedOthers === 0}
                            onclick={muteEveryone()}
                        >
                            {unmutedOthers === 0 ? "Everyone is muted" : "Mute all participants"}
                        </Button>
                        <Button variant="ghost" class=@{MENU_ROW} onclick={setHideSelf(!hideSelf)}>
                            {hideSelf ? "Show me in gallery" : "Hide me from gallery"}
                        </Button>
                    </div>
                </header>
                <div
                    class="flex min-h-0 flex-1 min-[1101px]:data-[backgrounds=true]:mr-[340px]"
                    data-backgrounds={backgroundsOpen}
                >
                    <main
                        class="room-stage relative min-h-0 min-w-0 flex-1 overflow-hidden"
                        data-layout={hasContent || !!featuredId}
                        pp-style={"--room-rows: " + railRows}
                    >
                        <section
                            class="room-hand-auto-lower absolute left-1/2 top-3 z-5 w-[min(360px,90%)] -translate-x-1/2 rounded-lg border border-edge bg-raised p-4 text-sm shadow-xl"
                            role="status"
                            hidden={!handRaised || handLowerAt === 0}
                            aria-label="Automatic hand lowering"
                        >
                            <div class="flex items-start gap-2">
                                <Icon name="hand" class="h-5 w-5 shrink-0 text-away" />
                                <strong class="flex-1">
                                    "Your hand will lower in "
                                    {handLowerSeconds}
                                    " seconds"
                                </strong>
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    class="text-muted-foreground"
                                    attrs=@{labelled("Dismiss and keep hand raised")}
                                    onclick={keepHandRaised()}
                                >
                                    <Icon name="close" class="size-4" />
                                </Button>
                            </div>
                            <p class="my-3 text-muted-foreground">"We noticed you spoke."</p>
                            <Button class="w-full" onclick={keepHandRaised()}>
                                "Keep it raised"
                            </Button>
                        </section>
                        <section
                            class="room-lobby-notice absolute left-1/2 top-3 z-5 w-[min(380px,90%)] -translate-x-1/2 rounded-lg border border-edge bg-raised p-4 shadow-xl"
                            hidden={!isOrganizer || lobbyRequests.length === 0 || roomPanel === "people" || handLowerAt > 0}
                            aria-label="Waiting in the lobby"
                        >
                            <strong class="text-sm">"Waiting in the lobby"</strong>
                            <template pp-for="request in lobbyRequests.slice(0, 1)">
                                <div key={request.id} class="mt-3 text-sm">
                                    <p>{request.name}</p>
                                    <div class="mt-3 flex gap-2">
                                        <Button
                                            variant="outline"
                                            class="flex-1"
                                            value={request.id}
                                            onclick={decideEntry(event.currentTarget.value, false)}
                                        >
                                            "Deny"
                                        </Button>
                                        <Button
                                            class="flex-1"
                                            value={request.id}
                                            onclick={decideEntry(event.currentTarget.value, true)}
                                        >
                                            "Admit"
                                        </Button>
                                    </div>
                                </div>
                            </template>
                        </section>
                        <div
                            class="room-hand-notice absolute left-1/2 top-3 z-4 flex max-w-[90%] -translate-x-1/2 items-center gap-3 rounded-lg border border-edge bg-raised px-4 py-3 text-sm shadow-xl"
                            role="status"
                            hidden={!handNotice || handLowerAt > 0}
                        >
                            <Icon name="hand" class="h-5 w-5 shrink-0 text-away" />
                            <span class="truncate">{handNotice}" raised their hand"</span>
                            <Button
                                variant="ghost"
                                size="icon-xs"
                                class="text-muted-foreground"
                                attrs=@{labelled("Dismiss raised hand notification")}
                                onclick={setHandNotice("")}
                            >
                                <Icon name="close" class="size-4" />
                            </Button>
                        </div>
                        <div
                            class="absolute left-1/2 top-[18px] z-2 -translate-x-1/2 rounded bg-raised px-4 py-2 text-[13px]"
                            role="status"
                            hidden={connectionStatus === "connected"}
                        >
                            {connectionStatus === "reconnecting" ? "Reconnecting to the meeting…" : "Connecting to the meeting…"}
                        </div>
                        <div
                            class={(focusView ? GALLERY_FOCUS : GALLERY) + " room-gallery"}
                            data-presentation={hasContent}
                            data-focus={!!featuredId}
                            pp-style={"--room-rows: " + railRows}
                            hidden={remoteCount === 0 && remotePeople.length === 0}
                        >
                            <template pp-for="slot in remoteSlots">
                                <div
                                    key={slot.index}
                                    class={slot.content ? TILE + " room-content-tile col-span-full" : TILE + " room-person-video"}
                                    data-attendee-id={slot.attendeeId}
                                    data-featured={slot.attendeeId === featuredId}
                                    data-raised={slot.raised}
                                    hidden={!slot.active}
                                >
                                    <video
                                        pp-ref={slot.ref}
                                        class={tileFits[slot.attendeeId] === false ? "h-full w-full object-cover" : "h-full w-full object-contain"}
                                        autoplay
                                        playsinline
                                    >
                                    </video>
                                    <div
                                        class="absolute inset-0 grid place-items-center bg-raised text-center text-sm text-muted-foreground"
                                        hidden={!slot.paused || !slot.content}
                                        role="status"
                                    >
                                        "Screen sharing is paused or unavailable"
                                    </div>
                                    <span
                                        class="room-hand-indicator absolute left-3 top-3 z-1 inline-flex items-center gap-2 rounded-md bg-black/75 px-2.5 py-2 text-xs font-medium text-away"
                                        role="status"
                                        aria-label={slot.name + " has raised their hand"}
                                        hidden={!slot.raised}
                                    >
                                        <Icon name="hand" class="h-4 w-4 shrink-0" />
                                        "Hand raised"
                                    </span>
                                    <span
                                        class="absolute bottom-2.5 left-3 max-w-[calc(100%-24px)] truncate rounded bg-black/60 px-[7px] py-1 text-xs"
                                    >
                                        <span
                                            class="room-muted-badge mr-1.5 inline-flex align-[-3px] text-danger-ink"
                                            role="img"
                                            aria-label={slot.name + " is muted"}
                                            hidden={!slot.muted}
                                        >
                                            <Icon name="mic-off" class="size-3.5" />
                                        </span>
                                        {slot.name}
                                        <Button
                                            variant="ghost"
                                            size="icon-xs"
                                            class="ml-2 size-5! text-white hover:bg-white/15"
                                            attrs=@{bound("aria-label", ui_expr!(aria-label={"Options for " + slot.name}))}
                                            hidden={slot.content}
                                            value={slot.attendeeId}
                                            name={slot.name}
                                            onclick={openTileMenu(event.currentTarget.value, event.currentTarget.name, event.currentTarget)}
                                        >
                                            <Icon name="more" class="size-4" />
                                        </Button>
                                    </span>
                                </div>
                            </template>
                            <template pp-for="person in remotePeople">
                                <div
                                    key={person.id}
                                    class={TILE + " room-person-off flex items-center justify-center"}
                                    data-attendee-id={person.id}
                                    data-featured={person.id === featuredId}
                                    data-raised={person.raised}
                                    hidden={person.hasVideo}
                                >
                                    <span class={AVATAR + " h-[120px] w-[120px] text-[42px]"}>
                                        {person.initials}
                                    </span>
                                    <span
                                        class="room-hand-indicator absolute left-3 top-3 z-1 inline-flex items-center gap-2 rounded-md bg-black/75 px-2.5 py-2 text-xs font-medium text-away"
                                        role="status"
                                        aria-label={person.name + " has raised their hand"}
                                        hidden={!person.raised}
                                    >
                                        <Icon name="hand" class="h-4 w-4 shrink-0" />
                                        "Hand raised"
                                    </span>
                                    <span
                                        class="absolute bottom-2.5 left-3 max-w-[calc(100%-24px)] truncate rounded bg-black/60 px-[7px] py-1 text-xs"
                                    >
                                        <span
                                            class="room-muted-badge mr-1.5 inline-flex align-[-3px] text-danger-ink"
                                            role="img"
                                            aria-label={person.name + " is muted"}
                                            hidden={!person.muted}
                                        >
                                            <Icon name="mic-off" class="size-3.5" />
                                        </span>
                                        {person.name}
                                        <Button
                                            variant="ghost"
                                            size="icon-xs"
                                            class="ml-2 size-5! text-white hover:bg-white/15"
                                            attrs=@{bound("aria-label", ui_expr!(aria-label={"Options for " + person.name}))}
                                            value={person.id}
                                            name={person.name}
                                            onclick={openTileMenu(event.currentTarget.value, event.currentTarget.name, event.currentTarget)}
                                        >
                                            <Icon name="more" class="size-4" />
                                        </Button>
                                    </span>
                                </div>
                            </template>
                        </div>
                        <div
                            class={(remoteCount === 0 && remotePeople.length === 0 ? SELF_ALONE : SELF_TILE) + " room-self-tile"}
                            data-featured={featuredId === selfAttendeeId && !!selfAttendeeId}
                            data-raised={handRaised}
                            hidden={hideSelf}
                        >
                            <video
                                pp-ref={roomLocalVideo}
                                class={tileFits[selfAttendeeId] === false ? "room-local-video absolute inset-0 h-full w-full object-cover" : "room-local-video absolute inset-0 h-full w-full object-contain"}
                                pp-style={selfVideoStyle}
                                hidden={!camera}
                                muted
                                autoplay
                                playsinline
                            >
                            </video>
                            <div
                                class="room-waiting absolute inset-0 flex flex-col items-center justify-center gap-12 max-[650px]:gap-[30px]"
                                hidden={camera}
                            >
                                <span
                                    class={remoteCount === 0 && remotePeople.length === 0 ? AVATAR_ALONE : AVATAR_TILE}
                                >
                                    {myInitials}
                                </span>
                                <h1
                                    class="px-4 text-center text-[clamp(18px,2vw,26px)] font-semibold"
                                    hidden={remoteCount > 0 || remotePeople.length > 0}
                                >
                                    {connectionStatus === "connected" ? "Waiting for others to join…" : "Joining your meeting…"}
                                </h1>
                            </div>
                            <span
                                class="absolute bottom-2.5 left-3 max-w-[calc(100%-24px)] truncate rounded bg-black/60 px-[7px] py-1 text-xs"
                            >
                                <span
                                    class="room-muted-badge mr-1.5 inline-flex align-[-3px] text-danger-ink"
                                    role="img"
                                    aria-label={"You are muted"}
                                    hidden={!micOff(selfAttendeeId)}
                                >
                                    <Icon name="mic-off" class="size-3.5" />
                                </span>
                                {displayName}
                                " (You)"
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    class="ml-2 size-5! text-white hover:bg-white/15"
                                    attrs=@{labelled("Options for your video")}
                                    onclick={openTileMenu(selfAttendeeId, displayName, event.currentTarget)}
                                >
                                    <Icon name="more" class="size-4" />
                                </Button>
                            </span>
                            <span
                                class="room-raised-hand room-hand-indicator absolute left-3 top-3 z-1 inline-flex items-center gap-2 rounded-md bg-black/75 px-2.5 py-2 text-xs font-medium text-away"
                                role="status"
                                aria-label="Your hand is raised"
                                hidden={!handRaised}
                            >
                                <Icon name="hand" class="h-4 w-4 shrink-0" />
                                "Hand raised"
                            </span>
                        </div>
                        <section
                            class="room-tile-menu absolute z-5 grid w-64 gap-1 overflow-y-auto rounded-lg border border-edge bg-raised p-2 text-sm shadow-2xl"
                            pp-style={tileMenu ? tileMenu.style : ""}
                            pp-ref={tileMenuElement}
                            hidden={!tileMenu}
                            aria-label="Participant options"
                        >
                            <div class="truncate border-b border-line px-3 py-2 font-semibold">
                                {tileMenu ? tileMenu.name : ""}
                            </div>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                hidden={!tileMenu || tileMenu.id !== selfAttendeeId}
                                attrs=@{Attrs::new().set("data-action", "switch-camera")}
                                onclick={switchCamera()}
                            >
                                "Switch camera"
                            </Button>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                attrs=@{Attrs::new().set("data-action", "pin")}
                                onclick={pinTile()}
                            >
                                {tileMenu && pinnedId === tileMenu.id ? "Unpin for me" : "Pin for me"}
                            </Button>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                hidden={!devicePreferences.signLanguageMode || !tileMenu || tileMenu.id === selfAttendeeId}
                                attrs=@{Attrs::new().set("data-action", "sign-language")}
                                onclick={pinTile()}
                            >
                                {tileMenu && pinnedId === tileMenu.id ? "Remove from sign language view" : "Keep in sign language view"}
                            </Button>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                attrs=@{Attrs::new().set("data-action", "fit")}
                                onclick={fitTile()}
                            >
                                {tileMenu && tileFits[tileMenu.id] === false ? "Fit to frame" : "Fill frame"}
                            </Button>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                hidden={!tileMenu || tileMenu.id !== selfAttendeeId}
                                attrs=@{Attrs::new().set("data-action", "hide-self")}
                                onclick={hideMyTile()}
                            >
                                "Remove me from gallery"
                            </Button>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                hidden={!isOrganizer}
                                attrs=@{Attrs::new().set("data-action", "spotlight")}
                                onclick={spotlightTile()}
                            >
                                {tileMenu && spotlightId === tileMenu.id ? "Stop spotlight for everyone" : "Spotlight for everyone"}
                            </Button>
                            <Button
                                variant="ghost"
                                class=@{MENU_ROW}
                                hidden={!isOrganizer || !tileMenu || tileMenu.id === selfAttendeeId || micOff(tileMenu.id)}
                                attrs=@{Attrs::new().set("data-action", "mute")}
                                onclick={muteTile()}
                            >
                                "Mute participant"
                            </Button>
                            <Button variant="ghost" class=@{MENU_ROW} onclick={setTileMenu(null)}>
                                "Close"
                            </Button>
                        </section>
                        <p
                            class="room-sign-language-notice absolute left-5 top-3 z-1 max-w-[80%] rounded bg-raised/90 px-3 py-2 text-xs text-ink"
                            hidden={!devicePreferences.signLanguageMode}
                        >
                            "Sign language mode is on. Choose Keep in sign language view from a participant's menu to keep their video prominent."
                        </p>
                        <div
                            class="room-muted-notice absolute bottom-5 left-5 z-2 flex max-w-[90%] items-center gap-3 rounded-md border border-edge bg-raised px-4 py-3 text-sm"
                            role="status"
                            hidden={!mutedNotice}
                        >
                            <Icon name="mic-off" class="size-5" />
                            <span>"You're muted. Turn on your microphone to speak."</span>
                            <button
                                type="button"
                                class="rounded border border-edge px-2 py-1 text-xs"
                                onclick={toggleMicrophone()}
                            >
                                "Unmute"
                            </button>
                            <button
                                type="button"
                                class="rounded p-1"
                                aria-label="Dismiss muted notification"
                                onclick={setMutedNotice(false)}
                            >
                                <Icon name="close" class="size-4" />
                            </button>
                        </div>
                        <div
                            class="room-reaction-toast absolute bottom-16 left-1/2 z-2 -translate-x-1/2 rounded-3xl bg-raised/70 px-6 py-3 text-[28px]"
                            role="status"
                            hidden={!reactionNotice}
                        >
                            {reactionNotice}
                        </div>
                        <p
                            class="absolute inset-x-5 bottom-5 z-2 rounded bg-destructive/20 p-2.5 text-[13px] text-danger-ink"
                            role="alert"
                            hidden={!error}
                        >
                            {error}
                        </p>
                    </main>
                    <aside
                        class="room-side-panel flex min-h-0 w-80 shrink-0 flex-col border-l border-line bg-surface max-[1000px]:w-[280px] max-[650px]:absolute max-[650px]:bottom-0 max-[650px]:right-0 max-[650px]:top-[116px] max-[650px]:z-2 max-[650px]:w-[min(320px,90vw)]"
                        hidden={!roomPanel}
                    >
                        <header
                            class="flex items-center justify-between px-5 py-[22px] text-[17px] font-semibold"
                        >
                            <h2>
                                {roomPanel === "people" ? "People (" + people.length + ")" : roomPanel === "recordings" ? "Recordings" : roomPanel === "devices" ? "Device settings" : "Meeting chat"}
                            </h2>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                attrs=@{labelled("Close meeting panel")}
                                onclick={setRoomPanel("")}
                            >
                                <Icon name="close" class="size-5" />
                            </Button>
                        </header>
                        <div
                            class="room-device-settings grid content-start gap-5 overflow-auto px-5 pb-6 text-[13px]"
                            hidden={roomPanel !== "devices"}
                        >
                            <section class="grid gap-2">
                                <h3
                                    class="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
                                >
                                    "Audio"
                                </h3>
                                <div
                                    class="grid gap-2 rounded-md border border-edge p-3"
                                    hidden={audioMode !== "none"}
                                >
                                    <p class="text-muted-foreground">
                                        "You joined without computer audio, so you can't hear or speak in this meeting."
                                    </p>
                                    <Button
                                        class="room-use-audio w-full"
                                        disabled={changingDevice}
                                        onclick={useComputerAudio()}
                                    >
                                        "Use computer audio"
                                    </Button>
                                </div>
                                <div class="grid gap-3" hidden={audioMode === "none"}>
                                    <label class="grid gap-1.5" for="room-microphone">
                                        <span class="flex items-center gap-2 font-medium">
                                            <Icon name="mic" class="size-4" />
                                            "Microphone"
                                        </span>
                                        <Select
                                            id="room-microphone"
                                            class="w-full text-[13px]"
                                            value={availableDevice(microphoneDevice, audioInputs)}
                                            onfocus={refreshDevices()}
                                            onchange={changeMicrophone(target.value)}
                                            disabled={changingDevice}
                                        >
                                            <SelectItem
                                                value=""
                                                selected={!availableDevice(microphoneDevice, audioInputs)}
                                            >
                                                {defaultDeviceName(audioInputs, "Default microphone")}
                                            </SelectItem>
                                            <template
                                                pp-for="device in selectableDevices(audioInputs)"
                                            >
                                                <SelectItem
                                                    key={device.deviceId + "|" + (availableDevice(microphoneDevice, audioInputs) === device.deviceId)}
                                                    value={device.deviceId}
                                                    selected={availableDevice(microphoneDevice, audioInputs) === device.deviceId}
                                                >
                                                    {device.label || "Microphone"}
                                                </SelectItem>
                                            </template>
                                        </Select>
                                    </label>
                                    <label class="grid gap-1.5" for="room-speaker">
                                        <span class="flex items-center gap-2 font-medium">
                                            <Icon name="speaker" class="size-4" />
                                            "Speaker"
                                        </span>
                                        <Select
                                            id="room-speaker"
                                            class="w-full text-[13px]"
                                            value={availableDevice(speakerDevice, audioOutputs)}
                                            onfocus={refreshDevices()}
                                            onchange={changeSpeaker(target.value)}
                                            disabled={changingDevice}
                                        >
                                            <SelectItem
                                                value=""
                                                selected={!availableDevice(speakerDevice, audioOutputs)}
                                            >
                                                {defaultDeviceName(audioOutputs, "Default speakers")}
                                            </SelectItem>
                                            <template
                                                pp-for="device in selectableDevices(audioOutputs)"
                                            >
                                                <SelectItem
                                                    key={device.deviceId + "|" + (availableDevice(speakerDevice, audioOutputs) === device.deviceId)}
                                                    value={device.deviceId}
                                                    selected={availableDevice(speakerDevice, audioOutputs) === device.deviceId}
                                                >
                                                    {device.label || "Speaker"}
                                                </SelectItem>
                                            </template>
                                        </Select>
                                    </label>
                                    <Button
                                        variant="link"
                                        class="h-auto! w-fit justify-start! px-0! text-[13px] text-link"
                                        onclick={testSpeaker()}
                                    >
                                        "Test speaker"
                                    </Button>
                                </div>
                            </section>
                            <section class="grid gap-2">
                                <h3
                                    class="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
                                >
                                    "Video"
                                </h3>
                                <label class="grid gap-1.5" for="room-camera">
                                    <span class="flex items-center gap-2 font-medium">
                                        <Icon name="video" class="size-4" />
                                        "Camera"
                                    </span>
                                    <Select
                                        id="room-camera"
                                        class="w-full text-[13px]"
                                        value={availableDevice(cameraDevice, videoInputs)}
                                        onfocus={refreshDevices()}
                                        onchange={changeCamera(target.value)}
                                        disabled={changingDevice || applyingBackground}
                                    >
                                        <SelectItem
                                            value=""
                                            selected={!availableDevice(cameraDevice, videoInputs)}
                                        >
                                            {defaultDeviceName(videoInputs, "Default camera")}
                                        </SelectItem>
                                        <template pp-for="device in selectableDevices(videoInputs)">
                                            <SelectItem
                                                key={device.deviceId + "|" + (availableDevice(cameraDevice, videoInputs) === device.deviceId)}
                                                value={device.deviceId}
                                                selected={availableDevice(cameraDevice, videoInputs) === device.deviceId}
                                            >
                                                {device.label || "Camera"}
                                            </SelectItem>
                                        </template>
                                    </Select>
                                </label>
                                <p class="text-xs text-muted-foreground" hidden={camera}>
                                    "Used the next time you turn your camera on."
                                </p>
                                <Button
                                    variant="outline"
                                    class="w-full"
                                    onclick={openRoomBackgrounds()}
                                >
                                    "Background effects"
                                </Button>
                            </section>
                        </div>
                        <div class="overflow-auto px-5" hidden={roomPanel !== "people"}>
                            <section
                                hidden={!isOrganizer || lobbyRequests.length === 0}
                                class="room-lobby-list border-b border-line pb-4"
                            >
                                <h3 class="my-3 text-sm font-semibold">
                                    "Waiting in the lobby ("
                                    {lobbyRequests.length}
                                    ")"
                                </h3>
                                <template pp-for="request in lobbyRequests">
                                    <div key={request.id} class="my-3 text-sm">
                                        <p>{request.name}</p>
                                        <div class="mt-2 flex gap-3">
                                            <Button
                                                variant="link"
                                                size="sm"
                                                class=@{LINK}
                                                value={request.id}
                                                onclick={decideEntry(event.currentTarget.value, true)}
                                            >
                                                "Admit"
                                            </Button>
                                            <Button
                                                variant="link"
                                                size="sm"
                                                class="h-auto! px-0! text-muted-foreground"
                                                value={request.id}
                                                onclick={decideEntry(event.currentTarget.value, false)}
                                            >
                                                "Deny"
                                            </Button>
                                        </div>
                                    </div>
                                </template>
                            </section>
                            <div class="my-3 flex items-center justify-between gap-3">
                                <p class="text-xs text-subtle">"In this meeting"</p>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    class="room-mute-all h-7! gap-1.5 px-2.5! text-xs"
                                    hidden={!isOrganizer || people.length < 2}
                                    disabled={mutingAll || unmutedOthers === 0}
                                    onclick={muteEveryone()}
                                >
                                    <Icon name="mic-off" class="size-3.5" />
                                    {mutingAll ? "Muting…" : unmutedOthers === 0 ? "All muted" : "Mute all"}
                                </Button>
                            </div>
                            <template pp-for="person in peopleRows">
                                <div
                                    key={[person.id, person.mic, person.raised].join("|")}
                                    class="room-person-row my-[18px] flex items-center gap-3 text-[13px]"
                                    data-attendee-id={person.id}
                                    data-muted={person.muted}
                                >
                                    <span
                                        class="grid h-9 w-9 shrink-0 place-items-center rounded-full bg-avatar text-avatar-ink"
                                    >
                                        {person.initials}
                                    </span>
                                    <strong class="flex-1 font-medium wrap-anywhere">
                                        {person.name}
                                        {person.id === selfAttendeeId ? " (You)" : ""}
                                    </strong>
                                    <span hidden={!person.raised}>
                                        <Icon name="hand" class="h-5 w-5" />
                                    </span>
                                    <span
                                        class={person.muted ? "inline-flex text-danger-ink" : "inline-flex text-muted-foreground"}
                                        role="img"
                                        aria-label={person.name + (person.muted ? " is muted" : " has their microphone on")}
                                        title={person.muted ? "Muted" : "Microphone on"}
                                        hidden={!person.mic}
                                    >
                                        <span class="contents" hidden={!person.muted}>
                                            <Icon name="mic-off" class="h-4 w-4" />
                                        </span>
                                        <span class="contents" hidden={person.muted}>
                                            <Icon name="mic" class="h-4 w-4" />
                                        </span>
                                    </span>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        class="h-7! px-2! text-xs text-link"
                                        hidden={!isOrganizer || person.self || person.muted}
                                        value={person.id}
                                        name={person.name}
                                        onclick={muteAttendee(event.currentTarget.value, event.currentTarget.name)}
                                    >
                                        "Mute"
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon-xs"
                                        attrs=@{bound("aria-label", ui_expr!(aria-label={"Options for " + person.name}))}
                                        value={person.id}
                                        name={person.name}
                                        onclick={openTileMenu(event.currentTarget.value, event.currentTarget.name, event.currentTarget)}
                                    >
                                        <Icon name="more" class="size-4" />
                                    </Button>
                                </div>
                            </template>
                            <Button variant="link" class=@{LINK} onclick={share()}>
                                "Copy invite link"
                            </Button>
                        </div>
                        <div class="overflow-auto px-5" hidden={roomPanel !== "recordings"}>
                            <p
                                class="my-3 text-[13px] leading-relaxed text-muted-foreground"
                                hidden={recordings.length > 0}
                            >
                                {recordingsLoading ? "Loading recordings…" : "Recordings of this meeting appear here. They are kept in Dilnaka Storage."}
                            </p>
                            <template pp-for="item in recordings">
                                <div key={item.id} class="my-4 flex items-center gap-3 text-[13px]">
                                    <div class="min-w-0 flex-1">
                                        <strong class="block font-medium">{item.started}</strong>
                                        <span class="text-xs text-muted-foreground">
                                            {recordingSummary(item)}
                                        </span>
                                    </div>
                                    <Button
                                        variant="link"
                                        size="sm"
                                        class=@{LINK}
                                        hidden={item.state !== "stored"}
                                        value={item.id}
                                        onclick={openRecording(event.currentTarget.value)}
                                    >
                                        "Download"
                                    </Button>
                                    <Button
                                        variant="link"
                                        size="sm"
                                        class=@{LINK}
                                        hidden={item.state !== "failed"}
                                        value={item.id}
                                        onclick={retryRecording(event.currentTarget.value)}
                                    >
                                        "Retry"
                                    </Button>
                                </div>
                            </template>
                        </div>
                        <div class="flex min-h-0 flex-1 flex-col" hidden={roomPanel !== "chat"}>
                            <div
                                class="room-chat-messages flex min-h-0 flex-1 flex-col gap-3.5 overflow-auto px-[18px]"
                                aria-live="polite"
                                pp-ref={chatLog}
                            >
                                <p
                                    class="text-[13px] leading-relaxed text-muted-foreground"
                                    hidden={chatMessages.length > 0}
                                >
                                    "Chat with everyone in this meeting. Messages stay with this meeting link."
                                </p>
                                <template pp-for="message in chatMessages">
                                    <article
                                        key={message.id}
                                        class="rounded-md bg-raised p-3 text-[13px] wrap-anywhere"
                                    >
                                        <div class="flex items-baseline justify-between gap-2">
                                            <strong class="min-w-0 truncate text-xs text-link">
                                                {message.name}
                                            </strong>
                                            <time
                                                class="shrink-0 text-[11px] text-muted-foreground"
                                            >
                                                {chatTime(message.at)}
                                            </time>
                                        </div>
                                        <p
                                            class="mt-[5px] whitespace-pre-wrap"
                                            hidden={!message.text}
                                        >
                                            {message.text}
                                        </p>
                                        <button
                                            type="button"
                                            class="room-chat-file mt-2 flex w-full items-center gap-2.5 rounded-md border border-edge bg-surface px-3 py-2 text-left hover:bg-hover"
                                            hidden={!message.file}
                                            value={message.id}
                                            title={message.file ? "Download " + message.file.name : ""}
                                            onclick={downloadChatFile(event.currentTarget.value)}
                                        >
                                            <span class="shrink-0 text-brand-ink">
                                                <Icon name="file" class="size-5" />
                                            </span>
                                            <span class="grid min-w-0 flex-1">
                                                <span class="truncate font-medium">
                                                    {message.file ? message.file.name : ""}
                                                </span>
                                                <span class="text-[11px] text-muted-foreground">
                                                    {message.file ? fileSize(message.file.bytes) : ""}
                                                </span>
                                            </span>
                                            <span class="shrink-0 text-muted-foreground">
                                                <Icon name="arrow-down" class="size-4" />
                                            </span>
                                        </button>
                                    </article>
                                </template>
                            </div>
                            <div
                                class="room-chat-pending mx-4 flex items-center gap-2 rounded-md border border-edge bg-raised px-3 py-2 text-[13px]"
                                hidden={!chatFile && !chatUploading}
                            >
                                <span class="shrink-0 text-brand-ink">
                                    <Icon name="attach" class="size-4" />
                                </span>
                                <span class="min-w-0 flex-1 truncate">
                                    {chatUploading ? "Uploading " + chatUploading + "…" : chatFile ? chatFile.name : ""}
                                </span>
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    attrs=@{labelled("Remove attachment")}
                                    hidden={!chatFile}
                                    onclick={setChatFile(null)}
                                >
                                    <Icon name="close" class="size-3.5" />
                                </Button>
                            </div>
                            <form
                                class="room-chat-composer relative m-4 flex items-center gap-0.5 rounded-md border border-edge bg-raised py-1 pl-3 pr-1 focus-within:border-ring"
                                onsubmit={sendChat(event)}
                            >
                                <div
                                    class="room-chat-attach-menu absolute bottom-full right-0 z-5 mb-2 grid w-max gap-1 rounded-md border bg-popover p-2 shadow-xl shadow-black/40"
                                    hidden={!chatAttachMenu}
                                >
                                    <Button
                                        variant="ghost"
                                        class=@{MENU_ROW}
                                        attrs=@{plain()}
                                        onclick={chooseChatUpload()}
                                    >
                                        <Icon name="upload" class="size-4" />
                                        "Upload from this device"
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        class=@{MENU_ROW}
                                        attrs=@{plain()}
                                        onclick={openChatPicker()}
                                    >
                                        <Icon name="folder" class="size-4" />
                                        "Choose from My files"
                                    </Button>
                                </div>
                                <input
                                    type="file"
                                    hidden
                                    aria-label="Choose a file to share"
                                    pp-ref={chatFileInput}
                                    onchange={uploadChatFile(event)}
                                />
                                <Input
                                    placeholder="Type a message"
                                    class="h-8! min-w-0 flex-1 border-0! bg-transparent! px-0! text-[13px] shadow-none! focus-visible:ring-0! dark:bg-transparent!"
                                    attrs=@{named("Message everyone").set("maxlength", "500")}
                                    value={chatDraft}
                                    oninput={setChatDraft(target.value)}
                                />
                                <Button
                                    variant="ghost"
                                    size="icon-sm"
                                    class="shrink-0 text-muted-foreground hover:text-brand-ink"
                                    attrs=@{labelled("Attach a file").merge(&plain())}
                                    hidden={anonymous}
                                    disabled={!!chatUploading || connectionStatus !== "connected"}
                                    onclick={setChatAttachMenu(!chatAttachMenu)}
                                >
                                    <Icon name="attach" class="size-[18px]" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon-sm"
                                    class="shrink-0 text-brand-ink disabled:text-muted-foreground"
                                    attrs=@{labelled("Send meeting message").merge(&submit())}
                                    disabled={(!chatDraft.trim() && !chatFile) || !!chatUploading || connectionStatus !== "connected"}
                                >
                                    <Icon name="send" class="size-[18px]" />
                                </Button>
                            </form>
                        </div>
                    </aside>
                </div>
            </section>
            <aside
                class="meeting-background-panel fixed inset-y-0 right-0 z-5 flex w-[340px] flex-col border-l border-line bg-surface max-[1100px]:w-[min(340px,92vw)] max-[1100px]:shadow-2xl max-[1100px]:shadow-black/40"
                aria-label="Background settings"
                pp-ref={backgroundPanel}
                style={session ? "top:" + toolbarBottom + "px" : ""}
                onpointerdown={dismissRoomMenus()}
                hidden={!backgroundsOpen}
            >
                <header class="flex items-center justify-between gap-3 px-5 py-6">
                    <h2 class="text-lg font-semibold">"Background settings"</h2>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        class="text-muted-foreground"
                        attrs=@{labelled("Close background settings")}
                        onclick={setBackgroundsOpen(false)}
                    >
                        <Icon name="close" class="size-5" />
                    </Button>
                </header>
                <div class="min-h-0 overflow-y-auto px-5 pb-6 pt-2">
                    <div class="grid grid-cols-3 gap-2">
                        <Button
                            variant="ghost"
                            class=@{BACKGROUND_OPTION}
                            attrs=@{bound("aria-pressed", ui_expr!(aria-pressed={backgroundId === "none"}))}
                            disabled={applyingBackground || joining}
                            onclick={selectBackground("none")}
                        >
                            <span class="text-[25px] leading-7">"⊘"</span>
                            <span>"None"</span>
                        </Button>
                        <Button
                            variant="ghost"
                            class=@{BACKGROUND_OPTION}
                            attrs=@{bound("aria-pressed", ui_expr!(aria-pressed={backgroundId === "blur"}))}
                            disabled={applyingBackground || joining}
                            onclick={selectBackground("blur")}
                        >
                            <span class="text-[25px] leading-7">"≋"</span>
                            <span>"Standard blur"</span>
                        </Button>
                        <label class=@{upload_background.as_str()}>
                            <Icon name="plus" class="size-6" />
                            <span>"Add new"</span>
                            <Input
                                type="file"
                                class="absolute inset-0 h-full! cursor-pointer opacity-0"
                                attrs=@{named("Upload background image").set("accept", "image/jpeg,image/png,image/webp")}
                                disabled={applyingBackground || joining}
                                onchange={uploadBackground(event)}
                            />
                        </label>
                        <template pp-for="background in backgrounds">
                            <Button
                                key={background.id}
                                variant="ghost"
                                class=@{BACKGROUND_OPTION}
                                attrs=@{bound("aria-label", ui_expr!(aria-label={background.name}))
                                    .merge(&bound("aria-pressed", ui_expr!(aria-pressed={backgroundId === background.id})))}
                                disabled={applyingBackground || joining}
                                value={background.id}
                                onclick={selectBackground(event.currentTarget.value)}
                            >
                                <img
                                    src={background.url}
                                    alt=""
                                    class="absolute inset-0 h-full w-full object-cover"
                                />
                                <span
                                    class="absolute inset-x-0 bottom-0 bg-black/60 px-0.5 py-[3px] text-center"
                                >
                                    {background.name}
                                </span>
                            </Button>
                        </template>
                    </div>
                    <p class="mt-[18px] text-xs leading-relaxed text-subtle" role="status">
                        {applyingBackground ? "Applying background…" : camera ? "Your selected background is applied to your camera and meeting video." : "Turn on your camera to preview your background."}
                    </p>
                    <p
                        class="meeting-background-error mt-3.5 text-[13px] leading-normal text-danger-ink"
                        role="alert"
                        hidden={!backgroundError}
                    >
                        {backgroundError}
                    </p>
                    <p class="mt-[18px] text-xs leading-relaxed text-subtle">
                        "Custom images stay in this tab. Camera processing happens on your device."
                    </p>
                </div>
            </aside>
            <NotificationSound />
            <div
                class="room-toast pointer-events-none fixed left-1/2 top-[88px] z-[60] w-max max-w-[min(560px,calc(100vw-32px))] -translate-x-1/2 rounded-md border border-edge bg-raised px-5 py-3.5 text-center text-sm text-ink shadow-xl shadow-black/40 max-[650px]:top-auto max-[650px]:bottom-24"
                role="status"
                aria-live="polite"
                hidden={!toast}
            >
                {toast}
            </div>
            <Dialog>
                <DialogTrigger attrs=@{hidden()}>
                    <span pp-ref={pickerOpener}></span>
                </DialogTrigger>
                <DialogClose attrs=@{hidden()}>
                    <span pp-ref={pickerCloser}></span>
                </DialogClose>
                <DialogContent
                    class="room-chat-picker grid h-[min(560px,calc(100dvh-32px))] grid-rows-[auto_auto_minmax(0,1fr)_auto] gap-0! p-0! sm:max-w-[640px]!"
                    onclose={closeChatPicker()}
                >
                    <DialogHeader class="px-6 pb-2 pt-6">
                        <DialogTitle class="text-[17px]">"Share a file from My files"</DialogTitle>
                    </DialogHeader>
                    <div
                        class="flex min-w-0 flex-wrap items-center gap-1 border-b border-line px-6 pb-3 text-[13px]"
                        aria-label="Folder path"
                    >
                        <button
                            type="button"
                            class={pickerListing.folder ? "rounded px-1.5 py-0.5 text-link hover:bg-hover" : "rounded px-1.5 py-0.5 font-semibold"}
                            onclick={openPickerFolder("")}
                        >
                            "My files"
                        </button>
                        <template pp-for="crumb in pickerListing.path">
                            <span key={crumb.id} class="flex items-center gap-1">
                                <span class="text-subtle">"/"</span>
                                <button
                                    type="button"
                                    class={crumb.id === pickerListing.folder ? "max-w-40 truncate rounded px-1.5 py-0.5 font-semibold" : "max-w-40 truncate rounded px-1.5 py-0.5 text-link hover:bg-hover"}
                                    value={crumb.id}
                                    onclick={openPickerFolder(event.currentTarget.value)}
                                >
                                    {crumb.name}
                                </button>
                            </span>
                        </template>
                    </div>
                    <div
                        class="min-h-0 overflow-auto px-3 py-2"
                        role="listbox"
                        aria-label="My files"
                    >
                        <div
                            class="room-picker-loading flex h-full min-h-32 flex-col items-center justify-center gap-3 text-[13px] text-muted-foreground"
                            role="status"
                            hidden={!pickerLoading}
                        >
                            <Spinner class="size-5" />
                            "Loading your files…"
                        </div>
                        <p class="px-3 py-3 text-[13px] text-danger-ink" hidden={!pickerError}>
                            {pickerError}
                        </p>
                        <div class="room-picker-list" hidden={pickerLoading}>
                            <div
                                class="grid grid-cols-[minmax(0,1fr)_120px_80px] gap-3 px-3 pb-1 pt-1 text-[11px] uppercase tracking-wide text-muted-foreground max-[520px]:grid-cols-[minmax(0,1fr)_70px]"
                                hidden={pickerListing.folders.length + pickerListing.files.length === 0}
                            >
                                <span>"Name"</span>
                                <span class="max-[520px]:hidden">"Modified"</span>
                                <span class="text-right">"Size"</span>
                            </div>
                            <template pp-for="folder in pickerListing.folders">
                                <button
                                    key={"folder|" + folder.id}
                                    type="button"
                                    class="room-picker-folder grid w-full grid-cols-[minmax(0,1fr)_120px_80px] items-center gap-3 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover max-[520px]:grid-cols-[minmax(0,1fr)_70px]"
                                    value={folder.id}
                                    title={"Open " + folder.name}
                                    onclick={openPickerFolder(event.currentTarget.value)}
                                >
                                    <span class="flex min-w-0 items-center gap-2.5">
                                        <span class="shrink-0 text-brand-ink">
                                            <Icon name="folder" class="size-5 fill-brand-ink/20" />
                                        </span>
                                        <span class="truncate font-medium">{folder.name}</span>
                                    </span>
                                    <span class="truncate text-muted-foreground max-[520px]:hidden">
                                        {folder.modified}
                                    </span>
                                    <span class="text-right text-muted-foreground">
                                        {folder.items === 1 ? "1 item" : folder.items + " items"}
                                    </span>
                                </button>
                            </template>
                            <template pp-for="file in pickerListing.files">
                                <button
                                    key={"file|" + file.id + "|" + (pickerChoice === file.id)}
                                    type="button"
                                    role="option"
                                    aria-selected={pickerChoice === file.id}
                                    class={pickerChoice === file.id ? "room-picker-file grid w-full grid-cols-[minmax(0,1fr)_120px_80px] items-center gap-3 rounded-md bg-brand/15 px-3 py-2 text-left text-[13px] ring-1 ring-brand-ink max-[520px]:grid-cols-[minmax(0,1fr)_70px]" : "room-picker-file grid w-full grid-cols-[minmax(0,1fr)_120px_80px] items-center gap-3 rounded-md px-3 py-2 text-left text-[13px] hover:bg-hover max-[520px]:grid-cols-[minmax(0,1fr)_70px]"}
                                    value={file.id}
                                    title={file.name}
                                    onclick={choosePickerFile(event.currentTarget.value)}
                                    ondblclick={attachPickerFile(event.currentTarget.value)}
                                >
                                    <span class="flex min-w-0 items-center gap-2.5">
                                        <span class="shrink-0 text-muted-foreground">
                                            <Icon name="file" class="size-5" />
                                        </span>
                                        <span class="truncate">{file.name}</span>
                                    </span>
                                    <span class="truncate text-muted-foreground max-[520px]:hidden">
                                        {file.uploaded}
                                    </span>
                                    <span class="text-right text-muted-foreground">
                                        {fileSize(file.bytes)}
                                    </span>
                                </button>
                            </template>
                            <p
                                class="px-3 py-6 text-center text-[13px] text-muted-foreground"
                                hidden={!!pickerError || pickerListing.folders.length + pickerListing.files.length > 0}
                            >
                                "This folder is empty."
                            </p>
                        </div>
                    </div>
                    <DialogFooter class="items-center border-t border-line px-6 py-4">
                        <span class="mr-auto min-w-0 truncate text-[13px] text-muted-foreground">
                            {pickerChoiceName()}
                        </span>
                        <DialogClose class=@{PICKER_CANCEL}>"Cancel"</DialogClose>
                        <Button
                            class="room-picker-attach min-w-24"
                            disabled={!pickerChoice}
                            onclick={attachPickerFile(pickerChoice)}
                        >
                            "Attach"
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
            <script>r##"

                    const devicePreferences = pp.memo(() => {
                        let saved = {};
                        try { saved = JSON.parse(localStorage.getItem('upurnaka.settings.v1.' + @{user.id.as_str()}) || '{}') || {}; } catch (_) {}
                        return {
                            microphoneId:typeof saved.microphoneId === 'string' ? saved.microphoneId : '',
                            notifySound:saved.notifySound !== false,
                            speakerId:typeof saved.speakerId === 'string' ? saved.speakerId : '',
                            cameraId:typeof saved.cameraId === 'string' ? saved.cameraId : '',
                            autoGainControl:saved.autoGainControl !== false,
                            noiseSuppression:saved.noiseSuppression !== false,
                            mirrorVideo:saved.mirrorVideo !== false,
                            videoBrightness:saved.videoBrightness === true,
                            softFocus:saved.softFocus === true,
                            signLanguageMode:saved.signLanguageMode === true,
                            mutedNotification:saved.mutedNotification !== false,
                            lockMediaOnScreenLock:saved.lockMediaOnScreenLock === true,
                            notifyCallSound:saved.notifyCallSound !== false,
                            notifySoundName:['default','chime','ping','pop'].includes(saved.notifySoundName) ? saved.notifySoundName : 'default'
                        };
                    }, []);
                    const selfVideoStyle = 'transform:scaleX(' + (devicePreferences.mirrorVideo ? '-1' : '1') + ');filter:brightness(' + (devicePreferences.videoBrightness ? '1.2' : '1') + ') blur(' + (devicePreferences.softFocus ? '0.6px' : '0px') + ')';

                    // Repeated class lists. The pre-join body carries
                    // `data-backgrounds`, so its children tighten up through
                    // `group-data-[backgrounds=true]/body` when the background
                    // panel takes 340px of a wide window.
                    const AUDIO_CARD = "overflow-hidden rounded-md border border-edge bg-raised";
                    const AUDIO_CHOICE = "flex min-h-[66px] cursor-pointer items-center gap-4 px-[26px] py-[23px] text-[17px] max-[480px]:p-5 min-[851px]:min-h-14 min-[851px]:shrink-0 min-[851px]:px-6 min-[851px]:py-4 min-[851px]:text-base min-[1101px]:group-data-[backgrounds=true]/body:px-4";
                    const DEVICE_ROW = "flex min-h-[47px] items-center gap-4 text-ink min-[851px]:min-h-[42px] min-[1101px]:group-data-[backgrounds=true]/body:gap-2";
                    const ROOM_TOOL = "flex h-[60px] min-w-14 cursor-pointer flex-col items-center justify-center gap-[5px] rounded-md px-2.5 py-1.5 text-xs text-ink hover:bg-hover aria-pressed:bg-hover aria-pressed:shadow-[inset_0_-2px_var(--color-brand-ink)] disabled:cursor-not-allowed disabled:opacity-40 max-[1000px]:min-w-[45px] max-[1000px]:p-1.5 max-[1000px]:text-[11px] max-[650px]:h-[52px] max-[650px]:min-w-10";
                    const ROOM_LEAVE = twMerge(ROOM_TOOL + " ml-3.5 text-danger-ink hover:bg-destructive/40 max-[1000px]:ml-1");
                    const GALLERY = "grid h-full auto-rows-[minmax(0,1fr)] grid-cols-[repeat(auto-fit,minmax(min(300px,100%),1fr))] gap-3 p-[18px]";
                    const GALLERY_FOCUS = twMerge(GALLERY + " grid-cols-1");
                    const TILE = "relative min-h-0 min-w-0 overflow-hidden rounded-lg bg-raised";
                    const SELF_ALONE = "absolute inset-0 z-1 overflow-hidden";
                    const SELF_TILE = "absolute bottom-5 right-5 z-1 h-[130px] w-[200px] overflow-hidden rounded-lg border border-edge bg-raised max-[650px]:h-[90px] max-[650px]:w-[130px]";
                    const AVATAR = "grid shrink-0 place-items-center rounded-full bg-avatar font-semibold text-avatar-ink";
                    const AVATAR_ALONE = AVATAR + " h-[clamp(140px,15vw,220px)] w-[clamp(140px,15vw,220px)] text-[clamp(50px,6vw,86px)]";
                    const AVATAR_TILE = AVATAR + " h-16 w-16 text-[25px]";
                    const BACKGROUND_OPTION = "meeting-background-option relative flex h-[70px] min-w-0 cursor-pointer flex-col items-center justify-center gap-[3px] overflow-hidden rounded-[3px] border-2 border-transparent bg-raised text-[11px] text-ink hover:border-subtle aria-pressed:border-brand-ink aria-pressed:ring-1 aria-pressed:ring-brand-ink aria-pressed:after:absolute aria-pressed:after:right-[3px] aria-pressed:after:top-px aria-pressed:after:w-[18px] aria-pressed:after:rounded-full aria-pressed:after:bg-brand-press aria-pressed:after:text-center aria-pressed:after:leading-[18px] aria-pressed:after:text-white aria-pressed:after:content-['✓'] disabled:cursor-wait disabled:opacity-60";

                    const url = @{Json(&link.url)};
                    const code = @{Json(&code)};
                    const providerReady = @{Json(&provider_ready)};
                    const recordingReady = @{Json(&recording_ready)};

                    // The SDK is a megabyte, so it is imported inside the join
                    // rather than on the page, and only the elements it is
                    // handed are ever touched by it: re-renders leave their
                    // srcObject alone because nothing below re-creates them.
                    const SDK_URL = "/js/vendor/amazon-chime-sdk.js";

                    const localVideo = pp.ref(null);
                    const roomLocalVideo = pp.ref(null);
                    // The two menu buttons are rahti-ui Buttons, which cannot carry a
                    // ref of their own; each holds a marker the button is found from.
                    const reactionTriggerMark = pp.ref(null);
                    const reactionTrigger = {get current() {return reactionTriggerMark.current?.closest("button") || null;}};
                    const roomToolbar = pp.ref(null);
                    const backgroundPanel = pp.ref(null);
                    const moreTriggerMark = pp.ref(null);
                    const moreTrigger = {get current() {return moreTriggerMark.current?.closest("button") || null;}};
                    const reactionMenu = pp.ref(null);
                    const moreMenu = pp.ref(null);
                    const startedAt = pp.ref(0);
                    const shareStream = pp.ref(null);
                    const meetingAudio = pp.ref(null);
                    const meetingMicrophone = pp.ref(null);
                    const meetingOutgoingMicrophone = pp.ref(null);
                    const microphoneLifetime = pp.ref(0);
                    const mediaLocked = pp.ref(false);
                    const [captureVersion, setCaptureVersion] = pp.state(0);
                    const mutedReminderAt = pp.ref(0);
                    const [mutedNotice, setMutedNotice] = pp.state(false);
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
                    const anonymous = @{anonymous};
                    const [showLauncher, setShowLauncher] = pp.state(anonymous);
                    // A passcode meeting asks for its code before anything
                    // else. The code is checked here for a quick answer and
                    // again by the server when a lobby ticket is asked for.
                    const needsPasscode = @{needs_passcode};
                    const [passcode, setPasscode] = pp.state(@{Json(&initial_passcode)});
                    const [passcodeOk, setPasscodeOk] = pp.state(!needsPasscode);
                    const [passcodeError, setPasscodeError] = pp.state("");
                    const [checkingPasscode, setCheckingPasscode] = pp.state(false);
                    const passcodeGate = needsPasscode && !passcodeOk;
                    async function checkPasscode(value) {
                        if (checkingPasscode) return;
                        setCheckingPasscode(true);
                        setPasscodeError("");
                        try {
                            await pp.rpc("check_meeting_passcode", { id: code, passcode: value });
                            setPasscodeOk(true);
                        } catch (failure) {
                            setPasscodeError(failure.body?.error || failure.message || "That passcode could not be checked.");
                        } finally {
                            setCheckingPasscode(false);
                        }
                    }
                    function submitPasscode(event) {
                        event.preventDefault();
                        const value = Object.fromEntries(new FormData(event.currentTarget)).passcode || "";
                        if (value.trim()) checkPasscode(value.trim());
                    }
                    pp.effect(() => {
                        // A shared link may carry the code; take it out of the
                        // address bar so it is not copied on by accident.
                        if (new URLSearchParams(location.search).has("p")) {
                            history.replaceState(history.state, "", location.pathname);
                        }
                        if (needsPasscode && passcode.trim()) checkPasscode(passcode.trim());
                    }, []);
                    const [installHelp, setInstallHelp] = pp.state("");
                    const installPrompt = pp.ref(null);
                    const onInstallPrompt = pp.event(event => { event.preventDefault(); installPrompt.current = event; });
                    pp.effect(() => {
                        window.addEventListener('beforeinstallprompt', onInstallPrompt);
                        return () => window.removeEventListener('beforeinstallprompt', onInstallPrompt);
                    }, [onInstallPrompt]);
                    async function installApp() {
                        const prompt = installPrompt.current;
                        if (prompt) {
                            installPrompt.current = null;
                            try {
                                await prompt.prompt();
                                const choice = await prompt.userChoice;
                                setInstallHelp(choice.outcome === 'accepted' ? 'App installed. Continue in this browser to join this meeting, or open the shared link in Upurnaka.' : 'You can continue in this browser without installing.');
                            } catch (_) { setInstallHelp('Use your browser menu to install Upurnaka, or continue in this browser.'); }
                        } else {
                            setInstallHelp('In Chrome or Edge, open the browser menu and choose Install Upurnaka or Install this site as an app. On iPhone or iPad, use Share → Add to Home Screen. You can also continue in this browser.');
                        }
                    }
                    async function cancelSetup() {
                        await cancelLobby();
                        await teardown();
                        setCamera(false); setCameraLive(false); setBackgroundsOpen(false);
                        setShowLauncher(true);
                    }
                    const [editingName, setEditingName] = pp.state(false);
                    const [audioMode, setAudioMode] = pp.state("computer");
                    const [microphoneDevice, setMicrophoneDevice] = pp.state(devicePreferences.microphoneId);
                    const [speakerDevice, setSpeakerDevice] = pp.state(devicePreferences.speakerId);
                    const [audioInputs, setAudioInputs] = pp.state([]);
                    const [audioOutputs, setAudioOutputs] = pp.state([]);
                    const [camera, setCamera] = pp.state(false);
                    const [cameraLive, setCameraLive] = pp.state(false);
                    const [remoteTiles, setRemoteTiles] = pp.state([false, false, false, false]);
                    const [tileDetails, setTileDetails] = pp.state([null, null, null, null]);
                    const [people, setPeople] = pp.state([]);
                    // Whether each other attendee's microphone is muted, as the
                    // media service reports it. Absent until it first reports,
                    // which a person who joined without audio never does.
                    const [micMuted, setMicMuted] = pp.state({});
                    const volumeWatched = pp.ref(new Set());
                    const [mutingAll, setMutingAll] = pp.state(false);
                    const [connectionStatus, setConnectionStatus] = pp.state("connecting");
                    const [elapsed, setElapsed] = pp.state(0);
                    const [roomPanel, setRoomPanel] = pp.state("");
                    const [handRaised, setHandRaised] = pp.state(false);
                    const [handLowerAt, setHandLowerAt] = pp.state(0);
                    const [handLowerSeconds, setHandLowerSeconds] = pp.state(0);
                    const handAutoKept = pp.ref(false);
                    const handSpeech = pp.ref({ volume: 0, muted: true, since: 0 });
                    const [reactionsOpen, setReactionsOpen] = pp.state(false);
                    const [toolbarBottom, setToolbarBottom] = pp.state(72);
                    const [roomMenuStyle, setRoomMenuStyle] = pp.state("");
                    const [reactionNotice, setReactionNotice] = pp.state("");
                    const [moreOpen, setMoreOpen] = pp.state(false);
                    const [focusView, setFocusView] = pp.state(false);
                    const [sharingScreen, setSharingScreen] = pp.state(false);
                    const [startingShare, setStartingShare] = pp.state(false);
                    const [chatMessages, setChatMessages] = pp.state([]);
                    const [chatDraft, setChatDraft] = pp.state("");
                    const [chatUnread, setChatUnread] = pp.state(0);
                    const chatLog = pp.ref(null);
                    const [displayName, setDisplayName] = pp.state(@{user.name.as_str()});
                    const [joining, setJoining] = pp.state(false);
                    const [session, setSession] = pp.state(null);
                    const [error, setError] = pp.state("");
                    const [toast, setToast] = pp.state("");
                    const [admission, setAdmission] = pp.state("");
                    const [lobbyWaiting, setLobbyWaiting] = pp.state(false);
                    const [lobbyRequests, setLobbyRequests] = pp.state([]);
                    const [isOrganizer, setIsOrganizer] = pp.state(false);
                    const [spotlightId, setSpotlightId] = pp.state("");
                    const [pinnedId, setPinnedId] = pp.state("");
                    const [hideSelf, setHideSelf] = pp.state(devicePreferences.signLanguageMode);
                    const [tileFits, setTileFits] = pp.state({});
                    const [tileMenu, setTileMenu] = pp.state(null);
                    const [handNotice, setHandNotice] = pp.state("");
                    const [recordingId, setRecordingId] = pp.state("");
                    const [recordingBusy, setRecordingBusy] = pp.state(false);
                    const [roomRecording, setRoomRecording] = pp.state(false);
                    const [recordings, setRecordings] = pp.state([]);
                    const [recordingsLoading, setRecordingsLoading] = pp.state(false);
                    // The MediaRecorder, the screen capture it reads and the
                    // audio graph mixing the call into it. A ref: none of it is
                    // rendered, and a re-render must never restart a recording.
                    const recorder = pp.ref(null);
                    const cameraChoice = pp.ref(0);
                    const connecting = pp.ref(false);
                    const tileMenuElement = pp.ref(null);

                    const remoteCount = remoteTiles.filter(Boolean).length;
                    const hasContent = tileDetails.some(tile => tile && tile.content);
                    const featuredId = pinnedId || spotlightId;
                    const railRows = Math.max(1, people.filter(person => person.id !== (session?.attendee.AttendeeId || '') && person.id !== featuredId).length + (hasContent && featuredId ? 1 : 0) + (!hideSelf && featuredId !== (session?.attendee.AttendeeId || '') ? 1 : 0));
                    // "muted", "on", or "" while the media service has not said:
                    // someone who joined without audio may never report a microphone.
                    function micState(id) {
                        if (!id) return "";
                        if (session && id === session.attendee.AttendeeId) return !microphone || audioMode === "none" ? "muted" : "on";
                        return micMuted[id] === true ? "muted" : micMuted[id] === false ? "on" : "";
                    }
                    function micOff(id) { return micState(id) === "muted"; }
                    const remoteSlots = remoteRefs.map((ref, index) => ({
                        index,
                        ref,
                        active: remoteTiles[index] || !!tileDetails[index]?.content,
                        paused: !remoteTiles[index],
                        name: tileDetails[index] ? tileDetails[index].name : "Participant",
                        content: tileDetails[index] ? tileDetails[index].content : false,
                        attendeeId: tileDetails[index] ? tileDetails[index].id : "",
                        raised: !!tileDetails[index] && !tileDetails[index].content && people.some(person => person.id === tileDetails[index].id && person.raised),
                        muted: !!tileDetails[index] && !tileDetails[index].content && micOff(tileDetails[index].id),
                    }));
                    const selfAttendeeId = session ? session.attendee.AttendeeId : "";
                    const remotePeople = people.filter(person => person.id !== selfAttendeeId).map(person => ({
                        ...person,
                        muted: micOff(person.id),
                        hasVideo: tileDetails.some((tile, index) => remoteTiles[index] && tile && tile.id === person.id),
                    }));
                    const peopleRows = people.map(person => ({ ...person, mic: micState(person.id), muted: micOff(person.id), self: person.id === selfAttendeeId }));
                    const unmutedOthers = peopleRows.filter(person => !person.self && !person.muted).length;
                    const myInitials = initials(displayName);
                    const recordingLive = !!recordingId || roomRecording;
                    const recordingAllowed = !!recordingId || (recordingReady && isOrganizer && connectionStatus === "connected" && !roomRecording);
                    const recordTitle = !recordingReady ? "Recording storage is not configured for this workspace"
                        : !isOrganizer ? "Only the organizer can record this meeting"
                        : recordingId ? "Stop recording and save it to storage"
                        : roomRecording ? "This meeting is already being recorded"
                        : "Record this meeting";
                    const elapsedLabel = Math.floor(elapsed / 60).toString().padStart(2, "0") + ":" + (elapsed % 60).toString().padStart(2, "0");

                    const onClockTick = pp.event(() => {
                        const now = Date.now();
                        if (startedAt.current) setElapsed(Math.floor((now - startedAt.current) / 1000));
                        if (!handRaised || handAutoKept.current) return;
                        if (connectionStatus !== "connected") {
                            handSpeech.current.since = 0;
                            setHandLowerAt(0);
                            return;
                        }
                        if (handLowerAt) {
                            setHandLowerSeconds(Math.max(0, Math.ceil((handLowerAt - now) / 1000)));
                            if (now >= handLowerAt) {
                                // Do not repeatedly retry a failed data-message send.
                                keepHandRaised();
                                updateHand(false);
                            }
                        } else if (!microphone || handSpeech.current.muted || call.current?.av.realtimeIsLocalAudioMuted()) {
                            handSpeech.current.since = 0;
                        } else if (handSpeech.current.volume >= 0.1 && !handSpeech.current.since) {
                            handSpeech.current.since = now;
                        } else if (handSpeech.current.since && now - handSpeech.current.since >= 2000) {
                            setHandLowerAt(now + 5000);
                            setHandLowerSeconds(5);
                        }
                    });

                    const notifyMutedSpeech = pp.event(volume => {
                        if (!devicePreferences.mutedNotification || microphone || audioMode !== 'computer' || connectionStatus !== 'connected' || !Number.isFinite(volume) || volume < 0.1) return;
                        if (Date.now() - mutedReminderAt.current < 8000) return;
                        mutedReminderAt.current = Date.now(); setMutedNotice(true);
                    });
                    pp.effect(() => {
                        if (microphone || !session) setMutedNotice(false);
                    }, [microphone, session !== null]);
                    pp.effect(() => {
                        if (!session || !devicePreferences.mutedNotification || !meetingMicrophone.current || typeof AudioContext !== 'function') return;
                        const context = new AudioContext();
                        const analyser = context.createAnalyser(); analyser.fftSize = 256;
                        context.createMediaStreamSource(meetingMicrophone.current).connect(analyser);
                        context.resume().catch(() => {});
                        const samples = new Uint8Array(analyser.fftSize);
                        const timer = setInterval(() => {
                            analyser.getByteTimeDomainData(samples); let sum = 0;
                            for (const sample of samples) sum += ((sample - 128) / 128) ** 2;
                            notifyMutedSpeech(Math.sqrt(sum / samples.length));
                        }, 250);
                        return () => { clearInterval(timer); context.close().catch(() => {}); };
                    }, [session !== null, captureVersion, notifyMutedSpeech]);
                    const lockMeetingMedia = pp.event(async () => {
                        const live = call.current;
                        if (!live) return;
                        microphoneLifetime.current += 1;
                        mediaLocked.current = true;
                        setMicrophone(false); setCamera(false); setMutedNotice(false);
                        meetingMicrophone.current?.getTracks().forEach(track => track.stop()); meetingMicrophone.current = null;
                        meetingOutgoingMicrophone.current?.getTracks().forEach(track => track.stop()); meetingOutgoingMicrophone.current = null;
                        setCaptureVersion(value => value + 1);
                        localVideo.current?.srcObject?.getTracks().forEach(track => track.stop());
                        roomLocalVideo.current?.srcObject?.getTracks().forEach(track => track.stop());
                        shareStream.current?.getTracks().forEach(track => track.stop()); shareStream.current = null; setSharingScreen(false);
                        if (recorder.current) releaseRecordingMedia(recorder.current);
                        await Promise.allSettled([
                            Promise.resolve().then(() => live.av.realtimeMuteLocalAudio()),
                            Promise.resolve().then(() => live.av.stopLocalVideoTile()),
                            Promise.resolve().then(() => live.av.stopContentShare()),
                            Promise.resolve().then(() => live.av.stopAudioInput()),
                            closeCamera(), recorder.current ? stopRecording() : Promise.resolve()
                        ]);
                        if (!disposed.current) setToast('Your screen locked. Camera, microphone, sharing, and recording are off. Turn devices on manually when you return.');
                    });
                    pp.effect(() => {
                        if (!session || !devicePreferences.lockMediaOnScreenLock) return;
                        if (typeof IdleDetector !== 'function') { setToast('Screen-lock protection is unavailable in this browser.'); return; }
                        let cancelled = false;
                        const abort = new AbortController();
                        const detector = new IdleDetector();
                        const changed = () => { if (!cancelled && detector.screenState === 'locked') lockMeetingMedia(); };
                        detector.addEventListener('change', changed);
                        (async () => {
                            try {
                                const permission = await navigator.permissions.query({name:'idle-detection'});
                                if (cancelled) return;
                                if (permission.state !== 'granted') { setToast('Screen-lock protection is not active. Enable its browser permission in Settings > Privacy.'); return; }
                                await detector.start({threshold:60000, signal:abort.signal});
                                changed();
                            } catch (_) { if (!cancelled) setToast('Screen-lock protection could not start. Check your browser permissions.'); }
                        })();
                        return () => { cancelled = true; abort.abort(); detector.removeEventListener('change', changed); };
                    }, [session !== null, lockMeetingMedia]);
                    const onLocalHandVolume = pp.event((attendeeId, volume, muted) => {
                        const live = call.current;
                        if (!live || attendeeId !== live.attendeeId) return;
                        if (typeof volume === 'number') notifyMutedSpeech(volume);
                        const activity = handSpeech.current;
                        if (volume !== null) activity.volume = volume;
                        if (muted !== null) activity.muted = muted;
                        const speaking = handRaised && microphone && connectionStatus === "connected" &&
                            !activity.muted && !live.av.realtimeIsLocalAudioMuted() && activity.volume >= 0.1;
                        if (!speaking) activity.since = 0;
                        else if (!activity.since) activity.since = Date.now();
                    });
                    pp.effect(() => {
                        if (!handNotice) return;
                        const timer = setTimeout(() => setHandNotice(''), 6000);
                        return () => clearTimeout(timer);
                    }, [handNotice]);

                    // Organizers hear a join request once, when it first appears.
                    const knownLobby = pp.ref(null);
                    const applyRoomState = pp.event(async (result, ticket) => {
                        const waitingIds = result.waiting.map(request => request.id);
                        if (isOrganizer && knownLobby.current && devicePreferences.notifyCallSound && waitingIds.some(id => !knownLobby.current.includes(id))) {
                            window.UpurnakaNotificationSound?.play(devicePreferences.notifySoundName);
                        }
                        knownLobby.current = waitingIds;
                        setLobbyRequests(result.waiting);
                        setSpotlightId(result.spotlight || '');
                        setRoomRecording(!!result.recording);
                        if (result.muted_by_host && microphone && call.current) {
                            call.current.av.realtimeMuteLocalAudio(); setMicrophone(false);
                            setToast('The organizer muted your microphone. You can unmute when ready.');
                        }
                        if (result.state === 'admitted' && !session && !connecting.current) await connectAdmitted(ticket);
                        if (result.state === 'denied' || result.state === 'left' || result.state === 'removed') {
                            setLobbyWaiting(false); setAdmission('');
                            if (session) { await teardown(); setSession(null); }
                            setError(result.state === 'denied' ? 'The organizer declined your request to join.' : result.state === 'removed' ? 'The organizer limited this meeting to invited people, and you were removed.' : 'Your meeting entry has ended.');
                        }
                    });
                    pp.effect(() => {
                        if (!admission) return;
                        let cancelled = false, busy = false;
                        async function poll() {
                            if (busy || cancelled) return; busy = true;
                            try {
                                const result = await pp.rpc('meeting_room_state', {id:code, ticket:admission});
                                if (!cancelled) await applyRoomState(result, admission);
                            } catch (failure) {
                                if (!cancelled) {
                                    if ([403, 404].includes(failure.status)) {
                                        setAdmission(''); setLobbyWaiting(false);
                                        if (session) { await teardown(); setSession(null); }
                                    }
                                    setError(failure.body?.error || failure.message || 'Could not update meeting controls.');
                                }
                            } finally { busy = false; }
                        }
                        poll();
                        const timer = setInterval(poll, 5000);
                        return () => { cancelled = true; clearInterval(timer); };
                    }, [admission]);

                    const decideEntry = pp.event(async function decideEntry(ticket, admit) {
                        try {
                            await pp.rpc('decide_meeting_entry', {id:code, ticket, admit});
                            setLobbyRequests(current => current.filter(request => request.id !== ticket));
                        } catch (failure) { setError(failure.body?.error || failure.message); }
                    });
                    async function cancelLobby() {
                        const ticket = admission;
                        setLobbyWaiting(false); setAdmission('');
                        if (ticket) {
                            try { await pp.rpc('leave_meeting_entry', {id:code,ticket}); }
                            catch (failure) { setError(failure.body?.error || failure.message); }
                        }
                    }
                    // Beside the button that asked, on whichever side has more room.
                    function tileMenuPlace(trigger) {
                        const gap = 8;
                        const width = 256; // `w-64`; the menu is still hidden, so it cannot be measured.
                        const stage = tileMenuElement.current.closest(".room-stage").getBoundingClientRect();
                        const anchor = trigger.getBoundingClientRect();
                        const left = Math.max(gap, Math.min(anchor.left - stage.left, stage.width - width - gap));
                        const above = anchor.top - stage.top;
                        const below = stage.bottom - anchor.bottom;
                        const side = above >= below ? "bottom:" + (stage.bottom - anchor.top + gap) : "top:" + (anchor.bottom - stage.top + gap);
                        return "left:" + left + "px;" + side + "px;max-height:" + (Math.max(above, below) - gap * 2) + "px";
                    }
                    const openTileMenu = pp.event(function openTileMenu(id, name, trigger) { setMoreOpen(false); setReactionsOpen(false); setTileMenu(tileMenu?.id === id ? null : {id,name,style:tileMenuPlace(trigger)}); });
                    pp.effect(() => {
                        if (!tileMenu) return;
                        const close = () => setTileMenu(null);
                        window.addEventListener("resize", close);
                        return () => window.removeEventListener("resize", close);
                    }, [tileMenu !== null]);
                    function pinTile() { if (tileMenu) setPinnedId(pinnedId === tileMenu.id ? '' : tileMenu.id); setTileMenu(null); }
                    function fitTile() { if (tileMenu) { const id = tileMenu.id; setTileFits(current => ({...current,[id]:current[id] === false})); } setTileMenu(null); }
                    function hideMyTile() { setHideSelf(true); if (pinnedId === selfAttendeeId) setPinnedId(''); setTileMenu(null); }
                    async function spotlightTile() {
                        if (!tileMenu) return;
                        const attendee = spotlightId === tileMenu.id ? null : tileMenu.id;
                        try { await pp.rpc('spotlight_participant', {id:code,attendee}); setSpotlightId(attendee || ''); setPinnedId(''); setTileMenu(null); }
                        catch(failure) { setError(failure.body?.error || failure.message); }
                    }
                    // Organizer mutes. The badge turns on at once; the media
                    // service confirms it when the person's browser mutes.
                    const muteAttendee = pp.event(async function muteAttendee(attendee, name) {
                        if (!attendee || attendee === selfAttendeeId) return;
                        try {
                            await pp.rpc('mute_participant', {id:code,attendee});
                            setMicMuted(current => ({ ...current, [attendee]: true }));
                            setTileMenu(null); setToast((name || 'Participant') + ' is muted.');
                        }
                        catch(failure) { setToast(failure.body?.error || failure.message || 'That participant could not be muted.'); }
                    });
                    function muteTile() { if (tileMenu) muteAttendee(tileMenu.id, tileMenu.name); }
                    const muteEveryone = pp.event(async function muteEveryone() {
                        if (!isOrganizer || mutingAll) return;
                        setMutingAll(true); setMoreOpen(false);
                        try {
                            const count = await pp.rpc('mute_all_participants', {id:code});
                            setMicMuted(current => {
                                const next = { ...current };
                                for (const person of people) if (person.id !== selfAttendeeId) next[person.id] = true;
                                return next;
                            });
                            setToast(count === 0 ? 'Nobody else is in the meeting to mute.' : count === 1 ? 'Muted 1 participant.' : 'Muted ' + count + ' participants.');
                        }
                        catch(failure) { setToast(failure.body?.error || failure.message || 'Everyone could not be muted.'); }
                        finally { setMutingAll(false); }
                    });
                    async function switchCamera() {
                        setTileMenu(null);
                        if (applyingBackground) return;
                        setApplyingBackground(true);
                        try {
                            const kit = await ensureDevices();
                            const cameras = await kit.controller.listVideoInputDevices();
                            if (cameras.length < 2) { setToast('No other camera is available.'); return; }
                            kit.cameraSelected = true;
                            cameraChoice.current = (cameraChoice.current + 1) % cameras.length;
                            setCameraDevice(cameras[cameraChoice.current].deviceId);
                            if (camera) {
                                await closeCamera(); await openCamera();
                                if (call.current) call.current.av.startLocalVideoTile(); else startPreview();
                            }
                        } catch(failure) { setError(failure.message); }
                        finally { setApplyingBackground(false); }
                    }
                    const onConnectionStarting = pp.event(reconnecting => {
                        setConnectionStatus(reconnecting ? "reconnecting" : "connecting");
                        handSpeech.current.since = 0;
                        handSpeech.current.volume = 0;
                        setHandLowerAt(0);
                        setHandLowerSeconds(0);
                    });
                    const onConnectionStarted = pp.event(() => {
                        setConnectionStatus("connected");
                        if (!startedAt.current) startedAt.current = Date.now();
                    });
                    const onShareStopped = pp.event(() => {
                        setSharingScreen(false);
                        if (shareStream.current) shareStream.current.getTracks().forEach(track => track.stop());
                        shareStream.current = null;
                    });

                    pp.effect(() => {
                        if (!session) return;
                        const timer = setInterval(onClockTick, 1000);
                        return () => clearInterval(timer);
                    }, [session !== null]);

                    pp.effect(() => {
                        if (!reactionNotice) return;
                        const timer = setTimeout(() => setReactionNotice(""), 4000);
                        return () => clearTimeout(timer);
                    }, [reactionNotice]);

                    function initials(name) {
                        return name.trim().split(/\s+/).slice(0, 2).map(word => word[0] || "").join("").toUpperCase() || "G";
                    }

                    function attendeeNameFrom(externalId) {
                        return String(externalId || "Guest").split("#")[0].slice(0, 64) || "Guest";
                    }

                    function toggleRoomPanel(panel) {
                        dismissRoomMenus();
                        setRoomPanel(roomPanel === panel ? "" : panel);
                        setBackgroundsOpen(false);
                    }

                    function openRoomBackgrounds() {
                        dismissRoomMenus();
                        setRoomPanel("");
                        setMoreOpen(false);
                        setBackgroundsOpen(true);
                    }

                    const onAttendeeVolume = pp.event((attendeeId, volume, muted) => {
                        if (typeof muted !== "boolean") return;
                        setMicMuted(current => current[attendeeId] === muted ? current : { ...current, [attendeeId]: muted });
                    });
                    function watchVolume(id, on) {
                        const live = call.current, watched = volumeWatched.current;
                        if (!live || id === live.attendeeId || watched.has(id) === on) return;
                        try {
                            if (on) live.av.realtimeSubscribeToVolumeIndicator(id, onAttendeeVolume);
                            else live.av.realtimeUnsubscribeFromVolumeIndicator(id, onAttendeeVolume);
                        } catch (_) { return; }
                        if (on) watched.add(id); else watched.delete(id);
                    }
                    const onPresence = pp.event((id, present, externalId) => {
                        if (id.includes("#content")) return;
                        watchVolume(id, present);
                        if (!present) {
                            setMicMuted(current => { const next = { ...current }; delete next[id]; return next; });
                            setPeople(current => current.filter(person => person.id !== id));
                            if (pinnedId === id) setPinnedId('');
                            if (tileMenu?.id === id) setTileMenu(null);
                            return;
                        }
                        const name = id === call.current?.attendeeId ? displayName : attendeeNameFrom(externalId);
                        setPeople(current => current.some(person => person.id === id) ? current : [...current, { id, name, initials: initials(name), raised: false }]);
                    });

                    function sendMeetingEvent(payload) {
                        const live = call.current;
                        if (!live || connectionStatus !== "connected") throw new Error("Wait for the meeting to connect.");
                        live.av.realtimeSendDataMessage("upurnaka-room", JSON.stringify(payload), 0);
                    }

                    function appendChat(message) {
                        setChatMessages(current => current.some(item => item.id === message.id) ? current : [...current.slice(-199), message]);
                    }

                    // What was said before this join, merged with anything that
                    // arrived while the request was on its way.
                    const loadChat = pp.event(async ticket => {
                        try {
                            const saved = await pp.rpc("meeting_chat_history", { id: code, ticket });
                            const known = new Set(saved.map(message => message.id));
                            setChatMessages(current => [...saved, ...current.filter(message => !known.has(message.id))].sort((a, b) => a.at - b.at).slice(-200));
                        } catch (failure) {
                            // Live messages keep arriving; opening the panel asks again.
                        }
                    });

                    // Sharing a file in the chat. Signed-in participants pick
                    // one from My files, or upload one there first; everyone
                    // in the call downloads through a link the server mints
                    // for their own ticket.
                    const [chatFile, setChatFile] = pp.state(null);
                    const [chatUploading, setChatUploading] = pp.state("");
                    const [chatAttachMenu, setChatAttachMenu] = pp.state(false);
                    const [chatPicker, setChatPicker] = pp.state(false);
                    const [pickerListing, setPickerListing] = pp.state({ folder: null, path: [], folders: [], files: [] });
                    const [pickerLoading, setPickerLoading] = pp.state(false);
                    const [pickerError, setPickerError] = pp.state("");
                    const chatFileInput = pp.ref(null);

                    function fileSize(bytes) {
                        if (!bytes) return "0 B";
                        const units = ["B", "KB", "MB", "GB"];
                        const step = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
                        const value = bytes / Math.pow(1024, step);
                        return (step === 0 ? value : value.toFixed(value < 10 ? 1 : 0)) + " " + units[step];
                    }
                    function chooseChatUpload() {
                        setChatAttachMenu(false);
                        chatFileInput.current?.click();
                    }
                    async function uploadChatFile(event) {
                        const file = event.currentTarget.files?.[0];
                        event.currentTarget.value = "";
                        if (!file || !admission) return;
                        setChatUploading(file.name);
                        try {
                            setChatFile(await pp.rpc("upload_meeting_chat_file", { id: code, ticket: admission, file }));
                        } catch (failure) {
                            setError(failure.errors?.file?.[0] || failure.body?.error || failure.message || "The file could not be uploaded.");
                        } finally {
                            setChatUploading("");
                        }
                    }
                    // Folders can be clicked faster than they load: only the
                    // newest request may replace the list or end the loading state.
                    const pickerRequest = pp.ref(0);
                    const openPickerFolder = pp.event(async function openPickerFolder(folder) {
                        if (!admission) return;
                        const request = ++pickerRequest.current;
                        setPickerLoading(true);
                        setPickerError("");
                        setPickerChoice("");
                        try {
                            const listing = await pp.rpc("meeting_chat_my_files", { id: code, ticket: admission, folder: folder || null });
                            if (request === pickerRequest.current) setPickerListing(listing);
                        } catch (failure) {
                            if (request === pickerRequest.current) setPickerError(failure.body?.error || failure.message || "Your files could not be loaded.");
                        } finally {
                            if (request === pickerRequest.current) setPickerLoading(false);
                        }
                    });
                    // My files opens in a dialog, like a file explorer: a
                    // folder opens on click, a file is chosen on click and
                    // attached with Attach or a double-click.
                    const [pickerChoice, setPickerChoice] = pp.state("");
                    const pickerOpener = pp.ref(null);
                    const pickerCloser = pp.ref(null);
                    function openChatPicker() {
                        setChatAttachMenu(false);
                        setChatPicker(true);
                        setPickerChoice("");
                        pickerOpener.current?.click();
                        openPickerFolder("");
                    }
                    function closeChatPicker() { setChatPicker(false); setPickerError(""); setPickerChoice(""); }
                    const choosePickerFile = pp.event(function choosePickerFile(id) { setPickerChoice(id); });
                    function pickerChoiceName() {
                        const file = pickerListing.files.find(item => item.id === pickerChoice);
                        return file ? file.name : "Choose a file to share.";
                    }
                    const attachPickerFile = pp.event(function attachPickerFile(id) {
                        const file = pickerListing.files.find(item => item.id === id);
                        if (!file) return;
                        setChatFile(file);
                        pickerCloser.current?.click();
                    });
                    const downloadChatFile = pp.event(async function downloadChatFile(messageId) {
                        if (!admission) return;
                        try {
                            const url = await pp.rpc("meeting_chat_file_url", { id: code, ticket: admission, message: messageId });
                            window.open(url, "_blank", "noopener");
                        } catch (failure) {
                            setError(failure.body?.error || failure.message || "The file could not be opened.");
                        }
                    });

                    function chatTime(at) {
                        const sent = new Date(at);
                        const today = sent.toDateString() === new Date().toDateString();
                        const time = { hour: "numeric", minute: "2-digit" };
                        return sent.toLocaleString([], today ? time : { month: "short", day: "numeric", ...time });
                    }

                    pp.effect(() => {
                        if (roomPanel !== "chat") return;
                        setChatUnread(0);
                        if (admission) loadChat(admission);
                    }, [roomPanel]);
                    pp.effect(() => {
                        if (roomPanel === "chat" && chatLog.current) chatLog.current.scrollTop = chatLog.current.scrollHeight;
                    }, [roomPanel, chatMessages.length]);

                    function announceChat(name, text) {
                        if (roomPanel === "chat") return;
                        setChatUnread(count => count + 1);
                        setToast(name + ": " + (text.length > 80 ? text.slice(0, 80) + "…" : text));
                        if (devicePreferences.notifySound) window.UpurnakaNotificationSound?.play(devicePreferences.notifySoundName);
                    }

                    const onRoomMessage = pp.event(message => {
                        if (message.throttled || message.data.length > 4096) return;
                        try {
                            const payload = message.json();
                            const sender = message.senderAttendeeId;
                            if (!sender || sender.includes("#content")) return;
                            const name = attendeeNameFrom(message.senderExternalUserId);
                            if (payload.kind === "chat" && payload.file === true && typeof payload.id === "string") {
                                // A shared file is read from the server, which
                                // is what says what was shared and by whom.
                                if (admission) loadChat(admission);
                                announceChat(name, typeof payload.text === "string" && payload.text.trim() ? payload.text.slice(0, 500) : "shared a file");
                            } else if (payload.kind === "chat" && typeof payload.text === "string" && payload.text.trim() && typeof payload.id === "string") {
                                const text = payload.text.slice(0, 500);
                                appendChat({ id: payload.id.slice(0, 64), name, text, at: Date.now() });
                                announceChat(name, text);
                            } else if (payload.kind === "hand" && typeof payload.raised === "boolean") {
                                if (payload.raised && !people.find(person => person.id === sender)?.raised) setHandNotice(name);
                                setPeople(current => current.map(person => person.id === sender ? { ...person, raised: payload.raised } : person));
                            } else if (payload.kind === "reaction" && ["👍", "❤️", "👏", "😄"].includes(payload.emoji)) {
                                setReactionNotice(payload.emoji + " " + name);
                            } else if (payload.kind === "recording" && typeof payload.on === "boolean") {
                                // A hint for an immediate badge; the room state
                                // poll is what decides, so a forged message
                                // lasts five seconds at most.
                                setRoomRecording(payload.on);
                                setToast(payload.on ? name + " started recording this meeting." : "Recording stopped.");
                            }
                        } catch (failure) {
                            // Ignore messages from clients that use a different protocol.
                        }
                    });

                    async function sendChat(event) {
                        event.preventDefault();
                        const text = chatDraft.trim().slice(0, 500);
                        const file = chatFile;
                        if ((!text && !file) || chatUploading) return;
                        setChatDraft("");
                        setChatFile(null);
                        setChatAttachMenu(false);
                        let message;
                        try {
                            message = await pp.rpc("send_meeting_chat", { id: code, ticket: admission, text, file: file ? file.id : null });
                        } catch (failure) {
                            setChatDraft(text);
                            setChatFile(file);
                            setError(failure.body?.error || failure.message);
                            return;
                        }
                        appendChat(message);
                        try { sendMeetingEvent({ kind: "chat", id: message.id, text: message.text, file: !!message.file }); }
                        catch (failure) { setError(failure.message); }
                    }

                    function raiseHand() {
                        updateHand(!handRaised);
                    }

                    function updateHand(raised) {
                        try {
                            sendMeetingEvent({ kind: "hand", raised });
                            setHandRaised(raised);
                            setHandLowerAt(0);
                            setHandLowerSeconds(0);
                            handAutoKept.current = false;
                            handSpeech.current.since = 0;
                            setPeople(current => current.map(person => person.id === selfAttendeeId ? { ...person, raised } : person));
                        } catch (failure) { setError(failure.message); }
                    }

                    function keepHandRaised() {
                        handAutoKept.current = true;
                        handSpeech.current.since = 0;
                        setHandLowerAt(0);
                        setHandLowerSeconds(0);
                    }

                    function sendReaction(emoji) {
                        try {
                            sendMeetingEvent({ kind: "reaction", emoji });
                            setReactionNotice(emoji + " You");
                            setReactionsOpen(false);
                        } catch (failure) { setError(failure.message); }
                    }

                    // Phone browsers do not offer screen capture to web pages at all.
                    function canShareScreen() {
                        return typeof navigator.mediaDevices?.getDisplayMedia === "function";
                    }

                    async function toggleScreenShare() {
                        const live = call.current;
                        if (!live || startingShare) return;
                        if (!canShareScreen()) {
                            setToast("This browser can't share your screen. Join from a computer to share.");
                            return;
                        }
                        if (sharingScreen) {
                            live.av.stopContentShare();
                            if (shareStream.current) shareStream.current.getTracks().forEach(track => track.stop());
                            shareStream.current = null;
                            setSharingScreen(false);
                            return;
                        }
                        setStartingShare(true);
                        setError("");
                        try {
                            const stream = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: false });
                            if (disposed.current || call.current !== live) { stream.getTracks().forEach(track => track.stop()); return; }
                            shareStream.current = stream;
                            await live.av.startContentShare(stream);
                            setSharingScreen(true);
                        } catch (failure) {
                            if (shareStream.current) shareStream.current.getTracks().forEach(track => track.stop());
                            shareStream.current = null;
                            if (failure.name !== "NotAllowedError") setError(failure.message || "Screen sharing could not start.");
                        } finally { setStartingShare(false); }
                    }

                    // Recording. The browser captures this tab, which is the
                    // meeting exactly as the organizer sees it, and mixes the
                    // call's audio with their own microphone: the tab's own
                    // audio would need a checkbox in the browser's picker that
                    // is easy to miss. Pieces go to the server every few
                    // seconds; the server hands the finished file to storage.
                    const RECORDING_TYPES = ["video/webm;codecs=vp9,opus", "video/webm;codecs=vp8,opus", "video/webm", "video/mp4"];
                    const RECORDING_SLICE = 5000;

                    async function toggleRecording() {
                        if (recordingBusy) return;
                        if (recordingId) { await stopRecording(); return; }
                        if (!recordingAllowed || !call.current) return;
                        setRecordingBusy(true);
                        setError("");
                        let capture = null, mix = null;
                        try {
                            const type = window.MediaRecorder && RECORDING_TYPES.find(candidate => MediaRecorder.isTypeSupported(candidate));
                            if (!type || !navigator.mediaDevices?.getDisplayMedia) throw new Error("This browser cannot record meetings.");
                            // First, while the click still counts as one: the
                            // browser asks for a gesture before showing its picker.
                            capture = await navigator.mediaDevices.getDisplayMedia({
                                video: { frameRate: 30 },
                                audio: false,
                                preferCurrentTab: true,
                                selfBrowserSurface: "include",
                                surfaceSwitching: "exclude",
                            });
                            if (disposed.current || !call.current) throw new Error("The meeting ended before recording started.");
                            mix = await recordingAudio();
                            const started = await pp.rpc("start_meeting_recording", { id: code, ticket: admission, content_type: type });
                            const stream = new MediaStream([...capture.getVideoTracks(), ...mix.destination.stream.getAudioTracks()]);
                            const media = new MediaRecorder(stream, { mimeType: type, videoBitsPerSecond: 2500000, audioBitsPerSecond: 128000 });
                            const live = { id: started.id, type, media, capture, mix, part: 0, sending: Promise.resolve(), failed: false, stopping: false };
                            media.ondataavailable = event => queueRecordingPiece(live, event.data);
                            // "Stop sharing" in the browser's own bar ends the capture.
                            capture.getVideoTracks()[0].onended = () => stopRecording();
                            recorder.current = live;
                            media.start(RECORDING_SLICE);
                            setRecordingId(started.id);
                            setRoomRecording(true);
                            setToast("Recording started. Everyone in the meeting can see it is being recorded.");
                            try { sendMeetingEvent({ kind: "recording", on: true }); } catch (failure) { /* the room state poll tells them */ }
                        } catch (failure) {
                            releaseRecordingMedia({ capture, mix });
                            if (failure.name !== "NotAllowedError") setError(failure.body?.error || failure.message || "Recording could not start.");
                        } finally {
                            setRecordingBusy(false);
                        }
                    }

                    // The call's audio element already carries every remote
                    // participant, mixed by Chime.
                    async function recordingAudio() {
                        const context = new AudioContext();
                        const destination = context.createMediaStreamDestination();
                        const remote = meetingAudio.current?.srcObject;
                        if (remote instanceof MediaStream && remote.getAudioTracks().length > 0) {
                            context.createMediaStreamSource(remote).connect(destination);
                        }
                        const gain = context.createGain();
                        gain.gain.value = microphone ? 1 : 0;
                        gain.connect(destination);
                        let voice = null;
                        if (audioMode === "computer") {
                            try {
                                voice = await navigator.mediaDevices.getUserMedia({ audio: microphoneDevice ? { deviceId: { exact: microphoneDevice } } : true });
                                context.createMediaStreamSource(voice).connect(gain);
                            } catch (failure) {
                                setToast("Recording without your microphone: it could not be opened.");
                            }
                        }
                        await context.resume();
                        return { context, destination, gain, voice };
                    }

                    function releaseRecordingMedia(live) {
                        if (live.capture) live.capture.getTracks().forEach(track => track.stop());
                        if (live.mix) {
                            if (live.mix.voice) live.mix.voice.getTracks().forEach(track => track.stop());
                            live.mix.context.close().catch(() => {});
                        }
                    }

                    // Muting yourself mutes you in the recording too.
                    pp.effect(() => {
                        const live = recorder.current;
                        if (live) live.mix.gain.gain.value = microphone ? 1 : 0;
                    }, [microphone, recordingId]);

                    // One piece at a time and in order: the server joins
                    // them into one file by number.
                    function queueRecordingPiece(live, blob) {
                        if (!blob || blob.size === 0 || live.failed) return;
                        const part = live.part++;
                        // The container alone: an unquoted codecs list does not
                        // survive the multipart header.
                        const chunk = new File([blob], "part-" + part, { type: live.type.split(";")[0] });
                        live.sending = live.sending.then(() => sendRecordingPiece(live, part, chunk));
                    }

                    async function sendRecordingPiece(live, part, chunk) {
                        if (live.failed) return;
                        for (let attempt = 1; attempt <= 4; attempt++) {
                            try {
                                await pp.rpc("append_meeting_recording", { id: code, recording: live.id, part, chunk });
                                return;
                            } catch (failure) {
                                const status = failure.status || 0;
                                if (status >= 400 && status < 500 && status !== 408 && status !== 429) break;
                                await new Promise(resolve => setTimeout(resolve, attempt * 1500));
                            }
                        }
                        live.failed = true;
                        onRecordingFailed();
                    }

                    const onRecordingFailed = pp.event(() => {
                        setError("The recording could not reach the server. What was saved so far is being kept.");
                        stopRecording();
                    });

                    const stopRecording = pp.event(async () => {
                        const live = recorder.current;
                        if (!live || live.stopping) return;
                        live.stopping = true;
                        setRecordingBusy(true);
                        try {
                            if (live.media.state !== "inactive") {
                                // The last piece arrives before `stop` does.
                                await new Promise(resolve => { live.media.onstop = resolve; live.media.stop(); });
                            }
                            releaseRecordingMedia(live);
                            await live.sending;
                            await pp.rpc("finish_meeting_recording", { id: code, recording: live.id });
                            if (!live.failed) setToast("Recording stopped. It is being saved to storage.");
                        } catch (failure) {
                            setError(failure.body?.error || failure.message || "The recording could not be finished.");
                        } finally {
                            if (recorder.current === live) recorder.current = null;
                            setRecordingId("");
                            setRoomRecording(false);
                            setRecordingBusy(false);
                            try { sendMeetingEvent({ kind: "recording", on: false }); } catch (failure) { /* left already */ }
                            if (roomPanel === "recordings") loadRecordings();
                        }
                    });

                    async function loadRecordings() {
                        setRecordingsLoading(true);
                        try { setRecordings(await pp.rpc("meeting_recordings", { id: code })); }
                        catch (failure) { setError(failure.body?.error || failure.message || "Recordings could not be listed."); }
                        finally { setRecordingsLoading(false); }
                    }

                    function openRecordings() {
                        dismissRoomMenus();
                        setBackgroundsOpen(false);
                        setRoomPanel("recordings");
                    }

                    // Refreshed while the panel is open, so "Saving…" turns
                    // into a download without a click.
                    pp.effect(() => {
                        if (roomPanel !== "recordings") return;
                        loadRecordings();
                        const timer = setInterval(loadRecordings, 5000);
                        return () => clearInterval(timer);
                    }, [roomPanel]);

                    function recordingSummary(item) {
                        const minutes = Math.floor(item.seconds / 60), seconds = item.seconds % 60;
                        const length = minutes + ":" + seconds.toString().padStart(2, "0");
                        const size = item.bytes >= 1048576 ? (item.bytes / 1048576).toFixed(1) + " MB" : Math.ceil(item.bytes / 1024) + " KB";
                        if (item.state === "recording") return "Recording now · " + length;
                        if (item.state === "uploading") return "Saving to storage… · " + size;
                        if (item.state === "failed") return "Could not be saved";
                        return length + " · " + size;
                    }

                    const retryRecording = pp.event(async function retryRecording(itemId) {
                        try {
                            await pp.rpc("retry_meeting_recording", { id: code, recording: itemId });
                            setToast("Saving the recording to storage again.");
                            loadRecordings();
                        } catch (failure) {
                            setError(failure.body?.error || failure.message || "The recording could not be uploaded again.");
                        }
                    });

                    const openRecording = pp.event(async function openRecording(itemId) {
                        // Opened before the await, while the click still allows a
                        // new window; pointed at the short-lived link once it arrives.
                        const tab = window.open("", "_blank");
                        try {
                            const link = await pp.rpc("meeting_recording_url", { id: code, recording: itemId });
                            if (tab) { tab.opener = null; tab.location.href = link; }
                            else location.assign(link);
                        } catch (failure) {
                            if (tab) tab.close();
                            setError(failure.body?.error || failure.message || "The recording could not be opened.");
                        }
                    });

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

                    const refreshDevices = pp.event(async () => {
                        try {
                            const available = await navigator.mediaDevices.enumerateDevices();
                            setAudioInputs(available.filter(device => device.kind === "audioinput" && device.deviceId));
                            setAudioOutputs(available.filter(device => device.kind === "audiooutput" && device.deviceId));
                            setVideoInputs(available.filter(device => device.kind === "videoinput" && device.deviceId));
                        } catch (failure) {
                            setError("Audio devices could not be listed. Check your browser permissions.");
                        }
                    });
                    pp.effect(() => {
                        refreshDevices();
                        navigator.mediaDevices?.addEventListener("devicechange", refreshDevices);
                        return () => navigator.mediaDevices?.removeEventListener("devicechange", refreshDevices);
                    }, [refreshDevices]);

                    // Chromium lists the default device a second time under these ids. The
                    // empty option already stands for it, so it takes the device's name.
                    const DEVICE_ALIASES = ["default", "communications"];
                    function selectableDevices(devices) {
                        return devices.filter(device => !DEVICE_ALIASES.includes(device.deviceId));
                    }
                    // The device in use, named: a chosen one, or the one the
                    // system default stands for (Chromium lists that default
                    // again under the id "default", sharing its group).
                    function effectiveDevice(current, devices) {
                        const chosen = availableDevice(current, devices);
                        if (chosen) return chosen;
                        const named = selectableDevices(devices);
                        const fallback = devices.find(device => device.deviceId === "default");
                        const same = fallback && named.find(device => device.groupId && device.groupId === fallback.groupId);
                        return same ? same.deviceId : named[0]?.deviceId || "";
                    }
                    function defaultDeviceName(devices, fallback) {
                        const device = devices.find(item => item.deviceId === "default");
                        return device?.label ? device.label.replace(/^Default - /, "") : fallback;
                    }
                    function availableDevice(id, devices) {
                        return selectableDevices(devices).some(device => device.deviceId === id) ? id : "";
                    }

                    function backgroundEffect(id) {
                        if (id === "none" || id === "blur") return { id };
                        const found = backgrounds.find(item => item.id === id);
                        if (!found) throw new Error("Choose a background from the list.");
                        return found;
                    }

                    function handleBackgroundKey(event) {
                        if (event.key !== "Escape") return;
                        if (tileMenu) { setTileMenu(null); event.preventDefault(); }
                        const trigger = reactionsOpen ? reactionTrigger.current : moreOpen ? moreTrigger.current : null;
                        dismissRoomMenus();
                        setBackgroundsOpen(false);
                        if (trigger) {
                            event.preventDefault();
                            trigger.focus();
                        }
                    }

                    // Device settings during a call: the microphone, speaker and
                    // camera change in place, without leaving the meeting.
                    const [videoInputs, setVideoInputs] = pp.state([]);
                    const [cameraDevice, setCameraDevice] = pp.state(devicePreferences.cameraId);
                    const [changingDevice, setChangingDevice] = pp.state(false);
                    const [deviceMenu, setDeviceMenu] = pp.state("");
                    function toggleDeviceMenu(kind) {
                        const opening = deviceMenu !== kind;
                        dismissRoomMenus();
                        setDeviceMenu(opening ? kind : "");
                        if (opening) refreshDevices();
                    }
                    function openDeviceSettings() {
                        dismissRoomMenus();
                        setMoreOpen(false);
                        setBackgroundsOpen(false);
                        setRoomPanel("devices");
                        refreshDevices();
                    }
                    const changeMicrophone = pp.event(async function changeMicrophone(id) {
                        setDeviceMenu("");
                        setMicrophoneDevice(id);
                        const live = call.current;
                        if (!live || audioMode !== "computer") return;
                        setChangingDevice(true);
                        const previous = [meetingMicrophone.current, meetingOutgoingMicrophone.current];
                        try {
                            if (!await startMeetingMicrophone(live.av, id)) { setError("That microphone could not be started."); return; }
                            previous.forEach(stream => stream?.getTracks().forEach(track => track.stop()));
                            if (!microphone) live.av.realtimeMuteLocalAudio();
                            setToast("Microphone changed.");
                        } catch (failure) {
                            setError("That microphone could not be started. Check that no other application is using it.");
                        } finally {
                            setChangingDevice(false);
                        }
                    });
                    const changeSpeaker = pp.event(async function changeSpeaker(id) {
                        setDeviceMenu("");
                        setSpeakerDevice(id);
                        const live = call.current;
                        if (!live || audioMode !== "computer") return;
                        try {
                            await live.av.chooseAudioOutput(id || null);
                            setToast("Speaker changed.");
                        } catch (failure) {
                            await live.av.chooseAudioOutput(null).catch(() => {});
                            setToast("That speaker is unavailable. Using the system default.");
                        }
                    });
                    const changeCamera = pp.event(async function changeCamera(id) {
                        setDeviceMenu("");
                        if (applyingBackground) return;
                        setCameraDevice(id);
                        setChangingDevice(true);
                        setApplyingBackground(true);
                        try {
                            const kit = await ensureDevices();
                            const cameras = await kit.controller.listVideoInputDevices();
                            cameraChoice.current = Math.max(0, cameras.findIndex(device => device.deviceId === id));
                            kit.cameraSelected = true;
                            if (camera) {
                                await closeCamera(); await openCamera();
                                if (call.current) call.current.av.startLocalVideoTile(); else startPreview();
                                setToast("Camera changed.");
                            }
                        } catch (failure) {
                            setError(failure.message || "That camera could not be started.");
                        } finally {
                            setApplyingBackground(false);
                            setChangingDevice(false);
                        }
                    });
                    // Joined with "Don't use audio": turn computer audio on now.
                    const useComputerAudio = pp.event(async function useComputerAudio() {
                        setDeviceMenu("");
                        const live = call.current;
                        if (!live || audioMode === "computer") return;
                        setChangingDevice(true);
                        try {
                            if (!await startMeetingMicrophone(live.av, microphoneDevice)) { setError("No microphone is available. Check your device settings."); return; }
                            if (speakerDevice) await live.av.chooseAudioOutput(speakerDevice).catch(() => live.av.chooseAudioOutput(null));
                            if (meetingAudio.current) await live.av.bindAudioElement(meetingAudio.current);
                            live.av.realtimeMuteLocalAudio();
                            setMicrophone(false);
                            setAudioMode("computer");
                            setToast("Computer audio is on. Your microphone is muted until you unmute.");
                        } catch (failure) {
                            setError("Computer audio could not be started. Check this site's microphone permission.");
                        } finally {
                            setChangingDevice(false);
                        }
                    });

                    function dismissRoomMenus() {
                        if (reactionsOpen) setReactionsOpen(false);
                        if (moreOpen) setMoreOpen(false);
                        if (deviceMenu) setDeviceMenu("");
                    }

                    function toggleRoomMenu(menu) {
                        const opening = menu === "reactions" ? !reactionsOpen : !moreOpen;
                        if (opening && window.innerWidth <= 650) setBackgroundsOpen(false);
                        setReactionsOpen(menu === "reactions" && opening);
                        setMoreOpen(menu === "more" && opening);
                    }

                    // The toolbar can wrap on phones. Keep settings beneath its actual
                    // height and clamp popovers to the stage, clear of the settings pane.
                    const positionRoomMenus = pp.event(() => {
                        if (!session) return;
                        if (backgroundsOpen && (reactionsOpen || moreOpen) && window.innerWidth <= 650) {
                            setBackgroundsOpen(false);
                            return;
                        }
                        const toolbar = roomToolbar.current;
                        if (!toolbar) return;
                        const bottom = Math.ceil(toolbar.getBoundingClientRect().bottom);
                        setToolbarBottom(bottom);
                        const menu = reactionsOpen ? reactionMenu.current : moreOpen ? moreMenu.current : null;
                        const trigger = reactionsOpen ? reactionTrigger.current : moreOpen ? moreTrigger.current : null;
                        if (!menu || !trigger) return;
                        const right = backgroundsOpen && backgroundPanel.current
                            ? backgroundPanel.current.getBoundingClientRect().left - 8 : window.innerWidth - 8;
                        const width = Math.min(menu.getBoundingClientRect().width, Math.max(0, right - 8));
                        const left = Math.max(8, Math.min(trigger.getBoundingClientRect().left, right - width));
                        const top = bottom + 4;
                        setRoomMenuStyle("left:" + left + "px;top:" + top + "px;max-width:" + Math.max(0, right - 8) + "px;max-height:" + Math.max(0, window.innerHeight - top - 8) + "px");
                    });
                    pp.layoutEffect(() => {
                        if (!session) return;
                        positionRoomMenus();
                        const observer = new ResizeObserver(positionRoomMenus);
                        [document.documentElement, roomToolbar.current, backgroundPanel.current, reactionMenu.current, moreMenu.current]
                            .filter(Boolean).forEach(element => observer.observe(element));
                        return () => observer.disconnect();
                    }, [session !== null, backgroundsOpen, reactionsOpen, moreOpen]);

                    function dismissRoomMenusOutside(event) {
                        if (tileMenuElement.current?.contains(event.target)) return;
                        // An options button toggles the menu on its own click; closing it
                        // here first would make that click open it again.
                        if (event.target.closest("[aria-label^='Options for ']")) return;
                        if (tileMenu) setTileMenu(null);
                        if ([reactionTrigger, moreTrigger, reactionMenu, moreMenu].some(ref => ref.current?.contains(event.target))) return;
                        if (event.target.closest(".room-device-anchor")) return;
                        dismissRoomMenus();
                    }

                    function copyRoomLink() {
                        dismissRoomMenus();
                        share();
                    }

                    const selectBackground = pp.event(async function selectBackground(id, custom) {
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
                    });

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
                            tone.onended = () => {
                                context.close();
                                if (speakerTest.current === context) speakerTest.current = null;
                            };
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
                            if (roomLocalVideo.current) {
                                live.av.bindVideoElement(tile.tileId, roomLocalVideo.current);
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
                        setTileDetails(current => withSlot(current, slot, { id: tile.boundAttendeeId, content: !!tile.isContent, name: tile.isContent ? "Shared screen" : attendeeNameFrom(tile.boundExternalUserId) }));
                        setRemoteTiles((current) => withSlot(current, slot, !tile.paused));
                    });

                    const onTileRemoved = pp.event((tileId) => {
                        const slot = tileSlots.current.get(tileId);
                        if (slot === undefined) return;
                        tileSlots.current.delete(tileId);
                        setRemoteTiles((current) => withSlot(current, slot, false));
                        setTileDetails(current => withSlot(current, slot, null));
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
                        // Before the call goes: the last piece is still in the recorder.
                        if (recorder.current) await stopRecording();
                        const live = call.current;
                        const ticket = admission;
                        setAdmission(''); setLobbyWaiting(false);
                        if (ticket) pp.rpc('leave_meeting_entry', {id:code,ticket}).catch(() => {});
                        call.current = null;
                        microphoneLifetime.current += 1;
                        mediaLocked.current = false;
                        meetingMicrophone.current?.getTracks().forEach(track => track.stop());
                        meetingMicrophone.current = null;
                        meetingOutgoingMicrophone.current?.getTracks().forEach(track => track.stop());
                        meetingOutgoingMicrophone.current = null;
                        setMutedNotice(false);
                        tileSlots.current = new Map();
                        setRemoteTiles([false, false, false, false]);
                        setTileDetails([null, null, null, null]);
                        setPeople([]);
                        setMicMuted({});
                        volumeWatched.current = new Set();
                        setRoomPanel("");
                        setBackgroundsOpen(false);
                        setMoreOpen(false);
                        setReactionsOpen(false);
                        setHandRaised(false);
                        setHandLowerAt(0); setHandLowerSeconds(0);
                        handAutoKept.current = false;
                        handSpeech.current = { volume: 0, muted: true, since: 0 };
                        setHandNotice(''); setPinnedId(''); setSpotlightId(''); setHideSelf(devicePreferences.signLanguageMode); setTileMenu(null);
                        setLobbyRequests([]);
                        setRoomRecording(false); setRecordings([]);
                        setReactionNotice("");
                        setSharingScreen(false);
                        setChatMessages([]);
                        setChatDraft("");
                        setChatUnread(0);
                        setElapsed(0);
                        startedAt.current = 0;
                        if (shareStream.current) shareStream.current.getTracks().forEach(track => track.stop());
                        shareStream.current = null;

                        // The camera goes off with the call rather than staying on
                        // over a pre-join screen nobody is looking at — and the
                        // button goes back with it, or it would read "on" above a
                        // picture that is no longer being taken.
                        setCamera(false);

                        try {
                            if (live) live.av.stopLocalVideoTile();
                            await closeCamera();
                            if (live) {
                                live.av.stopContentShare();
                                live.av.realtimeUnsubscribeToAttendeeIdPresence(onPresence);
                                live.av.realtimeUnsubscribeFromReceiveDataMessage("upurnaka-room");
                                live.av.realtimeUnsubscribeFromVolumeIndicator(live.attendeeId);
                                for (const id of volumeWatched.current) live.av.realtimeUnsubscribeFromVolumeIndicator(id, onAttendeeVolume);
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

                        if (!kit.cameraSelected) {
                            cameraChoice.current = Math.max(0, cameras.findIndex(device => device.deviceId === devicePreferences.cameraId));
                            kit.cameraSelected = true;
                        }

                        const input = await filteredCamera(kit, cameras[cameraChoice.current % cameras.length].deviceId, effect);
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
                    async function startMeetingMicrophone(av, deviceId = microphoneDevice) {
                        const lifetime = microphoneLifetime.current;
                        const microphones = await av.listAudioInputDevices();
                        if (!microphones.length) return false;
                        const selected = microphones.find(device => device.deviceId === deviceId) || microphones[0];
                        const input = await navigator.mediaDevices.getUserMedia({video:false, audio:{deviceId:{exact:selected.deviceId}, autoGainControl:devicePreferences.autoGainControl, noiseSuppression:devicePreferences.noiseSuppression, echoCancellation:true}});
                        if (disposed.current || lifetime !== microphoneLifetime.current) { input.getTracks().forEach(track => track.stop()); return false; }
                        const outgoing = devicePreferences.mutedNotification ? input.clone() : input;
                        meetingMicrophone.current = input; meetingOutgoingMicrophone.current = outgoing;
                        try { await av.startAudioInput(outgoing); }
                        catch (failure) { input.getTracks().forEach(track => track.stop()); outgoing.getTracks().forEach(track => track.stop()); throw failure; }
                        if (disposed.current || lifetime !== microphoneLifetime.current) { input.getTracks().forEach(track => track.stop()); outgoing.getTracks().forEach(track => track.stop()); return false; }
                        setCaptureVersion(value => value + 1);
                        mediaLocked.current = false;
                        return true;
                    }
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
                            audioVideoDidStartConnecting: onConnectionStarting,
                            audioVideoDidStart: onConnectionStarted,
                        });
                        av.addContentShareObserver({
                            contentShareDidStop: onShareStopped,
                        });
                        av.realtimeSubscribeToAttendeeIdPresence(onPresence);
                        av.realtimeSubscribeToReceiveDataMessage("upurnaka-room", onRoomMessage);

                        if (audioMode === "computer") {
                            await startMeetingMicrophone(av);
                            if (disposed.current) return;
                            if (speakerDevice) {
                                try { await av.chooseAudioOutput(speakerDevice); }
                                catch (_) { await av.chooseAudioOutput(null); setToast('Selected speaker unavailable. Using system default.'); }
                            }
                            if (meetingAudio.current) {
                                await av.bindAudioElement(meetingAudio.current);
                            }
                        }

                        call.current = { session: meetingSession, av, attendeeId: info.attendee.AttendeeId };
                        av.realtimeSubscribeToVolumeIndicator(info.attendee.AttendeeId, onLocalHandVolume);
                        setPeople([{ id: info.attendee.AttendeeId, name: displayName, initials: initials(displayName), raised: false }]);
                        av.start();

                        if (audioMode === "none") setMicrophone(false);
                        if (!microphone || audioMode === "none") av.realtimeMuteLocalAudio();

                        if (camera) {
                            // The preview was holding the video element; the
                            // meeting's own tile takes it over.
                            stopPreview();
                            await openCamera();
                            av.startLocalVideoTile();
                        }
                    }

                    // The server finds or creates the Dilnaka Meet conference and
                    // mints this person's seat. The join token in the reply
                    // is a credential for this browser and this attendee —
                    // it stays in this state and goes nowhere else.
                    async function joinNow() {
                        if (joining || lobbyWaiting || connecting.current || applyingBackground) return;
                        setJoining(true);
                        setError("");
                        setConnectionStatus("connecting");
                        try {
                            const entry = await pp.rpc('request_meeting_entry', {id:code,name:displayName,passcode:needsPasscode ? passcode.trim() : null});
                            setIsOrganizer(entry.host); setAdmission(entry.id);
                            if (entry.state === 'waiting') setLobbyWaiting(true);
                            else await connectAdmitted(entry.id);
                        } catch (failure) {
                            await teardown();
                            setError(failure.message || "The meeting could not be joined.");
                        } finally {
                            setJoining(false);
                        }
                    }
                    async function connectAdmitted(ticket) {
                        if (connecting.current) return;
                        connecting.current = true; setJoining(true); setLobbyWaiting(false);
                        try {
                            const info = await pp.rpc('join_meeting', {id:code,display_name:displayName,admission:ticket});
                            if (disposed.current) return;
                            await connect(info); setBackgroundsOpen(false); setSession(info); loadChat(ticket);
                        } catch(failure) {
                            setAdmission(''); await teardown(); setError(failure.body?.error || failure.message || 'Could not join this meeting.');
                        } finally { connecting.current = false; setJoining(false); }
                    }

                    async function toggleMicrophone() {
                        const next = !microphone;
                        const live = call.current;
                        if (!live) {setMicrophone(next); return;}
                        if (next) {
                            try { if (admission) await pp.rpc('resume_meeting_audio', {id:code,ticket:admission}); }
                            catch(failure) { setError(failure.body?.error || failure.message); return; }
                            if (mediaLocked.current) {
                                try { if (!await startMeetingMicrophone(live.av)) {setError('No microphone is available. Check your device settings.'); return;} }
                                catch (_) {setError('The microphone could not be restarted. Check your browser permissions.'); return;}
                            }
                            live.av.realtimeUnmuteLocalAudio();
                        } else {
                            live.av.realtimeMuteLocalAudio();
                        }
                        setMicrophone(next);
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
                        await cancelLobby();
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
            .robots(crate::rahti::Robots::new().index(false).follow(false))
            .description(format!("Join {title} on upurnaka.")),
    )
    .into_response())
}

/// Take a seat in the conference behind a meeting link.
///
/// The join code is checked the same way the page checked it: a revoked or
/// unknown one is a 404 here too, because an RPC a browser can call is a
/// second front door and guarding only the page would leave it open.
#[rpc]
pub async fn join_meeting(
    id: String,
    display_name: String,
    admission: Option<String>,
) -> Result<JoinInfo> {
    let id = id.trim().to_string();
    let display_name = display_name.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("id", &id)
        .max_chars("id", &id, 64)
        .required("display_name", &display_name)
        .max_chars("display_name", &display_name, 64);
    validator.finish()?;

    let ticket = if let Some(ticket) = admission {
        ticket
    } else {
        meetings::lobby::request(id.clone(), display_name.clone(), None)
            .await?
            .id
    };
    let visitor = meetings::lobby::claim(&id, &ticket).await?;
    match meetings::join(&id, &visitor.name).await {
        Ok(info) => {
            meetings::lobby::record(
                &ticket,
                &info.meeting.meeting_id,
                &info.attendee.attendee_id,
            )
            .await?;
            Ok(info)
        }
        Err(error) => {
            meetings::lobby::release(&ticket).await;
            Err(error)
        }
    }
}

#[rpc]
pub async fn request_meeting_entry(
    id: String,
    name: String,
    passcode: Option<String>,
) -> Result<meetings::lobby::Admission> {
    meetings::lobby::request(id, name, passcode).await
}

/// The passcode gate: answers `true`, or refuses with why.
#[rpc]
pub async fn check_meeting_passcode(id: String, passcode: String) -> Result<bool> {
    let mut validator = Validator::new();
    validator
        .required("id", &id)
        .max_chars("id", &id, 64)
        .max_chars("passcode", &passcode, 64);
    validator.finish()?;
    meetings::access::check_passcode(&id, &passcode).await
}
#[rpc]
pub async fn meeting_room_state(id: String, ticket: String) -> Result<meetings::lobby::RoomState> {
    meetings::lobby::snapshot(id, ticket).await
}
#[rpc(auth)]
pub async fn decide_meeting_entry(id: String, ticket: String, admit: bool) -> Result<bool> {
    meetings::lobby::decide(id, ticket, admit).await
}
#[rpc]
pub async fn leave_meeting_entry(id: String, ticket: String) -> Result<bool> {
    meetings::lobby::finish(id, ticket).await
}
#[rpc(auth)]
pub async fn spotlight_participant(id: String, attendee: Option<String>) -> Result<bool> {
    meetings::lobby::spotlight(id, attendee).await
}
#[rpc(auth)]
pub async fn mute_participant(id: String, attendee: String) -> Result<bool> {
    meetings::lobby::mute(id, attendee).await
}
#[rpc(auth)]
pub async fn mute_all_participants(id: String) -> Result<u64> {
    meetings::lobby::mute_all(id).await
}
#[rpc]
pub async fn meeting_chat_history(
    id: String,
    ticket: String,
) -> Result<Vec<meetings::chat::Message>> {
    meetings::chat::history(id, ticket).await
}

#[rpc]
pub async fn send_meeting_chat(
    id: String,
    ticket: String,
    text: String,
    file: Option<String>,
) -> Result<meetings::chat::Message> {
    meetings::chat::send(id, ticket, text, file).await
}

/// Upload a file from this device to the sender's My files, to share it.
#[rpc]
pub async fn upload_meeting_chat_file(
    id: String,
    ticket: String,
    file: crate::rahti::RpcFile,
) -> Result<crate::chat_files::Attachment> {
    meetings::chat::upload(id, ticket, file).await
}

/// The sender's My files, to choose one to share in the chat.
#[rpc]
pub async fn meeting_chat_my_files(
    id: String,
    ticket: String,
    folder: Option<String>,
) -> Result<crate::chat_files::Listing> {
    meetings::chat::my_files(id, ticket, folder).await
}

/// A short-lived download link for a file shared in this meeting's chat.
#[rpc]
pub async fn meeting_chat_file_url(id: String, ticket: String, message: String) -> Result<String> {
    meetings::chat::file_link(id, ticket, message).await
}

#[rpc]
pub async fn resume_meeting_audio(id: String, ticket: String) -> Result<bool> {
    meetings::lobby::resume_audio(id, ticket).await
}

/// Start recording the meeting. The organizer only, from a seat in the call.
#[rpc(auth)]
pub async fn start_meeting_recording(
    id: String,
    ticket: String,
    content_type: String,
) -> Result<recording::Started> {
    recording::start(id, ticket, content_type).await
}
/// One `MediaRecorder` timeslice. `chunk` streams to disk as it arrives.
#[rpc(auth)]
pub async fn append_meeting_recording(
    id: String,
    recording: String,
    part: i32,
    chunk: RpcUpload,
) -> Result<bool> {
    recording::append(id, recording, part, chunk).await
}
/// Stop taking pieces; the upload to Dilnaka Storage carries on without the page.
#[rpc(auth)]
pub async fn finish_meeting_recording(id: String, recording: String) -> Result<bool> {
    recording::finish(id, recording).await
}
#[rpc(auth)]
pub async fn meeting_recordings(id: String) -> Result<Vec<recording::Listed>> {
    recording::list(id).await
}
#[rpc(auth)]
pub async fn meeting_recording_url(id: String, recording: String) -> Result<String> {
    recording::access(id, recording).await
}
/// Upload a recording whose upload failed again.
#[rpc(auth)]
pub async fn retry_meeting_recording(id: String, recording: String) -> Result<bool> {
    recording::retry(id, recording).await
}
