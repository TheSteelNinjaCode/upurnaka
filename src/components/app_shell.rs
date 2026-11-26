//! The desktop window: title bar, activity rail, and the frame every section
//! renders inside.
//!
//! A layout cannot see the request, so the rail's selected item is a prop:
//! each `page.rs` roots itself in `<AppShell active="…">` and names its own
//! section. The page's content is the shell's `children`, and lands at the
//! `<slot />` beside the rail.

use serde::Serialize;

use crate::components::aria::{hidden, labelled, named};
use crate::components::avatar::ProfileAvatar;
use crate::components::icon::Icon;
use crate::components::keyboard_shortcuts::KeyboardShortcuts;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
use crate::components::rahti_ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator,
    DropdownMenuTrigger, SHORTCUT,
};
use crate::components::rahti_ui::input_group::{
    InputGroup, InputGroupAddon, input_group_input_variants,
};
use crate::components::rahti_ui::popover::{Popover, PopoverContent, PopoverTrigger};
use crate::components::user_preferences::UserPreferences;
use crate::rahti::{Html, Result, Validator, component, html, rpc};

/// Every destination on the rail, in the order the client shows them.
const RAIL: [(&str, &str, &str); 7] = [
    ("chat", "Chat", "/chat"),
    ("meet", "Meet", "/meet"),
    ("people", "People", "/people"),
    ("communities", "Communities", "/communities"),
    ("calendar", "Calendar", "/calendar"),
    ("activity", "Activity", "/activity"),
    ("files", "Files", "/files"),
];

#[rpc(auth)]
pub async fn sign_out_account() -> bool {
    rahti::auth::sign_out();
    true
}

#[derive(Serialize)]
pub struct SearchPerson {
    pub id: String,
    pub title: String,
    pub initials: String,
    pub is_self: bool,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub people: Vec<SearchPerson>,
    pub files: Vec<crate::chat_files::Attachment>,
}

#[derive(Serialize)]
pub struct MessageSearchDates {
    pub preset: String,
    pub label: String,
    pub from: String,
    pub to: String,
    pub start: i64,
    pub end: i64,
}

/// Interpret calendar dates in the configured application timezone, including DST.
#[rpc(auth)]
pub fn message_search_dates(
    preset: String,
    from: Option<String>,
    to: Option<String>,
) -> Result<MessageSearchDates> {
    use crate::rahti::time;
    let mut validator = Validator::new();
    validator.one_of(
        "preset",
        &preset.as_str(),
        &["today", "yesterday", "week", "month", "custom"],
    );
    validator.finish()?;
    let today = time::today();
    let invalid = || {
        crate::rahti::Error::new(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Choose valid From and To dates, with From on or before To.",
        )
    };
    let (first, last, label) = match preset.as_str() {
        "today" => (today, today, "Today".to_string()),
        "yesterday" => {
            let day = today.yesterday().map_err(|_| invalid())?;
            (day, day, "Yesterday".into())
        }
        "week" => (
            today
                .checked_sub(
                    jiff::Span::new().days(i64::from(today.weekday().to_monday_zero_offset())),
                )
                .map_err(|_| invalid())?,
            today,
            "This week".into(),
        ),
        "month" => (today.first_of_month(), today, "This month".into()),
        _ => {
            let first = time::Date::strptime("%Y-%m-%d", from.as_deref().unwrap_or(""))
                .map_err(|_| invalid())?;
            let last = time::Date::strptime("%Y-%m-%d", to.as_deref().unwrap_or(""))
                .map_err(|_| invalid())?;
            if first > last {
                return Err(invalid());
            }
            (
                first,
                last,
                format!(
                    "{} – {}",
                    first.strftime("%b %-d, %Y"),
                    last.strftime("%b %-d, %Y")
                ),
            )
        }
    };
    let (start, _) = time::day_bounds_utc(first).map_err(|_| invalid())?;
    let (_, end) = time::day_bounds_utc(last).map_err(|_| invalid())?;
    Ok(MessageSearchDates {
        preset,
        label,
        from: first.to_string(),
        to: last.to_string(),
        start: start.as_second(),
        end: end.as_second(),
    })
}

/// Search public directory names and the caller's own uploaded files only.
#[rpc(auth)]
pub async fn search_workspace(
    query: String,
    file_type: Option<String>,
    file_date: Option<String>,
    file_from: Option<String>,
    file_to: Option<String>,
) -> Result<SearchResponse> {
    use crate::models::{account, chat_file};
    use sea_orm::{
        ColumnTrait, Condition, EntityTrait, ExprTrait, QueryFilter, QueryOrder, QuerySelect,
        sea_query::{Expr, Func, LikeExpr},
    };
    let query = query.trim().to_lowercase();
    let file_type = file_type.unwrap_or_else(|| "all".into());
    let file_date = file_date.unwrap_or_else(|| "any".into());
    let mut validator = Validator::new();
    validator
        .required("query", &query)
        .max_chars("query", &query, 120)
        .one_of(
            "file_type",
            &file_type.as_str(),
            &[
                "all",
                "images",
                "pdf",
                "word",
                "excel",
                "powerpoint",
                "text",
                "loop",
                "onenote",
            ],
        )
        .one_of(
            "file_date",
            &file_date.as_str(),
            &[
                "any",
                "day",
                "today",
                "yesterday",
                "week",
                "month",
                "custom",
            ],
        );
    validator.finish()?;
    crate::auth::ready()?;
    rahti::authorization::authorize("directory.search")?;
    let user = crate::auth::current().ok_or_else(|| {
        crate::rahti::Error::new(
            axum::http::StatusCode::UNAUTHORIZED,
            "Authentication required",
        )
    })?;
    let pattern = format!(
        "%{}%",
        query
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_")
    );
    let people = account::Entity::find()
        .filter(account::Column::Active.eq(true))
        .filter(account::Column::Verified.eq(true))
        .filter(
            Expr::expr(Func::lower(Expr::col(account::Column::Name)))
                .like(LikeExpr::new(&pattern).escape('!')),
        )
        .order_by_asc(account::Column::Name)
        .limit(50)
        .select_only()
        .column(account::Column::Id)
        .column(account::Column::Name)
        .into_tuple::<(String, String)>()
        .all(crate::db::db())
        .await?
        .into_iter()
        .map(|(id, name)| {
            let initials = name
                .split_whitespace()
                .take(2)
                .filter_map(|part| part.chars().next())
                .collect::<String>()
                .to_uppercase();
            SearchPerson {
                is_self: id == user.id,
                id,
                title: name,
                initials,
            }
        })
        .collect();
    let mut file_query = chat_file::Entity::find()
        .filter(chat_file::Column::OwnerId.eq(user.id))
        .filter(
            Expr::expr(Func::lower(Expr::col(chat_file::Column::Name)))
                .like(LikeExpr::new(&pattern).escape('!')),
        );
    if file_type != "all" {
        let extensions: &[&str] = match file_type.as_str() {
            "images" => &["png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp"],
            "pdf" => &["pdf"],
            "word" => &["doc", "docx", "odt", "rtf"],
            "excel" => &["xls", "xlsx", "ods", "csv"],
            "powerpoint" => &["ppt", "pptx", "odp"],
            "loop" => &["loop", "fluid"],
            "onenote" => &["one", "onepkg", "onetoc2"],
            _ => &["txt", "md", "log"],
        };
        let mut types = Condition::any();
        for extension in extensions {
            types = types.add(
                Expr::expr(Func::lower(Expr::col(chat_file::Column::Name)))
                    .like(format!("%.{extension}")),
            );
        }
        if file_type == "images" {
            types = types.add(chat_file::Column::ContentType.starts_with("image/"));
        }
        if file_type == "pdf" {
            types = types.add(chat_file::Column::ContentType.eq("application/pdf"));
        }
        file_query = file_query.filter(types);
    }
    if file_date == "day" {
        file_query = file_query.filter(
            chat_file::Column::CreatedAt.gte(rahti::time::Timestamp::now().as_second() - 86400),
        );
    } else if file_date != "any" {
        let range = message_search_dates(file_date, file_from, file_to)?;
        file_query = file_query
            .filter(chat_file::Column::CreatedAt.gte(range.start))
            .filter(chat_file::Column::CreatedAt.lt(range.end));
    }
    let files = file_query
        .order_by_desc(chat_file::Column::CreatedAt)
        .limit(50)
        .all(crate::db::db())
        .await?
        .into_iter()
        .map(crate::chat_files::Attachment::from)
        .collect();
    Ok(SearchResponse { people, files })
}

/// Which icon each rail destination draws.
fn rail_icon(key: &str) -> &'static str {
    match key {
        "meet" => "video",
        "people" => "contacts",
        "activity" => "bell",
        "communities" => "communities",
        "calendar" => "calendar",
        "files" => "folder",
        _ => "chat",
    }
}

/// One square on the activity rail.
#[component]
pub fn RailLink(icon: &str, label: &str, href: &str, selected: bool) -> Html {
    let class = if selected {
        "group relative flex h-11 w-11 items-center justify-center rounded-md bg-raised text-brand-ink"
    } else {
        "group relative flex h-11 w-11 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-hover hover:text-ink"
    };

    let current = if selected { Some("page") } else { None };

    html! {
        <a href=@{href} class=@{class} title=@{label} aria-label=@{label} aria-current=@{current}>
            <span class="absolute left-0 h-5 w-0.5 rounded-r bg-brand-ink" hidden=@{!selected}>
            </span>
            <Icon name=@{icon} class="h-[22px] w-[22px]" />
        </a>
    }
}

//// The window frame. `active` is the rail key of the section being shown.
#[component]
pub fn AppShell(active: &str, children: Html) -> Html {
    let user = crate::auth::current().unwrap_or_default();
    let mut links: Vec<Html> = RAIL
        .iter()
        .map(|(key, label, href)| RailLink(rail_icon(key), label, href, *key == active))
        .collect();
    if user.role == crate::auth::Role::Admin {
        links.push(RailLink("contacts", "Users", "/users", active == "users"));
    }
    let rail = Html::concat(links);

    let apps_class = if active == "apps" {
        "flex h-11 w-11 items-center justify-center rounded-md bg-raised text-brand-ink"
    } else {
        "flex h-11 w-11 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-hover hover:text-ink"
    };
    let close_survey = button_variants(ButtonVariant::Outline, ButtonSize::Sm, "w-24");
    // Written as an element rather than an `<InputGroupInput>` because Ctrl+G
    // focuses it, and a ref is a binding a component tag cannot carry.
    let search_input = input_group_input_variants("text-[13px]");
    let more_trigger = button_variants(
        ButtonVariant::Ghost,
        ButtonSize::IconSm,
        "relative text-muted-foreground",
    );

    html! {
        <div class="ambient relative flex h-screen w-full flex-col overflow-hidden text-ink">
            <header class="relative z-40 flex h-12 shrink-0 items-center gap-1 px-2.5">
                <a
                    href="/chat"
                    class="ml-2 mr-1 flex size-8 items-center justify-center rounded-md hover:bg-hover"
                    title="Teams"
                    aria-label="Teams"
                >
                    <img src="/images/upurnaka-logo.png" alt="" class="size-[22px]" />
                </a>
                <Button
                    variant="ghost"
                    size="icon-sm"
                    class="text-muted-foreground"
                    attrs=@{labelled("Expand or collapse the list pane")}
                    onclick={togglePane()}
                >
                    <Icon name="panel" class="size-[18px]" />
                </Button>
                <div class="flex min-w-0 flex-1 items-center justify-center gap-1">
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        class="text-subtle"
                        attrs=@{labelled("Back")}
                        onclick={goBack()}
                    >
                        <Icon name="chevron-left" class="size-4" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        class="text-subtle"
                        attrs=@{labelled("Forward")}
                        onclick={goForward()}
                    >
                        <Icon name="chevron-right" class="size-4" />
                    </Button>
                    <div
                        class="relative ml-2 w-full max-w-[720px]"
                        onfocusout={globalSearchBlur(event)}
                    >
                        <InputGroup
                            class="global-search-input h-8! border-edge bg-popover shadow-none dark:bg-popover"
                        >
                            <InputGroupAddon>
                                <Icon name="search" class="size-4" />
                            </InputGroupAddon>
                            <input
                                type="text"
                                data-slot="input-group-control"
                                class=@{search_input.as_str()}
                                placeholder="Press Ctrl+G to go right to a chat or channel"
                                aria-label="Search"
                                aria-expanded={suggestionsOpen && !!query.trim() && !searchView}
                                aria-controls="global-search-suggestions"
                                autocomplete="off"
                                maxlength="120"
                                pp-ref={globalSearch}
                                value={query}
                                oninput={changeGlobalQuery(target.value)}
                                onfocus={setSuggestionsOpen(true)}
                                onkeydown={handleGlobalSearchKey(event)}
                            />
                            <InputGroupAddon align="inline-end">
                                <button
                                    type="button"
                                    class="flex size-6 items-center justify-center rounded text-muted-foreground hover:text-ink focus-visible:outline-2 focus-visible:outline-ring"
                                    aria-label="Clear search"
                                    hidden={!query && !searchView}
                                    onclick={clearGlobalSearch()}
                                >
                                    <Icon name="close" class="size-4" />
                                </button>
                            </InputGroupAddon>
                        </InputGroup>
                        <div
                            id="global-search-suggestions"
                            class="absolute left-0 right-0 top-10 rounded-lg border bg-popover p-1 text-popover-foreground shadow-xl"
                            hidden={!suggestionsOpen || !query.trim() || searchView}
                            aria-label="Search suggestions"
                        >
                            <button
                                type="button"
                                class="flex w-full items-center gap-3 rounded px-3 py-2.5 text-left text-[13px] hover:bg-hover focus-visible:outline-2 focus-visible:outline-ring"
                                onclick={submitGlobalSearch()}
                                pp-ref={searchSubmit}
                            >
                                <Icon name="search" class="size-4" />
                                <span>"Search for “"{query.trim()}"”"</span>
                            </button>
                            <template pp-for="result in searchResults">
                                <Button
                                    key={result.href}
                                    variant="ghost"
                                    size="sm"
                                    class="w-full justify-start! font-normal"
                                    value={result.href}
                                    onclick={visitSearchResult(event.currentTarget.value)}
                                >
                                    {result.label}
                                </Button>
                            </template>
                        </div>
                    </div>
                </div>
                <div class="relative flex items-center gap-1">
                    <DropdownMenu>
                        <DropdownMenuTrigger
                            menu="shell-more"
                            class=@{more_trigger.as_str()}
                            attrs=@{labelled("Settings and more")}
                        >
                            <Icon name="more" class="size-[18px]" />
                            <span
                                class="absolute right-1.5 top-1 size-1.5 rounded-full bg-brand-ink"
                                pp-ref={moreBadge}
                            >
                            </span>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent menu="shell-more" align="end" class="w-72">
                            <DropdownMenuItem href="/settings">
                                <Icon name="settings" class="size-4 text-muted-foreground" />
                                "Settings"
                                <span class=@{SHORTCUT} aria-hidden="true">"Ctrl+,"</span>
                            </DropdownMenuItem>
                            <DropdownMenuItem href="/settings?category=notifications">
                                <Icon name="bell" class="size-4 text-muted-foreground" />
                                "Notifications"
                            </DropdownMenuItem>
                            <DropdownMenuItem
                                attrs=@{named("Keyboard shortcuts").set("aria-keyshortcuts", "Control+.")}
                                onclick={openShortcuts()}
                            >
                                <Icon name="format" class="size-4 text-muted-foreground" />
                                "Keyboard shortcuts"
                                <span class=@{SHORTCUT} aria-hidden="true">"Ctrl+."</span>
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
                                <Icon name="check-circle" class="size-4 text-muted-foreground" />
                                "About"
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                    <DropdownMenu>
                        <DropdownMenuTrigger
                            menu="shell-profile"
                            class="relative ml-1 flex size-8 items-center justify-center rounded-full outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
                            attrs=@{named("Your profile").set("title", user.name.as_str())}
                        >
                            <ProfileAvatar size="md" presence="none" />
                            <span
                                class={"absolute -bottom-0.5 -right-0.5 size-2.5 rounded-full border-2 border-chrome " + PRESENCE_COLORS[presence]}
                            >
                            </span>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent menu="shell-profile" align="end" class="w-80">
                            <div class="flex items-center gap-3 px-2 py-2">
                                <ProfileAvatar size="lg" presence="none" />
                                <span class="min-w-0">
                                    <span class="block truncate text-[13px] font-semibold text-ink">
                                        @{user.name.as_str()}
                                    </span>
                                    <span class="block truncate text-xs text-muted-foreground">
                                        @{user.email.as_str()}
                                    </span>
                                </span>
                            </div>
                            <DropdownMenuSeparator />
                            <DropdownMenuLabel
                                class="text-[11px] font-semibold uppercase tracking-wide text-subtle"
                            >
                                "Presence"
                            </DropdownMenuLabel>
                            <template pp-for="choice in PRESENCE">
                                <DropdownMenuItem
                                    key={choice.key}
                                    value={choice.key}
                                    onclick={pickPresence(event.currentTarget.getAttribute("value"))}
                                >
                                    <span class={"size-2.5 rounded-full " + choice.dot}></span>
                                    {choice.label}
                                    <span class="ml-auto" hidden={presence !== choice.key}>
                                        <Icon name="check" class="size-4 text-brand-ink" />
                                    </span>
                                </DropdownMenuItem>
                            </template>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
                                <Icon name="person-add" class="size-4 text-muted-foreground" />
                                "Set a status message"
                            </DropdownMenuItem>
                            <DropdownMenuItem href="/account">"My account"</DropdownMenuItem>
                            <DropdownMenuItem variant="destructive" onclick={signOut()}>
                                "Sign out"
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
                <p
                    role="alert"
                    hidden={!signoutError}
                    class="absolute right-3 top-12 rounded-md border bg-popover px-3 py-2 text-xs text-danger-ink shadow-xl"
                >
                    {signoutError}
                </p>
            </header>
            <div
                class={paneOpen ? "shell-body isolate flex min-h-0 flex-1" : "shell-body isolate flex min-h-0 flex-1 panes-collapsed"}
            >
                <nav
                    class="flex w-[68px] shrink-0 flex-col items-center gap-1 pb-3"
                    aria-label="Teams sections"
                    pp-ref={appRail}
                    tabindex="-1"
                    hidden={!appBarOpen}
                >
                    @{rail}
                    <span class="flex-1"></span>
                    <a href="/apps" class=@{apps_class} title="Apps" aria-label="Apps">
                        <Icon name="apps" class="h-[22px] w-[22px]" />
                    </a>
                </nav>
                <div class="shell-page flex min-h-0 min-w-0 flex-1" hidden={searchView}>
                    <shortcutcontext.provider value={pageShortcuts}>
                        <searchcontext.provider value={searchStore}>
                            <UserPreferences user_id=@{user.id.as_str()}>
                                <slot />
                            </UserPreferences>
                        </searchcontext.provider>
                    </shortcutcontext.provider>
                </div>
                <section
                    class="global-search-view flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-tl-xl border-l border-t border-line bg-popover [&_[role=tablist]]:overflow-x-auto [&_[role=tablist]]:p-0.5"
                    hidden={!searchView}
                    aria-label="Search results"
                >
                    <header
                        class="global-search-header flex h-[60px] shrink-0 items-center border-b border-line px-5"
                    >
                        <div
                            class="mx-auto flex w-full max-w-[760px] gap-3"
                            role="tablist"
                            aria-label="Search categories"
                        >
                            <template pp-for="tab in searchTabs">
                                <button
                                    type="button"
                                    key={tab.id}
                                    role="tab"
                                    id={"global-search-tab-" + tab.id}
                                    aria-selected={searchTab === tab.id}
                                    aria-controls="global-search-panel"
                                    tabindex={searchTab === tab.id ? 0 : -1}
                                    pp-ref={element => searchTabRefs.current[tab.id] = element}
                                    class="global-search-tab h-9 shrink-0 rounded-full border-2 border-transparent bg-sunken px-3.5 py-0 text-[13px] leading-5 text-ink font-semibold hover:bg-hover aria-selected:border-brand-ink aria-selected:bg-brand-soft aria-selected:text-brand-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
                                    onclick={setSearchTab(tab.id)}
                                    onkeydown={searchTabKey(event)}
                                >
                                    {tab.label}
                                </button>
                            </template>
                        </div>
                    </header>
                    <div
                        class="global-search-filters shrink-0 border-b border-edge px-5 [&>div]:overflow-x-auto"
                        hidden={searchTab !== "messages" && searchTab !== "files"}
                        aria-label="Search filters"
                    >
                        <div
                            class="mx-auto flex h-10 w-full max-w-[760px] items-center gap-2"
                            hidden={searchTab !== "messages"}
                        >
                            <Popover>
                                <PopoverTrigger
                                    name="search-from"
                                    class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-muted-foreground"
                                    attrs=@{labelled("Filter messages by sender")}
                                >
                                    <span>
                                        {messageFilters.from === "all" ? "From" : "From: " + senderLabel(messageFilters.from)}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </PopoverTrigger>
                                <PopoverContent
                                    name="search-from"
                                    align="start"
                                    class="message-from-picker bg-popover text-ink [&_.message-filter-input:focus-visible]:border-transparent [&_.message-filter-input:focus-visible]:shadow-[inset_0_-2px_var(--color-brand-ink)] w-[350px] max-w-[calc(100vw-24px)] p-3"
                                    ontoggle={searchFilterToggled("from", event.newState)}
                                >
                                    <input
                                        type="search"
                                        class="message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink w-full"
                                        placeholder="Enter name, email or phone number"
                                        aria-label="Find message sender"
                                        value={senderQuery}
                                        oninput={setSenderQuery(target.value)}
                                        autocomplete="off"
                                    />
                                    <div
                                        class="mt-2 max-h-52 overflow-y-auto"
                                        hidden={!senderQuery.trim()}
                                    >
                                        <template pp-for="sender in filteredMessageSenders">
                                            <button
                                                type="button"
                                                key={sender.id}
                                                class="message-sender-option flex w-full items-center gap-2 rounded px-2 py-2 text-left text-[13px] hover:bg-hover"
                                                onclick={chooseMessageSender(sender.id)}
                                            >
                                                <span class="flex-1">{sender.label}</span>
                                                <span hidden={messageFilters.from !== sender.id}>
                                                    <Icon name="check" class="size-4" />
                                                </span>
                                            </button>
                                        </template>
                                        <p
                                            class="px-2 py-2 text-xs text-muted-foreground"
                                            hidden={filteredMessageSenders.length > 0}
                                        >
                                            "No matching senders."
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        class="mt-2 rounded px-2 py-1 text-[13px] text-link hover:bg-hover"
                                        hidden={messageFilters.from === "all"}
                                        onclick={chooseMessageSender("all")}
                                    >
                                        "Clear sender"
                                    </button>
                                    <button
                                        type="button"
                                        hidden
                                        popovertarget="popover-search-from"
                                        popovertargetaction="hide"
                                        pp-ref={messageFromCloser}
                                    >
                                    </button>
                                </PopoverContent>
                            </Popover>
                            <Popover>
                                <PopoverTrigger
                                    name="search-message-date"
                                    class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-muted-foreground"
                                    attrs=@{labelled("Filter messages by date")}
                                    onclick={prepareMessageDates()}
                                >
                                    <span>{messageFilters.dateLabel}</span>
                                    <Icon name="chevron-down" class="size-3" />
                                </PopoverTrigger>
                                <PopoverContent
                                    name="search-message-date"
                                    align="start"
                                    class="message-date-picker rounded-[3px] border-edge bg-popover text-ink w-[300px] max-w-[calc(100vw-24px)] p-0"
                                    ontoggle={searchFilterToggled("message-date", event.newState)}
                                >
                                    <div class="px-4 pb-3 pt-4">
                                        <h2 class="mb-1 text-[13px] font-semibold">"Date"</h2>
                                        <template pp-for="option in messageDatePresets">
                                            <button
                                                type="button"
                                                key={option.id}
                                                class="message-date-preset block h-8 w-full rounded text-left text-[13px] hover:bg-hover"
                                                onclick={applyMessageDates(option.id)}
                                                disabled={messageDateBusy}
                                            >
                                                {option.label}
                                            </button>
                                        </template>
                                    </div>
                                    <div class="space-y-3 border-t border-line px-4 pb-3 pt-4">
                                        <label
                                            class="message-date-field group/date relative block text-xs"
                                        >
                                            "From"
                                            <input
                                                type="date"
                                                class={messageDateFrom ? "message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink mt-1 w-full" : "message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink message-date-empty [&:not(:focus)::-webkit-datetime-edit]:text-transparent mt-1 w-full"}
                                                aria-label="Message date from"
                                                value={messageDateFrom}
                                                oninput={changeMessageDateFrom(event)}
                                                onchange={changeMessageDateFrom(event)}
                                                max={messageDateTo || null}
                                            />
                                            <span
                                                class="message-date-placeholder group-focus-within/date:hidden pointer-events-none absolute bottom-2 left-2 text-[13px]"
                                                hidden={!!messageDateFrom}
                                            >
                                                "Select a Date"
                                            </span>
                                        </label>
                                        <label class="block text-xs">
                                            "To"
                                            <input
                                                type="date"
                                                class="message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink mt-1 w-full"
                                                aria-label="Message date to"
                                                value={messageDateTo}
                                                oninput={changeMessageDateTo(event)}
                                                onchange={changeMessageDateTo(event)}
                                                min={messageDateFrom || null}
                                            />
                                        </label>
                                        <p
                                            class="text-xs text-danger-ink"
                                            role="alert"
                                            hidden={!messageDateError}
                                        >
                                            {messageDateError}
                                        </p>
                                        <div class="flex justify-end gap-3">
                                            <button
                                                type="button"
                                                class="rounded px-3 py-1.5 text-[13px] hover:bg-hover disabled:opacity-40"
                                                aria-label="Clear message date"
                                                onclick={clearMessageDates()}
                                                disabled={messageFilters.date === "any" && !messageDateFrom}
                                            >
                                                "Clear"
                                            </button>
                                            <button
                                                type="button"
                                                class="rounded bg-sunken px-6 py-1.5 text-[13px] font-semibold disabled:opacity-40"
                                                aria-label="Apply message date"
                                                onclick={applyMessageDates("custom")}
                                                disabled={!messageDateRangeValid || messageDateBusy}
                                            >
                                                "Apply"
                                            </button>
                                        </div>
                                    </div>
                                    <button
                                        type="button"
                                        hidden
                                        popovertarget="popover-search-message-date"
                                        popovertargetaction="hide"
                                        pp-ref={messageDateCloser}
                                    >
                                    </button>
                                </PopoverContent>
                            </Popover>
                            <Dialog>
                                <DialogTrigger
                                    class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-muted-foreground"
                                    attrs=@{labelled("More message filters")}
                                    onclick={prepareMoreMessageFilters()}
                                >
                                    {messageFilters.attachments || messageFilters.mentions ? "More · " + (Number(messageFilters.attachments) + Number(messageFilters.mentions)) : "More"}
                                </DialogTrigger>
                                <DialogContent
                                    class="message-more-dialog bg-popover text-ink [&>[data-slot=dialog-close]]:right-6 [&>[data-slot=dialog-close]]:top-6 w-[400px] max-w-[calc(100vw-32px)] gap-7 rounded-sm p-8 sm:max-w-[400px]"
                                    onclose={setMoreMessageFiltersOpen(false)}
                                >
                                    <DialogHeader>
                                        <DialogTitle class="text-lg">"More filters"</DialogTitle>
                                    </DialogHeader>
                                    <div
                                        class="grid grid-cols-2 gap-4 text-[13px] text-muted-foreground"
                                    >
                                        <label class="flex items-center gap-3">
                                            <input
                                                type="checkbox"
                                                class="size-4 accent-brand-ink"
                                                aria-label="Mentions me"
                                                checked={moreMentionsDraft}
                                                onchange={setMoreMentionsDraft(target.checked)}
                                            />
                                            "@mentions me"
                                        </label>
                                        <label class="flex items-center gap-3">
                                            <input
                                                type="checkbox"
                                                class="size-4 accent-brand-ink"
                                                aria-label="Has attachment"
                                                checked={moreAttachmentsDraft}
                                                onchange={setMoreAttachmentsDraft(target.checked)}
                                            />
                                            "Has attachment"
                                        </label>
                                    </div>
                                    <DialogFooter class="mt-2 gap-2">
                                        <button
                                            type="button"
                                            class="h-8 w-24 rounded-sm border border-edge text-[13px] disabled:opacity-40"
                                            aria-label="Clear message filters"
                                            onclick={clearMoreMessageDraft()}
                                            disabled={!moreMentionsDraft && !moreAttachmentsDraft}
                                        >
                                            "Clear"
                                        </button>
                                        <DialogClose
                                            class="h-8 w-24 rounded-sm border border-edge text-[13px] disabled:opacity-40"
                                            attrs=@{named("Apply message filters")}
                                            onclick={applyMoreMessageFilters()}
                                            disabled={moreMentionsDraft === messageFilters.mentions && moreAttachmentsDraft === messageFilters.attachments}
                                        >
                                            "Apply"
                                        </DialogClose>
                                    </DialogFooter>
                                </DialogContent>
                            </Dialog>
                            <button
                                type="button"
                                class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-link"
                                hidden={!messageFiltersActive}
                                onclick={resetMessageFilters()}
                            >
                                "Clear filters"
                            </button>
                        </div>
                        <div
                            class="mx-auto flex h-10 w-full max-w-[760px] items-center gap-2"
                            hidden={searchTab !== "files"}
                        >
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="search-file-type"
                                    class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-muted-foreground"
                                    attrs=@{labelled("Filter files by type")}
                                >
                                    <span>
                                        {fileFilters.type === "all" ? "File Type" : fileTypeLabel(fileFilters.type)}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                    menu="search-file-type"
                                    align="start"
                                    class="search-file-type-menu w-40 rounded-sm border-0 bg-popover p-1"
                                    ontoggle={searchFilterToggled("file-type", event.newState)}
                                >
                                    <template pp-for="option in searchFileTypes">
                                        <DropdownMenuItem
                                            key={option.id}
                                            class="search-file-type-option min-h-[38px] gap-2 px-2.5 py-2 text-[13px] focus:rounded-[3px] focus:bg-popover focus:outline-2 focus:-outline-offset-2 focus:outline-ink"
                                            onclick={setFileFilter("type", fileFilters.type === option.id ? "all" : option.id)}
                                        >
                                            <span
                                                class="flex size-4 shrink-0 items-center justify-center"
                                                aria-hidden="true"
                                                pp-style={"--file-kind-color: " + option.color}
                                            >
                                                <span hidden={option.id !== "images"}>
                                                    <Icon name="image" class="size-3.5" />
                                                </span>
                                                <svg
                                                    hidden={option.id === "images"}
                                                    class="size-4"
                                                    viewBox="0 0 20 20"
                                                    fill="none"
                                                >
                                                    <path d="M6 2h8l4 4v12H6V2Z" fill="#f5f5f5" />
                                                    <path d="M14 2v5h4" fill="#bdbdbd" />
                                                    <path
                                                        d="M9 9h6M9 12h6M9 15h6"
                                                        stroke="var(--file-kind-color)"
                                                        stroke-width="1"
                                                    />
                                                    <rect
                                                        class="file-kind-badge"
                                                        x="1"
                                                        y="7"
                                                        width="11"
                                                        height="11"
                                                        rx="1"
                                                        fill="var(--file-kind-color)"
                                                    />
                                                    <text
                                                        x="6.5"
                                                        y="15.3"
                                                        text-anchor="middle"
                                                        fill="white"
                                                        font-family="Arial,sans-serif"
                                                        pp-style={option.id === "pdf" ? "font-size:4.5px" : "font-size:8px"}
                                                        font-weight="700"
                                                    >
                                                        {option.glyph}
                                                    </text>
                                                </svg>
                                            </span>
                                            <span class="flex-1">{option.label}</span>
                                            <span hidden={fileFilters.type !== option.id}>
                                                <Icon name="check" class="size-4" />
                                            </span>
                                        </DropdownMenuItem>
                                    </template>
                                </DropdownMenuContent>
                            </DropdownMenu>
                            <Popover>
                                <PopoverTrigger
                                    name="search-file-owner"
                                    class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-muted-foreground"
                                    attrs=@{labelled("Filter files by sender")}
                                >
                                    <span>
                                        {fileFilters.owner === "all" ? "Shared by" : "Shared by: You"}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </PopoverTrigger>
                                <PopoverContent
                                    name="search-file-owner"
                                    align="start"
                                    class="message-from-picker bg-popover text-ink [&_.message-filter-input:focus-visible]:border-transparent [&_.message-filter-input:focus-visible]:shadow-[inset_0_-2px_var(--color-brand-ink)] file-owner-picker w-[350px] max-w-[calc(100vw-24px)] px-3 py-4"
                                    ontoggle={searchFilterToggled("file-owner", event.newState)}
                                >
                                    <input
                                        type="search"
                                        class="message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink w-full"
                                        placeholder="Enter name, email or phone number"
                                        aria-label="Find file sender"
                                        autocomplete="off"
                                        value={fileOwnerQuery}
                                        oninput={setFileOwnerQuery(target.value)}
                                    />
                                    <div class="mt-2" hidden={!fileOwnerQuery.trim()}>
                                        <button
                                            type="button"
                                            class="file-owner-option flex w-full items-center justify-between rounded px-2 py-2 text-left text-[13px] hover:bg-hover"
                                            hidden={!fileOwnerMatches}
                                            onclick={chooseFileOwner("self")}
                                        >
                                            "You"
                                            <span hidden={fileFilters.owner !== "self"}>
                                                <Icon name="check" class="size-4" />
                                            </span>
                                        </button>
                                        <p
                                            class="px-2 py-2 text-xs text-muted-foreground"
                                            hidden={fileOwnerMatches}
                                        >
                                            "No matching people."
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        class="mt-2 rounded px-2 py-1 text-[13px] text-link hover:bg-hover"
                                        hidden={fileFilters.owner === "all"}
                                        onclick={chooseFileOwner("all")}
                                    >
                                        "Clear sender"
                                    </button>
                                    <button
                                        type="button"
                                        hidden
                                        popovertarget="popover-search-file-owner"
                                        popovertargetaction="hide"
                                        pp-ref={fileOwnerCloser}
                                    >
                                    </button>
                                </PopoverContent>
                            </Popover>
                            <Popover>
                                <PopoverTrigger
                                    name="search-file-date"
                                    class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-muted-foreground"
                                    attrs=@{labelled("Filter files by date")}
                                    onclick={prepareFileDates()}
                                >
                                    <span>{fileFilters.dateLabel}</span>
                                    <Icon name="chevron-down" class="size-3" />
                                </PopoverTrigger>
                                <PopoverContent
                                    name="search-file-date"
                                    align="start"
                                    class="message-date-picker rounded-[3px] border-edge bg-popover text-ink file-date-picker w-[300px] max-w-[calc(100vw-24px)] p-0"
                                    ontoggle={searchFilterToggled("file-date", event.newState)}
                                >
                                    <div class="px-4 pb-3 pt-4">
                                        <h2 class="mb-1 text-[13px] font-semibold">"Date"</h2>
                                        <template pp-for="option in messageDatePresets">
                                            <button
                                                type="button"
                                                key={option.id}
                                                class="file-date-preset block h-8 w-full rounded text-left text-[13px] hover:bg-hover"
                                                onclick={applyFileDates(option.id)}
                                                disabled={fileDateBusy}
                                            >
                                                {option.label}
                                            </button>
                                        </template>
                                    </div>
                                    <div class="space-y-3 border-t border-line px-4 pb-3 pt-4">
                                        <label
                                            class="message-date-field group/date relative block text-xs"
                                        >
                                            "From"
                                            <input
                                                type="date"
                                                class={fileDateFrom ? "message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink mt-1 w-full" : "message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink message-date-empty [&:not(:focus)::-webkit-datetime-edit]:text-transparent mt-1 w-full"}
                                                aria-label="File date from"
                                                value={fileDateFrom}
                                                oninput={changeFileDateFrom(event)}
                                                onchange={changeFileDateFrom(event)}
                                                max={fileDateTo || null}
                                            />
                                            <span
                                                class="message-date-placeholder group-focus-within/date:hidden pointer-events-none absolute bottom-2 left-2 text-[13px]"
                                                hidden={!!fileDateFrom}
                                            >
                                                "Select a Date"
                                            </span>
                                        </label>
                                        <label
                                            class="message-date-field group/date relative block text-xs"
                                        >
                                            "To"
                                            <input
                                                type="date"
                                                class="message-filter-input h-[34px] rounded-[3px] border border-transparent bg-sunken px-2 text-[13px] outline-none focus-visible:border-brand-ink calendar-date-pretty [&:not(:focus)::-webkit-datetime-edit]:text-transparent mt-1 w-full"
                                                aria-label="File date to"
                                                value={fileDateTo}
                                                oninput={changeFileDateTo(event)}
                                                onchange={changeFileDateTo(event)}
                                                min={fileDateFrom || null}
                                            />
                                            <span
                                                class="calendar-date-display group-focus-within/date:hidden pointer-events-none absolute bottom-2 left-2 right-8 truncate text-[13px]"
                                            >
                                                {fileDateTo ? formatCalendarDate(fileDateTo) : "Select a Date"}
                                            </span>
                                        </label>
                                        <p
                                            class="text-xs text-danger-ink"
                                            role="alert"
                                            hidden={!fileDateError}
                                        >
                                            {fileDateError}
                                        </p>
                                        <div class="flex justify-end gap-3">
                                            <button
                                                type="button"
                                                class="rounded px-3 py-1.5 text-[13px] hover:bg-hover disabled:opacity-40"
                                                aria-label="Clear file date"
                                                onclick={clearFileDates()}
                                                disabled={fileFilters.date === "any" && !fileDateFrom}
                                            >
                                                "Clear"
                                            </button>
                                            <button
                                                type="button"
                                                class="rounded bg-sunken px-6 py-1.5 text-[13px] font-semibold disabled:opacity-40"
                                                aria-label="Apply file date"
                                                onclick={applyFileDates("custom")}
                                                disabled={!fileDateRangeValid || fileDateBusy}
                                            >
                                                "Apply"
                                            </button>
                                        </div>
                                    </div>
                                    <button
                                        type="button"
                                        hidden
                                        popovertarget="popover-search-file-date"
                                        popovertargetaction="hide"
                                        pp-ref={fileDateCloser}
                                    >
                                    </button>
                                </PopoverContent>
                            </Popover>
                            <button
                                type="button"
                                class="global-search-filter-trigger inline-flex h-10 items-center gap-1 whitespace-nowrap rounded-[3px] px-3 text-[13px] hover:bg-hover hover:text-ink focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink text-link"
                                hidden={!fileFiltersActive}
                                onclick={resetFileFilters()}
                            >
                                "Clear filters"
                            </button>
                        </div>
                    </div>
                    <div
                        id="global-search-panel"
                        class="scroll-thin min-h-0 flex-1 overflow-y-auto px-5 py-5"
                        role="tabpanel"
                        aria-labelledby={"global-search-tab-" + searchTab}
                        tabindex="0"
                        aria-busy={searchLoading}
                    >
                        <div class="mx-auto w-full max-w-[760px]">
                            <p class="sr-only" role="status">
                                {searchLoading ? "Searching" : visibleSearchRows.length + " " + searchTab + " results"}
                            </p>
                            <div class="space-y-4" hidden={searchLoading}>
                                <template pp-for="row in visibleSearchRows">
                                    <button
                                        type="button"
                                        key={row.key}
                                        class="global-search-card flex min-h-[72px] w-full items-center gap-3 rounded-lg border border-line bg-popover px-4 py-3 text-left hover:bg-hover focus-visible:outline-2 focus-visible:outline-ring"
                                        onclick={openSearchRow(row)}
                                    >
                                        <span
                                            class="flex size-8 shrink-0 items-center justify-center rounded-full bg-avatar text-xs font-semibold text-avatar-ink"
                                            hidden={row.kind === "files" || row.kind === "communities"}
                                        >
                                            {row.initials}
                                        </span>
                                        <span
                                            class="flex size-8 shrink-0 items-center justify-center rounded-md bg-brand-soft text-brand-ink"
                                            hidden={row.kind !== "files"}
                                        >
                                            <Icon name="file" class="size-5" />
                                        </span>
                                        <span
                                            class="flex size-8 shrink-0 items-center justify-center rounded-md bg-brand-soft text-brand-ink"
                                            hidden={row.kind !== "communities"}
                                        >
                                            <Icon name="communities" class="size-5" />
                                        </span>
                                        <span class="min-w-0 flex-1">
                                            <span
                                                class="block truncate text-[13px] font-semibold text-ink"
                                            >
                                                {row.title}
                                            </span>
                                            <span
                                                class="mt-1 block truncate text-xs text-muted-foreground"
                                                hidden={!row.subtitle}
                                            >
                                                {row.subtitle}
                                            </span>
                                        </span>
                                    </button>
                                </template>
                            </div>
                            <p
                                class="py-10 text-center text-sm text-muted-foreground"
                                hidden={!searchLoading}
                            >
                                "Searching…"
                            </p>
                            <div
                                class="py-16 text-center"
                                hidden={searchLoading || visibleSearchRows.length > 0 || !!workspaceSearchError}
                            >
                                <span class="mb-4 inline-flex text-subtle">
                                    <Icon name="search" class="size-10" />
                                </span>
                                <h1 class="text-lg font-semibold">
                                    {"No " + searchTab + " found"}
                                </h1>
                                <p class="mt-2 text-sm text-muted-foreground">
                                    "Try another name or keyword."
                                </p>
                            </div>
                            <div
                                class="rounded-lg border border-line bg-raised px-5 py-4 text-sm"
                                hidden={!workspaceSearchError}
                                role="alert"
                            >
                                <p>{workspaceSearchError}</p>
                                <button
                                    type="button"
                                    class="mt-3 rounded px-3 py-1.5 text-link hover:bg-hover"
                                    onclick={setSearchRetry(value => value + 1)}
                                >
                                    "Try again"
                                </button>
                            </div>
                            <p
                                class="mt-5 text-center text-xs text-subtle"
                                hidden={searchTab !== "messages" && searchTab !== "communities"}
                            >
                                "Results include conversations and communities on the current page."
                            </p>
                            <p
                                class="mt-5 text-center text-xs text-subtle"
                                hidden={searchLoading || visibleSearchRows.length < 50 || (searchTab !== "people" && searchTab !== "files")}
                            >
                                "Showing the first 50 matches. Refine your search to find more."
                            </p>
                        </div>
                    </div>
                </section>
            </div>
            <Dialog>
                <DialogTrigger attrs=@{hidden()}>
                    <span pp-ref={surveyOpener}></span>
                </DialogTrigger>
                <DialogContent class="sm:max-w-[600px]!" onclose={setSurveyActive(false)}>
                    <DialogHeader>
                        <DialogTitle class="text-center text-xl">
                            <span hidden={survey !== 1}>"What is your primary use for Teams?"</span>
                            <span hidden={survey !== 2}>"Choose a category"</span>
                            <span hidden={survey !== 3}>"Thank you!"</span>
                        </DialogTitle>
                        <DialogDescription class="text-center" hidden={survey !== 3}>
                            "Your responses were saved."
                        </DialogDescription>
                    </DialogHeader>
                    <div class="grid grid-cols-2 gap-4" hidden={survey !== 1}>
                        <template pp-for="choice in PRIMARY_USES">
                            <Button
                                key={choice.label}
                                variant="outline"
                                class="h-[94px]! flex-col whitespace-normal text-center text-[13px] font-semibold hover:border-brand"
                                value={choice.label}
                                onclick={pickUse(event.currentTarget.value)}
                            >
                                <span class="text-2xl leading-none" hidden={!choice.emoji}>
                                    {choice.emoji}
                                </span>
                                {choice.label}
                            </Button>
                        </template>
                    </div>
                    <div class="flex flex-col gap-3" hidden={survey !== 2}>
                        <template pp-for="label in CATEGORIES">
                            <Button
                                key={label}
                                variant="outline"
                                size="lg"
                                class="w-full text-[13px] font-semibold hover:border-brand"
                                disabled={saving}
                                value={label}
                                onclick={pickCategory(event.currentTarget.value)}
                            >
                                {label}
                            </Button>
                        </template>
                        <p class="text-center text-xs text-busy" hidden={!saveError}>{saveError}</p>
                    </div>
                    <div class="text-center" hidden={survey !== 3}>
                        <p class="text-5xl leading-none">"🎉"</p>
                        <p class="mt-3 text-xs text-subtle">{summary}</p>
                    </div>
                    <DialogFooter class="items-center sm:justify-between!">
                        <Button
                            variant="outline"
                            size="sm"
                            class="w-24"
                            hidden={survey === 1}
                            onclick={setSurvey(survey - 1)}
                        >
                            "Back"
                        </Button>
                        <span class="w-24" hidden={survey !== 1}></span>
                        <Button variant="link" size="sm">"Learn more"</Button>
                        <DialogClose class=@{close_survey.as_str()} hidden={survey !== 3}>
                            "Close"
                        </DialogClose>
                        <span class="w-24" hidden={survey === 3}></span>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
            <Dialog>
                <DialogTrigger attrs=@{hidden()}>
                    <span pp-ref={shortcutsOpener}></span>
                </DialogTrigger>
                <DialogClose attrs=@{hidden()}>
                    <span pp-ref={shortcutsCloser}></span>
                </DialogClose>
                <DialogContent
                    hide_close=@{true}
                    class="shortcuts-dialog h-[calc(100dvh-32px)] max-h-[1400px] w-[calc(100vw-32px)] max-w-[1080px]! gap-0 bg-raised! p-0! sm:max-w-[1080px]!"
                    onclose={restoreShortcutsFocus()}
                >
                    <KeyboardShortcuts />
                </DialogContent>
            </Dialog>
            <script>r##"
                const ShortcutContext = window.UpurnakaShortcutContext ||= pp.createContext(null);
                const SearchContext = window.UpurnakaSearchContext ||= pp.createContext(null);
                const pageShortcuts = pp.ref(null);
                const globalSearch = pp.ref(null);
                const searchSubmit = pp.ref(null);
                const appRail = pp.ref(null);
                const moreBadge = pp.ref(null);
                const shortcutsOpener = pp.ref(null);
                const shortcutsCloser = pp.ref(null);
                const surveyOpener = pp.ref(null);
                const [shortcutsActive, setShortcutsActive] = pp.state(false);
                const [surveyActive, setSurveyActive] = pp.state(false);
                const [appBarOpen, setAppBarOpen] = pp.state(true);
                const [signoutError, setSignoutError] = pp.state('');
                async function signOut() {
                    try { await pp.rpc('sign_out_account', {}); window.location.replace('/signin'); }
                    catch (e) { setSignoutError('Could not sign out. Please try again.'); }
                }
                // The two dialogs are rahti-ui's, which open from a trigger and
                // close from a close button; a keyboard shortcut or a menu row
                // reaches them by clicking the hidden pair each one carries.
                function openShortcuts() {
                    pageShortcuts.current?.dismissPopovers?.();
                    if (shortcutsActive) return;
                    setShortcutsActive(true);
                    shortcutsOpener.current?.click();
                }
                function closeShortcuts() {
                    shortcutsCloser.current?.click();
                }
                // The row that opened the dialog was in a menu that has since
                // closed, so focus goes back to the menu's own button.
                function restoreShortcutsFocus() {
                    setShortcutsActive(false);
                    moreBadge.current?.closest("button")?.focus();
                }
                // Document-level keyboard integration must also receive keys when
                // the body has focus. Keep ordinary controls in their markup.
                const handleGlobalKey = pp.event(event => {
                    if (event.defaultPrevented || event.isComposing) return;
                    if (moreMessageFiltersOpen) return;
                    const control = (event.ctrlKey || event.metaKey) && !event.altKey;
                    const key = event.key.toLowerCase();
                    if (control && !event.shiftKey && (key === "." || event.code === "Period")) {
                        event.preventDefault();
                        if (event.repeat) return;
                        if (shortcutsActive) closeShortcuts();
                        else openShortcuts();
                        return;
                    }
                    if (shortcutsActive || surveyActive) return;
                    if (control && !event.shiftKey && key === ",") {
                        event.preventDefault();
                        pp.redirect("/settings");
                        return;
                    }
                    if (searchView && event.key === 'Escape') {if (Object.values(openSearchFilters).some(Boolean)) return; event.preventDefault(); clearGlobalSearch(); return;}
                    if (!searchView) pageShortcuts.current?.handleKey?.(event);
                    if (event.defaultPrevented) return;
                    if (control && !event.shiftKey && ["e", "g", "/"].includes(key)) {
                        event.preventDefault();
                        if (key === "/") setQuery("/");
                        globalSearch.current?.focus(); globalSearch.current?.select();
                    } else if (control && !event.shiftKey && key === "l") {
                        event.preventDefault(); setAppBarOpen(true);
                        requestAnimationFrame(() => appRail.current?.focus());
                    } else if (control && !event.shiftKey && (key === "\\" || event.code === "Backslash")) {
                        event.preventDefault(); setAppBarOpen(open => !open);
                    } else if (control && !event.shiftKey && /^[1-7]$/.test(key)) {
                        event.preventDefault();
                        pp.redirect(["/chat", "/meet", "/people", "/communities", "/calendar", "/activity", "/apps"][Number(key) - 1]);
                    } else if (event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey && ["ArrowLeft", "ArrowRight"].includes(event.key)) {
                        event.preventDefault();
                        if (event.key === "ArrowLeft") goBack(); else goForward();
                    }
                });
                pp.effect(() => {
                    const previous = document.onkeydown;
                    const handler = event => {
                        handleGlobalKey(event);
                        if (!event.defaultPrevented) previous?.call(document, event);
                    };
                    document.onkeydown = handler;
                    return () => { if (document.onkeydown === handler) document.onkeydown = previous; };
                }, [handleGlobalKey]);
                const [query, setQuery] = pp.state("");
                const [suggestionsOpen, setSuggestionsOpen] = pp.state(false);
                const [searchView, setSearchView] = pp.state(false);
                const [searchTab, setSearchTab] = pp.state('people');
                const [messageFilters, setMessageFilters] = pp.state({from:'all',date:'any',dateLabel:'Date',dateStart:null,dateEnd:null,dateFrom:'',dateTo:'',attachments:false,mentions:false});
                const [senderQuery, setSenderQuery] = pp.state('');
                const messageFromCloser = pp.ref(null);
                const messageDateCloser = pp.ref(null);
                const applicationToday = @{crate::rahti::time::today().to_string()};
                const [messageDateFrom, setMessageDateFrom] = pp.state('');
                const [messageDateTo, setMessageDateTo] = pp.state(applicationToday);
                const [messageDateBusy, setMessageDateBusy] = pp.state(false);
                const [messageDateError, setMessageDateError] = pp.state('');
                const messageDateRangeValid = Boolean(messageDateFrom && messageDateTo && messageDateFrom <= messageDateTo);
                const messageDateTicket = pp.ref(0);
                const messageDatePresets = [{id:'today',label:'Today'},{id:'yesterday',label:'Yesterday'},{id:'week',label:'This week'},{id:'month',label:'This month'}];
                const [moreMessageFiltersOpen, setMoreMessageFiltersOpen] = pp.state(false);
                const [moreMentionsDraft, setMoreMentionsDraft] = pp.state(false);
                const [moreAttachmentsDraft, setMoreAttachmentsDraft] = pp.state(false);
                const [fileFilters, setFileFilters] = pp.state({type:'all',owner:'all',date:'any',dateLabel:'Date',dateFrom:'',dateTo:''});
                const [fileOwnerQuery, setFileOwnerQuery] = pp.state('');
                const fileOwnerCloser = pp.ref(null);
                const fileDateCloser = pp.ref(null);
                const [fileDateFrom, setFileDateFrom] = pp.state('');
                const [fileDateTo, setFileDateTo] = pp.state(applicationToday);
                const [fileDateBusy, setFileDateBusy] = pp.state(false);
                const [fileDateError, setFileDateError] = pp.state('');
                const fileDateTicket = pp.ref(0);
                const fileDateRangeValid = Boolean(fileDateFrom && fileDateTo && fileDateFrom <= fileDateTo);
                const fileOwnerMatches = ('You '+@{user.name.as_str()}+' '+@{user.email.as_str()}).toLocaleLowerCase().includes(fileOwnerQuery.trim().toLocaleLowerCase());
                const [openSearchFilters, setOpenSearchFilters] = pp.state({});
                const searchFileTypes = [{id:'excel',label:'Excel',color:'#107c41',glyph:'X'},{id:'images',label:'Images',color:'#c8c8c8',glyph:''},{id:'loop',label:'Loop',color:'#8661c5',glyph:'L'},{id:'onenote',label:'OneNote',color:'#8033a5',glyph:'N'},{id:'pdf',label:'PDF',color:'#d44830',glyph:'PDF'},{id:'powerpoint',label:'PowerPoint',color:'#d35230',glyph:'P'},{id:'text',label:'Text file',color:'#747474',glyph:'T'},{id:'word',label:'Word',color:'#185abd',glyph:'W'}];
                const messageFiltersActive = messageFilters.from !== 'all' || messageFilters.date !== 'any' || messageFilters.attachments || messageFilters.mentions;
                const fileFiltersActive = fileFilters.type !== 'all' || fileFilters.owner !== 'all' || fileFilters.date !== 'any';
                const [remoteSearchRows, setRemoteSearchRows] = pp.state([]);
                const [remoteSearchLoading, setRemoteSearchLoading] = pp.state(false);
                const [remoteSearchError, setRemoteSearchError] = pp.state('');
                const [searchRetry, setSearchRetry] = pp.state(0);
                const [localSearchItems, setLocalSearchItems] = pp.state({});
                const searchHandlers = pp.ref({});
                const searchTabRefs = pp.ref({});
                const searchTabs = [{id:'people',label:'People'},{id:'messages',label:'Messages'},{id:'files',label:'Files'},{id:'communities',label:'Communities'}];
                const publishSearch = pp.event((source,rows,open) => {
                    searchHandlers.current[source] = open;
                    setLocalSearchItems(previous => ({...previous,[source]:rows.map(row => ({...row,source}))}));
                });
                const removeSearch = pp.event(source => {
                    delete searchHandlers.current[source];
                    setLocalSearchItems(previous => {const next={...previous}; delete next[source]; return next;});
                });
                const searchStore = pp.memo(() => ({publish:publishSearch,remove:removeSearch}),[publishSearch,removeSearch]);
                const searchTerm = query.trim().replace(/^\//, "").toLocaleLowerCase();
                const searchLoading = searchView && ['people','files'].includes(searchTab) && remoteSearchLoading;
                const workspaceSearchError = ['people','files'].includes(searchTab) ? remoteSearchError : '';
                const localRows = Object.values(localSearchItems).flat().filter(row => (row.title+' '+(row.subtitle || '')).toLocaleLowerCase().includes(searchTerm));
                const messageSenders = [{id:'all',label:'Anyone'},...Array.from(new Map(Object.values(localSearchItems).flat().filter(row=>row.kind === 'messages' && row.authorId).map(row=>[row.authorId,{id:row.authorId,label:row.authorId === @{user.id.as_str()} ? 'You' : row.author}])).values())];
                const filteredMessageSenders = messageSenders.filter(sender=>sender.id !== 'all' && (sender.label+' '+(sender.id === @{user.id.as_str()} ? @{user.name.as_str()}+' '+@{user.email.as_str()} : '')).toLocaleLowerCase().includes(senderQuery.trim().toLocaleLowerCase()));
                const visibleSearchRows = [...remoteSearchRows,...localRows].filter(row => {
                    if (row.kind !== searchTab) return false;
                    if (row.kind === 'messages') return (messageFilters.from === 'all' || row.authorId === messageFilters.from) && (messageFilters.dateStart === null || (row.sentAt >= messageFilters.dateStart && row.sentAt < messageFilters.dateEnd)) && (!messageFilters.attachments || row.hasAttachments) && (!messageFilters.mentions || row.mentionsMe);
                    if (row.kind === 'files') return fileFilters.owner === 'all' || row.owner === 'self';
                    return true;
                });
                function senderLabel(id) {return messageSenders.find(sender=>sender.id===id)?.label || 'Selected person';}
                function fileTypeLabel(id) {return searchFileTypes.find(option=>option.id===id)?.label || 'File Type';}
                function searchFilterToggled(id,state) {setOpenSearchFilters(previous=>({...previous,[id]:state==='open'}));}
                function setMessageFilter(key,value) {setMessageFilters(previous=>({...previous,[key]:value}));}
                function chooseMessageSender(id) {setMessageFilter('from',id); setSenderQuery(''); messageFromCloser.current?.click();}
                function prepareMessageDates() {setMessageDateFrom(messageFilters.dateFrom); setMessageDateTo(messageFilters.dateTo || applicationToday); setMessageDateError('');}
                function changeMessageDateFrom(event) {setMessageDateFrom(event.currentTarget.value); setMessageDateError('');}
                function changeMessageDateTo(event) {setMessageDateTo(event.currentTarget.value); setMessageDateError('');}
                const applyMessageDates = pp.event(async preset => {
                    if (messageDateBusy) return;
                    const ticket = ++messageDateTicket.current;
                    setMessageDateBusy(true); setMessageDateError('');
                    try {
                        const range = await pp.rpc('message_search_dates',{preset,from:messageDateFrom || null,to:messageDateTo || null});
                        if (ticket !== messageDateTicket.current) return;
                        setMessageFilters(previous=>({...previous,date:range.preset,dateLabel:range.label,dateStart:range.start,dateEnd:range.end,dateFrom:range.from,dateTo:range.to}));
                        messageDateCloser.current?.click();
                    } catch (failure) {if (ticket === messageDateTicket.current) setMessageDateError(failure.message || 'The date filter could not be applied.');}
                    finally {if (ticket === messageDateTicket.current) setMessageDateBusy(false);}
                });
                function clearMessageDates() {
                    messageDateTicket.current++; setMessageDateBusy(false); setMessageDateError('');
                    setMessageDateFrom(''); setMessageDateTo(applicationToday);
                    setMessageFilters(previous=>({...previous,date:'any',dateLabel:'Date',dateStart:null,dateEnd:null,dateFrom:'',dateTo:''}));
                    messageDateCloser.current?.click();
                }
                function prepareMoreMessageFilters() {setMoreMentionsDraft(messageFilters.mentions); setMoreAttachmentsDraft(messageFilters.attachments); setMoreMessageFiltersOpen(true);}
                function clearMoreMessageDraft() {setMoreMentionsDraft(false); setMoreAttachmentsDraft(false);}
                function applyMoreMessageFilters() {setMessageFilters(previous=>({...previous,mentions:moreMentionsDraft,attachments:moreAttachmentsDraft}));}
                function setFileFilter(key,value) {
                    setFileFilters(previous=>({...previous,[key]:value}));
                    if (key !== 'owner') {setRemoteSearchLoading(true); setRemoteSearchRows(previous=>previous.filter(row=>row.kind!=='files'));}
                }
                function chooseFileOwner(owner) {setFileFilter('owner',owner); setFileOwnerQuery(''); fileOwnerCloser.current?.click();}
                function formatCalendarDate(value) {
                    try {return new Intl.DateTimeFormat(navigator.language,{weekday:'long',day:'numeric',month:'long',year:'numeric',timeZone:'UTC'}).format(new Date(value+'T12:00:00Z'));}
                    catch (_) {return value;}
                }
                function prepareFileDates() {setFileDateFrom(fileFilters.dateFrom); setFileDateTo(fileFilters.dateTo || applicationToday); setFileDateError('');}
                function changeFileDateFrom(event) {setFileDateFrom(event.currentTarget.value); setFileDateError('');}
                function changeFileDateTo(event) {setFileDateTo(event.currentTarget.value); setFileDateError('');}
                const applyFileDates = pp.event(async preset => {
                    if (fileDateBusy) return;
                    const ticket = ++fileDateTicket.current;
                    setFileDateBusy(true); setFileDateError('');
                    try {
                        const range = await pp.rpc('message_search_dates',{preset,from:fileDateFrom || null,to:fileDateTo || null});
                        if (ticket !== fileDateTicket.current) return;
                        setFileFilters(previous=>({...previous,date:range.preset,dateLabel:range.label,dateFrom:range.from,dateTo:range.to}));
                        setRemoteSearchLoading(true); setRemoteSearchRows(previous=>previous.filter(row=>row.kind!=='files'));
                        fileDateCloser.current?.click();
                    } catch (failure) {if (ticket === fileDateTicket.current) setFileDateError(failure.message || 'The date filter could not be applied.');}
                    finally {if (ticket === fileDateTicket.current) setFileDateBusy(false);}
                });
                function clearFileDates() {
                    fileDateTicket.current++; setFileDateBusy(false); setFileDateError(''); setFileDateFrom(''); setFileDateTo(applicationToday);
                    setFileFilters(previous=>({...previous,date:'any',dateLabel:'Date',dateFrom:'',dateTo:''}));
                    fileDateCloser.current?.click();
                }
                function resetMessageFilters() {
                    messageDateTicket.current++; setMessageDateBusy(false); setMessageDateError(''); setSenderQuery('');
                    setMessageFilters({from:'all',date:'any',dateLabel:'Date',dateStart:null,dateEnd:null,dateFrom:'',dateTo:'',attachments:false,mentions:false});
                }
                function resetFileFilters() {
                    fileDateTicket.current++; setFileDateBusy(false); setFileDateError(''); setFileOwnerQuery('');
                    setFileFilters({type:'all',owner:'all',date:'any',dateLabel:'Date',dateFrom:'',dateTo:''});
                }
                function changeGlobalQuery(value) {
                    setQuery(value); setSuggestionsOpen(true);
                    if (!value.trim()) {setSearchView(false); setRemoteSearchRows([]); setRemoteSearchError('');}
                    else if (searchView) {setRemoteSearchLoading(true); setRemoteSearchRows([]);}
                }
                function submitGlobalSearch() {
                    if (!query.trim()) return;
                    pageShortcuts.current?.dismissPopovers?.();
                    if (!searchView) {setSearchTab('people'); resetMessageFilters(); resetFileFilters();}
                    else setSearchRetry(value=>value+1);
                    setQuery(query.trim()); setRemoteSearchLoading(true);
                    setSearchView(true); setSuggestionsOpen(false);
                    globalSearch.current?.focus();
                }
                function clearGlobalSearch() {
                    setQuery(''); setSearchView(false); setSuggestionsOpen(false); setRemoteSearchRows([]); setRemoteSearchError(''); resetMessageFilters(); resetFileFilters();
                    globalSearch.current?.focus();
                }
                function globalSearchBlur(event) {
                    if (!event.currentTarget.contains(event.relatedTarget)) setSuggestionsOpen(false);
                }
                function searchTabKey(event) {
                    const index = searchTabs.findIndex(tab => tab.id === searchTab);
                    const next = event.key === 'Home' ? 0 : event.key === 'End' ? searchTabs.length-1 : event.key === 'ArrowRight' ? (index+1)%searchTabs.length : event.key === 'ArrowLeft' ? (index+searchTabs.length-1)%searchTabs.length : -1;
                    if (next < 0) return;
                    event.preventDefault(); const id = searchTabs[next].id;
                    setSearchTab(id); searchTabRefs.current[id]?.focus();
                }
                pp.effect(() => {
                    if (!searchView || !searchTerm) return;
                    let active = true;
                    setRemoteSearchLoading(true); setRemoteSearchRows([]); setRemoteSearchError('');
                    const timer = setTimeout(async () => {
                        try {
                            const response = await pp.rpc('search_workspace',{query:searchTerm,file_type:fileFilters.type,file_date:fileFilters.date,file_from:fileFilters.dateFrom || null,file_to:fileFilters.dateTo || null});
                            if (!active) return;
                            const people = response.people.map(person => ({...person,key:'person:'+person.id,kind:'people',subtitle:person.is_self ? 'You' : ''}));
                            const files = response.files.map(file => ({key:'file:'+file.id,id:file.id,kind:'files',title:file.name,subtitle:file.uploaded,owner:'self'}));
                            setRemoteSearchRows([...people,...files]);
                        } catch (failure) {if (active) setRemoteSearchError(failure.message || 'Search is unavailable. Please try again.');}
                        finally {if (active) setRemoteSearchLoading(false);}
                    },200);
                    return () => {active=false; clearTimeout(timer);};
                },[searchView,searchTerm,searchRetry,fileFilters.type,fileFilters.date,fileFilters.dateFrom,fileFilters.dateTo]);
                const openSearchRow = pp.event(row => {
                    setSearchView(false); setQuery(''); setSuggestionsOpen(false);
                    if (row.kind === 'files') {window.open('/chat/files/'+encodeURIComponent(row.id),'_blank','noopener,noreferrer'); return;}
                    const handler = searchHandlers.current[row.source || (row.kind === 'people' ? 'chat' : '')];
                    if (handler) {handler(row); return;}
                    if (row.kind === 'people') pp.redirect('/chat?person='+encodeURIComponent(row.id)+'&name='+encodeURIComponent(row.title));
                });
                const searchResults = [
                    {label: @{user.name.as_str()} + " (You)", href:"/chat"},
                    {label:"Meet", href:"/meet"}, {label:"People", href:"/people"},
                    {label:"Communities", href:"/communities"}, {label:"Calendar", href:"/calendar"},
                    {label:"Activity", href:"/activity"}, {label:"Files", href:"/files"}, {label:"Apps", href:"/apps"},
                    {label:"Settings", href:"/settings"}
                ].filter(result => result.label.toLocaleLowerCase().includes(searchTerm) || result.href.slice(1).includes(searchTerm));
                const visitSearchResult = pp.event(function visitSearchResult(href) {setQuery(""); setSuggestionsOpen(false); pp.redirect(href);});
                function handleGlobalSearchKey(event) {
                    if (event.isComposing) return;
                    if (event.key === "Escape") {event.preventDefault(); clearGlobalSearch();}
                    else if (event.key === 'ArrowDown' && query.trim() && !searchView) {event.preventDefault(); searchSubmit.current?.focus();}
                    else if (event.key === "Enter" && query.trim()) {
                        event.preventDefault(); submitGlobalSearch();
                    }
                }
                const [paneOpen, setPaneOpen] = pp.state(true);
                const [presence, setPresence] = pp.state("online");
                const [survey, setSurvey] = pp.state(1);
                const [primaryUse, setPrimaryUse] = pp.state("");
                const [category, setCategory] = pp.state("");
                const [saving, setSaving] = pp.state(false);
                const [saveError, setSaveError] = pp.state("");

                const PRESENCE = [
                    {key: "online", label: "Available", dot: "bg-online"},
                    {key: "busy", label: "Busy", dot: "bg-busy"},
                    {key: "away", label: "Be right back", dot: "bg-away"},
                    {key: "offline", label: "Appear offline", dot: "bg-offline"},
                ];
                const PRESENCE_COLORS = {
                    online: "bg-online",
                    busy: "bg-busy",
                    away: "bg-away",
                    offline: "bg-offline",
                };
                const PRIMARY_USES = [
                    {emoji: "📝", label: "Freelancing or consulting"},
                    {emoji: "📚", label: "Learning or teaching"},
                    {emoji: "💼", label: "Work or career"},
                    {emoji: "🫂", label: "Organized groups"},
                    {emoji: "💜", label: "Personal use"},
                    {emoji: "", label: "None of these"},
                ];
                const CATEGORIES = [
                    "Family and friends", "Group projects", "Individual projects",
                    "Hobbies", "Events", "Gaming or recreation",
                ];
                const summary = primaryUse && category ? primaryUse + " · " + category : "";

                function togglePane() {
                    setPaneOpen((open) => !open);
                }

                function goBack() {
                    if (searchView) {clearGlobalSearch(); return;}
                    window.history.back();
                }

                function goForward() {
                    window.history.forward();
                }

                const pickPresence = pp.event(function pickPresence(value) {
                    setPresence(value);
                });

                function openSurvey() {
                    setSaveError("");
                    setSurvey(1);
                    setSurveyActive(true);
                    surveyOpener.current?.click();
                }

                const pickUse = pp.event(function pickUse(value) {
                    setPrimaryUse(value);
                    setSaveError("");
                    setSurvey(2);
                });

                const pickCategory = pp.event(async function pickCategory(value) {
                    if (saving) return;
                    setCategory(value);
                    setSaving(true);
                    setSaveError("");
                    try {
                        await pp.rpc("save_usage_survey", { primary_use: primaryUse, category: value });
                        setSurvey(3);
                    } catch (failure) {
                        setSaveError(failure.message || "Your responses could not be saved.");
                    } finally {
                        setSaving(false);
                    }
                });
            "##</script>
        </div>
    }
}

// What the browser gets back once a survey answer is accepted.
#[derive(Serialize)]
pub struct SurveyReceipt {
    pub primary_use: String,
    pub category: String,
}

/// Record the two answers the usage survey collects.
///
/// Nothing is stored yet — the point of the round trip is that the answers are
/// checked in Rust, so an empty or over-long value is a 422 with the offending
/// field named rather than something the browser decides for itself.
#[rpc(auth)]
pub async fn save_usage_survey(primary_use: String, category: String) -> Result<SurveyReceipt> {
    let mut validator = Validator::new();
    validator
        .required("primary_use", &primary_use)
        .required("category", &category)
        .max_chars("primary_use", &primary_use, 60)
        .max_chars("category", &category, 60);
    validator.finish()?;

    Ok(SurveyReceipt {
        primary_use,
        category,
    })
}
