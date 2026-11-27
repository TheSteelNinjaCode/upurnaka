//! The unread count on the Activity rail item.
//!
//! The count lives in a small shared store (`window.UpurnakaActivity`) so the
//! Activity page can lower it the moment something is read, without waiting
//! for the next poll. The badge asks the server on load, every 30 seconds
//! while the tab is visible, and whenever the tab becomes visible again.

use crate::rahti::{Html, Result, component, html, rpc};

/// How many of the signed-in person's notifications are unread.
#[rpc(auth)]
pub async fn activity_unread_count() -> Result<u64> {
    crate::notifications::unread().await
}

#[component]
pub fn ActivityBadge() -> Html {
    html! {
        <span class="contents">
            <span
                class="activity-badge absolute right-1 top-1 h-4 min-w-4 rounded-full bg-busy px-1 text-center text-[10px] font-semibold leading-4 text-white"
                aria-hidden="true"
                hidden={!count}
            >
                {count > 99 ? "99+" : count}
            </span>
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
                const count = pp.syncExternalStore(store.subscribe, store.get);
                pp.effect(() => {
                    const load = async () => {
                        if (document.hidden) return;
                        try { store.set(await pp.rpc('activity_unread_count', {})); } catch (failure) { /* the next poll tries again */ }
                    };
                    load();
                    const timer = setInterval(load, 30000);
                    document.addEventListener('visibilitychange', load);
                    return () => { clearInterval(timer); document.removeEventListener('visibilitychange', load); };
                }, []);
            "##</script>
        </span>
    }
}
