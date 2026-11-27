///! Meet: the three ways a meeting starts, and the links this account owns.
use crate::components::app_shell::AppShell;
use crate::components::aria::{hidden, labelled, plain, submit};
use crate::components::icon::Icon;
use crate::components::rahti_ui::alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger,
};
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger,
};
use crate::components::rahti_ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::components::rahti_ui::field::{Field, FieldError, FieldGroup, FieldLabel};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::popover::{
    Popover, PopoverContent, PopoverHeader, PopoverTitle, PopoverTrigger,
};
use crate::meetings::{self, MeetingLink, access::AccessSettings, access::Invitee};
use crate::rahti::{Json, Metadata, Result, Validator, html, rpc};

/// One of the three actions across the top of the page.
const ACTION: &str = "h-12! w-full gap-3 text-[13px] font-semibold \
                      aria-expanded:bg-hover [&_svg]:size-[18px]!";

/// The close button in a popover's corner: it names the popover's id, so it
/// hides it with no script.
fn close_popover(name: &str, label: &str) -> Attrs {
    Attrs::new()
        .set("popovertarget", format!("popover-{name}"))
        .set("popovertargetaction", "hide")
        .merge(&labelled(label))
}

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Meet")
        .description("Create a meeting link, schedule a meeting, or join with an ID.")
}

pub async fn page() -> Result {
    // Rendered from the store, so a reload shows the same list rather than an
    // empty one: the cards below are seeded from this and only added to by
    // the browser afterwards.
    let links = meetings::list().await?;
    let user = crate::auth::current().unwrap_or_default();
    let suggested_title = format!("Meeting with {}", user.name);

    let compose_trigger = button_variants(ButtonVariant::Default, ButtonSize::Lg, ACTION);
    let action_trigger = button_variants(ButtonVariant::Outline, ButtonSize::Lg, ACTION);
    let join_link = button_variants(
        ButtonVariant::Secondary,
        ButtonSize::Default,
        "h-9 min-w-0 flex-1 rounded-md bg-foreground/10 text-[13px] font-semibold hover:bg-foreground/15",
    );
    let cancel = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Default,
        "meeting-rename-cancel min-w-28",
    );

    Ok(html! {
        <AppShell active="meet">
            <main
                class="scroll-thin relative min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="mx-auto w-full max-w-[1180px] px-14 py-9">
                    <h1 class="text-[22px] font-semibold tracking-tight text-ink">"Meet"</h1>
                    <div class="mt-7 grid grid-cols-1 gap-5 md:grid-cols-3">
                        <Popover>
                            <PopoverTrigger
                                name="meeting-compose"
                                class=@{compose_trigger.as_str()}
                                attrs=@{Attrs::new().set("aria-controls", "popover-meeting-compose")}
                            >
                                <Icon name="link" class="size-[18px]" />
                                "Create a meeting link"
                            </PopoverTrigger>
                            <PopoverContent
                                name="meeting-compose"
                                align="start"
                                class="w-[anchor-size(width)]! min-w-72 p-4"
                                ontoggle={resetCompose(event.newState)}
                            >
                                <PopoverHeader class="pr-8">
                                    <PopoverTitle class="text-[13px] font-semibold">
                                        "Give your meeting a title"
                                    </PopoverTitle>
                                </PopoverHeader>
                                <form class="mt-3" onsubmit={createLink(event)} novalidate>
                                    <FieldGroup class="gap-4">
                                        <Field>
                                            <Input
                                                name="title"
                                                placeholder="Meeting title"
                                                class="text-[13px]"
                                                attrs=@{Attrs::new()
                                                    .set("aria-label", "Meeting title")
                                                    .set("autocomplete", "off")
                                                    .set("maxlength", "120")}
                                                value={title}
                                                oninput={setTitle(target.value)}
                                            />
                                            <FieldError hidden={!error}>{error}</FieldError>
                                        </Field>
                                        <Button
                                            class="w-full text-[13px] font-semibold"
                                            attrs=@{submit()}
                                            disabled={creating || title.trim().length === 0}
                                        >
                                            {creating ? "Creating…" : "Create and copy link"}
                                        </Button>
                                    </FieldGroup>
                                </form>
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    class="absolute right-3 top-3 text-muted-foreground"
                                    attrs=@{close_popover("meeting-compose", "Close")}
                                >
                                    <span class="contents" pp-ref={composeCloser}>
                                        <Icon name="close" class="size-4" />
                                    </span>
                                </Button>
                            </PopoverContent>
                        </Popover>
                        <DropdownMenu>
                            <DropdownMenuTrigger
                                menu="meeting-schedule"
                                class=@{action_trigger.as_str()}
                                attrs=@{Attrs::new().set("aria-controls", "dropdown-menu-meeting-schedule")}
                            >
                                <Icon name="calendar-day" class="text-tint-pink" />
                                "Schedule a meeting"
                            </DropdownMenuTrigger>
                            <DropdownMenuContent
                                menu="meeting-schedule"
                                align="start"
                                class="w-[anchor-size(width)] min-w-64"
                            >
                                <DropdownMenuItem
                                    href="/calendar?new=1"
                                    class="min-h-10 text-[14px]"
                                >
                                    <img
                                        src="/images/upurnaka-logo.png"
                                        alt=""
                                        class="size-5 shrink-0 object-contain"
                                    />
                                    "Schedule in Upurnaka"
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                    href="https://calendar.google.com/calendar/r/eventedit"
                                    class="min-h-10 text-[14px]"
                                    attrs=@{Attrs::new().set("target", "_blank").set("rel", "noopener noreferrer")}
                                >
                                    <img
                                        src="/images/google-calendar.png"
                                        alt=""
                                        class="size-5 shrink-0 object-contain"
                                    />
                                    "Schedule in Google Calendar"
                                </DropdownMenuItem>
                            </DropdownMenuContent>
                        </DropdownMenu>
                        <Popover>
                            <PopoverTrigger
                                name="meeting-join"
                                class=@{action_trigger.as_str()}
                                attrs=@{Attrs::new().set("aria-controls", "popover-meeting-join")}
                            >
                                <Icon name="hash" class="text-tint-sky" />
                                "Join with a meeting ID"
                            </PopoverTrigger>
                            <PopoverContent
                                name="meeting-join"
                                align="end"
                                class="w-[360px]! max-w-[calc(100vw-100px)] p-5!"
                            >
                                <PopoverHeader class="mb-3 pr-8">
                                    <PopoverTitle class="text-[15px] font-semibold">
                                        "Join a meeting with an ID"
                                    </PopoverTitle>
                                </PopoverHeader>
                                <form onsubmit={join(event)} novalidate>
                                    <FieldGroup class="gap-4">
                                        <Field>
                                            <FieldLabel for="meeting-join-id" class="text-[13px]">
                                                "Meeting ID"
                                            </FieldLabel>
                                            <Input
                                                id="meeting-join-id"
                                                name="meeting"
                                                placeholder="Type a meeting ID"
                                                class="h-10!"
                                                attrs=@{Attrs::new().set("autocomplete", "off")}
                                                value={meetingId}
                                                oninput={setMeetingId(target.value)}
                                            />
                                        </Field>
                                        <Field>
                                            <FieldLabel
                                                for="meeting-join-passcode"
                                                class="text-[13px]"
                                            >
                                                "Meeting passcode"
                                            </FieldLabel>
                                            <Input
                                                id="meeting-join-passcode"
                                                name="passcode"
                                                placeholder="Only if the meeting has one"
                                                class="h-10!"
                                                attrs=@{Attrs::new().set("autocomplete", "off")}
                                                value={meetingPasscode}
                                                oninput={setMeetingPasscode(target.value)}
                                            />
                                        </Field>
                                        <Button
                                            class="h-10! w-full font-semibold"
                                            attrs=@{submit()}
                                            disabled={meetingId.trim().length === 0}
                                        >
                                            "Join meeting"
                                        </Button>
                                    </FieldGroup>
                                </form>
                                <Button
                                    variant="ghost"
                                    size="icon-sm"
                                    class="absolute right-3 top-3 text-muted-foreground"
                                    attrs=@{close_popover("meeting-join", "Close meeting ID dialog")}
                                >
                                    <Icon name="close" class="size-5" />
                                </Button>
                            </PopoverContent>
                        </Popover>
                    </div>
                    <h2 class="mt-10 text-[15px] font-semibold text-ink">"Meeting links"</h2>
                    <div class="mt-4 grid grid-cols-2 gap-5">
                        <template pp-for="link in links">
                            <div
                                key={link.id}
                                data-meeting-id={link.id}
                                class="group relative rounded-lg border border-edge bg-raised px-5 py-4"
                            >
                                <div class="flex items-start justify-between">
                                    <span
                                        class="flex size-9 items-center justify-center rounded-full bg-brand-soft text-brand-ink"
                                    >
                                        <Icon name="link" class="size-[18px]" />
                                    </span>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="-mr-2 text-muted-foreground hover:text-busy"
                                        attrs=@{labelled("Delete meeting link")}
                                        value={link.id}
                                        onclick={confirmRemove(linkById(event.currentTarget.value))}
                                    >
                                        <Icon name="trash" class="size-4" />
                                    </Button>
                                </div>
                                <div class="mt-3 flex min-w-0 items-center gap-2">
                                    <p class="min-w-0 truncate text-[15px] font-semibold text-ink">
                                        {link.title}
                                    </p>
                                    <Button
                                        variant="ghost"
                                        size="icon-xs"
                                        class="text-muted-foreground opacity-0 transition-opacity hover:text-brand-ink focus-visible:opacity-100 group-hover:opacity-100 group-focus-within:opacity-100 pointer-coarse:opacity-100"
                                        attrs=@{labelled("Edit meeting name")}
                                        value={link.id}
                                        onclick={openRename(linkById(event.currentTarget.value))}
                                    >
                                        <Icon name="edit" class="size-4" />
                                    </Button>
                                </div>
                                <p class="mt-1 text-[13px] text-muted-foreground">
                                    "Created "
                                    {link.created}
                                </p>
                                <p
                                    class="meeting-access-summary mt-1 flex items-center gap-1.5 text-[12px] text-subtle"
                                >
                                    <Icon name="shield" class="size-3.5" />
                                    {accessLabel(link.access)}
                                </p>
                                <div class="mt-4 flex items-center gap-3">
                                    <a
                                        href={link.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class={twMerge(meetingActionClass)}
                                    >
                                        "Join"
                                    </a>
                                    <Button
                                        variant="secondary"
                                        class={twMerge(meetingActionClass)}
                                        value={link.id}
                                        onclick={shareLink(linkById(event.currentTarget.value))}
                                    >
                                        "Share link"
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="meeting-access-open shrink-0 text-muted-foreground hover:text-brand-ink"
                                        attrs=@{labelled("Who can join")}
                                        value={link.id}
                                        onclick={openAccess(linkById(event.currentTarget.value))}
                                    >
                                        <Icon name="settings" class="size-4" />
                                    </Button>
                                </div>
                            </div>
                        </template>
                        <div
                            class={links.length === 0 ? "col-span-2 rounded-lg border border-edge bg-raised px-8 py-7" : "rounded-lg border border-edge bg-raised px-8 py-7"}
                        >
                            <p class="text-3xl leading-none">"🔗"</p>
                            <p class="mt-5 text-[15px] font-semibold text-ink">
                                "Quickly create, save, and share links with anyone."
                            </p>
                        </div>
                    </div>
                    <p class="mt-3 text-xs text-busy" hidden={!listError}>{listError}</p>
                    <div class="mt-11 flex items-center justify-between">
                        <h2 class="text-[15px] font-semibold text-ink">"Scheduled meetings"</h2>
                        <a
                            href="/calendar"
                            class="flex items-center gap-2 text-[13px] text-link hover:underline"
                        >
                            <Icon name="calendar-day" class="size-4" />
                            "Open calendar view"
                        </a>
                    </div>
                    <p class="mt-4 text-[13px] text-subtle">
                        "Nothing is scheduled. Meetings you accept show up here."
                    </p>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={renameOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={renameCloser}></span>
                    </DialogClose>
                    <DialogContent
                        class="meeting-rename-dialog sm:max-w-[380px]!"
                        onclose={finishRename()}
                    >
                        <DialogHeader>
                            <DialogTitle class="text-[17px]">"Edit meeting name"</DialogTitle>
                        </DialogHeader>
                        <form onsubmit={saveRename(event)} novalidate pp-ref={renameForm}>
                            <FieldGroup class="gap-4">
                                <Field>
                                    <Input
                                        name="title"
                                        class="h-10!"
                                        attrs=@{Attrs::new()
                                            .set("aria-label", "Meeting name")
                                            .set("maxlength", "120")
                                            .set("autocomplete", "off")
                                            .set("autofocus", "")}
                                        value={renameTitle}
                                        disabled={renaming}
                                        oninput={setRenameTitle(target.value)}
                                    />
                                    <FieldError hidden={!renameError}>{renameError}</FieldError>
                                </Field>
                                <DialogFooter>
                                    <DialogClose class=@{cancel.as_str()} disabled={renaming}>
                                        "Cancel"
                                    </DialogClose>
                                    <Button
                                        class="meeting-rename-save min-w-28"
                                        attrs=@{submit()}
                                        disabled={renaming || !renameTitle.trim()}
                                    >
                                        {renaming ? "Saving…" : "Save"}
                                    </Button>
                                </DialogFooter>
                            </FieldGroup>
                        </form>
                    </DialogContent>
                </Dialog>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={accessOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={accessCloser}></span>
                    </DialogClose>
                    <DialogContent
                        class="meeting-access-dialog sm:max-w-[460px]!"
                        onclose={finishAccess()}
                    >
                        <DialogHeader>
                            <DialogTitle class="text-[17px]">"Who can join"</DialogTitle>
                        </DialogHeader>
                        <form onsubmit={saveAccess(event)} novalidate>
                            <FieldGroup class="gap-4">
                                <div class="grid gap-2" role="radiogroup" aria-label="Who can join">
                                    <label
                                        class="flex cursor-pointer gap-3 rounded-md border border-edge p-3 has-checked:border-brand-ink"
                                    >
                                        <input
                                            type="radio"
                                            name="mode"
                                            value="open"
                                            class="mt-0.5 accent-brand"
                                            checked={accessMode === "open"}
                                            disabled={accessLoading || accessSaving}
                                            onchange={chooseMode("open")}
                                        />
                                        <span class="grid gap-0.5">
                                            <span class="text-[13px] font-semibold text-ink">
                                                "Anyone with the link"
                                            </span>
                                            <span class="text-[12px] text-muted-foreground">
                                                "Guests are welcome. Everyone waits in the lobby until you let them in."
                                            </span>
                                        </span>
                                    </label>
                                    <label
                                        class="flex cursor-pointer gap-3 rounded-md border border-edge p-3 has-checked:border-brand-ink"
                                    >
                                        <input
                                            type="radio"
                                            name="mode"
                                            value="passcode"
                                            class="mt-0.5 accent-brand"
                                            checked={accessMode === "passcode"}
                                            disabled={accessLoading || accessSaving}
                                            onchange={chooseMode("passcode")}
                                        />
                                        <span class="grid gap-0.5">
                                            <span class="text-[13px] font-semibold text-ink">
                                                "People with the link and passcode"
                                            </span>
                                            <span class="text-[12px] text-muted-foreground">
                                                "The link alone is not enough. After the passcode, people wait in the lobby."
                                            </span>
                                        </span>
                                    </label>
                                    <label
                                        class="flex cursor-pointer gap-3 rounded-md border border-edge p-3 has-checked:border-brand-ink"
                                    >
                                        <input
                                            type="radio"
                                            name="mode"
                                            value="invited"
                                            class="mt-0.5 accent-brand"
                                            checked={accessMode === "invited"}
                                            disabled={accessLoading || accessSaving}
                                            onchange={chooseMode("invited")}
                                        />
                                        <span class="grid gap-0.5">
                                            <span class="text-[13px] font-semibold text-ink">
                                                "Only people I invite"
                                            </span>
                                            <span class="text-[12px] text-muted-foreground">
                                                "Invited accounts go straight in. Anyone else is refused, even with the link."
                                            </span>
                                        </span>
                                    </label>
                                </div>
                                <div class="grid gap-2" hidden={accessMode !== "passcode"}>
                                    <FieldLabel for="meeting-access-passcode" class="text-[13px]">
                                        "Passcode"
                                    </FieldLabel>
                                    <div class="flex gap-2">
                                        <Input
                                            id="meeting-access-passcode"
                                            name="passcode"
                                            class="h-10! font-mono"
                                            attrs=@{Attrs::new()
                                                .set("autocomplete", "off")
                                                .set("maxlength", "32")
                                                .set("spellcheck", "false")}
                                            value={accessPasscode}
                                            disabled={accessSaving}
                                            oninput={setAccessPasscode(target.value)}
                                        />
                                        <Button
                                            variant="outline"
                                            class="h-10! shrink-0"
                                            attrs=@{plain()}
                                            onclick={newPasscode()}
                                        >
                                            "New code"
                                        </Button>
                                    </div>
                                    <p class="text-[12px] text-muted-foreground">
                                        "4 to 32 letters and numbers, not case-sensitive. Changes apply when you save."
                                    </p>
                                    <Button
                                        variant="link"
                                        class="h-auto! justify-start! px-0! text-[13px] text-link"
                                        attrs=@{plain()}
                                        onclick={copyPasscodeLink()}
                                    >
                                        "Copy link with passcode"
                                    </Button>
                                </div>
                                <div class="grid gap-2" hidden={accessMode !== "invited"}>
                                    <FieldLabel for="meeting-access-search" class="text-[13px]">
                                        "Invite people"
                                    </FieldLabel>
                                    <Input
                                        id="meeting-access-search"
                                        class="h-10!"
                                        placeholder="Search by name"
                                        attrs=@{Attrs::new().set("autocomplete", "off").set("maxlength", "80")}
                                        value={inviteQuery}
                                        disabled={accessLoading || accessSaving}
                                        oninput={setInviteQuery(target.value)}
                                    />
                                    <div
                                        class="meeting-invite-results grid max-h-40 overflow-y-auto rounded-md border border-edge"
                                        hidden={inviteResults.length === 0}
                                    >
                                        <template pp-for="person in inviteResults">
                                            <button
                                                key={person.id}
                                                type="button"
                                                class="flex items-center justify-between px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                                                value={person.id}
                                                onclick={addInvitee(event.currentTarget.value)}
                                            >
                                                <span class="truncate">{person.name}</span>
                                                <span class="text-[12px] text-link">"Add"</span>
                                            </button>
                                        </template>
                                    </div>
                                    <p class="text-[12px] text-brand-ink" hidden={!suggested}>
                                        "Suggested from the calendar invitation. Save to apply."
                                    </p>
                                    <ul class="meeting-invited grid gap-1">
                                        <template pp-for="person in invited">
                                            <li
                                                key={person.id}
                                                class="flex items-center justify-between rounded-md bg-hover px-3 py-1.5 text-[13px] text-ink"
                                            >
                                                <span class="truncate">{person.name}</span>
                                                <button
                                                    type="button"
                                                    class="text-[12px] text-muted-foreground hover:text-busy"
                                                    value={person.id}
                                                    onclick={removeInvitee(event.currentTarget.value)}
                                                >
                                                    "Remove"
                                                </button>
                                            </li>
                                        </template>
                                    </ul>
                                    <p
                                        class="text-[12px] text-muted-foreground"
                                        hidden={invited.length > 0}
                                    >
                                        "No one invited yet. Only you can join until you add people."
                                    </p>
                                </div>
                                <FieldError hidden={!accessError}>{accessError}</FieldError>
                                <DialogFooter>
                                    <DialogClose class=@{cancel.as_str()} disabled={accessSaving}>
                                        "Cancel"
                                    </DialogClose>
                                    <Button
                                        class="meeting-access-save min-w-28"
                                        attrs=@{submit()}
                                        disabled={accessSaving || accessLoading || (accessMode === "passcode" && accessPasscode.trim().length < 4)}
                                    >
                                        {accessSaving ? "Saving…" : "Save"}
                                    </Button>
                                </DialogFooter>
                            </FieldGroup>
                        </form>
                    </DialogContent>
                </Dialog>
                <AlertDialog>
                    <AlertDialogTrigger attrs=@{hidden()}>
                        <span pp-ref={removeOpener}></span>
                    </AlertDialogTrigger>
                    <AlertDialogContent size="sm">
                        <AlertDialogHeader>
                            <AlertDialogTitle>"Delete this meeting link?"</AlertDialogTitle>
                            <AlertDialogDescription>
                                "Anyone holding "
                                <strong class="font-semibold text-ink">
                                    {pendingRemove ? pendingRemove.title : ""}
                                </strong>
                                " will no longer be able to join with it."
                            </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                            <AlertDialogCancel>"Cancel"</AlertDialogCancel>
                            <AlertDialogAction variant="destructive" onclick={removeLink()}>
                                "Delete"
                            </AlertDialogAction>
                        </AlertDialogFooter>
                    </AlertDialogContent>
                </AlertDialog>
                <div
                    class="fixed bottom-4 left-4 z-50 flex items-center gap-3 rounded-md border border-edge bg-raised px-4 py-3 text-[13px] text-ink shadow-2xl shadow-black/70"
                    role="status"
                    hidden={!toast}
                >
                    <Icon name="check-circle" class="size-[18px] text-online" />
                    {toast}
                </div>
                <script>r##"
                    const meetingActionClass = @{join_link};
                    const [links, setLinks] = pp.state(@{Json(&links)});
                    const [title, setTitle] = pp.state(@{Json(&suggested_title)});
                    const [creating, setCreating] = pp.state(false);
                    const [error, setError] = pp.state("");
                    const [listError, setListError] = pp.state("");
                    const [meetingId, setMeetingId] = pp.state("");
                    const [meetingPasscode, setMeetingPasscode] = pp.state("");
                    const [toast, setToast] = pp.state("");
                    const [editingId, setEditingId] = pp.state("");
                    const [renameTitle, setRenameTitle] = pp.state("");
                    const [renameError, setRenameError] = pp.state("");
                    const [renaming, setRenaming] = pp.state(false);
                    const [pendingRemove, setPendingRemove] = pp.state(null);
                    const composeCloser = pp.ref(null);
                    const renameOpener = pp.ref(null);
                    const renameCloser = pp.ref(null);
                    const renameForm = pp.ref(null);
                    const removeOpener = pp.ref(null);

                    function resetCompose(state) {
                        if (state === "open") setError("");
                    }

                    // The rename and delete dialogs are rahti-ui's, which open
                    // from a trigger; a card reaches them by clicking the hidden
                    // trigger each one carries, once it has said which link.
                    const openRename = pp.event(function openRename(link) {
                        if (!link) return;
                        setRenameTitle(link.title);
                        setRenameError("");
                        setEditingId(link.id);
                        renameOpener.current?.click();
                        // The dialog leaves the caret after the name; renaming starts from
                        // the whole of it selected, so typing replaces it.
                        requestAnimationFrame(() => renameForm.current?.elements.title?.select());
                    });

                    function finishRename() {
                        setEditingId("");
                        setRenameError("");
                    }

                    async function saveRename(event) {
                        event.preventDefault();
                        if (renaming || !editingId) return;
                        const payload = Object.fromEntries(new FormData(event.currentTarget));
                        payload.id = editingId;
                        setRenaming(true);
                        setRenameError("");
                        try {
                            const updated = await pp.rpc("rename_meeting_link", payload);
                            setLinks(current => current.map(link => link.id === updated.id ? updated : link));
                            renameCloser.current?.click();
                            setToast("Meeting name updated");
                        } catch (failure) {
                            setRenameError(failure.errors?.title?.[0] || failure.message || "The meeting name could not be saved.");
                        } finally {
                            setRenaming(false);
                        }
                    }

                    // One timer, restarted whenever a new notice replaces the
                    // one on screen, and cleared when the page goes away.
                    pp.effect(() => {
                        if (!toast) return;
                        const timer = setTimeout(() => setToast(""), 4000);
                        return () => clearTimeout(timer);
                    }, [toast]);

                    // The server hands out a path, because it cannot know
                    // which host this browser used to reach it. The absolute
                    // URL is only ever needed here, at the moment of copying.
                    function absolute(path) {
                        return location.origin + path;
                    }

                    async function copy(text, notice) {
                        try {
                            await navigator.clipboard.writeText(text);
                            setToast(notice);
                        } catch (failure) {
                            setToast("Copy the link from the address bar: " + text);
                        }
                    }

                    async function createLink(event) {
                        event.preventDefault();
                        if (creating) return;

                        const payload = Object.fromEntries(new FormData(event.currentTarget));
                        setCreating(true);
                        setError("");
                        try {
                            const link = await pp.rpc("create_meeting_link", payload);
                            setLinks((current) => [link].concat(current));
                            composeCloser.current?.click();
                            await copy(absolute(link.url), "Link copied to clipboard");
                        } catch (failure) {
                            setError(failure.errors?.title?.[0] || failure.message || "The link could not be created.");
                        } finally {
                            setCreating(false);
                        }
                    }

                    // A passcode meeting's link carries its code, as a Teams
                    // invitation does, so whoever is sent it can go straight in.
                    function shareUrl(link) {
                        const path = link.access === "passcode" && link.passcode
                            ? link.url + "?p=" + encodeURIComponent(link.passcode)
                            : link.url;
                        return absolute(path);
                    }

                    const shareLink = pp.event(function shareLink(link) {
                        if (!link) return;
                        copy(shareUrl(link), link.access === "passcode" ? "Link with passcode copied" : "Link copied to clipboard");
                    });

                    function accessLabel(mode) {
                        if (mode === "passcode") return "Passcode required";
                        if (mode === "invited") return "Invited people only";
                        return "Anyone with the link";
                    }

                    const [accessId, setAccessId] = pp.state("");
                    const [accessMode, setAccessMode] = pp.state("open");
                    const [accessPasscode, setAccessPasscode] = pp.state("");
                    const [invited, setInvited] = pp.state([]);
                    const [suggested, setSuggested] = pp.state(false);
                    const [inviteQuery, setInviteQuery] = pp.state("");
                    const [inviteResults, setInviteResults] = pp.state([]);
                    const [accessError, setAccessError] = pp.state("");
                    const [accessLoading, setAccessLoading] = pp.state(false);
                    const [accessSaving, setAccessSaving] = pp.state(false);
                    const accessOpener = pp.ref(null);
                    const accessCloser = pp.ref(null);

                    const openAccess = pp.event(async function openAccess(link) {
                        if (!link) return;
                        setAccessId(link.id);
                        setAccessMode(link.access || "open");
                        setAccessPasscode(link.passcode || "");
                        setInvited([]);
                        setSuggested(false);
                        setInviteQuery("");
                        setInviteResults([]);
                        setAccessError("");
                        setAccessLoading(true);
                        accessOpener.current?.click();
                        try {
                            const settings = await pp.rpc("meeting_access", { id: link.id });
                            setAccessMode(settings.mode);
                            setAccessPasscode(settings.passcode || "");
                            setInvited(settings.invited);
                            setSuggested(settings.suggested);
                        } catch (failure) {
                            setAccessError(failure.message || "These settings could not be loaded.");
                        } finally {
                            setAccessLoading(false);
                        }
                    });

                    function finishAccess() {
                        setAccessId("");
                        setInviteQuery("");
                        setInviteResults([]);
                    }

                    function newPasscode() {
                        const alphabet = "23456789abcdefghjkmnpqrstuvwxyz";
                        const bytes = crypto.getRandomValues(new Uint8Array(8));
                        setAccessPasscode(Array.from(bytes, b => alphabet[b % alphabet.length]).join(""));
                    }

                    function chooseMode(mode) {
                        setAccessMode(mode);
                        setAccessError("");
                        if (mode === "passcode" && !accessPasscode) newPasscode();
                    }

                    pp.effect(() => {
                        const query = inviteQuery.trim();
                        if (!accessId || !query) { setInviteResults([]); return; }
                        let cancelled = false;
                        const timer = setTimeout(async () => {
                            try {
                                const people = await pp.rpc("search_meeting_invitees", { query });
                                if (!cancelled) setInviteResults(people);
                            } catch (failure) {
                                if (!cancelled) setAccessError(failure.message || "People could not be searched.");
                            }
                        }, 200);
                        return () => { cancelled = true; clearTimeout(timer); };
                    }, [inviteQuery, accessId]);

                    const addInvitee = pp.event(function addInvitee(id) {
                        const person = inviteResults.find(item => item.id === id);
                        if (!person || invited.some(item => item.id === id)) return;
                        setInvited(current => current.concat([person]));
                        setInviteQuery("");
                        setInviteResults([]);
                    });

                    const removeInvitee = pp.event(function removeInvitee(id) {
                        setInvited(current => current.filter(item => item.id !== id));
                    });

                    async function saveAccess(event) {
                        event.preventDefault();
                        if (accessSaving || accessLoading || !accessId) return;
                        setAccessSaving(true);
                        setAccessError("");
                        try {
                            const saved = await pp.rpc("save_meeting_access", {
                                id: accessId,
                                mode: accessMode,
                                passcode: accessMode === "passcode" ? accessPasscode : "",
                                invited: invited.map(person => person.id),
                            });
                            setLinks(current => current.map(link => link.id === saved.id
                                ? Object.assign({}, link, { access: saved.mode, passcode: saved.mode === "passcode" ? saved.passcode : null })
                                : link));
                            accessCloser.current?.click();
                            setToast("Meeting access updated");
                        } catch (failure) {
                            setAccessError(failure.errors?.passcode?.[0] || failure.errors?.invited?.[0] || failure.message || "These settings could not be saved.");
                        } finally {
                            setAccessSaving(false);
                        }
                    }

                    async function copyPasscodeLink() {
                        const link = links.find(item => item.id === accessId);
                        if (!link || !accessPasscode.trim()) return;
                        await copy(absolute(link.url + "?p=" + encodeURIComponent(accessPasscode.trim())), "Link with passcode copied");
                    }

                    // A card's actions name their link by id: a handler in a list
                    // row reads the row through its button, not the loop variable.
                    const linkById = pp.event(function linkById(id) {
                        return links.find(item => item.id === id);
                    });

                    const confirmRemove = pp.event(function confirmRemove(link) {
                        if (!link) return;
                        setPendingRemove(link);
                        removeOpener.current?.click();
                    });

                    async function removeLink() {
                        const link = pendingRemove;
                        if (!link) return;
                        setPendingRemove(null);
                        const kept = links;
                        setLinks((current) => current.filter((item) => item.id !== link.id));
                        try {
                            await pp.rpc("delete_meeting_link", { id: link.id });
                            setToast("Meeting link deleted");
                        } catch (failure) {
                            setLinks(kept);
                            setListError(failure.message || "The link could not be deleted.");
                        }
                    }

                    function join(event) {
                        event.preventDefault();
                        const id = meetingId.trim();
                        if (!id) return;
                        const passcode = meetingPasscode.trim();
                        const query = passcode ? "?p=" + encodeURIComponent(passcode) : "";
                        window.open("/meet/j/" + encodeURIComponent(id) + query, "_blank", "noopener,noreferrer");
                    }
                "##</script>
            </main>
        </AppShell>
    })
}

// Mint a meeting link and keep it.
#[rpc(auth)]
pub async fn create_meeting_link(title: String) -> Result<MeetingLink> {
    let title = title.trim().to_string();

    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;

    meetings::create(title).await
}

/// Rename a live link; the join code and existing conference stay the same.
#[rpc(auth)]
pub async fn rename_meeting_link(id: String, title: String) -> Result<MeetingLink> {
    let id = id.trim().to_string();
    let title = title.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("id", &id)
        .max_chars("id", &id, 64)
        .required("title", &title)
        .max_chars("title", &title, 120);
    validator.finish()?;
    meetings::rename(&id, title).await
}

/// Revoke a link the organizer no longer wants handed around.
///
/// Answers the same `true` whether the row was already gone or never existed:
/// the card is off the page either way, and a browser that asked twice is not
/// an error worth showing somebody.
#[rpc(auth)]
pub async fn delete_meeting_link(id: String) -> Result<bool> {
    let id = id.trim().to_string();

    let mut validator = Validator::new();
    validator.required("id", &id).max_chars("id", &id, 64);
    validator.finish()?;

    meetings::revoke(&id).await?;
    Ok(true)
}

/// The organizer's access settings for one of their links.
#[rpc(auth)]
pub async fn meeting_access(id: String) -> Result<AccessSettings> {
    let id = id.trim().to_string();
    let mut validator = Validator::new();
    validator.required("id", &id).max_chars("id", &id, 64);
    validator.finish()?;
    meetings::access::settings(&id).await
}

/// Change who may enter a meeting: open, passcode, or invited people only.
#[rpc(auth)]
pub async fn save_meeting_access(
    id: String,
    mode: String,
    passcode: String,
    invited: Vec<String>,
) -> Result<AccessSettings> {
    meetings::access::update(id.trim(), mode, passcode, invited).await
}

/// People the organizer can add to an invite list.
#[rpc(auth)]
pub async fn search_meeting_invitees(query: String) -> Result<Vec<Invitee>> {
    let mut validator = Validator::new();
    validator.max_chars("query", &query, 80);
    validator.finish()?;
    meetings::access::search(&query).await
}
