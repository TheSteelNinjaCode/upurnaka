//! Apps: the store rail item, and the catalogue behind it.

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::rahti::{Html, Metadata, html};

/// What the catalogue offers, as `(name, publisher, blurb, tint)`.
const CATALOGUE: [(&str, &str, &str, &str); 6] = [
    (
        "Tasks",
        "upurnaka",
        "Plan work and track what is left.",
        "from-[#5b6ff0] to-[#3b3f9e]",
    ),
    (
        "Boards",
        "upurnaka",
        "A column per stage, a card per job.",
        "from-[#2fb3a3] to-[#1d6f68]",
    ),
    (
        "Forms",
        "upurnaka",
        "Collect answers without a thread.",
        "from-[#d4705a] to-[#8a3f31]",
    ),
    (
        "Notes",
        "upurnaka",
        "Shared pages beside the chat.",
        "from-[#9a6ff0] to-[#5b3fae]",
    ),
    (
        "Approvals",
        "upurnaka",
        "Ask once, decide once, keep the record.",
        "from-[#e0a63c] to-[#8f6618]",
    ),
    (
        "Polls",
        "upurnaka",
        "Settle a question in a message.",
        "from-[#4aa6e0] to-[#25618f]",
    ),
];

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Apps")
        .description("Apps you can add to this workspace.")
}

pub async fn page() -> Html {
    let cards = Html::concat(CATALOGUE.iter().map(|(name, publisher, blurb, tint)| {
        let tint_class = format!("h-20 bg-linear-to-br {tint}");
        html! {
            <li class="overflow-hidden rounded-xl border border-edge bg-raised">
                <div class=@{tint_class}></div>
                <div class="p-5">
                    <p class="text-[14px] font-semibold text-ink">@{name}</p>
                    <p class="mt-0.5 text-xs text-subtle">@{publisher}</p>
                    <p class="mt-3 text-[13px] leading-relaxed text-muted">@{blurb}</p>
                    <button
                        type="button"
                        class="mt-5 h-8 w-full rounded border border-edge text-xs font-semibold text-ink transition-colors hover:bg-hover"
                    >
                        "Add"
                    </button>
                </div>
            </li>
        }
    }));

    html! {
        <AppShell active="apps">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[300px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">"Apps"</h1>
                    </div>
                    <div class="mt-3 px-4">
                        <button
                            type="button"
                            class={section === "all" ? NAV_ON : NAV_OFF}
                            onclick={setSection("all")}
                        >
                            <Icon name="apps" class="h-[18px] w-[18px]" />
                            "All apps"
                        </button>
                        <button
                            type="button"
                            class={section === "built" ? NAV_ON : NAV_OFF}
                            onclick={setSection("built")}
                        >
                            <Icon name="communities" class="h-[18px] w-[18px]" />
                            "Built for your org"
                        </button>
                        <button
                            type="button"
                            class={section === "mine" ? NAV_ON : NAV_OFF}
                            onclick={setSection("mine")}
                        >
                            <Icon name="check-circle" class="h-[18px] w-[18px]" />
                            "Your apps"
                        </button>
                    </div>
                </aside>
                <main
                    class="scroll-thin min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <div class="mx-auto w-full max-w-[1000px] px-12 py-9">
                        <h2 class="text-[22px] font-semibold tracking-tight text-ink">
                            "Apps for Teams"
                        </h2>
                        <p class="mt-2 text-[13px] text-muted">
                            "Add an app once and it shows up in every chat and channel you are in."
                        </p>
                        <ul class="mt-8 grid grid-cols-3 gap-5">@{cards}</ul>
                    </div>
                </main>
                <script>r##"
                    const NAV_ON = "mt-1 flex h-9 w-full items-center gap-3 rounded-md bg-raised px-3 text-left text-[13px] font-semibold text-ink";
                    const NAV_OFF = "mt-1 flex h-9 w-full items-center gap-3 rounded-md px-3 text-left text-[13px] text-muted transition-colors hover:bg-hover hover:text-ink";

                    const [section, setSection] = pp.state("all");
                "##</script>
            </div>
        </AppShell>
    }
}
