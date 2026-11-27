//! My account: the profile others see in meetings, and sign-in security.

use axum::http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};

use crate::{
    components::{
        app_shell::AppShell,
        aria::{hidden, labelled, named, plain, submit},
        avatar::ProfileAvatar,
        icon::Icon,
        rahti_ui::{
            attrs::Attrs,
            button::{Button, ButtonSize, ButtonVariant, button_variants},
            card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
            dialog::{
                Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle,
                DialogTrigger,
            },
            dropdown_menu::{
                DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
            },
            field::{Field, FieldError, FieldLabel},
            input::Input,
        },
    },
    rahti::{Error, Html, Metadata, Result, RpcFile, Validator, html, rpc},
};

pub fn metadata() -> Metadata {
    Metadata::new().title("My account")
}

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

pub async fn page() -> Result<Html> {
    crate::auth::ready()?;
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();
    let role_label = if user.role == crate::auth::Role::Admin {
        "Admin"
    } else {
        "User"
    };
    let remaining = crate::models::account::Entity::find_by_id(&user.id)
        .one(crate::db::db())
        .await?
        .map(|a| a.recovery_hashes.lines().count())
        .unwrap_or(0);
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
    Ok(html! {
        <AppShell active="account">
            <main class="flex-1 overflow-y-auto p-8 sm:p-12">
                <div class="max-w-2xl">
                    <h1 class="text-3xl font-semibold">"My account"</h1>
                    <p class="mb-8 mt-2 text-sm text-muted-foreground">
                        "Your profile and sign-in security."
                    </p>
                    <Card class="bg-surface">
                        <CardHeader>
                            <CardTitle class="text-lg">"Profile"</CardTitle>
                            <CardDescription>
                                "Your name and picture are what others see in meetings and chat."
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div class="mb-5 flex items-center gap-3">
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
                                        menu="account-profile"
                                        class=@{more.as_str()}
                                        attrs=@{labelled("More options")}
                                    >
                                        <span class="contents" pp-ref={profileMenuMark}>
                                            <Icon name="more" class="size-[18px]" />
                                        </span>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent
                                        menu="account-profile"
                                        align="end"
                                        class="profile-menu w-[260px]"
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
                            <dl class="space-y-4 text-sm">
                                <div>
                                    <dt class="text-muted-foreground">"Role"</dt>
                                    <dd class="mt-1">@{role_label}</dd>
                                </div>
                                <div>
                                    <dt class="text-muted-foreground">"Email address"</dt>
                                    <dd class="mt-1">@{user.email.as_str()}</dd>
                                </div>
                            </dl>
                        </CardContent>
                    </Card>
                    <Card class="mt-5 bg-surface">
                        <CardHeader>
                            <CardTitle class="text-lg">"Sign-in security"</CardTitle>
                            <CardDescription class="text-online">
                                "Two-step verification is enabled"
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <p class="text-sm text-muted-foreground">
                                "Sign in with your password and authenticator app."
                            </p>
                            <p class="mt-4 text-sm">@{remaining}" unused recovery codes"</p>
                            <p class="mt-2 text-xs text-muted-foreground">
                                "Keep your recovery codes safe. Each can be used once in place of an authenticator code."
                            </p>
                        </CardContent>
                    </Card>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={nameOpener}></span>
                    </DialogTrigger>
                    <DialogContent
                        class="profile-dialog sm:max-w-[460px]!"
                        onclose={finishProfile()}
                    >
                        <DialogHeader>
                            <DialogTitle>"Edit name"</DialogTitle>
                        </DialogHeader>
                        <form onsubmit={saveName(event)}>
                            <Field>
                                <FieldLabel for="profile-name">"Display name"</FieldLabel>
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
                        class="profile-dialog picture-editor open:flex! h-[min(504px,calc(100dvh-32px))]! w-[min(640px,calc(100vw-32px))]! max-w-none! flex-col gap-0! overflow-hidden px-8! pt-7! pb-10! max-[600px]:p-6!"
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

                    // Both dialogs are rahti-ui's, which open from a trigger;
                    // a menu row reaches each through its hidden one.
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
                "##</script>
            </main>
        </AppShell>
    })
}
