use crate::{
    components::icon::Icon,
    rahti::{Bindings, Html, Json, component, html},
};

const CATEGORY_TAB: &str = "grid h-10 min-w-0 place-items-center rounded text-muted-foreground hover:bg-hover aria-selected:bg-hover aria-selected:text-brand-ink max-[440px]:h-9 max-[440px]:[&>svg]:size-[18px]";

/// The Fluent emoji panel: search, Recent and nine categories. Hosts open it
/// in their own popover, pass `open` while it shows, bump `session` from the
/// click that opens it (the panel then starts over from Recent with an empty
/// search, before anyone can type) and `select` to receive a glyph. `closes` names the popover a choice dismisses; `label` is the
/// accessible name of each emoji button, with `{}` for the glyph.
#[component]
pub fn EmojiPanel(
    recent_key: &str,
    label: &str,
    closes: &str,
    placeholder: &str,
    bindings: Bindings,
) -> Html {
    let user = crate::auth::current()
        .map(|user| user.id)
        .unwrap_or_default();
    let storage_key = format!("upurnaka.{recent_key}.recent.{user}");
    html! {
        <div class="emoji-panel" @{..bindings}>
            <div
                class="channel-emoji-search-wrap flex items-center gap-2.5 rounded border-b-2 border-brand-ink bg-sunken px-3 [&>svg]:shrink-0 [&>svg]:text-muted-foreground"
            >
                <input
                    type="search"
                    class="h-10 w-full min-w-0 border-0 bg-transparent text-[15px] text-ink outline-none focus-visible:shadow-none focus-visible:outline-none"
                    aria-label="Search emoji"
                    placeholder=@{placeholder}
                    value={query}
                    oninput={setQuery(target.value)}
                    onkeydown={searchKey(event)}
                    pp-ref={searchInput}
                    autocomplete="off"
                />
                <Icon name="search" class="size-5" />
            </div>
            <h3 class="channel-emoji-heading mb-3 mt-[22px] text-base font-semibold">{heading}</h3>
            <div
                class="channel-emoji-scroll scroll-thin h-[242px] overflow-y-auto overscroll-contain"
                pp-ref={emojiScroll}
                aria-label="Emoji choices"
            >
                <div
                    class="community-channel-icon-grid grid grid-cols-6 gap-1.5"
                    onkeydown={gridKey(event)}
                >
                    <template pp-for="emoji in visibleEmojis">
                        <button
                            key={emoji.glyph}
                            type="button"
                            class="grid h-12 min-w-0 place-items-center rounded border border-transparent p-[5px] hover:border-brand-ink hover:bg-hover focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-ink aria-pressed:border-brand-ink aria-pressed:bg-hover"
                            aria-label={actionLabel(emoji.glyph)}
                            title={emoji.name}
                            aria-pressed={normalize(selected) === normalize(emoji.glyph)}
                            data-emoji={emoji.glyph}
                            pp-ref={element => emojiButtons.current[emoji.glyph] = element}
                            tabindex={tabStop === emoji.glyph ? 0 : -1}
                            onfocus={setFocusedGlyph(emoji.glyph)}
                            popovertarget=@{closes}
                            popovertargetaction="hide"
                            onclick={choose(emoji.glyph)}
                        >
                            <img
                                src={emoji.image}
                                class="pointer-events-none size-[38px] object-contain"
                                alt=""
                                width="38"
                                height="38"
                                loading="lazy"
                                decoding="async"
                            />
                        </button>
                    </template>
                </div>
                <p
                    class="channel-emoji-empty px-3 py-[60px] text-center text-sm text-muted-foreground"
                    hidden={visibleEmojis.length !== 0}
                >
                    "No emoji found. Try another search."
                </p>
            </div>
            <p class="sr-only" role="status">{visibleEmojis.length + " emoji available"}</p>
            <div
                class="channel-emoji-categories mt-3.5 grid grid-cols-10 gap-0.5 border-t border-edge pt-2"
                role="tablist"
                aria-label="Emoji categories"
                onkeydown={categoryKey(event)}
            >
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Recent"
                    aria-label="Recent emoji"
                    aria-selected={category === "recent" && !query.trim()}
                    tabindex={category === "recent" ? 0 : -1}
                    onclick={selectCategory("recent")}
                    pp-ref={element => categoryButtons.current.recent = element}
                >
                    <Icon name="clock" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Smileys and emotions"
                    aria-label="Smileys and emotions"
                    aria-selected={category === "smileys" && !query.trim()}
                    tabindex={category === "smileys" ? 0 : -1}
                    onclick={selectCategory("smileys")}
                    pp-ref={element => categoryButtons.current.smileys = element}
                >
                    <Icon name="emoji" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Hands and gestures"
                    aria-label="Hands and gestures"
                    aria-selected={category === "gestures" && !query.trim()}
                    tabindex={category === "gestures" ? 0 : -1}
                    onclick={selectCategory("gestures")}
                    pp-ref={element => categoryButtons.current.gestures = element}
                >
                    <Icon name="hand" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="People"
                    aria-label="People emoji"
                    aria-selected={category === "people" && !query.trim()}
                    tabindex={category === "people" ? 0 : -1}
                    onclick={selectCategory("people")}
                    pp-ref={element => categoryButtons.current.people = element}
                >
                    <Icon name="people" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Animals and nature"
                    aria-label="Animals and nature"
                    aria-selected={category === "animals" && !query.trim()}
                    tabindex={category === "animals" ? 0 : -1}
                    onclick={selectCategory("animals")}
                    pp-ref={element => categoryButtons.current.animals = element}
                >
                    <Icon name="paw" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Food and drink"
                    aria-label="Food and drink"
                    aria-selected={category === "food" && !query.trim()}
                    tabindex={category === "food" ? 0 : -1}
                    onclick={selectCategory("food")}
                    pp-ref={element => categoryButtons.current.food = element}
                >
                    <Icon name="pizza" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Travel and places"
                    aria-label="Travel and places"
                    aria-selected={category === "travel" && !query.trim()}
                    tabindex={category === "travel" ? 0 : -1}
                    onclick={selectCategory("travel")}
                    pp-ref={element => categoryButtons.current.travel = element}
                >
                    <Icon name="location" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Objects"
                    aria-label="Objects emoji"
                    aria-selected={category === "objects" && !query.trim()}
                    tabindex={category === "objects" ? 0 : -1}
                    onclick={selectCategory("objects")}
                    pp-ref={element => categoryButtons.current.objects = element}
                >
                    <Icon name="light-bulb" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Activities"
                    aria-label="Activities emoji"
                    aria-selected={category === "activities" && !query.trim()}
                    tabindex={category === "activities" ? 0 : -1}
                    onclick={selectCategory("activities")}
                    pp-ref={element => categoryButtons.current.activities = element}
                >
                    <Icon name="volleyball" class="size-5" />
                </button>
                <button
                    type="button"
                    role="tab"
                    class=@{CATEGORY_TAB}
                    title="Symbols"
                    aria-label="Symbols emoji"
                    aria-selected={category === "symbols" && !query.trim()}
                    tabindex={category === "symbols" ? 0 : -1}
                    onclick={selectCategory("symbols")}
                    pp-ref={element => categoryButtons.current.symbols = element}
                >
                    <Icon name="hash" class="size-5" />
                </button>
            </div>
            <script>r##"
                const catalog = pp.memo(() => @{Json(&crate::emoji::catalog())}, []);
                function normalize(glyph) { return String(glyph || '').replace(/\uFE0F/g, ''); }
                const lookup = pp.memo(() => new Map(catalog.map(emoji => [normalize(emoji.glyph), emoji])), [catalog]);
                const selected = pp.props.value || '';
                const storageKey = @{storage_key.as_str()};
                const labelPattern = @{Json(&label)};
                function actionLabel(glyph) { return labelPattern.replace('{}', glyph); }
                const defaults = ['😲','😧','😠','😍','😎','🙌','🤜','🔥','🎉','👀','💯','💡','✔️','➕','✖️','🔜','🚢','🤔','🥺','😭','🤭','😌','🤯','📎'];
                function canonical(values) { return [...new Set(values.filter(value => typeof value === 'string').map(value => lookup.get(normalize(value))?.glyph).filter(Boolean))].slice(0,24); }
                const initialRecent = pp.memo(() => {
                    try { const saved = JSON.parse(localStorage.getItem(storageKey)); if (Array.isArray(saved)) return canonical(saved); } catch (_) {}
                    return canonical(defaults);
                }, [lookup, storageKey]);
                const [recent, setRecent] = pp.state(initialRecent);
                const [query, setQuery] = pp.state('');
                const [category, setCategory] = pp.state('recent');
                const [focusedGlyph, setFocusedGlyph] = pp.state('');
                const emojiButtons = pp.ref({});
                const searchInput = pp.ref(null), emojiScroll = pp.ref(null), categoryButtons = pp.ref({});
                const open = !!pp.props.open;
                const session = pp.props.session || 0;
                const categoryNames = {recent:'Recent',smileys:'Smileys and emotions',gestures:'Hands and gestures',people:'People',animals:'Animals and nature',food:'Food and drink',travel:'Travel and places',objects:'Objects',activities:'Activities',symbols:'Symbols'};
                const heading = query.trim() ? 'Search results' : categoryNames[category];
                const visibleEmojis = pp.memo(() => {
                    const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
                    if (words.length) {
                        const needle=query.trim().toLowerCase();
                        const exact=emoji=>normalize(emoji.glyph)===normalize(needle)||emoji.name.toLowerCase()===needle;
                        return catalog.filter(emoji => words.every(word => [emoji.glyph,emoji.name,...emoji.keywords].join(' ').toLowerCase().includes(word))).sort((a,b)=>Number(exact(b))-Number(exact(a)));
                    }
                    if (category === 'recent') return recent.map(glyph => lookup.get(normalize(glyph))).filter(Boolean);
                    return catalog.filter(emoji => emoji.category === category);
                }, [catalog,lookup,query,category,recent]);
                const tabStop = visibleEmojis.some(emoji=>emoji.glyph===focusedGlyph) ? focusedGlyph : visibleEmojis[0]?.glyph;
                const focusSearch = pp.event(() => { if (open) searchInput.current?.focus(); });
                // Each opening starts from Recent with an empty search, as in Teams.
                // The reset follows the opening click, not the popover's toggle
                // event, which arrives asynchronously and could undo typing.
                pp.effect(() => { setQuery('');setCategory('recent');setFocusedGlyph(''); },[session]);
                pp.effect(() => { if (!open) return;const frame=requestAnimationFrame(focusSearch);return ()=>cancelAnimationFrame(frame); },[open,focusSearch]);
                pp.effect(() => { if (emojiScroll.current) emojiScroll.current.scrollTop=0; },[query,category]);
                function choose(glyph) {
                    const next = canonical([glyph,...recent]);setRecent(next);
                    try { localStorage.setItem(storageKey,JSON.stringify(next)); } catch (_) {}
                    pp.props.select?.(glyph);
                }
                function selectCategory(id) { setCategory(id);setQuery('');setFocusedGlyph(''); }
                function searchKey(event) {
                    if (!['Enter','ArrowDown'].includes(event.key)) return;
                    event.preventDefault();
                    const first = emojiButtons.current[visibleEmojis[0]?.glyph];
                    if (event.key === 'Enter') first?.click();else first?.focus();
                }
                function gridKey(event) {
                    let index=visibleEmojis.findIndex(emoji=>emojiButtons.current[emoji.glyph]===document.activeElement);
                    if(index<0)return;
                    if(event.key==='ArrowRight')index+=1;
                    else if(event.key==='ArrowLeft')index-=1;
                    else if(event.key==='ArrowDown')index+=6;
                    else if(event.key==='ArrowUp')index-=6;
                    else if(event.key==='Home')index=0;
                    else if(event.key==='End')index=visibleEmojis.length-1;
                    else return;
                    event.preventDefault();index=Math.max(0,Math.min(index,visibleEmojis.length-1));
                    emojiButtons.current[visibleEmojis[index]?.glyph]?.focus();
                }
                function categoryKey(event) {
                    const ids=Object.keys(categoryNames);let index=ids.indexOf(category);
                    if(event.key==='ArrowRight')index=(index+1)%ids.length;
                    else if(event.key==='ArrowLeft')index=(index+ids.length-1)%ids.length;
                    else if(event.key==='Home')index=0;
                    else if(event.key==='End')index=ids.length-1;
                    else return;
                    event.preventDefault();selectCategory(ids[index]);categoryButtons.current[ids[index]]?.focus();
                }
            "##</script>
        </div>
    }
}
