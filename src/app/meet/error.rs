//! What a failed page under `/meet` shows.
//!
//! Mostly it shows one thing: a meeting link that has been revoked, or a join
//! code nobody ever minted. Without a boundary that answer is
//! `application/problem+json`, which is correct and useless to the person
//! holding the link — so the boundary renders it in the shell, with the way
//! back.

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::rahti::{ErrorInfo, Html, html};

pub fn error(error: ErrorInfo) -> Html {
    let reference = error.request_id().map(|id| format!("Reference: {id}"));

    html! {
        <AppShell active="meet">
            <main
                class="scroll-thin min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="mx-auto w-full max-w-[640px] px-14 py-20 text-center">
                    <span
                        class="inline-flex h-14 w-14 items-center justify-center rounded-full bg-brand-soft text-brand-ink"
                    >
                        <Icon name="link" class="h-7 w-7" />
                    </span>
                    <h1 class="mt-6 text-[22px] font-semibold tracking-tight text-ink">
                        @{error.reason()}
                    </h1>
                    <p class="mt-3 text-[13px] text-muted">@{error.message()}</p>
                    <p class="mt-8">
                        <a
                            href="/meet"
                            class="inline-flex h-10 items-center rounded-md bg-brand px-6 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                        >
                            "Back to Meet"
                        </a>
                    </p>
                    <p class="mt-6 text-xs text-subtle">@{reference}</p>
                </div>
            </main>
        </AppShell>
    }
}
