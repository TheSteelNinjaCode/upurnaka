//! Meet: the three ways a meeting starts, and the links this account owns.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::rahti::time;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Meet")
        .description("Create a meeting link, schedule a meeting, or join with an ID.")
}

pub async fn page() -> Html {
    html! {
        <AppShell active="meet">
            <main
                class="scroll-thin min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="mx-auto w-full max-w-[960px] px-14 py-9">
                    <h1 class="text-[22px] font-semibold tracking-tight text-ink">"Meet"</h1>
                    <div class="mt-7 grid grid-cols-3 gap-4">
                        <button
                            type="button"
                            class="flex h-12 items-center justify-center gap-3 rounded-md bg-brand text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                            disabled={creating}
                            onclick={createLink()}
                        >
                            <Icon name="link" class="h-[18px] w-[18px]" />
                            {creating ? "Creating…" : "Create a meeting link"}
                        </button>
                        <a
                            href="/calendar"
                            class="flex h-12 items-center justify-center gap-3 rounded-md border border-edge bg-raised text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                        >
                            <Icon name="calendar-day" class="h-[18px] w-[18px]" />
                            "Schedule a meeting"
                        </a>
                        <button
                            type="button"
                            class="flex h-12 items-center justify-center gap-3 rounded-md border border-edge bg-raised text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                            onclick={setJoinOpen(open => !open)}
                        >
                            <Icon name="hash" class="h-[18px] w-[18px]" />
                            "Join with a meeting ID"
                        </button>
                    </div>
                    <div
                        class="mt-4 rounded-md border border-edge bg-raised px-5 py-4"
                        hidden={!joinOpen}
                    >
                        <form class="flex items-center gap-3" onsubmit={join(event)} novalidate>
                            <input
                                name="meeting"
                                type="text"
                                class="h-9 flex-1 rounded border border-edge bg-surface px-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                                placeholder="Enter a meeting ID"
                                aria-label="Meeting ID"
                                autocomplete="off"
                                value={meetingId}
                                oninput={setMeetingId(target.value)}
                            />
                            <button
                                type="submit"
                                class="h-9 rounded bg-brand px-5 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                disabled={meetingId.trim().length === 0}
                            >
                                "Join"
                            </button>
                        </form>
                        <p class="mt-2 text-xs text-muted" hidden={!joinNotice}>{joinNotice}</p>
                    </div>
                    <h2 class="mt-10 text-[15px] font-semibold text-ink">"Meeting links"</h2>
                    <div
                        class="mt-4 rounded-lg border border-edge bg-raised px-8 py-7"
                        hidden={links.length > 0}
                    >
                        <p class="text-3xl leading-none">"🔗"</p>
                        <p class="mt-5 text-[15px] font-semibold text-ink">
                            "Quickly create, save, and share links with anyone."
                        </p>
                        <p class="mt-5">
                            <button type="button" class="text-[13px] text-link hover:underline">
                                "Learn more about meeting links"
                            </button>
                        </p>
                    </div>
                    <ul class="mt-4 flex flex-col gap-2" hidden={links.length === 0}>
                        <template pp-for="link in links">
                            <li
                                key={link.id}
                                class="flex items-center gap-4 rounded-md border border-edge bg-raised px-5 py-4"
                            >
                                <span
                                    class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-brand-soft text-brand-ink"
                                >
                                    <Icon name="link" class="h-[18px] w-[18px]" />
                                </span>
                                <span class="min-w-0 flex-1">
                                    <span class="block truncate text-[13px] font-semibold text-ink">
                                        {link.title}
                                    </span>
                                    <span class="block truncate text-xs text-muted">
                                        {link.url}
                                    </span>
                                </span>
                                <span class="shrink-0 text-xs text-subtle">{link.created}</span>
                                <button
                                    type="button"
                                    class="h-8 shrink-0 rounded border border-edge px-3 text-xs font-semibold text-ink transition-colors hover:bg-hover"
                                    onclick={copyLink(link.url)}
                                >
                                    "Copy"
                                </button>
                            </li>
                        </template>
                    </ul>
                    <p class="mt-3 text-xs text-busy" hidden={!error}>{error}</p>
                    <div class="mt-11 flex items-center justify-between">
                        <h2 class="text-[15px] font-semibold text-ink">"Scheduled meetings"</h2>
                        <a
                            href="/calendar"
                            class="flex items-center gap-2 text-[13px] text-link hover:underline"
                        >
                            <Icon name="calendar-day" class="h-4 w-4" />
                            "Open calendar view"
                        </a>
                    </div>
                    <p class="mt-4 text-[13px] text-subtle">
                        "Nothing is scheduled. Meetings you accept show up here."
                    </p>
                </div>
                <script>r##"
                    const [links, setLinks] = pp.state([]);
                    const [creating, setCreating] = pp.state(false);
                    const [error, setError] = pp.state("");
                    const [joinOpen, setJoinOpen] = pp.state(false);
                    const [meetingId, setMeetingId] = pp.state("");
                    const [joinNotice, setJoinNotice] = pp.state("");

                    async function createLink() {
                        if (creating) return;
                        setCreating(true);
                        setError("");
                        try {
                            const link = await pp.rpc("create_meeting_link", {
                                title: "Meeting with Jefferson Abraham",
                            });
                            setLinks((current) => [link].concat(current));
                        } catch (failure) {
                            setError(failure.message || "The link could not be created.");
                        } finally {
                            setCreating(false);
                        }
                    }

                    function copyLink(url) {
                        if (navigator.clipboard) {
                            navigator.clipboard.writeText(url);
                        }
                    }

                    function join(event) {
                        event.preventDefault();
                        const id = meetingId.trim();
                        if (!id) return;
                        setJoinNotice("Looking for meeting " + id + "…");
                    }
                "##</script>
            </main>
        </AppShell>
    }
}

/// A saved meeting link.
#[derive(Serialize)]
pub struct MeetingLink {
    pub id: String,
    pub title: String,
    pub url: String,
    pub created: String,
}

/// Mint a meeting link.
///
/// The identifier is the server's to hand out: a browser that picked its own
/// could collide with somebody else's meeting, so the round trip is the point
/// even while nothing is persisted yet.
#[rpc]
pub async fn create_meeting_link(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;

    let now = time::now();
    let id = format!("{:x}", now.timestamp_micros());

    Ok(MeetingLink {
        url: format!("https://meet.upurnaka.test/j/{id}"),
        created: now.format("%b %-d, %-I:%M %p").to_string(),
        id,
        title,
    })
}
