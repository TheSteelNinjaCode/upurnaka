// rahti-ui: popover
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.3.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Popover — a port of shadcn/ui's `popover`.
//!
//! shadcn's is Radix's Popover: a portal, a dismiss layer, a focus scope and
//! Floating UI underneath doing collision detection. This one is the **HTML
//! popover API**, the same trade [`DropdownMenu`](super::dropdown_menu) makes
//! and for the same reasons.
//!
//! ```text
//! Radix                                  here
//! ────────────────────────────────────   ─────────────────────────────────────
//! Portal — escape the stacking context   the top layer
//! DismissableLayer — click outside       light dismiss, built in
//! Escape to close                        built in
//! open state + onOpenChange              the `popovertarget` attribute
//! Floating UI placement                  CSS anchor positioning; script after
//! FocusScope                             the script
//! ```
//!
//! A `<button popovertarget="…">` opens and closes a popover **with no
//! JavaScript at all** — it is a declarative association in the markup, the way
//! `<label for>` is. So a popover here opens, closes, dismisses on an outside
//! click and dismisses on `Escape` before the runtime has loaded, and the script
//! is left with three jobs: place it where the browser cannot do it in CSS, move
//! focus in and back out, and publish the `data-state` Radix publishes.
//!
//! ## The trigger and the panel are paired by name
//!
//! `popovertarget` takes an **id**, and the two halves are separate components
//! whose children are opaque to each other. So the pairing is a name the page
//! writes on both — the same divergence [`Accordion`](super::accordion)'s `name`
//! and [`DropdownMenu`](super::dropdown_menu)'s `menu` made:
//!
//! ```ignore
//! <Popover>
//!     <PopoverTrigger name="dimensions" class=@{button_variants(…)}>"Open"</PopoverTrigger>
//!     <PopoverContent name="dimensions" class="w-80">…</PopoverContent>
//! </Popover>
//! ```
//!
//! The name becomes `id="popover-dimensions"` on the panel and
//! `popovertarget="popover-dimensions"` on the trigger. It is *not* the
//! `<button>`'s form name — nothing here submits — and it has to be unique in
//! the document and a valid CSS identifier, because it is also an anchor name. A
//! [`debug_assert`] catches the spellings that are not.
//!
//! ## Position, twice over
//!
//! A popover is in the top layer, so it has no ancestor to be positioned
//! against. There are two ways to place one and this does both:
//!
//! 1. **CSS anchor positioning**, as inline styles: `anchor-name` on the trigger
//!    and `position-anchor` plus `position-area` on the panel. Where the browser
//!    has it the panel is placed with no JavaScript, and
//!    `position-try-fallbacks` flips it when it would go off screen. It is an
//!    inline style rather than a class because the anchor name varies per
//!    popover, and Tailwind can only emit classes it finds spelled out in the
//!    source — the same wall [`Tabs`](super::tabs) hit.
//! 2. **The script**, for the browsers that do not have it yet. It measures on
//!    `toggle` and writes `left`/`top`, with the same flip.
//!
//! The script asks `CSS.supports` and does nothing when the first one is already
//! handling it.
//!
//! ## Which shadcn popover this is
//!
//! The registry, and every class string is shadcn's with three changes:
//!
//! - `data-[state=open]:` becomes `open:`, Tailwind's variant for `[open]`,
//!   `:open` **and** `:popover-open`. shadcn's enter animation crosses with the
//!   prefix rewritten and nothing else touched.
//! - The three `data-[state=closed]:` exit utilities are dropped. A popover is
//!   `display: none` the instant it closes, so a keyframe exit would never be
//!   seen — the same wall [`Sheet`](super::sheet) hit, which paid a script to
//!   get around it.
//! - `origin-(--radix-popover-content-transform-origin)` is dropped: it is a
//!   custom property Floating UI publishes and nothing here does. The zoom falls
//!   back to `center`.
//!
//! The script still writes `data-state`, even though no class list here reads
//! it, because a caller pasting shadcn's own `data-[state=open]:` utilities into
//! [`class`](PopoverProps::class) should find them working. It survives the
//! attribute reconciler for a specific reason — see [`PopoverContent`].
//!
//! ## The parts
//!
//! ```html
//! <div data-slot="popover" class="contents">
//!   <button data-slot="popover-trigger" type="button" popovertarget="popover-dimensions"
//!           aria-haspopup="dialog" aria-expanded="false"
//!           style="anchor-name:--popover-dimensions">…</button>
//!   <div data-slot="popover-content" id="popover-dimensions" popover="auto" role="dialog"
//!        tabindex="-1" data-side="bottom" data-align="center" data-state="closed"
//!        data-popover="dimensions"
//!        style="position-anchor:--popover-dimensions;position-area:block-end center;
//!               margin-block:4px;…">
//!     <div data-slot="popover-header">
//!       <div data-slot="popover-title">…</div>
//!       <p data-slot="popover-description">…</p>
//!     </div>
//!   </div>
//!   <span data-slot="popover-scope" hidden></span>
//!   <script>…</script>
//! </div>
//! ```

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, Json, component, html};

/// The root: `display: contents`, so the trigger inside it lays out where it was
/// written.
///
/// Radix's Root renders nothing at all. This has to render something — the
/// script lives in it and searches from it — so it renders a box and takes the
/// box back out of the layout. Same arrangement, and same reason, as
/// [`DropdownMenu`](super::dropdown_menu)'s root.
pub const ROOT: &str = "contents";

/// The panel. shadcn's string, less the Radix custom property and the exit
/// animation.
///
/// `open:` is `data-[state=open]:` — Tailwind's `open:` variant matches
/// `:popover-open`, so shadcn's enter animation crosses with the prefix
/// rewritten and nothing else touched.
pub const CONTENT: &str = "z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground \
                           shadow-md outline-hidden \
                           data-[side=bottom]:slide-in-from-top-2 \
                           data-[side=left]:slide-in-from-right-2 \
                           data-[side=right]:slide-in-from-left-2 \
                           data-[side=top]:slide-in-from-bottom-2 \
                           open:animate-in open:fade-in-0 open:zoom-in-95";

/// What a `popover` needs said about it that a Radix portal did not.
///
/// The UA stylesheet pins a popover with `inset: 0` and centres it with
/// `margin: auto`, sizes it `fit-content`, and gives it a border and padding.
/// Tailwind's preflight already clears the margin, border and padding through
/// its `*` rule; the inset it leaves alone, and an uncleared `inset: 0` fights
/// every `left`/`top` the anchor or the script writes.
pub const POPOVER_RESET: &str = "inset-auto";

/// A title and a description stacked. shadcn's string, unchanged.
pub const HEADER: &str = "flex flex-col gap-1 text-sm";

/// The title. shadcn's string, unchanged — the size comes from [`HEADER`].
pub const TITLE: &str = "font-medium";

/// The description. shadcn's string, unchanged.
pub const DESCRIPTION: &str = "text-muted-foreground";

/// How far the panel sits from its anchor — shadcn's `sideOffset = 4`.
pub const SIDE_OFFSET: u32 = 4;

/// Which side of the anchor the panel opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl PopoverSide {
    /// The name written into `data-side`, which shadcn's four slide utilities
    /// read.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// The CSS `position-area` this side asks for.
    ///
    /// The span keeps the panel's near edge lined up with the anchor's rather
    /// than centring it, which is what `align="start"` means in Radix.
    pub const fn area(self, align: PopoverAlign) -> &'static str {
        match (self, align) {
            (Self::Bottom, PopoverAlign::Start) => "block-end span-inline-end",
            (Self::Bottom, PopoverAlign::Center) => "block-end center",
            (Self::Bottom, PopoverAlign::End) => "block-end span-inline-start",
            (Self::Top, PopoverAlign::Start) => "block-start span-inline-end",
            (Self::Top, PopoverAlign::Center) => "block-start center",
            (Self::Top, PopoverAlign::End) => "block-start span-inline-start",
            (Self::Left, PopoverAlign::Start) => "inline-start span-block-end",
            (Self::Left, PopoverAlign::Center) => "inline-start center",
            (Self::Left, PopoverAlign::End) => "inline-start span-block-start",
            (Self::Right, PopoverAlign::Start) => "inline-end span-block-end",
            (Self::Right, PopoverAlign::Center) => "inline-end center",
            (Self::Right, PopoverAlign::End) => "inline-end span-block-start",
        }
    }

    /// Which margin carries [`SIDE_OFFSET`] under a `position-area`.
    ///
    /// The offset has to be a margin, because `position-area` places the panel
    /// against the anchor's edge and there is no gap property for it. But a
    /// margin on *all four* sides also pushes the panel along the axis it is
    /// **aligned** on, and an `align="start"` panel then sits 4px in from the
    /// edge it is supposed to line up with — measured, and visible.
    ///
    /// So the offset goes on one axis only: the block axis for a panel above or
    /// below, the inline axis for one beside. That is also what survives
    /// `position-try-fallbacks` — a flip swaps which edge of the axis is used
    /// and the margin is already on both of them, where a single
    /// `margin-top` would leave the flipped panel touching its anchor.
    pub const fn offset_axis(self) -> &'static str {
        match self {
            Self::Top | Self::Bottom => "margin-block",
            Self::Left | Self::Right => "margin-inline",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `side="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "top" => Self::Top,
            "left" => Self::Left,
            "right" => Self::Right,
            _ => Self::Bottom,
        }
    }
}

/// Which of the anchor's edges the panel lines up with.
///
/// shadcn's popover defaults to `center` where its dropdown menu defaults to
/// `start` — a difference kept rather than smoothed over, because it is part of
/// what makes the two look like themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    Start,
    #[default]
    Center,
    End,
}

impl PopoverAlign {
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

/// The document id a popover's name becomes.
pub fn popover_id(name: &str) -> String {
    format!("popover-{}", name.trim())
}

/// The CSS anchor name a popover's name becomes.
pub fn anchor_name(name: &str) -> String {
    format!("--popover-{}", name.trim())
}

/// Is this name usable as both a document id and a CSS identifier?
///
/// It has to be both, because the pairing is a `popovertarget` *and* an
/// `anchor-name`. Letters, digits, `-` and `_`, not starting with a digit.
fn nameable(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Which of the seven a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Trigger,
    Content,
    Anchor,
    Header,
    Title,
    Description,
}

impl Part {
    /// The `data-slot` this part carries — shadcn's, unchanged.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "popover",
            Self::Trigger => "popover-trigger",
            Self::Content => "popover-content",
            Self::Anchor => "popover-anchor",
            Self::Header => "popover-header",
            Self::Title => "popover-title",
            Self::Description => "popover-description",
        }
    }

    /// Does this part take part in the trigger/panel pairing?
    const fn named(self) -> bool {
        matches!(self, Self::Trigger | Self::Content | Self::Anchor)
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part) -> String {
    match part {
        Part::Root => ROOT.to_string(),
        Part::Content => cn(&[POPOVER_RESET, CONTENT]),
        Part::Header => HEADER.to_string(),
        Part::Title => TITLE.to_string(),
        Part::Description => DESCRIPTION.to_string(),
        Part::Trigger | Part::Anchor => String::new(),
    }
}

/// The class list shadcn's `cn("z-50 w-72 …", className)` returns.
///
/// Write `data-side` alongside it — the four slide utilities read it — and give
/// the element `popover` and an `id`, because the open and close are the
/// `popovertarget` association rather than anything in the class list.
pub fn popover_content_variants(class: &str) -> String {
    cn(&[POPOVER_RESET, CONTENT, class])
}

/// [`PopoverHeader`]'s class list, the caller's own included.
pub fn popover_header_variants(class: &str) -> String {
    cn(&[HEADER, class])
}

/// [`PopoverTitle`]'s class list, the caller's own included.
pub fn popover_title_variants(class: &str) -> String {
    cn(&[TITLE, class])
}

/// [`PopoverDescription`]'s class list, the caller's own included.
pub fn popover_description_variants(class: &str) -> String {
    cn(&[DESCRIPTION, class])
}

/// One Popover part's props, with shadcn's defaults already in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverProps {
    part: Part,
    name: String,
    side: PopoverSide,
    align: PopoverAlign,
    class: String,
    attrs: Attrs,
}

impl Default for PopoverProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            name: String::new(),
            side: PopoverSide::Bottom,
            align: PopoverAlign::Center,
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

macro_rules! builder {
    ($($fname:ident => $part:expr, $doc:literal;)*) => {
        $(
            #[doc = $doc]
            pub fn $fname() -> PopoverProps {
                PopoverProps { part: $part, ..PopoverProps::default() }
            }
        )*
    };
}

builder! {
    popover => Part::Root, "A default [`Popover`]'s props.";
    popover_trigger => Part::Trigger, "A default [`PopoverTrigger`]'s props.";
    popover_content => Part::Content, "A default [`PopoverContent`]'s props.";
    popover_anchor => Part::Anchor, "A default [`PopoverAnchor`]'s props.";
    popover_header => Part::Header, "A default [`PopoverHeader`]'s props.";
    popover_title => Part::Title, "A default [`PopoverTitle`]'s props.";
    popover_description => Part::Description, "A default [`PopoverDescription`]'s props.";
}

impl PopoverProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the part's own, `className`-style.
    ///
    /// On a trigger this is the whole class list — shadcn's has none of its own,
    /// because its `asChild` was always meant to hand the look to a Button.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// The name pairing a trigger with its panel.
    ///
    /// Becomes `popovertarget` on the trigger and `id` on the panel, and an
    /// `anchor-name` on both. Not the `<button>`'s form name — nothing here
    /// submits. It has to be unique in the document and a valid CSS identifier:
    /// letters, digits, `-` and `_`, not starting with a digit.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(
            name.trim().is_empty() || nameable(&name),
            "rahti-ui: `{name}` cannot be a Popover name — it is also a CSS anchor \
             name. Use letters, digits, `-` and `_`, not starting with a digit."
        );
        self.name = name;
        self
    }

    /// Which side of the anchor the panel opens on.
    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    /// Which of the anchor's edges the panel lines up with.
    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = align;
        self
    }

    /// The part's own id.
    ///
    /// Not for a panel: that id is the [`name`](Self::name), because
    /// `popovertarget` has to be able to find it.
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
    /// `popover`, `popovertarget` and the panel's `id` are the association that
    /// makes the component work without a script — bind any of them and it stops
    /// working before the runtime arrives rather than after. And a panel is an
    /// effect-managed surface, so `data-state`, `style` and `hidden` are not
    /// reconciled on it at all; see [`PopoverContent`].
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression.
    ///
    /// A handler on a part runs beside the script's own rather than instead of
    /// it — `ontoggle` on a panel is the usual one, and it still gets placed.
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
    pub fn needs_scope(&self) -> bool {
        matches!(self.part, Part::Root)
            || self
                .attrs
                .iter()
                .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The inline style that anchors a panel to its trigger, or nothing.
    ///
    /// Written as a style rather than a class because the anchor name varies per
    /// popover, and Tailwind can only emit classes it finds spelled out in the
    /// source. Where the browser has anchor positioning this places the panel
    /// with no JavaScript at all; where it does not, every declaration is
    /// ignored and the script does it instead.
    fn position_style(&self) -> Option<String> {
        if self.name.trim().is_empty() {
            return None;
        }

        match self.part {
            Part::Trigger | Part::Anchor => {
                Some(format!("anchor-name:{}", anchor_name(&self.name)))
            }
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

    /// The open tag's attributes, in shadcn's order.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());
        let named = self.part.named() && !self.name.trim().is_empty();

        base = match self.part {
            Part::Trigger => {
                // A `<button>`, and not as a matter of taste: `popovertarget` is
                // only an attribute of `<button>` and the button-ish `<input>`s.
                // On anything else it parses, serialises into the HTML, and does
                // nothing — `popoverTargetElement` reads `null`, clicking is
                // inert, and nothing is logged anywhere. Measured in the browser
                // while fixing the Dropdown Menu's submenus.
                let mut trigger = base.set("type", "button");
                if named {
                    // The whole of the open and close, and not a line of it is
                    // JavaScript.
                    trigger = trigger
                        .set("popovertarget", popover_id(&self.name))
                        .set("aria-haspopup", "dialog")
                        // Served closed and moved by the script's `toggle`
                        // handler. The platform has no declarative spelling for
                        // it, and a trigger that never announces itself as
                        // expanded is the one thing the `popovertarget` trade
                        // would otherwise cost.
                        .set("aria-expanded", "false");
                }
                trigger
            }

            Part::Content => {
                let mut content = base
                    .set("popover", "auto")
                    // Radix's role, kept. The panel is not modal — the platform
                    // lets focus leave it — which is Radix's default too.
                    .set("role", "dialog")
                    // So the script has something to focus when the panel holds
                    // nothing focusable of its own.
                    .set("tabindex", "-1")
                    .set("data-side", self.side.slug())
                    .set("data-align", self.align.slug())
                    .set("data-state", "closed");
                if named {
                    content = content
                        .set("id", popover_id(&self.name))
                        // Not the same thing as the id: the id is what
                        // `popovertarget` resolves, and the script rewrites it
                        // on a collision. The name is how a panel finds the
                        // anchor written for it.
                        .set("data-popover", self.name.trim());
                }
                content
            }

            Part::Anchor if named => base.set("data-popover", self.name.trim()),

            _ => base,
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
    ///
    /// The root has no `render`: it *is* its script.
    pub fn render(&self, children: Html) -> Html {
        debug_assert!(
            !matches!(self.part, Part::Root),
            "rahti-ui: a Popover root is its script. Use the <Popover> tag."
        );

        let attrs = self.attr_list();

        match self.part {
            Part::Trigger => html! {
                <button @{..attrs}>@{children}</button>
            },
            Part::Description => html! {
                <p @{..attrs}>@{children}</p>
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
fn build(mut props: PopoverProps, id: &str, attrs: Attrs, bindings: Bindings) -> PopoverProps {
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

/// A trigger and the panel it opens.
///
/// ```ignore
/// <Popover>
///     <PopoverTrigger name="dimensions" class=@{button_variants(…)}>"Open"</PopoverTrigger>
///     <PopoverContent name="dimensions" class="w-80">
///         <PopoverHeader>
///             <PopoverTitle>"Dimensions"</PopoverTitle>
///             <PopoverDescription>"Set the dimensions for the layer."</PopoverDescription>
///         </PopoverHeader>
///     </PopoverContent>
/// </Popover>
/// ```
///
/// `display: contents`, so the trigger inside it lays out where it was written.
/// The `name` has to be on both halves — `popovertarget` takes an id, and the
/// two components cannot see each other.
///
/// ## What is not the script
///
/// Opening, closing, dismissing on an outside click and dismissing on `Escape`
/// are the popover API's, and they work before the runtime has loaded. Where the
/// browser has CSS anchor positioning, so does the placement. The script is left
/// with focus — into the panel when it opens, back to the trigger when it closes
/// — with placement in the browsers that cannot do it in CSS, and with the
/// `data-state` and `aria-expanded` the platform has no declarative spelling
/// for.
///
/// None of its state is `pp.state`, for the reason
/// [`ScrollArea`](super::scroll_area) gives: a re-render would hand the
/// attribute reconciler a chance to undo the inline `left`/`top` it writes.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Popover(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let props = build(popover().class(class), id, attrs, bindings);
    let root = props.attr_list();
    let offset = SIDE_OFFSET;
    let focusable = Json(
        "a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),\
         textarea:not([disabled]),[tabindex]:not([tabindex='-1'])",
    );
    // The `<input>` types that have a text selection to move a caret within.
    // `setSelectionRange` throws on the others — `number`, `email`, `date`,
    // `color` and the rest — so the list is an allowlist rather than a filter.
    let textual = Json(["text", "search", "url", "tel", "password"]);

    html! {
        <div @{..root}>
            <slot />
            <span data-slot="popover-scope" hidden=@{true} pp-ref={scopeRef}></span>
            <script>r##"
                const OFFSET = @{offset};
                const FOCUSABLE = @{focusable};
                const TEXTUAL = @{textual};

                // `focus()` on a text field leaves the caret at offset 0, in
                // front of whatever is already in it — so the first keystroke
                // lands at the wrong end of the value. Radix does not have that
                // problem because its focus scope *selects* the field, and
                // shadcn's popover therefore opens with the caret after the
                // text. This is the same intent with nothing highlighted.
                const caretToEnd = (el) => {
                    const textual =
                        el.tagName === "TEXTAREA" ||
                        (el.tagName === "INPUT" &&
                            TEXTUAL.includes((el.getAttribute("type") || "text").toLowerCase()));
                    if (!textual) return;
                    const at = el.value.length;
                    // Guarded anyway: a field can refuse a selection for reasons
                    // its `type` does not announce, and a throw here would take
                    // the placement above it down with it.
                    try {
                        el.setSelectionRange(at, at);
                    } catch (ignored) {}
                };

                const scopeRef = pp.ref(null);
                // Only ever used to break an id collision — see below.
                const uid = pp.id();

                pp.effect(() => {
                    const marker = scopeRef.current;
                    if (!marker) return;

                    const root = marker.closest("[data-slot=popover]");
                    if (!root) return;

                    const own = (slot) =>
                        Array.from(root.querySelectorAll("[data-slot=" + slot + "]"))
                            .filter((el) => el.closest("[data-slot=popover]") === root);

                    const panels = own("popover-content");
                    if (panels.length === 0) return;

                    // A component that renders its children twice — a Sidebar
                    // does, once for its mobile drawer — puts this panel's id in
                    // the document twice, and `popovertarget` resolves to
                    // whichever comes first. Whoever is not the first renames
                    // itself and its trigger, so the two stop fighting over the
                    // name. `pp.id()` is per component instance, so the two
                    // copies pick different ones.
                    for (const panel of panels) {
                        const id = panel.getAttribute("id");
                        if (!id) continue;
                        if (document.getElementById(id) === panel) continue;
                        const trigger = root.querySelector(
                            "[popovertarget=" + CSS.escape(id) + "]"
                        );
                        const unique = id + "-" + uid;
                        panel.setAttribute("id", unique);
                        if (trigger) trigger.setAttribute("popovertarget", unique);
                    }

                    // Where the browser has CSS anchor positioning the inline
                    // styles have already placed the panel, and this does
                    // nothing at all. Asked once rather than per open.
                    const anchored =
                        typeof CSS !== "undefined" &&
                        CSS.supports &&
                        CSS.supports("position-area", "block-end center");

                    const triggerFor = (panel) => {
                        const id = panel.getAttribute("id");
                        if (!id) return null;
                        return root.querySelector("[popovertarget=" + CSS.escape(id) + "]");
                    };

                    // An explicit <PopoverAnchor> carrying this panel's name, if
                    // the page wrote one. The last wins, which is also the rule
                    // CSS applies when two elements share an `anchor-name`.
                    const anchorFor = (panel) => {
                        const name = panel.getAttribute("data-popover");
                        if (!name) return null;
                        const found = own("popover-anchor").filter(
                            (el) => el.getAttribute("data-popover") === name
                        );
                        return found.length ? found[found.length - 1] : null;
                    };

                    // The trigger and an explicit anchor carry the *same*
                    // `anchor-name`, and CSS resolves that to whichever comes
                    // last in the document. Taking the trigger's away settles it
                    // the same way whatever order the page wrote them in.
                    for (const panel of panels) {
                        if (!anchorFor(panel)) continue;
                        const trigger = triggerFor(panel);
                        if (trigger) trigger.style.removeProperty("anchor-name");
                    }

                    // The fallback placement: measure the anchor, put the panel
                    // beside it, and flip when it would run off the edge. This
                    // is the part CSS anchor positioning does declaratively.
                    const place = (panel) => {
                        const anchor = anchorFor(panel) || triggerFor(panel);
                        if (!anchor) return;

                        const at = anchor.getBoundingClientRect();
                        const box = panel.getBoundingClientRect();
                        const side = panel.getAttribute("data-side") || "bottom";
                        const align = panel.getAttribute("data-align") || "center";
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

                        // The offset is already in `left`/`top` here. The inline
                        // `margin` that carries it under `position-area` would
                        // otherwise add it a second time.
                        panel.style.margin = "0";
                        panel.style.position = "fixed";
                        panel.style.left = Math.max(4, Math.min(left, room.w - box.width - 4)) + "px";
                        panel.style.top = Math.max(4, Math.min(top, room.h - box.height - 4)) + "px";
                    };

                    const toggled = (event) => {
                        const panel = event.currentTarget;
                        const trigger = triggerFor(panel);
                        const open = event.newState === "open";

                        // Radix publishes both of these and the platform
                        // publishes neither. `data-state` is what shadcn's own
                        // `data-[state=open]:` utilities read, so a class list
                        // pasted from the docs works.
                        panel.setAttribute("data-state", open ? "open" : "closed");
                        if (trigger) trigger.setAttribute("aria-expanded", open ? "true" : "false");

                        if (!open) {
                            // Give the trigger its focus back, which is what a
                            // dialog does for free and a popover does not.
                            if (trigger && panel.contains(document.activeElement)) trigger.focus();
                            return;
                        }

                        if (!anchored) place(panel);

                        // Radix's FocusScope, in one line: the first thing worth
                        // typing into, or the panel itself when there is none.
                        // `preventScroll` because the panel is already in view —
                        // it was just placed beside its anchor.
                        const landed = panel.querySelector(FOCUSABLE) || panel;
                        landed.focus({ preventScroll: true });
                        caretToEnd(landed);
                    };

                    for (const panel of panels) panel.addEventListener("toggle", toggled);

                    return () => {
                        for (const panel of panels) panel.removeEventListener("toggle", toggled);
                    };
                }, []);
            "##</script>
        </div>
    }
}

/// The button that opens the panel.
///
/// shadcn's trigger has no class list of its own — its `asChild` was always
/// meant to hand the look to a Button — so `button_variants(…)` here is how that
/// crosses. `name` has to match the [`PopoverContent`] beside it.
///
/// It is a `<button>` because `popovertarget` is only an attribute of `<button>`
/// and the button-ish `<input>`s. On a `<div>` it parses, serialises, and does
/// nothing: `popoverTargetElement` reads `null`, clicking is inert, and nothing
/// is logged anywhere.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PopoverTrigger(
    class: &str,
    name: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        popover_trigger().class(class).name(name),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    html! {
        <button @{..attrs}>
            <slot />
        </button>
    }
}

/// The panel — a `popover` in the top layer.
///
/// ```ignore
/// <PopoverContent name="dimensions" class="w-80" side="right" align="start">…</PopoverContent>
/// ```
///
/// Being in the top layer is the point rather than a detail: a popover opened
/// from inside anything that clips — a [`ScrollArea`](super::scroll_area), a
/// [`Sidebar`](super::sidebar), a cell in a scrolling table — would otherwise be
/// cut off at the edge of it.
///
/// `side` and `align` are Radix's, and they place it twice: as a `position-area`
/// for browsers with CSS anchor positioning, and as the numbers the script uses
/// for the rest. Both flip when the panel would run off screen.
///
/// ## What the runtime will not touch here
///
/// The browser runtime keeps a set of overlay `data-slot`s whose attributes it
/// refuses to reconcile, and `popover-content` is one of them: `data-state`,
/// `data-open`, `style`, `aria-hidden`, `aria-checked`, `hidden` and `inert` are
/// neither set nor removed on this element. That is why the script's
/// `data-state` and its inline `left`/`top` survive a re-render — and equally
/// why binding any of those seven from a call site does nothing at all. Bind
/// something else, or write it from an effect.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PopoverContent(
    class: &str,
    name: &str,
    side: &str,
    align: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        popover_content()
            .class(class)
            .name(name)
            .side(PopoverSide::parse(side))
            .align(PopoverAlign::parse(align)),
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

/// Something other than the trigger for the panel to open beside.
///
/// ```ignore
/// <Popover>
///     <PopoverAnchor name="row" class="rounded-md border p-4">"Row"</PopoverAnchor>
///     <PopoverTrigger name="row" class=@{button_variants(…)}>"Edit"</PopoverTrigger>
///     <PopoverContent name="row">…</PopoverContent>
/// </Popover>
/// ```
///
/// Radix's Anchor, and it works the same way: give it the popover's `name` and
/// the panel is placed against *it* rather than against the button that opened
/// it. Both carry the same `anchor-name`, CSS resolves that to whichever comes
/// last in the document, and the script takes the trigger's away at mount so the
/// answer does not depend on the order the two were written in.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PopoverAnchor(
    class: &str,
    name: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        popover_anchor().class(class).name(name),
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

/// A title and a description stacked, at the top of a panel.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PopoverHeader(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(popover_header().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// The panel's title.
///
/// A `<div>` rather than a heading, which is shadcn's own choice: a popover
/// turns up in the middle of a document, and a heading there would land in the
/// outline underneath whatever section it happens to sit in.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PopoverTitle(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(popover_title().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// The line under the title.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PopoverDescription(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(popover_description().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <p @{..attrs}>
            <slot />
        </p>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, PopoverAlign, PopoverSide, anchor_name, nameable, popover, popover_anchor,
        popover_content, popover_content_variants, popover_description, popover_header, popover_id,
        popover_title, popover_trigger,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_panel_carries_shadcns_defaults() {
        let markup = popover_content()
            .name("dimensions")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(
            markup.contains(r#"data-slot="popover-content""#),
            "{markup}"
        );
        assert!(markup.contains(r#"popover="auto""#), "{markup}");
        assert!(markup.contains(r#"role="dialog""#), "{markup}");
        assert!(markup.contains(r#"tabindex="-1""#), "{markup}");
        // shadcn's own defaults: bottom, and centred rather than started.
        assert!(markup.contains(r#"data-side="bottom""#), "{markup}");
        assert!(markup.contains(r#"data-align="center""#), "{markup}");
        assert!(markup.contains("w-72"), "{markup}");
        assert!(markup.contains("bg-popover"), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
    }

    /// The pairing is two attributes and an id, and neither half can see the
    /// other — so the name has to be written on both.
    #[test]
    fn the_pairing_is_an_id_and_no_javascript() {
        let trigger = popover_trigger()
            .name("dimensions")
            .render(Html::empty())
            .into_test_string();
        let panel = popover_content()
            .name("dimensions")
            .render(Html::empty())
            .into_test_string();

        assert_eq!(popover_id("dimensions"), "popover-dimensions");
        assert!(
            trigger.contains(r#"popovertarget="popover-dimensions""#),
            "{trigger}"
        );
        assert!(panel.contains(r#"id="popover-dimensions""#), "{panel}");
        // And the panel knows its own name, which is how it finds an anchor.
        assert!(panel.contains(r#"data-popover="dimensions""#), "{panel}");
    }

    /// `popovertarget` is only an attribute of `<button>` and the button-ish
    /// `<input>`s. On anything else it is inert, with nothing logged anywhere —
    /// measured in the browser, and asserted here so it stays a `<button>`.
    #[test]
    fn a_trigger_is_a_button_because_popovertarget_only_works_on_one() {
        let markup = popover_trigger()
            .name("dimensions")
            .render(Html::escape("Open"))
            .into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(markup.contains(r#"type="button""#), "{markup}");
        assert!(markup.contains(r#"aria-haspopup="dialog""#), "{markup}");
        // Served closed; the script's `toggle` handler moves it.
        assert!(markup.contains(r#"aria-expanded="false""#), "{markup}");
        assert!(markup.ends_with(">Open</button>"), "{markup}");
    }

    /// A trigger with no name is a bare button — no `popovertarget` pointing at
    /// an id that is not in the document, and no anchor name for nothing.
    #[test]
    fn an_unnamed_part_makes_no_association_at_all() {
        let trigger = popover_trigger().render(Html::empty()).into_test_string();

        assert!(!trigger.contains("popovertarget"), "{trigger}");
        assert!(!trigger.contains("aria-expanded"), "{trigger}");
        assert!(!trigger.contains("anchor-name"), "{trigger}");

        let panel = popover_content().render(Html::empty()).into_test_string();

        assert!(!panel.contains(" id="), "{panel}");
        assert!(!panel.contains("position-anchor"), "{panel}");
        // The rest of the panel is still a panel.
        assert!(panel.contains(r#"popover="auto""#), "{panel}");
    }

    /// The placement is said twice: once as CSS the browser can act on with no
    /// script, and once as the `data-*` the script reads when it cannot.
    #[test]
    fn side_and_align_are_written_twice() {
        let panel = popover_content()
            .name("edit")
            .side(PopoverSide::Right)
            .align(PopoverAlign::Start)
            .render(Html::empty())
            .into_test_string();

        assert!(panel.contains(r#"data-side="right""#), "{panel}");
        assert!(panel.contains(r#"data-align="start""#), "{panel}");
        assert!(panel.contains("position-anchor:--popover-edit"), "{panel}");
        assert!(
            panel.contains("position-area:inline-end span-block-end"),
            "{panel}"
        );
        // The offset is on one axis only. All four margins would also push a
        // `start`-aligned panel 4px off the edge it lines up with.
        assert!(panel.contains("margin-inline:4px"), "{panel}");
        assert!(!panel.contains("margin:4px"), "{panel}");
        assert_eq!(PopoverSide::Right.offset_axis(), "margin-inline");
        assert_eq!(PopoverSide::Bottom.offset_axis(), "margin-block");
        assert!(
            panel.contains("position-try-fallbacks:flip-block,flip-inline"),
            "{panel}"
        );

        let trigger = popover_trigger()
            .name("edit")
            .render(Html::empty())
            .into_test_string();

        assert_eq!(anchor_name("edit"), "--popover-edit");
        assert!(trigger.contains("anchor-name:--popover-edit"), "{trigger}");

        assert_eq!(PopoverSide::parse(""), PopoverSide::Bottom);
        assert_eq!(PopoverSide::parse("top"), PopoverSide::Top);
        assert_eq!(PopoverAlign::parse(""), PopoverAlign::Center);
        assert_eq!(PopoverAlign::parse("end"), PopoverAlign::End);
    }

    /// An anchor is the trigger's `anchor-name` on some other element, so the
    /// panel opens beside that instead.
    #[test]
    fn an_anchor_carries_the_same_anchor_name_as_the_trigger() {
        let anchor = popover_anchor()
            .name("row")
            .render(Html::escape("Row"))
            .into_test_string();

        assert!(anchor.contains(r#"data-slot="popover-anchor""#), "{anchor}");
        assert!(anchor.contains(r#"data-popover="row""#), "{anchor}");
        assert!(anchor.contains("anchor-name:--popover-row"), "{anchor}");
        // Nothing else: it is a box the panel is measured against.
        assert!(!anchor.contains("popovertarget"), "{anchor}");
        assert!(!anchor.contains("class="), "{anchor}");
    }

    /// The three parts shadcn added to the registry, and the strings are its own.
    #[test]
    fn the_header_parts_are_shadcns_three_strings() {
        let header = popover_header().render(Html::empty()).into_test_string();
        let title = popover_title().render(Html::empty()).into_test_string();
        let description = popover_description()
            .render(Html::empty())
            .into_test_string();

        assert!(
            header.contains(r#"class="flex flex-col gap-1 text-sm""#),
            "{header}"
        );
        assert!(title.contains(r#"class="font-medium""#), "{title}");
        assert!(
            description.contains(r#"class="text-muted-foreground""#),
            "{description}"
        );
        // shadcn's own elements: a div for the title, a p for the description.
        assert!(title.starts_with("<div "), "{title}");
        assert!(description.starts_with("<p "), "{description}");
    }

    /// The deltas from shadcn's Radix version, as assertions rather than prose.
    /// Each one is a decision the module docs argue for.
    #[test]
    fn the_port_drops_what_only_floating_ui_could_publish() {
        let list = popover_content_variants("");

        // 1. Radix's transform origin is a custom property nothing here writes.
        assert!(!list.contains("--radix-"), "{list}");
        // 2. A popover is `display: none` the instant it closes, so an exit
        //    animation would never be seen.
        assert!(!list.contains("animate-out"), "{list}");
        assert!(!list.contains("data-[state=closed]"), "{list}");
        // 3. The enter animation crosses with the prefix rewritten.
        assert!(list.contains("open:animate-in"), "{list}");
        assert!(list.contains("open:zoom-in-95"), "{list}");
        // 4. And the UA's `inset: 0` is cleared, or it fights every placement.
        assert!(list.contains("inset-auto"), "{list}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = popover_content()
            .name("dimensions")
            .class("w-80")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("w-80"), "{markup}");
    }

    /// A trigger's class list is the caller's alone — shadcn's has none, because
    /// its `asChild` was always meant to hand the look to a Button.
    #[test]
    fn a_trigger_wears_only_what_the_caller_gave_it() {
        let markup = popover_trigger()
            .name("dimensions")
            .class("inline-flex h-9 rounded-md border")
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"class="inline-flex h-9 rounded-md border""#),
            "{markup}"
        );
        assert!(!markup.contains("twMerge"), "{markup}");
    }

    /// A part with nothing to resolve serves a plain class list and needs no
    /// scope. The root always needs one — it *is* its script.
    #[test]
    fn only_an_override_or_a_binding_needs_the_runtime() {
        let props = popover_content().name("dimensions");

        assert!(!props.needs_scope());
        assert!(
            !props
                .render(Html::empty())
                .into_test_string()
                .contains("twMerge")
        );

        assert!(!popover_content().class("w-80").needs_scope());
        assert!(
            popover_content()
                .on(
                    "toggle",
                    crate::components::rahti_ui::attrs::ui_handler!({ seen() })
                )
                .needs_scope()
        );
        assert!(popover().needs_scope());
        assert!(!popover_title().needs_scope());
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = popover_content()
            .name("dimensions")
            .on(
                "toggle",
                crate::components::rahti_ui::attrs::ui_handler!({ seen(event) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"pp-ontoggle="seen(event)""#), "{markup}");
    }

    /// The caller's `attrs` are laid over the part's own, last-wins, and `unset`
    /// takes one away entirely.
    #[test]
    fn attrs_are_laid_over_the_parts_own() {
        let markup = popover_content()
            .name("dimensions")
            .attrs(
                Attrs::new()
                    .set("aria-label", "Dimensions")
                    .unset("tabindex"),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"aria-label="Dimensions""#), "{markup}");
        assert!(!markup.contains("tabindex"), "{markup}");
    }

    /// The name is a document id *and* a CSS anchor name, so it has to be
    /// spellable as both.
    #[test]
    fn a_name_has_to_be_a_css_identifier_too() {
        assert!(nameable("dimensions"));
        assert!(nameable("row-2"));
        assert!(nameable("_edit"));

        assert!(!nameable(""));
        assert!(!nameable("2rows"));
        assert!(!nameable("my popover"));
        assert!(!nameable("row.2"));
    }
}

#[cfg(test)]
mod audit {
    use super::{CONTENT, DESCRIPTION, HEADER, POPOVER_RESET, TITLE};
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered.
    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("CONTENT", cn(&[POPOVER_RESET, CONTENT])),
            ("HEADER", HEADER.to_string()),
            ("TITLE", TITLE.to_string()),
            ("DESCRIPTION", DESCRIPTION.to_string()),
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
            ("HEADER", HEADER),
            ("TITLE", TITLE),
            ("DESCRIPTION", DESCRIPTION),
        ] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
