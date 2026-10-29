//! Communities: the invitation to start one, and the ones already started.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::aria::{hidden, submit};
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::card::{Card, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger,
};
use crate::components::rahti_ui::field::{Field, FieldLabel};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::textarea::Textarea;
use crate::rahti::{Html, Metadata, Result, RpcFile, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Communities")
        .description("Bring your community together in one place.")
}

/// A template chip under "Create with a template".
const TEMPLATE: &str = "rounded-full text-[13px] font-semibold";

pub async fn page() -> Html {
    let cancel_link = button_variants(
        ButtonVariant::Link,
        ButtonSize::Default,
        "text-[13px] text-link",
    );
    html! {
        <AppShell active="communities">
            <main
                class="ambient-strong scroll-thin relative min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line"
            >
                <div
                    class="mx-auto grid w-full max-w-[1280px] grid-cols-2 items-center gap-12 px-16 py-16"
                    hidden={communities.length > 0}
                >
                    <div>
                        <h1 class="text-[30px] font-semibold leading-tight tracking-tight text-ink">
                            "Build your community"
                        </h1>
                        <p
                            class="mt-5 max-w-[320px] text-[14px] leading-relaxed text-muted-foreground"
                        >
                            "Bring your community together in one place to plan events, stay organized, and get more done."
                        </p>
                        <Button
                            size="lg"
                            class="mt-8 text-[13px] font-semibold"
                            onclick={openCreate("")}
                        >
                            "Create your own"
                        </Button>
                        <h2 class="mt-12 text-[17px] font-semibold text-ink">
                            "Create with a template"
                        </h2>
                        <div class="mt-5 flex items-center gap-3">
                            <Button
                                variant="outline"
                                size="sm"
                                class=@{TEMPLATE}
                                onclick={openCreate("School")}
                            >
                                <span class="leading-none">"🎓"</span>
                                "School"
                            </Button>
                            <Button
                                variant="outline"
                                size="sm"
                                class=@{TEMPLATE}
                                onclick={openCreate("Business")}
                            >
                                <span class="leading-none">"🏪"</span>
                                "Business"
                            </Button>
                            <Button
                                variant="outline"
                                size="sm"
                                class=@{TEMPLATE}
                                onclick={openCreate("Life")}
                            >
                                <span class="leading-none">"🏅"</span>
                                "Life"
                            </Button>
                        </div>
                    </div>
                    <div class="relative h-[460px]">
                        <div
                            class="absolute inset-x-6 bottom-6 top-6 rounded-2xl bg-linear-to-br from-[#4b3f9e] to-[#2a2560]"
                        >
                        </div>
                        <div
                            class="absolute inset-x-12 bottom-14 top-14 rounded-xl bg-white/95 p-5 shadow-2xl shadow-black/50"
                        >
                            <div
                                class="grid h-full grid-cols-6 grid-rows-6 gap-1 rounded-lg bg-[#f1f1f6] p-2"
                            >
                                <div
                                    class="col-span-3 col-start-2 row-span-2 row-start-4 rounded bg-[#a7e3b0]"
                                >
                                </div>
                                <div
                                    class="col-span-2 col-start-5 row-span-3 row-start-3 rounded bg-[#a9c9f5]"
                                >
                                </div>
                                <div
                                    class="col-span-2 col-start-1 row-span-1 row-start-6 rounded bg-[#d9d2f7]"
                                >
                                </div>
                            </div>
                        </div>
                        <div
                            class="absolute left-0 top-14 w-[330px] rounded-xl bg-linear-to-br from-[#c9bcff] to-[#9f8ff2] p-5 shadow-2xl shadow-black/40"
                        >
                            <p class="text-[11px] font-semibold text-[#2d2258]">
                                "Non-profits, mentoring, fundraising"
                            </p>
                            <p class="mt-2 text-[26px] font-semibold leading-tight text-[#1d1540]">
                                "Time to rally the volunteers"
                            </p>
                        </div>
                        <div
                            class="absolute right-2 top-28 flex w-[290px] items-center gap-3 rounded-xl bg-[#2a2550]/95 px-4 py-3 shadow-2xl shadow-black/50"
                        >
                            <span
                                class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-white/15 text-ink"
                            >
                                <Icon name="calendar-day" class="h-[18px] w-[18px]" />
                            </span>
                            <span class="min-w-0 flex-1">
                                <span class="block truncate text-[13px] font-semibold text-ink">
                                    "Onboard volunteers"
                                </span>
                                <span class="block truncate text-[11px] text-muted-foreground">
                                    "Monday 10:00 AM"
                                </span>
                            </span>
                            <span
                                class="shrink-0 rounded bg-brand px-3 py-1 text-[11px] font-semibold text-white"
                            >
                                "Join"
                            </span>
                        </div>
                    </div>
                </div>
                <div
                    class="mx-auto w-full max-w-[1100px] px-16 py-12"
                    hidden={communities.length === 0}
                >
                    <div class="flex items-center justify-between">
                        <h1 class="text-[22px] font-semibold tracking-tight text-ink">
                            "Your communities"
                        </h1>
                        <Button class="text-[13px] font-semibold" onclick={openCreate("")}>
                            "Create your own"
                        </Button>
                    </div>
                    <div class="mt-7 grid grid-cols-3 gap-5">
                        <template pp-for="community in communities">
                            <div
                                key={community.id}
                                tabindex="-1"
                                pp-ref={element => communityElements.current[community.id] = element}
                                class={selectedCommunity === community.id ? "rounded-xl outline-2 outline-brand-ink" : ""}
                            >
                                <Card class="gap-4 overflow-hidden bg-raised pt-0 pb-5">
                                    <div
                                        class="flex h-24 items-center justify-center bg-linear-to-br from-[#6f63d8] to-[#3a3270] text-5xl"
                                        aria-hidden="true"
                                    >
                                        <img
                                            src={community.picture || "/images/upurnaka-logo.png"}
                                            alt=""
                                            class="h-full w-full object-cover"
                                            hidden={!community.picture}
                                        />
                                        <span hidden={!!community.picture}>{community.avatar}</span>
                                    </div>
                                    <CardHeader class="gap-1 px-5">
                                        <CardTitle class="truncate text-[15px]">
                                            {community.name}
                                        </CardTitle>
                                        <CardDescription class="text-xs">
                                            {community.category}
                                        </CardDescription>
                                        <p
                                            class="line-clamp-2 text-sm text-muted-foreground"
                                            hidden={!community.description}
                                        >
                                            {community.description}
                                        </p>
                                    </CardHeader>
                                    <CardFooter class="px-5 text-xs text-subtle">
                                        "1 member · you are the owner"
                                    </CardFooter>
                                </Card>
                            </div>
                        </template>
                    </div>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={createOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={createCloser}></span>
                    </DialogClose>
                    <DialogContent class="community-create-dialog" hide_close=@{true}>
                        <div class="flex items-start justify-between gap-6">
                            <DialogHeader class="text-left!">
                                <DialogTitle class="text-[24px] font-bold leading-tight">
                                    {step === 2 ? "Choose a category" : template ? "Create your " + template.toLowerCase() + " community" : "Create your community"}
                                </DialogTitle>
                            </DialogHeader>
                            <button
                                type="button"
                                class="community-create-close"
                                aria-label="Close community creation"
                                onclick={createCloser.current?.click()}
                            >
                                <Icon name="close" class="size-6" />
                            </button>
                        </div>
                        <form
                            class="community-create-form"
                            onsubmit={goToCategory(event)}
                            novalidate
                            hidden={step !== 1}
                        >
                            <div class="community-identity">
                                <button
                                    type="button"
                                    class="community-avatar"
                                    aria-label="Edit community image"
                                    aria-describedby="community-image-help"
                                    disabled={saving}
                                    onclick={openImagePicker()}
                                >
                                    <img
                                        src={picturePreview || "/images/upurnaka-logo.png"}
                                        alt=""
                                        class="community-avatar-photo"
                                        hidden={!picturePreview}
                                        onerror={rejectPicture()}
                                    />
                                    <span
                                        class="community-avatar-emoji"
                                        aria-hidden="true"
                                        hidden={!!picturePreview}
                                    >
                                        {avatar}
                                    </span>
                                    <span class="community-avatar-edit">"Edit"</span>
                                </button>
                                <input
                                    id="community-picture"
                                    type="file"
                                    accept="image/png,image/jpeg"
                                    pp-ref={pictureInput}
                                    onchange={choosePicture(event)}
                                    hidden
                                />
                                <Field class="min-w-0 flex-1 gap-1">
                                    <FieldLabel
                                        r#for="community-name"
                                        class="text-[15px] font-normal"
                                    >
                                        "Community name"
                                    </FieldLabel>
                                    <Input
                                        id="community-name"
                                        name="name"
                                        placeholder="Enter a community name"
                                        class="community-name-input"
                                        attrs=@{Attrs::new().set("autocomplete", "off").set("autofocus", "").set("maxlength", "80")}
                                        value={name}
                                        oninput={setName(target.value)}
                                        invalid={!!fieldErrors.name}
                                    />
                                    <p
                                        class="text-xs text-busy"
                                        role="alert"
                                        hidden={!fieldErrors.name}
                                    >
                                        {fieldErrors.name?.join(" ")}
                                    </p>
                                </Field>
                            </div>
                            <p id="community-image-help" class="mt-2 text-xs text-muted-foreground">
                                "PNG or JPEG, up to 2 MB."
                            </p>
                            <p
                                class="mt-2 text-sm text-busy"
                                role="alert"
                                hidden={!pictureError && !fieldErrors.picture}
                            >
                                {pictureError || fieldErrors.picture?.join(" ")}
                            </p>
                            <Field class="mt-8 gap-2">
                                <FieldLabel
                                    r#for="community-description"
                                    class="text-[15px] font-normal"
                                >
                                    "Description (optional)"
                                </FieldLabel>
                                <Textarea
                                    id="community-description"
                                    name="description"
                                    class="community-description"
                                    placeholder="Write a short description about your community so people know what it's about."
                                    attrs=@{Attrs::new().set("maxlength", "1000")}
                                    value={description}
                                    oninput={setDescription(target.value)}
                                />
                                <p
                                    class="text-xs text-busy"
                                    role="alert"
                                    hidden={!fieldErrors.description}
                                >
                                    {fieldErrors.description?.join(" ")}
                                </p>
                            </Field>
                            <div class="mt-8">
                                <div class="mb-2 flex items-center justify-between gap-4">
                                    <label for="community-guidelines" class="text-[15px]">
                                        "Community guidelines"
                                    </label>
                                    <button
                                        type="button"
                                        class="community-guidelines-edit"
                                        aria-label={editingGuidelines ? "Done editing community guidelines" : "Edit community guidelines"}
                                        aria-expanded={editingGuidelines}
                                        onclick={toggleGuidelines()}
                                    >
                                        <span hidden={editingGuidelines}>
                                            <Icon name="edit" class="size-5" />
                                        </span>
                                        <span hidden={!editingGuidelines}>"Done"</span>
                                    </button>
                                </div>
                                <p class="community-guidelines-preview" hidden={editingGuidelines}>
                                    {guidelines}
                                </p>
                                <textarea
                                    id="community-guidelines"
                                    name="guidelines"
                                    class="community-description w-full focus-visible:outline-2 focus-visible:outline-brand-ink"
                                    maxlength="2000"
                                    pp-ref={guidelinesInput}
                                    value={guidelines}
                                    oninput={setGuidelines(target.value)}
                                    hidden={!editingGuidelines}
                                >
                                </textarea>
                                <p
                                    class="text-xs text-busy"
                                    role="alert"
                                    hidden={!fieldErrors.guidelines}
                                >
                                    {fieldErrors.guidelines?.join(" ")}
                                </p>
                            </div>
                            <p class="mt-4 text-sm text-busy" role="alert" hidden={!error}>
                                {error}
                            </p>
                            <DialogFooter class="community-create-footer">
                                <Button
                                    class="min-w-32 text-[15px] font-semibold"
                                    attrs=@{submit()}
                                    disabled={saving || name.trim().length < 2 || (picture && !picturePreview)}
                                >
                                    {saving ? "Creating…" : template ? "Create" : "Next"}
                                </Button>
                            </DialogFooter>
                        </form>
                        <div hidden={step !== 2}>
                            <div class="flex flex-col gap-3">
                                <template pp-for="category in CATEGORIES">
                                    <Button
                                        key={category}
                                        variant="outline"
                                        size="lg"
                                        class="h-12! w-full text-[13px] font-semibold hover:border-brand"
                                        disabled={saving}
                                        onclick={create(category)}
                                    >
                                        {category}
                                    </Button>
                                </template>
                            </div>
                            <p class="mt-4 text-center text-xs text-busy" hidden={!error}>
                                {error}
                            </p>
                            <DialogFooter class="mt-7 sm:justify-between!">
                                <Button
                                    variant="outline"
                                    class="w-24 text-[13px] font-semibold"
                                    onclick={setStep(1)}
                                >
                                    "Back"
                                </Button>
                                <DialogClose class=@{cancel_link.as_str()}>"Cancel"</DialogClose>
                            </DialogFooter>
                        </div>
                    </DialogContent>
                </Dialog>
                <style>r##"
                    .community-create-dialog {
                        width: min(800px, calc(100vw - 40px)); max-width: 800px;
                        min-height: min(740px, calc(100dvh - 40px));
                        max-height: calc(100dvh - 40px); overflow-y: auto;
                        padding: 32px; border: 1px solid #d6d6d6; border-radius: 5px;
                        background: #292929; color: #f5f5f5; gap: 34px;
                        grid-template-rows: auto 1fr;
                    }
                    .community-create-close { color: #ccc; padding: 4px; border-radius: 4px; }
                    .community-create-close:hover { background: #3d3d3d; }
                    .community-create-form { display: flex; flex-direction: column; min-height: 0; }
                    .community-identity { display: flex; align-items: center; gap: 26px; }
                    .community-avatar { position: relative; flex-shrink: 0; width: 118px; height: 118px; overflow: hidden; border: 1px solid #777; border-radius: 5px; background: linear-gradient(140deg, #faedcf, #eccc83); }
                    .community-avatar-emoji { display: grid; place-items: center; height: 100%; padding-bottom: 12px; font-size: 65px; }
                    .community-avatar-photo { width: 100%; height: 100%; object-fit: cover; }
                    .community-avatar-edit { position: absolute; bottom: 0; inset-inline: 0; background: #625332cc; padding: 3px; color: white; font-size: 16px; }
                    .community-name-input { height: 58px; border: 0; border-bottom: 1px solid #939393; border-radius: 0; background: transparent; padding: 4px 12px; font-size: 30px; font-weight: 700; box-shadow: none; }
                    .community-name-input:focus-visible { border-color: #a6a0ff; outline: 2px solid #a6a0ff; outline-offset: 2px; }
                    .community-description { min-height: 110px; resize: vertical; border: 0; border-radius: 5px; background: #202020; padding: 12px 16px; font-size: 16px; line-height: 1.5; }
                    .community-description::placeholder { color: #999; }
                    .community-guidelines-preview { white-space: pre-wrap; overflow-wrap: anywhere; font-size: 16px; line-height: 1.5; padding-right: 24px; }
                    .community-guidelines-edit { color: #ccc; padding: 4px; border-radius: 4px; font-size: 13px; }
                    .community-guidelines-edit:hover { background: #3d3d3d; }
                    .community-create-footer { margin-top: auto; padding-top: 40px; }
                    @media (max-width: 540px) {
                        .community-create-dialog { padding: 22px; gap: 26px; }
                        .community-identity { gap: 16px; }
                        .community-avatar { width: 84px; height: 84px; }
                        .community-avatar-emoji { font-size: 44px; }
                        .community-name-input { font-size: 23px; }
                    }
                "##</style>
                <script>r##"
                    const [communities, setCommunities] = pp.state([]);
                    const searchStore = pp.context(window.UpurnakaSearchContext);
                    const communityElements = pp.ref({});
                    const [selectedCommunity, setSelectedCommunity] = pp.state('');
                    const openSearchCommunity = pp.event(row => {
                        setSelectedCommunity(row.id);
                        requestAnimationFrame(() => {
                            const card = communityElements.current[row.id];
                            card?.focus({preventScroll:true}); card?.scrollIntoView({block:'center'});
                        });
                    });
                    pp.effect(() => {
                        searchStore?.publish('communities',communities.map(community => ({key:'community:'+community.id,id:community.id,kind:'communities',title:community.name,subtitle:community.category})),openSearchCommunity);
                    },[communities,searchStore,openSearchCommunity]);
                    pp.effect(() => () => searchStore?.remove('communities'),[]);
                    const createOpener = pp.ref(null);
                    const createCloser = pp.ref(null);
                    const CATEGORIES = [
                        "Family and friends", "Group projects", "Individual projects",
                        "Hobbies", "Events", "Gaming or recreation",
                    ];
                    const DEFAULT_GUIDELINES = "Be kind and respectful to your fellow community members. Don't be rude or cruel. Participate as yourself and don't post anything that violates community guidelines.";
                    const pictureInput = pp.ref(null);
                    const [picture, setPicture] = pp.state(null);
                    const [picturePreview, setPicturePreview] = pp.state("");
                    const [pictureError, setPictureError] = pp.state("");
                    const pictureLoaded = pp.event(event => setPicturePreview(String(event.target.result || "")));
                    const pictureFailed = pp.event(() => rejectPicture());
                    pp.effect(() => {
                        if (!picture) return;
                        const reader = new FileReader();
                        reader.onload = pictureLoaded;
                        reader.onerror = pictureFailed;
                        reader.readAsDataURL(picture);
                        return () => { reader.onload = null; reader.onerror = null; if (reader.readyState === 1) reader.abort(); };
                    }, [picture]);
                    const guidelinesInput = pp.ref(null);
                    const [description, setDescription] = pp.state("");
                    const [guidelines, setGuidelines] = pp.state(DEFAULT_GUIDELINES);
                    const [avatar, setAvatar] = pp.state("👥");
                    const [editingGuidelines, setEditingGuidelines] = pp.state(false);
                    pp.effect(() => {
                        if (editingGuidelines) guidelinesInput.current?.focus();
                    }, [editingGuidelines]);
                    const [fieldErrors, setFieldErrors] = pp.state({});
                    const [step, setStep] = pp.state(1);
                    const [name, setName] = pp.state("");
                    const [template, setTemplate] = pp.state("");
                    const [error, setError] = pp.state("");
                    const [saving, setSaving] = pp.state(false);

                    function openCreate(fromTemplate) {
                        setTemplate(fromTemplate);
                        setName(fromTemplate);
                        setDescription("");
                        setGuidelines(DEFAULT_GUIDELINES);
                        setAvatar({School:"🎓",Business:"🏪",Life:"🏅"}[fromTemplate] || "👥");
                        setPicture(null);
                        setPicturePreview("");
                        setPictureError("");
                        if (pictureInput.current) pictureInput.current.value = "";
                        setEditingGuidelines(false);
                        setFieldErrors({});
                        setError("");
                        setStep(1);
                        createOpener.current?.click();
                    }

                    function openImagePicker() {
                        if (saving || !pictureInput.current) return;
                        pictureInput.current.value = "";
                        pictureInput.current.click();
                    }

                    function choosePicture(event) {
                        const file = event.currentTarget.files[0];
                        if (!file) return;
                        if (!['image/png', 'image/jpeg'].includes(file.type) || file.size === 0 || file.size > 2 * 1024 * 1024) {
                            setPictureError("Choose a PNG or JPEG picture up to 2 MB.");
                            return;
                        }
                        setPictureError("");
                        setFieldErrors({...fieldErrors, picture: undefined});
                        setPicturePreview("");
                        setPicture(file);
                    }

                    function rejectPicture() {
                        setPicture(null);
                        setPicturePreview("");
                        setPictureError("This picture could not be read. Choose a valid PNG or JPEG.");
                    }

                    function toggleGuidelines() {
                        const editing = !editingGuidelines;
                        setEditingGuidelines(editing);
                    }

                    function goToCategory(event) {
                        event.preventDefault();
                        if (saving || name.trim().length < 2 || (picture && !picturePreview)) return;
                        if (template) create(template);
                        else setStep(2);
                    }

                    async function create(category) {
                        if (saving) return;
                        setSaving(true);
                        setError("");
                        try {
                            const community = await pp.rpc("create_community", {
                                name: name.trim(),
                                category: category,
                                description: description.trim(),
                                guidelines: guidelines.trim(),
                                avatar: avatar,
                                picture: picture,
                            });
                            setCommunities((current) => current.concat([community]));
                            createCloser.current?.click();
                        } catch (failure) {
                            setFieldErrors(failure.errors || {});
                            setError(failure.message || "The community could not be created.");
                            if (failure.errors && !failure.errors.category) setStep(1);
                        } finally {
                            setSaving(false);
                        }
                    }
                "##</script>
            </main>
        </AppShell>
    }
}

/// A community this account owns.
#[derive(Serialize)]
pub struct Community {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub guidelines: String,
    pub avatar: String,
    pub picture: Option<String>,
}

/// The categories the create flow offers. A value outside this list is a 422,
/// so a hand-made request cannot invent one.
const CATEGORIES: [&str; 9] = [
    "Family and friends",
    "Group projects",
    "Individual projects",
    "Hobbies",
    "Events",
    "Gaming or recreation",
    "School",
    "Business",
    "Life",
];

#[rpc(auth)]
pub async fn create_community(
    name: String,
    category: String,
    description: Option<String>,
    guidelines: Option<String>,
    avatar: Option<String>,
    picture: Option<RpcFile>,
) -> Result<Community> {
    let name = name.trim().to_string();
    let category = category.trim().to_string();
    let description = description.unwrap_or_default().trim().to_string();
    let guidelines = guidelines.unwrap_or_default().trim().to_string();
    let avatar = avatar.unwrap_or_else(|| "👥".to_string());
    let allowed_images: Vec<String> = ["🎓", "🏪", "🏅", "👥", "🌱", "⚽", "🎨", "💡"]
        .iter()
        .map(|value| value.to_string())
        .collect();
    let allowed: Vec<String> = CATEGORIES.iter().map(|value| value.to_string()).collect();

    let mut validator = Validator::new();
    validator
        .required("name", &name)
        .min_chars("name", &name, 2)
        .max_chars("name", &name, 80)
        .one_of("category", &category, &allowed)
        .max_chars("description", &description, 1000)
        .max_chars("guidelines", &guidelines, 2000)
        .one_of("avatar", &avatar, &allowed_images);
    validator.finish()?;

    let picture = match picture {
        Some(file) => Some(community_picture(file).await?),
        None => None,
    };

    let id = format!("{:x}", crate::rahti::time::now().timestamp_micros());

    Ok(Community {
        id,
        name,
        category,
        description,
        guidelines,
        avatar,
        picture,
    })
}

/// Decode the content rather than trusting the upload's name or MIME type.
/// Return only a bounded, re-encoded thumbnail to the community card.
async fn community_picture(picture: RpcFile) -> Result<String> {
    use base64::Engine;

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
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(normalized.unwrap())
    ))
}
