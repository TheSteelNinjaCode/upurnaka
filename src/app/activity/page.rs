//! Activity: the notification feed, and the empty state it starts in.

use crate::components::app_shell::AppShell;
use crate::components::aria::labelled;
use crate::components::icon::Icon;
use crate::components::rahti_ui::button::Button;
use crate::components::rahti_ui::toggle::Toggle;
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
                            <Toggle
                                size="sm"
                                class="px-0! text-muted-foreground has-[:checked]:text-brand-ink"
                                attrs=@{labelled("Show unread only")}
                                checked={unreadOnly}
                                onchange={setUnreadOnly(target.checked)}
                            >
                                <Icon name="filter" class="size-[18px]" />
                            </Toggle>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{labelled("Notification settings")}
                            >
                                <Icon name="settings" class="size-[18px]" />
                            </Button>
                        </div>
                    </div>
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-5 pt-5">
                        <p class="text-[13px] leading-relaxed text-muted-foreground">
                            "You will see @mentions, reactions and other notifications here."
                        </p>
                    </div>
                </aside>
                <main
                    class="flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                </main>
                <script>r##"
                    const [unreadOnly, setUnreadOnly] = pp.state(false);
                "##</script>
            </div>
        </AppShell>
    }
}
