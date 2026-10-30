// rahti-ui: tooltip
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Tooltip — a port of shadcn/ui's `tooltip`.
//!
//! The third component built on the same machinery as
//! [`Popover`](super::popover) and [`HoverCard`](super::hover_card): a
//! `popover` in the top layer, placed with CSS anchor positioning, with a script
//! for the gesture the platform has no attribute for.
//!
//! What makes it its own component rather than a hover card with different
//! padding:
//!
//! - **It is a label, not content.** `role="tooltip"`, and the trigger points at
//!   it with `aria-describedby` — which is the whole of the accessibility story
//!   and the reason a tooltip must never hold anything interactive.
//! - **It opens on `top` by default**, where the other two open on `bottom`.
//!   shadcn's Base UI version says `side = "top"`.
//! - **It has an arrow**, and the arrow is why `data-side` has to tell the
//!   truth — see below.
//! - **It opens instantly.** Base UI's own `delay` is 600ms, but shadcn's
//!   `TooltipProvider` sets `delay = 0` and tells you to put that provider in
//!   your root layout, so the tooltip everybody actually gets is instant. Zero
//!   is the default here for the same reason.
//! - **A click closes it**, and there is no `popovertarget` fallback. A tooltip
//!   sits on buttons that do things; a click that popped up a label as well as
//!   firing the action would be worse than no tooltip. That is also Base UI's
//!   behaviour, and it is the one place this port does *less* than
//!   [`HoverCard`](super::hover_card) on purpose.
//!
//! ## Where the delays live
//!
//! `delay` and `closeDelay` are props of the **root** here, and of the
//! [`TooltipProvider`](TooltipProvider) above it — which is where Base UI puts
//! them. ([`HoverCard`](super::hover_card) puts its two on the *trigger*,
//! because Base UI's PreviewCard does. The two components disagree upstream and
//! they disagree here.) The order is root, then provider, then
//! [`DELAY_MS`]/[`CLOSE_DELAY_MS`], and each level renders `data-delay` and
//! `data-close-delay` for the script to read.
//!
//! ## `data-side` has to tell the truth
//!
//! Every other component here treats `data-side` as what the *page asked for*.
//! A tooltip cannot: `position-try-fallbacks` flips it near a viewport edge, and
//! an arrow still glued to the side it was asked for would then be pointing away
//! from its trigger, floating over nothing. It is the most visible wrong thing
//! this library could render.
//!
//! So the script measures where the tooltip actually landed on every open and
//! writes the real side onto the content and the arrow. The slide-in direction
//! gets fixed by the same write, for free.
//!
//! ## Which shadcn tooltip this is
//!
//! shadcn's Base UI version — the one behind the linked docs — rather than the
//! Radix registry entry, because the two genuinely differ here: Base UI's is
//! `inline-flex … max-w-xs` with the Kbd support, where Radix's is `w-fit` with
//! `text-balance`. The changes are the usual three:
//!
//! - `data-open:`/`data-[state=delayed-open]:` become `open:`, Tailwind's
//!   variant for `:popover-open`.
//! - The `data-closed:` exit utilities are dropped: a popover is `display: none`
//!   the instant it closes, so a keyframe exit would never be seen.
//! - `origin-(--transform-origin)` is dropped — Base UI's positioner publishes
//!   that custom property and nothing here does. The zoom falls back to
//!   `center`.
//!
//! Base UI's logical `data-[side=inline-start]`/`inline-end` rules are dropped
//! too. This port names four physical sides and never writes the logical ones,
//! so those rules could not match.
//!
//! ## The parts
//!
//! ```html
//! <div data-slot="tooltip" class="contents" data-delay="0" data-close-delay="0">
//!   <button data-slot="tooltip-trigger" type="button" data-tooltip="save"
//!           data-state="closed" aria-describedby="tooltip-save"
//!           style="anchor-name:--tooltip-save">Save</button>
//!   <div data-slot="tooltip-content" id="tooltip-save" popover="auto" role="tooltip"
//!        data-tooltip="save" data-side="top" data-align="center" data-state="closed"
//!        style="position-anchor:--tooltip-save;position-area:block-start center;
//!               margin-block:4px;…">
//!     Save your work
//!     <div data-slot="tooltip-arrow" data-side="top"></div>
//!   </div>
//!   <span data-slot="tooltip-scope" hidden></span>
//!   <script>…</script>
//! </div>
//! ```

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, Json, component, html};

/// The root and the provider: `display: contents`, so the trigger inside lays
/// out where it was written.
pub const ROOT: &str = "contents";

/// The tooltip. shadcn's Base UI string, less the custom property and the exit
/// animation.
///
/// `open:` is `data-open:` and `data-[state=delayed-open]:` at once — Tailwind's
/// `open:` variant matches `:popover-open`, and a popover is either open or it
/// is not, so the two states shadcn distinguishes collapse into one here.
///
/// The four `data-[slot=kbd]` rules are shadcn's and are kept although this
/// library has no Kbd component yet: they are what the string will need the day
/// it does, and until then they simply never match.
///
/// # `open:inline-flex`, and why it cannot be plain `inline-flex`
///
/// shadcn's string says `inline-flex`, and it has to be conditional here.
/// Everything hiding a closed popover is the one UA rule
/// `[popover]:not(:popover-open):not(dialog) { display: none }` — and a UA rule
/// loses to *any* author declaration. So an unconditional `display: inline-flex`
/// leaves every tooltip on the page permanently visible, laid out in normal flow
/// beside its trigger, with the anchor positioning silently doing nothing
/// because the element was never in the top layer.
///
/// Measured in the browser rather than reasoned about: thirteen tooltips
/// rendered inline down the demo page. It is invisible in shadcn's own version
/// because Base UI does not use the popover API at all — its positioner mounts
/// and unmounts the popup instead, so nothing there depends on that UA rule.
///
/// This is the trap any future component that sets `display` on a popover will
/// hit.
///
/// # `w-max`, and why it cannot be shadcn's `w-fit`
///
/// The second declaration the port had to change, and the second one only
/// visible in a browser. `position-area: block-start center` makes the
/// **anchor's own box the containing block** — so `width: fit-content`, which is
/// bounded by the containing block, collapses the tooltip to the width of the
/// button it is labelling. Measured: a 72px trigger gave a 72px tooltip, three
/// lines tall, with a scrollbar. `width: max-content` is not bounded by the
/// containing block, so it takes the width of its text and overflows the area
/// symmetrically — which centres it on the anchor, and `max-w-xs` still caps it
/// and wraps. Measured again: 112 × 28, centred, from the same markup.
///
/// The alternative that also works is `position-area: block-start span-all` plus
/// `justify-self: anchor-center`; it measures identically and costs a newer CSS
/// feature to depend on, so `w-max` wins.
///
/// [`Popover`](super::popover) and [`HoverCard`](super::hover_card) never hit
/// this because their content has a fixed width — `w-72`, `w-64` — and a fixed
/// width is not bounded by its containing block either. Only intrinsic sizing
/// is, and this is the only overlay here that sizes to its content.
pub const CONTENT: &str = "z-50 open:inline-flex w-max max-w-xs items-center gap-1.5 rounded-md \
                           bg-foreground px-3 py-1.5 text-xs text-background \
                           has-data-[slot=kbd]:pr-1.5 \
                           data-[side=bottom]:slide-in-from-top-2 \
                           data-[side=left]:slide-in-from-right-2 \
                           data-[side=right]:slide-in-from-left-2 \
                           data-[side=top]:slide-in-from-bottom-2 \
                           **:data-[slot=kbd]:relative **:data-[slot=kbd]:isolate \
                           **:data-[slot=kbd]:z-50 **:data-[slot=kbd]:rounded-sm \
                           open:animate-in open:fade-in-0 open:zoom-in-95";

/// The arrow — a rotated square poking out of the tooltip's edge.
///
/// shadcn's values, with two edits. Base UI's positioner supplies the arrow's
/// *centring* at runtime, so `left-1/2` and `-translate-x-1/2` are added here
/// for the two sides that needed it. And shadcn's unprefixed
/// `translate-y-[calc(-50%-2px)]` — which the left/right rules then override by
/// specificity — is written out per side instead, so no two rules in this list
/// answer the same question and [`audit`](super::utils::audit) can prove it.
///
/// The centring is right for `align="center"`, which is shadcn's default and
/// very nearly always what a tooltip wants. An `align="start"` tooltip keeps a
/// centred arrow, which is the one thing here Floating UI would have done
/// better.
pub const ARROW: &str = "absolute z-50 size-2.5 rotate-45 rounded-[2px] bg-foreground \
                         fill-foreground \
                         data-[side=top]:-bottom-2.5 data-[side=top]:left-1/2 \
                         data-[side=top]:-translate-x-1/2 \
                         data-[side=top]:translate-y-[calc(-50%-2px)] \
                         data-[side=bottom]:top-1 data-[side=bottom]:left-1/2 \
                         data-[side=bottom]:-translate-x-1/2 \
                         data-[side=bottom]:translate-y-[calc(-50%-2px)] \
                         data-[side=left]:-right-1 data-[side=left]:top-1/2 \
                         data-[side=left]:-translate-y-1/2 \
                         data-[side=right]:-left-1 data-[side=right]:top-1/2 \
                         data-[side=right]:-translate-y-1/2";

/// What a `popover` needs said about it that a portal did not.
///
/// Two UA declarations to undo, where the other overlays here only have one.
///
/// **`inset: 0`.** Tailwind's preflight clears the margin, border and padding
/// through its `*` rule; the inset it leaves alone, and an uncleared `inset: 0`
/// fights every `left`/`top` the anchor or the script writes.
///
/// **`overflow: auto`**, which is also in Chrome's `[popover]` rule and which no
/// other component in this library has had to care about. This one has
/// [`ARROW`], and an arrow's whole job is to stick out of the box — so `auto`
/// treats it as overflowing content, gives the tooltip scrollbars, and then
/// reserves 15px for one. Measured: the tooltip came out 112px wide with a 97px
/// client box, and the arrow — placed at `left: 50%` of that client box — sat
/// 8px left of centre. Both symptoms, one cause, and neither of them looks like
/// an overflow problem from the screenshot.
pub const POPOVER_RESET: &str = "inset-auto overflow-visible";

/// How far the tooltip sits from its trigger — shadcn's `sideOffset = 4`.
pub const SIDE_OFFSET: u32 = 4;

/// How long the pointer has to stay before the tooltip opens.
///
/// Zero, which is shadcn's number and not Base UI's. The primitive defaults to
/// 600ms; shadcn's `TooltipProvider` overrides it to `delay = 0` and the docs
/// tell you to put that provider in your root layout, so the tooltip anybody
/// using shadcn actually gets is instant.
///
/// A tooltip is a label for the thing under the pointer, and a label that
/// arrives half a second late has already been read around.
pub const DELAY_MS: u32 = 0;

/// How long the tooltip survives the pointer leaving — Base UI's own default.
///
/// Zero here as well, and unlike [`HoverCard`](super::hover_card) that is not a
/// problem: nothing in a tooltip is meant to be reached with the pointer, so
/// there is no gap to cross. Give it a value and the script's
/// enter-cancels-the-close handling makes the tooltip hoverable, which is what
/// Base UI's `hoverable` means.
pub const CLOSE_DELAY_MS: u32 = 0;

/// Which side of the trigger the tooltip opens on.
///
/// `Top` by default, which is shadcn's `side = "top"` and the one place the
/// three overlay components in this library disagree — the popover and the hover
/// card both open below.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipSide {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl TooltipSide {
    /// The name written into `data-side`, which the slide utilities and every
    /// rule in [`ARROW`] read.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// The CSS `position-area` this side asks for.
    pub const fn area(self, align: TooltipAlign) -> &'static str {
        match (self, align) {
            (Self::Bottom, TooltipAlign::Start) => "block-end span-inline-end",
            (Self::Bottom, TooltipAlign::Center) => "block-end center",
            (Self::Bottom, TooltipAlign::End) => "block-end span-inline-start",
            (Self::Top, TooltipAlign::Start) => "block-start span-inline-end",
            (Self::Top, TooltipAlign::Center) => "block-start center",
            (Self::Top, TooltipAlign::End) => "block-start span-inline-start",
            (Self::Left, TooltipAlign::Start) => "inline-start span-block-end",
            (Self::Left, TooltipAlign::Center) => "inline-start center",
            (Self::Left, TooltipAlign::End) => "inline-start span-block-start",
            (Self::Right, TooltipAlign::Start) => "inline-end span-block-end",
            (Self::Right, TooltipAlign::Center) => "inline-end center",
            (Self::Right, TooltipAlign::End) => "inline-end span-block-start",
        }
    }

    /// Which margin carries [`SIDE_OFFSET`] under a `position-area`.
    ///
    /// One axis only. A margin on all four sides also pushes the tooltip along
    /// the axis it is *aligned* on, which puts an `align="start"` tooltip 4px in
    /// from the edge it is supposed to line up with.
    pub const fn offset_axis(self) -> &'static str {
        match self {
            Self::Top | Self::Bottom => "margin-block",
            Self::Left | Self::Right => "margin-inline",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `side="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            "right" => Self::Right,
            _ => Self::Top,
        }
    }
}

/// Which of the trigger's edges the tooltip lines up with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipAlign {
    Start,
    #[default]
    Center,
    End,
}

impl TooltipAlign {
    /// The name written into `data-align`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `align="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "start" => Self::Start,
            "end" => Self::End,
            _ => Self::Center,
        }
    }
}

/// The document id a tooltip's name becomes.
pub fn tooltip_id(name: &str) -> String {
    format!("tooltip-{}", name.trim())
}

/// The CSS anchor name a tooltip's name becomes.
pub fn anchor_name(name: &str) -> String {
    format!("--tooltip-{}", name.trim())
}

/// A delay prop read back, or [`None`] when the caller said nothing.
///
/// A `&str` rather than a number because an unwritten numeric prop arrives as
/// `0`, and `0` is the *default* here — the two have to stay tellable apart, or
/// a root would silently stop inheriting its provider's.
pub fn parse_delay(raw: &str) -> Option<u32> {
    match raw.trim() {
        "" => None,
        given => given.parse().ok(),
    }
}

/// Is this name usable as both a document id and a CSS identifier?
fn nameable(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Which of the five a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Provider,
    Trigger,
    Content,
    Arrow,
}

impl Part {
    /// The `data-slot` this part carries — shadcn's, unchanged.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "tooltip",
            Self::Provider => "tooltip-provider",
            Self::Trigger => "tooltip-trigger",
            Self::Content => "tooltip-content",
            Self::Arrow => "tooltip-arrow",
        }
    }

    /// Does this part publish a pair of delays for the parts under it?
    const fn times(self) -> bool {
        matches!(self, Self::Root | Self::Provider)
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part) -> String {
    match part {
        Part::Root | Part::Provider => ROOT.to_string(),
        Part::Content => cn(&[POPOVER_RESET, CONTENT]),
        Part::Arrow => ARROW.to_string(),
        Part::Trigger => String::new(),
    }
}

/// The class list shadcn's `cn("z-50 inline-flex …", className)` returns.
pub fn tooltip_content_variants(class: &str) -> String {
    cn(&[POPOVER_RESET, CONTENT, class])
}

/// The arrow's class list, the caller's own included.
pub fn tooltip_arrow_variants(class: &str) -> String {
    cn(&[ARROW, class])
}

/// One Tooltip part's props, with shadcn's defaults already in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipProps {
    part: Part,
    name: String,
    side: TooltipSide,
    align: TooltipAlign,
    href: String,
    delay: Option<u32>,
    close_delay: Option<u32>,
    class: String,
    attrs: Attrs,
}

impl Default for TooltipProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            name: String::new(),
            side: TooltipSide::Top,
            align: TooltipAlign::Center,
            href: String::new(),
            delay: None,
            close_delay: None,
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

macro_rules! builder {
    ($($fname:ident => $part:expr, $doc:literal;)*) => {
        $(
            #[doc = $doc]
            pub fn $fname() -> TooltipProps {
                TooltipProps { part: $part, ..TooltipProps::default() }
            }
        )*
    };
}

builder! {
    tooltip => Part::Root, "A default [`Tooltip`]'s props.";
    tooltip_provider => Part::Provider, "A default [`TooltipProvider`]'s props.";
    tooltip_trigger => Part::Trigger, "A default [`TooltipTrigger`]'s props.";
    tooltip_content => Part::Content, "A default [`TooltipContent`]'s props.";
    tooltip_arrow => Part::Arrow, "A default arrow's props — [`TooltipContent`] renders one.";
}

impl TooltipProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the part's own, `className`-style.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// The name pairing a trigger with its tooltip.
    ///
    /// Becomes `data-tooltip` on both, `id` on the tooltip, `aria-describedby`
    /// on the trigger, and an `anchor-name`. It has to be unique in the document
    /// and a valid CSS identifier: letters, digits, `-` and `_`, not starting
    /// with a digit.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(
            name.trim().is_empty() || nameable(&name),
            "rahti-ui: `{name}` cannot be a Tooltip name — it is also a CSS anchor \
             name. Use letters, digits, `-` and `_`, not starting with a digit."
        );
        self.name = name;
        self
    }

    /// Which side of the trigger the tooltip opens on.
    ///
    /// What the page *asks for*. The script overwrites `data-side` with where
    /// the tooltip actually landed, because the arrow has to point at the
    /// trigger even after a viewport-edge flip.
    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    /// Which of the trigger's edges the tooltip lines up with.
    pub fn align(mut self, align: TooltipAlign) -> Self {
        self.align = align;
        self
    }

    /// Make the trigger a link rather than a button.
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = href.into();
        self
    }

    /// How long the pointer has to stay before the tooltip opens.
    ///
    /// On a root or a [`TooltipProvider`]. Unset means *inherit*, which is why
    /// this is an [`Option`] — `0` is the default and would otherwise be
    /// indistinguishable from saying nothing.
    pub fn delay(mut self, ms: u32) -> Self {
        self.delay = Some(ms);
        self
    }

    /// How long the tooltip survives the pointer leaving.
    pub fn close_delay(mut self, ms: u32) -> Self {
        self.close_delay = Some(ms);
        self
    }

    /// A delay prop straight off a tag: unwritten leaves it inherited.
    pub fn delay_prop(self, raw: &str) -> Self {
        match parse_delay(raw) {
            Some(ms) => self.delay(ms),
            None => self,
        }
    }

    /// A close-delay prop straight off a tag: unwritten leaves it inherited.
    pub fn close_delay_prop(self, raw: &str) -> Self {
        match parse_delay(raw) {
            Some(ms) => self.close_delay(ms),
            None => self,
        }
    }

    /// The part's own id.
    ///
    /// Not for a tooltip: that id is the [`name`](Self::name), because
    /// `aria-describedby` has to be able to find it.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date.
    ///
    /// Not `data-side` on a tooltip: the script owns that one, and a binding
    /// would be reconciled back to whatever the page asked for the moment
    /// anything re-rendered.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression.
    pub fn on(self, event: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let name = if event.starts_with("on") {
            event.to_string()
        } else {
            format!("on{event}")
        };
        self.bind(&name, expr)
    }

    /// Everything in `attrs`, laid over what is already set.
    pub fn attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = merge(std::mem::take(&mut self.attrs), &attrs);
        self
    }

    /// This part's own utilities, before the caller's.
    fn own_classes(&self) -> String {
        part_classes(self.part)
    }

    /// The class list this part renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[&self.own_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// The root always does. Like [`HoverCard`](super::hover_card) and unlike
    /// everything else in this library, the opening gesture is the script's —
    /// the platform has no declarative "show this while the pointer is there".
    pub fn needs_scope(&self) -> bool {
        matches!(self.part, Part::Root)
            || self
                .attrs
                .iter()
                .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The inline style that anchors a tooltip to its trigger, or nothing.
    fn position_style(&self) -> Option<String> {
        if self.name.trim().is_empty() {
            return None;
        }

        match self.part {
            Part::Trigger => Some(format!("anchor-name:{}", anchor_name(&self.name))),
            Part::Content => Some(format!(
                "position-anchor:{};position-area:{};{}:{}px;\
                 position-try-fallbacks:flip-block,flip-inline",
                anchor_name(&self.name),
                self.side.area(self.align),
                self.side.offset_axis(),
                SIDE_OFFSET,
            )),
            _ => None,
        }
    }

    /// Is this trigger a link?
    fn is_link(&self) -> bool {
        matches!(self.part, Part::Trigger) && !self.href.trim().is_empty()
    }

    /// The open tag's attributes, in shadcn's order.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());
        let named = !self.name.trim().is_empty();

        base = match self.part {
            // Both levels always publish a pair, because each is the fallback
            // for whatever is under it. A root with nothing of its own is
            // filled in from its provider by the script, not here — the server
            // cannot see up the tree.
            Part::Root => base
                .set("data-delay", self.delay.unwrap_or(DELAY_MS).to_string())
                .set(
                    "data-close-delay",
                    self.close_delay.unwrap_or(CLOSE_DELAY_MS).to_string(),
                ),

            Part::Provider => {
                let mut provider = base;
                if let Some(ms) = self.delay {
                    provider = provider.set("data-delay", ms.to_string());
                }
                if let Some(ms) = self.close_delay {
                    provider = provider.set("data-close-delay", ms.to_string());
                }
                provider
            }

            Part::Trigger => {
                let mut trigger = base.set("data-state", "closed");
                if !self.is_link() {
                    trigger = trigger.set("type", "button");
                }
                if named {
                    trigger = trigger
                        .set("data-tooltip", self.name.trim())
                        // The whole of the accessibility story. A tooltip is a
                        // description of its trigger, not a region to travel
                        // to — which is also why nothing in one may be
                        // interactive.
                        .set("aria-describedby", tooltip_id(&self.name));
                }
                if self.is_link() {
                    trigger = trigger.set("href", self.href.trim());
                }
                trigger
            }

            Part::Content => {
                let mut content = base
                    .set("popover", "auto")
                    .set("role", "tooltip")
                    .set("data-side", self.side.slug())
                    .set("data-align", self.align.slug())
                    .set("data-state", "closed");
                if named {
                    content = content
                        .set("id", tooltip_id(&self.name))
                        .set("data-tooltip", self.name.trim());
                }
                content
            }

            // Its own `data-side`, so shadcn's arrow rules can be spelled
            // exactly as shadcn spells them rather than reaching through a
            // `group-` variant for the parent's.
            Part::Arrow => base
                .set("data-side", self.side.slug())
                .set("aria-hidden", "true"),
        };

        if let Some(style) = self.position_style() {
            base = base.set("style", style);
        }

        let own = self.own_classes();
        base = if self.class.trim().is_empty() {
            match own.is_empty() {
                true => base,
                false => base.set("class", own),
            }
        } else if own.is_empty() {
            base.set("class", self.class.trim())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&own, &self.class]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The rendered element, with no boundary around it and no `<script>` in it.
    pub fn render(&self, children: Html) -> Html {
        debug_assert!(
            !matches!(self.part, Part::Root),
            "rahti-ui: a Tooltip root is its script. Use the <Tooltip> tag."
        );

        let attrs = self.attr_list();

        match self.part {
            Part::Trigger if self.is_link() => html! {
                <a @{..attrs}>@{children}</a>
            },
            Part::Trigger => html! {
                <button @{..attrs}>@{children}</button>
            },
            _ => html! {
                <div @{..attrs}>@{children}</div>
            },
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// The three lines every part repeats.
fn build(mut props: TooltipProps, id: &str, attrs: Attrs, bindings: Bindings) -> TooltipProps {
    if !id.is_empty() {
        props = props.id(id);
    }
    props = props.attrs(attrs);
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }
    props
}

/// A default delay for every [`Tooltip`] inside it.
///
/// ```ignore
/// // src/app/layout.rs
/// <TooltipProvider delay="0">
///     <slot />
/// </TooltipProvider>
/// ```
///
/// shadcn's provider, and shadcn's advice about it: put one in the root layout.
/// Its whole job there is `delay = 0` — Base UI's own default is 600ms, and
/// shadcn overrides it for every tooltip in the app from this one place.
///
/// This port defaults to zero anyway, so a provider is optional rather than
/// load-bearing. It is here because a page copied from shadcn's docs writes one,
/// and because *raising* the delay app-wide is a real thing to want.
///
/// `display: contents`, so it lays nothing out. Unlike Radix's, it holds no
/// state and starts no timers — every [`Tooltip`] under it reads the delay off
/// this element when it has none of its own.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TooltipProvider(
    class: &str,
    delay: &str,
    close_delay: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        tooltip_provider()
            .class(class)
            .delay_prop(delay)
            .close_delay_prop(close_delay),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// A trigger and the label it describes.
///
/// ```ignore
/// <Tooltip>
///     <TooltipTrigger name="save" class=@{button_variants(…)}>"Save"</TooltipTrigger>
///     <TooltipContent name="save">"Save your work"</TooltipContent>
/// </Tooltip>
/// ```
///
/// `display: contents`, so the trigger inside it lays out where it was written.
/// The `name` has to be on both halves — the two components cannot see each
/// other.
///
/// ## What the script does
///
/// The opening gesture, because the platform has no attribute for a hover: the
/// pointer entering the trigger opens it, leaving closes it, and a keyboard
/// focus opens it too — but only a `:focus-visible` one, so clicking a button
/// does not leave its own tooltip hanging over it afterwards. A click closes the
/// tooltip outright, which is Base UI's behaviour and the reason this component
/// has no `popovertarget` fallback: a tooltip lives on buttons that do things.
///
/// It also writes the real `data-side` on every open — see the module docs — and
/// places the tooltip in browsers without CSS anchor positioning.
///
/// What is *not* the script: the top layer, light dismiss, `Escape`, and the
/// placement wherever anchor positioning exists.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Tooltip(
    class: &str,
    delay: &str,
    close_delay: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        tooltip()
            .class(class)
            .delay_prop(delay)
            .close_delay_prop(close_delay),
        id,
        attrs,
        bindings,
    );
    let root = props.attr_list();
    let offset = SIDE_OFFSET;
    let delay_fallback = DELAY_MS;
    let close_fallback = CLOSE_DELAY_MS;
    let authored = props.delay.is_some();
    let authored_close = props.close_delay.is_some();
    let hint = Json("hint");

    html! {
        <div @{..root}>
            <slot />
            <span data-slot="tooltip-scope" hidden=@{true} pp-ref={scopeRef}></span>
            <script>r##"
                const OFFSET = @{offset};
                const DELAY_FALLBACK = @{delay_fallback};
                const CLOSE_FALLBACK = @{close_fallback};
                const OWN_DELAY = @{authored};
                const OWN_CLOSE = @{authored_close};
                const HINT = @{hint};

                const scopeRef = pp.ref(null);
                // Only ever used to break an id collision — see below.
                const uid = pp.id();

                pp.effect(() => {
                    const marker = scopeRef.current;
                    if (!marker) return;

                    const root = marker.closest("[data-slot=tooltip]");
                    if (!root) return;

                    const own = (slot) =>
                        Array.from(root.querySelectorAll("[data-slot=" + slot + "]"))
                            .filter((el) => el.closest("[data-slot=tooltip]") === root);

                    const tips = own("tooltip-content");
                    if (tips.length === 0) return;

                    // Root, then provider, then the constant. The root always
                    // renders a pair — the server cannot see up the tree to know
                    // whether a provider had something to say — so `OWN_*` is
                    // how a root that was written with a delay is told apart
                    // from one that merely rendered the default.
                    const read = (el, name, fallback) => {
                        if (!el) return fallback;
                        const raw = parseInt(el.getAttribute(name), 10);
                        return Number.isFinite(raw) && raw >= 0 ? raw : fallback;
                    };
                    const provider = root.closest("[data-slot=tooltip-provider]");
                    const DELAY = OWN_DELAY
                        ? read(root, "data-delay", DELAY_FALLBACK)
                        : read(provider, "data-delay", DELAY_FALLBACK);
                    const CLOSE_DELAY = OWN_CLOSE
                        ? read(root, "data-close-delay", CLOSE_FALLBACK)
                        : read(provider, "data-close-delay", CLOSE_FALLBACK);

                    // A component that renders its children twice — a Sidebar
                    // does, once for its mobile drawer — puts this tooltip's id
                    // in the document twice, and `aria-describedby` then names
                    // whichever comes first. Whoever is not the first renames
                    // itself and repoints its trigger.
                    for (const tip of tips) {
                        const id = tip.getAttribute("id");
                        if (!id) continue;
                        if (document.getElementById(id) === tip) continue;
                        const named = root.querySelector(
                            "[aria-describedby=" + CSS.escape(id) + "]"
                        );
                        const unique = id + "-" + uid;
                        tip.setAttribute("id", unique);
                        if (named) named.setAttribute("aria-describedby", unique);
                    }

                    // `popover="hint"` is the platform's own value for exactly
                    // this: showing a hint does not dismiss an open `auto`
                    // popover, so a tooltip on a control inside a menu no longer
                    // shuts the menu. An unsupported value silently degrades to
                    // `manual` — losing light dismiss and Escape — so it is
                    // applied by reading it back rather than by being served.
                    for (const tip of tips) {
                        try {
                            tip.popover = HINT;
                            if (tip.popover !== HINT) tip.popover = "auto";
                        } catch (ignored) {
                            tip.popover = "auto";
                        }
                    }

                    const anchored =
                        typeof CSS !== "undefined" &&
                        CSS.supports &&
                        CSS.supports("position-area", "block-start center");

                    const triggerFor = (tip) => {
                        const name = tip.getAttribute("data-tooltip");
                        if (!name) return null;
                        const found = own("tooltip-trigger").filter(
                            (el) => el.getAttribute("data-tooltip") === name
                        );
                        return found.length ? found[0] : null;
                    };

                    const tipFor = (trigger) => {
                        const name = trigger.getAttribute("data-tooltip");
                        if (!name) return null;
                        const found = tips.filter(
                            (tip) => tip.getAttribute("data-tooltip") === name
                        );
                        return found.length ? found[0] : null;
                    };

                    // Where the tooltip *actually* is, which is not always where
                    // the page asked for it: `position-try-fallbacks` flips it
                    // near a viewport edge. The arrow is glued to `data-side`,
                    // so a stale one leaves it pointing away from its trigger
                    // over empty space. A flip stays on the axis it started on,
                    // so which axis to measure comes from the current value.
                    const settleSide = (tip) => {
                        const trigger = triggerFor(tip);
                        if (!trigger) return;
                        const a = trigger.getBoundingClientRect();
                        const b = tip.getBoundingClientRect();
                        const was = tip.getAttribute("data-side") || "top";
                        const now =
                            was === "left" || was === "right"
                                ? b.left + b.width / 2 < a.left + a.width / 2
                                    ? "left"
                                    : "right"
                                : b.top + b.height / 2 < a.top + a.height / 2
                                  ? "top"
                                  : "bottom";
                        if (now === was) return;
                        tip.setAttribute("data-side", now);
                        for (const arrow of tip.querySelectorAll(
                            "[data-slot=tooltip-arrow]"
                        )) {
                            arrow.setAttribute("data-side", now);
                        }
                    };

                    // The fallback placement, for browsers without CSS anchor
                    // positioning.
                    const place = (tip) => {
                        const trigger = triggerFor(tip);
                        if (!trigger) return;

                        const at = trigger.getBoundingClientRect();
                        const box = tip.getBoundingClientRect();
                        const side = tip.getAttribute("data-side") || "top";
                        const align = tip.getAttribute("data-align") || "center";
                        const room = { w: window.innerWidth, h: window.innerHeight };

                        let top = 0;
                        let left = 0;

                        if (side === "top" || side === "bottom") {
                            top = side === "bottom" ? at.bottom + OFFSET : at.top - box.height - OFFSET;
                            if (top + box.height > room.h - 4) top = at.top - box.height - OFFSET;
                            if (top < 4) top = at.bottom + OFFSET;
                            left =
                                align === "end"
                                    ? at.right - box.width
                                    : align === "start"
                                      ? at.left
                                      : at.left + at.width / 2 - box.width / 2;
                        } else {
                            left = side === "right" ? at.right + OFFSET : at.left - box.width - OFFSET;
                            if (left + box.width > room.w - 4) left = at.left - box.width - OFFSET;
                            if (left < 4) left = at.right + OFFSET;
                            top =
                                align === "end"
                                    ? at.bottom - box.height
                                    : align === "start"
                                      ? at.top
                                      : at.top + at.height / 2 - box.height / 2;
                        }

                        // The offset is already in `left`/`top` here; the inline
                        // margin that carries it under `position-area` would add
                        // it a second time.
                        tip.style.margin = "0";
                        tip.style.position = "fixed";
                        tip.style.left = Math.max(4, Math.min(left, room.w - box.width - 4)) + "px";
                        tip.style.top = Math.max(4, Math.min(top, room.h - box.height - 4)) + "px";
                    };

                    const pending = new Map();
                    const timers = (tip) => {
                        let t = pending.get(tip);
                        if (!t) {
                            t = { open: 0, close: 0 };
                            pending.set(tip, t);
                        }
                        return t;
                    };
                    const settle = (tip) => {
                        const t = timers(tip);
                        clearTimeout(t.open);
                        clearTimeout(t.close);
                    };

                    const show = (tip) => {
                        if (!tip.matches(":popover-open")) tip.showPopover();
                    };
                    const hide = (tip) => {
                        if (tip.matches(":popover-open")) tip.hidePopover();
                    };

                    const wantOpen = (tip, delay) => {
                        settle(tip);
                        if (tip.matches(":popover-open")) return;
                        if (delay <= 0) return show(tip);
                        timers(tip).open = setTimeout(() => show(tip), delay);
                    };
                    const wantClosed = (tip, delay) => {
                        settle(tip);
                        if (!tip.matches(":popover-open")) return;
                        if (delay <= 0) return hide(tip);
                        timers(tip).close = setTimeout(() => hide(tip), delay);
                    };

                    const enteredTrigger = (event) => {
                        const tip = tipFor(event.currentTarget);
                        if (tip) wantOpen(tip, DELAY);
                    };
                    const leftTrigger = (event) => {
                        const tip = tipFor(event.currentTarget);
                        if (tip) wantClosed(tip, CLOSE_DELAY);
                    };

                    // `:focus-visible` rather than any focus: a tooltip that
                    // stayed up after a mouse click on its own button would sit
                    // over whatever the click just did.
                    const focusedTrigger = (event) => {
                        const trigger = event.currentTarget;
                        if (!trigger.matches(":focus-visible")) return;
                        const tip = tipFor(trigger);
                        if (tip) wantOpen(tip, 0);
                    };
                    const blurredTrigger = (event) => {
                        const tip = tipFor(event.currentTarget);
                        if (tip) wantClosed(tip, 0);
                    };

                    // A click dismisses it. The trigger is a control that does
                    // something, and a label still hanging over it afterwards is
                    // in the way. Base UI does the same.
                    const clickedTrigger = (event) => {
                        const tip = tipFor(event.currentTarget);
                        if (tip) wantClosed(tip, 0);
                    };

                    // Only reachable when a page has given this root a close
                    // delay — which is what makes a tooltip hoverable at all.
                    const enteredTip = (event) => settle(event.currentTarget);
                    const leftTip = (event) => wantClosed(event.currentTarget, CLOSE_DELAY);

                    const toggled = (event) => {
                        const tip = event.currentTarget;
                        const trigger = triggerFor(tip);
                        const open = event.newState === "open";

                        tip.setAttribute("data-state", open ? "open" : "closed");
                        if (trigger) trigger.setAttribute("data-state", open ? "open" : "closed");

                        if (!open) return settle(tip);
                        if (!anchored) place(tip);
                        settleSide(tip);
                    };

                    const triggers = [];
                    for (const tip of tips) {
                        tip.addEventListener("toggle", toggled);
                        tip.addEventListener("pointerenter", enteredTip);
                        tip.addEventListener("pointerleave", leftTip);

                        const trigger = triggerFor(tip);
                        if (!trigger || triggers.indexOf(trigger) >= 0) continue;
                        triggers.push(trigger);
                        trigger.addEventListener("pointerenter", enteredTrigger);
                        trigger.addEventListener("pointerleave", leftTrigger);
                        trigger.addEventListener("focus", focusedTrigger);
                        trigger.addEventListener("blur", blurredTrigger);
                        trigger.addEventListener("click", clickedTrigger);
                    }

                    return () => {
                        for (const tip of tips) {
                            settle(tip);
                            tip.removeEventListener("toggle", toggled);
                            tip.removeEventListener("pointerenter", enteredTip);
                            tip.removeEventListener("pointerleave", leftTip);
                        }
                        for (const trigger of triggers) {
                            trigger.removeEventListener("pointerenter", enteredTrigger);
                            trigger.removeEventListener("pointerleave", leftTrigger);
                            trigger.removeEventListener("focus", focusedTrigger);
                            trigger.removeEventListener("blur", blurredTrigger);
                            trigger.removeEventListener("click", clickedTrigger);
                        }
                    };
                }, []);
            "##</script>
        </div>
    }
}

/// The control the tooltip describes.
///
/// shadcn's trigger has no class list of its own — its `render` prop was always
/// meant to hand the look to a Button — so `button_variants(…)` here is how that
/// crosses. `name` has to match the [`TooltipContent`] beside it.
///
/// A `<button type="button">`, or an `<a>` when given an
/// [`href`](TooltipProps::href). It carries `aria-describedby`, which is the
/// whole of the accessibility contract: the tooltip *describes* this control.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TooltipTrigger(
    class: &str,
    name: &str,
    href: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        tooltip_trigger().class(class).name(name).href(href),
        id,
        attrs,
        bindings,
    );
    let is_link = props.is_link();
    let attrs = props.attr_list();

    match is_link {
        true => html! {
            <a @{..attrs}>
                <slot />
            </a>
        },
        false => html! {
            <button @{..attrs}>
                <slot />
            </button>
        },
    }
}

/// The label — a `popover` in the top layer, with shadcn's arrow inside it.
///
/// ```ignore
/// <TooltipContent name="save" side="right">"Save your work"</TooltipContent>
/// ```
///
/// The arrow is rendered for you, exactly as shadcn's component renders its own
/// rather than exposing it: it is not a decision a call site has to make, and
/// getting it wrong is very visible.
///
/// `side` defaults to `top` — shadcn's default for this component, and different
/// from [`Popover`](super::popover)'s and [`HoverCard`](super::hover_card)'s
/// `bottom`. Whatever it is, the script overwrites `data-side` on every open
/// with where the tooltip really landed, so a viewport-edge flip takes the arrow
/// and the slide-in direction with it.
///
/// **Nothing in here may be interactive.** A tooltip is `aria-describedby`
/// content: a screen reader reads it as part of the trigger, and the pointer
/// only reaches it when a page has set a close delay. Put a link in one and it
/// is unreachable for half your users. That is upstream's rule too.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TooltipContent(
    class: &str,
    name: &str,
    side: &str,
    align: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let side = TooltipSide::parse(side);
    let props = build(
        tooltip_content()
            .class(class)
            .name(name)
            .side(side)
            .align(TooltipAlign::parse(align)),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();
    let arrow = tooltip_arrow().side(side).attr_list();

    html! {
        <div @{..attrs}>
            <slot />
            <div @{..arrow}></div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, Tooltip, TooltipAlign, TooltipContent, TooltipProvider, TooltipSide, anchor_name,
        nameable, parse_delay, tooltip, tooltip_arrow, tooltip_content, tooltip_content_variants,
        tooltip_id, tooltip_trigger,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_tooltip_carries_shadcns_defaults() {
        let markup = TooltipContent(
            "",
            "save",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Save"),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"data-slot="tooltip-content""#),
            "{markup}"
        );
        assert!(markup.contains(r#"popover="auto""#), "{markup}");
        assert!(markup.contains(r#"role="tooltip""#), "{markup}");
        assert!(markup.contains(r#"id="tooltip-save""#), "{markup}");
        // `top`, which is shadcn's default here and nowhere else in the library.
        assert!(markup.contains(r#"data-side="top""#), "{markup}");
        assert!(markup.contains(r#"data-align="center""#), "{markup}");
        assert!(markup.contains("bg-foreground"), "{markup}");
        assert!(markup.contains("max-w-xs"), "{markup}");
    }

    /// shadcn renders the arrow itself rather than exposing it, and so does
    /// this — it is not a decision a call site should have to make.
    #[test]
    fn the_content_renders_its_own_arrow() {
        let markup = TooltipContent(
            "",
            "save",
            "right",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="tooltip-arrow""#), "{markup}");
        // The arrow carries the side too, so shadcn's rules can be spelled the
        // way shadcn spells them.
        assert!(
            markup.matches(r#"data-side="right""#).count() >= 2,
            "{markup}"
        );
        assert!(markup.contains(r#"aria-hidden="true""#), "{markup}");
        assert!(markup.contains("rotate-45"), "{markup}");
    }

    /// The tooltip describes its trigger. That is the whole contract, and it is
    /// why nothing inside one may be interactive.
    #[test]
    fn the_trigger_is_described_by_the_tooltip() {
        let markup = tooltip_trigger()
            .name("save")
            .render(Html::escape("Save"))
            .into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(markup.contains(r#"type="button""#), "{markup}");
        assert!(
            markup.contains(r#"aria-describedby="tooltip-save""#),
            "{markup}"
        );
        assert!(markup.contains(r#"data-tooltip="save""#), "{markup}");
        assert!(markup.contains("anchor-name:--tooltip-save"), "{markup}");
        assert_eq!(tooltip_id("save"), "tooltip-save");
        assert_eq!(anchor_name("save"), "--tooltip-save");
    }

    /// A tooltip lives on controls that do things, so a click must not pop one
    /// up. There is no `popovertarget` here, unlike every other overlay in the
    /// library.
    #[test]
    fn a_trigger_never_carries_popovertarget() {
        let button = tooltip_trigger()
            .name("save")
            .render(Html::empty())
            .into_test_string();
        let link = tooltip_trigger()
            .name("docs")
            .href("/docs")
            .render(Html::empty())
            .into_test_string();

        assert!(!button.contains("popovertarget"), "{button}");
        assert!(!link.contains("popovertarget"), "{link}");
        assert!(link.starts_with("<a "), "{link}");
        assert!(link.contains(r#"href="/docs""#), "{link}");
        assert!(!link.contains("type="), "{link}");
    }

    #[test]
    fn side_and_align_are_written_twice() {
        let tip = tooltip_content()
            .name("save")
            .side(TooltipSide::Left)
            .align(TooltipAlign::End)
            .render(Html::empty())
            .into_test_string();

        assert!(tip.contains(r#"data-side="left""#), "{tip}");
        assert!(tip.contains("position-anchor:--tooltip-save"), "{tip}");
        assert!(
            tip.contains("position-area:inline-start span-block-start"),
            "{tip}"
        );
        // One axis only — all four margins would push an aligned tooltip off the
        // edge it lines up with.
        assert!(tip.contains("margin-inline:4px"), "{tip}");
        assert!(!tip.contains("margin:4px"), "{tip}");
        assert_eq!(TooltipSide::Bottom.offset_axis(), "margin-block");

        assert_eq!(TooltipSide::parse(""), TooltipSide::Top);
        assert_eq!(TooltipSide::parse("bottom"), TooltipSide::Bottom);
        assert_eq!(TooltipAlign::parse(""), TooltipAlign::Center);
    }

    /// The root always publishes a pair; a provider publishes only what it was
    /// given, so a root can tell "inherit" from "the default".
    #[test]
    fn delays_come_from_the_root_then_the_provider() {
        let root = Tooltip(
            "",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(root.contains(r#"data-delay="0""#), "{root}");
        assert!(root.contains(r#"data-close-delay="0""#), "{root}");
        // The script needs to know whether the root meant it.
        assert!(root.contains("const OWN_DELAY = false"), "{root}");

        let slow = Tooltip(
            "",
            "700",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(slow.contains(r#"data-delay="700""#), "{slow}");
        assert!(slow.contains("const OWN_DELAY = true"), "{slow}");

        let quiet = TooltipProvider(
            "",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        let loud = TooltipProvider(
            "",
            "500",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(!quiet.contains("data-delay"), "{quiet}");
        assert!(loud.contains(r#"data-delay="500""#), "{loud}");
    }

    #[test]
    fn an_unwritten_delay_is_not_a_zero_delay() {
        assert_eq!(parse_delay(""), None);
        assert_eq!(parse_delay("0"), Some(0));
        assert_eq!(parse_delay("700"), Some(700));
        assert_eq!(parse_delay("soon"), None);
    }

    /// The deltas from shadcn's Base UI version, as assertions rather than
    /// prose.
    #[test]
    fn the_port_drops_what_only_the_positioner_could_publish() {
        let list = tooltip_content_variants("");

        assert!(!list.contains("--transform-origin"), "{list}");
        assert!(!list.contains("animate-out"), "{list}");
        assert!(!list.contains("data-closed"), "{list}");
        // Base UI's logical sides are never written by this port.
        assert!(!list.contains("inline-start"), "{list}");
        assert!(!list.contains("inline-end"), "{list}");
        // The two states shadcn distinguishes collapse into `:popover-open`.
        assert!(!list.contains("delayed-open"), "{list}");
        assert!(list.contains("open:animate-in"), "{list}");
        assert!(list.contains("inset-auto"), "{list}");
    }

    /// The two shadcn declarations this port had to *change* rather than drop or
    /// re-prefix. Both were found in a browser, and both look correct on paper.
    #[test]
    fn the_two_declarations_the_popover_api_forced() {
        let list = tooltip_content_variants("");

        // 1. A UA rule is what hides a closed popover, and a UA rule loses to
        //    any author declaration — so an unconditional `display` leaves every
        //    tooltip on the page visible in normal flow, never in the top layer.
        assert!(list.contains("open:inline-flex"), "{list}");
        assert!(!list.contains(" inline-flex"), "{list}");
        assert!(!list.starts_with("inline-flex"), "{list}");
        for display in [" block", " flex", " grid", " hidden", " inline"] {
            assert!(!list.contains(display), "{display} in {list}");
        }

        // 2. `position-area … center` makes the anchor the containing block, so
        //    `fit-content` collapses the tooltip to the width of the button it
        //    is labelling. `max-content` is not bounded by it.
        assert!(list.contains("w-max"), "{list}");
        assert!(!list.contains("w-fit"), "{list}");
        // And the cap that makes it wrap is still shadcn's.
        assert!(list.contains("max-w-xs"), "{list}");

        // 3. Chrome's `[popover]` rule also says `overflow: auto`, and an arrow
        //    exists to stick out of the box — so `auto` reads it as overflowing
        //    content and gives the tooltip a scrollbar to reach it with.
        assert!(list.contains("overflow-visible"), "{list}");
        assert!(list.contains("inset-auto"), "{list}");
    }

    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = tooltip_content()
            .name("save")
            .class("max-w-sm")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("max-w-sm"), "{markup}");
    }

    #[test]
    fn only_the_root_always_needs_the_runtime() {
        assert!(tooltip().needs_scope());
        assert!(!tooltip_content().name("save").needs_scope());
        assert!(!tooltip_arrow().needs_scope());
        assert!(!tooltip_content().class("max-w-sm").needs_scope());
        assert!(
            tooltip_trigger()
                .name("save")
                .on(
                    "click",
                    crate::components::rahti_ui::attrs::ui_handler!({ save() })
                )
                .needs_scope()
        );
    }

    #[test]
    fn attrs_are_laid_over_the_parts_own() {
        let markup = tooltip_content()
            .name("save")
            .attrs(Attrs::new().set("data-testid", "tip").unset("data-align"))
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-testid="tip""#), "{markup}");
        assert!(!markup.contains("data-align"), "{markup}");
    }

    #[test]
    fn a_name_has_to_be_a_css_identifier_too() {
        assert!(nameable("save"));
        assert!(nameable("save-2"));
        assert!(!nameable(""));
        assert!(!nameable("2save"));
        assert!(!nameable("save it"));
    }
}

#[cfg(test)]
mod audit {
    use super::{ARROW, CONTENT, POPOVER_RESET};
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered.
    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("CONTENT", cn(&[POPOVER_RESET, CONTENT])),
            ("ARROW", ARROW.to_string()),
        ] {
            let found = clashes(&list);

            assert!(
                found.is_empty(),
                "{name}: {} clashes:\n{}",
                found.len(),
                found.join("\n")
            );
        }
    }

    /// Each layer is also clean on its own, so a failure above points at the
    /// join rather than at one string.
    #[test]
    fn no_layer_contradicts_itself() {
        for (name, layer) in [
            ("POPOVER_RESET", POPOVER_RESET),
            ("CONTENT", CONTENT),
            ("ARROW", ARROW),
        ] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
