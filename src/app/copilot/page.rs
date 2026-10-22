//! Copilot: the assistant pane, before an assistant is connected to it.

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::rahti::{Html, Metadata, html};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Copilot")
        .description("Ask Copilot about your chats, meetings and files.")
}

pub async fn page() -> Html {
    html! {
        <AppShell active="copilot">
            <main
                class="flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <header class="flex h-14 shrink-0 items-center gap-3 px-8">
                    <Icon name="copilot" class="h-6 w-6" />
                    <h1 class="text-[15px] font-semibold text-ink">"Copilot"</h1>
                </header>
                <div class="scroll-thin flex min-h-0 flex-1 flex-col overflow-y-auto px-8">
                    <div class="m-auto flex max-w-md flex-col items-center py-10 text-center">
                        <Icon name="copilot" class="h-16 w-16" />
                        <h2 class="mt-6 text-[20px] font-semibold text-ink">
                            "Ask Copilot anything"
                        </h2>
                        <p class="mt-2.5 text-[13px] leading-relaxed text-muted">
                            "Catch up on a chat, draft a reply, or summarise a meeting. Copilot answers from the conversations you already have access to."
                        </p>
                        <div class="mt-7 flex flex-wrap items-center justify-center gap-2">
                            <button
                                type="button"
                                class="h-8 rounded-full border border-edge bg-raised px-4 text-xs text-ink transition-colors hover:bg-hover"
                                onclick={setDraft("Catch me up on my unread chats")}
                            >
                                "Catch me up on my unread chats"
                            </button>
                            <button
                                type="button"
                                class="h-8 rounded-full border border-edge bg-raised px-4 text-xs text-ink transition-colors hover:bg-hover"
                                onclick={setDraft("Draft a reply to my last message")}
                            >
                                "Draft a reply to my last message"
                            </button>
                            <button
                                type="button"
                                class="h-8 rounded-full border border-edge bg-raised px-4 text-xs text-ink transition-colors hover:bg-hover"
                                onclick={setDraft("What is on my calendar this week?")}
                            >
                                "What is on my calendar this week?"
                            </button>
                        </div>
                        <p class="mt-7 text-xs text-away" hidden={!notice}>{notice}</p>
                    </div>
                </div>
                <div class="shrink-0 px-8 pb-6 pt-2">
                    <form
                        class="mx-auto flex max-w-2xl items-center gap-2 rounded-lg border border-edge bg-raised px-3 py-1.5 focus-within:border-brand"
                        onsubmit={ask(event)}
                        novalidate
                    >
                        <input
                            name="prompt"
                            type="text"
                            class="h-9 min-w-0 flex-1 bg-transparent text-[13px] text-ink placeholder:text-muted focus:outline-none"
                            placeholder="Ask Copilot"
                            aria-label="Ask Copilot"
                            autocomplete="off"
                            value={draft}
                            oninput={setDraft(target.value)}
                        />
                        <button
                            type="submit"
                            class="flex h-8 w-8 items-center justify-center rounded text-brand-ink transition-colors hover:bg-hover disabled:text-subtle"
                            title="Send"
                            aria-label="Send"
                            disabled={draft.trim().length === 0}
                        >
                            <Icon name="send" class="h-[18px] w-[18px]" />
                        </button>
                    </form>
                </div>
                <script>r##"
                    const [draft, setDraft] = pp.state("");
                    const [notice, setNotice] = pp.state("");

                    function ask(event) {
                        event.preventDefault();
                        if (draft.trim().length === 0) return;
                        setNotice("Copilot is not connected to this workspace yet, so nothing was sent.");
                    }
                "##</script>
            </main>
        </AppShell>
    }
}
