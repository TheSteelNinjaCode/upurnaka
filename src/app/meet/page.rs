//! Meet: the three ways a meeting starts, and the links this account owns.

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::meetings::{self, MeetingLink};
use crate::rahti::{Json, Metadata, Result, Validator, html, rpc};

/// What the title box is filled in with, the way Teams fills it in with the
/// organizer's name.

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
    let user = crate::auth::current().unwrap_or_default();
    let suggested_title = format!("Meeting with {}", user.name);

    Ok(html! {
        <AppShell active="meet">
            <main
                class="scroll-thin relative min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
                onkeydown={handlePopoverKey(event)}
            >
                <div class="mx-auto w-full max-w-[1180px] px-14 py-9">
                    <h1 class="text-[22px] font-semibold tracking-tight text-ink">"Meet"</h1>
                    <div class="mt-7 grid grid-cols-1 gap-5 md:grid-cols-3">
                        <div class="relative">
                            <button
                                type="button"
                                class="relative z-40 flex h-12 w-full items-center justify-center gap-3 rounded-md bg-brand text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
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
                                        class="h-9 w-full rounded-t bg-hover px-3 text-[13px] text-ink border-b-2 border-brand placeholder:text-subtle focus:outline-none"
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
                        <div class="relative">
                            <button
                                type="button"
                                pp-ref={scheduleTrigger}
                                class={scheduleOpen ? ACTION_OPEN : ACTION}
                                aria-expanded={scheduleOpen}
                                aria-controls="meeting-schedule-options"
                                onclick={toggleSchedule()}
                            >
                                <Icon
                                    name="calendar-day"
                                    class="h-[18px] w-[18px] text-[#d58bc4]"
                                />
                                "Schedule a meeting"
                            </button>
                            <div
                                id="meeting-schedule-options"
                                class="absolute left-0 top-full z-40 mt-1 w-full rounded-md border border-edge bg-raised p-1 shadow-2xl shadow-black/50"
                                hidden={!scheduleOpen}
                            >
                                <a
                                    pp-ref={scheduleFirst}
                                    href="/calendar?new=1"
                                    class="flex min-h-10 items-center gap-3 rounded px-3 text-[14px] text-ink hover:bg-hover focus-visible:outline-2 focus-visible:outline-ink"
                                >
                                    <Icon name="people" class="h-5 w-5 text-brand-ink" />
                                    "Schedule in Upurnaka"
                                </a>
                                <a
                                    href="https://calendar.google.com/calendar/r/eventedit"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    onclick={closeSchedule()}
                                    class="flex min-h-10 items-center gap-3 rounded px-3 text-[14px] text-ink hover:bg-hover focus-visible:outline-2 focus-visible:outline-ink"
                                >
                                    <span
                                        aria-hidden="true"
                                        class="grid h-5 w-5 place-items-center rounded-sm bg-white text-[11px] font-semibold text-[#4285f4]"
                                    >
                                        "31"
                                    </span>
                                    "Schedule in Google Calendar"
                                </a>
                            </div>
                        </div>
                        <div class="relative">
                            <button
                                type="button"
                                pp-ref={joinTrigger}
                                class={joinOpen ? ACTION_OPEN : ACTION}
                                aria-expanded={joinOpen}
                                aria-controls="meeting-join-dialog"
                                aria-haspopup="dialog"
                                onclick={toggleJoin()}
                            >
                                <Icon name="hash" class="h-[18px] w-[18px] text-[#80b5df]" />
                                "Join with a meeting ID"
                            </button>
                            <div
                                id="meeting-join-dialog"
                                role="dialog"
                                aria-labelledby="meeting-join-title"
                                class="absolute right-0 top-full z-40 mt-1 w-[360px] max-w-[calc(100vw-100px)] rounded-md border border-edge bg-raised p-5 shadow-2xl shadow-black/50"
                                hidden={!joinOpen}
                            >
                                <header class="mb-3 flex items-center justify-between gap-3">
                                    <h2
                                        id="meeting-join-title"
                                        class="text-[15px] font-semibold text-ink"
                                    >
                                        "Join a meeting with an ID"
                                    </h2>
                                    <button
                                        type="button"
                                        aria-label="Close meeting ID dialog"
                                        class="grid h-7 w-7 place-items-center rounded text-muted hover:bg-hover hover:text-ink"
                                        onclick={closeJoin()}
                                    >
                                        <Icon name="close" class="h-5 w-5" />
                                    </button>
                                </header>
                                <form onsubmit={join(event)} novalidate>
                                    <label
                                        for="meeting-join-id"
                                        class="mb-2 block text-[13px] text-ink"
                                    >
                                        "Meeting ID"
                                    </label>
                                    <input
                                        id="meeting-join-id"
                                        pp-ref={joinInput}
                                        name="meeting"
                                        type="text"
                                        class="h-10 w-full rounded-t border-b-2 border-transparent bg-surface px-3 text-[14px] text-ink placeholder:text-muted focus:border-brand-ink focus:outline-none"
                                        placeholder="Type a meeting ID"
                                        aria-label="Meeting ID"
                                        autocomplete="off"
                                        value={meetingId}
                                        oninput={setMeetingId(target.value)}
                                    />
                                    <label
                                        for="meeting-join-passcode"
                                        class="mb-2 mt-4 block text-[13px] text-ink"
                                    >
                                        "Meeting passcode"
                                    </label>
                                    <input
                                        id="meeting-join-passcode"
                                        type="text"
                                        disabled
                                        placeholder="Not required for these meeting links"
                                        class="h-10 w-full rounded bg-surface px-3 text-[13px] text-muted disabled:cursor-not-allowed"
                                    />
                                    <button
                                        type="submit"
                                        class="mt-5 h-10 w-full rounded bg-brand px-5 text-[14px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:bg-surface disabled:text-subtle disabled:cursor-not-allowed"
                                        disabled={meetingId.trim().length === 0}
                                    >
                                        "Join meeting"
                                    </button>
                                </form>
                            </div>
                        </div>
                    </div>
                    <h2 class="mt-10 text-[15px] font-semibold text-ink">"Meeting links"</h2>
                    <div class="mt-4 grid grid-cols-2 gap-5">
                        <template pp-for="link in links">
                            <div
                                key={link.id}
                                data-meeting-id={link.id}
                                class="group relative rounded-lg border border-edge bg-raised px-5 py-4"
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
                                <div class="mt-3 flex min-w-0 items-center gap-2">
                                    <p class="min-w-0 truncate text-[15px] font-semibold text-ink">
                                        {link.title}
                                    </p>
                                    <button
                                        type="button"
                                        class={editingId === link.id ? RENAME_EDITING : RENAME}
                                        aria-label="Edit meeting name"
                                        title="Edit meeting name"
                                        onclick={openRename(link, event)}
                                    >
                                        <Icon name="edit" class="h-4 w-4" />
                                    </button>
                                </div>
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
                    hidden={menuFor === "" && !compose && !editingId && !scheduleOpen && !joinOpen}
                    onclick={dismiss()}
                >
                </div>
                <div
                    class="meeting-rename-dialog fixed z-45 w-[min(340px,calc(100vw-32px))] rounded-md border border-line bg-raised p-[22px] shadow-2xl shadow-black/70"
                    role="dialog"
                    aria-labelledby="meeting-rename-title"
                    style={renamePosition}
                    hidden={!editingId}
                    onkeydown={handleRenameKey(event)}
                >
                    <header class="mb-[18px] flex items-center justify-between gap-4">
                        <h2 id="meeting-rename-title" class="text-[17px] font-semibold">
                            "Edit meeting name"
                        </h2>
                        <button
                            type="button"
                            class="grid cursor-pointer place-items-center text-muted hover:text-ink disabled:cursor-not-allowed disabled:opacity-50"
                            aria-label="Close name editor"
                            disabled={renaming}
                            onclick={closeRename()}
                        >
                            <Icon name="close" class="h-5 w-5" />
                        </button>
                    </header>
                    <form onsubmit={saveRename(event)} novalidate>
                        <input
                            pp-ref={renameInput}
                            class="h-10 w-full rounded border-b-2 border-brand-ink bg-surface px-3 py-2 text-sm text-ink outline-none"
                            name="title"
                            aria-label="Meeting name"
                            type="text"
                            maxlength="120"
                            autocomplete="off"
                            value={renameTitle}
                            disabled={renaming}
                            oninput={setRenameTitle(target.value)}
                        />
                        <p
                            class="mt-2.5 text-xs text-danger-ink"
                            role="alert"
                            hidden={!renameError}
                        >
                            {renameError}
                        </p>
                        <footer class="mt-4 flex justify-end gap-3">
                            <button
                                type="button"
                                class="meeting-rename-cancel h-[38px] min-w-28 cursor-pointer rounded border border-edge bg-transparent text-sm font-semibold transition-colors hover:bg-hover disabled:cursor-not-allowed disabled:opacity-50"
                                disabled={renaming}
                                onclick={closeRename()}
                            >
                                "Cancel"
                            </button>
                            <button
                                type="submit"
                                class="meeting-rename-save h-[38px] min-w-28 cursor-pointer rounded bg-brand text-sm font-semibold text-white transition-colors hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50"
                                disabled={renaming || !renameTitle.trim()}
                            >
                                {renaming ? "Saving…" : "Save"}
                            </button>
                        </footer>
                    </form>
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
                    const ACTION = "relative z-40 flex h-12 w-full items-center justify-center gap-3 rounded-md border border-edge bg-raised text-[13px] font-semibold text-ink transition-colors hover:bg-hover";
                    const ACTION_OPEN = twMerge(ACTION + " bg-hover");
                    const EMPTY = "rounded-lg border border-edge bg-raised px-8 py-7";
                    const EMPTY_WIDE = EMPTY + " col-span-2";
                    // The rename pencil shows on card hover or focus; touch screens
                    // have no hover, so it stays visible there.
                    const RENAME = "grid h-[26px] w-[26px] shrink-0 cursor-pointer place-items-center rounded text-muted opacity-0 transition-[opacity,color] duration-100 hover:bg-hover hover:text-brand-ink focus-visible:bg-hover focus-visible:text-brand-ink group-hover:opacity-100 group-focus-within:opacity-100 pointer-coarse:opacity-100";
                    const RENAME_EDITING = twMerge(RENAME + " opacity-100 text-brand-ink");

                    const [links, setLinks] = pp.state(@{Json(&links)});
                    const [compose, setCompose] = pp.state(false);
                    const [title, setTitle] = pp.state(@{Json(&suggested_title)});
                    const [creating, setCreating] = pp.state(false);
                    const [error, setError] = pp.state("");
                    const [listError, setListError] = pp.state("");
                    const [menuFor, setMenuFor] = pp.state("");
                    const [joinOpen, setJoinOpen] = pp.state(false);
                    const [scheduleOpen, setScheduleOpen] = pp.state(false);
                    const joinTrigger = pp.ref(null);
                    const joinInput = pp.ref(null);
                    const scheduleTrigger = pp.ref(null);
                    const scheduleFirst = pp.ref(null);
                    const shellControls = pp.context(window.UpurnakaShortcutContext);
                    const dismissPopovers = pp.event(() => {
                        setCompose(false);
                        setJoinOpen(false);
                        setScheduleOpen(false);
                        setMenuFor("");
                    });
                    pp.imperativeHandle(shellControls, () => ({dismissPopovers}), [shellControls, dismissPopovers]);
                    pp.effect(() => {
                        if (joinOpen) joinInput.current?.focus();
                    }, [joinOpen]);
                    pp.effect(() => {
                        if (scheduleOpen) scheduleFirst.current?.focus();
                    }, [scheduleOpen]);
                    const [meetingId, setMeetingId] = pp.state("");
                    const [toast, setToast] = pp.state("");
                    const [editingId, setEditingId] = pp.state("");
                    const [renameTitle, setRenameTitle] = pp.state("");
                    const [renameError, setRenameError] = pp.state("");
                    const [renaming, setRenaming] = pp.state(false);
                    const [renamePosition, setRenamePosition] = pp.state("");
                    const renameInput = pp.ref(null);
                    const renameTrigger = pp.ref(null);

                    pp.effect(() => {
                        if (editingId && renameInput.current) {
                            renameInput.current.focus();
                            renameInput.current.select();
                        }
                    }, [editingId]);

                    function openRename(link, event) {
                        setJoinOpen(false);
                        setScheduleOpen(false);
                        const anchor = event.currentTarget.getBoundingClientRect();
                        const width = Math.min(340, innerWidth - 32);
                        const left = Math.max(16, Math.min(anchor.left - 24, innerWidth - width - 16));
                        const top = Math.max(16, Math.min(anchor.bottom + 8, innerHeight - 260));
                        renameTrigger.current = event.currentTarget;
                        setCompose(false);
                        setMenuFor("");
                        setRenameTitle(link.title);
                        setRenameError("");
                        setRenamePosition("left:" + left + "px;top:" + top + "px");
                        setEditingId(link.id);
                    }

                    function closeRename() {
                        if (renaming) return;
                        setEditingId("");
                        setRenameError("");
                        if (renameTrigger.current?.isConnected) renameTrigger.current.focus();
                    }

                    function handleRenameKey(event) {
                        if (event.key === "Escape") {
                            event.preventDefault();
                            closeRename();
                        }
                    }

                    async function saveRename(event) {
                        event.preventDefault();
                        if (renaming || !editingId) return;
                        const payload = Object.fromEntries(new FormData(event.currentTarget));
                        payload.id = editingId;
                        setRenaming(true);
                        setRenameError("");
                        try {
                            const updated = await pp.rpc("rename_meeting_link", payload);
                            setLinks(current => current.map(link => link.id === updated.id ? updated : link));
                            setEditingId("");
                            if (renameTrigger.current?.isConnected) renameTrigger.current.focus();
                            setToast("Meeting name updated");
                        } catch (failure) {
                            setRenameError(failure.errors?.title?.[0] || failure.message || "The meeting name could not be saved.");
                        } finally {
                            setRenaming(false);
                        }
                    }

                    // One timer, restarted whenever a new notice replaces the
                    // one on screen, and cleared when the page goes away.
                    pp.effect(() => {
                        if (!toast) return;
                        const timer = setTimeout(() => setToast(""), 4000);
                        return () => clearTimeout(timer);
                    }, [toast]);

                    function toggleCompose() {
                        setJoinOpen(false);
                        setScheduleOpen(false);
                        setMenuFor("");
                        setError("");
                        setCompose((open) => !open);
                    }

                    function closeCompose() {
                        setCompose(false);
                        setError("");
                    }

                    function toggleJoin() {
                        setScheduleOpen(false);
                        setCompose(false);
                        setMenuFor("");
                        setJoinOpen((open) => !open);
                    }

                    function toggleSchedule() {
                        setCompose(false);
                        setJoinOpen(false);
                        setMenuFor("");
                        setScheduleOpen(open => !open);
                    }

                    function closeSchedule() {
                        setScheduleOpen(false);
                        scheduleTrigger.current?.focus();
                    }

                    function closeJoin() {
                        setJoinOpen(false);
                        joinTrigger.current?.focus();
                    }

                    function handlePopoverKey(event) {
                        if (event.key !== "Escape") return;
                        if (joinOpen) { event.preventDefault(); closeJoin(); }
                        if (scheduleOpen) { event.preventDefault(); closeSchedule(); }
                    }

                    function openMenu(id) {
                        setJoinOpen(false);
                        setScheduleOpen(false);
                        setCompose(false);
                        setMenuFor((current) => (current === id ? "" : id));
                    }

                    // The backdrop sits under both the popover and a card
                    // menu, so one click closes whichever is open.
                    function dismiss() {
                        if (joinOpen) closeJoin();
                        if (scheduleOpen) closeSchedule();
                        closeRename();
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
#[rpc(auth)]
pub async fn create_meeting_link(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;

    meetings::create(title).await
}

/// Rename a live link; the join code and existing conference stay the same.
#[rpc(auth)]
pub async fn rename_meeting_link(id: String, title: String) -> Result<MeetingLink> {
    let id = id.trim().to_string();
    let title = title.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("id", &id)
        .max_chars("id", &id, 64)
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;
    meetings::rename(&id, title).await
}

/// Revoke a link the organizer no longer wants handed around.
///
/// Answers the same `true` whether the row was already gone or never existed:
/// the card is off the page either way, and a browser that asked twice is not
/// an error worth showing somebody.
#[rpc(auth)]
pub async fn delete_meeting_link(id: String) -> Result<bool> {
    let id = id.trim().to_string();

    let mut validator = Validator::new();
    validator.required("id", &id).max_chars("id", &id, 64);
    validator.finish()?;

    meetings::revoke(&id).await?;
    Ok(true)
}
