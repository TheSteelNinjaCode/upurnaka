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
const RAIL: [(&str, &str, &str); 6] = [
    ("chat", "Chat", "/chat"),
    ("meet", "Meet", "/meet"),
    ("people", "People", "/people"),
    ("communities", "Communities", "/communities"),
    ("calendar", "Calendar", "/calendar"),
    ("activity", "Activity", "/activity"),
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

/// Search public directory names and the caller's own uploaded files only.
#[rpc(auth)]
pub async fn search_workspace(
    query: String,
    file_type: Option<String>,
    file_date: Option<String>,
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
            ],
        )
        .one_of(
            "file_date",
            &file_date.as_str(),
            &["any", "day", "week", "month"],
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
    let days = match file_date.as_str() {
        "day" => 1,
        "week" => 7,
        "month" => 30,
        _ => 0,
    };
    if days > 0 {
        file_query = file_query.filter(
            chat_file::Column::CreatedAt
                .gte(crate::rahti::time::Utc::now().timestamp() - days * 86400),
        );
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
                            class="global-search-input h-8! border-transparent bg-raised dark:bg-raised"
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
                    class="global-search-view flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-tl-xl border-l border-t border-line bg-surface"
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
                                    class="global-search-tab shrink-0 rounded-full border-2 border-transparent bg-sunken px-3.5 py-1.5 text-[13px] font-semibold hover:bg-hover aria-selected:border-brand-ink aria-selected:bg-brand-soft aria-selected:text-brand-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
                                    onclick={setSearchTab(tab.id)}
                                    onkeydown={searchTabKey(event)}
                                >
                                    {tab.label}
                                </button>
                            </template>
                        </div>
                    </header>
                    <div
                        class="global-search-filters shrink-0 border-b border-line px-5"
                        hidden={searchTab !== "messages" && searchTab !== "files"}
                        aria-label="Search filters"
                    >
                        <div
                            class="mx-auto flex h-10 w-full max-w-[760px] items-center gap-2"
                            hidden={searchTab !== "messages"}
                        >
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="search-from"
                                    class="global-search-filter-trigger"
                                    attrs=@{labelled("Filter messages by sender")}
                                >
                                    <span>
                                        {messageFilters.from === "all" ? "From" : "From: " + senderLabel(messageFilters.from)}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                    menu="search-from"
                                    align="start"
                                    class="w-60"
                                    ontoggle={searchFilterToggled("from", event.newState)}
                                >
                                    <template pp-for="sender in messageSenders">
                                        <DropdownMenuItem
                                            key={sender.id}
                                            onclick={setMessageFilter("from", sender.id)}
                                        >
                                            <span class="flex-1">{sender.label}</span>
                                            <span hidden={messageFilters.from !== sender.id}>
                                                <Icon name="check" class="size-4" />
                                            </span>
                                        </DropdownMenuItem>
                                    </template>
                                </DropdownMenuContent>
                            </DropdownMenu>
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="search-message-date"
                                    class="global-search-filter-trigger"
                                    attrs=@{labelled("Filter messages by date")}
                                >
                                    <span>
                                        {messageFilters.date === "any" ? "Date" : dateLabel(messageFilters.date)}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                    menu="search-message-date"
                                    align="start"
                                    class="w-52"
                                    ontoggle={searchFilterToggled("message-date", event.newState)}
                                >
                                    <template pp-for="option in searchDateOptions">
                                        <DropdownMenuItem
                                            key={option.id}
                                            onclick={setMessageFilter("date", option.id)}
                                        >
                                            <span class="flex-1">{option.label}</span>
                                            <span hidden={messageFilters.date !== option.id}>
                                                <Icon name="check" class="size-4" />
                                            </span>
                                        </DropdownMenuItem>
                                    </template>
                                </DropdownMenuContent>
                            </DropdownMenu>
                            <Popover>
                                <PopoverTrigger
                                    name="search-message-more"
                                    class="global-search-filter-trigger"
                                    attrs=@{labelled("More message filters")}
                                >
                                    {messageFilters.attachments || messageFilters.important ? "More · " + (Number(messageFilters.attachments) + Number(messageFilters.important)) : "More"}
                                </PopoverTrigger>
                                <PopoverContent
                                    name="search-message-more"
                                    side="bottom"
                                    align="start"
                                    class="w-60 p-2"
                                    attrs=@{named("More message filters")}
                                    ontoggle={searchFilterToggled("more", event.newState)}
                                >
                                    <label
                                        class="flex cursor-pointer items-center gap-3 rounded px-3 py-2 text-[13px] hover:bg-hover"
                                    >
                                        <input
                                            type="checkbox"
                                            checked={messageFilters.attachments}
                                            onchange={setMessageFilter("attachments", target.checked)}
                                        />
                                        "Has attachments"
                                    </label>
                                    <label
                                        class="flex cursor-pointer items-center gap-3 rounded px-3 py-2 text-[13px] hover:bg-hover"
                                    >
                                        <input
                                            type="checkbox"
                                            checked={messageFilters.important}
                                            onchange={setMessageFilter("important", target.checked)}
                                        />
                                        "Important"
                                    </label>
                                    <button
                                        type="button"
                                        class="mt-1 w-full rounded px-3 py-2 text-left text-[13px] text-link hover:bg-hover"
                                        onclick={resetMessageFilters()}
                                    >
                                        "Reset message filters"
                                    </button>
                                </PopoverContent>
                            </Popover>
                            <button
                                type="button"
                                class="global-search-filter-trigger text-link"
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
                                    class="global-search-filter-trigger"
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
                                    class="w-52"
                                    ontoggle={searchFilterToggled("file-type", event.newState)}
                                >
                                    <template pp-for="option in searchFileTypes">
                                        <DropdownMenuItem
                                            key={option.id}
                                            onclick={setFileFilter("type", option.id)}
                                        >
                                            <span class="flex-1">{option.label}</span>
                                            <span hidden={fileFilters.type !== option.id}>
                                                <Icon name="check" class="size-4" />
                                            </span>
                                        </DropdownMenuItem>
                                    </template>
                                </DropdownMenuContent>
                            </DropdownMenu>
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="search-file-owner"
                                    class="global-search-filter-trigger"
                                    attrs=@{labelled("Filter files by sender")}
                                >
                                    <span>
                                        {fileFilters.owner === "all" ? "Shared by" : "Shared by: You"}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                    menu="search-file-owner"
                                    align="start"
                                    class="w-52"
                                    ontoggle={searchFilterToggled("file-owner", event.newState)}
                                >
                                    <DropdownMenuItem onclick={setFileFilter("owner", "all")}>
                                        <span class="flex-1">"Anyone"</span>
                                        <span hidden={fileFilters.owner !== "all"}>
                                            <Icon name="check" class="size-4" />
                                        </span>
                                    </DropdownMenuItem>
                                    <DropdownMenuItem onclick={setFileFilter("owner", "self")}>
                                        <span class="flex-1">"You"</span>
                                        <span hidden={fileFilters.owner !== "self"}>
                                            <Icon name="check" class="size-4" />
                                        </span>
                                    </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="search-file-date"
                                    class="global-search-filter-trigger"
                                    attrs=@{labelled("Filter files by date")}
                                >
                                    <span>
                                        {fileFilters.date === "any" ? "Date" : dateLabel(fileFilters.date)}
                                    </span>
                                    <Icon name="chevron-down" class="size-3" />
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                    menu="search-file-date"
                                    align="start"
                                    class="w-52"
                                    ontoggle={searchFilterToggled("file-date", event.newState)}
                                >
                                    <template pp-for="option in searchDateOptions">
                                        <DropdownMenuItem
                                            key={option.id}
                                            onclick={setFileFilter("date", option.id)}
                                        >
                                            <span class="flex-1">{option.label}</span>
                                            <span hidden={fileFilters.date !== option.id}>
                                                <Icon name="check" class="size-4" />
                                            </span>
                                        </DropdownMenuItem>
                                    </template>
                                </DropdownMenuContent>
                            </DropdownMenu>
                            <button
                                type="button"
                                class="global-search-filter-trigger text-link"
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
                                        class="global-search-card flex min-h-[72px] w-full items-center gap-3 rounded-lg border border-line bg-raised px-4 py-3 text-left hover:bg-hover focus-visible:outline-2 focus-visible:outline-ring"
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
            <style>r##"
                .global-search-input[data-slot=input-group] { box-shadow:none; }
                .dark .global-search-input[data-slot=input-group] { background:#292929; border-color:#3d3d3d; }
                .dark .global-search-view { background:#1f1f1f; }
                .dark .global-search-card { background:#292929; border-color:#171717; }
                .dark .global-search-card:hover { background:#333; }
                .global-search-view [role=tablist] { overflow-x:auto; padding:2px; }
                .global-search-tab { height:36px; padding:0 14px; line-height:20px; }
                .global-search-filter-trigger { display:inline-flex; height:40px; align-items:center; gap:4px; border-radius:3px; padding:0 12px; font-size:13px; color:var(--color-muted-foreground); white-space:nowrap; }
                .global-search-filter-trigger:hover { background:var(--color-hover); color:var(--color-ink); }
                .global-search-filter-trigger:focus-visible { outline:2px solid var(--color-brand-ink); outline-offset:-2px; }
                .global-search-filters > div { overflow-x:auto; }
                .dark .global-search-header { border-color:#141414; }
                .dark .global-search-filters { border-color:#383838; }
                .dark .global-search-tab { background:#141414; color:#f5f5f5; }
                .dark .global-search-tab:hover { background:#2f2f2f; }
                .dark .global-search-tab[aria-selected=true] { background:#3e3d6c; border-color:#deddea; color:#b7b6f5; }
            "##</style>
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
                    if (document.activeElement === document.body) moreBadge.current?.closest("button")?.focus();
                }
                // Document-level keyboard integration must also receive keys when
                // the body has focus. Keep ordinary controls in their markup.
                const handleGlobalKey = pp.event(event => {
                    if (event.defaultPrevented || event.isComposing) return;
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
                const [messageFilters, setMessageFilters] = pp.state({from:'all',date:'any',attachments:false,important:false});
                const [fileFilters, setFileFilters] = pp.state({type:'all',owner:'all',date:'any'});
                const [openSearchFilters, setOpenSearchFilters] = pp.state({});
                const searchDateOptions = [{id:'any',label:'Any time'},{id:'day',label:'Past 24 hours'},{id:'week',label:'Past 7 days'},{id:'month',label:'Past 30 days'}];
                const searchFileTypes = [{id:'all',label:'All types'},{id:'images',label:'Images'},{id:'pdf',label:'PDF'},{id:'word',label:'Word'},{id:'excel',label:'Excel'},{id:'powerpoint',label:'PowerPoint'},{id:'text',label:'Text'}];
                const messageFiltersActive = messageFilters.from !== 'all' || messageFilters.date !== 'any' || messageFilters.attachments || messageFilters.important;
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
                const visibleSearchRows = [...remoteSearchRows,...localRows].filter(row => {
                    if (row.kind !== searchTab) return false;
                    if (row.kind === 'messages') return (messageFilters.from === 'all' || row.authorId === messageFilters.from) && matchesSearchDate(row.sentAt,messageFilters.date) && (!messageFilters.attachments || row.hasAttachments) && (!messageFilters.important || row.important);
                    if (row.kind === 'files') return fileFilters.owner === 'all' || row.owner === 'self';
                    return true;
                });
                function matchesSearchDate(timestamp,period) {const days={day:1,week:7,month:30}[period]; return !days || (Number.isFinite(timestamp) && timestamp >= Date.now()/1000-days*86400);}
                function senderLabel(id) {return messageSenders.find(sender=>sender.id===id)?.label || 'Selected person';}
                function dateLabel(id) {return searchDateOptions.find(option=>option.id===id)?.label || 'Date';}
                function fileTypeLabel(id) {return searchFileTypes.find(option=>option.id===id)?.label || 'File Type';}
                function searchFilterToggled(id,state) {setOpenSearchFilters(previous=>({...previous,[id]:state==='open'}));}
                function setMessageFilter(key,value) {setMessageFilters(previous=>({...previous,[key]:value}));}
                function setFileFilter(key,value) {
                    setFileFilters(previous=>({...previous,[key]:value}));
                    if (key !== 'owner') {setRemoteSearchLoading(true); setRemoteSearchRows(previous=>previous.filter(row=>row.kind!=='files'));}
                }
                function resetMessageFilters() {setMessageFilters({from:'all',date:'any',attachments:false,important:false});}
                function resetFileFilters() {setFileFilters({type:'all',owner:'all',date:'any'});}
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
                            const response = await pp.rpc('search_workspace',{query:searchTerm,file_type:fileFilters.type,file_date:fileFilters.date});
                            if (!active) return;
                            const people = response.people.map(person => ({...person,key:'person:'+person.id,kind:'people',subtitle:person.is_self ? 'You' : ''}));
                            const files = response.files.map(file => ({key:'file:'+file.id,id:file.id,kind:'files',title:file.name,subtitle:file.uploaded,owner:'self'}));
                            setRemoteSearchRows([...people,...files]);
                        } catch (failure) {if (active) setRemoteSearchError(failure.message || 'Search is unavailable. Please try again.');}
                        finally {if (active) setRemoteSearchLoading(false);}
                    },200);
                    return () => {active=false; clearTimeout(timer);};
                },[searchView,searchTerm,searchRetry,fileFilters.type,fileFilters.date]);
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
                    {label:"Activity", href:"/activity"}, {label:"Apps", href:"/apps"},
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
