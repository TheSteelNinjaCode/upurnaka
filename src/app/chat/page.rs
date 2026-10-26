//! Chat: the conversation list on the left, one thread on the right.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::meetings::{self, MeetingLink};
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
                                aria-expanded={searchOpen}
                                aria-pressed={searchOpen}
                                pp-ref={chatSearchTrigger}
                                onclick={toggleChatSearch()}
                            >
                                <Icon name="search" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Meet now"
                                aria-label="Meet now"
                                aria-expanded={meetingOpen}
                                pp-ref={meetNowTrigger}
                                onclick={openMeetNow(event)}
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
                    <div class="chat-list-search" hidden={!searchOpen}>
                        <input
                            pp-ref={chatSearchInput}
                            type="search"
                            aria-label="Filter by person or chat name"
                            placeholder="Filter by person or chat name"
                            value={chatFilter}
                            oninput={setChatFilter(target.value)}
                            onkeydown={handleSearchKey(event)}
                        />
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
                        <div class="chat-self-entry mt-0.5" hidden={!favoritesOpen || !selfMatches}>
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
                            hidden={!chatsOpen || hasChatFilter}
                        >
                            "Nobody has written to you yet. Start a chat and it will show up here."
                        </p>
                        <p
                            class="px-3 py-2 text-xs leading-relaxed text-subtle"
                            role="status"
                            hidden={!hasChatFilter || selfMatches}
                        >
                            "No chats match your filter."
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
                                onclick={openConversationSearch()}
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
                    <div
                        class="chat-conversation-search px-8 py-2"
                        hidden={!conversationSearchOpen}
                    >
                        <input
                            type="search"
                            class="w-full rounded border border-edge bg-raised px-3 py-2 text-sm"
                            aria-label="Find messages in this conversation"
                            placeholder="Find in this conversation"
                            pp-ref={conversationSearchInput}
                            value={conversationQuery}
                            oninput={setConversationQuery(target.value)}
                            onkeydown={handleConversationSearchKey(event)}
                        />
                        <p class="mt-1 text-xs text-muted" role="status">
                            {matchingMessages.length}
                            " matching messages"
                        </p>
                    </div>
                    <div
                        class="scroll-thin flex min-h-0 flex-1 flex-col overflow-y-auto px-8"
                        tabindex="-1"
                        aria-label="Messages"
                        pp-ref={messagePane}
                    >
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
                            <template pp-for="message in matchingMessages">
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
                            pp-ref={composerForm}
                            novalidate
                        >
                            <textarea
                                name="text"
                                rows={expandedCompose ? 5 : 1}
                                class={expandedCompose ? "min-h-32 min-w-0 flex-1 resize-none bg-transparent py-2 text-[13px] text-ink placeholder:text-muted focus:outline-none" : "h-9 min-w-0 flex-1 resize-none bg-transparent py-2 text-[13px] text-ink placeholder:text-muted focus:outline-none"}
                                placeholder="Type a message"
                                autocomplete="off"
                                aria-label="Type a message"
                                pp-ref={composerInput}
                                onkeydown={handleComposerKey(event)}
                                value={draft}
                                oninput={setDraft(target.value)}
                            >
                            </textarea>
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
                                aria-expanded={expandedCompose}
                                onclick={toggleCompose()}
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
                <div class="chat-meeting-backdrop" hidden={!meetingOpen} onclick={closeMeetNow()}>
                </div>
                <section
                    class="chat-meet-now"
                    role="dialog"
                    aria-labelledby="chat-meet-now-title"
                    style={meetingPosition}
                    hidden={!meetingOpen}
                    onkeydown={handleMeetingKey(event)}
                >
                    <header>
                        <h2 id="chat-meet-now-title">"Start a meeting now"</h2>
                        <button
                            type="button"
                            aria-label="Close meeting popup"
                            disabled={meetingBusy}
                            onclick={closeMeetNow()}
                        >
                            <Icon name="close" class="h-5 w-5" />
                        </button>
                    </header>
                    <form onsubmit={startChatMeeting(event)} novalidate>
                        <label for="chat-meeting-name">"Meeting name"</label>
                        <input
                            id="chat-meeting-name"
                            pp-ref={meetingNameInput}
                            name="title"
                            aria-label="Meeting name"
                            type="text"
                            maxlength="120"
                            autocomplete="off"
                            value={meetingTitle}
                            oninput={changeMeetingTitle(target.value)}
                            disabled={meetingBusy}
                        />
                        <p class="chat-meeting-error" role="alert" hidden={!meetingError}>
                            {meetingError}
                        </p>
                        <div class="chat-meeting-share" hidden={!sharedMeeting}>
                            <input
                                aria-label="Meeting link"
                                readonly
                                value={sharedMeeting ? meetingAbsolute(sharedMeeting.url) : ""}
                            />
                            <p role="status">{meetingNotice}</p>
                            <a
                                href={sharedMeeting ? sharedMeeting.url : "/meet"}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                "Open meeting in a new tab"
                            </a>
                        </div>
                        <button
                            type="button"
                            class="chat-meeting-share-button"
                            disabled={meetingBusy || !meetingTitle.trim()}
                            onclick={getChatMeetingLink()}
                        >
                            {meetingBusy ? "Creating…" : "Get a link to share"}
                        </button>
                        <button
                            type="submit"
                            class="chat-meeting-start"
                            disabled={meetingBusy || !meetingTitle.trim()}
                        >
                            {meetingBusy ? "Please wait…" : "Start meeting"}
                        </button>
                    </form>
                </section>
                <style>r##"
                    .chat-list-search {padding:12px 20px 0;}
                    .chat-list-search input {width:100%;height:36px;padding:8px 12px;background:#292929;border:0;border-bottom:2px solid var(--color-brand-ink);border-radius:4px;color:var(--color-ink);font-size:13px;outline:none;}
                    .chat-list-search input::placeholder {color:#999;}
                    .list-pane button[aria-label="Search chats"][aria-pressed="true"],.list-pane button[aria-label="Meet now"][aria-expanded="true"] {color:var(--color-brand-ink);}
                    .chat-meeting-backdrop {position:fixed;inset:0;z-index:45;}
                    .chat-meet-now {position:fixed;z-index:46;width:min(380px,calc(100vw - 32px));background:#292929;border:1px solid #333;border-radius:5px;padding:22px;box-shadow:0 10px 28px #0007;}
                    .chat-meet-now>header {display:flex;align-items:center;justify-content:space-between;gap:18px;margin-bottom:22px;}
                    .chat-meet-now h2 {font-size:17px;font-weight:600;}
                    .chat-meet-now>header button {color:#ccc;cursor:pointer;}
                    .chat-meet-now label {display:block;font-size:14px;margin-bottom:6px;}
                    .chat-meet-now input {width:100%;height:40px;padding:8px 12px;background:#222;border:1px solid #666;border-bottom:2px solid var(--color-brand-ink);border-radius:4px;color:white;outline:none;font-size:14px;}
                    .chat-meet-now form>button {width:100%;height:40px;display:block;border-radius:4px;font-size:14px;font-weight:600;cursor:pointer;}
                    .chat-meeting-share-button {border:1px solid #666;margin-top:22px;background:transparent;}
                    .chat-meeting-share-button:hover {background:#333;}
                    .chat-meeting-start {margin-top:10px;background:var(--color-brand);color:white;}
                    .chat-meeting-start:hover {background:var(--color-brand-hover);}
                    .chat-meet-now button:disabled {opacity:.5;cursor:not-allowed;}
                    .chat-meeting-error {font-size:12px;color:#ffb0ba;margin-top:10px;}
                    .chat-meeting-share {margin-top:14px;}
                    .chat-meeting-share input {font-size:12px;border:1px solid #555;}
                    .chat-meeting-share p {font-size:12px;color:#bbb;margin-top:8px;}
                    .chat-meeting-share a {display:inline-block;font-size:12px;color:var(--color-link);margin-top:6px;}
                    .chat-meet-now[hidden],.chat-list-search[hidden],.chat-meeting-backdrop[hidden],.chat-meeting-share[hidden] {display:none;}
                "##</style>
                <script>r##"
                    const PILL_ON = "h-7 shrink-0 rounded-full border border-brand bg-brand-soft px-3 text-xs font-semibold text-ink";
                    const PILL_OFF = "h-7 shrink-0 rounded-full border border-edge px-3 text-xs text-muted transition-colors hover:bg-hover hover:text-ink";
                    const ROW_ON = "flex w-full items-center gap-3 rounded-md bg-raised px-3 py-2 text-left";
                    const ROW_OFF = "flex w-full items-center gap-3 rounded-md px-3 py-2 text-left transition-colors hover:bg-hover";

                    const shortcuts = pp.context(window.UpurnakaShortcutContext);
                    const composerInput = pp.ref(null);
                    const composerForm = pp.ref(null);
                    const messagePane = pp.ref(null);
                    const conversationSearchInput = pp.ref(null);
                    const [expandedCompose, setExpandedCompose] = pp.state(false);
                    const [conversationSearchOpen, setConversationSearchOpen] = pp.state(false);
                    const [conversationQuery, setConversationQuery] = pp.state("");
                    const [filters, setFilters] = pp.state({ unread: false, meetings: false, unmuted: false });
                    const [favoritesOpen, setFavoritesOpen] = pp.state(true);
                    const [chatsOpen, setChatsOpen] = pp.state(true);
                    const [selected, setSelected] = pp.state("self");
                    const [messages, setMessages] = pp.state([]);
                    const [draft, setDraft] = pp.state("");
                    const [sending, setSending] = pp.state(false);
                    const [error, setError] = pp.state("");
                    const [searchOpen, setSearchOpen] = pp.state(false);
                    const [chatFilter, setChatFilter] = pp.state("");
                    const chatSearchInput = pp.ref(null);
                    const chatSearchTrigger = pp.ref(null);
                    const [meetingOpen, setMeetingOpen] = pp.state(false);
                    const [meetingTitle, setMeetingTitle] = pp.state("Meeting with Jefferson Abraham");
                    const [meetingBusy, setMeetingBusy] = pp.state(false);
                    const [meetingError, setMeetingError] = pp.state("");
                    const [meetingNotice, setMeetingNotice] = pp.state("");
                    const [sharedMeeting, setSharedMeeting] = pp.state(null);
                    const [meetingPosition, setMeetingPosition] = pp.state("");
                    const meetingNameInput = pp.ref(null);
                    const meetNowTrigger = pp.ref(null);
                    const hasChatFilter = !!chatFilter.trim() || filters.unread || filters.meetings || filters.unmuted;
                    const selfMatches = "Jefferson Abraham (You)".toLocaleLowerCase().includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings;

                    const matchingMessages = messages.filter(message => message.text.toLocaleLowerCase().includes(conversationQuery.trim().toLocaleLowerCase()));
                    const handleChatShortcut = pp.event(event => {
                        if (meetingOpen || event.isComposing) return;
                        const key = event.key.toLowerCase();
                        const control = (event.ctrlKey || event.metaKey) && !event.altKey;
                        let action = null;
                        if (control && !event.shiftKey && key === "r") action = () => composerInput.current?.focus();
                        else if (control && !event.shiftKey && key === "m") action = () => messagePane.current?.focus();
                        else if (control && !event.shiftKey && key === "f") action = openConversationSearch;
                        else if (control && !event.shiftKey && key === "j") action = () => {
                            messagePane.current?.focus();
                            if (messagePane.current) messagePane.current.scrollTop = messagePane.current.scrollHeight;
                        };
                        else if (control && event.shiftKey && key === "f") action = openChatSearch;
                        else if (control && event.shiftKey && key === "x") action = toggleCompose;
                        else if (event.ctrlKey && event.altKey && !event.metaKey && !event.shiftKey) {
                            if (key === "z") action = () => {setFilters({unread:false, meetings:false, unmuted:false}); setChatFilter("");};
                            else if (["u", "q", "b"].includes(key)) action = () => {
                                setFilters({unread:key === "u", meetings:key === "b", unmuted:key === "q"});
                            };
                            else if (key === "c") action = () => {setFilters({unread:false, meetings:false, unmuted:false}); setChatFilter(""); setFavoritesOpen(true); setChatsOpen(true);};
                        } else if (event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey && key === "q") {
                            action = () => {setFavoritesOpen(false); setChatsOpen(false);};
                        }
                        if (action) {event.preventDefault(); if (!event.repeat) action();}
                    });
                    pp.imperativeHandle(shortcuts, () => ({handleKey:handleChatShortcut}), [shortcuts, handleChatShortcut]);
                    pp.effect(() => {
                        if (conversationSearchOpen) conversationSearchInput.current?.focus();
                    }, [conversationSearchOpen]);
                    function openConversationSearch() {
                        setConversationSearchOpen(true);
                        conversationSearchInput.current?.focus();
                    }
                    function openChatSearch() {setSearchOpen(true); chatSearchInput.current?.focus();}
                    function handleConversationSearchKey(event) {
                        if (event.key === "Escape") {
                            event.preventDefault(); setConversationSearchOpen(false); setConversationQuery(""); composerInput.current?.focus();
                        }
                    }
                    function toggleCompose() {
                        setExpandedCompose(open => !open);
                        composerInput.current?.focus();
                    }
                    function handleComposerKey(event) {
                        if (event.isComposing || event.repeat) return;
                        if (event.key === "Enter" && !event.shiftKey && !event.altKey && ((!expandedCompose && !event.ctrlKey && !event.metaKey) || event.ctrlKey || event.metaKey)) {
                            event.preventDefault(); composerForm.current?.requestSubmit();
                        }
                    }
                    pp.effect(() => {
                        if (searchOpen && chatSearchInput.current) chatSearchInput.current.focus();
                    }, [searchOpen]);
                    pp.effect(() => {
                        if (meetingOpen && !meetingBusy && meetingNameInput.current) {
                            meetingNameInput.current.focus();
                            meetingNameInput.current.select();
                        }
                    }, [meetingOpen, meetingBusy]);

                    function toggleChatSearch() {
                        setSearchOpen(!searchOpen);
                        if (searchOpen) setChatFilter("");
                    }
                    function handleSearchKey(event) {
                        if (event.key === "Escape") {
                            setChatFilter(""); setSearchOpen(false);
                            if (chatSearchTrigger.current) chatSearchTrigger.current.focus();
                        }
                    }
                    function openMeetNow(event) {
                        const anchor = event.currentTarget.getBoundingClientRect();
                        const width = Math.min(380, innerWidth - 32);
                        const left = Math.max(16, Math.min(anchor.left - width / 2, innerWidth - width - 16));
                        const top = Math.max(16, Math.min(anchor.bottom + 10, innerHeight - 430));
                        setMeetingPosition("left:" + left + "px;top:" + top + "px");
                        setMeetingError("");
                        setMeetingOpen(true);
                    }
                    function closeMeetNow() {
                        if (meetingBusy) return;
                        setMeetingOpen(false);
                        if (meetNowTrigger.current) meetNowTrigger.current.focus();
                    }
                    function handleMeetingKey(event) {
                        if (event.key === "Escape") { event.preventDefault(); closeMeetNow(); }
                    }
                    function changeMeetingTitle(value) {
                        setMeetingTitle(value);
                        setSharedMeeting(null);
                        setMeetingNotice("");
                        setMeetingError("");
                    }
                    function meetingAbsolute(url) { return location.origin + url; }
                    async function ensureChatMeeting(requestedTitle) {
                        if (sharedMeeting && sharedMeeting.title === requestedTitle.trim()) return sharedMeeting;
                        const created = await pp.rpc("create_chat_meeting", { title: requestedTitle });
                        setSharedMeeting(created);
                        return created;
                    }
                    async function getChatMeetingLink() {
                        if (meetingBusy) return;
                        setMeetingBusy(true);
                        setMeetingError("");
                        try {
                            const created = await ensureChatMeeting(meetingNameInput.current.value);
                            try {
                                await navigator.clipboard.writeText(meetingAbsolute(created.url));
                                setMeetingNotice("Meeting link copied to clipboard.");
                            } catch (failure) {
                                setMeetingNotice("Your meeting link is ready. Select the link above to copy it.");
                            }
                        } catch (failure) { setMeetingError(failure.errors?.title?.[0] || failure.message || "The meeting could not be created."); }
                        finally { setMeetingBusy(false); }
                    }
                    async function startChatMeeting(event) {
                        event.preventDefault();
                        const payload = Object.fromEntries(new FormData(event.currentTarget));
                        const requestedTitle = String(payload.title || "").trim();
                        if (meetingBusy || !requestedTitle) return;
                        setMeetingError("");
                        if (sharedMeeting && sharedMeeting.title === requestedTitle) {
                            window.open(sharedMeeting.url, "_blank", "noopener,noreferrer");
                            return;
                        }
                        // Open during the click gesture, then navigate only after
                        // the server has created the real meeting link.
                        const tab = window.open("about:blank", "_blank");
                        if (tab) tab.opener = null;
                        setMeetingBusy(true);
                        try {
                            const created = await ensureChatMeeting(requestedTitle);
                            if (tab && !tab.closed) tab.location.replace(meetingAbsolute(created.url));
                            else setMeetingNotice("Your meeting is ready. Use Open meeting in a new tab above.");
                        } catch (failure) {
                            try {
                                if (tab && !tab.closed && tab.location.href === "about:blank") tab.close();
                            } catch (closedTab) {
                                // A tab the user navigated elsewhere is theirs to keep.
                            }
                            setMeetingError(failure.errors?.title?.[0] || failure.message || "The meeting could not be started.");
                        } finally { setMeetingBusy(false); }
                    }

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

/// Chat's Meet now action creates the same persistent link listed by Meet.
#[rpc]
pub async fn create_chat_meeting(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;
    meetings::create(title).await
}
