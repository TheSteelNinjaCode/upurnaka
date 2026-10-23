//! Where a meeting link lands: the pre-join screen for one join code.
//!
//! `/meet/j/{id}`. This is the half of a meeting link that makes it a link at
//! all — somebody who was sent one opens it here, sets their camera and
//! microphone the way they want them, and joins.
//!
//! The call itself is not connected yet. `crate::meetings::provision` is the
//! seam a real provider plugs into, and until one does, "Join now" puts the
//! reader in the lobby and says so rather than pretending.

use axum::extract::Path;
use axum::http::StatusCode;

use crate::components::app_shell::AppShell;
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::meetings;
use crate::rahti::{Error, Html, Json, Metadata, Page, Result, html};

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
                        class="mt-7 flex aspect-video w-full flex-col items-center justify-center gap-4 rounded-xl border border-edge bg-sunken"
                    >
                        <Avatar initials="JA" size="xl" presence="online" />
                        <p class="text-[13px] text-muted" hidden={camera}>
                            "Your camera is turned off"
                        </p>
                        <p class="text-[13px] text-muted" hidden={!camera}>
                            "Camera preview appears here once a meeting provider is connected."
                        </p>
                    </div>
                    <div class="mt-5 flex items-center justify-center gap-3">
                        <button
                            type="button"
                            class={microphone ? CONTROL_ON : CONTROL_OFF}
                            aria-pressed={microphone}
                            onclick={setMicrophone(on => !on)}
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
                            onclick={setCamera(on => !on)}
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
                    <div class="mt-7 flex items-center justify-center gap-3" hidden={joined}>
                        <button
                            type="button"
                            class="h-10 rounded-md bg-brand px-8 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                            onclick={setJoined(true)}
                        >
                            "Join now"
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
                    <div
                        class="mt-7 rounded-lg border border-edge bg-raised px-6 py-5 text-center"
                        hidden={!joined}
                    >
                        <p class="text-[15px] font-semibold text-ink">"You're in the lobby"</p>
                        <p class="mt-2 text-[13px] text-muted">
                            "The link resolves and the meeting is real, but no call is dialled yet — a meeting provider has still to be connected."
                        </p>
                        <button
                            type="button"
                            class="mt-5 h-9 rounded-md border border-edge bg-surface px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                            onclick={setJoined(false)}
                        >
                            "Leave"
                        </button>
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

                    const [microphone, setMicrophone] = pp.state(true);
                    const [camera, setCamera] = pp.state(false);
                    const [joined, setJoined] = pp.state(false);
                    const [toast, setToast] = pp.state("");

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
