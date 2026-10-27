//! People: the contact list, and the invitation that fills it.

use axum::http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::avatar::ProfileAvatar;
use crate::components::icon::Icon;
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

pub async fn page() -> Html {
    let user = crate::auth::current().unwrap_or_default();
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
                            <ProfileAvatar size="md" presence="online" />
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
                                aria-haspopup="menu"
                                aria-expanded={profileMenuOpen}
                                aria-controls={profileMenuId}
                                pp-ref={profileTrigger}
                                onclick={openProfileMenu()}
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
                <div
                    id={profileMenuId}
                    pp-ref={profileMenu}
                    popover="auto"
                    role="menu"
                    aria-label="Profile options"
                    class="people-profile-menu rounded-md border border-edge bg-raised p-1 text-sm text-ink shadow-xl"
                    pp-style={profileMenuPosition}
                    ontoggle={setProfileMenuOpen(event.newState === "open")}
                    onkeydown={profileMenuKey(event)}
                >
                    <button
                        type="button"
                        role="menuitem"
                        pp-ref={editNameAction}
                        class="flex w-full items-center gap-3 rounded px-3 py-2 text-left hover:bg-hover focus-visible:outline-2 focus-visible:outline-brand-ink"
                        onclick={editProfile("name")}
                    >
                        <Icon name="compose" class="h-4 w-4" />
                        "Edit name"
                    </button>
                    <button
                        type="button"
                        role="menuitem"
                        pp-ref={editPictureAction}
                        class="flex w-full items-center gap-3 rounded px-3 py-2 text-left hover:bg-hover focus-visible:outline-2 focus-visible:outline-brand-ink"
                        onclick={editProfile("picture")}
                    >
                        <Icon name="image" class="h-4 w-4" />
                        "Change profile picture"
                    </button>
                </div>
                <dialog
                    pp-ref={profileDialog}
                    class={profileMode === "picture" ? "people-profile-dialog picture-editor m-auto rounded-lg text-ink shadow-2xl" : "people-profile-dialog m-auto w-full max-w-[460px] rounded-lg border border-edge bg-raised p-7 text-ink shadow-2xl"}
                    aria-labelledby={profileTitleId}
                    oncancel={cancelProfile(event)}
                >
                    <h2 id={profileTitleId} class="text-lg font-semibold">
                        {profileMode === "name" ? "Edit name" : "Profile picture"}
                    </h2>
                    <form class="mt-5 space-y-4" onsubmit={saveProfile(event)}>
                        <label class="block text-sm" hidden={profileMode !== "name"}>
                            "Display name"
                            <input
                                pp-ref={profileNameInput}
                                name="name"
                                value={profileName}
                                oninput={setProfileName(target.value)}
                                disabled={profileMode !== "name" || profileBusy}
                                required
                                maxlength="80"
                                class="mt-2 h-10 w-full rounded border border-edge bg-surface px-3 outline-none focus:border-brand-ink"
                            />
                        </label>
                        <div class="picture-editor-body" hidden={profileMode !== "picture"}>
                            <div
                                class="picture-editor-preview"
                                aria-label="Profile picture preview"
                            >
                                <span hidden={!!pictureSource}>@{user.initials()}</span>
                                <template pp-for="source in pictureSource ? [pictureSource] : []">
                                    <img key={source} src={source} alt="Profile picture preview" />
                                </template>
                            </div>
                            <div class="picture-editor-actions">
                                <button
                                    type="button"
                                    class="picture-editor-action"
                                    pp-ref={profileUploadButton}
                                    onclick={profilePictureInput.current?.click()}
                                    disabled={profileBusy}
                                >
                                    <svg
                                        width="24"
                                        height="24"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="1.4"
                                        aria-hidden="true"
                                    >
                                        <path d="M4 3h16M12 22V6m-7 7 7-7 7 7" />
                                    </svg>
                                    "Upload"
                                </button>
                                <button
                                    type="button"
                                    class="picture-editor-action"
                                    onclick={removePicture()}
                                    disabled={profileBusy || !pictureSource}
                                >
                                    <Icon name="trash" class="h-6 w-6" />
                                    "Remove"
                                </button>
                                <input
                                    pp-ref={profilePictureInput}
                                    name="picture"
                                    type="file"
                                    accept="image/png,image/jpeg"
                                    disabled={profileMode !== "picture" || profileBusy}
                                    onchange={choosePicture(event)}
                                    hidden
                                />
                            </div>
                        </div>
                        <p role="alert" hidden={!profileError} class="text-sm text-danger-ink">
                            {profileError}
                        </p>
                        <div class="picture-editor-footer flex justify-end gap-3">
                            <p hidden={profileMode !== "picture"}>
                                "Changes will be reflected in your upurnaka account."
                            </p>
                            <button
                                type="button"
                                onclick={closeProfile()}
                                disabled={profileBusy}
                                class="h-9 rounded border border-edge px-4 hover:bg-hover"
                            >
                                {profileMode === "picture" ? "Close" : "Cancel"}
                            </button>
                            <button
                                type="submit"
                                disabled={profileBusy || (profileMode === "picture" && !pictureChanged)}
                                class="h-9 rounded bg-brand px-4 font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                            >
                                {profileBusy ? "Saving…" : "Save"}
                            </button>
                        </div>
                    </form>
                </dialog>
                <style>r##"
                    .people-profile-menu {position:fixed;inset:auto;margin:0;width:260px;max-width:calc(100vw - 16px);}
                    .people-profile-dialog::backdrop {background:#0009;}
                    .picture-editor {box-sizing:border-box;width:min(640px,calc(100vw - 32px));height:min(504px,calc(100dvh - 32px));max-width:none;max-height:none;padding:28px 32px 40px;background:#292929;border:0;border-radius:8px;color:#f5f5f5;}
                    .picture-editor h2 {font-size:20px;line-height:24px;font-weight:700;}
                    .picture-editor form {display:flex;flex-direction:column;height:calc(100% - 24px);margin:0;}
                    .picture-editor form > * {margin-block:0;}
                    .picture-editor-body {flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:18px;min-height:0;}
                    .picture-editor-preview {position:relative;display:flex;align-items:center;justify-content:center;width:196px;height:196px;max-width:38dvh;max-height:38dvh;aspect-ratio:1;flex-shrink:0;overflow:hidden;border-radius:50%;background:#d2dceb;color:#19345b;font-size:76px;font-weight:600;line-height:1;}
                    .picture-editor-preview img {position:absolute;inset:0;width:100%;height:100%;object-fit:cover;}
                    .picture-editor-actions {display:flex;align-items:center;gap:22px;}
                    .picture-editor-action {display:flex;align-items:center;gap:8px;padding:6px 8px;border-radius:4px;color:#ccc;font-size:14px;font-weight:600;}
                    .picture-editor-action svg {width:18px;height:18px;}
                    .picture-editor-action:hover:not(:disabled) {background:#ffffff0d;color:white;}
                    .picture-editor-action:focus-visible {outline:2px solid white;outline-offset:1px;}
                    .picture-editor-action:disabled {opacity:.35;cursor:default;}
                    .picture-editor-footer {align-items:center;gap:18px;}
                    .picture-editor-footer p {margin-right:auto;font-size:12px;line-height:18px;}
                    .picture-editor-footer button {flex-shrink:0;min-width:96px;height:32px;border-radius:4px;font-size:14px;font-weight:600;}
                    .picture-editor-footer button[type=button] {background:transparent;border:1px solid #666;}
                    .picture-editor-footer button[type=submit] {background:#5852b4;}
                    .picture-editor-footer button[type=submit]:disabled {background:#202020;color:#656565;opacity:1;}
                    .picture-editor [role=alert]:not([hidden]) {margin-bottom:16px;font-size:14px;}
                    @media(max-width:600px), (max-height:540px) {
                        .picture-editor {padding:24px;height:min(504px,calc(100dvh - 24px));}
                        .picture-editor-preview {width:min(196px,34dvh);height:min(196px,34dvh);font-size:64px;}
                        .picture-editor-body {gap:16px;}
                        .picture-editor-footer {flex-wrap:wrap;gap:12px;}
                        .picture-editor-footer p {flex-basis:100%;}
                    }
                "##</style>
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
                    const profileMenu = pp.ref(null);
                    const profileTrigger = pp.ref(null);
                    const editNameAction = pp.ref(null);
                    const editPictureAction = pp.ref(null);
                    const profileDialog = pp.ref(null);
                    const profileNameInput = pp.ref(null);
                    const profilePictureInput = pp.ref(null);
                    const profileUploadButton = pp.ref(null);
                    const profileMenuId = pp.id();
                    const profileTitleId = pp.id();
                    const [profileMenuOpen, setProfileMenuOpen] = pp.state(false);
                    const [profileMenuPosition, setProfileMenuPosition] = pp.state('');
                    const [profileMode, setProfileMode] = pp.state('name');
                    const [profileName, setProfileName] = pp.state(@{user.name.as_str()});
                    const [profileBusy, setProfileBusy] = pp.state(false);
                    const [profileError, setProfileError] = pp.state('');
                    const savedPicture = @{user.has_picture} ? '/profile-picture' : '';
                    const [picturePreview, setPicturePreview] = pp.state('');
                    const [pictureRemoved, setPictureRemoved] = pp.state(false);
                    const pictureSource = picturePreview || (pictureRemoved ? '' : savedPicture);
                    const pictureChanged = !!picturePreview || (pictureRemoved && !!savedPicture);
                    function openProfileMenu() {
                        if (profileMenu.current.matches(':popover-open')) {profileMenu.current.hidePopover(); return;}
                        const anchor = profileTrigger.current.getBoundingClientRect();
                        setProfileMenuPosition('left:' + Math.max(8, Math.min(anchor.left, innerWidth - 268)) + 'px;top:' + Math.max(8, Math.min(anchor.bottom + 4, innerHeight - 100)) + 'px');
                        profileMenu.current.showPopover(); editNameAction.current?.focus();
                    }
                    function profileMenuKey(event) {
                        if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
                            event.preventDefault();
                            const first = event.key === 'Home' || (event.key !== 'End' && document.activeElement !== editNameAction.current);
                            (first ? editNameAction : editPictureAction).current?.focus();
                        } else if (event.key === 'Escape' || event.key === 'Tab') {
                            if (event.key === 'Escape') event.preventDefault();
                            profileMenu.current.hidePopover(); profileTrigger.current?.focus();
                        }
                    }
                    function editProfile(mode) {
                        profileMenu.current.hidePopover(); setProfileMode(mode); setProfileError('');
                        setProfileName(@{user.name.as_str()});
                        setPicturePreview(''); setPictureRemoved(false);
                        if (profilePictureInput.current) profilePictureInput.current.value = '';
                        profileDialog.current.showModal();
                    }
                    pp.effect(() => {
                        if (!profileDialog.current?.open) return;
                        (profileMode === 'name' ? profileNameInput : profileUploadButton).current?.focus();
                    }, [profileMode]);
                    function closeProfile() {
                        if (profileBusy) return;
                        profileDialog.current.close(); profileTrigger.current?.focus();
                        if (profilePictureInput.current) profilePictureInput.current.value = '';
                        setPicturePreview(''); setPictureRemoved(false);
                    }
                    function cancelProfile(event) {event.preventDefault(); closeProfile();}
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
                    async function saveProfile(event) {
                        event.preventDefault(); if (profileBusy || (profileMode === 'picture' && !pictureChanged)) return;
                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        setProfileBusy(true); setProfileError('');
                        try {
                            if (profileMode === 'name') await pp.rpc('update_profile_name', data);
                            else if (pictureRemoved) await pp.rpc('remove_profile_picture');
                            else await pp.rpc('update_profile_picture', data);
                            location.reload();
                        } catch (failure) {
                            setProfileError(failure.errors?.name?.[0] || failure.errors?.picture?.[0] || failure.message || 'Your profile could not be saved.');
                        } finally {setProfileBusy(false);}
                    }

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
