use crate::{
    components::{app_shell::AppShell, icon::Icon},
    rahti::{Html, Metadata, html},
};

pub fn metadata() -> Metadata {
    Metadata::new().title("Settings")
}

pub async fn page() -> Html {
    let cloud_ready = crate::meetings::recording::is_configured();
    html! {
        <AppShell active="settings">
            <div class="settings-layout flex min-h-0 min-w-0 flex-1">
                <aside
                    class="settings-sidebar list-pane flex w-[300px] shrink-0 flex-col px-4 pb-5 pt-5"
                >
                    <h1 class="px-3 text-xl font-semibold">"Settings"</h1>
                    <div class="settings-search relative mb-4 mt-6">
                        <span
                            class="pointer-events-none absolute left-3 top-2.5 text-muted-foreground"
                        >
                            <Icon name="search" class="size-4" />
                        </span>
                        <input
                            type="search"
                            class="h-9 w-full rounded-md border border-transparent bg-raised pl-9 pr-3 text-[13px] outline-none focus:border-brand-ink"
                            placeholder="Find in Settings"
                            aria-label="Find in Settings"
                            value={query}
                            oninput={search(target.value)}
                            pp-ref={searchInput}
                        />
                    </div>
                    <nav class="flex flex-col gap-1" aria-label="Settings categories">
                        <template pp-for="item in categories">
                            <button
                                key={item.key}
                                type="button"
                                class={"settings-nav flex w-full items-center gap-3 rounded-md border px-3 py-2 text-left text-[13px] " + (active === item.key ? "border-edge bg-hover text-ink" : "border-transparent text-muted-foreground hover:bg-hover hover:text-ink")}
                                aria-current={active === item.key ? "page" : null}
                                onclick={choose(item.key)}
                                hidden={!matchesCategory(item.key)}
                            >
                                <span class="settings-category-icon" aria-hidden="true">
                                    <span hidden={item.key !== "general"}>
                                        <Icon name="settings" class="size-5" />
                                    </span>
                                    <span hidden={item.key !== "appearance"}>
                                        <Icon name="sparkle" class="size-5" />
                                    </span>
                                    <span hidden={item.key !== "chats"}>
                                        <Icon name="chat" class="size-5" />
                                    </span>
                                    <span hidden={item.key !== "accessibility"}>
                                        <Icon name="format" class="size-5" />
                                    </span>
                                    <span hidden={item.key !== "files"}>
                                        <Icon name="attach" class="size-5" />
                                    </span>
                                </span>
                                {item.label}
                            </button>
                        </template>
                    </nav>
                    <p
                        class="mt-3 px-3 text-sm text-muted-foreground"
                        hidden={matchingCategories.length > 0}
                    >
                        "No settings found. Try storage, theme, or chat."
                    </p>
                    <div class="mt-auto pt-8">
                        <a
                            href="/account"
                            class="flex items-center gap-3 rounded-md px-3 py-2 text-[13px] text-muted-foreground hover:bg-hover hover:text-ink"
                        >
                            <Icon name="contacts" class="size-5" />
                            "My account"
                        </a>
                        <p class="mt-3 px-3 text-xs text-subtle">
                            "Upurnaka · Settings for this browser"
                        </p>
                    </div>
                </aside>
                <main
                    class="settings-main mb-2 mr-2 flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-xl border border-line bg-surface"
                    aria-label="Settings"
                >
                    <header
                        class="flex min-h-16 shrink-0 items-center justify-between gap-4 border-b border-line px-6"
                    >
                        <h2 class="text-lg font-semibold">{activeLabel}</h2>
                        <a
                            href="/chat"
                            class="rounded p-1 text-muted-foreground hover:bg-hover"
                            aria-label="Close settings"
                        >
                            <Icon name="close" class="size-5" />
                        </a>
                    </header>
                    <div
                        class="settings-content scroll-thin flex-1 overflow-y-auto p-5"
                        pp-ref={content}
                    >
                        <div
                            class="settings-intro mb-5 flex flex-wrap items-center justify-between gap-3"
                        >
                            <p
                                class="text-xs text-muted-foreground"
                                role="status"
                                aria-live="polite"
                            >
                                {settings.status}
                            </p>
                            <button
                                type="button"
                                class="settings-secondary"
                                onclick={setConfirmReset(true)}
                            >
                                "Restore defaults"
                            </button>
                        </div>
                        <p
                            class="mb-4 text-sm text-danger-ink"
                            role="alert"
                            hidden={!settings.saveFailed}
                        >
                            {settings.status}
                        </p>
                        <section class="settings-card" hidden={!show("regional")}>
                            <div class="settings-card-heading">
                                <Icon name="format" class="size-5" />
                                <div>
                                    <h3>"Language and regional formats"</h3>
                                    <p>"Choose how dates and times appear in the preview below."</p>
                                </div>
                            </div>
                            <label class="settings-row">
                                <span>"Follow browser language"</span>
                                <input
                                    class="settings-switch"
                                    type="checkbox"
                                    role="switch"
                                    checked={prefs.followLocale}
                                    onchange={settings.update("followLocale", target.checked)}
                                />
                            </label>
                            <label class="settings-row">
                                <span>"Regional format"</span>
                                <select
                                    class="settings-select"
                                    aria-label="Regional format"
                                    value={prefs.locale}
                                    disabled={prefs.followLocale}
                                    onchange={settings.update("locale", target.value)}
                                >
                                    <option value="en-US">"English (United States)"</option>
                                    <option value="en-GB">"English (United Kingdom)"</option>
                                    <option value="es-NI">"Español (Nicaragua)"</option>
                                </select>
                            </label>
                            <label class="settings-row">
                                <span>"Use 12-hour time"</span>
                                <input
                                    class="settings-switch"
                                    type="checkbox"
                                    role="switch"
                                    checked={prefs.hour12}
                                    onchange={settings.update("hour12", target.checked)}
                                />
                            </label>
                            <div class="settings-note">
                                "Preview: "
                                {formatPreview}
                                " · Interface language: English"
                            </div>
                        </section>
                        <section class="settings-card" hidden={!show("timezone")}>
                            <div class="settings-card-heading">
                                <Icon name="clock" class="size-5" />
                                <div>
                                    <h3>"Time zone"</h3>
                                    <p>
                                        "Your browser follows the time zone set on your computer."
                                    </p>
                                </div>
                            </div>
                            <p class="settings-detail">
                                "Current time zone: "
                                <strong>{timeZone}</strong>
                            </p>
                            <p class="settings-note">
                                "Change your computer's date and time settings to update this time zone."
                            </p>
                        </section>
                        <section class="settings-card" hidden={!show("theme")}>
                            <div class="settings-card-heading">
                                <Icon name="sparkle" class="size-5" />
                                <div>
                                    <h3>"Theme"</h3>
                                    <p>"Make Upurnaka feel like your space."</p>
                                </div>
                            </div>
                            <div class="settings-choices" role="radiogroup" aria-label="Theme">
                                <template pp-for="theme in themes">
                                    <label
                                        key={theme.key}
                                        class={"settings-choice " + (prefs.theme === theme.key ? "is-selected" : "")}
                                    >
                                        <span class={"settings-theme-preview preview-" + theme.key}>
                                            <span></span>
                                            <i></i>
                                            <b></b>
                                        </span>
                                        <span class="flex items-center justify-between gap-3">
                                            {theme.label}
                                            <input
                                                type="radio"
                                                name="theme"
                                                value={theme.key}
                                                checked={prefs.theme === theme.key}
                                                onchange={settings.update("theme", target.value)}
                                            />
                                        </span>
                                    </label>
                                </template>
                            </div>
                        </section>
                        <section class="settings-card" hidden={!show("text")}>
                            <div class="settings-card-heading">
                                <Icon name="format" class="size-5" />
                                <div>
                                    <h3>"Text size"</h3>
                                    <p>"Adjust text throughout Upurnaka."</p>
                                </div>
                            </div>
                            <label class="settings-row">
                                <span>"Text size"</span>
                                <select
                                    class="settings-select"
                                    aria-label="Text size"
                                    value={prefs.textSize}
                                    onchange={settings.update("textSize", target.value)}
                                >
                                    <option value="standard">"Standard"</option>
                                    <option value="large">"Large"</option>
                                </select>
                            </label>
                        </section>
                        <section class="settings-card" hidden={!show("composer")}>
                            <div class="settings-card-heading">
                                <Icon name="chat" class="size-5" />
                                <div>
                                    <h3>"Message composer"</h3>
                                    <p>"Choose how you write and send chat messages."</p>
                                </div>
                            </div>
                            <label class="settings-row">
                                <span>
                                    "Press Enter to send"
                                    <small>
                                        "When off, use Ctrl+Enter or the Send button. Shift+Enter always starts a new line."
                                    </small>
                                </span>
                                <input
                                    class="settings-switch"
                                    type="checkbox"
                                    role="switch"
                                    aria-label="Press Enter to send"
                                    checked={prefs.enterToSend}
                                    onchange={settings.update("enterToSend", target.checked)}
                                />
                            </label>
                            <label class="settings-row">
                                <span>
                                    "Check spelling while typing"
                                    <small>
                                        "Uses your browser's spelling tools and installed languages."
                                    </small>
                                </span>
                                <input
                                    class="settings-switch"
                                    type="checkbox"
                                    role="switch"
                                    aria-label="Check spelling while typing"
                                    checked={prefs.spellcheck}
                                    onchange={settings.update("spellcheck", target.checked)}
                                />
                            </label>
                        </section>
                        <section class="settings-card" hidden={!show("motion")}>
                            <div class="settings-card-heading">
                                <Icon name="video" class="size-5" />
                                <div>
                                    <h3>"Motion"</h3>
                                    <p>"Keep the interface comfortable to use."</p>
                                </div>
                            </div>
                            <label class="settings-row">
                                <span>
                                    "Reduce motion"
                                    <small>"Minimize interface animations and transitions."</small>
                                </span>
                                <input
                                    class="settings-switch"
                                    type="checkbox"
                                    role="switch"
                                    checked={prefs.reducedMotion}
                                    onchange={settings.update("reducedMotion", target.checked)}
                                />
                            </label>
                        </section>
                        <section class="settings-card" hidden={!show("storage")}>
                            <div class="settings-card-heading">
                                <Icon name="attach" class="size-5" />
                                <div>
                                    <h3>"File storage"</h3>
                                    <p>
                                        "Choose your preferred location for files: your computer or Dilnaka storage."
                                    </p>
                                </div>
                            </div>
                            <div
                                class="settings-choices storage-choices"
                                role="radiogroup"
                                aria-label="Preferred file storage"
                            >
                                <label
                                    class={"settings-choice storage-choice " + (prefs.storage === "computer" ? "is-selected" : "")}
                                >
                                    <span class="flex items-center justify-between">
                                        <Icon name="panel" class="size-7" />
                                        <input
                                            type="radio"
                                            name="storage"
                                            value="computer"
                                            checked={prefs.storage === "computer"}
                                            onchange={settings.update("storage", target.value)}
                                        />
                                    </span>
                                    <strong>"Computer"</strong>
                                    <span class="text-xs text-muted-foreground">
                                        "Keep downloaded files on this device."
                                    </span>
                                </label>
                                <label
                                    class={"settings-choice storage-choice " + (prefs.storage === "dilnaka" ? "is-selected" : "")}
                                >
                                    <span class="flex items-center justify-between">
                                        <Icon name="apps" class="size-7" />
                                        <input
                                            type="radio"
                                            name="storage"
                                            value="dilnaka"
                                            checked={prefs.storage === "dilnaka"}
                                            onchange={settings.update("storage", target.value)}
                                        />
                                    </span>
                                    <strong>"Dilnaka storage"</strong>
                                    <span class="text-xs text-muted-foreground">
                                        "Your connected cloud storage."
                                    </span>
                                </label>
                            </div>
                            <div class="settings-storage-summary">
                                <span class="settings-storage-dot"></span>
                                <span>
                                    "Preferred storage: "
                                    <strong>
                                        {prefs.storage === "computer" ? "Computer" : "Dilnaka storage"}
                                    </strong>
                                </span>
                            </div>
                            <p class="settings-note">
                                "This saves your preference. Existing files are not moved. Meeting recordings currently use Dilnaka regardless of this choice."
                            </p>
                        </section>
                        <section class="settings-card" hidden={!show("downloads")}>
                            <div class="settings-card-heading">
                                <Icon name="arrow-right" class="size-5" />
                                <div>
                                    <h3>"Downloads"</h3>
                                    <p>"Location"</p>
                                </div>
                            </div>
                            <p class="settings-detail font-semibold">
                                "Your browser's download folder"
                            </p>
                            <p class="settings-note">
                                "To change the folder or always ask where to save a file, open your browser's download settings. Upurnaka uses those settings for downloads to your computer."
                            </p>
                        </section>
                        <section class="settings-card" hidden={!show("cloud")}>
                            <div class="settings-card-heading">
                                <Icon name="apps" class="size-5" />
                                <div>
                                    <h3>"Dilnaka storage"</h3>
                                    <p>"Cloud connection"</p>
                                </div>
                            </div>
                            <p class="settings-detail font-semibold">
                                @{if cloud_ready { "Configured for this workspace" } else { "Not configured for this workspace" }}
                            </p>
                            <p class="settings-note">
                                @{if cloud_ready { "Meeting recordings can be uploaded to Dilnaka. Storage usage and quota are not available here yet." } else { "Contact your workspace administrator to connect Dilnaka storage. You can still download files to your computer." }}
                            </p>
                        </section>
                        <div class="settings-card" hidden={!confirmReset}>
                            <h3 class="font-semibold">"Restore default settings?"</h3>
                            <p class="mt-2 text-sm text-muted-foreground">
                                "This resets preferences for your account in this browser. Your files and messages are kept."
                            </p>
                            <div class="mt-4 flex gap-3">
                                <button
                                    type="button"
                                    class="settings-secondary"
                                    onclick={setConfirmReset(false)}
                                >
                                    "Cancel"
                                </button>
                                <button type="button" class="settings-primary" onclick={restore()}>
                                    "Restore defaults"
                                </button>
                            </div>
                        </div>
                    </div>
                </main>
                <script>r##"
                    const settings = pp.context(window.UpurnakaSettingsContext);
                    const shortcuts = pp.context(window.UpurnakaShortcutContext);
                    const prefs = settings.preferences;
                    const categories = [
                        {key:'general', label:'General'},
                        {key:'appearance', label:'Appearance'},
                        {key:'chats', label:'Chats'},
                        {key:'accessibility', label:'Accessibility'},
                        {key:'files', label:'Files and links'}
                    ];
                    const cards = [
                        {id:'regional', category:'general', text:'language regional formats browser locale date time 12 hour'},
                        {id:'timezone', category:'general', text:'time zone computer clock'},
                        {id:'theme', category:'appearance', text:'theme dark light system appearance'},
                        {id:'text', category:'appearance', text:'text size standard large accessibility'},
                        {id:'composer', category:'chats', text:'message composer enter send spelling spellcheck typing keyboard'},
                        {id:'motion', category:'accessibility', text:'reduce motion animations transitions'},
                        {id:'storage', category:'files', text:'file storage computer local device cloud dilnaka'},
                        {id:'downloads', category:'files', text:'downloads folder location save browser files'},
                        {id:'cloud', category:'files', text:'dilnaka cloud storage quota connection'}
                    ];
                    const themes = [{key:'light',label:'Light'}, {key:'dark',label:'Dark'}, {key:'system',label:'Follow system'}];
                    const [active, setActive] = pp.state('general');
                    const [query, setQuery] = pp.state('');
                    const [confirmReset, setConfirmReset] = pp.state(false);
                    const searchInput = pp.ref(null);
                    const content = pp.ref(null);
                    const needle = query.trim().toLocaleLowerCase();
                    function matchesCard(card) { return !needle || (card.text + ' ' + categories.find(item => item.key === card.category).label).toLocaleLowerCase().includes(needle); }
                    function matchesCategory(key) { return cards.some(card => card.category === key && matchesCard(card)); }
                    const matchingCategories = categories.filter(item => matchesCategory(item.key));
                    const activeLabel = categories.find(item => item.key === active)?.label || 'General';
                    function show(id) { const card = cards.find(item => item.id === id); return card.category === active && matchesCard(card); }
                    function choose(key) { setActive(key); setConfirmReset(false); content.current?.scrollTo(0, 0); }
                    function search(value) {
                        setQuery(value);
                        const term = value.trim().toLocaleLowerCase();
                        const matched = cards.filter(card => (card.text + ' ' + categories.find(item => item.key === card.category).label).toLocaleLowerCase().includes(term));
                        if (!matched.some(card => card.category === active) && matched.length) setActive(matched[0].category);
                        content.current?.scrollTo(0, 0);
                    }
                    function restore() { settings.reset(); setConfirmReset(false); }
                    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
                    const locale = prefs.followLocale ? navigator.language : prefs.locale;
                    const formatPreview = new Intl.DateTimeFormat(locale, {dateStyle:'medium', timeStyle:'short', hour12:prefs.hour12}).format(new Date());
                    const handleKey = pp.event(event => {
                        if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
                            event.preventDefault(); searchInput.current?.focus(); searchInput.current?.select();
                        }
                    });
                    pp.effect(() => {
                        if (!shortcuts) return;
                        const handler = {handleKey};
                        shortcuts.current = handler;
                        return () => { if (shortcuts.current === handler) shortcuts.current = null; };
                    }, [shortcuts, handleKey]);
                "##</script>
            </div>
        </AppShell>
    }
}
