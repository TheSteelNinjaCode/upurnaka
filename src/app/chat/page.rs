//! Chat: the conversation list on the left, one thread on the right.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::rahti::time;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Chat")
        .description("Your Teams conversations.")
}

pub async fn page() -> Html {
    html! {
        <AppShell active="chat">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[350px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">"Chat"</h1>
                        <div class="flex items-center gap-0.5">
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Search chats"
                                aria-label="Search chats"
                            >
                                <Icon name="search" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Meet now"
                                aria-label="Meet now"
                            >
                                <Icon name="video" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="New chat"
                                aria-label="New chat"
                            >
                                <Icon name="compose" class="h-[18px] w-[18px]" />
                            </button>
                        </div>
                    </div>
                    <div class="flex items-center gap-2 px-5 py-3">
                        <button
                            type="button"
                            class={filters.unread ? PILL_ON : PILL_OFF}
                            onclick={toggleFilter("unread")}
                        >
                            "Unread"
                        </button>
                        <button
                            type="button"
                            class={filters.meetings ? PILL_ON : PILL_OFF}
                            onclick={toggleFilter("meetings")}
                        >
                            "Meeting chats"
                        </button>
                        <button
                            type="button"
                            class={filters.unmuted ? PILL_ON : PILL_OFF}
                            onclick={toggleFilter("unmuted")}
                        >
                            "Unmuted"
                        </button>
                    </div>
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-2 pb-4">
                        <button
                            type="button"
                            class="flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-xs font-semibold text-muted transition-colors hover:bg-hover hover:text-ink"
                            aria-expanded={favoritesOpen}
                            onclick={setFavoritesOpen(open => !open)}
                        >
                            <span class={favoritesOpen ? "flex" : "flex -rotate-90"}>
                                <Icon name="chevron-down" class="h-3.5 w-3.5" />
                            </span>
                            "Favorites"
                        </button>
                        <div class="mt-0.5" hidden={!favoritesOpen}>
                            <button
                                type="button"
                                class={selected === "self" ? ROW_ON : ROW_OFF}
                                onclick={setSelected("self")}
                            >
                                <Avatar initials="JA" size="md" presence="online" />
                                <span class="truncate text-[13px] font-semibold text-ink">
                                    "Jefferson Abraham (You)"
                                </span>
                            </button>
                        </div>
                        <button
                            type="button"
                            class="mt-3 flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-xs font-semibold text-muted transition-colors hover:bg-hover hover:text-ink"
                            aria-expanded={chatsOpen}
                            onclick={setChatsOpen(open => !open)}
                        >
                            <span class={chatsOpen ? "flex" : "flex -rotate-90"}>
                                <Icon name="chevron-down" class="h-3.5 w-3.5" />
                            </span>
                            "Chats"
                        </button>
                        <p
                            class="px-3 py-2 text-xs leading-relaxed text-subtle"
                            hidden={!chatsOpen}
                        >
                            "Nobody has written to you yet. Start a chat and it will show up here."
                        </p>
                    </div>
                </aside>
                <main
                    class="flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <header class="flex h-14 shrink-0 items-center gap-3 px-5">
                        <Avatar initials="JA" size="md" presence="online" />
                        <h2 class="truncate text-[15px] font-semibold text-ink">
                            "Jefferson Abraham (You)"
                        </h2>
                        <div class="ml-auto flex items-center gap-0.5">
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Find in conversation"
                                aria-label="Find in conversation"
                            >
                                <Icon name="search" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="More options"
                                aria-label="More options"
                            >
                                <Icon name="more" class="h-[18px] w-[18px]" />
                            </button>
                        </div>
                    </header>
                    <div class="scroll-thin flex min-h-0 flex-1 flex-col overflow-y-auto px-8">
                        <div
                            class="m-auto flex max-w-sm flex-col items-center py-10 text-center"
                            hidden={messages.length > 0}
                        >
                            <Avatar initials="JA" size="xl" presence="none" />
                            <h3 class="mt-7 text-[17px] font-semibold text-ink">
                                "This is your space"
                            </h3>
                            <p class="mt-2.5 text-[13px] leading-relaxed text-muted">
                                "This chat is just for you…with you. Use it for drafts, send files to yourself, or get to know chat features a little better."
                            </p>
                        </div>
                        <div class="w-full py-5" hidden={messages.length === 0}>
                            <template pp-for="message in messages">
                                <div key={message.id} class="flex gap-3 py-2">
                                    <span
                                        class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-avatar text-xs font-semibold text-avatar-ink"
                                    >
                                        {message.initials}
                                    </span>
                                    <div class="min-w-0">
                                        <p class="flex items-baseline gap-2">
                                            <span class="text-[13px] font-semibold text-ink">
                                                {message.author}
                                            </span>
                                            <span class="text-[11px] text-subtle">
                                                {message.time}
                                            </span>
                                        </p>
                                        <p
                                            class="mt-0.5 whitespace-pre-wrap break-words text-[13px] leading-relaxed text-muted"
                                        >
                                            {message.text}
                                        </p>
                                    </div>
                                </div>
                            </template>
                        </div>
                    </div>
                    <div class="shrink-0 px-8 pb-6 pt-2">
                        <form
                            class="flex items-center gap-2 rounded-lg border border-edge bg-raised px-3 py-1.5 focus-within:border-brand"
                            onsubmit={send(event)}
                            novalidate
                        >
                            <input
                                name="text"
                                type="text"
                                class="h-9 min-w-0 flex-1 bg-transparent text-[13px] text-ink placeholder:text-muted focus:outline-none"
                                placeholder="Type a message"
                                autocomplete="off"
                                aria-label="Type a message"
                                value={draft}
                                oninput={setDraft(target.value)}
                            />
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Emoji"
                                aria-label="Emoji"
                            >
                                <Icon name="emoji" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Attach a file"
                                aria-label="Attach a file"
                            >
                                <Icon name="attach" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Send an image"
                                aria-label="Send an image"
                            >
                                <Icon name="image" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Format"
                                aria-label="Format"
                            >
                                <Icon name="format" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="More actions"
                                aria-label="More actions"
                            >
                                <Icon name="plus" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="submit"
                                class="flex h-8 w-8 items-center justify-center rounded text-brand-ink transition-colors hover:bg-hover disabled:text-subtle"
                                title="Send"
                                aria-label="Send"
                                disabled={draft.trim().length === 0}
                            >
                                <Icon name="send" class="h-[18px] w-[18px]" />
                            </button>
                        </form>
                        <p class="mt-2 text-xs text-busy" hidden={!error}>{error}</p>
                    </div>
                </main>
                <script>r##"
                    const PILL_ON = "h-7 shrink-0 rounded-full border border-brand bg-brand-soft px-3 text-xs font-semibold text-ink";
                    const PILL_OFF = "h-7 shrink-0 rounded-full border border-edge px-3 text-xs text-muted transition-colors hover:bg-hover hover:text-ink";
                    const ROW_ON = "flex w-full items-center gap-3 rounded-md bg-raised px-3 py-2 text-left";
                    const ROW_OFF = "flex w-full items-center gap-3 rounded-md px-3 py-2 text-left transition-colors hover:bg-hover";

                    const [filters, setFilters] = pp.state({ unread: false, meetings: false, unmuted: false });
                    const [favoritesOpen, setFavoritesOpen] = pp.state(true);
                    const [chatsOpen, setChatsOpen] = pp.state(true);
                    const [selected, setSelected] = pp.state("self");
                    const [messages, setMessages] = pp.state([]);
                    const [draft, setDraft] = pp.state("");
                    const [sending, setSending] = pp.state(false);
                    const [error, setError] = pp.state("");

                    function toggleFilter(name) {
                        setFilters((current) => ({ ...current, [name]: !current[name] }));
                    }

                    async function send(event) {
                        event.preventDefault();
                        const text = draft.trim();
                        if (!text || sending) return;

                        setSending(true);
                        setError("");
                        try {
                            const message = await pp.rpc("send_message", { text });
                            setMessages((current) => current.concat([message]));
                            setDraft("");
                        } catch (failure) {
                            setError(failure.message || "The message could not be sent.");
                        } finally {
                            setSending(false);
                        }
                    }
                "##</script>
            </div>
        </AppShell>
    }
}

/// One line in the thread, as the browser renders it.
#[derive(Serialize)]
pub struct Message {
    pub id: String,
    pub author: String,
    pub initials: String,
    pub time: String,
    pub text: String,
}

/// Accept a line the composer sent.
///
/// The thread is browser state for now, so the server's job is the part the
/// browser should not decide: the text is checked here, and the stamped time
/// comes from the application clock rather than from whatever the device
/// believes the hour is.
#[rpc]
pub async fn send_message(text: String) -> Result<Message> {
    let text = text.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("text", &text)
        .max_chars("text", &text, 2000);
    validator.finish()?;

    let now = time::now();

    Ok(Message {
        id: format!("m{}", now.timestamp_micros()),
        author: "Jefferson Abraham".to_string(),
        initials: "JA".to_string(),
        time: now.format("%-I:%M %p").to_string(),
        text,
    })
}
