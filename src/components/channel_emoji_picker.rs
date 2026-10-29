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
        <div class="channel-emoji-picker" @{..bindings}>
            <Popover id="channel-icon">
                <PopoverTrigger
                    name="channel-icon"
                    class="community-channel-icon-trigger"
                    attrs=@{Attrs::new().set("aria-label", "Choose channel icon").set("type", "button")}
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
                    <EmojiPanel
                        recent_key="channel-emoji"
                        label="Use {} icon"
                        closes="popover-channel-icon"
                        placeholder="Search"
                        value={selected}
                        open={open}
                        select={onChoose}
                    />
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
                @media(max-width:440px) { .community-channel-icon-options { padding:14px; } }
            "##</style>
            <script>r##"
                const catalog = pp.memo(() => @{Json(&crate::emoji::catalog())}, []);
                function normalize(glyph) { return String(glyph || '').replace(/\uFE0F/g, ''); }
                const selected = pp.props.value || '💬';
                const selectedEmoji = pp.memo(() => catalog.find(emoji => normalize(emoji.glyph) === normalize(selected)), [catalog, selected]);
                const [open, setOpen] = pp.state(false);
                function onPickerToggle(event) { setOpen(event.newState === 'open'); }
                const onChoose = pp.event(glyph => pp.props.select?.(glyph));
            "##</script>
        </div>
    }
}
