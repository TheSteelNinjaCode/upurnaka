//! Calendar's meeting-ID entry point; meetings open in their own window.
use crate::components::aria::{labelled, named, submit};
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::field::{Field, FieldError, FieldGroup, FieldLabel};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::popover::{
    Popover, PopoverContent, PopoverHeader, PopoverTitle, PopoverTrigger,
};
use crate::rahti::{Html, component, html};

/// The popover's name: its id is `popover-join-by-id`, and the close button
/// names that id to hide it with no script.
const NAME: &str = "join-by-id";

#[component]
pub fn JoinMeetingById() -> Html {
    let trigger = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Default,
        "text-[13px] font-semibold",
    );
    let close = Attrs::new()
        .set("popovertarget", format!("popover-{NAME}"))
        .set("popovertargetaction", "hide")
        .merge(&labelled("Close meeting ID dialog"));
    html! {
        <div>
            <Popover>
                <PopoverTrigger
                    name=@{NAME}
                    class=@{trigger.as_str()}
                    attrs=@{named("Join with an ID")}
                >
                    <Icon name="hash" class="size-4" />
                    "Join with an ID"
                </PopoverTrigger>
                <PopoverContent
                    name=@{NAME}
                    align="end"
                    class="calendar-join-dialog w-[340px]! max-w-[calc(100vw-16px)] p-5!"
                    ontoggle={resetJoin(event.newState)}
                >
                    <PopoverHeader class="mb-4 pr-8">
                        <PopoverTitle class="text-[15px] font-semibold">
                            "Join a meeting with an ID"
                        </PopoverTitle>
                    </PopoverHeader>
                    <form onsubmit={join(event)} novalidate>
                        <FieldGroup class="gap-4">
                            <Field>
                                <FieldLabel r#for="join-meeting-id" class="text-[13px]">
                                    "Meeting ID"
                                    <span class="text-danger-ink" aria-hidden="true">"*"</span>
                                </FieldLabel>
                                <Input
                                    id="join-meeting-id"
                                    name="meeting"
                                    required
                                    placeholder="Type a meeting ID"
                                    class="h-10!"
                                    attrs=@{Attrs::new()
                                        .set("maxlength", "80")
                                        .set("autocomplete", "off")
                                        .set("spellcheck", "false")}
                                    value={meetingId}
                                    oninput={changeId(target.value)}
                                />
                                <FieldError hidden={!error}>{error}</FieldError>
                            </Field>
                            <Field>
                                <FieldLabel r#for="join-meeting-passcode" class="text-[13px]">
                                    "Meeting passcode"
                                </FieldLabel>
                                <Input
                                    id="join-meeting-passcode"
                                    disabled
                                    placeholder="Not required for Upurnaka meetings"
                                    class="h-10!"
                                />
                            </Field>
                            <a
                                href={joinUrl}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-sm text-link hover:underline"
                                hidden={!blocked}
                            >
                                "Open meeting"
                            </a>
                            <Button
                                class="h-10! w-full font-semibold"
                                attrs=@{submit()}
                                disabled={!meetingId.trim()}
                            >
                                "Join meeting"
                            </Button>
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
                const [meetingId, setMeetingId] = pp.state('');
                const [error, setError] = pp.state('');
                const [blocked, setBlocked] = pp.state(false);
                const joinUrl = '/meet/j/' + encodeURIComponent(meetingId.trim());
                function resetJoin(state) {
                    if (state === 'open') {setError(''); setBlocked(false);}
                }
                function changeId(value) {setMeetingId(value); setError(''); setBlocked(false);}
                function join(event) {
                    event.preventDefault();
                    const data = Object.fromEntries(new FormData(event.currentTarget));
                    const id = String(data.meeting || '').trim();
                    if (!/^[A-Za-z0-9-]{1,80}$/.test(id)) {
                        setError('Enter a valid meeting ID using letters, numbers or hyphens.');
                        event.currentTarget.elements.meeting.focus(); return;
                    }
                    const tab = window.open('about:blank', '_blank');
                    if (!tab) {setError('Allow pop-ups, or use Open meeting below.'); setBlocked(true); return;}
                    tab.opener = null;
                    tab.location.replace('/meet/j/' + encodeURIComponent(id));
                    closer.current?.click();
                }
            "##</script>
        </div>
    }
}
