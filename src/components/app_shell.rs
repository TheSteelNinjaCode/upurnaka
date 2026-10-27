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
                    <div class="relative ml-2 w-full max-w-[720px]">
                        <InputGroup class="h-8! border-transparent bg-raised dark:bg-raised">
                            <InputGroupAddon>
                                <Icon name="search" class="size-4" />
                            </InputGroupAddon>
                            <input
                                type="text"
                                data-slot="input-group-control"
                                class=@{search_input.as_str()}
                                placeholder="Press Ctrl+G to go right to a chat or channel"
                                aria-label="Search"
                                pp-ref={globalSearch}
                                value={query}
                                oninput={setQuery(target.value)}
                                onkeydown={handleGlobalSearchKey(event)}
                            />
                        </InputGroup>
                        <div
                            class="absolute left-0 right-0 top-10 rounded-lg border bg-popover p-1 text-popover-foreground shadow-xl"
                            hidden={!query.trim()}
                            aria-label="Search results"
                        >
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
                            <p
                                class="px-3 py-2 text-sm text-muted-foreground"
                                hidden={searchResults.length > 0}
                            >
                                "No matching chats or apps."
                            </p>
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
                            <DropdownMenuItem>
                                <Icon name="settings" class="size-4 text-muted-foreground" />
                                "Settings"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
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
                            <DropdownMenuItem onclick={openSurvey()}>
                                <Icon name="sparkle" class="size-4 text-brand-ink" />
                                "Tell us how you use Teams"
                            </DropdownMenuItem>
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
                <shortcutcontext.provider value={pageShortcuts}>
                    <slot />
                </shortcutcontext.provider>
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
                const pageShortcuts = pp.ref(null);
                const globalSearch = pp.ref(null);
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
                    pageShortcuts.current?.handleKey?.(event);
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
                const searchTerm = query.trim().replace(/^\//, "").toLocaleLowerCase();
                const searchResults = [
                    {label: @{user.name.as_str()} + " (You)", href:"/chat"},
                    {label:"Meet", href:"/meet"}, {label:"People", href:"/people"},
                    {label:"Communities", href:"/communities"}, {label:"Calendar", href:"/calendar"},
                    {label:"Activity", href:"/activity"}, {label:"Apps", href:"/apps"}
                ].filter(result => result.label.toLocaleLowerCase().includes(searchTerm) || result.href.slice(1).includes(searchTerm));
                function visitSearchResult(href) {setQuery(""); pp.redirect(href);}
                function handleGlobalSearchKey(event) {
                    if (event.isComposing) return;
                    if (event.key === "Escape") {event.preventDefault(); setQuery("");}
                    else if (event.key === "Enter" && searchResults.length && query.trim()) {
                        event.preventDefault(); visitSearchResult(searchResults[0].href);
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
                    window.history.back();
                }

                function goForward() {
                    window.history.forward();
                }

                function pickPresence(value) {
                    setPresence(value);
                }

                function openSurvey() {
                    setSaveError("");
                    setSurvey(1);
                    setSurveyActive(true);
                    surveyOpener.current?.click();
                }

                function pickUse(value) {
                    setPrimaryUse(value);
                    setSaveError("");
                    setSurvey(2);
                }

                async function pickCategory(value) {
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
                }
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
