//! Communities: the invitation to start one, and the ones already started.

use serde::Serialize;

use crate::components::app_shell::AppShell;
use crate::components::icon::Icon;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Communities")
        .description("Bring your community together in one place.")
}

pub async fn page() -> Html {
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
                        <p class="mt-5 max-w-[320px] text-[14px] leading-relaxed text-muted">
                            "Bring your community together in one place to plan events, stay organized, and get more done."
                        </p>
                        <button
                            type="button"
                            class="mt-8 h-10 rounded-md bg-brand px-6 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                            onclick={openCreate("")}
                        >
                            "Create your own"
                        </button>
                        <h2 class="mt-12 text-[17px] font-semibold text-ink">
                            "Create with a template"
                        </h2>
                        <div class="mt-5 flex items-center gap-3">
                            <button
                                type="button"
                                class="flex h-8 items-center gap-2 rounded-full border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                onclick={openCreate("School")}
                            >
                                <span class="leading-none">"🎓"</span>
                                "School"
                            </button>
                            <button
                                type="button"
                                class="flex h-8 items-center gap-2 rounded-full border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                onclick={openCreate("Business")}
                            >
                                <span class="leading-none">"🏪"</span>
                                "Business"
                            </button>
                            <button
                                type="button"
                                class="flex h-8 items-center gap-2 rounded-full border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                onclick={openCreate("Life")}
                            >
                                <span class="leading-none">"🏅"</span>
                                "Life"
                            </button>
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
                                <span class="block truncate text-[11px] text-muted">
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
                        <button
                            type="button"
                            class="h-9 rounded-md bg-brand px-5 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                            onclick={openCreate("")}
                        >
                            "Create your own"
                        </button>
                    </div>
                    <div class="mt-7 grid grid-cols-3 gap-5">
                        <template pp-for="community in communities">
                            <div
                                key={community.id}
                                class="overflow-hidden rounded-xl border border-edge bg-raised"
                            >
                                <div class="h-24 bg-linear-to-br from-[#6f63d8] to-[#3a3270]"></div>
                                <div class="p-5">
                                    <p class="truncate text-[15px] font-semibold text-ink">
                                        {community.name}
                                    </p>
                                    <p class="mt-1 text-xs text-muted">{community.category}</p>
                                    <p class="mt-4 text-xs text-subtle">
                                        "1 member · you are the owner"
                                    </p>
                                </div>
                            </div>
                        </template>
                    </div>
                </div>
                <div
                    class="absolute inset-0 z-20 flex items-center justify-center bg-black/60 px-6"
                    hidden={!createOpen}
                >
                    <div
                        class="w-full max-w-[520px] rounded-lg border border-edge bg-raised px-9 py-8 shadow-2xl shadow-black/70"
                    >
                        <h2 class="text-center text-xl font-semibold text-ink">
                            {step === 1 ? "Name your community" : "Choose a category"}
                        </h2>
                        <form
                            class="mt-7"
                            onsubmit={goToCategory(event)}
                            novalidate
                            hidden={step !== 1}
                        >
                            <label
                                class="block text-xs font-semibold text-muted"
                                for="community-name"
                            >
                                "Community name"
                            </label>
                            <input
                                id="community-name"
                                name="name"
                                type="text"
                                class="mt-2 h-10 w-full rounded border border-edge bg-surface px-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                                placeholder="Saturday football club"
                                autocomplete="off"
                                value={name}
                                oninput={setName(target.value)}
                            />
                            <p class="mt-3 text-xs text-muted" hidden={!template}>
                                "Starting from the "
                                {template}
                                " template."
                            </p>
                            <div class="mt-8 flex items-center justify-end gap-3">
                                <button
                                    type="button"
                                    class="h-9 rounded border border-edge px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                    onclick={closeCreate()}
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="submit"
                                    class="h-9 rounded bg-brand px-5 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                    disabled={name.trim().length === 0}
                                >
                                    "Next"
                                </button>
                            </div>
                        </form>
                        <div hidden={step !== 2}>
                            <div class="mt-7 flex flex-col gap-3">
                                <button
                                    type="button"
                                    class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                    onclick={create("Family and friends")}
                                >
                                    "Family and friends"
                                </button>
                                <button
                                    type="button"
                                    class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                    onclick={create("Group projects")}
                                >
                                    "Group projects"
                                </button>
                                <button
                                    type="button"
                                    class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                    onclick={create("Individual projects")}
                                >
                                    "Individual projects"
                                </button>
                                <button
                                    type="button"
                                    class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                    onclick={create("Hobbies")}
                                >
                                    "Hobbies"
                                </button>
                                <button
                                    type="button"
                                    class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                    onclick={create("Events")}
                                >
                                    "Events"
                                </button>
                                <button
                                    type="button"
                                    class="h-12 rounded-md border border-edge bg-surface text-[13px] font-semibold text-ink transition-colors hover:border-brand hover:bg-hover"
                                    onclick={create("Gaming or recreation")}
                                >
                                    "Gaming or recreation"
                                </button>
                            </div>
                            <p class="mt-4 text-center text-xs text-busy" hidden={!error}>
                                {error}
                            </p>
                            <div class="mt-7 flex items-center justify-between">
                                <button
                                    type="button"
                                    class="h-9 w-24 rounded border border-edge px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                    onclick={setStep(1)}
                                >
                                    "Back"
                                </button>
                                <button
                                    type="button"
                                    class="text-[13px] text-link hover:underline"
                                    onclick={closeCreate()}
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
                <script>r##"
                    const [communities, setCommunities] = pp.state([]);
                    const [createOpen, setCreateOpen] = pp.state(false);
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
                        setCreateOpen(true);
                    }

                    function closeCreate() {
                        setCreateOpen(false);
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
                            setCreateOpen(false);
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

#[rpc]
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
