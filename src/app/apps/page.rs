//! Apps: the store rail item, and the catalogue behind it.

use crate::components::app_shell::AppShell;
use crate::components::aria::named;
use crate::components::icon::Icon;
use crate::components::rahti_ui::button::Button;
use crate::components::rahti_ui::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::rahti_ui::toggle_group::{ToggleGroup, ToggleGroupItem};
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

/// A section in the apps pane: muted until it is the chosen one.
const NAV: &str = "h-9! w-full justify-start! gap-3 rounded-md! px-3! text-[13px] \
                   text-muted-foreground has-[:checked]:bg-raised! has-[:checked]:font-semibold \
                   has-[:checked]:text-ink!";

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Apps")
        .description("Apps you can add to this workspace.")
}

pub async fn page() -> Html {
    let cards = Html::concat(CATALOGUE.iter().map(|(name, publisher, blurb, tint)| {
        let tint_class = format!("h-20 bg-linear-to-br {tint}");
        html! {
            <li>
                <Card class="h-full gap-4 overflow-hidden bg-raised pt-0 pb-5">
                    <div class=@{tint_class}></div>
                    <CardHeader class="gap-1 px-5">
                        <CardTitle class="text-[14px]">@{name}</CardTitle>
                        <CardDescription class="text-xs text-subtle">@{publisher}</CardDescription>
                    </CardHeader>
                    <CardContent class="flex-1 px-5">
                        <p class="text-[13px] leading-relaxed text-muted-foreground">@{blurb}</p>
                    </CardContent>
                    <CardFooter class="px-5">
                        <Button variant="outline" size="sm" class="w-full text-xs font-semibold">
                            "Add"
                        </Button>
                    </CardFooter>
                </Card>
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
                    <ToggleGroup
                        r#type="single"
                        name="apps-section"
                        variant="default"
                        class="mt-3 w-full flex-col! items-stretch gap-1 px-4"
                        attrs=@{named("App sections")}
                    >
                        <ToggleGroupItem
                            r#type="single"
                            name="apps-section"
                            value="all"
                            pressed
                            class=@{NAV}
                            onchange={setSection(target.value)}
                        >
                            <Icon name="apps" class="size-[18px]" />
                            "All apps"
                        </ToggleGroupItem>
                        <ToggleGroupItem
                            r#type="single"
                            name="apps-section"
                            value="built"
                            class=@{NAV}
                            onchange={setSection(target.value)}
                        >
                            <Icon name="communities" class="size-[18px]" />
                            "Built for your org"
                        </ToggleGroupItem>
                        <ToggleGroupItem
                            r#type="single"
                            name="apps-section"
                            value="mine"
                            class=@{NAV}
                            onchange={setSection(target.value)}
                        >
                            <Icon name="check-circle" class="size-[18px]" />
                            "Your apps"
                        </ToggleGroupItem>
                    </ToggleGroup>
                </aside>
                <main
                    class="scroll-thin min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <div class="mx-auto w-full max-w-[1000px] px-12 py-9">
                        <h2 class="text-[22px] font-semibold tracking-tight text-ink">
                            "Apps for Teams"
                        </h2>
                        <p class="mt-2 text-[13px] text-muted-foreground">
                            "Add an app once and it shows up in every chat and channel you are in."
                        </p>
                        <ul class="mt-8 grid grid-cols-3 gap-5">@{cards}</ul>
                    </div>
                </main>
                <script>r##"
                    const [section, setSection] = pp.state("all");
                "##</script>
            </div>
        </AppShell>
    }
}
