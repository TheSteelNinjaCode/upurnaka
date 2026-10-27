//! Communities: the invitation to start one, and the ones already started.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::aria::{hidden, submit};
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::card::{Card, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
use crate::components::rahti_ui::field::{Field, FieldLabel};
use crate::components::rahti_ui::input::Input;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Communities")
        .description("Bring your community together in one place.")
}

/// A template chip under "Create with a template".
const TEMPLATE: &str = "rounded-full text-[13px] font-semibold";

pub async fn page() -> Html {
    let cancel = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Default,
        "text-[13px] font-semibold",
    );
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
                            <Card
                                key={community.id}
                                class="gap-4 overflow-hidden bg-raised pt-0 pb-5"
                            >
                                <div class="h-24 bg-linear-to-br from-[#6f63d8] to-[#3a3270]"></div>
                                <CardHeader class="gap-1 px-5">
                                    <CardTitle class="truncate text-[15px]">
                                        {community.name}
                                    </CardTitle>
                                    <CardDescription class="text-xs">
                                        {community.category}
                                    </CardDescription>
                                </CardHeader>
                                <CardFooter class="px-5 text-xs text-subtle">
                                    "1 member · you are the owner"
                                </CardFooter>
                            </Card>
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
                    <DialogContent class="sm:max-w-[520px]!">
                        <DialogHeader>
                            <DialogTitle class="text-center text-xl">
                                {step === 1 ? "Name your community" : "Choose a category"}
                            </DialogTitle>
                            <DialogDescription class="text-center" hidden={!template || step !== 1}>
                                "Starting from the "
                                {template}
                                " template."
                            </DialogDescription>
                        </DialogHeader>
                        <form onsubmit={goToCategory(event)} novalidate hidden={step !== 1}>
                            <Field>
                                <FieldLabel
                                    r#for="community-name"
                                    class="text-xs text-muted-foreground"
                                >
                                    "Community name"
                                </FieldLabel>
                                <Input
                                    id="community-name"
                                    name="name"
                                    placeholder="Saturday football club"
                                    class="h-10! text-[13px]"
                                    attrs=@{Attrs::new().set("autocomplete", "off").set("autofocus", "")}
                                    value={name}
                                    oninput={setName(target.value)}
                                />
                            </Field>
                            <DialogFooter class="mt-8">
                                <DialogClose class=@{cancel.as_str()}>"Cancel"</DialogClose>
                                <Button
                                    class="text-[13px] font-semibold"
                                    attrs=@{submit()}
                                    disabled={name.trim().length === 0}
                                >
                                    "Next"
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
                <script>r##"
                    const [communities, setCommunities] = pp.state([]);
                    const createOpener = pp.ref(null);
                    const createCloser = pp.ref(null);
                    const CATEGORIES = [
                        "Family and friends", "Group projects", "Individual projects",
                        "Hobbies", "Events", "Gaming or recreation",
                    ];
                    const [step, setStep] = pp.state(1);
                    const [name, setName] = pp.state("");
                    const [template, setTemplate] = pp.state("");
                    const [error, setError] = pp.state("");
                    const [saving, setSaving] = pp.state(false);

                    function openCreate(fromTemplate) {
                        setTemplate(fromTemplate);
                        setName(fromTemplate ? fromTemplate + " community" : "");
                        setError("");
                        setStep(1);
                        createOpener.current?.click();
                    }

                    function goToCategory(event) {
                        event.preventDefault();
                        if (name.trim().length === 0) return;
                        setStep(2);
                    }

                    async function create(category) {
                        if (saving) return;
                        setSaving(true);
                        setError("");
                        try {
                            const community = await pp.rpc("create_community", {
                                name: name.trim(),
                                category: category,
                            });
                            setCommunities((current) => current.concat([community]));
                            createCloser.current?.click();
                        } catch (failure) {
                            setError(failure.message || "The community could not be created.");
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
}

/// The categories the create flow offers. A value outside this list is a 422,
/// so a hand-made request cannot invent one.
const CATEGORIES: [&str; 6] = [
    "Family and friends",
    "Group projects",
    "Individual projects",
    "Hobbies",
    "Events",
    "Gaming or recreation",
];

#[rpc(auth)]
pub async fn create_community(name: String, category: String) -> Result<Community> {
    let name = name.trim().to_string();
    let category = category.trim().to_string();
    let allowed: Vec<String> = CATEGORIES.iter().map(|value| value.to_string()).collect();

    let mut validator = Validator::new();
    validator
        .required("name", &name)
        .min_chars("name", &name, 2)
        .max_chars("name", &name, 80)
        .one_of("category", &category, &allowed);
    validator.finish()?;

    let id = format!("{:x}", crate::rahti::time::now().timestamp_micros());

    Ok(Community { id, name, category })
}
