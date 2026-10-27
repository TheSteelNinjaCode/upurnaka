use crate::rahti::{Html, component, html};

/// Device-local preferences, isolated by account. Consumers share one live store.
#[component]
pub fn UserPreferences(user_id: &str, children: Html) -> Html {
    let default_storage = if crate::meetings::recording::is_configured() {
        "dilnaka"
    } else {
        "computer"
    };
    html! {
        <div class="user-preferences contents">
            <settingscontext.provider value={store}>
                <slot />
            </settingscontext.provider>
            <script>r##"
                const SettingsContext = window.UpurnakaSettingsContext ||= pp.createContext(null);
                const storageKey = 'upurnaka.settings.v1.' + @{user_id};
                const defaults = {theme:'dark', textSize:'standard', reducedMotion:false, spellcheck:true, enterToSend:true, followLocale:true, locale:'en-US', hour12:true, storage:@{default_storage}, storageExplicit:false};
                const choices = {theme:['dark','light','system'], textSize:['standard','large'], locale:['en-US','en-GB','es-NI'], storage:['computer','dilnaka']};
                const initial = pp.memo(() => {
                    let saved = {};
                    try { saved = JSON.parse(localStorage.getItem(storageKey) || '{}') || {}; } catch (_) {}
                    // Older saves included Computer even when the user only
                    // changed an unrelated setting. Adopt the workspace default
                    // unless a storage choice was deliberate or already Dilnaka.
                    if (saved.storageExplicit !== true && saved.storage !== 'dilnaka') saved = {...saved, storage:defaults.storage};
                    return Object.fromEntries(Object.entries(defaults).map(([key, value]) => [key,
                        choices[key] ? (choices[key].includes(saved[key]) ? saved[key] : value) : (typeof saved[key] === typeof value ? saved[key] : value)
                    ]));
                }, [storageKey]);
                const [preferences, setPreferences] = pp.state(initial);
                const [status, setStatus] = pp.state('Preferences are saved on this browser for your account.');
                const [saveFailed, setSaveFailed] = pp.state(false);
                const update = pp.event((key, value) => {
                    if (!(key in defaults) || (choices[key] ? !choices[key].includes(value) : typeof value !== typeof defaults[key])) return;
                    const next = {...preferences, [key]:value, storageExplicit:key === 'storage' ? true : preferences.storageExplicit};
                    try {
                        localStorage.setItem(storageKey, JSON.stringify(next));
                        setPreferences(next); setSaveFailed(false); setStatus('Saved on this browser.');
                    } catch (_) {
                        setSaveFailed(true); setStatus('Could not save. Allow browser storage and try again.');
                    }
                });
                const reset = pp.event(() => {
                    try {
                        localStorage.removeItem(storageKey);
                        setPreferences({...defaults}); setSaveFailed(false); setStatus('Default settings restored.');
                    } catch (_) { setSaveFailed(true); setStatus('Could not reset. Allow browser storage and try again.'); }
                });
                const store = pp.memo(() => ({preferences, update, reset, status, saveFailed}), [preferences, update, reset, status, saveFailed]);
                pp.effect(() => {
                    const root = document.documentElement;
                    const media = matchMedia('(prefers-color-scheme: light)');
                    const apply = () => { root.dataset.appearance = preferences.theme === 'system' ? (media.matches ? 'light' : 'dark') : preferences.theme; };
                    apply();
                    root.dataset.textSize = preferences.textSize;
                    root.dataset.reducedMotion = String(preferences.reducedMotion);
                    media.addEventListener('change', apply);
                    return () => { media.removeEventListener('change', apply); delete root.dataset.appearance; delete root.dataset.textSize; delete root.dataset.reducedMotion; };
                }, [preferences.theme, preferences.textSize, preferences.reducedMotion]);
            "##</script>
        </div>
    }
}
