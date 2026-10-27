//! Calendar's meeting-ID entry point; meetings open in their own window.
use crate::components::icon::Icon;
use crate::rahti::{Html, component, html};

#[component]
pub fn JoinMeetingById() -> Html {
    html! {
        <div>
            <button
                type="button"
                class="flex h-9 items-center gap-2 rounded-md border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                aria-label="Join with an ID"
                aria-haspopup="dialog"
                aria-expanded={open}
                aria-controls={dialogId}
                pp-ref={trigger}
                onclick={toggleJoin()}
            >
                <Icon name="hash" class="h-4 w-4" />
                "Join with an ID"
            </button>
            <section
                id={dialogId}
                pp-ref={dialog}
                popover="auto"
                role="dialog"
                aria-labelledby={titleId}
                class="calendar-join-dialog rounded-md border border-edge bg-raised p-5 text-ink shadow-2xl"
                pp-style={position}
                ontoggle={setOpen(event.newState === "open")}
                onkeydown={handleKey(event)}
            >
                <header class="mb-5 flex items-center justify-between gap-3">
                    <h2 id={titleId} class="text-[15px] font-semibold">
                        "Join a meeting with an ID"
                    </h2>
                    <button
                        type="button"
                        aria-label="Close meeting ID dialog"
                        class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-hover hover:text-ink"
                        onclick={closeJoin()}
                    >
                        <Icon name="close" class="h-5 w-5" />
                    </button>
                </header>
                <form onsubmit={join(event)} novalidate>
                    <label for={inputId} class="mb-2 block text-[13px]">
                        "Meeting ID "
                        <span class="text-danger-ink" aria-hidden="true">"*"</span>
                    </label>
                    <input
                        id={inputId}
                        pp-ref={meetingInput}
                        name="meeting"
                        type="text"
                        required
                        maxlength="80"
                        autocomplete="off"
                        spellcheck="false"
                        aria-label="Meeting ID"
                        aria-describedby={error ? errorId : null}
                        aria-invalid={!!error}
                        placeholder="Type a meeting ID"
                        class="h-10 w-full rounded-t border-b-2 border-edge bg-surface px-3 text-[14px] outline-none placeholder:text-muted focus:border-brand-ink"
                        value={meetingId}
                        oninput={changeId(target.value)}
                    />
                    <label for={passcodeId} class="mb-2 mt-4 block text-[13px]">
                        "Meeting passcode"
                    </label>
                    <input
                        id={passcodeId}
                        type="text"
                        disabled
                        placeholder="Not required for Upurnaka meetings"
                        class="h-10 w-full rounded-t border-b-2 border-edge bg-surface px-3 text-[13px] text-muted disabled:cursor-not-allowed"
                    />
                    <p
                        id={errorId}
                        role="alert"
                        class="mt-3 text-xs text-danger-ink"
                        hidden={!error}
                    >
                        {error}
                    </p>
                    <a
                        href={joinUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="mt-3 block text-sm text-link hover:underline"
                        hidden={!blocked}
                    >
                        "Open meeting"
                    </a>
                    <button
                        type="submit"
                        disabled={!meetingId.trim()}
                        class="mt-4 h-10 w-full rounded bg-brand px-4 text-sm font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:bg-surface disabled:text-subtle"
                    >
                        "Join meeting"
                    </button>
                </form>
            </section>
            <style>r##"
                .calendar-join-dialog {position:fixed;inset:auto;margin:0;width:340px;max-width:calc(100vw - 16px);max-height:calc(100dvh - 16px);overflow-y:auto;}
            "##</style>
            <script>r##"
                const trigger = pp.ref(null);
                const dialog = pp.ref(null);
                const meetingInput = pp.ref(null);
                const dialogId = pp.id();
                const titleId = pp.id();
                const inputId = pp.id();
                const passcodeId = pp.id();
                const errorId = pp.id();
                const [open, setOpen] = pp.state(false);
                const [position, setPosition] = pp.state('');
                const [meetingId, setMeetingId] = pp.state('');
                const [error, setError] = pp.state('');
                const [blocked, setBlocked] = pp.state(false);
                const joinUrl = '/meet/j/' + encodeURIComponent(meetingId.trim());
                function toggleJoin() {
                    if (dialog.current.matches(':popover-open')) {closeJoin(); return;}
                    const anchor = trigger.current.getBoundingClientRect();
                    const width = Math.min(340, innerWidth - 16);
                    const left = Math.max(8, Math.min(anchor.right - width, innerWidth - width - 8));
                    setPosition('left:' + left + 'px;top:' + Math.max(8, Math.min(anchor.bottom + 8, innerHeight - 360)) + 'px');
                    setError(''); setBlocked(false);
                    dialog.current.showPopover(); meetingInput.current?.focus();
                }
                function closeJoin() {dialog.current.hidePopover(); trigger.current?.focus();}
                function handleKey(event) {
                    if (event.key === 'Escape') {event.preventDefault(); event.stopPropagation(); closeJoin();}
                }
                function changeId(value) {setMeetingId(value); setError(''); setBlocked(false);}
                function join(event) {
                    event.preventDefault();
                    const data = Object.fromEntries(new FormData(event.currentTarget));
                    const id = String(data.meeting || '').trim();
                    if (!/^[A-Za-z0-9-]{1,80}$/.test(id)) {
                        setError('Enter a valid meeting ID using letters, numbers or hyphens.');
                        meetingInput.current?.focus(); return;
                    }
                    const tab = window.open('about:blank', '_blank');
                    if (!tab) {setError('Allow pop-ups, or use Open meeting below.'); setBlocked(true); return;}
                    tab.opener = null;
                    tab.location.replace('/meet/j/' + encodeURIComponent(id));
                    closeJoin();
                }
            "##</script>
        </div>
    }
}
