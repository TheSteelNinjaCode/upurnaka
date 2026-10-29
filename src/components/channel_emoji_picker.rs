use crate::{
    components::{
        icon::Icon,
        rahti_ui::{
            attrs::Attrs,
            popover::{Popover, PopoverContent, PopoverTrigger},
        },
    },
    rahti::{Bindings, Html, Json, component, html},
};

/// Search state stays inside this boundary, away from the community feed.
#[component]
pub fn ChannelEmojiPicker(bindings: Bindings) -> Html {
    let user = crate::auth::current()
        .map(|user| user.id)
        .unwrap_or_default();
    html! {
        <div class="channel-emoji-picker" @{..bindings}>
            <Popover id="channel-icon">
                <PopoverTrigger
                    name="channel-icon"
                    class="community-channel-icon-trigger"
                    attrs=@{Attrs::new().set("aria-label", "Choose channel icon").set("type", "button")}
                    onclick={preparePicker()}
                >
                    <span hidden={selected !== "💬"}>
                        <Icon name="emoji-add" class="size-5" />
                    </span>
                    <img
                        src={selectedEmoji?.image || "/favicon.ico"}
                        alt=""
                        hidden={selected === "💬" || !selectedEmoji}
                        class="channel-selected-emoji"
                    />
                    <span class="sr-only" hidden={selected === "💬"}>{selected}</span>
                    <Icon name="chevron-down" class="size-3" />
                </PopoverTrigger>
                <PopoverContent
                    name="channel-icon"
                    class="community-channel-icon-options"
                    side="top"
                    align="start"
                    ontoggle={onPickerToggle(event)}
                >
                    <div class="channel-emoji-search-wrap">
                        <input
                            type="search"
                            aria-label="Search emoji"
                            placeholder="Search"
                            value={query}
                            oninput={setQuery(target.value)}
                            onkeydown={searchKey(event)}
                            pp-ref={searchInput}
                            autocomplete="off"
                        />
                        <Icon name="search" class="size-5" />
                    </div>
                    <h3 class="channel-emoji-heading">{heading}</h3>
                    <div
                        class="channel-emoji-scroll"
                        pp-ref={emojiScroll}
                        aria-label="Emoji choices"
                    >
                        <div class="community-channel-icon-grid" onkeydown={gridKey(event)}>
                            <template pp-for="emoji in visibleEmojis">
                                <button
                                    key={emoji.glyph}
                                    type="button"
                                    aria-label={"Use " + emoji.glyph + " icon"}
                                    title={emoji.name}
                                    aria-pressed={normalize(selected) === normalize(emoji.glyph)}
                                    data-emoji={emoji.glyph}
                                    pp-ref={element => emojiButtons.current[emoji.glyph] = element}
                                    tabindex={tabStop === emoji.glyph ? 0 : -1}
                                    onfocus={setFocusedGlyph(emoji.glyph)}
                                    popovertarget="popover-channel-icon"
                                    popovertargetaction="hide"
                                    onclick={choose(emoji.glyph)}
                                >
                                    <img
                                        src={emoji.image}
                                        alt=""
                                        width="38"
                                        height="38"
                                        loading="lazy"
                                        decoding="async"
                                    />
                                </button>
                            </template>
                        </div>
                        <p class="channel-emoji-empty" hidden={visibleEmojis.length !== 0}>
                            "No emoji found. Try another search."
                        </p>
                    </div>
                    <p class="sr-only" role="status">{visibleEmojis.length + " emoji available"}</p>
                    <div
                        class="channel-emoji-categories"
                        role="tablist"
                        aria-label="Emoji categories"
                        onkeydown={categoryKey(event)}
                    >
                        <button
                            type="button"
                            role="tab"
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
                </PopoverContent>
            </Popover>
            <style>r##"
                .channel-emoji-picker { flex-shrink:0; }
                .community-channel-icon-trigger .channel-selected-emoji { width:22px;height:22px;object-fit:contain; }
                .community-channel-icon-options { width:420px;max-width:calc(100vw - 24px);padding:22px;background:#292929;color:#eee;border:1px solid #383838;border-radius:5px;box-shadow:0 12px 32px #0006; }
                @supports (position-anchor:--popover-channel-icon) {
                    .community-channel-icon-options { position-try-fallbacks:--channel-emoji-beside,flip-block,flip-inline !important; }
                    @position-try --channel-emoji-beside { position-area:inline-end;margin:0 0 0 6px; }
                }
                .channel-emoji-search-wrap { display:flex;align-items:center;gap:10px;background:#202020;border-bottom:2px solid #9b8aef;border-radius:4px;padding:0 12px; }
                .channel-emoji-search-wrap input { width:100%;min-width:0;height:40px;background:transparent;border:0;outline:none;color:#eee;font-size:15px; }
                .channel-emoji-search-wrap input:focus-visible { outline:none;box-shadow:none; }
                .channel-emoji-search-wrap svg { flex-shrink:0;color:#bbb; }
                .channel-emoji-heading { margin:22px 0 12px;font-size:16px;font-weight:600; }
                .channel-emoji-scroll { height:242px;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;scrollbar-color:#777 transparent; }
                .community-channel-icon-grid { display:grid;grid-template-columns:repeat(6,minmax(0,1fr));gap:6px; }
                .community-channel-icon-grid button { display:grid;place-items:center;height:48px;min-width:0;padding:5px;border:1px solid transparent;border-radius:4px; }
                .community-channel-icon-grid button img { width:38px;height:38px;object-fit:contain;pointer-events:none; }
                .community-channel-icon-grid button:hover,.community-channel-icon-grid button[aria-pressed=true] { background:#48444d;border-color:#aaa0ef; }
                .community-channel-icon-grid button:focus-visible { outline:2px solid #aaa0ef;outline-offset:-2px; }
                .channel-emoji-empty { padding:60px 12px;text-align:center;font-size:14px;color:#aaa; }
                .channel-emoji-categories { display:grid;grid-template-columns:repeat(10,minmax(0,1fr));gap:2px;border-top:1px solid #555;margin-top:14px;padding-top:8px; }
                .channel-emoji-categories button { display:grid;place-items:center;height:40px;min-width:0;color:#aaa;border-radius:4px; }
                .channel-emoji-categories button:hover { background:#3b3b3b; }
                .channel-emoji-categories button[aria-selected=true] { color:#a69bff;background:#3b3b3b; }
                @media(max-width:440px) { .community-channel-icon-options { padding:14px; } .channel-emoji-categories button { height:36px; } .channel-emoji-categories svg { width:18px;height:18px; } }
            "##</style>
            <script>r##"
                const catalog = pp.memo(() => @{Json(&crate::emoji::catalog())}, []);
                function normalize(glyph) { return String(glyph || '').replace(/\uFE0F/g, ''); }
                const lookup = pp.memo(() => new Map(catalog.map(emoji => [normalize(emoji.glyph), emoji])), [catalog]);
                const selected = pp.props.value || '💬';
                const selectedEmoji = lookup.get(normalize(selected));
                const storageKey = 'upurnaka.channel-emoji.recent.' + @{user.as_str()};
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
                const [open, setOpen] = pp.state(false);
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
                function preparePicker() { setQuery('');setCategory('recent'); }
                function onPickerToggle(event) { setOpen(event.newState === 'open'); }
                const focusSearch = pp.event(() => { if (open) searchInput.current?.focus(); });
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
