//! Meet: the three ways a meeting starts, and the links this account owns.

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::meetings::{self, MeetingLink};
use crate::rahti::{Json, Metadata, Result, Validator, html, rpc};

/// What the title box is filled in with, the way Teams fills it in with the
/// organizer's name.
const ACCOUNT_NAME: &str = "Jefferson Abraham";

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Meet")
        .description("Create a meeting link, schedule a meeting, or join with an ID.")
}

pub async fn page() -> Result {
    // Rendered from the store, so a reload shows the same list rather than an
    // empty one: the cards below are seeded from this and only added to by
    // the browser afterwards.
    let links = meetings::list().await?;
    let suggested_title = format!("Meeting with {ACCOUNT_NAME}");

    Ok(html! {
        <AppShell active="meet">
            <main
                class="scroll-thin relative min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="mx-auto w-full max-w-[1180px] px-14 py-9">
                    <h1 class="text-[22px] font-semibold tracking-tight text-ink">"Meet"</h1>
                    <div class="mt-7 grid grid-cols-3 gap-5">
                        <div class="relative">
                            <button
                                type="button"
                                class="flex h-12 w-full items-center justify-center gap-3 rounded-md bg-brand text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                                onclick={toggleCompose()}
                            >
                                <Icon name="link" class="h-[18px] w-[18px]" />
                                "Create a meeting link"
                            </button>
                            <div
                                class="absolute left-0 top-full z-40 mt-1 w-full rounded-md border border-edge bg-raised p-4 shadow-2xl shadow-black/70"
                                hidden={!compose}
                            >
                                <div class="flex items-start justify-between gap-3">
                                    <p class="text-[13px] font-semibold text-ink">
                                        "Give your meeting a title"
                                    </p>
                                    <button
                                        type="button"
                                        class="-mr-1 -mt-1 flex h-6 w-6 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                        aria-label="Close"
                                        onclick={closeCompose()}
                                    >
                                        <Icon name="close" class="h-4 w-4" />
                                    </button>
                                </div>
                                <form class="mt-3" onsubmit={createLink(event)} novalidate>
                                    <input
                                        name="title"
                                        type="text"
                                        class="h-9 w-full rounded-t bg-hover px-3 text-[13px] text-ink placeholder:text-subtle focus:outline-none"
                                        style="border-bottom: 2px solid var(--color-brand)"
                                        aria-label="Meeting title"
                                        autocomplete="off"
                                        placeholder="Meeting title"
                                        value={title}
                                        oninput={setTitle(target.value)}
                                    />
                                    <p class="mt-2 text-xs text-busy" hidden={!error}>{error}</p>
                                    <button
                                        type="submit"
                                        class="mt-4 h-9 w-full rounded bg-brand text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                        disabled={creating || title.trim().length === 0}
                                    >
                                        {creating ? "Creating…" : "Create and copy link"}
                                    </button>
                                </form>
                            </div>
                        </div>
                        <a
                            href="/calendar"
                            class="flex h-12 items-center justify-center gap-3 rounded-md border border-edge bg-raised text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                        >
                            <Icon name="calendar-day" class="h-[18px] w-[18px]" />
                            "Schedule a meeting"
                        </a>
                        <button
                            type="button"
                            class="flex h-12 items-center justify-center gap-3 rounded-md border border-edge bg-raised text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                            onclick={toggleJoin()}
                        >
                            <Icon name="hash" class="h-[18px] w-[18px]" />
                            "Join with a meeting ID"
                        </button>
                    </div>
                    <div
                        class="mt-4 rounded-md border border-edge bg-raised px-5 py-4"
                        hidden={!joinOpen}
                    >
                        <form class="flex items-center gap-3" onsubmit={join(event)} novalidate>
                            <input
                                name="meeting"
                                type="text"
                                class="h-9 flex-1 rounded border border-edge bg-surface px-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                                placeholder="Enter a meeting ID"
                                aria-label="Meeting ID"
                                autocomplete="off"
                                value={meetingId}
                                oninput={setMeetingId(target.value)}
                            />
                            <button
                                type="submit"
                                class="h-9 rounded bg-brand px-5 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                disabled={meetingId.trim().length === 0}
                            >
                                "Join"
                            </button>
                        </form>
                    </div>
                    <h2 class="mt-10 text-[15px] font-semibold text-ink">"Meeting links"</h2>
                    <div class="mt-4 grid grid-cols-2 gap-5">
                        <template pp-for="link in links">
                            <div
                                key={link.id}
                                class="relative rounded-lg border border-edge bg-raised px-5 py-4"
                            >
                                <div class="flex items-start justify-between">
                                    <span
                                        class="flex h-9 w-9 items-center justify-center rounded-full bg-brand-soft text-brand-ink"
                                    >
                                        <Icon name="link" class="h-[18px] w-[18px]" />
                                    </span>
                                    <button
                                        type="button"
                                        class="-mr-2 flex h-8 w-8 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                        aria-label="More options"
                                        onclick={openMenu(link.id)}
                                    >
                                        <Icon name="more" class="h-[18px] w-[18px]" />
                                    </button>
                                </div>
                                <p class="mt-3 truncate text-[15px] font-semibold text-ink">
                                    {link.title}
                                </p>
                                <p class="mt-1 text-[13px] text-muted">"Created "{link.created}</p>
                                <div class="mt-4 flex items-center gap-3">
                                    <a
                                        href={link.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="flex h-9 flex-1 items-center justify-center rounded bg-hover text-[13px] font-semibold text-ink transition-colors hover:bg-edge"
                                    >
                                        "Join"
                                    </a>
                                    <button
                                        type="button"
                                        class="h-9 flex-1 rounded bg-hover text-[13px] font-semibold text-ink transition-colors hover:bg-edge"
                                        onclick={shareLink(link.url)}
                                    >
                                        "Share link"
                                    </button>
                                </div>
                                <div
                                    class="absolute right-4 top-12 z-40 w-44 rounded-md border border-edge bg-raised p-1 shadow-2xl shadow-black/70"
                                    hidden={menuFor !== link.id}
                                >
                                    <button
                                        type="button"
                                        class="flex h-9 w-full items-center gap-3 rounded px-3 text-left text-[13px] text-ink transition-colors hover:bg-hover"
                                        onclick={shareLink(link.url)}
                                    >
                                        <Icon name="copy" class="h-4 w-4" />
                                        "Copy link"
                                    </button>
                                    <button
                                        type="button"
                                        class="flex h-9 w-full items-center gap-3 rounded px-3 text-left text-[13px] text-busy transition-colors hover:bg-hover"
                                        onclick={removeLink(link.id)}
                                    >
                                        <Icon name="trash" class="h-4 w-4" />
                                        "Delete"
                                    </button>
                                </div>
                            </div>
                        </template>
                        <div class={links.length === 0 ? EMPTY_WIDE : EMPTY}>
                            <p class="text-3xl leading-none">"🔗"</p>
                            <p class="mt-5 text-[15px] font-semibold text-ink">
                                "Quickly create, save, and share links with anyone."
                            </p>
                            <p class="mt-5">
                                <a href="/apps" class="text-[13px] text-link hover:underline">
                                    "Learn more about meeting links"
                                </a>
                            </p>
                        </div>
                    </div>
                    <p class="mt-3 text-xs text-busy" hidden={!listError}>{listError}</p>
                    <div class="mt-11 flex items-center justify-between">
                        <h2 class="text-[15px] font-semibold text-ink">"Scheduled meetings"</h2>
                        <a
                            href="/calendar"
                            class="flex items-center gap-2 text-[13px] text-link hover:underline"
                        >
                            <Icon name="calendar-day" class="h-4 w-4" />
                            "Open calendar view"
                        </a>
                    </div>
                    <p class="mt-4 text-[13px] text-subtle">
                        "Nothing is scheduled. Meetings you accept show up here."
                    </p>
                </div>
                <div
                    class="fixed inset-0 z-30"
                    hidden={menuFor === "" && !compose}
                    onclick={dismiss()}
                >
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
                    const EMPTY = "rounded-lg border border-edge bg-raised px-8 py-7";
                    const EMPTY_WIDE = EMPTY + " col-span-2";

                    const [links, setLinks] = pp.state(@{Json(&links)});
                    const [compose, setCompose] = pp.state(false);
                    const [title, setTitle] = pp.state(@{Json(&suggested_title)});
                    const [creating, setCreating] = pp.state(false);
                    const [error, setError] = pp.state("");
                    const [listError, setListError] = pp.state("");
                    const [menuFor, setMenuFor] = pp.state("");
                    const [joinOpen, setJoinOpen] = pp.state(false);
                    const [meetingId, setMeetingId] = pp.state("");
                    const [toast, setToast] = pp.state("");

                    // One timer, restarted whenever a new notice replaces the
                    // one on screen, and cleared when the page goes away.
                    pp.effect(() => {
                        if (!toast) return;
                        const timer = setTimeout(() => setToast(""), 4000);
                        return () => clearTimeout(timer);
                    }, [toast]);

                    function toggleCompose() {
                        setMenuFor("");
                        setError("");
                        setCompose((open) => !open);
                    }

                    function closeCompose() {
                        setCompose(false);
                        setError("");
                    }

                    function toggleJoin() {
                        setCompose(false);
                        setMenuFor("");
                        setJoinOpen((open) => !open);
                    }

                    function openMenu(id) {
                        setCompose(false);
                        setMenuFor((current) => (current === id ? "" : id));
                    }

                    // The backdrop sits under both the popover and a card
                    // menu, so one click closes whichever is open.
                    function dismiss() {
                        setCompose(false);
                        setMenuFor("");
                    }

                    // The server hands out a path, because it cannot know
                    // which host this browser used to reach it. The absolute
                    // URL is only ever needed here, at the moment of copying.
                    function absolute(path) {
                        return location.origin + path;
                    }

                    async function copy(text, notice) {
                        try {
                            await navigator.clipboard.writeText(text);
                            setToast(notice);
                        } catch (failure) {
                            setToast("Copy the link from the address bar: " + text);
                        }
                    }

                    async function createLink(event) {
                        event.preventDefault();
                        if (creating) return;

                        const payload = Object.fromEntries(new FormData(event.currentTarget));
                        setCreating(true);
                        setError("");
                        try {
                            const link = await pp.rpc("create_meeting_link", payload);
                            setLinks((current) => [link].concat(current));
                            setCompose(false);
                            await copy(absolute(link.url), "Link copied to clipboard");
                        } catch (failure) {
                            setError(failure.message || "The link could not be created.");
                        } finally {
                            setCreating(false);
                        }
                    }

                    function shareLink(url) {
                        setMenuFor("");
                        copy(absolute(url), "Link copied to clipboard");
                    }

                    async function removeLink(id) {
                        setMenuFor("");
                        const kept = links;
                        setLinks((current) => current.filter((link) => link.id !== id));
                        try {
                            await pp.rpc("delete_meeting_link", { id });
                            setToast("Meeting link deleted");
                        } catch (failure) {
                            setLinks(kept);
                            setListError(failure.message || "The link could not be deleted.");
                        }
                    }

                    function join(event) {
                        event.preventDefault();
                        const id = meetingId.trim();
                        if (!id) return;
                        window.open("/meet/j/" + encodeURIComponent(id), "_blank", "noopener,noreferrer");
                    }
                "##</script>
            </main>
        </AppShell>
    })
}

/// Mint a meeting link and keep it.
#[rpc]
pub async fn create_meeting_link(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;

    meetings::create(title).await
}

/// Revoke a link the organizer no longer wants handed around.
///
/// Answers the same `true` whether the row was already gone or never existed:
/// the card is off the page either way, and a browser that asked twice is not
/// an error worth showing somebody.
#[rpc]
pub async fn delete_meeting_link(id: String) -> Result<bool> {
    let id = id.trim().to_string();

    let mut validator = Validator::new();
    validator.required("id", &id).max_chars("id", &id, 64);
    validator.finish()?;

    meetings::revoke(&id).await?;
    Ok(true)
}
