//! Calendar's immediate-meeting action, backed by the shared meeting store.
use crate::components::aria::{labelled, named, plain, submit};
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::field::{Field, FieldError, FieldGroup, FieldLabel};
use crate::components::rahti_ui::input::{Input, input_variants};
use crate::components::rahti_ui::popover::{
    Popover, PopoverContent, PopoverHeader, PopoverTitle, PopoverTrigger,
};
use crate::meetings::{self, MeetingLink};
use crate::rahti::{Html, Result, Validator, component, html, rpc};

/// The popover's name: its id is `popover-meet-now`, and the close button
/// names that id to hide it with no script.
const NAME: &str = "meet-now";

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
    let trigger = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Default,
        "text-[13px] font-semibold",
    );
    // An element rather than an `<Input>`, so its own focus handler runs as
    // the popover focuses it; it wears the Input's class list.
    let title_input = input_variants("h-10!");
    let close = Attrs::new()
        .set("popovertarget", format!("popover-{NAME}"))
        .set("popovertargetaction", "hide")
        .merge(&labelled("Close meet now dialog"));
    html! {
        <div>
            <Popover>
                <PopoverTrigger name=@{NAME} class=@{trigger.as_str()} attrs=@{named("Meet now")}>
                    <Icon name="video" class="size-4" />
                    "Meet now"
                </PopoverTrigger>
                <PopoverContent
                    name=@{NAME}
                    align="end"
                    class="calendar-meet-now w-[360px]! max-w-[calc(100vw-16px)] p-5!"
                    ontoggle={prepare(event.newState)}
                >
                    <PopoverHeader class="mb-4 pr-8">
                        <PopoverTitle class="text-[15px] font-semibold">
                            "Start a meeting now"
                        </PopoverTitle>
                    </PopoverHeader>
                    <form onsubmit={startMeeting(event)} novalidate>
                        <FieldGroup class="gap-4">
                            <Field>
                                <FieldLabel for="meet-now-title" class="text-[13px]">
                                    "Meeting name"
                                </FieldLabel>
                                <input
                                    id="meet-now-title"
                                    data-slot="input"
                                    name="title"
                                    type="text"
                                    required
                                    maxlength="120"
                                    autocomplete="off"
                                    class=@{title_input.as_str()}
                                    value={name}
                                    onfocus={selectAfterFocus(event.currentTarget)}
                                    oninput={changeName(target.value)}
                                    disabled={busy}
                                />
                                <FieldError hidden={!error}>{error}</FieldError>
                            </Field>
                            <Field class="calendar-meet-share" hidden={!shared}>
                                <FieldLabel
                                    for="meet-now-link"
                                    class="text-xs text-muted-foreground"
                                >
                                    "Meeting link"
                                </FieldLabel>
                                <Input
                                    id="meet-now-link"
                                    class="text-xs"
                                    attrs=@{Attrs::new().set("readonly", "")}
                                    value={shared ? absolute(shared.url) : ""}
                                    onfocus={event.currentTarget.select()}
                                />
                                <p
                                    role="status"
                                    class="text-xs text-muted-foreground"
                                    hidden={!notice}
                                >
                                    {notice}
                                </p>
                                <a
                                    href={shared ? shared.url : "/meet"}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="text-xs text-link hover:underline"
                                >
                                    "Open meeting"
                                </a>
                            </Field>
                            <div class="flex flex-col gap-2.5">
                                <Button
                                    attrs=@{plain()}
                                    variant="outline"
                                    class="calendar-meet-share-button h-10! w-full font-semibold"
                                    disabled={busy || !name.trim()}
                                    onclick={getShareLink()}
                                >
                                    {busy ? "Please wait…" : "Get a link to share"}
                                </Button>
                                <Button
                                    class="calendar-meet-start h-10! w-full font-semibold"
                                    attrs=@{submit()}
                                    disabled={busy || !name.trim()}
                                >
                                    {busy ? "Please wait…" : "Start meeting"}
                                </Button>
                            </div>
                        </FieldGroup>
                    </form>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        class="absolute right-3 top-3 text-muted-foreground"
                        attrs=@{close}
                    >
                        <span class="contents" pp-ref={closer}>
                            <Icon name="close" class="size-5" />
                        </span>
                    </Button>
                </PopoverContent>
            </Popover>
            <script>r##"
                const closer = pp.ref(null);
                const [name, setName] = pp.state(@{default_title});
                const [busy, setBusy] = pp.state(false);
                const [shared, setShared] = pp.state(null);
                const [notice, setNotice] = pp.state('');
                const [error, setError] = pp.state('');
                function absolute(url) {return location.origin + url;}
                function prepare(state) {
                    if (state === 'open') setError('');
                }
                // The popover puts the caret after the suggested name as it
                // focuses the field; selecting once that has run lets typing replace
                // the whole name instead.
                function selectAfterFocus(field) {
                    queueMicrotask(() => field.select());
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
                        const meeting = await ensureMeeting(name);
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
                        if (tab && !tab.closed) {tab.location.replace(absolute(meeting.url)); closer.current?.click();}
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
