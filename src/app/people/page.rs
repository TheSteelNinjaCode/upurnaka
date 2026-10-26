//! People: the contact list, and the invitation that fills it.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::avatar::Avatar;
use crate::components::icon::Icon;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("People")
        .description("Your Teams contacts, and who is active now.")
}

pub async fn page() -> Html {
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();
    html! {
        <AppShell active="people">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[420px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">"People"</h1>
                        <button
                            type="button"
                            class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                            title="People settings"
                            aria-label="People settings"
                        >
                            <Icon name="settings" class="h-[18px] w-[18px]" />
                        </button>
                    </div>
                    <div class="mt-3 px-4">
                        <button
                            type="button"
                            class={group === "all" ? NAV_ON : NAV_OFF}
                            onclick={setGroup("all")}
                        >
                            <Icon name="people" class="h-[18px] w-[18px]" />
                            "All contacts"
                        </button>
                        <button
                            type="button"
                            class={group === "active" ? NAV_ON : NAV_OFF}
                            onclick={setGroup("active")}
                        >
                            <Icon name="check-circle" class="h-[18px] w-[18px]" />
                            "Active now"
                        </button>
                    </div>
                    <div class="mx-5 mt-3 h-px bg-line"></div>
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-4 py-3">
                        <div
                            class="flex items-center gap-3 rounded-md px-3 py-2 transition-colors hover:bg-hover"
                        >
                            <Avatar initials=@{initials.as_str()} size="md" presence="online" />
                            <span class="min-w-0 flex-1">
                                <span class="block truncate text-[13px] font-semibold text-ink">
                                    @{user.name.as_str()}
                                </span>
                                <span class="block truncate text-xs text-muted">
                                    "Your profile"
                                </span>
                            </span>
                            <button
                                type="button"
                                class="flex h-8 w-8 shrink-0 items-center justify-center rounded text-muted transition-colors hover:bg-raised hover:text-ink"
                                title="More options"
                                aria-label="More options"
                            >
                                <Icon name="more" class="h-[18px] w-[18px]" />
                            </button>
                        </div>
                        <template pp-for="person in shown">
                            <div
                                key={person.email}
                                class="flex items-center gap-3 rounded-md px-3 py-2 transition-colors hover:bg-hover"
                            >
                                <span
                                    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-avatar text-xs font-semibold text-avatar-ink"
                                >
                                    {person.initials}
                                </span>
                                <span class="min-w-0 flex-1">
                                    <span class="block truncate text-[13px] font-semibold text-ink">
                                        {person.email}
                                    </span>
                                    <span class="block truncate text-xs text-muted">
                                        {person.status}
                                    </span>
                                </span>
                            </div>
                        </template>
                    </div>
                    <div class="shrink-0 p-4">
                        <button
                            type="button"
                            class="flex h-9 w-full items-center justify-center gap-3 rounded-md border border-edge bg-raised text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                            onclick={openInvite()}
                        >
                            <Icon name="person-add" class="h-[18px] w-[18px]" />
                            "Invite to Teams"
                        </button>
                    </div>
                </aside>
                <main
                    class="relative flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <header class="flex h-16 shrink-0 items-center justify-between px-8">
                        <h2 class="text-[17px] font-semibold text-ink">
                            {group === "all" ? "All contacts" : "Active now"}
                        </h2>
                        <div class="relative w-64">
                            <Icon
                                name="search"
                                class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted"
                            />
                            <input
                                type="text"
                                class="h-8 w-full rounded-md border border-edge bg-raised pl-9 pr-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                                placeholder="Find a contact"
                                aria-label="Find a contact"
                                autocomplete="off"
                                value={query}
                                oninput={setQuery(target.value)}
                            />
                        </div>
                    </header>
                    <div class="scroll-thin flex min-h-0 flex-1 flex-col overflow-y-auto px-8 pb-8">
                        <div
                            class="m-auto flex max-w-sm flex-col items-center py-16 text-center"
                            hidden={shown.length > 0}
                        >
                            <span class="relative block h-32 w-44">
                                <span
                                    class="absolute left-7 top-1 block h-11 w-11 rounded-full bg-raised"
                                >
                                </span>
                                <span
                                    class="absolute left-2 top-14 block h-16 w-20 rounded-t-[44px] bg-raised"
                                >
                                </span>
                                <span
                                    class="absolute right-11 top-9 block h-8 w-8 rounded-full bg-linear-to-br from-[#d4b4ff] to-[#7c5cff]"
                                >
                                </span>
                                <span
                                    class="absolute right-2 top-[72px] block h-14 w-16 rounded-t-[34px] bg-linear-to-br from-[#c48cff] to-[#7a4ddb]"
                                >
                                </span>
                                <span
                                    class="absolute bottom-1 left-5 block h-4 w-16 rounded-full bg-linear-to-r from-[#5bd16b] via-[#e8c64a] to-[#f08a3c]"
                                >
                                </span>
                            </span>
                            <h3 class="mt-7 text-[17px] font-semibold text-ink">
                                "Bring your contacts with you"
                            </h3>
                            <p class="mt-2.5 text-[13px] leading-relaxed text-muted">
                                "Sync your device contacts, or invite folks to Teams."
                            </p>
                            <button
                                type="button"
                                class="mt-7 h-9 rounded-md bg-brand px-6 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                                onclick={openInvite()}
                            >
                                "Sync device contacts"
                            </button>
                        </div>
                        <div class="grid grid-cols-3 gap-4 pt-2" hidden={shown.length === 0}>
                            <template pp-for="person in shown">
                                <div
                                    key={person.email}
                                    class="rounded-lg border border-edge bg-raised p-5"
                                >
                                    <span
                                        class="flex h-11 w-11 items-center justify-center rounded-full bg-avatar text-sm font-semibold text-avatar-ink"
                                    >
                                        {person.initials}
                                    </span>
                                    <p class="mt-4 truncate text-[13px] font-semibold text-ink">
                                        {person.email}
                                    </p>
                                    <p class="mt-1 text-xs text-muted">{person.status}</p>
                                </div>
                            </template>
                        </div>
                    </div>
                    <div
                        class="absolute inset-0 z-20 flex items-center justify-center bg-black/55 px-6"
                        hidden={!inviteOpen}
                    >
                        <div
                            class="w-full max-w-[460px] rounded-lg border border-edge bg-raised p-8 shadow-2xl shadow-black/70"
                        >
                            <h3 class="text-[17px] font-semibold text-ink">"Invite to Teams"</h3>
                            <p class="mt-2 text-[13px] text-muted">
                                "They will get an email with a link to this workspace."
                            </p>
                            <form class="mt-6" onsubmit={invite(event)} novalidate>
                                <label
                                    class="block text-xs font-semibold text-muted"
                                    for="invite-email"
                                >
                                    "Email address"
                                </label>
                                <input
                                    id="invite-email"
                                    name="email"
                                    type="email"
                                    class="mt-2 h-9 w-full rounded border border-edge bg-surface px-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                                    placeholder="name@example.com"
                                    autocomplete="email"
                                    value={email}
                                    oninput={setEmail(target.value)}
                                />
                                <p class="mt-2 text-xs text-busy" hidden={!fieldError}>
                                    {fieldError}
                                </p>
                                <div class="mt-7 flex items-center justify-end gap-3">
                                    <button
                                        type="button"
                                        class="h-9 rounded border border-edge px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                        onclick={closeInvite()}
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        type="submit"
                                        class="h-9 rounded bg-brand px-5 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                        disabled={sending}
                                    >
                                        {sending ? "Sending…" : "Send invite"}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                </main>
                <script>r##"
                    const NAV_ON = "mt-1 flex h-9 w-full items-center gap-3 rounded-md bg-raised px-3 text-left text-[13px] font-semibold text-ink";
                    const NAV_OFF = "mt-1 flex h-9 w-full items-center gap-3 rounded-md px-3 text-left text-[13px] text-muted transition-colors hover:bg-hover hover:text-ink";

                    const [group, setGroup] = pp.state("all");
                    const [query, setQuery] = pp.state("");
                    const [people, setPeople] = pp.state([]);
                    const [inviteOpen, setInviteOpen] = pp.state(false);
                    const [email, setEmail] = pp.state("");
                    const [fieldError, setFieldError] = pp.state("");
                    const [sending, setSending] = pp.state(false);

                    const needle = query.trim().toLowerCase();
                    const shown = people.filter((person) => {
                        if (group === "active" && !person.active) return false;
                        if (!needle) return true;
                        return person.email.toLowerCase().includes(needle);
                    });

                    function openInvite() {
                        setFieldError("");
                        setInviteOpen(true);
                    }

                    function closeInvite() {
                        setInviteOpen(false);
                    }

                    async function invite(event) {
                        event.preventDefault();
                        if (sending) return;

                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        setSending(true);
                        setFieldError("");
                        try {
                            const person = await pp.rpc("invite_to_teams", data);
                            setPeople((current) => current.concat([person]));
                            setEmail("");
                            setInviteOpen(false);
                        } catch (failure) {
                            const errors = failure.errors || {};
                            const first = errors.email ? errors.email[0] : null;
                            setFieldError(first || failure.message || "The invitation could not be sent.");
                        } finally {
                            setSending(false);
                        }
                    }
                "##</script>
            </div>
        </AppShell>
    }
}

/// Somebody who has been invited but has not joined yet.
#[derive(Serialize)]
pub struct Invitee {
    pub email: String,
    pub initials: String,
    pub status: String,
    pub active: bool,
}

/// Invite an address to this workspace.
///
/// The address is checked in Rust, so a malformed one comes back as a 422 with
/// `email` named and the dialog shows that message under the field.
#[rpc(auth)]
pub async fn invite_to_teams(email: String) -> Result<Invitee> {
    let email = email.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("email", &email)
        .email("email", &email)
        .max_chars("email", &email, 254);
    validator.finish()?;

    let initials = email
        .split(|c: char| c == '.' || c == '@' || c == '_' || c == '-')
        .filter(|part| !part.is_empty())
        .take(2)
        .map(|part| part.chars().next().unwrap_or('?').to_ascii_uppercase())
        .collect::<String>();

    Ok(Invitee {
        email,
        initials,
        status: "Invited".to_string(),
        active: false,
    })
}
