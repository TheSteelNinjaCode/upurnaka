use crate::rahti::{Html, component, html};

/// Device-local preferences, isolated by account. Consumers share one live store.
#[component]
pub fn UserPreferences(user_id: &str, children: Html) -> Html {
    html! {
        <div class="user-preferences contents">
            <settingscontext.provider value={store}>
                <slot />
            </settingscontext.provider>
            <script>r##"
                const SettingsContext = window.UpurnakaSettingsContext ||= pp.createContext(null);
                const storageKey = 'upurnaka.settings.v1.' + @{user_id};
                const defaults = {theme:'dark', textSize:'standard', reducedMotion:false, spellcheck:true, enterToSend:true, followLocale:true, locale:'en-US', hour12:true, storage:'computer'};
                const choices = {theme:['dark','light','system'], textSize:['standard','large'], locale:['en-US','en-GB','es-NI'], storage:['computer','dilnaka']};
                const initial = pp.memo(() => {
                    let saved = {};
                    try { saved = JSON.parse(localStorage.getItem(storageKey) || '{}') || {}; } catch (_) {}
                    return Object.fromEntries(Object.entries(defaults).map(([key, value]) => [key,
                        choices[key] ? (choices[key].includes(saved[key]) ? saved[key] : value) : (typeof saved[key] === typeof value ? saved[key] : value)
                    ]));
                }, [storageKey]);
                const [preferences, setPreferences] = pp.state(initial);
                const [status, setStatus] = pp.state('Preferences are saved on this browser for your account.');
                const [saveFailed, setSaveFailed] = pp.state(false);
                const update = pp.event((key, value) => {
                    if (!(key in defaults) || (choices[key] ? !choices[key].includes(value) : typeof value !== typeof defaults[key])) return;
                    const next = {...preferences, [key]:value};
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
