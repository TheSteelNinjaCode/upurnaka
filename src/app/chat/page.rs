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
    let user = crate::auth::current().unwrap_or_default();
    // GIPHY requires direct browser requests; this is a public web integration key.
    let giphy_key = std::env::var("GIPHY_API_KEY").unwrap_or_default();
    #[cfg(test)]
    let giphy_key = crate::app_tests::giphy::key_for(&user.id).unwrap_or(giphy_key);
    let initials = user.initials();
    let self_label = format!("{} (You)", user.name);
    html! {
        <AppShell active="chat">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[350px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">"Chat"</h1>
                        <div class="flex items-center gap-0.5">
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink aria-pressed:text-brand-ink"
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
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink aria-expanded:text-brand-ink"
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
                                aria-keyshortcuts="Control+n"
                                onclick={openNewChat()}
                                disabled={sending}
                            >
                                <Icon name="compose" class="h-[18px] w-[18px]" />
                            </button>
                        </div>
                    </div>
                    <div class="chat-list-search px-5 pt-3" hidden={!searchOpen}>
                        <input
                            pp-ref={chatSearchInput}
                            class="h-9 w-full rounded border-b-2 border-brand-ink bg-raised px-3 py-2 text-[13px] text-ink outline-none placeholder:text-subtle"
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
                                onclick={selectThread("self")}
                                disabled={sending}
                            >
                                <Avatar initials=@{initials.as_str()} size="md" presence="online" />
                                <span class="truncate text-[13px] font-semibold text-ink">
                                    @{self_label.as_str()}
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
                        <div
                            class={selected === "new" ? ROW_ON : ROW_OFF}
                            hidden={!chatsOpen || !newRowMatches}
                        >
                            <button
                                type="button"
                                class="flex min-w-0 flex-1 items-center gap-3 text-left"
                                onclick={openNewChat()}
                                disabled={sending}
                                aria-label="New message"
                            >
                                <span
                                    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-hover text-muted"
                                >
                                    <Icon name="chat" class="h-4 w-4" />
                                </span>
                                <span class="truncate text-[13px] font-semibold">
                                    "New message"
                                </span>
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 shrink-0 items-center justify-center rounded text-muted hover:bg-hover hover:text-ink"
                                aria-label="New message options"
                                aria-haspopup="menu"
                                aria-expanded={draftMenuOpen}
                                aria-controls={draftMenuId}
                                pp-ref={draftMenuTrigger}
                                onclick={openDraftMenu()}
                                disabled={sending}
                            >
                                <Icon name="more" class="h-5 w-5" />
                            </button>
                        </div>
                        <template pp-for="thread in visibleThreads">
                            <button
                                key={thread.id}
                                type="button"
                                class={selected === thread.id ? ROW_ON : ROW_OFF}
                                hidden={!chatsOpen}
                                onclick={selectThread(thread.id)}
                                disabled={sending}
                            >
                                <span
                                    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-avatar text-xs font-semibold text-avatar-ink"
                                >
                                    {initials(thread.label)}
                                </span>
                                <span class="min-w-0 truncate text-[13px] font-semibold">
                                    {thread.label}
                                </span>
                            </button>
                        </template>
                        <p
                            class="px-3 py-2 text-xs leading-relaxed text-subtle"
                            hidden={!chatsOpen || hasChatFilter || hasNewChat || threads.length > 1}
                        >
                            "Nobody has written to you yet. Start a chat and it will show up here."
                        </p>
                        <p
                            class="px-3 py-2 text-xs leading-relaxed text-subtle"
                            role="status"
                            hidden={!hasChatFilter || selfMatches || visibleThreads.length > 0 || newRowMatches}
                        >
                            "No chats match your filter."
                        </p>
                    </div>
                </aside>
                <main
                    class="flex min-h-0 min-w-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <header class="flex h-14 shrink-0 items-center gap-3 px-5" hidden={newChat}>
                        <span
                            class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-avatar text-xs font-semibold text-avatar-ink"
                        >
                            {initials(currentThread.label)}
                        </span>
                        <h2 class="truncate text-[15px] font-semibold text-ink">
                            {currentThread.label}
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
                    <section
                        class="new-chat-header relative shrink-0 border-b border-line"
                        hidden={!newChat}
                        aria-label="New conversation recipients"
                    >
                        <div
                            class="new-chat-group flex min-h-16 items-center gap-3.5 px-7 py-3 text-sm focus-within:shadow-[inset_0_-2px_var(--color-brand-ink)] border-b border-edge"
                            hidden={!groupNameOpen}
                        >
                            <label for={groupNameId} class="shrink-0 text-ink">"Group name:"</label>
                            <input
                                id={groupNameId}
                                class="min-w-[140px] flex-1 bg-transparent text-ink outline-none placeholder:text-muted"
                                pp-ref={groupNameInput}
                                aria-label="Group name"
                                maxlength="80"
                                value={groupName}
                                oninput={setGroupName(target.value)}
                                disabled={sending}
                            />
                        </div>
                        <div
                            class="flex min-h-16 items-center gap-3.5 px-7 py-3 text-sm focus-within:shadow-[inset_0_-2px_var(--color-brand-ink)]"
                        >
                            <label for={recipientId} class="shrink-0 text-ink">"To:"</label>
                            <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
                                <template pp-for="recipient in recipients">
                                    <span
                                        class="new-chat-chip inline-flex max-w-full items-center gap-1.5 rounded-[5px] bg-brand-soft px-2 py-1 text-[13px]"
                                        key={recipient}
                                    >
                                        <span class="truncate">{recipient}</span>
                                        <button
                                            type="button"
                                            class="grid shrink-0 cursor-pointer place-items-center rounded text-muted"
                                            aria-label={"Remove " + recipient}
                                            onclick={removeRecipient(recipient)}
                                            disabled={sending}
                                        >
                                            <Icon name="close" class="h-3.5 w-3.5" />
                                        </button>
                                    </span>
                                </template>
                                <input
                                    id={recipientId}
                                    pp-ref={recipientInput}
                                    class="min-w-[140px] flex-1 bg-transparent text-ink outline-none placeholder:text-muted"
                                    aria-label="Recipients"
                                    placeholder="Enter name, email or phone number"
                                    autocomplete="off"
                                    value={recipientQuery}
                                    oninput={changeRecipient(target.value)}
                                    onkeydown={handleRecipientKey(event)}
                                    disabled={sending}
                                    aria-invalid={!!recipientError}
                                    aria-describedby={recipientErrorId}
                                />
                            </div>
                            <button
                                type="button"
                                class="grid h-8 w-8 shrink-0 cursor-pointer place-items-center rounded text-muted hover:bg-hover"
                                aria-label={groupNameOpen ? "Hide group name" : "Show group name"}
                                aria-expanded={groupNameOpen}
                                onclick={toggleGroupName()}
                                disabled={sending}
                            >
                                <span class={groupNameOpen ? "flex rotate-180" : "flex"}>
                                    <Icon name="chevron-down" class="h-5 w-5" />
                                </span>
                            </button>
                        </div>
                        <p
                            class="new-chat-recipient-error px-7 pb-2 text-xs text-danger-ink"
                            id={recipientErrorId}
                            role="alert"
                            hidden={!recipientError}
                        >
                            {recipientError}
                        </p>
                        <div
                            class="absolute inset-x-7 top-full z-20 rounded-md border border-edge bg-raised p-1 shadow-xl shadow-black/40"
                            hidden={!recipientQuery.trim()}
                        >
                            <button
                                type="button"
                                class="flex w-full cursor-pointer items-center gap-2.5 rounded px-3 py-2.5 text-left text-[13px] hover:bg-hover"
                                hidden={!suggestSelf}
                                onclick={addRecipient(selfLabel)}
                                disabled={sending}
                            >
                                <span
                                    class="flex h-8 w-8 items-center justify-center rounded-full bg-avatar text-xs text-avatar-ink"
                                >
                                    @{initials.as_str()}
                                </span>
                                <span>@{self_label.as_str()}</span>
                            </button>
                            <button
                                type="button"
                                class="flex w-full cursor-pointer items-center gap-2.5 rounded px-3 py-2.5 text-left text-[13px] hover:bg-hover"
                                onclick={addRecipient(recipientQuery)}
                                disabled={sending}
                            >
                                "Add "
                                <strong>{recipientQuery}</strong>
                            </button>
                        </div>
                    </section>
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
                            hidden={newChat || messages.length > 0}
                        >
                            <Avatar initials=@{initials.as_str()} size="xl" presence="none" />
                            <h3 class="mt-7 text-[17px] font-semibold text-ink">
                                "This is your space"
                            </h3>
                            <p class="mt-2.5 text-[13px] leading-relaxed text-muted">
                                "This chat is just for you…with you. Use it for drafts, send files to yourself, or get to know chat features a little better."
                            </p>
                        </div>
                        <div
                            class="new-chat-empty flex flex-1 flex-col items-center justify-center gap-6 py-7 text-center"
                            hidden={!newChat}
                        >
                            <div class="relative isolate h-[185px] w-[200px]" aria-hidden="true">
                                <span class="new-chat-bubble"></span>
                                <span class="new-chat-pencil"></span>
                                <span class="new-chat-dot purple"></span>
                                <span class="new-chat-dot green"></span>
                                <span class="new-chat-dot yellow"></span>
                            </div>
                            <h2 class="mt-3.5 text-[17px] font-semibold">
                                "You're starting a new conversation"
                            </h2>
                            <p class="text-sm text-muted">"Type your first message below."</p>
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
                                        <template
                                            pp-for="url in message.gif_url ? [message.gif_url] : []"
                                        >
                                            <img
                                                key={url}
                                                src={url}
                                                alt="GIF from GIPHY"
                                                class="chat-message-gif"
                                                loading="lazy"
                                            />
                                        </template>
                                    </div>
                                </div>
                            </template>
                        </div>
                    </div>
                    <div class="shrink-0 px-8 pb-6 pt-2">
                        <div class="chat-draft-gif" hidden={!draftGif}>
                            <template pp-for="gif in draftGif ? [draftGif] : []">
                                <img
                                    key={gif.id}
                                    src={gif.url}
                                    alt={gif.title || "Selected GIF from GIPHY"}
                                />
                            </template>
                            <button
                                type="button"
                                aria-label="Remove selected GIF"
                                onclick={setDraftGif(null)}
                                disabled={sending}
                            >
                                <Icon name="close" class="h-4 w-4" />
                            </button>
                        </div>
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
                                aria-haspopup="dialog"
                                aria-expanded={emojiOpen}
                                aria-controls={emojiPickerId}
                                pp-ref={emojiTrigger}
                                onclick={toggleEmojiPicker()}
                                disabled={sending}
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
                                disabled={(!draft.trim() && !draftGif) || sending || (newChat && recipients.length === 0)}
                            >
                                <Icon name="send" class="h-[18px] w-[18px]" />
                            </button>
                        </form>
                        <p class="mt-2 text-xs text-subtle" hidden={selected === "self"}>
                            "New conversations are previews. Messages aren't delivered to recipients yet."
                        </p>
                        <p class="mt-2 text-xs text-busy" hidden={!error}>{error}</p>
                    </div>
                </main>
                <div class="fixed inset-0 z-45" hidden={!meetingOpen} onclick={closeMeetNow()}>
                </div>
                <section
                    class="chat-meet-now fixed z-46 w-[min(380px,calc(100vw-32px))] rounded-md border border-line bg-raised p-[22px] shadow-2xl shadow-black/70"
                    role="dialog"
                    aria-labelledby="chat-meet-now-title"
                    style={meetingPosition}
                    hidden={!meetingOpen}
                    onkeydown={handleMeetingKey(event)}
                >
                    <header class="mb-[22px] flex items-center justify-between gap-[18px]">
                        <h2 id="chat-meet-now-title" class="text-[17px] font-semibold">
                            "Start a meeting now"
                        </h2>
                        <button
                            type="button"
                            class="cursor-pointer text-muted hover:text-ink disabled:cursor-not-allowed disabled:opacity-50"
                            aria-label="Close meeting popup"
                            disabled={meetingBusy}
                            onclick={closeMeetNow()}
                        >
                            <Icon name="close" class="h-5 w-5" />
                        </button>
                    </header>
                    <form onsubmit={startChatMeeting(event)} novalidate>
                        <label for="chat-meeting-name" class="mb-1.5 block text-sm">
                            "Meeting name"
                        </label>
                        <input
                            id="chat-meeting-name"
                            class="h-10 w-full rounded border border-edge border-b-2 border-b-brand-ink bg-surface px-3 py-2 text-ink outline-none text-sm"
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
                        <p
                            class="mt-2.5 text-xs text-danger-ink"
                            role="alert"
                            hidden={!meetingError}
                        >
                            {meetingError}
                        </p>
                        <div class="chat-meeting-share mt-3.5" hidden={!sharedMeeting}>
                            <input
                                class="h-10 w-full rounded border border-edge border-b-2 border-b-brand-ink bg-surface px-3 py-2 text-ink outline-none text-xs"
                                aria-label="Meeting link"
                                readonly
                                value={sharedMeeting ? meetingAbsolute(sharedMeeting.url) : ""}
                            />
                            <p class="mt-2 text-xs text-muted" role="status">{meetingNotice}</p>
                            <a
                                class="mt-1.5 inline-block text-xs text-link hover:underline"
                                href={sharedMeeting ? sharedMeeting.url : "/meet"}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                "Open meeting in a new tab"
                            </a>
                        </div>
                        <button
                            type="button"
                            class="chat-meeting-share-button mt-[22px] block h-10 w-full cursor-pointer rounded text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50 border border-edge bg-transparent hover:bg-hover"
                            disabled={meetingBusy || !meetingTitle.trim()}
                            onclick={getChatMeetingLink()}
                        >
                            {meetingBusy ? "Creating…" : "Get a link to share"}
                        </button>
                        <button
                            type="submit"
                            class="chat-meeting-start mt-2.5 block h-10 w-full cursor-pointer rounded text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50 bg-brand text-white hover:bg-brand-hover"
                            disabled={meetingBusy || !meetingTitle.trim()}
                        >
                            {meetingBusy ? "Please wait…" : "Start meeting"}
                        </button>
                    </form>
                </section>
                <div
                    id={draftMenuId}
                    pp-ref={draftMenu}
                    popover="auto"
                    role="menu"
                    aria-label="New message options"
                    class="new-chat-menu rounded-md border border-edge bg-raised p-1 text-sm text-ink shadow-xl"
                    pp-style={draftMenuPosition}
                    ontoggle={setDraftMenuOpen(event.newState === "open")}
                    onkeydown={handleDraftMenuKey(event)}
                >
                    <button
                        type="button"
                        role="menuitem"
                        pp-ref={draftWindowAction}
                        class="flex w-full items-center gap-3 rounded px-3 py-2 text-left hover:bg-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand-ink"
                        onclick={openDraftWindow()}
                        disabled={sending}
                    >
                        <Icon name="maximize" class="h-4 w-4" />
                        "Open in new window"
                    </button>
                    <button
                        type="button"
                        role="menuitem"
                        pp-ref={draftDiscardAction}
                        class="flex w-full items-center gap-3 rounded px-3 py-2 text-left hover:bg-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand-ink"
                        onclick={discardNewChat()}
                        disabled={sending}
                    >
                        <Icon name="trash" class="h-4 w-4" />
                        "Discard"
                    </button>
                </div>
                <section
                    id={emojiPickerId}
                    pp-ref={emojiPicker}
                    popover="auto"
                    role="dialog"
                    aria-label="Emoji and GIF picker"
                    class="chat-emoji-picker"
                    pp-style={emojiPosition}
                    ontoggle={setEmojiOpen(event.newState === "open")}
                    onkeydown={handleEmojiKey(event)}
                >
                    <div
                        class="chat-emoji-tabs"
                        role="tablist"
                        aria-label="Media type"
                        onkeydown={handleEmojiTabKey(event)}
                    >
                        <button
                            type="button"
                            role="tab"
                            pp-ref={emojiAllTab}
                            aria-selected={emojiTab === "all"}
                            tabindex={emojiTab === "all" ? 0 : -1}
                            aria-controls={emojiPanelId}
                            onclick={setEmojiTab("all")}
                        >
                            "All"
                        </button>
                        <button
                            type="button"
                            role="tab"
                            pp-ref={emojiOnlyTab}
                            aria-selected={emojiTab === "emoji"}
                            tabindex={emojiTab === "emoji" ? 0 : -1}
                            aria-controls={emojiPanelId}
                            onclick={setEmojiTab("emoji")}
                        >
                            "Emoji"
                        </button>
                        <button
                            type="button"
                            role="tab"
                            pp-ref={emojiGifTab}
                            aria-selected={emojiTab === "gifs"}
                            tabindex={emojiTab === "gifs" ? 0 : -1}
                            aria-controls={emojiPanelId}
                            onclick={setEmojiTab("gifs")}
                        >
                            "GIFs"
                        </button>
                    </div>
                    <div class="chat-emoji-search">
                        <input
                            type="search"
                            pp-ref={emojiSearchInput}
                            aria-label="Find something fun"
                            placeholder="Find something fun"
                            value={emojiQuery}
                            oninput={searchMedia(target.value)}
                        />
                        <Icon name="search" class="h-[18px] w-[18px]" />
                    </div>
                    <div
                        class="chat-emoji-content scroll-thin"
                        id={emojiPanelId}
                        role="tabpanel"
                        aria-label={emojiTab === "gifs" ? "GIFs" : "Emoji"}
                    >
                        <div hidden={emojiTab === "gifs"}>
                            <div class="chat-emoji-heading">
                                <h3>"Emoji"</h3>
                                <button
                                    type="button"
                                    hidden={emojiTab !== "all"}
                                    onclick={setEmojiTab("emoji")}
                                >
                                    "See all "
                                    <Icon name="chevron-right" class="h-4 w-4" />
                                </button>
                            </div>
                            <div
                                class="chat-emoji-categories"
                                hidden={emojiTab !== "emoji"}
                                aria-label="Emoji categories"
                            >
                                <template pp-for="category in emojiCategories">
                                    <button
                                        key={category.id}
                                        type="button"
                                        aria-label={category.name}
                                        title={category.name}
                                        aria-pressed={emojiCategory === category.id}
                                        onclick={setEmojiCategory(category.id)}
                                    >
                                        {category.icon}
                                    </button>
                                </template>
                            </div>
                            <div class="chat-emoji-grid" aria-label="Emoji choices">
                                <template pp-for="item in shownEmojis">
                                    <button
                                        key={item.symbol}
                                        type="button"
                                        aria-label={item.name}
                                        title={item.name}
                                        onclick={insertEmoji(item.symbol)}
                                    >
                                        {item.symbol}
                                    </button>
                                </template>
                            </div>
                            <p
                                class="chat-emoji-empty"
                                role="status"
                                hidden={shownEmojis.length > 0}
                            >
                                "No emoji found. Try another search."
                            </p>
                        </div>
                        <div hidden={emojiTab === "emoji"}>
                            <div class="chat-emoji-heading">
                                <h3>"GIFs"</h3>
                                <button
                                    type="button"
                                    hidden={emojiTab !== "all"}
                                    onclick={setEmojiTab("gifs")}
                                >
                                    "See all "
                                    <Icon name="chevron-right" class="h-4 w-4" />
                                </button>
                            </div>
                            <p class="chat-emoji-empty" role="status" hidden={!gifLoading}>
                                "Loading GIFs…"
                            </p>
                            <p class="chat-emoji-empty" role="status" hidden={!gifError}>
                                {gifError}
                            </p>
                            <button
                                type="button"
                                class="chat-gif-retry"
                                hidden={!gifError || !giphyKey}
                                onclick={setGifRetry(value => value + 1)}
                            >
                                "Try again"
                            </button>
                            <div
                                class="chat-gif-grid"
                                aria-label="GIF choices"
                                aria-busy={gifLoading}
                            >
                                <template pp-for="gif in visibleGifs">
                                    <button
                                        key={gif.id}
                                        type="button"
                                        aria-label={gif.title || "Choose GIF"}
                                        onclick={chooseGif(gif)}
                                    >
                                        <img
                                            src={gif.preview}
                                            alt={gif.title || "GIF from GIPHY"}
                                            loading="lazy"
                                        />
                                    </button>
                                </template>
                            </div>
                            <p
                                class="chat-emoji-empty"
                                role="status"
                                hidden={gifLoading || !!gifError || gifs.length > 0}
                            >
                                "No GIFs found. Try another search."
                            </p>
                            <button
                                type="button"
                                class="chat-gif-more"
                                hidden={emojiTab !== "gifs" || !gifMore || !!gifError}
                                disabled={gifLoading}
                                onclick={setGifOffset(gifs.length)}
                            >
                                "Load more"
                            </button>
                        </div>
                    </div>
                    <footer class="chat-giphy-attribution" hidden={emojiTab === "emoji"}>
                        <a
                            href="https://giphy.com/"
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label="GIFs powered by GIPHY"
                        >
                            <img src="/images/powered-by-giphy.png" alt="GIFs powered by GIPHY" />
                        </a>
                        <a
                            href="https://support.giphy.com/hc/en-us/articles/360020027752-GIPHY-API-Terms-of-Service"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "View Terms"
                        </a>
                    </footer>
                </section>
                <style>r##"
                    .chat-emoji-picker {position:fixed;inset:auto;margin:0;width:360px;max-width:calc(100vw - 16px);height:480px;padding:0;border:1px solid #393939;border-radius:8px;background:#292929;color:#f5f5f5;box-shadow:0 12px 32px #0006;overflow:hidden;}
                    .chat-emoji-picker:popover-open {display:flex;flex-direction:column;}
                    .chat-emoji-tabs {display:flex;flex-shrink:0;background:#242424;padding:0 12px;}
                    .chat-emoji-tabs button {position:relative;flex:1;height:56px;font-size:15px;color:#ccc;}
                    .chat-emoji-tabs button[aria-selected=true] {color:white;font-weight:600;}
                    .chat-emoji-tabs button[aria-selected=true]::after {content:'';position:absolute;bottom:0;left:calc(50% - 10px);width:20px;height:3px;border-radius:3px;background:#9287ff;}
                    .chat-emoji-search {position:relative;margin:16px 16px 8px;flex-shrink:0;}
                    .chat-emoji-search input {width:100%;height:38px;padding:0 38px 0 12px;border:0;border-radius:5px;background:#202020;color:white;font-size:14px;outline-offset:2px;}
                    .chat-emoji-search > svg {position:absolute;right:12px;top:10px;color:#aaa;pointer-events:none;}
                    .chat-emoji-content {overflow-y:auto;min-height:0;flex:1;padding:8px 16px 16px;}
                    .chat-emoji-heading {display:flex;align-items:center;justify-content:space-between;margin:12px 0 10px;font-size:14px;}
                    .chat-emoji-heading h3 {font-weight:600;}
                    .chat-emoji-heading button {display:flex;align-items:center;gap:4px;color:#ccc;}
                    .chat-emoji-grid {display:grid;grid-template-columns:repeat(6,minmax(0,1fr));gap:4px;}
                    .chat-emoji-grid button {height:44px;font-size:29px;line-height:1;border-radius:5px;}
                    .chat-emoji-picker button:hover {background:#ffffff0d;}
                    .chat-emoji-picker button:focus-visible {outline:2px solid #b6acff;outline-offset:-2px;}
                    .chat-emoji-categories {display:flex;flex-wrap:wrap;gap:3px;margin-bottom:12px;}
                    .chat-emoji-categories button {width:34px;height:32px;border-radius:4px;font-size:19px;}
                    .chat-emoji-categories button[aria-pressed=true] {background:#514b79;}
                    .chat-emoji-empty {padding:16px 0;font-size:13px;color:#b6b6bd;line-height:1.5;}
                    .chat-gif-grid {display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:6px;}
                    .chat-gif-grid button {height:92px;border-radius:5px;overflow:hidden;background:#202020;}
                    .chat-gif-grid img {width:100%;height:100%;object-fit:cover;}
                    .chat-giphy-attribution {flex-shrink:0;display:flex;align-items:center;justify-content:center;gap:10px;padding:10px 12px;border-top:1px solid #393939;background:#242424;font-size:12px;}
                    .chat-giphy-attribution img {width:125px;height:auto;}
                    .chat-giphy-attribution > a:last-child {color:#a89aff;}
                    .chat-gif-more,.chat-gif-retry {padding:8px 12px;border-radius:4px;color:#c4baff;font-size:13px;}
                    .chat-gif-more {display:block;margin:10px auto 0;}
                    .chat-message-gif {display:block;max-width:min(320px,100%);max-height:280px;border-radius:8px;margin-top:6px;}
                    .chat-draft-gif {position:relative;display:inline-flex;margin-bottom:8px;padding:8px;border:1px solid #393939;border-radius:8px;}
                    .chat-draft-gif img {max-width:180px;max-height:120px;border-radius:4px;}
                    .chat-draft-gif button {position:absolute;right:4px;top:4px;padding:5px;border-radius:50%;background:#202020;color:white;}
                    .new-chat-menu {position:fixed;inset:auto;margin:0;width:240px;max-width:calc(100vw - 16px);}
                    /* The new-conversation illustration: layered gradients and pseudo-element
                       shapes that utilities cannot express readably. */
                    .new-chat-bubble {position:absolute;inset:14px 16px 10px 28px;border-radius:50%;background:radial-gradient(circle at 32% 28%,#48484d,#29292f 58%,#1f1f24 80%);box-shadow:inset 3px 2px 7px #ffffff20,4px 5px 8px #0005;}
                    .new-chat-bubble::after {content:"";position:absolute;bottom:-4px;left:22px;width:30px;height:38px;transform:skewY(-30deg);border-radius:0 0 5px 10px;background:linear-gradient(120deg,#35353c,#222228);z-index:-1;}
                    .new-chat-pencil {position:absolute;left:114px;top:4px;width:40px;height:126px;transform:rotate(40deg);border-radius:20px 20px 2px 2px;background:linear-gradient(90deg,#ffb342,#ffdd7a 45%,#ed883b);box-shadow:inset -5px 0 6px #a74d3a40,4px 5px 10px #0006;}
                    .new-chat-pencil::before {content:"";position:absolute;top:0;left:0;right:0;height:30px;border-radius:20px 20px 0 0;background:linear-gradient(90deg,#777,#bbb,#888);}
                    .new-chat-pencil::after {content:"";position:absolute;top:calc(100% - 1px);left:0;width:0;height:0;border-left:20px solid transparent;border-right:20px solid transparent;border-top:30px solid #ffe495;}
                    .new-chat-dot {position:absolute;top:108px;width:21px;height:21px;border-radius:50%;box-shadow:inset -3px -3px 5px #0005,1px 2px 3px #0005;}
                    .new-chat-dot.purple {left:5px;background:radial-gradient(at 30% 20%,#dbbaff,#9460ed 65%);}
                    .new-chat-dot.green {left:31px;background:radial-gradient(at 30% 20%,#f3ffbd,#a3ce53 65%);}
                    .new-chat-dot.yellow {left:57px;background:radial-gradient(at 30% 20%,#fffccb,#edd445 65%);}
                "##</style>
                <script>r##"
                    const PILL_ON = "h-7 shrink-0 rounded-full border border-brand bg-brand-soft px-3 text-xs font-semibold text-ink";
                    const PILL_OFF = "h-7 shrink-0 rounded-full border border-edge px-3 text-xs text-muted transition-colors hover:bg-hover hover:text-ink";
                    const ROW_ON = "flex w-full items-center gap-3 rounded-md bg-raised px-3 py-2 text-left";
                    const ROW_OFF = "flex w-full items-center gap-3 rounded-md px-3 py-2 text-left transition-colors hover:bg-hover";

                    const shortcuts = pp.context(window.UpurnakaShortcutContext);
                    const composerInput = pp.ref(null);
                    const emojiTrigger = pp.ref(null);
                    const emojiPicker = pp.ref(null);
                    const emojiSearchInput = pp.ref(null);
                    const emojiAllTab = pp.ref(null);
                    const emojiOnlyTab = pp.ref(null);
                    const emojiGifTab = pp.ref(null);
                    const emojiPickerId = pp.id();
                    const emojiPanelId = pp.id();
                    const emojiSelection = pp.ref({start:0, end:0});
                    const emojiCaret = pp.ref(null);
                    const [emojiOpen, setEmojiOpen] = pp.state(false);
                    const [emojiPosition, setEmojiPosition] = pp.state('');
                    const [emojiTab, setEmojiTab] = pp.state('all');
                    const [emojiQuery, setEmojiQuery] = pp.state('');
                    const [emojiCategory, setEmojiCategory] = pp.state('faces');
                    const giphyKey = @{giphy_key};
                    const [gifs, setGifs] = pp.state([]);
                    const [gifLoading, setGifLoading] = pp.state(false);
                    const [gifError, setGifError] = pp.state('');
                    const [gifOffset, setGifOffset] = pp.state(0);
                    const [gifMore, setGifMore] = pp.state(false);
                    const [gifRetry, setGifRetry] = pp.state(0);
                    const visibleGifs = emojiTab === 'all' ? gifs.slice(0,6) : gifs;
                    const emojiCategories = [
                        {id:'faces', name:'Smileys and emotions', icon:'😀', entries:[['😮','Surprised wow'],['😰','Anxious worried'],['😠','Angry'],['😍','Heart eyes love'],['😎','Cool sunglasses'],['🙌','Raised hands hooray'],['🫶','Heart hands'],['🔥','Fire lit'],['🎉','Party celebration'],['👀','Eyes looking'],['💯','Hundred perfect'],['💡','Light bulb idea'],['😀','Grinning smile happy'],['😂','Tears of joy laugh'],['😊','Smiling blush'],['🥰','Smiling hearts'],['🤔','Thinking'],['😭','Crying sad'],['🥳','Party face'],['😴','Sleeping tired'],['🤩','Star struck'],['😅','Sweat smile'],['😉','Wink'],['🤗','Hug']]},
                        {id:'hands', name:'People and gestures', icon:'👋', entries:[['👋','Wave hello goodbye'],['👍','Thumbs up yes'],['👎','Thumbs down no'],['👏','Clap applause'],['🙏','Thanks prayer please'],['🤝','Handshake agreement'],['✌️','Victory peace'],['💪','Flex muscle strong'],['🤞','Fingers crossed luck'],['👌','Okay hand'],['🫡','Salute'],['🤷','Shrug']]},
                        {id:'nature', name:'Animals and nature', icon:'🐻', entries:[['🐶','Dog puppy'],['🐱','Cat kitten'],['🐻','Bear'],['🦊','Fox'],['🦋','Butterfly'],['🐝','Bee'],['🌻','Sunflower flower'],['🌱','Seedling plant'],['🌳','Tree'],['🌈','Rainbow'],['☀️','Sun sunny'],['🌙','Moon night']]},
                        {id:'food', name:'Food and drink', icon:'🍔', entries:[['☕','Coffee hot drink'],['🍕','Pizza'],['🍔','Burger'],['🍟','French fries'],['🎂','Birthday cake'],['🍰','Cake slice'],['🍎','Apple fruit'],['🥑','Avocado'],['🌮','Taco'],['🍿','Popcorn'],['🍩','Doughnut donut'],['🧃','Juice']]},
                        {id:'activities', name:'Activities and travel', icon:'⚽', entries:[['⚽','Soccer football'],['🏀','Basketball'],['🏆','Trophy winner'],['🎮','Video game controller'],['🎵','Music note'],['🎨','Art palette'],['🚗','Car'],['✈️','Airplane travel'],['🚀','Rocket launch'],['🏠','House home'],['🏖️','Beach holiday'],['🌍','Earth world']]},
                        {id:'symbols', name:'Objects and symbols', icon:'❤️', entries:[['❤️','Red heart love'],['💜','Purple heart'],['✅','Check done yes'],['❌','Cross no'],['⭐','Star'],['✨','Sparkles'],['🎁','Gift present'],['📅','Calendar date'],['📌','Pin'],['💻','Laptop computer'],['🔔','Bell notification'],['⚠️','Warning']]}
                    ];
                    const emojiNeedle = emojiQuery.trim().toLocaleLowerCase();
                    const emojiItems = emojiCategories.flatMap(category => category.entries.map(([symbol,name]) => ({symbol,name,category:category.id})));
                    const shownEmojis = emojiItems.filter(item => emojiNeedle ? item.name.toLocaleLowerCase().includes(emojiNeedle) || item.symbol === emojiNeedle : emojiTab !== 'emoji' || item.category === emojiCategory).slice(0, emojiTab === 'all' && !emojiNeedle ? 12 : 120);
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
                    // Conversations currently live in this page, like the original
                    // self-chat. The message RPC validates text; it does not deliver
                    // messages to external recipients.
                    const selfName = @{user.name.as_str()};
                    const selfLabel = @{self_label.as_str()};
                    const selfEmail = @{user.email.as_str()};
                    const [threads, setThreads] = pp.state([{id:"self", label:selfLabel, messages:[], draft:"", gif:null, recipients:[]}]);
                    const [hasNewChat, setHasNewChat] = pp.state(false);
                    const [newDraft, setNewDraft] = pp.state("");
                    const [newGif, setNewGif] = pp.state(null);
                    const [recipients, setRecipients] = pp.state([]);
                    const [recipientQuery, setRecipientQuery] = pp.state("");
                    const [recipientError, setRecipientError] = pp.state("");
                    const [groupName, setGroupName] = pp.state("");
                    const [groupNameOpen, setGroupNameOpen] = pp.state(false);
                    const draftMenu = pp.ref(null);
                    const draftMenuTrigger = pp.ref(null);
                    const draftWindowAction = pp.ref(null);
                    const draftDiscardAction = pp.ref(null);
                    const draftMenuId = pp.id();
                    const [draftMenuOpen, setDraftMenuOpen] = pp.state(false);
                    const [draftMenuPosition, setDraftMenuPosition] = pp.state("");
                    const recipientInput = pp.ref(null);
                    const groupNameInput = pp.ref(null);
                    const recipientId = pp.id();
                    const groupNameId = pp.id();
                    const recipientErrorId = pp.id();
                    const newChat = selected === "new";
                    const currentThread = threads.find(thread => thread.id === selected) || threads[0];
                    const messages = newChat ? [] : currentThread.messages;
                    const draft = newChat ? newDraft : currentThread.draft;
                    const draftGif = newChat ? newGif : currentThread.gif;
                    const suggestSelf = !recipients.includes(selfLabel) && recipientQuery.trim().length > 0 && (selfLabel + ' ' + selfEmail).toLocaleLowerCase().includes(recipientQuery.toLocaleLowerCase());
                    const [sending, setSending] = pp.state(false);
                    const [error, setError] = pp.state("");
                    const [searchOpen, setSearchOpen] = pp.state(false);
                    const [chatFilter, setChatFilter] = pp.state("");
                    const chatSearchInput = pp.ref(null);
                    const chatSearchTrigger = pp.ref(null);
                    const [meetingOpen, setMeetingOpen] = pp.state(false);
                    const [meetingTitle, setMeetingTitle] = pp.state("Meeting with " + selfName);
                    const [meetingBusy, setMeetingBusy] = pp.state(false);
                    const [meetingError, setMeetingError] = pp.state("");
                    const [meetingNotice, setMeetingNotice] = pp.state("");
                    const [sharedMeeting, setSharedMeeting] = pp.state(null);
                    const [meetingPosition, setMeetingPosition] = pp.state("");
                    const meetingNameInput = pp.ref(null);
                    const meetNowTrigger = pp.ref(null);
                    const hasChatFilter = !!chatFilter.trim() || filters.unread || filters.meetings || filters.unmuted;
                    const selfMatches = selfLabel.toLocaleLowerCase().includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings;

                    const newRowMatches = hasNewChat && "new message".includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings;
                    const visibleThreads = threads.filter(thread => thread.id !== "self" && thread.label.toLocaleLowerCase().includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings);
                    function initials(label) {return label.trim().split(/\s+/).slice(0,2).map(word => word[0] || "").join("").toUpperCase();}
                    function setDraft(value) {
                        if (newChat) setNewDraft(value);
                        else setThreads(previous => previous.map(thread => thread.id === selected ? {...thread, draft:value} : thread));
                    }
                    function setDraftGif(gif) {
                        if (newChat) setNewGif(gif);
                        else setThreads(previous => previous.map(thread => thread.id === selected ? {...thread, gif} : thread));
                    }
                    function searchMedia(value) {setEmojiQuery(value); setGifOffset(0);}
                    function giphyImage(value) {
                        try {const url = new URL(value); return url.protocol === 'https:' && /^(media[0-4]?|i)\.giphy\.com$/.test(url.hostname) && !url.username && !url.password && !url.port ? url.href : '';}
                        catch (failure) {return '';}
                    }
                    function chooseGif(gif) {
                        if (sending) return;
                        setDraftGif({id:gif.id, url:gif.url, title:gif.title});
                        emojiPicker.current.hidePopover(); composerInput.current?.focus();
                    }
                    pp.effect(() => {
                        if (!emojiOpen || emojiTab === 'emoji') return;
                        if (!giphyKey) {setGifError('GIPHY is not configured yet.'); setGifLoading(false); return;}
                        let active = true;
                        const controller = new AbortController();
                        setGifLoading(true); setGifError('');
                        if (!gifOffset) setGifs([]);
                        const timer = setTimeout(async () => {
                            const timeout = setTimeout(() => controller.abort(), 10000);
                            try {
                                const url = new URL('https://api.giphy.com/v1/gifs/' + (emojiQuery.trim() ? 'search' : 'trending'));
                                url.search = new URLSearchParams({api_key:giphyKey, limit:'18', offset:String(gifOffset), rating:'g', ...(emojiQuery.trim() ? {q:emojiQuery} : {})});
                                const response = await fetch(url, {signal:controller.signal, credentials:'omit', referrerPolicy:'no-referrer'});
                                if (!response.ok) throw new Error(response.status === 429 ? 'GIPHY is busy. Please try again later.' : 'GIFs could not be loaded. Please try again.');
                                const body = await response.json();
                                if (!Array.isArray(body.data)) throw new Error('GIFs could not be loaded. Please try again.');
                                const items = body.data.map(item => {
                                    if (!/^[A-Za-z0-9]{1,80}$/.test(item.id)) throw new Error('GIPHY returned an unsupported GIF. Please try again.');
                                    const url = 'https://media.giphy.com/media/' + item.id + '/giphy.gif';
                                    return {id:item.id, title:item.title || 'GIF from GIPHY', url, preview:giphyImage(item.images?.fixed_height_small?.url) || url};
                                });
                                if (!active) return;
                                setGifs(previous => gifOffset ? previous.concat(items.filter(item => !previous.some(existing => existing.id === item.id))) : items);
                                setGifMore(items.length > 0 && gifOffset + items.length < Math.min(499, body.pagination?.total_count || 0));
                            } catch (failure) {
                                if (active) setGifError(failure.name === 'AbortError' ? 'GIPHY took too long to respond. Please try again.' : (failure.message === 'Failed to fetch' ? 'GIFs could not be loaded. Check your connection and try again.' : failure.message));
                            } finally {clearTimeout(timeout); if (active) setGifLoading(false);}
                        }, emojiQuery.trim() ? 350 : 0);
                        return () => {active = false; clearTimeout(timer); controller.abort();};
                    }, [emojiOpen, emojiTab, emojiQuery, gifOffset, gifRetry]);
                    function toggleEmojiPicker() {
                        if (emojiPicker.current.matches(':popover-open')) {emojiPicker.current.hidePopover(); return;}
                        const input = composerInput.current;
                        emojiSelection.current = {start:input.selectionStart ?? draft.length, end:input.selectionEnd ?? draft.length};
                        const anchor = emojiTrigger.current.getBoundingClientRect();
                        const width = Math.min(360, innerWidth - 16);
                        const left = Math.max(8, Math.min(anchor.right - width, innerWidth - width - 8));
                        setEmojiPosition('left:' + left + 'px;bottom:' + (innerHeight - anchor.top + 8) + 'px;max-height:' + Math.max(100, anchor.top - 16) + 'px');
                        emojiPicker.current.showPopover(); emojiSearchInput.current?.focus();
                    }
                    function insertEmoji(symbol) {
                        if (sending) return;
                        const {start,end} = emojiSelection.current;
                        emojiCaret.current = start + symbol.length;
                        setDraft(draft.slice(0,start) + symbol + draft.slice(end));
                        emojiPicker.current.hidePopover(); composerInput.current?.focus();
                    }
                    pp.layoutEffect(() => {
                        if (emojiCaret.current === null) return;
                        composerInput.current?.setSelectionRange(emojiCaret.current, emojiCaret.current);
                        emojiCaret.current = null;
                    }, [draft]);
                    function handleEmojiKey(event) {
                        if (event.key === 'Escape') {
                            event.preventDefault(); event.stopPropagation();
                            emojiPicker.current.hidePopover(); emojiTrigger.current?.focus();
                        }
                    }
                    function handleEmojiTabKey(event) {
                        if (!['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) return;
                        event.preventDefault();
                        const names = ['all','emoji','gifs'];
                        const refs = [emojiAllTab,emojiOnlyTab,emojiGifTab];
                        let index = names.indexOf(emojiTab);
                        if (event.key === 'Home') index = 0;
                        else if (event.key === 'End') index = 2;
                        else index = (index + (event.key === 'ArrowRight' ? 1 : 2)) % 3;
                        setEmojiTab(names[index]); refs[index].current?.focus();
                    }
                    function selectThread(id) {
                        if (sending) return;
                        emojiPicker.current?.hidePopover();
                        setSelected(id); setConversationQuery(""); setConversationSearchOpen(false); setError("");
                    }
                    function openNewChat() {
                        if (sending) return;
                        emojiPicker.current?.hidePopover();
                        setHasNewChat(true); setSelected("new"); setChatsOpen(true);
                        setChatFilter(""); setFilters({unread:false, meetings:false, unmuted:false});
                        setConversationQuery(""); setConversationSearchOpen(false); setError("");
                        recipientInput.current?.focus();
                    }
                    function openDraftMenu() {
                        if (sending) return;
                        if (draftMenu.current?.matches(':popover-open')) {draftMenu.current.hidePopover(); return;}
                        const anchor = draftMenuTrigger.current.getBoundingClientRect();
                        const width = Math.min(240, innerWidth - 16);
                        const left = Math.max(8, Math.min(anchor.left, innerWidth - width - 8));
                        const top = Math.max(8, Math.min(anchor.bottom + 4, innerHeight - 100));
                        setDraftMenuPosition('left:' + left + 'px;top:' + top + 'px');
                        draftMenu.current?.showPopover();
                        draftWindowAction.current?.focus();
                    }
                    function handleDraftMenuKey(event) {
                        if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
                            event.preventDefault();
                            const first = event.key === 'Home' || (event.key !== 'End' && document.activeElement !== draftWindowAction.current);
                            (first ? draftWindowAction : draftDiscardAction).current?.focus();
                        } else if (event.key === 'Escape') {
                            event.preventDefault(); draftMenu.current?.hidePopover(); draftMenuTrigger.current?.focus();
                        } else if (event.key === 'Tab') {
                            draftMenu.current?.hidePopover(); draftMenuTrigger.current?.focus();
                        }
                    }
                    function discardNewChat() {
                        if (sending) return;
                        draftMenu.current?.hidePopover();
                        setHasNewChat(false); setNewDraft(''); setNewGif(null); setRecipients([]); setRecipientQuery('');
                        setRecipientError(''); setGroupName(''); setGroupNameOpen(false);
                        if (newChat) selectThread('self');
                        setError(''); composerInput.current?.focus();
                    }
                    function openDraftWindow() {
                        if (sending) return;
                        draftMenu.current?.hidePopover();
                        const popup = window.open('about:blank', '_blank', 'popup,width=1000,height=760');
                        if (!popup) {setError('Allow pop-ups to open this draft in a new window.'); return;}
                        try {
                            popup.sessionStorage.setItem('upurnaka.chat.compose', JSON.stringify({owner:selfEmail, draft:newDraft, gif:newGif, recipients, recipientQuery, groupName, groupNameOpen}));
                            popup.opener = null;
                            popup.location.replace('/chat?compose=1');
                        } catch (failure) {
                            popup.close(); setError('The draft could not be opened. Please try again.');
                        }
                    }
                    pp.effect(() => {
                        const params = new URLSearchParams(location.search);
                        if (params.get('compose') !== '1') return;
                        openNewChat();
                        try {
                            const raw = sessionStorage.getItem('upurnaka.chat.compose');
                            sessionStorage.removeItem('upurnaka.chat.compose');
                            const saved = raw ? JSON.parse(raw) : null;
                            if (saved?.owner === selfEmail) {
                                setNewDraft(typeof saved.draft === 'string' ? saved.draft : '');
                                if (saved.gif && /^[A-Za-z0-9]{1,80}$/.test(saved.gif.id)) setNewGif({id:saved.gif.id, title:String(saved.gif.title || 'GIF from GIPHY'), url:'https://media.giphy.com/media/' + saved.gif.id + '/giphy.gif'});
                                setRecipients(Array.isArray(saved.recipients) ? saved.recipients.filter(value => typeof value === 'string').slice(0, 20) : []);
                                setRecipientQuery(typeof saved.recipientQuery === 'string' ? saved.recipientQuery : '');
                                setGroupName(typeof saved.groupName === 'string' ? saved.groupName : '');
                                setGroupNameOpen(saved.groupNameOpen === true);
                            }
                        } catch (failure) {setError('The draft could not be restored. You can start a new message.');}
                    }, []);
                    pp.effect(() => {if (newChat) recipientInput.current?.focus();}, [newChat]);
                    pp.effect(() => {if (newChat && groupNameOpen) groupNameInput.current?.focus();}, [newChat, groupNameOpen]);
                    function toggleGroupName() {
                        setGroupNameOpen(open => !open);
                        if (groupNameOpen) recipientInput.current?.focus();
                    }
                    function changeRecipient(value) {setRecipientQuery(value); setRecipientError("");}
                    function addRecipient(raw) {
                        const value = raw.trim();
                        if (!value) return;
                        if (value.length > 120 || (value.includes("@") && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value))) {
                            setRecipientError("Enter a valid name, email address or phone number (up to 120 characters)."); return;
                        }
                        if (recipients.length >= 20) {setRecipientError("You can add up to 20 recipients."); return;}
                        if (!recipients.some(item => item.toLocaleLowerCase() === value.toLocaleLowerCase())) setRecipients(previous => [...previous, value]);
                        setRecipientQuery(""); setRecipientError(""); recipientInput.current?.focus();
                    }
                    function removeRecipient(value) {setRecipients(previous => previous.filter(item => item !== value)); recipientInput.current?.focus();}
                    function handleRecipientKey(event) {
                        if (event.isComposing) return;
                        if (event.key === "Enter" || event.key === ";" || event.key === ",") {
                            event.preventDefault(); addRecipient(recipientQuery);
                        } else if (event.key === "Backspace" && !recipientQuery && recipients.length) {
                            event.preventDefault(); setRecipients(previous => previous.slice(0,-1));
                        } else if (event.key === "Escape") {
                            event.preventDefault(); setRecipientQuery(""); setRecipientError(""); composerInput.current?.focus();
                        }
                    }
                    const matchingMessages = messages.filter(message => message.text.toLocaleLowerCase().includes(conversationQuery.trim().toLocaleLowerCase()));
                    const handleChatShortcut = pp.event(event => {
                        if (meetingOpen || event.isComposing) return;
                        const key = event.key.toLowerCase();
                        const control = (event.ctrlKey || event.metaKey) && !event.altKey;
                        let action = null;
                        if (control && !event.shiftKey && key === "n") action = openNewChat;
                        else if (control && !event.shiftKey && key === "r") action = () => composerInput.current?.focus();
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
                        if ((!text && !draftGif) || sending || (newChat && recipients.length === 0)) return;
                        const targetId = selected;
                        const targetLabel = groupName.trim() || recipients.join(", ");
                        const targetRecipients = [...recipients];

                        setSending(true);
                        setError("");
                        try {
                            const message = await pp.rpc("send_message", { text, gif_id: draftGif?.id || null });
                            if (targetId === "new") {
                                const id = "chat-" + crypto.randomUUID();
                                setThreads(previous => [...previous, {id, label:targetLabel, recipients:targetRecipients, messages:[message], draft:"", gif:null}]);
                                setSelected(id); setHasNewChat(false); setNewDraft(""); setNewGif(null);
                                setRecipients([]); setRecipientQuery(""); setGroupName(""); setGroupNameOpen(false);
                            } else {
                                setThreads(previous => previous.map(thread => thread.id === targetId ? {...thread, messages:[...thread.messages, message], draft:"", gif:null} : thread));
                            }
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
    pub gif_url: Option<String>,
}

/// Accept a line the composer sent.
///
/// The thread is browser state for now, so the server's job is the part the
/// browser should not decide: the text is checked here, and the stamped time
/// comes from the application clock rather than from whatever the device
/// believes the hour is.
#[rpc(auth)]
pub async fn send_message(text: String, gif_id: Option<String>) -> Result<Message> {
    let text = text.trim().to_string();

    let mut validator = Validator::new();
    validator
        .check(
            "text",
            !text.is_empty() || gif_id.is_some(),
            "Write a message or choose a GIF.",
        )
        .max_chars("text", &text, 2000);
    if let Some(id) = &gif_id {
        validator.check(
            "gif_id",
            !id.is_empty() && id.len() <= 80 && id.bytes().all(|c| c.is_ascii_alphanumeric()),
            "Choose a valid GIPHY GIF.",
        );
    }
    validator.finish()?;

    let now = time::now();
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();

    Ok(Message {
        id: format!("m{}", now.timestamp_micros()),
        author: user.name,
        initials,
        time: now.format("%-I:%M %p").to_string(),
        text,
        gif_url: gif_id.map(|id| format!("https://media.giphy.com/media/{id}/giphy.gif")),
    })
}

/// Chat's Meet now action creates the same persistent link listed by Meet.
#[rpc(auth)]
pub async fn create_chat_meeting(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;
    meetings::create(title).await
}
