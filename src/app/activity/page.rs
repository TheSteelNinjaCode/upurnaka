//! Activity: the meeting events this person was told about, newest first.
//!
//! Selecting one marks it read and shows it in the main pane, with the link
//! that acts on it. The rail badge shares the unread count through
//! `window.UpurnakaActivity` (see `components/activity_badge.rs`), so reading
//! here lowers it at once.

use crate::components::app_shell::AppShell;
use crate::components::aria::labelled;
use crate::components::icon::Icon;
use crate::components::rahti_ui::button::Button;
use crate::components::rahti_ui::toggle::Toggle;
use crate::notifications::{self, Item};
use crate::rahti::{Html, Json, Metadata, Result, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Activity")
        .description("Meeting invitations, mentions, lobby requests and recordings.")
}

/// The newest notifications, for a refresh while the page is open.
#[rpc(auth)]
pub async fn activity_feed() -> Result<Vec<Item>> {
    notifications::feed().await
}

/// Mark one read; answers with how many are still unread.
#[rpc(auth)]
pub async fn read_activity(id: String) -> Result<u64> {
    notifications::mark_read(id).await
}

/// Mark all read; answers with how many are still unread, which is none.
#[rpc(auth)]
pub async fn read_all_activity() -> Result<u64> {
    notifications::mark_all_read().await
}

/// A kind's icon, shown only on the rows of that kind.
const KIND_ICON: &str =
    "flex size-9 shrink-0 items-center justify-center rounded-full bg-brand-soft text-brand-ink";

pub async fn page() -> Result<Html> {
    let items = notifications::feed().await?;
    Ok(html! {
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
                                attrs=@{labelled("Mark all as read").set("title", "Mark all as read")}
                                disabled={unread === 0}
                                onclick={readAll()}
                            >
                                <Icon name="check-circle" class="size-[18px]" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                href="/settings?category=notifications"
                                attrs=@{labelled("Notification settings")}
                            >
                                <Icon name="settings" class="size-[18px]" />
                            </Button>
                        </div>
                    </div>
                    <p class="px-5 pt-1 text-xs text-busy" role="alert" hidden={!error}>{error}</p>
                    <div
                        class="activity-list scroll-thin min-h-0 flex-1 overflow-y-auto px-3 pt-3 pb-3"
                        role="list"
                        aria-label="Notifications"
                    >
                        <p
                            class="px-2 pt-2 text-[13px] leading-relaxed text-muted-foreground"
                            hidden={shown.length > 0}
                        >
                            {unreadOnly && items.length > 0 ? "You're all caught up. Nothing unread." : "Meeting invitations, @mentions in meeting chat, people waiting in your lobby and your recordings will show up here."}
                        </p>
                        <template pp-for="item in shown">
                            <button
                                key={item.id}
                                type="button"
                                role="listitem"
                                class={"activity-row relative flex w-full items-start gap-3 rounded-md px-3 py-2.5 text-left transition-colors hover:bg-hover" + (item.id === selectedId ? " bg-raised" : "")}
                                aria-current={item.id === selectedId ? "true" : null}
                                value={item.id}
                                onclick={open(event.currentTarget.value)}
                            >
                                <span
                                    class="absolute left-1 top-1/2 size-2 -translate-y-1/2 rounded-full bg-brand-ink"
                                    hidden={item.read}
                                >
                                </span>
                                <span class=@{KIND_ICON} hidden={item.kind !== "invite"}>
                                    <Icon name="person-add" class="size-[18px]" />
                                </span>
                                <span class=@{KIND_ICON} hidden={item.kind !== "scheduled"}>
                                    <Icon name="calendar" class="size-[18px]" />
                                </span>
                                <span class=@{KIND_ICON} hidden={item.kind !== "lobby"}>
                                    <Icon name="people" class="size-[18px]" />
                                </span>
                                <span class=@{KIND_ICON} hidden={item.kind !== "mention"}>
                                    <Icon name="at" class="size-[18px]" />
                                </span>
                                <span class=@{KIND_ICON} hidden={item.kind !== "recording"}>
                                    <Icon name="record" class="size-[18px]" />
                                </span>
                                <span class="min-w-0 flex-1">
                                    <span
                                        class={"block truncate text-[13px] " + (item.read ? "text-ink" : "font-semibold text-ink")}
                                    >
                                        {item.title}
                                    </span>
                                    <span class="block truncate text-xs text-muted-foreground">
                                        {item.body}
                                    </span>
                                    <span class="mt-0.5 block text-[11px] text-subtle">
                                        {ago(item.at)}
                                    </span>
                                </span>
                            </button>
                        </template>
                    </div>
                </aside>
                <main
                    class="flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <div
                        class="m-auto max-w-sm px-8 text-center text-[13px] text-muted-foreground"
                        hidden={!!selected}
                    >
                        <Icon name="bell" class="mx-auto mb-4 size-10 text-subtle" />
                        {items.length ? "Select an item to see it here." : "Nothing yet. You will be told here when something happens in your meetings."}
                    </div>
                    <template pp-for="item in selected ? [selected] : []">
                        <article
                            key={item.id}
                            class="activity-detail mx-auto w-full max-w-xl px-8 py-10"
                        >
                            <p class="text-xs text-subtle">{when(item.at)}</p>
                            <h2 class="mt-2 text-xl font-semibold text-ink">{item.title}</h2>
                            <p
                                class="mt-3 whitespace-pre-wrap break-words text-[13px] leading-relaxed text-muted-foreground"
                            >
                                {item.body}
                            </p>
                            <a
                                href={item.link}
                                class="mt-6 inline-flex h-9 items-center rounded-md bg-brand px-4 text-[13px] font-semibold text-white hover:opacity-90"
                            >
                                {action(item)}
                            </a>
                        </article>
                    </template>
                </main>
                <script>r##"
                    const store = window.UpurnakaActivity ||= (() => {
                        let count = 0;
                        const listeners = new Set();
                        return {
                            subscribe(listener) { listeners.add(listener); return () => listeners.delete(listener); },
                            get: () => count,
                            set(next) { if (next === count) return; count = next; listeners.forEach((listener) => listener()); },
                        };
                    })();
                    const [items, setItems] = pp.state(@{Json(&items)});
                    const [unreadOnly, setUnreadOnly] = pp.state(false);
                    const [selectedId, setSelectedId] = pp.state('');
                    const [error, setError] = pp.state('');
                    const unread = items.filter((item) => !item.read).length;
                    const shown = unreadOnly ? items.filter((item) => !item.read || item.id === selectedId) : items;
                    const selected = items.find((item) => item.id === selectedId) || null;

                    const ACTIONS = {invite: 'Open meeting', scheduled: 'Open meeting', lobby: 'Join and let them in', mention: 'Open meeting', recording: 'Open'};
                    function action(item) {
                        if (item.kind === 'recording') return item.link.startsWith('/files') ? 'Go to recordings' : 'Open meeting';
                        return ACTIONS[item.kind] || 'Open';
                    }
                    function ago(at) {
                        const seconds = Math.round((Date.now() - at) / 1000);
                        const format = new Intl.RelativeTimeFormat(undefined, {numeric: 'auto'});
                        if (seconds < 60) return format.format(-seconds, 'second');
                        if (seconds < 3600) return format.format(-Math.floor(seconds / 60), 'minute');
                        if (seconds < 86400) return format.format(-Math.floor(seconds / 3600), 'hour');
                        if (seconds < 7 * 86400) return format.format(-Math.floor(seconds / 86400), 'day');
                        return new Date(at).toLocaleDateString(undefined, {month: 'short', day: 'numeric'});
                    }
                    function when(at) {
                        return new Date(at).toLocaleString(undefined, {weekday: 'short', month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit'});
                    }

                    const open = pp.event(async function open(id) {
                        setSelectedId(id);
                        setError('');
                        const item = items.find((row) => row.id === id);
                        if (!item || item.read) return;
                        setItems((rows) => rows.map((row) => row.id === id ? {...row, read: true} : row));
                        try { store.set(await pp.rpc('read_activity', {id})); }
                        catch (failure) {
                            setItems((rows) => rows.map((row) => row.id === id ? {...row, read: false} : row));
                            setError(failure.message || 'This could not be marked as read.');
                        }
                    });
                    const readAll = pp.event(async function readAll() {
                        setError('');
                        const before = items;
                        setItems((rows) => rows.map((row) => ({...row, read: true})));
                        try { store.set(await pp.rpc('read_all_activity', {})); }
                        catch (failure) { setItems(before); setError(failure.message || 'Your activity could not be marked as read.'); }
                    });

                    // New activity arrives while the page is open: refresh on the
                    // badge's rhythm, and whenever the tab comes back into view.
                    pp.effect(() => {
                        const load = async () => {
                            if (document.hidden) return;
                            try { setItems(await pp.rpc('activity_feed', {})); } catch (failure) { /* the next refresh tries again */ }
                        };
                        const timer = setInterval(load, 30000);
                        document.addEventListener('visibilitychange', load);
                        return () => { clearInterval(timer); document.removeEventListener('visibilitychange', load); };
                    }, []);
                "##</script>
            </div>
        </AppShell>
    })
}
