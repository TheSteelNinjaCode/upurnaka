//! Activity: the notification feed, and the empty state it starts in.

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::rahti::{Html, Metadata, html};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Activity")
        .description("Mentions, reactions and other notifications.")
}

pub async fn page() -> Html {
    html! {
        <AppShell active="activity">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[350px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">
                            "Activity"
                        </h1>
                        <div class="flex items-center gap-0.5">
                            <button
                                type="button"
                                class={unreadOnly ? ICON_ON : ICON_OFF}
                                title="Show unread only"
                                aria-label="Show unread only"
                                aria-pressed={unreadOnly}
                                onclick={setUnreadOnly(on => !on)}
                            >
                                <Icon name="filter" class="h-[18px] w-[18px]" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Notification settings"
                                aria-label="Notification settings"
                            >
                                <Icon name="settings" class="h-[18px] w-[18px]" />
                            </button>
                        </div>
                    </div>
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-5 pt-5">
                        <p class="text-[13px] leading-relaxed text-muted">
                            "You will see @mentions, reactions and other notifications here."
                        </p>
                    </div>
                </aside>
                <main
                    class="flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                </main>
                <script>r##"
                    const ICON_ON = "flex h-8 w-8 items-center justify-center rounded-md bg-raised text-brand-ink";
                    const ICON_OFF = "flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink";

                    const [unreadOnly, setUnreadOnly] = pp.state(false);
                "##</script>
            </div>
        </AppShell>
    }
}
