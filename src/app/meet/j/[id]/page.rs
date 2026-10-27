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

use crate::components::aria::{bound, labelled, named, submit};
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::{Attrs, ui_expr};
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::label::Label;
use crate::components::rahti_ui::radio_group::{RadioGroup, RadioGroupItem};
use crate::components::rahti_ui::select::{Select, SelectItem};
use crate::components::rahti_ui::switch::Switch;
use crate::meetings::{self, JoinInfo, chime, recording};
use crate::rahti::{Error, Html, Json, Metadata, Page, Result, RpcUpload, Validator, html, rpc};

/// A tool in the meeting toolbar: an icon over its label, underlined while on.
const ROOM_TOOL: &str = "h-[60px]! min-w-14 flex-col gap-[5px] px-2.5! py-1.5 text-xs font-normal \
                         text-ink aria-pressed:bg-hover \
                         aria-pressed:shadow-[inset_0_-2px_var(--color-brand-ink)] \
                         max-[1000px]:min-w-[45px] max-[1000px]:p-1.5! max-[1000px]:text-[11px] \
                         max-[650px]:h-[52px]! max-[650px]:min-w-10 \
                         [&_svg]:size-6! max-[650px]:[&_svg]:size-5!";

/// Leaving the meeting: the toolbar tool in the danger colour.
const ROOM_LEAVE: &str = "ml-3.5 text-danger-ink! hover:bg-destructive/40! max-[1000px]:ml-1";

/// A device picker on the pre-join screen.
const DEVICE_SELECT: &str = "w-full min-w-0 border-transparent bg-transparent text-base shadow-none \
                             dark:bg-transparent \
                             min-[1101px]:group-data-[backgrounds=true]/body:text-sm";

/// A background tile in the background panel, outlined while chosen.
const BACKGROUND_OPTION: &str = "meeting-background-option relative h-[70px]! min-w-0 flex-col \
                                 gap-[3px] overflow-hidden rounded-[3px]! border-2 \
                                 border-transparent bg-raised p-0! text-xs font-normal \
                                 aria-pressed:border-brand-ink";

/// A row in a menu the room draws itself: the reactions, More, a tile's options.
const MENU_ROW: &str = "w-full justify-start! px-4! py-2.5 text-left text-[13px] font-normal";

/// An action that reads as a link.
const LINK: &str = "h-auto! px-0! text-link";

pub async fn page(Path(id): Path<String>) -> Result<Page<Html>> {
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

    // Decided on the server, because it is a deployment fact rather than
    // something the browser could discover: with no provider configured the
    // button must not pretend there is a call to join.
    let provider_ready = chime::is_configured();
    let recording_ready = recording::is_configured();

    let room_leave = format!("{ROOM_TOOL} {ROOM_LEAVE}");
    let cancel_join = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Lg,
        "min-w-[118px] px-[22px] text-base font-semibold",
    );
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
            <div
                class="meeting-body group/body mx-auto max-w-[1460px] px-5 pb-6 min-[851px]:flex min-[851px]:h-full min-[851px]:flex-col min-[851px]:gap-[clamp(10px,1.8dvh,20px)] min-[851px]:px-[clamp(24px,4vw,64px)] min-[851px]:pb-4 min-[1101px]:data-[backgrounds=true]:ml-0 min-[1101px]:data-[backgrounds=true]:w-[calc(100%-340px)] min-[1101px]:data-[backgrounds=true]:px-6"
                data-backgrounds={backgroundsOpen}
                hidden={session !== null}
            >
                <div
                    class="meeting-account relative mx-auto flex w-fit max-w-full items-center gap-4 rounded-b-md bg-raised px-6 py-5 shadow-md shadow-black/20 max-[480px]:gap-2.5 max-[480px]:p-3.5 min-[851px]:shrink-0 min-[851px]:py-[clamp(12px,1.8dvh,20px)]"
                >
                    <Avatar initials=@{initials.as_str()} size="lg" presence="none" />
                    <div
                        class="flex min-w-40 flex-col gap-0.5 text-[15px] max-[480px]:min-w-0 max-[480px]:text-[13px]"
                    >
                        <strong>{displayName || "Your name"}</strong>
                        <span class="text-[13px] text-muted-foreground">"Joining as a guest"</span>
                    </div>
                    <Button variant="link" class=@{LINK} onclick={setEditingName(!editingName)}>
                        {editingName ? "Done" : "Change"}
                    </Button>
                    <div
                        class="meeting-name-editor absolute inset-x-0 top-full z-2 grid gap-2 bg-raised p-4 text-[13px] shadow-lg shadow-black/30"
                        hidden={!editingName}
                    >
                        <Label r#for="meeting-display-name" class="font-normal">"Your name"</Label>
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
                                class="absolute inset-0 h-full w-full -scale-x-100 object-cover"
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
                                    <Icon name="mic" class="h-6 w-6" />
                                    <Select
                                        class=@{DEVICE_SELECT}
                                        attrs=@{named("Microphone device")}
                                        value={microphoneDevice}
                                        onfocus={refreshDevices()}
                                        onchange={setMicrophoneDevice(target.value)}
                                        disabled={session !== null}
                                    >
                                        <SelectItem value="">"Default microphone"</SelectItem>
                                        <template pp-for="device in audioInputs">
                                            <SelectItem
                                                key={device.deviceId}
                                                value={device.deviceId}
                                            >
                                                {device.label || "Microphone"}
                                            </SelectItem>
                                        </template>
                                    </Select>
                                    <Switch
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
                                        value={speakerDevice}
                                        onfocus={refreshDevices()}
                                        onchange={setSpeakerDevice(target.value)}
                                        disabled={session !== null}
                                    >
                                        <SelectItem value="">"Default speakers"</SelectItem>
                                        <template pp-for="device in audioOutputs">
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
                                    class="mb-0.5 ml-auto mt-4 flex h-auto! px-0! text-[15px] text-link min-[851px]:mt-2.5"
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
                    <a href="/meet" class=@{cancel_join.as_str()}>"Cancel"</a>
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
                            attrs=@{named("Meeting chat").merge(&bound("aria-pressed", ui_expr!(aria-pressed={roomPanel === "chat"})))}
                            onclick={toggleRoomPanel("chat")}
                        >
                            <Icon name="chat" class="h-6 w-6 max-[650px]:h-5 max-[650px]:w-5" />
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
                        <Button
                            variant="ghost"
                            class=@{ROOM_TOOL}
                            attrs=@{named("Share screen").merge(&bound("aria-pressed", ui_expr!(aria-pressed={sharingScreen})))}
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
                                        {slot.name}
                                        <Button
                                            variant="ghost"
                                            size="icon-xs"
                                            class="ml-2 size-5! text-white hover:bg-white/15"
                                            attrs=@{bound("aria-label", ui_expr!(aria-label={"Options for " + slot.name}))}
                                            hidden={slot.content}
                                            value={slot.attendeeId}
                                            name={slot.name}
                                            onclick={openTileMenu(event.currentTarget.value, event.currentTarget.name)}
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
                                        {person.name}
                                        <Button
                                            variant="ghost"
                                            size="icon-xs"
                                            class="ml-2 size-5! text-white hover:bg-white/15"
                                            attrs=@{bound("aria-label", ui_expr!(aria-label={"Options for " + person.name}))}
                                            value={person.id}
                                            name={person.name}
                                            onclick={openTileMenu(event.currentTarget.value, event.currentTarget.name)}
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
                                class={tileFits[selfAttendeeId] === false ? "room-local-video absolute inset-0 h-full w-full -scale-x-100 object-cover" : "room-local-video absolute inset-0 h-full w-full -scale-x-100 object-contain"}
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
                                {displayName}
                                " (You)"
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    class="ml-2 size-5! text-white hover:bg-white/15"
                                    attrs=@{labelled("Options for your video")}
                                    onclick={openTileMenu(selfAttendeeId, displayName)}
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
                            class="room-tile-menu absolute bottom-16 left-5 z-5 grid w-64 gap-1 rounded-lg border border-edge bg-raised p-2 text-sm shadow-2xl"
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
                                hidden={!isOrganizer || !tileMenu || tileMenu.id === selfAttendeeId}
                                attrs=@{Attrs::new().set("data-action", "mute")}
                                onclick={muteTile()}
                            >
                                "Mute participant"
                            </Button>
                            <Button variant="ghost" class=@{MENU_ROW} onclick={setTileMenu(null)}>
                                "Close"
                            </Button>
                        </section>
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
                                {roomPanel === "people" ? "People (" + people.length + ")" : roomPanel === "recordings" ? "Recordings" : "Meeting chat"}
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
                            <p class="my-3 text-xs text-subtle">"In this meeting"</p>
                            <template pp-for="person in people">
                                <div
                                    key={person.id}
                                    class="my-[18px] flex items-center gap-3 text-[13px]"
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
                                    <Button
                                        variant="ghost"
                                        size="icon-xs"
                                        attrs=@{bound("aria-label", ui_expr!(aria-label={"Options for " + person.name}))}
                                        value={person.id}
                                        name={person.name}
                                        onclick={openTileMenu(event.currentTarget.value, event.currentTarget.name)}
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
                            >
                                <p
                                    class="text-[13px] leading-relaxed text-muted-foreground"
                                    hidden={chatMessages.length > 0}
                                >
                                    "Chat with everyone in this meeting. Messages are available during the call."
                                </p>
                                <template pp-for="message in chatMessages">
                                    <article
                                        key={message.id}
                                        class="rounded-md bg-raised p-3 text-[13px] wrap-anywhere"
                                    >
                                        <strong class="text-xs text-link">{message.name}</strong>
                                        <p class="mt-[5px] whitespace-pre-wrap">{message.text}</p>
                                    </article>
                                </template>
                            </div>
                            <form class="m-4 flex gap-2" onsubmit={sendChat(event)}>
                                <Input
                                    placeholder="Type a message"
                                    class="min-w-0 flex-1 text-[13px]"
                                    attrs=@{named("Message everyone").set("maxlength", "500")}
                                    value={chatDraft}
                                    oninput={setChatDraft(target.value)}
                                />
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    attrs=@{labelled("Send meeting message").merge(&submit())}
                                    disabled={!chatDraft.trim() || connectionStatus !== "connected"}
                                >
                                    <Icon name="send" class="size-5" />
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
                                r#type="file"
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
            <div
                class="fixed bottom-6 left-6 rounded-md border border-edge bg-raised px-5 py-3.5 text-sm"
                role="status"
                hidden={!toast}
            >
                {toast}
            </div>
            <script>r##"

                    // Repeated class lists. The pre-join body carries
                    // `data-backgrounds`, so its children tighten up through
                    // `group-data-[backgrounds=true]/body` when the background
                    // panel takes 340px of a wide window.
                    const AUDIO_CARD = "overflow-hidden rounded-md border border-edge bg-raised";
                    const AUDIO_CHOICE = "flex min-h-[66px] cursor-pointer items-center gap-4 px-[26px] py-[23px] text-[17px] max-[480px]:p-5 min-[851px]:min-h-14 min-[851px]:shrink-0 min-[851px]:px-6 min-[851px]:py-4 min-[851px]:text-base min-[1101px]:group-data-[backgrounds=true]/body:px-4";
                    const DEVICE_ROW = "flex min-h-[47px] items-center gap-4 text-ink min-[851px]:min-h-[42px] min-[1101px]:group-data-[backgrounds=true]/body:gap-2";
                    const DEVICE_SELECT = "w-full min-w-0 text-ellipsis bg-transparent py-1.5 text-base text-ink [&_option]:bg-raised min-[1101px]:group-data-[backgrounds=true]/body:text-sm";
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
                    const [tileDetails, setTileDetails] = pp.state([null, null, null, null]);
                    const [people, setPeople] = pp.state([]);
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
                    const [hideSelf, setHideSelf] = pp.state(false);
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
                    const remoteSlots = remoteRefs.map((ref, index) => ({
                        index,
                        ref,
                        active: remoteTiles[index] || !!tileDetails[index]?.content,
                        paused: !remoteTiles[index],
                        name: tileDetails[index] ? tileDetails[index].name : "Participant",
                        content: tileDetails[index] ? tileDetails[index].content : false,
                        attendeeId: tileDetails[index] ? tileDetails[index].id : "",
                        raised: !!tileDetails[index] && !tileDetails[index].content && people.some(person => person.id === tileDetails[index].id && person.raised),
                    }));
                    const selfAttendeeId = session ? session.attendee.AttendeeId : "";
                    const remotePeople = people.filter(person => person.id !== selfAttendeeId).map(person => ({
                        ...person,
                        hasVideo: tileDetails.some((tile, index) => remoteTiles[index] && tile && tile.id === person.id),
                    }));
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

                    const onLocalHandVolume = pp.event((attendeeId, volume, muted) => {
                        const live = call.current;
                        if (!live || attendeeId !== live.attendeeId) return;
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

                    const applyRoomState = pp.event(async (result, ticket) => {
                        setLobbyRequests(result.waiting);
                        setSpotlightId(result.spotlight || '');
                        setRoomRecording(!!result.recording);
                        if (result.muted_by_host && microphone && call.current) {
                            call.current.av.realtimeMuteLocalAudio(); setMicrophone(false);
                            setToast('The organizer muted your microphone. You can unmute when ready.');
                        }
                        if (result.state === 'admitted' && !session && !connecting.current) await connectAdmitted(ticket);
                        if (result.state === 'denied' || result.state === 'left') {
                            setLobbyWaiting(false); setAdmission('');
                            if (session) { await teardown(); setSession(null); }
                            setError(result.state === 'denied' ? 'The organizer declined your request to join.' : 'Your meeting entry has ended.');
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
                                if (!cancelled) setError(failure.body?.error || failure.message || 'Could not update meeting controls.');
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
                    const openTileMenu = pp.event(function openTileMenu(id, name) { setMoreOpen(false); setReactionsOpen(false); setTileMenu(tileMenu?.id === id ? null : {id,name}); });
                    function pinTile() { if (tileMenu) setPinnedId(pinnedId === tileMenu.id ? '' : tileMenu.id); setTileMenu(null); }
                    function fitTile() { if (tileMenu) { const id = tileMenu.id; setTileFits(current => ({...current,[id]:current[id] === false})); } setTileMenu(null); }
                    function hideMyTile() { setHideSelf(true); if (pinnedId === selfAttendeeId) setPinnedId(''); setTileMenu(null); }
                    async function spotlightTile() {
                        if (!tileMenu) return;
                        const attendee = spotlightId === tileMenu.id ? null : tileMenu.id;
                        try { await pp.rpc('spotlight_participant', {id:code,attendee}); setSpotlightId(attendee || ''); setPinnedId(''); setTileMenu(null); }
                        catch(failure) { setError(failure.body?.error || failure.message); }
                    }
                    async function muteTile() {
                        if (!tileMenu) return;
                        try { await pp.rpc('mute_participant', {id:code,attendee:tileMenu.id}); setTileMenu(null); setToast('Participant muted.'); }
                        catch(failure) { setError(failure.body?.error || failure.message); }
                    }
                    async function switchCamera() {
                        setTileMenu(null);
                        if (applyingBackground) return;
                        setApplyingBackground(true);
                        try {
                            const kit = await ensureDevices();
                            const cameras = await kit.controller.listVideoInputDevices();
                            if (cameras.length < 2) { setToast('No other camera is available.'); return; }
                            cameraChoice.current = (cameraChoice.current + 1) % cameras.length;
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

                    const onPresence = pp.event((id, present, externalId) => {
                        if (id.includes("#content")) return;
                        if (!present) {
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

                    const onRoomMessage = pp.event(message => {
                        if (message.throttled || message.data.length > 4096) return;
                        try {
                            const payload = message.json();
                            const sender = message.senderAttendeeId;
                            if (!sender || sender.includes("#content")) return;
                            const name = attendeeNameFrom(message.senderExternalUserId);
                            if (payload.kind === "chat" && typeof payload.text === "string" && payload.text.trim() && typeof payload.id === "string") {
                                appendChat({ id: sender + ":" + payload.id.slice(0, 64), name, text: payload.text.slice(0, 500) });
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

                    function sendChat(event) {
                        event.preventDefault();
                        const text = chatDraft.trim().slice(0, 500);
                        if (!text) return;
                        try {
                            const id = crypto.randomUUID();
                            sendMeetingEvent({ kind: "chat", id, text });
                            appendChat({ id: selfAttendeeId + ":" + id, name: displayName, text });
                            setChatDraft("");
                        } catch (failure) { setError(failure.message); }
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

                    async function toggleScreenShare() {
                        const live = call.current;
                        if (!live || startingShare) return;
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

                    function dismissRoomMenus() {
                        if (reactionsOpen) setReactionsOpen(false);
                        if (moreOpen) setMoreOpen(false);
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
                        if (tileMenu) setTileMenu(null);
                        if ([reactionTrigger, moreTrigger, reactionMenu, moreMenu].some(ref => ref.current?.contains(event.target))) return;
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
                        tileSlots.current = new Map();
                        setRemoteTiles([false, false, false, false]);
                        setTileDetails([null, null, null, null]);
                        setPeople([]);
                        setRoomPanel("");
                        setBackgroundsOpen(false);
                        setMoreOpen(false);
                        setReactionsOpen(false);
                        setHandRaised(false);
                        setHandLowerAt(0); setHandLowerSeconds(0);
                        handAutoKept.current = false;
                        handSpeech.current = { volume: 0, muted: true, since: 0 };
                        setHandNotice(''); setPinnedId(''); setSpotlightId(''); setHideSelf(false); setTileMenu(null);
                        setLobbyRequests([]);
                        setRoomRecording(false); setRecordings([]);
                        setReactionNotice("");
                        setSharingScreen(false);
                        setChatMessages([]);
                        setChatDraft("");
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
                            const microphones = await av.listAudioInputDevices();
                            if (microphones.length > 0) {
                                await av.startAudioInput(microphoneDevice || microphones[0].deviceId);
                            }
                            if (speakerDevice) await av.chooseAudioOutput(speakerDevice);
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

                    // The server finds or creates the Chime conference and
                    // mints this person's seat. The join token in the reply
                    // is a credential for this browser and this attendee —
                    // it stays in this state and goes nowhere else.
                    async function joinNow() {
                        if (joining || lobbyWaiting || connecting.current || applyingBackground) return;
                        setJoining(true);
                        setError("");
                        setConnectionStatus("connecting");
                        try {
                            const entry = await pp.rpc('request_meeting_entry', {id:code,name:displayName});
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
                            await connect(info); setBackgroundsOpen(false); setSession(info);
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
            .description(format!("Join {title} on upurnaka.")),
    ))
}

/// Take a seat in the conference behind a meeting link.
///
/// The join code is checked the same way the page checked it: a revoked or
/// unknown one is a 404 here too, because an RPC a browser can call is a
/// second front door and guarding only the page would leave it open.
#[rpc(auth)]
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
        meetings::lobby::request(id.clone(), display_name.clone())
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

#[rpc(auth)]
pub async fn request_meeting_entry(id: String, name: String) -> Result<meetings::lobby::Admission> {
    meetings::lobby::request(id, name).await
}
#[rpc(auth)]
pub async fn meeting_room_state(id: String, ticket: String) -> Result<meetings::lobby::RoomState> {
    meetings::lobby::snapshot(id, ticket).await
}
#[rpc(auth)]
pub async fn decide_meeting_entry(id: String, ticket: String, admit: bool) -> Result<bool> {
    meetings::lobby::decide(id, ticket, admit).await
}
#[rpc(auth)]
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
