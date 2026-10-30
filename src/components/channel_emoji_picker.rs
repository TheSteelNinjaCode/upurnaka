use crate::{
    components::{
        emoji_panel::EmojiPanel,
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
    html! {
        <div class="channel-emoji-picker shrink-0" @{..bindings}>
            <Popover id="channel-icon">
                <PopoverTrigger
                    name="channel-icon"
                    class="community-channel-icon-trigger"
                    attrs=@{Attrs::new().set("aria-label", "Choose channel icon").set("type", "button")}
                    onclick={setSession(session + 1)}
                >
                    <span hidden={selected !== "💬"}>
                        <Icon name="emoji-add" class="size-5" />
                    </span>
                    <img
                        src={selectedEmoji?.image || "/favicon.ico"}
                        alt=""
                        hidden={selected === "💬" || !selectedEmoji}
                        class="channel-selected-emoji size-[22px] object-contain"
                    />
                    <span class="sr-only" hidden={selected === "💬"}>{selected}</span>
                    <Icon name="chevron-down" class="size-3" />
                </PopoverTrigger>
                <PopoverContent
                    name="channel-icon"
                    class="community-channel-icon-options w-[420px] max-w-[calc(100vw-24px)] rounded-[5px] border border-edge bg-popover p-[22px] text-ink shadow-xl shadow-black/40 max-[440px]:p-3.5"
                    side="top"
                    align="start"
                    ontoggle={onPickerToggle(event)}
                >
                    <EmojiPanel
                        recent_key="channel-emoji"
                        label="Use {} icon"
                        closes="popover-channel-icon"
                        placeholder="Search"
                        value={selected}
                        open={open}
                        session={session}
                        select={onChoose}
                    />
                </PopoverContent>
            </Popover>
            <style>r##"
                /* Anchor fallbacks have no utility form. */
                @supports (position-anchor:--popover-channel-icon) {
                    .community-channel-icon-options { position-try-fallbacks:--channel-emoji-beside,flip-block,flip-inline !important; }
                    @position-try --channel-emoji-beside { position-area:inline-end;margin:0 0 0 6px; }
                }
            "##</style>
            <script>r##"
                const catalog = pp.memo(() => @{Json(&crate::emoji::catalog())}, []);
                function normalize(glyph) { return String(glyph || '').replace(/\uFE0F/g, ''); }
                const selected = pp.props.value || '💬';
                const selectedEmoji = pp.memo(() => catalog.find(emoji => normalize(emoji.glyph) === normalize(selected)), [catalog, selected]);
                const [open, setOpen] = pp.state(false);
                const [session, setSession] = pp.state(0);
                function onPickerToggle(event) { setOpen(event.newState === 'open'); }
                const onChoose = pp.event(glyph => pp.props.select?.(glyph));
            "##</script>
        </div>
    }
}
