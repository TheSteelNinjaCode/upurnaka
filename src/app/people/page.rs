//! People: the contact list, and the invitation that fills it.

use axum::http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::aria::{hidden, labelled, named, plain, submit};
use crate::components::avatar::ProfileAvatar;
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::avatar::{Avatar, AvatarFallback};
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::card::Card;
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
use crate::components::rahti_ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::components::rahti_ui::field::{Field, FieldError, FieldLabel};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::input_group::{InputGroup, InputGroupAddon, InputGroupInput};
use crate::components::rahti_ui::separator::Separator;
use crate::components::rahti_ui::toggle_group::{ToggleGroup, ToggleGroupItem};
use crate::rahti::{Error, Html, Metadata, Result, RpcFile, Validator, html, rpc};

fn profile_owner() -> Result<String> {
    let user = crate::auth::current()
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Authentication required"))?;
    crate::rahti::authorization::authorize_resource("profile.update", &user.id)?;
    crate::auth::ready()?;
    Ok(user.id)
}

#[rpc(auth)]
pub async fn update_profile_name(name: String) -> Result<bool> {
    let name = name.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("name", &name)
        .max_chars("name", &name, 80)
        .check(
            "name",
            !name.chars().any(char::is_control),
            "Use a name without control characters.",
        );
    validator.finish()?;
    let owner = profile_owner()?;
    use crate::models::account::{Column, Entity};
    Entity::update_many()
        .col_expr(Column::Name, Expr::value(name))
        .filter(Column::Id.eq(owner))
        .exec(crate::db::db())
        .await?;
    Ok(true)
}

#[rpc(auth)]
pub async fn remove_profile_picture() -> Result<bool> {
    let owner = profile_owner()?;
    use crate::models::account::{Column, Entity};
    Entity::update_many()
        .col_expr(Column::Picture, Expr::value(Option::<Vec<u8>>::None))
        .filter(Column::Id.eq(owner))
        .exec(crate::db::db())
        .await?;
    Ok(true)
}

#[rpc(auth)]
pub async fn update_profile_picture(picture: RpcFile) -> Result<bool> {
    let owner = profile_owner()?;
    let mut validator = Validator::new();
    validator.check(
        "picture",
        !picture.is_empty() && picture.len() <= 2 * 1024 * 1024,
        "Choose a PNG or JPEG picture up to 2 MB.",
    );
    validator.finish()?;
    let bytes = picture.into_bytes().await?;
    let normalized =
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<u8>, String> {
            let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
                .with_guessed_format()
                .map_err(|_| "Choose a valid PNG or JPEG picture.".to_string())?;
            if !matches!(
                reader.format(),
                Some(image::ImageFormat::Png | image::ImageFormat::Jpeg)
            ) {
                return Err("Choose a PNG or JPEG picture.".into());
            }
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(4096);
            limits.max_image_height = Some(4096);
            limits.max_alloc = Some(64 * 1024 * 1024);
            reader.limits(limits);
            let decoded = reader.decode().map_err(|_| {
                "Choose a valid picture no larger than 4096 × 4096 pixels.".to_string()
            })?;
            let thumbnail = decoded.resize_to_fill(256, 256, image::imageops::FilterType::Lanczos3);
            let mut output = std::io::Cursor::new(Vec::new());
            thumbnail
                .write_to(&mut output, image::ImageFormat::Png)
                .map_err(|_| "This picture could not be processed.".to_string())?;
            Ok(output.into_inner())
        })
        .await?;
    let mut validator = Validator::new();
    validator.check(
        "picture",
        normalized.is_ok(),
        normalized
            .as_ref()
            .err()
            .map(String::as_str)
            .unwrap_or("Choose a valid picture."),
    );
    validator.finish()?;
    use crate::models::account::{Column, Entity};
    Entity::update_many()
        .col_expr(Column::Picture, Expr::value(normalized.unwrap()))
        .filter(Column::Id.eq(owner))
        .exec(crate::db::db())
        .await?;
    Ok(true)
}

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("People")
        .description("Your Teams contacts, and who is active now.")
}

/// A group in the people pane: muted until it is the chosen one.
const NAV: &str = "h-9! w-full justify-start! gap-3 rounded-md! px-3! text-[13px] \
                   text-muted-foreground has-[:checked]:bg-raised! has-[:checked]:font-semibold \
                   has-[:checked]:text-ink!";

/// A contact's initials on a disc.
const INITIALS: &str = "bg-avatar! font-semibold text-avatar-ink!";

pub async fn page() -> Html {
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();
    let cancel = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Default,
        "text-[13px] font-semibold",
    );
    let more = button_variants(
        ButtonVariant::Ghost,
        ButtonSize::IconSm,
        "shrink-0 text-muted-foreground",
    );
    let close_picture = button_variants(ButtonVariant::Outline, ButtonSize::Sm, "min-w-24");
    html! {
        <AppShell active="people">
            <div class="flex min-h-0 flex-1">
                <aside class="list-pane flex w-[420px] shrink-0 flex-col">
                    <div class="flex items-center justify-between pl-5 pr-3 pt-1.5">
                        <h1 class="text-[20px] font-semibold tracking-tight text-ink">"People"</h1>
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            class="text-muted-foreground"
                            attrs=@{labelled("People settings")}
                        >
                            <Icon name="settings" class="size-[18px]" />
                        </Button>
                    </div>
                    <ToggleGroup
                        r#type="single"
                        name="people-group"
                        class="mt-3 w-full flex-col! items-stretch gap-1 px-4"
                        attrs=@{named("Contact groups")}
                    >
                        <ToggleGroupItem
                            r#type="single"
                            name="people-group"
                            value="all"
                            pressed
                            class=@{NAV}
                            onchange={setGroup(target.value)}
                        >
                            <Icon name="people" class="size-[18px]" />
                            "All contacts"
                        </ToggleGroupItem>
                        <ToggleGroupItem
                            r#type="single"
                            name="people-group"
                            value="active"
                            class=@{NAV}
                            onchange={setGroup(target.value)}
                        >
                            <Icon name="check-circle" class="size-[18px]" />
                            "Active now"
                        </ToggleGroupItem>
                    </ToggleGroup>
                    <Separator class="mx-5 mt-3 w-auto!" />
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-4 py-3">
                        <div
                            class="flex items-center gap-3 rounded-md px-3 py-2 transition-colors hover:bg-hover"
                        >
                            <ProfileAvatar size="md" presence="online" />
                            <span class="min-w-0 flex-1">
                                <span class="block truncate text-[13px] font-semibold text-ink">
                                    @{user.name.as_str()}
                                </span>
                                <span class="block truncate text-xs text-muted-foreground">
                                    "Your profile"
                                </span>
                            </span>
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="people-profile"
                                    class=@{more.as_str()}
                                    attrs=@{labelled("More options")}
                                >
                                    <span class="contents" pp-ref={profileMenuMark}>
                                        <Icon name="more" class="size-[18px]" />
                                    </span>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                    menu="people-profile"
                                    align="end"
                                    class="people-profile-menu w-[260px]"
                                    attrs=@{named("Profile options")}
                                >
                                    <DropdownMenuItem onclick={editName()}>
                                        <Icon name="compose" class="size-4" />
                                        "Edit name"
                                    </DropdownMenuItem>
                                    <DropdownMenuItem onclick={editPicture()}>
                                        <Icon name="image" class="size-4" />
                                        "Change profile picture"
                                    </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                        </div>
                        <template pp-for="person in shown">
                            <div
                                key={person.email}
                                class="flex items-center gap-3 rounded-md px-3 py-2 transition-colors hover:bg-hover"
                            >
                                <Avatar>
                                    <AvatarFallback class=@{INITIALS}>
                                        {person.initials}
                                    </AvatarFallback>
                                </Avatar>
                                <span class="min-w-0 flex-1">
                                    <span class="block truncate text-[13px] font-semibold text-ink">
                                        {person.email}
                                    </span>
                                    <span class="block truncate text-xs text-muted-foreground">
                                        {person.status}
                                    </span>
                                </span>
                            </div>
                        </template>
                    </div>
                    <div class="shrink-0 p-4">
                        <Button
                            variant="outline"
                            class="w-full gap-3 text-[13px] font-semibold"
                            onclick={openInvite()}
                        >
                            <Icon name="person-add" class="size-[18px]" />
                            "Invite to Teams"
                        </Button>
                    </div>
                </aside>
                <main
                    class="relative flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
                >
                    <header class="flex h-16 shrink-0 items-center justify-between px-8">
                        <h2 class="text-[17px] font-semibold text-ink">
                            {group === "all" ? "All contacts" : "Active now"}
                        </h2>
                        <InputGroup class="h-8! w-64 bg-raised dark:bg-raised">
                            <InputGroupAddon>
                                <Icon name="search" class="size-4" />
                            </InputGroupAddon>
                            <InputGroupInput
                                placeholder="Find a contact"
                                class="text-[13px]"
                                attrs=@{named("Find a contact").set("autocomplete", "off")}
                                value={query}
                                oninput={setQuery(target.value)}
                            />
                        </InputGroup>
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
                            <p class="mt-2.5 text-[13px] leading-relaxed text-muted-foreground">
                                "Sync your device contacts, or invite folks to Teams."
                            </p>
                            <Button class="mt-7 text-[13px] font-semibold" onclick={openInvite()}>
                                "Sync device contacts"
                            </Button>
                        </div>
                        <div class="grid grid-cols-3 gap-4 pt-2" hidden={shown.length === 0}>
                            <template pp-for="person in shown">
                                <Card key={person.email} class="gap-0 bg-raised p-5">
                                    <Avatar size="lg">
                                        <AvatarFallback class=@{INITIALS}>
                                            {person.initials}
                                        </AvatarFallback>
                                    </Avatar>
                                    <p class="mt-4 truncate text-[13px] font-semibold text-ink">
                                        {person.email}
                                    </p>
                                    <p class="mt-1 text-xs text-muted-foreground">
                                        {person.status}
                                    </p>
                                </Card>
                            </template>
                        </div>
                    </div>
                </main>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={inviteOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={inviteCloser}></span>
                    </DialogClose>
                    <DialogContent class="sm:max-w-[460px]!">
                        <DialogHeader>
                            <DialogTitle class="text-[17px]">"Invite to Teams"</DialogTitle>
                            <DialogDescription class="text-[13px]">
                                "They will get an email with a link to this workspace."
                            </DialogDescription>
                        </DialogHeader>
                        <form onsubmit={invite(event)} novalidate>
                            <Field>
                                <FieldLabel
                                    r#for="invite-email"
                                    class="text-xs text-muted-foreground"
                                >
                                    "Email address"
                                </FieldLabel>
                                <Input
                                    id="invite-email"
                                    name="email"
                                    r#type="email"
                                    placeholder="name@example.com"
                                    class="text-[13px]"
                                    attrs=@{Attrs::new().set("autocomplete", "email").set("autofocus", "")}
                                    value={email}
                                    oninput={setEmail(target.value)}
                                />
                                <FieldError hidden={!fieldError}>{fieldError}</FieldError>
                            </Field>
                            <DialogFooter class="mt-7">
                                <DialogClose class=@{cancel.as_str()}>"Cancel"</DialogClose>
                                <Button
                                    class="text-[13px] font-semibold"
                                    attrs=@{submit()}
                                    disabled={sending}
                                >
                                    {sending ? "Sending…" : "Send invite"}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={nameOpener}></span>
                    </DialogTrigger>
                    <DialogContent
                        class="people-profile-dialog sm:max-w-[460px]!"
                        onclose={finishProfile()}
                    >
                        <DialogHeader>
                            <DialogTitle>"Edit name"</DialogTitle>
                        </DialogHeader>
                        <form onsubmit={saveName(event)}>
                            <Field>
                                <FieldLabel r#for="profile-name">"Display name"</FieldLabel>
                                <Input
                                    id="profile-name"
                                    name="name"
                                    required
                                    class="h-10!"
                                    attrs=@{Attrs::new().set("maxlength", "80").set("autofocus", "")}
                                    value={profileName}
                                    oninput={setProfileName(target.value)}
                                    disabled={profileBusy}
                                />
                                <FieldError hidden={!profileError}>{profileError}</FieldError>
                            </Field>
                            <DialogFooter class="mt-6">
                                <DialogClose class=@{cancel.as_str()} disabled={profileBusy}>
                                    "Cancel"
                                </DialogClose>
                                <Button attrs=@{submit()} disabled={profileBusy}>
                                    {profileBusy ? "Saving…" : "Save"}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={pictureOpener}></span>
                    </DialogTrigger>
                    <DialogContent
                        hide_close=@{true}
                        class="people-profile-dialog picture-editor open:flex! h-[min(504px,calc(100dvh-32px))]! w-[min(640px,calc(100vw-32px))]! max-w-none! flex-col gap-0! overflow-hidden px-8! pt-7! pb-10! max-[600px]:p-6!"
                        onclose={finishProfile()}
                    >
                        <DialogHeader>
                            <DialogTitle class="text-xl font-bold">"Profile picture"</DialogTitle>
                        </DialogHeader>
                        <form class="flex min-h-0 flex-1 flex-col" onsubmit={savePicture(event)}>
                            <div
                                class="picture-editor-body flex min-h-0 flex-1 flex-col items-center justify-center gap-[18px]"
                            >
                                <div
                                    class="picture-editor-preview relative flex aspect-square size-[min(196px,38dvh)] shrink-0 items-center justify-center overflow-hidden rounded-full bg-[#d2dceb] text-[76px] font-semibold leading-none text-[#19345b] max-[600px]:text-[64px]"
                                    aria-label="Profile picture preview"
                                >
                                    <span hidden={!!pictureSource}>@{initials.as_str()}</span>
                                    <template
                                        pp-for="source in pictureSource ? [pictureSource] : []"
                                    >
                                        <img
                                            key={source}
                                            src={source}
                                            alt="Profile picture preview"
                                            class="absolute inset-0 size-full object-cover"
                                        />
                                    </template>
                                </div>
                                <div class="picture-editor-actions flex items-center gap-[22px]">
                                    <Button
                                        attrs=@{plain()}
                                        variant="ghost"
                                        class="text-sm font-semibold text-muted-foreground"
                                        onclick={profilePictureInput.current?.click()}
                                        disabled={profileBusy}
                                    >
                                        <Icon name="upload" class="size-[18px]" />
                                        "Upload"
                                    </Button>
                                    <Button
                                        attrs=@{plain()}
                                        variant="ghost"
                                        class="text-sm font-semibold text-muted-foreground"
                                        onclick={removePicture()}
                                        disabled={profileBusy || !pictureSource}
                                    >
                                        <Icon name="trash" class="size-[18px]" />
                                        "Remove"
                                    </Button>
                                    <input
                                        pp-ref={profilePictureInput}
                                        name="picture"
                                        type="file"
                                        accept="image/png,image/jpeg"
                                        disabled={profileBusy}
                                        onchange={choosePicture(event)}
                                        hidden
                                    />
                                </div>
                            </div>
                            <p
                                role="alert"
                                hidden={!profileError}
                                class="mb-4 text-sm text-danger-ink"
                            >
                                {profileError}
                            </p>
                            <DialogFooter class="items-center gap-[18px] max-[600px]:flex-wrap">
                                <p
                                    class="mr-auto text-xs leading-[18px] text-muted-foreground max-[600px]:basis-full"
                                >
                                    "Changes will be reflected in your upurnaka account."
                                </p>
                                <DialogClose class=@{close_picture.as_str()} disabled={profileBusy}>
                                    "Close"
                                </DialogClose>
                                <Button
                                    size="sm"
                                    class="min-w-24"
                                    attrs=@{submit()}
                                    disabled={profileBusy || !pictureChanged}
                                >
                                    {profileBusy ? "Saving…" : "Save"}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
                <script>r##"
                    const [group, setGroup] = pp.state("all");
                    const [query, setQuery] = pp.state("");
                    const [people, setPeople] = pp.state([]);
                    const [email, setEmail] = pp.state("");
                    const [fieldError, setFieldError] = pp.state("");
                    const [sending, setSending] = pp.state(false);
                    const inviteOpener = pp.ref(null);
                    const inviteCloser = pp.ref(null);
                    const nameOpener = pp.ref(null);
                    const pictureOpener = pp.ref(null);
                    const profilePictureInput = pp.ref(null);
                    const profileMenuMark = pp.ref(null);
                    const [profileName, setProfileName] = pp.state(@{user.name.as_str()});
                    const [profileBusy, setProfileBusy] = pp.state(false);
                    const [profileError, setProfileError] = pp.state('');
                    const savedPicture = @{user.has_picture} ? '/profile-picture' : '';
                    const [picturePreview, setPicturePreview] = pp.state('');
                    const [pictureRemoved, setPictureRemoved] = pp.state(false);
                    const pictureSource = picturePreview || (pictureRemoved ? '' : savedPicture);
                    const pictureChanged = !!picturePreview || (pictureRemoved && !!savedPicture);

                    // The three dialogs are rahti-ui's, which open from a trigger;
                    // a menu row or a button reaches each through its hidden one.
                    function editName() {
                        resetProfile();
                        setProfileName(@{user.name.as_str()});
                        nameOpener.current?.click();
                    }
                    pp.effect(() => {
                        if (new URLSearchParams(location.search).get('edit') === 'profile') editName();
                    }, []);
                    function editPicture() {
                        resetProfile();
                        pictureOpener.current?.click();
                    }
                    // Closing a profile dialog hands focus back to the menu button
                    // it was opened from; the menu row itself has closed with the menu.
                    function finishProfile() {
                        resetProfile();
                        profileMenuMark.current?.closest("button")?.focus();
                    }
                    function resetProfile() {
                        setProfileError('');
                        setPicturePreview(''); setPictureRemoved(false);
                        if (profilePictureInput.current) profilePictureInput.current.value = '';
                    }
                    async function choosePicture(event) {
                        const file = event.currentTarget.files[0];
                        if (!file) return;
                        setProfileError('');
                        if (!['image/png', 'image/jpeg'].includes(file.type) || file.size > 2 * 1024 * 1024) {
                            profilePictureInput.current.value = '';
                            setPicturePreview(''); setPictureRemoved(false);
                            setProfileError('Choose a PNG or JPEG picture up to 2 MB.'); return;
                        }
                        try {
                            const source = await new Promise((resolve, reject) => {
                                const reader = new FileReader(); reader.onload = () => resolve(reader.result); reader.onerror = reject; reader.readAsDataURL(file);
                            });
                            if (profilePictureInput.current?.files[0] !== file) return;
                            setPicturePreview(source); setPictureRemoved(false);
                        } catch (failure) {setProfileError('This picture could not be read. Choose another file.');}
                    }
                    function removePicture() {
                        if (profileBusy) return;
                        profilePictureInput.current.value = '';
                        setPicturePreview(''); setPictureRemoved(true); setProfileError('');
                    }
                    async function save(work) {
                        setProfileBusy(true); setProfileError('');
                        try {
                            await work();
                            location.reload();
                        } catch (failure) {
                            setProfileError(failure.errors?.name?.[0] || failure.errors?.picture?.[0] || failure.message || 'Your profile could not be saved.');
                        } finally {setProfileBusy(false);}
                    }
                    function saveName(event) {
                        event.preventDefault(); if (profileBusy) return;
                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        save(() => pp.rpc('update_profile_name', data));
                    }
                    function savePicture(event) {
                        event.preventDefault(); if (profileBusy || !pictureChanged) return;
                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        save(() => pictureRemoved ? pp.rpc('remove_profile_picture') : pp.rpc('update_profile_picture', data));
                    }

                    const needle = query.trim().toLowerCase();
                    const shown = people.filter((person) => {
                        if (group === "active" && !person.active) return false;
                        if (!needle) return true;
                        return person.email.toLowerCase().includes(needle);
                    });

                    function openInvite() {
                        setFieldError("");
                        inviteOpener.current?.click();
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
                            inviteCloser.current?.click();
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

// Somebody who has been invited but has not joined yet.
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
