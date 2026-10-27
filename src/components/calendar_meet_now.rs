//! Calendar's immediate-meeting action, backed by the shared meeting store.
use crate::components::icon::Icon;
use crate::meetings::{self, MeetingLink};
use crate::rahti::{Html, Result, Validator, component, html, rpc};

#[rpc(auth)]
pub async fn create_calendar_meeting_link(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;
    meetings::create(title).await
}

#[component]
pub fn CalendarMeetNow() -> Html {
    let user = crate::auth::current().unwrap_or_default();
    let default_title = format!("Meeting with {}", user.name);
    html! {
        <div>
            <button
                type="button"
                class="flex h-9 items-center gap-2 rounded-md border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                aria-label="Meet now"
                aria-haspopup="dialog"
                aria-expanded={open}
                aria-controls={dialogId}
                pp-ref={trigger}
                onclick={toggleMeetNow()}
            >
                <Icon name="video" class="h-4 w-4" />
                "Meet now"
            </button>
            <section
                id={dialogId}
                pp-ref={dialog}
                popover="auto"
                role="dialog"
                aria-labelledby={titleId}
                class="calendar-meet-now rounded-md border border-edge bg-raised p-5 text-ink shadow-2xl"
                pp-style={position}
                ontoggle={setOpen(event.newState === "open")}
                onkeydown={handleKey(event)}
            >
                <header class="mb-5 flex items-center justify-between gap-3">
                    <h2 id={titleId} class="text-[15px] font-semibold">"Start a meeting now"</h2>
                    <button
                        type="button"
                        aria-label="Close meet now dialog"
                        class="flex h-7 w-7 items-center justify-center rounded text-muted hover:bg-hover hover:text-ink"
                        onclick={closeMeetNow()}
                    >
                        <Icon name="close" class="h-5 w-5" />
                    </button>
                </header>
                <form onsubmit={startMeeting(event)} novalidate>
                    <label for={nameId} class="mb-2 block text-[13px]">"Meeting name"</label>
                    <input
                        id={nameId}
                        pp-ref={nameInput}
                        name="title"
                        type="text"
                        maxlength="120"
                        autocomplete="off"
                        required
                        value={name}
                        oninput={changeName(target.value)}
                        disabled={busy}
                        aria-describedby={error ? errorId : null}
                        aria-invalid={!!error}
                        class="h-10 w-full rounded-t border-b-2 border-edge bg-surface px-3 text-[14px] text-ink outline-none focus:border-brand-ink"
                    />
                    <p
                        id={errorId}
                        role="alert"
                        class="mt-3 text-xs text-danger-ink"
                        hidden={!error}
                    >
                        {error}
                    </p>
                    <div class="calendar-meet-share mt-4" hidden={!shared}>
                        <label for={linkId} class="mb-2 block text-xs text-muted">
                            "Meeting link"
                        </label>
                        <input
                            id={linkId}
                            pp-ref={linkInput}
                            readonly
                            aria-label="Meeting link"
                            value={shared ? absolute(shared.url) : ""}
                            class="h-9 w-full rounded border border-edge bg-surface px-2 text-xs"
                            onfocus={event.currentTarget.select()}
                        />
                        <p role="status" class="mt-2 text-xs text-muted" hidden={!notice}>
                            {notice}
                        </p>
                        <a
                            href={shared ? shared.url : "/meet"}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="mt-2 inline-block text-xs text-link hover:underline"
                        >
                            "Open meeting"
                        </a>
                    </div>
                    <button
                        type="button"
                        class="calendar-meet-share-button mt-5 h-10 w-full rounded border border-edge text-sm font-semibold hover:bg-hover disabled:opacity-50"
                        disabled={busy || !name.trim()}
                        onclick={getShareLink()}
                    >
                        {busy ? "Please wait…" : "Get a link to share"}
                    </button>
                    <button
                        type="submit"
                        class="calendar-meet-start mt-2.5 h-10 w-full rounded bg-brand text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                        disabled={busy || !name.trim()}
                    >
                        {busy ? "Please wait…" : "Start meeting"}
                    </button>
                </form>
            </section>
            <style>r##"
                .calendar-meet-now {position:fixed;inset:auto;margin:0;width:360px;max-width:calc(100vw - 16px);max-height:calc(100dvh - 16px);overflow-y:auto;}
            "##</style>
            <script>r##"
                const trigger = pp.ref(null);
                const dialog = pp.ref(null);
                const nameInput = pp.ref(null);
                const linkInput = pp.ref(null);
                const dialogId = pp.id();
                const titleId = pp.id();
                const nameId = pp.id();
                const linkId = pp.id();
                const errorId = pp.id();
                const [open, setOpen] = pp.state(false);
                const [position, setPosition] = pp.state('');
                const [name, setName] = pp.state(@{default_title});
                const [busy, setBusy] = pp.state(false);
                const [shared, setShared] = pp.state(null);
                const [notice, setNotice] = pp.state('');
                const [error, setError] = pp.state('');
                function absolute(url) {return location.origin + url;}
                function toggleMeetNow() {
                    if (dialog.current.matches(':popover-open')) {closeMeetNow(); return;}
                    const anchor = trigger.current.getBoundingClientRect();
                    const width = Math.min(360, innerWidth - 16);
                    setPosition('left:' + Math.max(8, Math.min(anchor.right - width, innerWidth - width - 8)) + 'px;top:' + Math.max(8, Math.min(anchor.bottom + 8, innerHeight - 440)) + 'px');
                    setError(''); dialog.current.showPopover();
                    nameInput.current?.focus(); nameInput.current?.select();
                }
                function closeMeetNow() {dialog.current.hidePopover(); trigger.current?.focus();}
                function handleKey(event) {
                    if (event.key === 'Escape') {event.preventDefault(); event.stopPropagation(); closeMeetNow();}
                }
                function changeName(value) {setName(value); setShared(null); setNotice(''); setError('');}
                async function ensureMeeting(title) {
                    if (shared && shared.title === title.trim()) return shared;
                    const meeting = await pp.rpc('create_calendar_meeting_link', {title});
                    setShared(meeting); return meeting;
                }
                function failureMessage(failure) {return failure.errors?.title?.[0] || failure.message || 'The meeting could not be created. Please try again.';}
                async function getShareLink() {
                    if (busy || !name.trim()) return;
                    setBusy(true); setError(''); setNotice('');
                    try {
                        const meeting = await ensureMeeting(nameInput.current.value);
                        try {await navigator.clipboard.writeText(absolute(meeting.url)); setNotice('Meeting link copied to clipboard.');}
                        catch (failure) {setNotice('Your meeting link is ready. Select the link above to copy it.');}
                    } catch (failure) {setError(failureMessage(failure));}
                    finally {setBusy(false);}
                }
                async function startMeeting(event) {
                    event.preventDefault();
                    const data = Object.fromEntries(new FormData(event.currentTarget));
                    const title = String(data.title || '').trim();
                    if (busy || !title) return;
                    const tab = window.open('about:blank', '_blank');
                    if (tab) tab.opener = null;
                    setBusy(true); setError(''); setNotice('');
                    try {
                        const meeting = await ensureMeeting(title);
                        if (tab && !tab.closed) {tab.location.replace(absolute(meeting.url)); closeMeetNow();}
                        else setNotice('Your meeting is ready. Use Open meeting above.');
                    } catch (failure) {
                        try {if (tab && !tab.closed && tab.location.href === 'about:blank') tab.close();} catch (failure) {}
                        setError(failureMessage(failure));
                    } finally {setBusy(false);}
                }
            "##</script>
        </div>
    }
}
