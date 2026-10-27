//! Chat: the conversation list on the left, one thread on the right.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::aria::{bound, hidden, labelled, named, plain, submit};
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::{Attrs, ui_expr};
use crate::components::rahti_ui::avatar::{Avatar as UiAvatar, AvatarFallback};
use crate::components::rahti_ui::badge::Badge;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger,
};
use crate::components::rahti_ui::input::{Input, input_variants};
use crate::components::rahti_ui::input_group::{
    InputGroup, InputGroupAddon, input_group_input_variants, input_group_textarea_variants,
};
use crate::components::rahti_ui::label::Label;
use crate::components::rahti_ui::popover::{
    Popover, PopoverContent, PopoverHeader, PopoverTitle, PopoverTrigger,
};
use crate::components::rahti_ui::toggle::Toggle;
use crate::components::rahti_ui::toggle_group::{ToggleGroup, ToggleGroupItem};
use crate::meetings::{self, MeetingLink};
use crate::rahti::time;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

/// Initials on a disc, for a conversation or a message author.
const INITIALS: &str = "bg-avatar! text-xs font-semibold text-avatar-ink!";

/// The emoji and GIF picker: a column of tabs, search, choices and attribution.
const PICKER: &str = "chat-emoji-picker h-[480px] max-h-[calc(100dvh-16px)] w-[360px]!                       max-w-[calc(100vw-16px)] flex-col overflow-hidden p-0! open:flex";

/// The heading over a run of emoji or GIFs, with its "See all" beside it.
const HEADING: &str = "mb-2.5 mt-3 flex items-center justify-between text-sm";

/// A picker's empty, loading or failure line.
const EMPTY: &str = "chat-emoji-empty py-4 text-[13px] leading-normal text-muted-foreground";

// The new-conversation illustration: layered gradients and pseudo-element
// shapes, drawn with arbitrary utilities.
const BUBBLE: &str = "absolute inset-[14px_16px_10px_28px] rounded-full                       bg-[radial-gradient(circle_at_32%_28%,#48484d,#29292f_58%,#1f1f24_80%)]                       shadow-[inset_3px_2px_7px_#ffffff20,4px_5px_8px_#0005]                       after:absolute after:-bottom-1 after:left-[22px] after:-z-10                       after:h-[38px] after:w-[30px] after:-skew-y-30 after:rounded-[0_0_5px_10px]                       after:bg-[linear-gradient(120deg,#35353c,#222228)] after:content-['']";
const PENCIL: &str = "absolute left-[114px] top-1 h-[126px] w-10 rotate-40                       rounded-[20px_20px_2px_2px]                       bg-[linear-gradient(90deg,#ffb342,#ffdd7a_45%,#ed883b)]                       shadow-[inset_-5px_0_6px_#a74d3a40,4px_5px_10px_#0006]                       before:absolute before:inset-x-0 before:top-0 before:h-[30px]                       before:rounded-[20px_20px_0_0] before:bg-[linear-gradient(90deg,#777,#bbb,#888)]                       before:content-[''] after:absolute after:left-0 after:top-[calc(100%-1px)]                       after:size-0 after:border-x-[20px] after:border-t-[30px]                       after:border-x-transparent after:border-t-[#ffe495] after:content-['']";
const DOT_PURPLE: &str = "absolute left-[5px] top-[108px] size-[21px] rounded-full                           bg-[radial-gradient(at_30%_20%,#dbbaff,#9460ed_65%)]                           shadow-[inset_-3px_-3px_5px_#0005,1px_2px_3px_#0005]";
const DOT_GREEN: &str = "absolute left-[31px] top-[108px] size-[21px] rounded-full                          bg-[radial-gradient(at_30%_20%,#f3ffbd,#a3ce53_65%)]                          shadow-[inset_-3px_-3px_5px_#0005,1px_2px_3px_#0005]";
const DOT_YELLOW: &str = "absolute left-[57px] top-[108px] size-[21px] rounded-full                           bg-[radial-gradient(at_30%_20%,#fffccb,#edd445_65%)]                           shadow-[inset_-3px_-3px_5px_#0005,1px_2px_3px_#0005]";

/// A button that hides the popover `name`: its id is `popover-{name}`, named
/// by `popovertarget`, so no script is involved.
fn close_popover(name: &str, label: &str) -> Attrs {
    Attrs::new()
        .set("popovertarget", format!("popover-{name}"))
        .set("popovertargetaction", "hide")
        .merge(&labelled(label))
}

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Chat")
        .description("Your Teams conversations.")
}

/// A row in the conversation list, highlighted while it is the open one.
const ROW: &str = "rounded-md transition-colors";

/// A list row's button: the whole width, its content from the left.
const ROW_BUTTON: &str = "h-auto! w-full justify-start! gap-3 px-3! py-2 text-left font-normal \
                          hover:bg-hover";

/// A section heading in the conversation list.
const SECTION: &str = "h-auto! w-full justify-start! gap-1.5 rounded px-2! py-1.5 text-xs \
                       font-semibold text-muted-foreground hover:bg-hover hover:text-ink";

/// A filter pill above the list.
const PILL: &str = "h-7! min-w-0 shrink-0 rounded-full! px-3! text-xs text-muted-foreground \
                    has-[:checked]:border-brand has-[:checked]:bg-brand-soft! \
                    has-[:checked]:font-semibold has-[:checked]:text-ink!";

/// A tab across the top of the emoji and GIF picker.
const MEDIA_TAB: &str = "relative h-14! flex-1 rounded-none! text-[15px] text-muted-foreground \
                         hover:bg-transparent! has-[:checked]:font-semibold \
                         has-[:checked]:text-ink! has-[:checked]:bg-transparent! \
                         after:absolute after:bottom-0 after:left-[calc(50%-10px)] after:h-[3px] \
                         after:w-5 after:rounded-[3px] after:bg-brand-ink after:opacity-0 \
                         after:content-[''] has-[:checked]:after:opacity-100";

pub async fn page() -> Html {
    let user = crate::auth::current().unwrap_or_default();
    // GIPHY requires direct browser requests; this is a public web integration key.
    let giphy_key = std::env::var("GIPHY_API_KEY").unwrap_or_default();
    #[cfg(test)]
    let giphy_key = crate::app_tests::giphy::key_for(&user.id).unwrap_or(giphy_key);
    let initials = user.initials();
    let self_label = format!("{} (You)", user.name);
    let icon_button = button_variants(
        ButtonVariant::Ghost,
        ButtonSize::IconSm,
        "text-muted-foreground",
    );
    // The fields below are elements rather than `<Input>` tags because each one
    // is focused from a script — opening a search, starting a new chat — and a
    // ref is a binding a component tag cannot carry. They wear the same class
    // lists the components would.
    let list_search = input_variants("h-9! text-[13px]");
    let inline_field = input_group_input_variants("text-sm");
    let conversation_search = input_variants("text-sm");
    let meeting_name = input_variants("h-10! text-sm");
    let emoji_search = input_group_input_variants("text-sm");
    let composer_field = input_group_textarea_variants("min-h-9 py-2 text-[13px]");
    let share_button = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Default,
        "chat-meeting-share-button h-10! w-full font-semibold",
    );
    html! {
        <AppShell active="chat">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[350px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">"Chat"</h1>
                        <div class="flex items-center gap-0.5">
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{labelled("Search chats")}
                                onclick={toggleChatSearch()}
                            >
                                <span
                                    class={searchOpen ? "contents text-brand-ink" : "contents"}
                                    pp-ref={chatSearchTrigger}
                                >
                                    <Icon name="search" class="size-[18px]" />
                                </span>
                            </Button>
                            <Popover>
                                <PopoverTrigger
                                    name="chat-meet-now"
                                    class=@{icon_button.as_str()}
                                    attrs=@{labelled("Meet now")}
                                >
                                    <Icon name="video" class="size-[18px]" />
                                </PopoverTrigger>
                                <PopoverContent
                                    name="chat-meet-now"
                                    class="chat-meet-now w-[min(380px,calc(100vw-32px))]! p-[22px]!"
                                    ontoggle={meetNowToggled(event.newState)}
                                >
                                    <PopoverHeader class="mb-[18px] pr-8">
                                        <PopoverTitle class="text-[17px] font-semibold">
                                            "Start a meeting now"
                                        </PopoverTitle>
                                    </PopoverHeader>
                                    <form onsubmit={startChatMeeting(event)} novalidate>
                                        <div class="flex flex-col gap-2">
                                            <Label r#for="chat-meeting-name">"Meeting name"</Label>
                                            <input
                                                id="chat-meeting-name"
                                                data-slot="input"
                                                aria-label="Meeting name"
                                                class=@{meeting_name.as_str()}
                                                pp-ref={meetingNameInput}
                                                onfocus={selectAfterFocus(event.currentTarget)}
                                                name="title"
                                                type="text"
                                                maxlength="120"
                                                autocomplete="off"
                                                value={meetingTitle}
                                                oninput={changeMeetingTitle(target.value)}
                                                disabled={meetingBusy}
                                            />
                                        </div>
                                        <p
                                            class="mt-2.5 text-xs text-danger-ink"
                                            role="alert"
                                            hidden={!meetingError}
                                        >
                                            {meetingError}
                                        </p>
                                        <div
                                            class="chat-meeting-share mt-3.5"
                                            hidden={!sharedMeeting}
                                        >
                                            <Input
                                                class="h-10! text-xs"
                                                attrs=@{named("Meeting link").set("readonly", "")}
                                                value={sharedMeeting ? meetingAbsolute(sharedMeeting.url) : ""}
                                            />
                                            <p
                                                class="mt-2 text-xs text-muted-foreground"
                                                role="status"
                                            >
                                                {meetingNotice}
                                            </p>
                                            <a
                                                class="mt-1.5 inline-block text-xs text-link hover:underline"
                                                href={sharedMeeting ? sharedMeeting.url : "/meet"}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                            >
                                                "Open meeting in a new tab"
                                            </a>
                                        </div>
                                        <div class="mt-[22px] flex flex-col gap-2.5">
                                            <Button
                                                attrs=@{plain()}
                                                variant="outline"
                                                class=@{share_button.as_str()}
                                                disabled={meetingBusy || !meetingTitle.trim()}
                                                onclick={getChatMeetingLink()}
                                            >
                                                {meetingBusy ? "Creating…" : "Get a link to share"}
                                            </Button>
                                            <Button
                                                class="chat-meeting-start h-10! w-full font-semibold"
                                                attrs=@{submit()}
                                                disabled={meetingBusy || !meetingTitle.trim()}
                                            >
                                                {meetingBusy ? "Please wait…" : "Start meeting"}
                                            </Button>
                                        </div>
                                    </form>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="absolute right-3 top-3 text-muted-foreground"
                                        attrs=@{close_popover("chat-meet-now", "Close meeting popup")}
                                        disabled={meetingBusy}
                                    >
                                        <Icon name="close" class="size-5" />
                                    </Button>
                                </PopoverContent>
                            </Popover>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{labelled("New chat").set("aria-keyshortcuts", "Control+n")}
                                onclick={openNewChat()}
                                disabled={sending}
                            >
                                <Icon name="compose" class="size-[18px]" />
                            </Button>
                        </div>
                    </div>
                    <div class="chat-list-search px-5 pt-3" hidden={!searchOpen}>
                        <input
                            pp-ref={chatSearchInput}
                            data-slot="input"
                            class=@{list_search.as_str()}
                            type="search"
                            aria-label="Filter by person or chat name"
                            placeholder="Filter by person or chat name"
                            value={chatFilter}
                            oninput={setChatFilter(target.value)}
                            onkeydown={handleSearchKey(event)}
                        />
                    </div>
                    <div class="flex items-center gap-2 px-5 py-3">
                        <Toggle
                            variant="outline"
                            size="sm"
                            class=@{PILL}
                            checked={filters.unread}
                            onchange={setFilter("unread", target.checked)}
                        >
                            "Unread"
                        </Toggle>
                        <Toggle
                            variant="outline"
                            size="sm"
                            class=@{PILL}
                            checked={filters.meetings}
                            onchange={setFilter("meetings", target.checked)}
                        >
                            "Meeting chats"
                        </Toggle>
                        <Toggle
                            variant="outline"
                            size="sm"
                            class=@{PILL}
                            checked={filters.unmuted}
                            onchange={setFilter("unmuted", target.checked)}
                        >
                            "Unmuted"
                        </Toggle>
                    </div>
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-2 pb-4">
                        <div>
                            <Button
                                variant="ghost"
                                class=@{SECTION}
                                attrs=@{bound("aria-expanded", ui_expr!(aria-expanded={favoritesOpen}))}
                                onclick={setFavoritesOpen(open => !open)}
                            >
                                <span class={favoritesOpen ? "flex" : "flex -rotate-90"}>
                                    <Icon name="chevron-down" class="size-3.5" />
                                </span>
                                "Favorites"
                            </Button>
                            <div hidden={!favoritesOpen}>
                                <div
                                    class={selected === "self" ? "chat-self-entry group mt-0.5 flex items-center rounded-md bg-raised pr-1" : "chat-self-entry group mt-0.5 flex items-center rounded-md pr-1 hover:bg-hover"}
                                    hidden={!selfMatches || !selfFavorite}
                                >
                                    <Button
                                        variant="ghost"
                                        class="h-auto! min-w-0 flex-1 justify-start! gap-3 px-3! py-2 text-left font-normal hover:bg-transparent!"
                                        onclick={selectThread("self")}
                                        disabled={sending}
                                    >
                                        <Avatar
                                            initials=@{initials.as_str()}
                                            size="md"
                                            presence="online"
                                        />
                                        <span class="truncate text-[13px] font-semibold text-ink">
                                            @{self_label.as_str()}
                                        </span>
                                    </Button>
                                    <DropdownMenu>
                                        <DropdownMenuTrigger
                                            menu="chat-self-favorite"
                                            class="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring"
                                            attrs=@{labelled("Self chat options")}
                                        >
                                            <Icon name="more" class="size-4" />
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent
                                            menu="chat-self-favorite"
                                            align="start"
                                            class="w-48"
                                            attrs=@{named("Self chat options")}
                                        >
                                            <DropdownMenuItem onclick={openSelfWindow()}>
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.5"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M14 3h7v7M21 3l-11 11M10 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-5"
                                                    />
                                                </svg>
                                                "Open in new window"
                                            </DropdownMenuItem>
                                            <DropdownMenuSeparator />
                                            <DropdownMenuItem
                                                class="self-chat-favorite-action"
                                                onclick={setSelfFavorite(value => !value)}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.5"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z"
                                                    />
                                                    <path d="M3 3l18 18" hidden={!selfFavorite} />
                                                </svg>
                                                {selfFavorite ? "Unfavorite" : "Favorite"}
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onclick={setSelfHidden(true)}>
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.5"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="m3 3 18 18M10.6 10.6a2 2 0 0 0 2.8 2.8M9.9 5.2A11 11 0 0 1 12 5c7 0 10 7 10 7a17 17 0 0 1-3 4M6.2 6.2A18 18 0 0 0 2 12s3 7 10 7a12 12 0 0 0 5.8-1.8"
                                                    />
                                                </svg>
                                                "Hide"
                                            </DropdownMenuItem>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>
                            </div>
                        </div>
                        <div class="mt-3">
                            <Button
                                variant="ghost"
                                class=@{SECTION}
                                attrs=@{bound("aria-expanded", ui_expr!(aria-expanded={chatsOpen}))}
                                onclick={setChatsOpen(open => !open)}
                            >
                                <span class={chatsOpen ? "flex" : "flex -rotate-90"}>
                                    <Icon name="chevron-down" class="size-3.5" />
                                </span>
                                "Chats"
                            </Button>
                            <div hidden={!chatsOpen}>
                                <div
                                    class={selected === "self" ? "chat-self-entry group mt-0.5 flex items-center rounded-md bg-raised pr-1" : "chat-self-entry group mt-0.5 flex items-center rounded-md pr-1 hover:bg-hover"}
                                    hidden={!selfMatches || selfFavorite}
                                >
                                    <Button
                                        variant="ghost"
                                        class="h-auto! min-w-0 flex-1 justify-start! gap-3 px-3! py-2 text-left font-normal hover:bg-transparent!"
                                        onclick={selectThread("self")}
                                        disabled={sending}
                                    >
                                        <Avatar
                                            initials=@{initials.as_str()}
                                            size="md"
                                            presence="online"
                                        />
                                        <span class="truncate text-[13px] font-semibold text-ink">
                                            @{self_label.as_str()}
                                        </span>
                                    </Button>
                                    <DropdownMenu>
                                        <DropdownMenuTrigger
                                            menu="chat-self-regular"
                                            class="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring"
                                            attrs=@{labelled("Self chat options")}
                                        >
                                            <Icon name="more" class="size-4" />
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent
                                            menu="chat-self-regular"
                                            align="start"
                                            class="w-48"
                                            attrs=@{named("Self chat options")}
                                        >
                                            <DropdownMenuItem onclick={openSelfWindow()}>
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.5"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M14 3h7v7M21 3l-11 11M10 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-5"
                                                    />
                                                </svg>
                                                "Open in new window"
                                            </DropdownMenuItem>
                                            <DropdownMenuSeparator />
                                            <DropdownMenuItem
                                                class="self-chat-favorite-action"
                                                onclick={setSelfFavorite(value => !value)}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.5"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z"
                                                    />
                                                    <path d="M3 3l18 18" hidden={!selfFavorite} />
                                                </svg>
                                                {selfFavorite ? "Unfavorite" : "Favorite"}
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onclick={setSelfHidden(true)}>
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.5"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="m3 3 18 18M10.6 10.6a2 2 0 0 0 2.8 2.8M9.9 5.2A11 11 0 0 1 12 5c7 0 10 7 10 7a17 17 0 0 1-3 4M6.2 6.2A18 18 0 0 0 2 12s3 7 10 7a12 12 0 0 0 5.8-1.8"
                                                    />
                                                </svg>
                                                "Hide"
                                            </DropdownMenuItem>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>
                                <div
                                    class={selected === "new" ? "flex items-center rounded-md bg-raised pr-1" : "flex items-center rounded-md pr-1 hover:bg-hover"}
                                    hidden={!newRowMatches}
                                >
                                    <Button
                                        variant="ghost"
                                        class="h-auto! min-w-0 flex-1 justify-start! gap-3 px-3! py-2 text-left font-normal hover:bg-transparent!"
                                        attrs=@{named("New message")}
                                        onclick={openNewChat()}
                                        disabled={sending}
                                    >
                                        <span
                                            class="flex size-8 shrink-0 items-center justify-center rounded-full bg-hover text-muted-foreground"
                                        >
                                            <Icon name="chat" class="size-4" />
                                        </span>
                                        <span class="truncate text-[13px] font-semibold">
                                            "New message"
                                        </span>
                                    </Button>
                                    <DropdownMenu>
                                        <DropdownMenuTrigger
                                            menu="chat-draft"
                                            class=@{icon_button.as_str()}
                                            attrs=@{labelled("New message options")}
                                        >
                                            <Icon name="more" class="size-5" />
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent
                                            menu="chat-draft"
                                            align="start"
                                            class="new-chat-menu w-60"
                                            attrs=@{named("New message options")}
                                        >
                                            <DropdownMenuItem onclick={openDraftWindow()}>
                                                <Icon name="maximize" class="size-4" />
                                                "Open in new window"
                                            </DropdownMenuItem>
                                            <DropdownMenuItem
                                                variant="destructive"
                                                onclick={discardNewChat()}
                                            >
                                                <Icon name="trash" class="size-4" />
                                                "Discard"
                                            </DropdownMenuItem>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>
                                <template pp-for="thread in visibleThreads">
                                    <div
                                        key={thread.id}
                                        class={selected === thread.id ? "chat-thread-row flex items-center rounded-md bg-raised" : "chat-thread-row flex items-center rounded-md"}
                                    >
                                        <Button
                                            variant="ghost"
                                            class=@{ROW_BUTTON}
                                            value={thread.id}
                                            onclick={selectThread(event.currentTarget.value)}
                                            disabled={sending}
                                        >
                                            <UiAvatar>
                                                <AvatarFallback class=@{INITIALS}>
                                                    {initials(thread.label)}
                                                </AvatarFallback>
                                            </UiAvatar>
                                            <span
                                                class="min-w-0 truncate text-[13px] font-semibold"
                                            >
                                                {thread.label}
                                            </span>
                                        </Button>
                                        <div
                                            class="flex shrink-0 gap-0.5 pr-1"
                                            hidden={!settings.preferences.singlePointerControls}
                                        >
                                            <button
                                                type="button"
                                                class="rounded p-1 text-muted-foreground hover:bg-hover hover:text-ink disabled:opacity-30"
                                                aria-label={"Move " + thread.label + " up"}
                                                disabled={sending || visibleThreads[0].id === thread.id}
                                                onclick={moveThread(thread.id, -1)}
                                            >
                                                <Icon name="chevron-up" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                class="rounded p-1 text-muted-foreground hover:bg-hover hover:text-ink disabled:opacity-30"
                                                aria-label={"Move " + thread.label + " down"}
                                                disabled={sending || visibleThreads[visibleThreads.length - 1].id === thread.id}
                                                onclick={moveThread(thread.id, 1)}
                                            >
                                                <Icon name="chevron-down" class="size-4" />
                                            </button>
                                        </div>
                                    </div>
                                </template>
                                <p
                                    class="px-3 py-2 text-xs leading-relaxed text-subtle"
                                    hidden={hasChatFilter || hasNewChat || threads.length > 1 || (!selfFavorite && !selfHidden)}
                                >
                                    "Nobody has written to you yet. Start a chat and it will show up here."
                                </p>
                            </div>
                        </div>
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
                        <UiAvatar>
                            <AvatarFallback class=@{INITIALS}>
                                {initials(currentThread.id === "self" ? selfName : currentThread.label)}
                            </AvatarFallback>
                        </UiAvatar>
                        <h2 class="truncate text-[15px] font-semibold text-ink">
                            {currentThread.label}
                        </h2>
                        <div class="ml-auto flex items-center gap-0.5">
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{labelled("Find in conversation")}
                                onclick={openConversationSearch()}
                            >
                                <Icon name="search" class="size-[18px]" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{labelled("More options")}
                            >
                                <Icon name="more" class="size-[18px]" />
                            </Button>
                        </div>
                    </header>
                    <section
                        class="new-chat-header relative shrink-0 border-b border-line"
                        hidden={!newChat}
                        aria-label="New conversation recipients"
                    >
                        <div
                            class="new-chat-group flex min-h-16 items-center gap-3.5 border-b border-edge px-7 py-3 text-sm focus-within:shadow-[inset_0_-2px_var(--color-brand-ink)]"
                            hidden={!groupNameOpen}
                        >
                            <Label r#for="chat-group-name" class="shrink-0 font-normal">
                                "Group name:"
                            </Label>
                            <input
                                id="chat-group-name"
                                data-slot="input-group-control"
                                class=@{inline_field.as_str()}
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
                            <Label r#for="chat-recipients" class="shrink-0 font-normal">
                                "To:"
                            </Label>
                            <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
                                <template pp-for="recipient in recipients">
                                    <Badge
                                        key={recipient}
                                        variant="secondary"
                                        class="new-chat-chip max-w-full gap-1.5 rounded-[5px] bg-brand-soft py-1 pr-1 text-[13px] font-normal"
                                    >
                                        <span class="truncate">{recipient}</span>
                                        <Button
                                            variant="ghost"
                                            size="icon-xs"
                                            class="size-5! text-muted-foreground"
                                            attrs=@{bound("aria-label", ui_expr!(aria-label={"Remove " + recipient}))}
                                            value={recipient}
                                            onclick={removeRecipient(event.currentTarget.value)}
                                            disabled={sending}
                                        >
                                            <Icon name="close" class="size-3.5" />
                                        </Button>
                                    </Badge>
                                </template>
                                <input
                                    id="chat-recipients"
                                    data-slot="input-group-control"
                                    pp-ref={recipientInput}
                                    class=@{inline_field.as_str()}
                                    aria-label="Recipients"
                                    aria-describedby="chat-recipient-error"
                                    placeholder="Enter name, email or phone number"
                                    autocomplete="off"
                                    value={recipientQuery}
                                    oninput={changeRecipient(target.value)}
                                    onkeydown={handleRecipientKey(event)}
                                    disabled={sending}
                                    aria-invalid={!!recipientError}
                                />
                            </div>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{bound("aria-label", ui_expr!(aria-label={groupNameOpen ? "Hide group name" : "Show group name"}))
                                    .merge(&bound("aria-expanded", ui_expr!(aria-expanded={groupNameOpen})))}
                                onclick={toggleGroupName()}
                                disabled={sending}
                            >
                                <span class={groupNameOpen ? "flex rotate-180" : "flex"}>
                                    <Icon name="chevron-down" class="size-5" />
                                </span>
                            </Button>
                        </div>
                        <p
                            class="new-chat-recipient-error px-7 pb-2 text-xs text-danger-ink"
                            id="chat-recipient-error"
                            role="alert"
                            hidden={!recipientError}
                        >
                            {recipientError}
                        </p>
                        <div
                            class="absolute inset-x-7 top-full z-20 rounded-md border bg-popover p-1 shadow-xl shadow-black/40"
                            hidden={!recipientQuery.trim()}
                        >
                            <Button
                                variant="ghost"
                                class="h-auto! w-full justify-start! gap-2.5 px-3! py-2.5 text-[13px] font-normal"
                                hidden={!suggestSelf}
                                onclick={addRecipient(selfLabel)}
                                disabled={sending}
                            >
                                <Avatar initials=@{initials.as_str()} size="md" presence="none" />
                                <span>@{self_label.as_str()}</span>
                            </Button>
                            <Button
                                variant="ghost"
                                class="h-auto! w-full justify-start! gap-1 px-3! py-2.5 text-[13px] font-normal"
                                onclick={addRecipient(recipientQuery)}
                                disabled={sending}
                            >
                                "Add "
                                <strong>{recipientQuery}</strong>
                            </Button>
                        </div>
                    </section>
                    <div
                        class="chat-conversation-search px-8 py-2"
                        hidden={!conversationSearchOpen}
                    >
                        <input
                            type="search"
                            data-slot="input"
                            class=@{conversation_search.as_str()}
                            aria-label="Find messages in this conversation"
                            placeholder="Find in this conversation"
                            pp-ref={conversationSearchInput}
                            value={conversationQuery}
                            oninput={setConversationQuery(target.value)}
                            onkeydown={handleConversationSearchKey(event)}
                        />
                        <p class="mt-1 text-xs text-muted-foreground" role="status">
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
                            <p class="mt-2.5 text-[13px] leading-relaxed text-muted-foreground">
                                "This chat is just for you…with you. Use it for drafts, send files to yourself, or get to know chat features a little better."
                            </p>
                        </div>
                        <div
                            class="new-chat-empty flex flex-1 flex-col items-center justify-center gap-6 py-7 text-center"
                            hidden={!newChat}
                        >
                            <div class="relative isolate h-[185px] w-[200px]" aria-hidden="true">
                                <span class=@{BUBBLE}></span>
                                <span class=@{PENCIL}></span>
                                <span class=@{DOT_PURPLE}></span>
                                <span class=@{DOT_GREEN}></span>
                                <span class=@{DOT_YELLOW}></span>
                            </div>
                            <h2 class="mt-3.5 text-[17px] font-semibold">
                                "You're starting a new conversation"
                            </h2>
                            <p class="text-sm text-muted-foreground">
                                "Type your first message below."
                            </p>
                        </div>
                        <div class="w-full py-5" hidden={messages.length === 0}>
                            <template pp-for="message in matchingMessages">
                                <div key={message.id} class="flex gap-3 py-2">
                                    <UiAvatar class="mt-0.5">
                                        <AvatarFallback class=@{INITIALS}>
                                            {message.initials}
                                        </AvatarFallback>
                                    </UiAvatar>
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
                                            class="mt-0.5 whitespace-pre-wrap break-words text-[13px] leading-relaxed text-muted-foreground"
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
                                                class="chat-message-gif mt-1.5 block max-h-[280px] max-w-[min(320px,100%)] rounded-lg"
                                                loading="lazy"
                                            />
                                        </template>
                                    </div>
                                </div>
                            </template>
                        </div>
                    </div>
                    <div class="shrink-0 px-8 pb-6 pt-2">
                        <div
                            class="chat-draft-gif relative mb-2 inline-flex rounded-lg border border-edge p-2"
                            hidden={!draftGif}
                        >
                            <template pp-for="gif in draftGif ? [draftGif] : []">
                                <img
                                    key={gif.id}
                                    src={gif.url}
                                    alt={gif.title || "Selected GIF from GIPHY"}
                                    class="max-h-[120px] max-w-[180px] rounded"
                                />
                            </template>
                            <Button
                                variant="secondary"
                                size="icon-xs"
                                class="absolute right-1 top-1 rounded-full!"
                                attrs=@{named("Remove selected GIF")}
                                onclick={setDraftGif(null)}
                                disabled={sending}
                            >
                                <Icon name="close" class="size-4" />
                            </Button>
                        </div>
                        <form onsubmit={send(event)} pp-ref={composerForm} novalidate>
                            <InputGroup class="h-auto! bg-raised px-1 dark:bg-raised">
                                <textarea
                                    name="text"
                                    data-slot="input-group-control"
                                    rows={expandedCompose ? 5 : 1}
                                    class=@{composer_field.as_str()}
                                    placeholder="Type a message"
                                    autocomplete="off"
                                    spellcheck={settings.preferences.spellcheck ? "true" : "false"}
                                    aria-label="Type a message"
                                    pp-ref={composerInput}
                                    onkeydown={handleComposerKey(event)}
                                    value={draft}
                                    oninput={setDraft(target.value)}
                                >
                                </textarea>
                                <InputGroupAddon align="inline-end" class="gap-0.5 self-end py-1">
                                    <Popover>
                                        <PopoverTrigger
                                            name="chat-emoji"
                                            class=@{icon_button.as_str()}
                                            attrs=@{labelled("Emoji")}
                                            onclick={rememberSelection()}
                                        >
                                            <Icon name="emoji" class="size-[18px]" />
                                        </PopoverTrigger>
                                        <PopoverContent
                                            name="chat-emoji"
                                            side="top"
                                            align="end"
                                            class=@{PICKER}
                                            attrs=@{named("Emoji and GIF picker")}
                                            ontoggle={emojiToggled(event.newState)}
                                        >
                                            // The search comes first in the document so the popover
                                            // focuses it on open; the tabs are drawn above it.
                                            <InputGroup
                                                class="chat-emoji-search mx-4 mb-2 mt-4 w-auto! shrink-0"
                                            >
                                                <input
                                                    type="search"
                                                    data-slot="input-group-control"
                                                    class=@{emoji_search.as_str()}
                                                    pp-ref={emojiSearchInput}
                                                    aria-label="Find something fun"
                                                    placeholder="Find something fun"
                                                    value={emojiQuery}
                                                    oninput={searchMedia(target.value)}
                                                />
                                                <InputGroupAddon align="inline-end">
                                                    <Icon name="search" class="size-[18px]" />
                                                </InputGroupAddon>
                                            </InputGroup>
                                            <ToggleGroup
                                                r#type="single"
                                                name="chat-media"
                                                class="chat-emoji-tabs order-first w-full shrink-0 rounded-none! bg-sunken px-3"
                                                attrs=@{named("Media type")}
                                            >
                                                <ToggleGroupItem
                                                    r#type="single"
                                                    name="chat-media"
                                                    value="all"
                                                    pressed
                                                    class=@{MEDIA_TAB}
                                                    checked={emojiTab === "all"}
                                                    onchange={chooseMediaTab(event.target)}
                                                >
                                                    "All"
                                                </ToggleGroupItem>
                                                <ToggleGroupItem
                                                    r#type="single"
                                                    name="chat-media"
                                                    value="emoji"
                                                    class=@{MEDIA_TAB}
                                                    checked={emojiTab === "emoji"}
                                                    onchange={chooseMediaTab(event.target)}
                                                >
                                                    "Emoji"
                                                </ToggleGroupItem>
                                                <ToggleGroupItem
                                                    r#type="single"
                                                    name="chat-media"
                                                    value="gifs"
                                                    class=@{MEDIA_TAB}
                                                    checked={emojiTab === "gifs"}
                                                    onchange={chooseMediaTab(event.target)}
                                                >
                                                    "GIFs"
                                                </ToggleGroupItem>
                                            </ToggleGroup>
                                            <div
                                                class="chat-emoji-content scroll-thin min-h-0 flex-1 overflow-y-auto px-4 pb-4 pt-2"
                                                role="region"
                                                aria-label={emojiTab === "gifs" ? "GIFs" : "Emoji"}
                                            >
                                                <div hidden={emojiTab === "gifs"}>
                                                    <div class=@{HEADING}>
                                                        <h3 class="font-semibold">"Emoji"</h3>
                                                        <Button
                                                            attrs=@{plain()}
                                                            variant="ghost"
                                                            size="xs"
                                                            class="text-muted-foreground"
                                                            hidden={emojiTab !== "all"}
                                                            onclick={setEmojiTab("emoji")}
                                                        >
                                                            "See all"
                                                            <Icon
                                                                name="chevron-right"
                                                                class="size-4"
                                                            />
                                                        </Button>
                                                    </div>
                                                    <div
                                                        class="chat-emoji-categories mb-3 flex flex-wrap gap-[3px]"
                                                        hidden={emojiTab !== "emoji"}
                                                        aria-label="Emoji categories"
                                                    >
                                                        <template
                                                            pp-for="category in emojiCategories"
                                                        >
                                                            <div
                                                                key={category.id}
                                                                class={emojiCategory === category.id ? "rounded bg-brand-soft" : "rounded"}
                                                            >
                                                                <Button
                                                                    attrs=@{plain()}
                                                                    variant="ghost"
                                                                    size="icon-sm"
                                                                    class="text-[19px]"
                                                                    title={category.name}
                                                                    value={category.id}
                                                                    onclick={setEmojiCategory(event.currentTarget.value)}
                                                                >
                                                                    {category.icon}
                                                                    <span class="sr-only">
                                                                        {category.name}
                                                                    </span>
                                                                </Button>
                                                            </div>
                                                        </template>
                                                    </div>
                                                    <div
                                                        class="chat-emoji-grid grid grid-cols-6 gap-1"
                                                        aria-label="Emoji choices"
                                                    >
                                                        <template pp-for="item in shownEmojis">
                                                            <Button
                                                                attrs=@{plain()}
                                                                key={item.symbol}
                                                                variant="ghost"
                                                                class="h-11! px-0! text-[29px] leading-none"
                                                                value={item.symbol}
                                                                onclick={insertEmoji(event.currentTarget.value)}
                                                            >
                                                                {item.symbol}
                                                                <span class="sr-only">
                                                                    {item.name}
                                                                </span>
                                                            </Button>
                                                        </template>
                                                    </div>
                                                    <p
                                                        class=@{EMPTY}
                                                        role="status"
                                                        hidden={shownEmojis.length > 0}
                                                    >
                                                        "No emoji found. Try another search."
                                                    </p>
                                                </div>
                                                <div hidden={emojiTab === "emoji"}>
                                                    <div class=@{HEADING}>
                                                        <h3 class="font-semibold">"GIFs"</h3>
                                                        <Button
                                                            attrs=@{plain()}
                                                            variant="ghost"
                                                            size="xs"
                                                            class="text-muted-foreground"
                                                            hidden={emojiTab !== "all"}
                                                            onclick={setEmojiTab("gifs")}
                                                        >
                                                            "See all"
                                                            <Icon
                                                                name="chevron-right"
                                                                class="size-4"
                                                            />
                                                        </Button>
                                                    </div>
                                                    <p
                                                        class=@{EMPTY}
                                                        role="status"
                                                        hidden={!gifLoading}
                                                    >
                                                        "Loading GIFs…"
                                                    </p>
                                                    <p
                                                        class=@{EMPTY}
                                                        role="status"
                                                        hidden={!gifError}
                                                    >
                                                        {gifError}
                                                    </p>
                                                    <Button
                                                        attrs=@{plain()}
                                                        variant="link"
                                                        size="sm"
                                                        class="chat-gif-retry text-link"
                                                        hidden={!gifError || !giphyKey}
                                                        onclick={setGifRetry(value => value + 1)}
                                                    >
                                                        "Try again"
                                                    </Button>
                                                    <div
                                                        class="chat-gif-grid grid grid-cols-3 gap-1.5"
                                                        aria-label="GIF choices"
                                                        aria-busy={gifLoading}
                                                    >
                                                        <template pp-for="gif in visibleGifs">
                                                            <Button
                                                                attrs=@{plain()}
                                                                key={gif.id}
                                                                variant="ghost"
                                                                class="h-[92px]! overflow-hidden bg-sunken p-0!"
                                                                value={gif.id}
                                                                onclick={chooseGif(event.currentTarget.value)}
                                                            >
                                                                <img
                                                                    src={gif.preview}
                                                                    alt={gif.title || "GIF from GIPHY"}
                                                                    loading="lazy"
                                                                    class="size-full object-cover"
                                                                />
                                                            </Button>
                                                        </template>
                                                    </div>
                                                    <p
                                                        class=@{EMPTY}
                                                        role="status"
                                                        hidden={gifLoading || !!gifError || gifs.length > 0}
                                                    >
                                                        "No GIFs found. Try another search."
                                                    </p>
                                                    <Button
                                                        attrs=@{plain()}
                                                        variant="link"
                                                        size="sm"
                                                        class="chat-gif-more mx-auto mt-2.5 flex text-link"
                                                        hidden={emojiTab !== "gifs" || !gifMore || !!gifError}
                                                        disabled={gifLoading}
                                                        onclick={setGifOffset(gifs.length)}
                                                    >
                                                        "Load more"
                                                    </Button>
                                                </div>
                                            </div>
                                            <footer
                                                class="chat-giphy-attribution flex shrink-0 items-center justify-center gap-2.5 border-t bg-sunken px-3 py-2.5 text-xs"
                                                hidden={emojiTab === "emoji"}
                                            >
                                                <a
                                                    href="https://giphy.com/"
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    aria-label="GIFs powered by GIPHY"
                                                >
                                                    <img
                                                        src="/images/powered-by-giphy.png"
                                                        alt="GIFs powered by GIPHY"
                                                        class="h-auto w-[125px]"
                                                    />
                                                </a>
                                                <a
                                                    href="https://support.giphy.com/hc/en-us/articles/360020027752-GIPHY-API-Terms-of-Service"
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    class="text-link"
                                                >
                                                    "View Terms"
                                                </a>
                                            </footer>
                                            <Button
                                                variant="ghost"
                                                attrs=@{(hidden().merge(&close_popover("chat-emoji", "Close emoji picker"))).merge(&plain())}
                                            >
                                                <span pp-ref={emojiCloser}></span>
                                            </Button>
                                        </PopoverContent>
                                    </Popover>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="text-muted-foreground"
                                        attrs=@{(labelled("Attach a file")).merge(&plain())}
                                    >
                                        <Icon name="attach" class="size-[18px]" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="text-muted-foreground"
                                        attrs=@{(labelled("Send an image")).merge(&plain())}
                                    >
                                        <Icon name="image" class="size-[18px]" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="text-muted-foreground"
                                        attrs=@{(labelled("Format")).merge(&plain())}
                                        onclick={toggleCompose()}
                                    >
                                        <span
                                            class={expandedCompose ? "contents text-brand-ink" : "contents"}
                                        >
                                            <Icon name="format" class="size-[18px]" />
                                        </span>
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="text-muted-foreground"
                                        attrs=@{(labelled("More actions")).merge(&plain())}
                                    >
                                        <Icon name="plus" class="size-[18px]" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="text-brand-ink"
                                        attrs=@{labelled("Send").merge(&submit())}
                                        disabled={(!draft.trim() && !draftGif) || sending || (newChat && recipients.length === 0)}
                                    >
                                        <Icon name="send" class="size-[18px]" />
                                    </Button>
                                </InputGroupAddon>
                            </InputGroup>
                        </form>
                        <p class="mt-2 text-xs text-subtle" hidden={selected === "self"}>
                            "New conversations are previews. Messages aren't delivered to recipients yet."
                        </p>
                        <p class="mt-2 text-xs text-busy" hidden={!error}>{error}</p>
                    </div>
                </main>
                <script>r##"
                    const shortcuts = pp.context(window.UpurnakaShortcutContext);
                    const settings = pp.context(window.UpurnakaSettingsContext);
                    const composerInput = pp.ref(null);
                    const emojiCloser = pp.ref(null);
                    const emojiSearchInput = pp.ref(null);
                    const emojiSelection = pp.ref({start:0, end:0});
                    const emojiCaret = pp.ref(null);
                    const [emojiOpen, setEmojiOpen] = pp.state(false);
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
                    const [selfFavorite, setSelfFavorite] = pp.state(true);
                    const [selfHidden, setSelfHidden] = pp.state(false);
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
                    const recipientInput = pp.ref(null);
                    const groupNameInput = pp.ref(null);
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
                    const meetingNameInput = pp.ref(null);
                    const hasChatFilter = !!chatFilter.trim() || filters.unread || filters.meetings || filters.unmuted;
                    const selfMatches = (!selfHidden || !!chatFilter.trim()) && selfLabel.toLocaleLowerCase().includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings;

                    const newRowMatches = hasNewChat && "new message".includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings;
                    const visibleThreads = threads.filter(thread => thread.id !== "self" && thread.label.toLocaleLowerCase().includes(chatFilter.trim().toLocaleLowerCase()) && !filters.unread && !filters.meetings);
                    function moveThread(id, direction) {
                        const index = visibleThreads.findIndex(thread => thread.id === id);
                        const neighbor = visibleThreads[index + direction];
                        if (!neighbor) return;
                        setThreads(previous => {
                            const next = previous.slice();
                            const from = next.findIndex(thread => thread.id === id);
                            const to = next.findIndex(thread => thread.id === neighbor.id);
                            [next[from], next[to]] = [next[to], next[from]];
                            return next;
                        });
                    }
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
                    const chooseGif = pp.event(function chooseGif(id) {
                        const gif = gifs.find(item => item.id === id);
                        if (sending || !gif) return;
                        setDraftGif({id:gif.id, url:gif.url, title:gif.title});
                        closeEmoji(); returnToComposer();
                    });
                    // The picker is rahti-ui's Popover, which hands focus back to
                    // its trigger as it closes; the composer takes it after that.
                    function closeEmoji() {if (emojiOpen) emojiCloser.current?.click();}
                    function returnToComposer() {setTimeout(() => composerInput.current?.focus(), 0);}
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
                    // The caret is read as the trigger is pressed, before the picker
                    // takes focus away from the composer.
                    function rememberSelection() {
                        const input = composerInput.current;
                        if (!input || emojiOpen) return;
                        emojiSelection.current = {start:input.selectionStart ?? draft.length, end:input.selectionEnd ?? draft.length};
                    }
                    // A tab chosen with the pointer takes focus too, so the arrow
                    // keys carry on from it rather than from wherever focus was.
                    function chooseMediaTab(tab) {
                        setEmojiTab(tab.value);
                        tab.focus();
                    }
                    function emojiToggled(state) {
                        setEmojiOpen(state === 'open');
                    }
                    const insertEmoji = pp.event(function insertEmoji(symbol) {
                        if (sending) return;
                        const {start,end} = emojiSelection.current;
                        emojiCaret.current = start + symbol.length;
                        setDraft(draft.slice(0,start) + symbol + draft.slice(end));
                        closeEmoji(); returnToComposer();
                    });
                    pp.layoutEffect(() => {
                        if (emojiCaret.current === null) return;
                        composerInput.current?.setSelectionRange(emojiCaret.current, emojiCaret.current);
                        emojiCaret.current = null;
                    }, [draft]);
                    const selectThread = pp.event(function selectThread(id) {
                        if (sending) return;
                        if (id === 'self') setSelfHidden(false);
                        closeEmoji();
                        setSelected(id); setConversationQuery(""); setConversationSearchOpen(false); setError("");
                    });
                    function openNewChat() {
                        if (sending) return;
                        closeEmoji();
                        setHasNewChat(true); setSelected("new"); setChatsOpen(true);
                        setChatFilter(""); setFilters({unread:false, meetings:false, unmuted:false});
                        setConversationQuery(""); setConversationSearchOpen(false); setError("");
                        recipientInput.current?.focus();
                    }
                    function discardNewChat() {
                        if (sending) return;
                        setHasNewChat(false); setNewDraft(''); setNewGif(null); setRecipients([]); setRecipientQuery('');
                        setRecipientError(''); setGroupName(''); setGroupNameOpen(false);
                        if (newChat) selectThread('self');
                        setError(''); composerInput.current?.focus();
                    }
                    function openSelfWindow() {
                        window.open('/chat', '_blank', 'noopener,noreferrer,popup,width=1000,height=760');
                    }
                    function openDraftWindow() {
                        if (sending) return;
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
                    const removeRecipient = pp.event(function removeRecipient(value) {setRecipients(previous => previous.filter(item => item !== value)); recipientInput.current?.focus();});
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
                        if (event.key === "Enter" && !event.shiftKey && !event.altKey && ((!expandedCompose && settings.preferences.enterToSend && !event.ctrlKey && !event.metaKey) || event.ctrlKey || event.metaKey)) {
                            event.preventDefault(); composerForm.current?.requestSubmit();
                        }
                    }
                    pp.effect(() => {
                        if (searchOpen && chatSearchInput.current) chatSearchInput.current.focus();
                    }, [searchOpen]);

                    function toggleChatSearch() {
                        setSearchOpen(!searchOpen);
                        if (searchOpen) setChatFilter("");
                    }
                    function handleSearchKey(event) {
                        if (event.key === "Escape") {
                            setChatFilter(""); setSearchOpen(false);
                            chatSearchTrigger.current?.closest("button")?.focus();
                        }
                    }
                    // When a create finishes with the popover still open, the name
                    // field takes focus back, ready for another try.
                    const meetingWasBusy = pp.ref(false);
                    pp.effect(() => {
                        if (meetingWasBusy.current && !meetingBusy && meetingOpen) {
                            meetingNameInput.current?.focus();
                            meetingNameInput.current?.select();
                        }
                        meetingWasBusy.current = meetingBusy;
                    }, [meetingBusy]);
                    function meetNowToggled(state) {
                        setMeetingOpen(state === "open");
                        if (state === "open") setMeetingError("");
                    }
                    // The popover puts the caret after the suggested name as it
                    // focuses the field; selecting once that has run lets typing
                    // replace the whole name instead.
                    function selectAfterFocus(field) {
                        queueMicrotask(() => field.select());
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
                            const created = await ensureChatMeeting(meetingTitle);
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

                    function setFilter(name, on) {
                        setFilters((current) => ({ ...current, [name]: on }));
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
