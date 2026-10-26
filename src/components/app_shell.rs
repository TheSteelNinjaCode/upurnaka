//! The desktop window: title bar, activity rail, and the frame every section
//! renders inside.
//!
//! A layout cannot see the request, so the rail's selected item is a prop:
//! each `page.rs` roots itself in `<AppShell active="…">` and names its own
//! section. The page's content is the shell's `children`, and lands at the
//! `<slot />` beside the rail.

use serde::Serialize;

use crate::components::icon::Icon;
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
        "group relative flex h-11 w-11 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
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

/// The window frame. `active` is the rail key of the section being shown.
#[component]
pub fn AppShell(active: &str, children: Html) -> Html {
    let rail = Html::concat(
        RAIL.iter()
            .map(|(key, label, href)| RailLink(rail_icon(key), label, href, *key == active)),
    );

    let apps_class = if active == "apps" {
        "flex h-11 w-11 items-center justify-center rounded-md bg-raised text-brand-ink"
    } else {
        "flex h-11 w-11 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
    };

    html! {
        <div class="ambient relative flex h-screen w-full flex-col overflow-hidden text-ink">
            <header class="relative z-40 flex h-12 shrink-0 items-center gap-1 px-2.5">
                <a
                    href="/chat"
                    class="ml-2 mr-1 flex h-8 w-8 items-center justify-center rounded-md hover:bg-hover"
                    title="Teams"
                    aria-label="Teams"
                >
                    <img src="/images/upurnaka-logo.png" alt="" class="h-[22px] w-[22px]" />
                </a>
                <button
                    type="button"
                    class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                    title="Expand or collapse the list pane"
                    aria-label="Expand or collapse the list pane"
                    onclick={togglePane()}
                >
                    <Icon name="panel" class="h-[18px] w-[18px]" />
                </button>
                <div class="flex min-w-0 flex-1 items-center justify-center gap-1">
                    <button
                        type="button"
                        class="flex h-8 w-8 items-center justify-center rounded-md text-subtle transition-colors hover:bg-hover hover:text-ink"
                        title="Back"
                        aria-label="Back"
                        onclick={goBack()}
                    >
                        <Icon name="chevron-left" class="h-4 w-4" />
                    </button>
                    <button
                        type="button"
                        class="flex h-8 w-8 items-center justify-center rounded-md text-subtle transition-colors hover:bg-hover hover:text-ink"
                        title="Forward"
                        aria-label="Forward"
                        onclick={goForward()}
                    >
                        <Icon name="chevron-right" class="h-4 w-4" />
                    </button>
                    <div class="relative ml-2 w-full max-w-[720px]">
                        <Icon
                            name="search"
                            class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted"
                        />
                        <input
                            type="text"
                            class="h-8 w-full rounded-md border border-transparent bg-raised pl-9 pr-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                            placeholder="Press Ctrl+G to go right to a chat or channel"
                            aria-label="Search"
                            value={query}
                            oninput={setQuery(target.value)}
                        />
                    </div>
                </div>
                <div class="relative flex items-center gap-1">
                    <button
                        type="button"
                        class="relative flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                        title="Settings and more"
                        aria-label="Settings and more"
                        aria-expanded={menu === "more"}
                        onclick={toggleMenu("more")}
                    >
                        <Icon name="more" class="h-[18px] w-[18px]" />
                        <span
                            class="absolute right-1.5 top-1 h-1.5 w-1.5 rounded-full bg-brand-ink"
                        >
                        </span>
                    </button>
                    <button
                        type="button"
                        class="relative ml-1 flex h-8 w-8 items-center justify-center rounded-full"
                        title="Jefferson Abraham"
                        aria-label="Your profile"
                        aria-expanded={menu === "profile"}
                        onclick={toggleMenu("profile")}
                    >
                        <span
                            class="flex h-8 w-8 items-center justify-center rounded-full bg-avatar text-xs font-semibold text-avatar-ink"
                        >
                            "JA"
                        </span>
                        <span
                            class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-chrome"
                            pp-style={presenceStyle}
                        >
                        </span>
                    </button>
                    <div
                        class="absolute right-10 top-9 w-72 rounded-lg border border-edge bg-raised p-1 shadow-2xl shadow-black/60"
                        hidden={menu !== "more"}
                    >
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                            onclick={closeMenus()}
                        >
                            <Icon name="settings" class="h-4 w-4 text-muted" />
                            "Settings"
                        </button>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                            onclick={closeMenus()}
                        >
                            <Icon name="bell" class="h-4 w-4 text-muted" />
                            "Notifications"
                        </button>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                            onclick={closeMenus()}
                        >
                            <Icon name="format" class="h-4 w-4 text-muted" />
                            "Keyboard shortcuts"
                        </button>
                        <div class="my-1 h-px bg-line"></div>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                            onclick={openSurvey()}
                        >
                            <Icon name="sparkle" class="h-4 w-4 text-brand-ink" />
                            "Tell us how you use Teams"
                        </button>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                            onclick={closeMenus()}
                        >
                            <Icon name="check-circle" class="h-4 w-4 text-muted" />
                            "About"
                        </button>
                    </div>
                    <div
                        class="absolute right-0 top-9 w-80 rounded-lg border border-edge bg-raised p-1 shadow-2xl shadow-black/60"
                        hidden={menu !== "profile"}
                    >
                        <div class="flex items-center gap-3 px-3 py-3">
                            <span
                                class="flex h-11 w-11 items-center justify-center rounded-full bg-avatar text-sm font-semibold text-avatar-ink"
                            >
                                "JA"
                            </span>
                            <span class="min-w-0">
                                <span class="block truncate text-[13px] font-semibold text-ink">
                                    "Jefferson Abraham"
                                </span>
                                <span class="block truncate text-xs text-muted">
                                    "abrahamjefferson38@gmail.com"
                                </span>
                            </span>
                        </div>
                        <div class="my-1 h-px bg-line"></div>
                        <p
                            class="px-3 pb-1 pt-2 text-[11px] font-semibold uppercase tracking-wide text-subtle"
                        >
                            "Presence"
                        </p>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] hover:bg-hover"
                            onclick={pickPresence("online")}
                        >
                            <span class="h-2.5 w-2.5 rounded-full bg-online"></span>
                            "Available"
                            <Icon name="check" class="ml-auto h-4 w-4 text-brand-ink" />
                        </button>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] hover:bg-hover"
                            onclick={pickPresence("busy")}
                        >
                            <span class="h-2.5 w-2.5 rounded-full bg-busy"></span>
                            "Busy"
                        </button>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] hover:bg-hover"
                            onclick={pickPresence("away")}
                        >
                            <span class="h-2.5 w-2.5 rounded-full bg-away"></span>
                            "Be right back"
                        </button>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] hover:bg-hover"
                            onclick={pickPresence("offline")}
                        >
                            <span class="h-2.5 w-2.5 rounded-full bg-offline"></span>
                            "Appear offline"
                        </button>
                        <div class="my-1 h-px bg-line"></div>
                        <button
                            type="button"
                            class="flex w-full items-center gap-3 rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                            onclick={closeMenus()}
                        >
                            <Icon name="person-add" class="h-4 w-4 text-muted" />
                            "Set a status message"
                        </button>
                    </div>
                </div>
            </header>
            <div class={paneOpen ? "flex min-h-0 flex-1" : "flex min-h-0 flex-1 panes-collapsed"}>
                <nav
                    class="flex w-[68px] shrink-0 flex-col items-center gap-1 pb-3"
                    aria-label="Teams sections"
                >
                    @{rail}
                    <span class="flex-1"></span>
                    <a href="/apps" class=@{apps_class} title="Apps" aria-label="Apps">
                        <Icon name="apps" class="h-[22px] w-[22px]" />
                    </a>
                </nav>
                <slot />
            </div>
            <div class="absolute inset-0 z-30" hidden={menu === ""} onclick={closeMenus()}></div>
            <div
                class="absolute inset-0 z-50 flex items-center justify-center bg-black/60 px-6"
                hidden={survey === 0}
            >
                <div
                    class="w-full max-w-[600px] rounded-lg border border-edge bg-raised px-10 py-9 shadow-2xl shadow-black/70"
                >
                    <div hidden={survey !== 1}>
                        <h2 class="text-center text-xl font-semibold text-ink">
                            "What is your primary use for Teams?"
                        </h2>
                        <div class="mt-8 grid grid-cols-2 gap-4">
                            <button
                                type="button"
                                class="flex h-[94px] flex-col items-center justify-center gap-2 rounded-md border border-edge bg-surface px-3 text-center text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickUse("Freelancing or consulting")}
                            >
                                <span class="text-2xl leading-none">"📝"</span>
                                "Freelancing or consulting"
                            </button>
                            <button
                                type="button"
                                class="flex h-[94px] flex-col items-center justify-center gap-2 rounded-md border border-edge bg-surface px-3 text-center text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickUse("Learning or teaching")}
                            >
                                <span class="text-2xl leading-none">"📚"</span>
                                "Learning or teaching"
                            </button>
                            <button
                                type="button"
                                class="flex h-[94px] flex-col items-center justify-center gap-2 rounded-md border border-edge bg-surface px-3 text-center text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickUse("Work or career")}
                            >
                                <span class="text-2xl leading-none">"💼"</span>
                                "Work or career"
                            </button>
                            <button
                                type="button"
                                class="flex h-[94px] flex-col items-center justify-center gap-2 rounded-md border border-edge bg-surface px-3 text-center text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickUse("Organized groups")}
                            >
                                <span class="text-2xl leading-none">"🫂"</span>
                                "Organized groups"
                            </button>
                            <button
                                type="button"
                                class="flex h-[94px] flex-col items-center justify-center gap-2 rounded-md border border-edge bg-surface px-3 text-center text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickUse("Personal use")}
                            >
                                <span class="text-2xl leading-none">"💜"</span>
                                "Personal use"
                            </button>
                            <button
                                type="button"
                                class="flex h-[94px] flex-col items-center justify-center gap-2 rounded-md border border-edge bg-surface px-3 text-center text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickUse("None of these")}
                            >
                                "None of these"
                            </button>
                        </div>
                        <p class="mt-8 text-center">
                            <button type="button" class="text-[13px] text-link hover:underline">
                                "Learn more"
                            </button>
                        </p>
                    </div>
                    <div hidden={survey !== 2}>
                        <h2 class="text-center text-xl font-semibold text-ink">
                            "Choose a category"
                        </h2>
                        <div class="mt-7 flex flex-col gap-3">
                            <button
                                type="button"
                                class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickCategory("Family and friends")}
                            >
                                "Family and friends"
                            </button>
                            <button
                                type="button"
                                class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickCategory("Group projects")}
                            >
                                "Group projects"
                            </button>
                            <button
                                type="button"
                                class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickCategory("Individual projects")}
                            >
                                "Individual projects"
                            </button>
                            <button
                                type="button"
                                class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickCategory("Hobbies")}
                            >
                                "Hobbies"
                            </button>
                            <button
                                type="button"
                                class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickCategory("Events")}
                            >
                                "Events"
                            </button>
                            <button
                                type="button"
                                class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                onclick={pickCategory("Gaming or recreation")}
                            >
                                "Gaming or recreation"
                            </button>
                        </div>
                        <p class="mt-4 text-center text-xs text-busy" hidden={!saveError}>
                            {saveError}
                        </p>
                        <div class="mt-7 flex items-center justify-between">
                            <button
                                type="button"
                                class="h-8 w-24 rounded border border-edge bg-surface text-[13px] font-semibold text-ink hover:bg-hover"
                                onclick={setSurvey(1)}
                            >
                                "Back"
                            </button>
                            <button type="button" class="text-[13px] text-link hover:underline">
                                "Learn more"
                            </button>
                            <span class="w-24"></span>
                        </div>
                    </div>
                    <div hidden={survey !== 3}>
                        <p class="mt-4 text-center text-5xl leading-none">"🎉"</p>
                        <h2 class="mt-7 text-center text-xl font-semibold text-ink">
                            "Thank you!"
                        </h2>
                        <p class="mt-2 text-center text-[13px] text-muted">
                            "Your responses were saved."
                        </p>
                        <p class="mt-2 text-center text-xs text-subtle">{summary}</p>
                        <div class="mt-9 flex items-center justify-between">
                            <button
                                type="button"
                                class="h-8 w-24 rounded border border-edge bg-surface text-[13px] font-semibold text-ink hover:bg-hover"
                                onclick={setSurvey(2)}
                            >
                                "Back"
                            </button>
                            <button type="button" class="text-[13px] text-link hover:underline">
                                "Learn more"
                            </button>
                            <button
                                type="button"
                                class="h-8 w-24 rounded border border-edge bg-surface text-[13px] font-semibold text-ink hover:bg-hover"
                                onclick={closeSurvey()}
                            >
                                "Close"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
            <script>r##"
                const [query, setQuery] = pp.state("");
                const [paneOpen, setPaneOpen] = pp.state(true);
                const [menu, setMenu] = pp.state("");
                const [presence, setPresence] = pp.state("online");
                const [survey, setSurvey] = pp.state(0);
                const [primaryUse, setPrimaryUse] = pp.state("");
                const [category, setCategory] = pp.state("");
                const [saving, setSaving] = pp.state(false);
                const [saveError, setSaveError] = pp.state("");

                const PRESENCE_COLORS = {
                    online: "#6bb700",
                    busy: "#c4314b",
                    away: "#eaa300",
                    offline: "#8a8886",
                };

                const presenceStyle = "background-color: " + PRESENCE_COLORS[presence];
                const summary = primaryUse && category ? primaryUse + " · " + category : "";

                function togglePane() {
                    setPaneOpen((open) => !open);
                }

                function toggleMenu(name) {
                    setMenu((open) => (open === name ? "" : name));
                }

                function closeMenus() {
                    setMenu("");
                }

                function goBack() {
                    window.history.back();
                }

                function goForward() {
                    window.history.forward();
                }

                function pickPresence(value) {
                    setPresence(value);
                    setMenu("");
                }

                function openSurvey() {
                    setMenu("");
                    setSaveError("");
                    setSurvey(1);
                }

                function closeSurvey() {
                    setSurvey(0);
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

/// What the browser gets back once a survey answer is accepted.
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
#[rpc]
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
