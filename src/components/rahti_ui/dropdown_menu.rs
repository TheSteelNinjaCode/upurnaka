// rahti-ui: dropdown-menu
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Dropdown Menu — a port of shadcn/ui's `dropdown-menu`.
//!
//! shadcn's is Radix's DropdownMenu: a portal, a focus scope, a dismiss layer, a
//! roving tabindex, and Floating UI underneath doing collision detection. This
//! one is the **HTML popover API**, which is the platform's answer to the first
//! three of those and arrived long after Radix was written.
//!
//! ## What `popover` gives away for free
//!
//! ```text
//! Radix                                  here
//! ────────────────────────────────────   ─────────────────────────────────────
//! Portal — escape the stacking context   the top layer
//! DismissableLayer — click outside       light dismiss, built in
//! Escape to close                        built in
//! open state + onOpenChange              the `popovertarget` attribute
//! ```
//!
//! The last row is the one worth stopping on. A `<button popovertarget="…">`
//! opens and closes a popover **with no JavaScript at all** — it is a
//! declarative association in the markup, the way `<label for>` is. So the menu
//! in this component opens, closes, dismisses on an outside click and dismisses
//! on `Escape` before the runtime has loaded, and the script is left with two
//! jobs: place it, and add the arrow keys.
//!
//! Escaping `overflow: hidden` is not a nicety here. The menu this component
//! exists for lives inside a [`Sidebar`](super::sidebar), whose content pane
//! clips — a menu that is not in the top layer is a menu with its bottom half
//! cut off.
//!
//! ## The trigger and the menu are paired by name
//!
//! `popovertarget` takes an **id**, and the two halves are separate components
//! whose children are opaque to each other. So the pairing is a name the page
//! writes on both, which is the same divergence
//! [`Accordion`](super::accordion)'s `name` and [`Tabs`](super::tabs)' `value`
//! made, and for the same reason:
//!
//! ```ignore
//! <DropdownMenu>
//!     <DropdownMenuTrigger menu="user">"Open"</DropdownMenuTrigger>
//!     <DropdownMenuContent menu="user">…</DropdownMenuContent>
//! </DropdownMenu>
//! ```
//!
//! The name becomes `id="dropdown-menu-user"` on the menu and
//! `popovertarget="dropdown-menu-user"` on the trigger. It has to be unique in
//! the document and a valid CSS identifier, because it is also an anchor name —
//! see below. A [`debug_assert`] catches the spellings that are not.
//!
//! ## Position, twice over
//!
//! A popover is in the top layer, so it has no ancestor to be positioned
//! against. There are two ways to place one and this does both:
//!
//! 1. **CSS anchor positioning**, as inline styles: `anchor-name` on the trigger
//!    and `position-anchor` plus `position-area` on the menu. Where the browser
//!    has it, the menu is placed correctly *with no JavaScript*, and
//!    `position-try-fallbacks` flips it when it would go off screen. It is
//!    written as an inline style rather than a class on purpose: the name varies
//!    per menu, and Tailwind can only emit classes it finds spelled out in the
//!    source — the same wall [`Tabs`](super::tabs) hit.
//! 2. **The script**, for the browsers that do not have it yet. It measures on
//!    `beforetoggle` and writes `left`/`top`, with the same flip.
//!
//! The script asks `CSS.supports` and does nothing when the first one is
//! already handling it.
//!
//! ## Which shadcn dropdown menu this is
//!
//! The registry, and every class string is shadcn's with two changes:
//!
//! - `data-[state=open]:` becomes `open:`, which is Tailwind's variant for
//!   `[open]`, `:open` **and** `:popover-open`. The same substitution
//!   [`Collapsible`](super::collapsible) made, and it works here for free.
//! - The four `data-[state=closed]:` exit utilities are dropped. A popover is
//!   `display: none` the instant it closes, so a keyframe exit would never be
//!   seen — the same wall [`Sheet`](super::sheet) hit, which paid a script to
//!   get around it. A menu is not worth that, and shadcn's enter animation
//!   survives untouched.
//!
//! Two Radix custom properties go with them:
//! `max-h-(--radix-dropdown-menu-content-available-height)` and
//! `origin-(--radix-dropdown-menu-content-transform-origin)` are published by
//! Floating UI and by nothing here. [`AVAILABLE_HEIGHT`] is what replaces the
//! first; the second is a transform origin for the zoom, and `center` is what it
//! falls back to.
//!
//! ## The parts
//!
//! ```html
//! <div data-slot="dropdown-menu" class="contents">
//!   <button data-slot="dropdown-menu-trigger" popovertarget="dropdown-menu-user"
//!           style="anchor-name:--dropdown-menu-user">…</button>
//!   <div data-slot="dropdown-menu-content" id="dropdown-menu-user" popover="auto"
//!        role="menu" data-side="bottom" style="position-anchor:--dropdown-menu-user;…">
//!     <div data-slot="dropdown-menu-label">…</div>
//!     <div data-slot="dropdown-menu-separator"></div>
//!     <a data-slot="dropdown-menu-item" role="menuitem" href="/account">…</a>
//!   </div>
//!   <span data-slot="dropdown-menu-anchor" hidden></span>
//!   <script>…</script>
//! </div>
//! ```
//!
//! ## An item is a link when you give it one
//!
//! shadcn's item is a `<div role="menuitem">` that Radix makes activatable with
//! JavaScript. Here an item with an [`href`](DropdownMenuProps::href) renders as
//! an `<a role="menuitem">`, which the keyboard and the browser already know how
//! to activate — so `Account`, `Billing` and `Log out` work with the runtime
//! absent. Without an `href` it is shadcn's `<div>`, and the script supplies
//! `Enter` and `Space`.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_icons::check::Check;
use crate::components::rahti_icons::chevron_right::ChevronRight;
use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, Json, component, html};

/// The root: `display: contents`, so the trigger inside it lays out where it was
/// written.
///
/// Radix's Root renders nothing at all. This has to render something — the
/// script lives in it and searches from it — so it renders a box and takes the
/// box back out of the layout. Same arrangement, and same reason, as
/// [`Sheet`](super::sheet)'s root.
pub const ROOT: &str = "contents";

/// The menu. shadcn's string, less the two Radix custom properties and the exit
/// animation.
///
/// `open:` is `data-[state=open]:` — Tailwind's `open:` variant matches
/// `:popover-open`, so shadcn's enter animation crosses with the prefix
/// rewritten and nothing else touched.
///
/// `text-left` is an addition, and it is a consequence of not portalling. Radix
/// renders its content into a portal at the end of `<body>`, so the menu
/// inherits nothing from wherever the trigger happens to sit. A popover stays
/// where it was written, so a menu inside a `<td class="text-right">` — which is
/// exactly where a table's row-actions menu lives — inherited `text-align:
/// right` and drew its label against the wrong edge. Measured in the browser on
/// both the Table and Data Table pages before this was added. Physical rather
/// than logical, matching the rest of this library.
pub const CONTENT: &str = "z-50 min-w-[8rem] overflow-x-hidden overflow-y-auto rounded-md \
                           border bg-popover p-1 text-left text-popover-foreground shadow-md \
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
///
/// [`AVAILABLE_HEIGHT`] rides along here because it answers the same question
/// Radix answered with a custom property: how tall may this get before it runs
/// off the screen.
pub const POPOVER_RESET: &str = "inset-auto";

/// Radix publishes an available height from Floating UI; this asks the browser.
///
/// `max-h-[…]` with a `dvh` unit is the same answer arrived at without a
/// measurement — the menu may grow to the viewport less a little breathing room,
/// and `overflow-y-auto` in [`CONTENT`] scrolls it after that.
pub const AVAILABLE_HEIGHT: &str = "max-h-[calc(100dvh-2rem)]";

/// A submenu. shadcn's, with the same changes as [`CONTENT`] — the `text-left`
/// included, for the same inheritance reason.
pub const SUB_CONTENT: &str = "z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover \
                               p-1 text-left text-popover-foreground shadow-lg \
                               data-[side=bottom]:slide-in-from-top-2 \
                               data-[side=left]:slide-in-from-right-2 \
                               data-[side=right]:slide-in-from-left-2 \
                               data-[side=top]:slide-in-from-bottom-2 \
                               open:animate-in open:fade-in-0 open:zoom-in-95";

/// One row. shadcn's string, unchanged.
pub const ITEM: &str = "relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 \
                        text-sm outline-hidden select-none focus:bg-accent \
                        focus:text-accent-foreground data-[disabled]:pointer-events-none \
                        data-[disabled]:opacity-50 data-[inset]:pl-8 \
                        data-[variant=destructive]:text-destructive \
                        data-[variant=destructive]:focus:bg-destructive/10 \
                        data-[variant=destructive]:focus:text-destructive \
                        dark:data-[variant=destructive]:focus:bg-destructive/20 \
                        [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                        [&_svg:not([class*='size-'])]:size-4 \
                        [&_svg:not([class*='text-'])]:text-muted-foreground \
                        data-[variant=destructive]:*:[svg]:text-destructive!";

/// A row with room for a tick or a dot on its left. shadcn's, unchanged.
pub const CHOICE_ITEM: &str = "relative flex cursor-default items-center gap-2 rounded-sm py-1.5 \
                               pr-2 pl-8 text-sm outline-hidden select-none focus:bg-accent \
                               focus:text-accent-foreground data-[disabled]:pointer-events-none \
                               data-[disabled]:opacity-50 [&_svg]:pointer-events-none \
                               [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

/// The box the tick or dot sits in. shadcn's, unchanged.
pub const INDICATOR: &str =
    "pointer-events-none absolute left-2 flex size-3.5 items-center justify-center";

/// A heading over a run of rows. shadcn's, unchanged.
pub const LABEL: &str = "px-2 py-1.5 text-sm font-medium data-[inset]:pl-8";

/// A rule between runs. shadcn's, unchanged.
pub const SEPARATOR: &str = "-mx-1 my-1 h-px bg-border";

/// The key hint on the right of a row. shadcn's, unchanged.
pub const SHORTCUT: &str = "ml-auto text-xs tracking-widest text-muted-foreground";

/// A row that opens a submenu. shadcn's, unchanged.
/// shadcn's string plus the two utilities a `<button>` costs.
///
/// This row is a `<button>` where shadcn's is a `<div>`, and it has to be:
/// `popovertarget` is only an attribute of `<button>` and the button-ish
/// `<input>`s, and that attribute is the whole of this component's open and
/// close. On a `<div>` it is inert — `popoverTargetElement` reads `null` and the
/// submenu never opens at all.
///
/// What the element change costs, both measured in the browser rather than
/// guessed:
///
/// - **`w-full`.** A `<button>` is shrink-to-fit even at `display: flex`, so the
///   row came out 120px wide beside 215px siblings. That is visible twice: the
///   `ml-auto` chevron stops at the end of the text instead of the end of the
///   row, and — because `anchor-name` is on *this* element — the submenu anchors
///   to the short box and opens *inside* the parent menu rather than beside it.
/// - **`text-left`.** A `<button>` inherits `text-align: center` from the UA
///   stylesheet. Invisible on one line and wrong the moment a label wraps.
///
/// One more utility is not shadcn's *spelling* but is shadcn's *behaviour*.
/// Radix marks this row `data-[state=open]` while its submenu is open, so the
/// row stays highlighted; a port with no JavaScript has no such attribute, and
/// the obvious `open:bg-accent` is **dead** — `open:` compiles to
/// `:is([open], :popover-open, :open)`, and the trigger is never any of those.
/// The popover is its next sibling, so the working spelling reaches across:
///
/// ```text
/// [&:has(+[popover]:popover-open)]:bg-accent
/// ```
///
/// Measured: the trigger does not match `:popover-open`, and it does match
/// `:has(+ [popover]:popover-open)`. A popover in the top layer is still in the
/// DOM where its siblings can see it — only its painting moves.
///
/// Everything else is shadcn's, unchanged. Same shape of fix as
/// [`Separator`](super::separator)'s `HR_RESET` and [`Sheet`](super::sheet)'s
/// `DIALOG_RESET`: the cost of using the element the platform requires.
pub const SUB_TRIGGER: &str = "flex w-full cursor-default items-center gap-2 rounded-sm px-2 \
                               py-1.5 text-left text-sm outline-hidden select-none \
                               focus:bg-accent focus:text-accent-foreground data-[inset]:pl-8 \
                               [&:has(+[popover]:popover-open)]:bg-accent \
                               [&:has(+[popover]:popover-open)]:text-accent-foreground \
                               [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                               [&_svg:not([class*='size-'])]:size-4 \
                               [&_svg:not([class*='text-'])]:text-muted-foreground";

/// The chevron shadcn puts on a submenu row.
pub const SUB_TRIGGER_ICON: &str = "ml-auto size-4";

/// How far the menu sits from its trigger — shadcn's `sideOffset = 4`.
pub const SIDE_OFFSET: u32 = 4;

/// How long an open submenu is given before the pointer moving to another row
/// closes it.
///
/// Radix calls this the grace period, and without one a submenu is very hard to
/// reach: the diagonal from a sub-trigger to the menu beside it clips the row
/// *below* the trigger on the way, and closing on that row would shut the
/// submenu just as the pointer sets off for it. Entering any menu cancels the
/// pending close, so the only thing that survives the delay is a pointer that
/// really did go somewhere else.
pub const SUBMENU_GRACE_MS: u32 = 150;

/// Which side of the trigger the menu opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl MenuSide {
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
    /// The span keeps the menu's near edge lined up with the trigger's rather
    /// than centring it, which is what `align="start"` means in Radix.
    pub const fn area(self, align: MenuAlign) -> &'static str {
        match (self, align) {
            (Self::Bottom, MenuAlign::Start) => "block-end span-inline-end",
            (Self::Bottom, MenuAlign::Center) => "block-end center",
            (Self::Bottom, MenuAlign::End) => "block-end span-inline-start",
            (Self::Top, MenuAlign::Start) => "block-start span-inline-end",
            (Self::Top, MenuAlign::Center) => "block-start center",
            (Self::Top, MenuAlign::End) => "block-start span-inline-start",
            (Self::Left, MenuAlign::Start) => "inline-start span-block-end",
            (Self::Left, MenuAlign::Center) => "inline-start center",
            (Self::Left, MenuAlign::End) => "inline-start span-block-start",
            (Self::Right, MenuAlign::Start) => "inline-end span-block-end",
            (Self::Right, MenuAlign::Center) => "inline-end center",
            (Self::Right, MenuAlign::End) => "inline-end span-block-start",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `side="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "bottom" => Self::Bottom,
            "top" => Self::Top,
            "left" => Self::Left,
            "right" => Self::Right,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a DropdownMenu side. \
                     One of: top, bottom, left, right."
                );
                Self::Bottom
            }
        }
    }
}

/// Which of the trigger's edges the menu lines up with — Radix's `align`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuAlign {
    #[default]
    Start,
    Center,
    End,
}

impl MenuAlign {
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
            "" | "start" => Self::Start,
            "center" => Self::Center,
            "end" => Self::End,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a DropdownMenu alignment. \
                     One of: start, center, end."
                );
                Self::Start
            }
        }
    }
}

/// A row's tone — shadcn's `variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    #[default]
    Default,
    Destructive,
}

impl ItemVariant {
    /// The name written into `data-variant`, which [`ITEM`]'s destructive
    /// utilities read.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `variant="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "destructive" => Self::Destructive,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a DropdownMenuItem variant. \
                     One of: default, destructive."
                );
                Self::Default
            }
        }
    }
}

/// The document id a menu's name becomes.
pub fn menu_id(name: &str) -> String {
    format!("dropdown-menu-{}", name.trim())
}

/// The CSS anchor name a menu's name becomes.
pub fn anchor_name(name: &str) -> String {
    format!("--dropdown-menu-{}", name.trim())
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

/// Which of the fourteen a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Trigger,
    Content,
    Group,
    Item,
    CheckboxItem,
    RadioGroup,
    RadioItem,
    Label,
    Separator,
    Shortcut,
    Sub,
    SubTrigger,
    SubContent,
}

impl Part {
    /// The `data-slot` this part carries — shadcn's, unchanged.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "dropdown-menu",
            Self::Trigger => "dropdown-menu-trigger",
            Self::Content => "dropdown-menu-content",
            Self::Group => "dropdown-menu-group",
            Self::Item => "dropdown-menu-item",
            Self::CheckboxItem => "dropdown-menu-checkbox-item",
            Self::RadioGroup => "dropdown-menu-radio-group",
            Self::RadioItem => "dropdown-menu-radio-item",
            Self::Label => "dropdown-menu-label",
            Self::Separator => "dropdown-menu-separator",
            Self::Shortcut => "dropdown-menu-shortcut",
            Self::Sub => "dropdown-menu-sub",
            Self::SubTrigger => "dropdown-menu-sub-trigger",
            Self::SubContent => "dropdown-menu-sub-content",
        }
    }

    /// The ARIA role this part carries, where it has one.
    const fn role(self) -> Option<&'static str> {
        match self {
            Self::Content | Self::SubContent => Some("menu"),
            Self::Group => Some("group"),
            Self::Item | Self::SubTrigger => Some("menuitem"),
            Self::CheckboxItem => Some("menuitemcheckbox"),
            Self::RadioGroup => Some("group"),
            Self::RadioItem => Some("menuitemradio"),
            Self::Separator => Some("separator"),
            _ => None,
        }
    }

    /// Is this part a row the keyboard walks?
    const fn is_row(self) -> bool {
        matches!(
            self,
            Self::Item | Self::CheckboxItem | Self::RadioItem | Self::SubTrigger
        )
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part) -> String {
    match part {
        Part::Root | Part::Sub => ROOT.to_string(),
        Part::Content => cn(&[POPOVER_RESET, AVAILABLE_HEIGHT, CONTENT]),
        Part::SubContent => cn(&[POPOVER_RESET, SUB_CONTENT]),
        Part::Item => ITEM.to_string(),
        Part::CheckboxItem | Part::RadioItem => CHOICE_ITEM.to_string(),
        Part::Label => LABEL.to_string(),
        Part::Separator => SEPARATOR.to_string(),
        Part::Shortcut => SHORTCUT.to_string(),
        Part::SubTrigger => SUB_TRIGGER.to_string(),
        Part::Group | Part::RadioGroup | Part::Trigger => String::new(),
    }
}

/// The class list shadcn's `cn("z-50 min-w-[8rem] …", className)` returns.
///
/// Write `data-side` alongside it — the four slide utilities read it — and give
/// the element `popover` and an `id`, because the open and close are the
/// `popovertarget` association rather than anything in the class list.
pub fn dropdown_menu_content_variants(class: &str) -> String {
    cn(&[POPOVER_RESET, AVAILABLE_HEIGHT, CONTENT, class])
}

/// [`DropdownMenuItem`]'s class list, the caller's own included.
pub fn dropdown_menu_item_variants(class: &str) -> String {
    cn(&[ITEM, class])
}

/// [`DropdownMenuCheckboxItem`]'s and [`DropdownMenuRadioItem`]'s class list.
pub fn dropdown_menu_choice_item_variants(class: &str) -> String {
    cn(&[CHOICE_ITEM, class])
}

/// [`DropdownMenuLabel`]'s class list, the caller's own included.
pub fn dropdown_menu_label_variants(class: &str) -> String {
    cn(&[LABEL, class])
}

/// [`DropdownMenuSeparator`]'s class list, the caller's own included.
pub fn dropdown_menu_separator_variants(class: &str) -> String {
    cn(&[SEPARATOR, class])
}

/// [`DropdownMenuShortcut`]'s class list, the caller's own included.
pub fn dropdown_menu_shortcut_variants(class: &str) -> String {
    cn(&[SHORTCUT, class])
}

/// [`DropdownMenuSubTrigger`]'s class list, the caller's own included.
pub fn dropdown_menu_sub_trigger_variants(class: &str) -> String {
    cn(&[SUB_TRIGGER, class])
}

/// [`DropdownMenuSubContent`]'s class list, the caller's own included.
pub fn dropdown_menu_sub_content_variants(class: &str) -> String {
    cn(&[POPOVER_RESET, SUB_CONTENT, class])
}

/// One Dropdown Menu part's props, with shadcn's defaults already in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownMenuProps {
    part: Part,
    menu: String,
    side: MenuSide,
    align: MenuAlign,
    variant: ItemVariant,
    inset: bool,
    disabled: bool,
    checked: bool,
    href: String,
    class: String,
    attrs: Attrs,
}

impl Default for DropdownMenuProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            menu: String::new(),
            side: MenuSide::Bottom,
            align: MenuAlign::Start,
            variant: ItemVariant::Default,
            inset: false,
            disabled: false,
            checked: false,
            href: String::new(),
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

macro_rules! builder {
    ($($name:ident => $part:expr, $doc:literal;)*) => {
        $(
            #[doc = $doc]
            pub fn $name() -> DropdownMenuProps {
                DropdownMenuProps { part: $part, ..DropdownMenuProps::default() }
            }
        )*
    };
}

builder! {
    dropdown_menu => Part::Root, "A default [`DropdownMenu`]'s props.";
    dropdown_menu_trigger => Part::Trigger, "A default [`DropdownMenuTrigger`]'s props.";
    dropdown_menu_content => Part::Content, "A default [`DropdownMenuContent`]'s props.";
    dropdown_menu_group => Part::Group, "A default [`DropdownMenuGroup`]'s props.";
    dropdown_menu_item => Part::Item, "A default [`DropdownMenuItem`]'s props.";
    dropdown_menu_checkbox_item => Part::CheckboxItem, "A default [`DropdownMenuCheckboxItem`]'s props.";
    dropdown_menu_radio_group => Part::RadioGroup, "A default [`DropdownMenuRadioGroup`]'s props.";
    dropdown_menu_radio_item => Part::RadioItem, "A default [`DropdownMenuRadioItem`]'s props.";
    dropdown_menu_label => Part::Label, "A default [`DropdownMenuLabel`]'s props.";
    dropdown_menu_separator => Part::Separator, "A default [`DropdownMenuSeparator`]'s props.";
    dropdown_menu_shortcut => Part::Shortcut, "A default [`DropdownMenuShortcut`]'s props.";
    dropdown_menu_sub => Part::Sub, "A default [`DropdownMenuSub`]'s props.";
    dropdown_menu_sub_trigger => Part::SubTrigger, "A default [`DropdownMenuSubTrigger`]'s props.";
    dropdown_menu_sub_content => Part::SubContent, "A default [`DropdownMenuSubContent`]'s props.";
}

impl DropdownMenuProps {
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

    /// The name pairing a trigger with its menu.
    ///
    /// Becomes `popovertarget` on the trigger and `id` on the menu, and an
    /// `anchor-name` on both. It has to be unique in the document and a valid
    /// CSS identifier — letters, digits, `-` and `_`, not starting with a digit.
    pub fn menu(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(
            name.trim().is_empty() || nameable(&name),
            "rahti-ui: `{name}` cannot be a DropdownMenu name — it is also a CSS \
             anchor name. Use letters, digits, `-` and `_`, not starting with a digit."
        );
        self.menu = name;
        self
    }

    /// Which side of the trigger the menu opens on.
    pub fn side(mut self, side: MenuSide) -> Self {
        self.side = side;
        self
    }

    /// Which of the trigger's edges the menu lines up with.
    pub fn align(mut self, align: MenuAlign) -> Self {
        self.align = align;
        self
    }

    /// A row's tone.
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Indent this row to line up with the ones that have a tick — shadcn's
    /// `inset`, written as `data-inset`.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// A row that cannot be chosen, written as `data-disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A checkbox or radio row's state, written as `aria-checked`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Make this row a link.
    ///
    /// An item with an `href` renders as `<a role="menuitem">`, which the
    /// browser already knows how to focus and activate — so the row works with
    /// the runtime absent. Without one it is shadcn's `<div>` and the script
    /// supplies `Enter` and `Space`.
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = href.into();
        self
    }

    /// The part's own id.
    ///
    /// Not for a menu: that id is the [`menu`](Self::menu) name, because
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
    /// `popover`, `popovertarget` and the menu's `id` are the association that
    /// makes the component work without a script — bind any of them and it stops
    /// working before the runtime arrives rather than after.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression.
    ///
    /// A handler on a row runs beside the script's own rather than instead of
    /// it. `onclick` on an item is the usual one, and the menu still closes.
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

    /// The inline style that anchors a menu to its trigger, or nothing.
    ///
    /// Written as a style rather than a class because the anchor name varies per
    /// menu, and Tailwind can only emit classes it finds spelled out in the
    /// source. Where the browser has anchor positioning this places the menu
    /// with no JavaScript at all; where it does not, every declaration is
    /// ignored and the script does it instead.
    fn position_style(&self) -> Option<String> {
        if self.menu.trim().is_empty() {
            return None;
        }

        match self.part {
            Part::Trigger | Part::SubTrigger => {
                Some(format!("anchor-name:{}", anchor_name(&self.menu)))
            }
            Part::Content | Part::SubContent => Some(format!(
                "position-anchor:{};position-area:{};margin:{}px;\
                 position-try-fallbacks:flip-block,flip-inline",
                anchor_name(&self.menu),
                self.side.area(self.align),
                SIDE_OFFSET,
            )),
            _ => None,
        }
    }

    /// The open tag's attributes, in shadcn's order.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());

        if let Some(role) = self.part.role() {
            base = base.set("role", role);
        }

        base = match self.part {
            Part::Trigger | Part::SubTrigger => {
                // Both are `<button type="button">`, and for a submenu that is
                // not a preference — `popovertarget` is **only** an attribute of
                // `<button>` and the button-ish `<input>`s. On the `<div>` a
                // shadcn SubTrigger renders it is inert: `popoverTargetElement`
                // reads `null`, clicking does nothing, and the submenu never
                // opens. Measured in the browser; see [`SUB_TRIGGER`].
                let mut trigger = base.set("type", "button");
                if !self.menu.trim().is_empty() {
                    // The whole of the open and close, and not a line of it is
                    // JavaScript.
                    trigger = trigger
                        .set("popovertarget", menu_id(&self.menu))
                        .set("aria-haspopup", "menu")
                        // Served closed and moved by the script's `toggle`
                        // handler, as the Popover's is. The platform has no
                        // declarative spelling for it, and a menu button that
                        // never announces itself as expanded is the one thing
                        // the `popovertarget` trade would otherwise cost.
                        .set("aria-expanded", "false");

                    // A submenu also opens on hover, so the default *toggle*
                    // action would work against itself: the pointer opens the
                    // menu on the way to the click, and the click then closes
                    // what it was aiming at. `show` makes a click idempotent and
                    // leaves closing to Escape, a light dismiss, or the pointer
                    // going elsewhere.
                    if matches!(self.part, Part::SubTrigger) {
                        trigger = trigger.set("popovertargetaction", "show");
                    }
                }
                if self.inset {
                    trigger = trigger.set("data-inset", "");
                }
                trigger
            }

            Part::Content | Part::SubContent => {
                let mut content = base
                    .set("popover", "auto")
                    .set("data-side", self.side.slug())
                    .set("data-align", self.align.slug());
                if !self.menu.trim().is_empty() {
                    content = content.set("id", menu_id(&self.menu));
                }
                content
            }

            Part::Item | Part::CheckboxItem | Part::RadioItem => {
                let mut item = base
                    // A row is reached with the arrow keys rather than with Tab,
                    // which is the ARIA menu pattern — one tab stop for the menu
                    // and a roving focus inside it.
                    .set("tabindex", "-1");
                if matches!(self.part, Part::Item) {
                    item = item.set("data-variant", self.variant.slug());
                }
                if matches!(self.part, Part::CheckboxItem | Part::RadioItem) {
                    item = item.set(
                        "aria-checked",
                        match self.checked {
                            true => "true",
                            false => "false",
                        },
                    );
                }
                if self.inset {
                    item = item.set("data-inset", "");
                }
                if self.disabled {
                    item = item.set("data-disabled", "").set("aria-disabled", "true");
                }
                if !self.href.trim().is_empty() {
                    item = item.set("href", self.href.trim());
                }
                item
            }

            // shadcn's Label takes `inset` too, and `data-[inset]:pl-8` in
            // `LABEL` is what reads it — a heading over a run of ticked rows
            // lines up with their text rather than with their ticks.
            Part::Label if self.inset => base.set("data-inset", ""),

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
            "rahti-ui: a DropdownMenu root is its script. Use the <DropdownMenu> tag."
        );

        let attrs = self.attr_list();

        match self.part {
            Part::Trigger | Part::SubTrigger => html! {
                <button @{..attrs}>@{children}</button>
            },
            Part::Shortcut => html! {
                <span @{..attrs}>@{children}</span>
            },
            Part::Item if !self.href.trim().is_empty() => {
                html! {
                    <a @{..attrs}>@{children}</a>
                }
            }
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
fn build(
    mut props: DropdownMenuProps,
    id: &str,
    attrs: Attrs,
    bindings: Bindings,
) -> DropdownMenuProps {
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

/// A trigger and the menu it opens.
///
/// ```ignore
/// <DropdownMenu>
///     <DropdownMenuTrigger menu="user" class=@{button_variants(…)}>"Open"</DropdownMenuTrigger>
///     <DropdownMenuContent menu="user" class="w-56">
///         <DropdownMenuLabel>"My account"</DropdownMenuLabel>
///         <DropdownMenuSeparator />
///         <DropdownMenuItem href="/account">"Account"</DropdownMenuItem>
///     </DropdownMenuContent>
/// </DropdownMenu>
/// ```
///
/// `display: contents`, so the trigger inside it lays out where it was written.
/// The `menu` name has to be on both halves — `popovertarget` takes an id, and
/// the two components cannot see each other.
///
/// ## What is not the script
///
/// Opening, closing, dismissing on an outside click and dismissing on `Escape`
/// are the popover API's, and they work before the runtime has loaded. Where the
/// browser has CSS anchor positioning, so does the placement. The script is left
/// with the keyboard — arrow keys, `Home`, `End`, `Enter`, `Space`, and the
/// focus that moves into the menu when it opens and back to the trigger when it
/// closes — and with placing the menu in the browsers that cannot do it in CSS.
///
/// None of its state is `pp.state`, for the reason
/// [`ScrollArea`](super::scroll_area) gives: a re-render would hand the
/// attribute reconciler a chance to undo the inline `left`/`top` it writes.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenu(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(dropdown_menu().class(class), id, attrs, bindings);
    let root = props.attr_list();
    let offset = SIDE_OFFSET;
    let grace = SUBMENU_GRACE_MS;
    let rows = Json(
        "[data-slot=dropdown-menu-item],[data-slot=dropdown-menu-checkbox-item],\
         [data-slot=dropdown-menu-radio-item],[data-slot=dropdown-menu-sub-trigger]",
    );

    html! {
        <div @{..root}>
            <slot />
            <span data-slot="dropdown-menu-anchor" hidden=@{true} pp-ref={anchorRef}></span>
            <script>r##"
                const OFFSET = @{offset};
                const GRACE_MS = @{grace};
                const ROWS = @{rows};

                const anchorRef = pp.ref(null);
                // Only ever used to break an id collision — see below.
                const uid = pp.id();

                pp.effect(() => {
                    const anchor = anchorRef.current;
                    if (!anchor) return;

                    const root = anchor.closest("[data-slot=dropdown-menu]");
                    if (!root) return;

                    const own = (slot) =>
                        Array.from(root.querySelectorAll("[data-slot=" + slot + "]"))
                            .filter((el) => el.closest("[data-slot=dropdown-menu]") === root);

                    const menus = own("dropdown-menu-content").concat(
                        own("dropdown-menu-sub-content")
                    );
                    if (menus.length === 0) return;

                    // A component that renders its children twice — a Sidebar
                    // does, once for its mobile drawer — puts this menu's id in
                    // the document twice, and `popovertarget` resolves to
                    // whichever comes first. Whoever is not the first renames
                    // itself and its trigger, so the two stop fighting over the
                    // name. `pp.id()` is per component instance, so the two
                    // copies pick different ones.
                    for (const menu of menus) {
                        const id = menu.getAttribute("id");
                        if (!id) continue;
                        if (document.getElementById(id) === menu) continue;
                        const trigger = root.querySelector(
                            "[popovertarget=" + CSS.escape(id) + "]"
                        );
                        const unique = id + "-" + uid;
                        menu.setAttribute("id", unique);
                        if (trigger) trigger.setAttribute("popovertarget", unique);
                    }

                    // Where the browser has CSS anchor positioning the inline
                    // styles have already placed the menu, and this does nothing
                    // at all. Asked once rather than per open.
                    const anchored =
                        typeof CSS !== "undefined" &&
                        CSS.supports &&
                        CSS.supports("position-area", "block-end span-inline-end");

                    const triggerFor = (menu) => {
                        const id = menu.getAttribute("id");
                        if (!id) return null;
                        return root.querySelector("[popovertarget=" + CSS.escape(id) + "]");
                    };

                    const rowsIn = (menu) =>
                        Array.from(menu.querySelectorAll(ROWS)).filter(
                            (row) => !row.hasAttribute("data-disabled") && !row.hidden && row.getClientRects().length > 0
                        );

                    // The fallback placement: measure the trigger, put the menu
                    // beside it, and flip when it would run off the edge. This
                    // is the part CSS anchor positioning does declaratively.
                    const place = (menu) => {
                        const trigger = triggerFor(menu);
                        if (!trigger) return;

                        const at = trigger.getBoundingClientRect();
                        const box = menu.getBoundingClientRect();
                        const side = menu.getAttribute("data-side") || "bottom";
                        const align = menu.getAttribute("data-align") || "start";
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
                                    : align === "center"
                                      ? at.left + at.width / 2 - box.width / 2
                                      : at.left;
                        } else {
                            left = side === "right" ? at.right + OFFSET : at.left - box.width - OFFSET;
                            if (left + box.width > room.w - 4) left = at.left - box.width - OFFSET;
                            if (left < 4) left = at.right + OFFSET;
                            top =
                                align === "end"
                                    ? at.bottom - box.height
                                    : align === "center"
                                      ? at.top + at.height / 2 - box.height / 2
                                      : at.top;
                        }

                        menu.style.position = "fixed";
                        menu.style.left = Math.max(4, Math.min(left, room.w - box.width - 4)) + "px";
                        menu.style.top = Math.max(4, Math.min(top, room.h - box.height - 4)) + "px";
                    };

                    const focusRow = (menu, index) => {
                        const rows = rowsIn(menu);
                        if (rows.length === 0) return;
                        const at = ((index % rows.length) + rows.length) % rows.length;
                        rows[at].focus();
                    };

                    const toggled = (event) => {
                        const menu = event.currentTarget;
                        const trigger = triggerFor(menu);
                        if (trigger) trigger.setAttribute("aria-expanded", event.newState === "open" ? "true" : "false");
                        if (event.newState !== "open") {
                            // Give the trigger its focus back, which is what a
                            // dialog does for free and a popover does not.
                            if (trigger && menu.contains(document.activeElement)) trigger.focus();
                            return;
                        }
                        if (!anchored) place(menu);
                        focusRow(menu, 0);
                    };

                    const keys = (event) => {
                        const menu = event.currentTarget;
                        const rows = rowsIn(menu);
                        if (rows.length === 0) return;
                        const at = rows.indexOf(document.activeElement);

                        if (event.key === "ArrowDown") focusRow(menu, at + 1);
                        else if (event.key === "ArrowUp") focusRow(menu, at - 1);
                        else if (event.key === "Home") focusRow(menu, 0);
                        else if (event.key === "End") focusRow(menu, rows.length - 1);
                        else if (event.key === "Enter" || event.key === " ") {
                            const row = document.activeElement;
                            if (!row || rows.indexOf(row) < 0) return;
                            // An `<a>` activates itself; a `<div role=menuitem>`
                            // is why this branch exists at all.
                            if (row.tagName !== "A") {
                                event.preventDefault();
                                row.click();
                            }
                            return;
                        } else return;

                        event.preventDefault();
                    };

                    // A row that was chosen closes the menu it was in, which is
                    // what every menu does and what `popovertarget` cannot say
                    // from inside.
                    const chose = (event) => {
                        const row = event.target.closest(ROWS);
                        if (!row) return;
                        if (row.hasAttribute("data-disabled")) return;
                        if (row.matches("[data-slot=dropdown-menu-sub-trigger]")) return;
                        event.currentTarget.hidePopover();
                    };

                    // Which menu a row belongs to. A submenu's markup sits
                    // *inside* its parent menu, so `contains` is not the
                    // question — the nearest menu is.
                    const menuOf = (row) =>
                        row.closest(
                            "[data-slot=dropdown-menu-content],[data-slot=dropdown-menu-sub-content]"
                        );

                    const subFor = (trigger) => {
                        const id = trigger.getAttribute("popovertarget");
                        return id ? document.getElementById(id) : null;
                    };

                    // The submenus this menu owns, not the ones its submenus do.
                    const subsOf = (menu) =>
                        Array.from(
                            menu.querySelectorAll("[data-slot=dropdown-menu-sub-trigger]")
                        )
                            .filter((trigger) => menuOf(trigger) === menu)
                            .map(subFor)
                            .filter(Boolean);

                    const closeSubs = (menu, keep) => {
                        for (const sub of subsOf(menu)) {
                            if (sub !== keep && sub.matches(":popover-open")) sub.hidePopover();
                        }
                    };

                    let grace = 0;

                    // Radix highlights the row under the pointer by *focusing*
                    // it, which is why shadcn's class list says `focus:bg-accent`
                    // and never `hover:`. Following it means one row is
                    // highlighted rather than two — a hovered one and a focused
                    // one — and `Enter` acts on whatever the eye is on.
                    const pointed = (event) => {
                        const menu = event.currentTarget;
                        const row = event.target.closest(ROWS);
                        // `pointermove` bubbles, so a move inside a submenu
                        // reaches its parent as well. Only the menu the row
                        // actually belongs to acts on it.
                        if (!row || menuOf(row) !== menu) return;
                        if (row.hasAttribute("data-disabled")) return;

                        // `preventScroll`: the pointer is already where it wants
                        // to be, and a menu that scrolls itself out from under
                        // the cursor is unusable.
                        if (document.activeElement !== row) row.focus({ preventScroll: true });

                        clearTimeout(grace);
                        const wanted = row.matches("[data-slot=dropdown-menu-sub-trigger]")
                            ? subFor(row)
                            : null;

                        if (wanted) {
                            closeSubs(menu, wanted);
                            if (!wanted.matches(":popover-open")) wanted.showPopover();
                        } else {
                            // Delayed, not immediate — see `SUBMENU_GRACE_MS`.
                            grace = setTimeout(() => closeSubs(menu, null), GRACE_MS);
                        }
                    };

                    // Entering any menu cancels a pending close, which is what
                    // lets the diagonal to a submenu work even when it lands on
                    // the submenu's padding rather than on one of its rows.
                    const entered = () => clearTimeout(grace);

                    // And leaving one drops the highlight, so no row is left
                    // looking hovered after the pointer has gone. Moving into a
                    // submenu is not leaving: its markup is inside this menu, so
                    // `contains` is the right question here.
                    const left = (event) => {
                        const menu = event.currentTarget;
                        if (event.relatedTarget && menu.contains(event.relatedTarget)) return;
                        const row = document.activeElement;
                        if (row && menuOf(row) === menu) row.blur();
                    };

                    for (const menu of menus) {
                        menu.addEventListener("toggle", toggled);
                        menu.addEventListener("keydown", keys);
                        menu.addEventListener("click", chose);
                        menu.addEventListener("pointermove", pointed);
                        menu.addEventListener("pointerenter", entered);
                        menu.addEventListener("pointerleave", left);
                    }

                    return () => {
                        clearTimeout(grace);
                        for (const menu of menus) {
                            menu.removeEventListener("toggle", toggled);
                            menu.removeEventListener("keydown", keys);
                            menu.removeEventListener("click", chose);
                            menu.removeEventListener("pointermove", pointed);
                            menu.removeEventListener("pointerenter", entered);
                            menu.removeEventListener("pointerleave", left);
                        }
                    };
                }, []);
            "##</script>
        </div>
    }
}

/// The button that opens the menu.
///
/// shadcn's trigger has no class list of its own — its `asChild` was always
/// meant to hand the look to a Button — so `button_variants(…)` here is how that
/// crosses. `menu` has to match the [`DropdownMenuContent`] beside it.
///
/// The `popovertarget` it carries is the whole of the open and close, and none
/// of it is JavaScript.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuTrigger(
    class: &str,
    menu: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_trigger().class(class).menu(menu),
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

/// The menu — a `popover` in the top layer.
///
/// ```ignore
/// <DropdownMenuContent menu="user" class="w-56" side="right" align="end">…</DropdownMenuContent>
/// ```
///
/// Being in the top layer is the point rather than a detail: this component
/// exists to be used inside a [`Sidebar`](super::sidebar), whose content pane
/// clips, and a menu that is not in the top layer is a menu with its bottom half
/// cut off.
///
/// `side` and `align` are Radix's, and they place it twice — as a
/// `position-area` for browsers with CSS anchor positioning, and as the numbers
/// the script uses for the rest. Both flip when the menu would run off screen.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuContent(
    class: &str,
    menu: &str,
    side: &str,
    align: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_content()
            .class(class)
            .menu(menu)
            .side(MenuSide::parse(side))
            .align(MenuAlign::parse(align)),
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

/// One row.
///
/// ```ignore
/// <DropdownMenuItem href="/account">"Account"</DropdownMenuItem>
/// <DropdownMenuItem variant="destructive">"Delete"</DropdownMenuItem>
/// ```
///
/// With an `href` it is an `<a role="menuitem">`, which the browser can already
/// focus and activate — so the row works with the runtime absent. Without one it
/// is shadcn's `<div>`, and the script supplies `Enter` and `Space`.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuItem(
    class: &str,
    href: &str,
    variant: &str,
    inset: bool,
    disabled: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_item()
            .class(class)
            .href(href)
            .variant(ItemVariant::parse(variant))
            .inset(inset)
            .disabled(disabled),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    if href.trim().is_empty() {
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    } else {
        html! {
            <a @{..attrs}>
                <slot />
            </a>
        }
    }
}

/// A row with a tick on its left.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuCheckboxItem(
    class: &str,
    checked: bool,
    disabled: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_checkbox_item()
            .class(class)
            .checked(checked)
            .disabled(disabled),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();
    let tick = match checked {
        true => html! {
            <span class=@{INDICATOR}>
                <Check class="size-4" />
            </span>
        },
        false => Html::default(),
    };

    html! {
        <div @{..attrs}>
            @{tick}
            <slot />
        </div>
    }
}

/// A row with a dot on its left.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuRadioItem(
    class: &str,
    checked: bool,
    disabled: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_radio_item()
            .class(class)
            .checked(checked)
            .disabled(disabled),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();
    // shadcn's `CircleIcon className="size-2 fill-current"`, which is a filled
    // dot and not worth an icon file.
    let dot = match checked {
        true => html! {
            <span class=@{INDICATOR}>
                <span class="size-2 rounded-full bg-current"></span>
            </span>
        },
        false => Html::default(),
    };

    html! {
        <div @{..attrs}>
            @{dot}
            <slot />
        </div>
    }
}

/// A row that opens a submenu.
///
/// The same `popovertarget` association as the top-level trigger, so a submenu
/// opens with no JavaScript either. Give it and its [`DropdownMenuSubContent`]
/// a `menu` name of their own.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuSubTrigger(
    class: &str,
    menu: &str,
    inset: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_sub_trigger()
            .class(class)
            .menu(menu)
            .inset(inset),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    // A `<button>`, not shadcn's `<div>`. `popovertarget` is only an attribute
    // of `<button>`, and the whole of this component's open and close is that
    // attribute — see [`SUB_TRIGGER`] for the two utilities the element change
    // costs.
    html! {
        <button @{..attrs}>
            <slot />
            <ChevronRight class=@{SUB_TRIGGER_ICON} />
        </button>
    }
}

/// A submenu, opening to the right by default.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuSubContent(
    class: &str,
    menu: &str,
    side: &str,
    align: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_sub_content()
            .class(class)
            .menu(menu)
            .side(match side {
                "" => MenuSide::Right,
                other => MenuSide::parse(other),
            })
            .align(MenuAlign::parse(align)),
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

/// A macro for the six parts that are a `<div>` or `<span>`, a class list and
/// children.
macro_rules! plain {
    ($($name:ident => $builder:ident, $el:ident, $doc:literal;)*) => {
        $(
            #[doc = $doc]
            #[component]
            #[allow(clippy::too_many_arguments)]
            pub fn $name(
                class: &str,
                id: &str,
                bindings: Bindings,
                attrs: Attrs,
                children: Html,
            ) -> Html {
                let props = build($builder().class(class), id, attrs, bindings);
                let attrs = props.attr_list();

                html! { <$el @{..attrs}><slot /></$el> }
            }
        )*
    };
}

plain! {
    DropdownMenuGroup => dropdown_menu_group, div, "A run of related rows.";
    DropdownMenuRadioGroup => dropdown_menu_radio_group, div, "A run of rows where one is chosen.";
    DropdownMenuShortcut => dropdown_menu_shortcut, span, "The key hint on the right of a row.";
    DropdownMenuSub => dropdown_menu_sub, div, "A submenu's trigger and menu, paired.";
}

/// A heading over a run of rows.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuLabel(
    class: &str,
    inset: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dropdown_menu_label().class(class).inset(inset),
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

/// A rule between runs.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DropdownMenuSeparator(class: &str, id: &str, bindings: Bindings, attrs: Attrs) -> Html {
    let props = build(dropdown_menu_separator().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}></div>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, CONTENT, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel,
        DropdownMenuProps, DropdownMenuSeparator, DropdownMenuSubTrigger, DropdownMenuTrigger,
        ItemVariant, MenuAlign, MenuSide, POPOVER_RESET, SUB_TRIGGER, SUBMENU_GRACE_MS,
        anchor_name, dropdown_menu, dropdown_menu_content, dropdown_menu_content_variants,
        dropdown_menu_item, menu_id, nameable,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    /// A submenu row has to be a `<button>`.
    ///
    /// Regression, measured in the browser: `popovertarget` is only an attribute
    /// of `<button>` and the button-ish `<input>`s. shadcn's SubTrigger is a
    /// `<div role="menuitem">`, and on a `<div>` the attribute is inert —
    /// `popoverTargetElement` reads `null`, the click does nothing and the
    /// submenu never opens. Since that attribute is the whole of this
    /// component's open and close, the element is not a matter of taste here.
    #[test]
    fn a_sub_trigger_is_a_button_or_it_does_nothing() {
        let markup = DropdownMenuSubTrigger(
            "",
            "invite",
            false,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Invite people"),
        )
        .into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(markup.ends_with("</button>"), "{markup}");
        assert!(markup.contains(r#"type="button""#), "{markup}");
        assert!(markup.contains(r#"role="menuitem""#), "{markup}");
        assert!(
            markup.contains(r#"popovertarget="dropdown-menu-invite""#),
            "{markup}"
        );
        assert!(markup.contains(r#"aria-haspopup="menu""#), "{markup}");
        // The two utilities the element change costs, both visible without
        // them: a shrink-to-fit row, and a wrapped label centred.
        assert!(markup.contains("w-full"), "{markup}");
        assert!(markup.contains("text-left"), "{markup}");
        assert!(SUB_TRIGGER.contains("w-full"), "{SUB_TRIGGER}");
        assert!(SUB_TRIGGER.contains("text-left"), "{SUB_TRIGGER}");
    }

    /// The row stays highlighted while its submenu is open, as Radix's does.
    ///
    /// `open:bg-accent` is the obvious spelling and it is dead: `open:` compiles
    /// to `:is([open], :popover-open, :open)` and a *trigger* is none of those —
    /// the popover is its next sibling. Measured both halves in the browser.
    #[test]
    fn a_sub_trigger_highlights_from_its_siblings_state() {
        assert!(
            SUB_TRIGGER.contains("[&:has(+[popover]:popover-open)]:bg-accent"),
            "{SUB_TRIGGER}"
        );
        assert!(
            SUB_TRIGGER.contains("[&:has(+[popover]:popover-open)]:text-accent-foreground"),
            "{SUB_TRIGGER}"
        );
        // The dead spelling is gone rather than carried beside the live one.
        assert!(!SUB_TRIGGER.contains("open:bg-accent"), "{SUB_TRIGGER}");
        assert!(
            !SUB_TRIGGER.contains("open:text-accent-foreground"),
            "{SUB_TRIGGER}"
        );
    }

    /// A submenu opens on hover, so a click on its trigger must not toggle.
    ///
    /// Without this the pointer opens the menu on its way to the click and the
    /// click then closes what it was aiming at.
    #[test]
    fn a_sub_trigger_click_shows_rather_than_toggles() {
        let markup = DropdownMenuSubTrigger(
            "",
            "invite",
            false,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"popovertargetaction="show""#), "{markup}");

        // The top-level trigger keeps the default toggle — a click there is the
        // only way it opens, and the same click has to be able to close it.
        let top = DropdownMenuTrigger(
            "",
            "user",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(!top.contains("popovertargetaction"), "{top}");
    }

    /// The pointer highlights a row by focusing it, as Radix does — which is
    /// what makes `focus:bg-accent` the right class list and keeps one row
    /// highlighted rather than two.
    #[test]
    fn the_pointer_drives_focus_and_opens_submenus() {
        let markup = DropdownMenu(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"addEventListener("pointermove", pointed)"#),
            "{markup}"
        );
        assert!(
            markup.contains("row.focus({ preventScroll: true })"),
            "{markup}"
        );
        // Hover opens the submenu, and a grace period keeps the diagonal to it
        // from closing it on the way.
        assert!(markup.contains("wanted.showPopover()"), "{markup}");
        assert!(
            markup.contains(&format!("const GRACE_MS = {SUBMENU_GRACE_MS}")),
            "{markup}"
        );
        assert!(
            markup.contains("closeSubs(menu, null), GRACE_MS"),
            "{markup}"
        );
        // Leaving drops the highlight; entering cancels a pending close.
        assert!(
            markup.contains(r#"addEventListener("pointerenter", entered)"#),
            "{markup}"
        );
        assert!(
            markup.contains(r#"addEventListener("pointerleave", left)"#),
            "{markup}"
        );
        // And everything it added is taken off again.
        for name in ["pointermove", "pointerenter", "pointerleave"] {
            assert!(
                markup.contains(&format!(r#"removeEventListener("{name}""#)),
                "{name} is never removed"
            );
        }
    }

    fn menu(name: &str, side: &str, align: &str) -> String {
        DropdownMenuContent(
            "",
            name,
            side,
            align,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("rows"),
        )
        .into_test_string()
    }

    /// The pairing, and the whole of the open and close.
    #[test]
    fn the_trigger_points_at_the_menu_by_name() {
        let trigger = DropdownMenuTrigger(
            "",
            "user",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Open"),
        )
        .into_test_string();
        let content = menu("user", "", "");

        assert!(
            trigger.contains(r#"popovertarget="dropdown-menu-user""#),
            "{trigger}"
        );
        assert!(trigger.contains(r#"aria-haspopup="menu""#), "{trigger}");
        assert!(content.contains(r#"id="dropdown-menu-user""#), "{content}");
        assert!(content.contains(r#"popover="auto""#), "{content}");
        assert_eq!(menu_id("user"), "dropdown-menu-user");
    }

    /// …and neither half is a binding, because both work before the runtime.
    #[test]
    fn the_association_is_plain_markup() {
        let trigger = DropdownMenuTrigger(
            "",
            "user",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(!trigger.contains("{"), "{trigger}");
        assert!(!trigger.contains("<script"), "{trigger}");
    }

    /// The anchor styles, which place the menu with no JavaScript where the
    /// browser has them — written inline because Tailwind cannot emit a class
    /// built from a runtime name.
    #[test]
    fn both_halves_carry_the_anchor_name() {
        let trigger = DropdownMenuTrigger(
            "",
            "user",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();
        let content = menu("user", "", "");

        assert!(
            trigger.contains("anchor-name:--dropdown-menu-user"),
            "{trigger}"
        );
        assert!(
            content.contains("position-anchor:--dropdown-menu-user"),
            "{content}"
        );
        assert!(
            content.contains("position-area:block-end span-inline-end"),
            "{content}"
        );
        assert!(content.contains("position-try-fallbacks"), "{content}");
        assert_eq!(anchor_name("user"), "--dropdown-menu-user");
    }

    #[test]
    fn the_side_and_align_are_written_for_the_css_and_the_script() {
        let markup = menu("m", "right", "end");

        assert!(markup.contains(r#"data-side="right""#), "{markup}");
        assert!(markup.contains(r#"data-align="end""#), "{markup}");
        assert!(
            markup.contains("position-area:inline-end span-block-start"),
            "{markup}"
        );
    }

    /// The UA pins a popover with `inset: 0`, which fights every placement.
    #[test]
    fn the_popover_inset_is_cleared() {
        let list = dropdown_menu_content_variants("");

        assert!(list.starts_with(POPOVER_RESET), "{list}");
        assert!(list.contains("inset-auto"), "{list}");
    }

    /// `open:` is Tailwind's variant for `:popover-open`, so shadcn's enter
    /// animation crossed with the prefix rewritten.
    #[test]
    fn the_enter_animation_reads_the_popover_state() {
        assert!(CONTENT.contains("open:animate-in"), "{CONTENT}");
        assert!(CONTENT.contains("open:zoom-in-95"), "{CONTENT}");
        // A popover is `display: none` the instant it closes, so an exit
        // keyframe would never be seen.
        assert!(!CONTENT.contains("animate-out"), "{CONTENT}");
        assert!(!CONTENT.contains("data-[state="), "{CONTENT}");
    }

    /// An item with an href is a link, which works with no runtime at all.
    #[test]
    fn an_item_with_an_href_is_an_anchor() {
        let link = DropdownMenuItem(
            "",
            "/account",
            "",
            false,
            false,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Account"),
        )
        .into_test_string();
        let plain = DropdownMenuItem(
            "",
            "",
            "",
            false,
            false,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Copy"),
        )
        .into_test_string();

        assert!(link.starts_with("<a "), "{link}");
        assert!(link.contains(r#"href="/account""#), "{link}");
        assert!(link.contains(r#"role="menuitem""#), "{link}");
        assert!(plain.starts_with("<div "), "{plain}");
        assert!(plain.contains(r#"role="menuitem""#), "{plain}");
    }

    /// One tab stop for the menu and a roving focus inside it — the ARIA menu
    /// pattern.
    #[test]
    fn a_row_is_reached_with_the_arrows_rather_than_tab() {
        let markup = DropdownMenuItem(
            "",
            "",
            "",
            false,
            false,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"tabindex="-1""#), "{markup}");
    }

    #[test]
    fn a_destructive_row_says_so_where_the_class_list_reads_it() {
        let markup = DropdownMenuItem(
            "",
            "",
            "destructive",
            false,
            true,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-variant="destructive""#), "{markup}");
        assert!(markup.contains(r#"data-disabled="""#), "{markup}");
        assert!(markup.contains(r#"aria-disabled="true""#), "{markup}");
    }

    #[test]
    fn the_label_and_separator_are_shadcns() {
        let label = DropdownMenuLabel(
            "",
            true,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Account"),
        )
        .into_test_string();
        let rule = DropdownMenuSeparator("", "", crate::rahti::Bindings::default(), Attrs::new())
            .into_test_string();

        assert!(
            label.contains(r#"data-slot="dropdown-menu-label""#),
            "{label}"
        );
        assert!(label.contains(r#"data-inset="""#), "{label}");
        assert!(rule.contains(r#"role="separator""#), "{rule}");
        assert!(rule.contains("-mx-1 my-1 h-px bg-border"), "{rule}");
    }

    /// The root is `display: contents` and holds the script.
    #[test]
    fn the_root_is_contents_and_carries_the_script() {
        let markup = DropdownMenu(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("inside"),
        )
        .into_test_string();

        assert!(markup.contains(r#"class="contents""#), "{markup}");
        assert!(
            markup.contains(r#"data-slot="dropdown-menu-anchor""#),
            "{markup}"
        );
        assert!(markup.contains(r#"pp-ref="{anchorRef}""#), "{markup}");
        assert!(markup.contains("<script"), "{markup}");
    }

    /// The name is a CSS identifier as well as a document id.
    #[test]
    fn a_name_has_to_work_as_both() {
        assert!(nameable("user"));
        assert!(nameable("team-switcher"));
        assert!(nameable("menu_1"));
        assert!(!nameable(""));
        assert!(!nameable("1menu"));
        assert!(!nameable("my menu"));
        assert!(!nameable("menu.one"));
    }

    #[test]
    fn only_the_root_always_needs_a_scope() {
        assert!(dropdown_menu().needs_scope());
        assert!(!dropdown_menu_content().menu("m").needs_scope());
        assert!(!dropdown_menu_item().needs_scope());
        assert!(!dropdown_menu_item().class("pl-8").needs_scope());
    }

    #[test]
    fn an_unknown_word_falls_back_to_shadcns() {
        assert_eq!(MenuSide::parse(""), MenuSide::Bottom);
        assert_eq!(MenuAlign::parse(""), MenuAlign::Start);
        assert_eq!(ItemVariant::parse(""), ItemVariant::Default);
        let props = DropdownMenuProps::default();
        assert_eq!(props.side, MenuSide::Bottom);
        assert!(!props.inset);
    }
}

#[cfg(test)]
mod audit {
    use super::{
        AVAILABLE_HEIGHT, CHOICE_ITEM, CONTENT, INDICATOR, ITEM, LABEL, POPOVER_RESET, SEPARATOR,
        SHORTCUT, SUB_CONTENT, SUB_TRIGGER,
    };
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// A menu states its own text alignment, because it is not portalled.
    ///
    /// Radix moves its content to the end of `<body>`, so it inherits nothing
    /// from the trigger's surroundings. A popover stays where it was written,
    /// and a row-actions menu is written inside a right-aligned cell — which
    /// dragged the label and the rows to the wrong edge until `text-left` was
    /// added. Measured on `/components/table` and `/components/data-table`.
    #[test]
    fn a_menu_does_not_inherit_its_containers_alignment() {
        assert!(CONTENT.contains("text-left"), "{CONTENT}");
        assert!(SUB_CONTENT.contains("text-left"), "{SUB_CONTENT}");
    }

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered.
    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("CONTENT", cn(&[POPOVER_RESET, AVAILABLE_HEIGHT, CONTENT])),
            ("SUB_CONTENT", cn(&[POPOVER_RESET, SUB_CONTENT])),
            ("ITEM", ITEM.to_string()),
            ("CHOICE_ITEM", CHOICE_ITEM.to_string()),
            ("INDICATOR", INDICATOR.to_string()),
            ("LABEL", LABEL.to_string()),
            ("SEPARATOR", SEPARATOR.to_string()),
            ("SHORTCUT", SHORTCUT.to_string()),
            ("SUB_TRIGGER", SUB_TRIGGER.to_string()),
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

    /// Two Radix custom properties nothing here publishes, and the exit
    /// animation a popover cannot show. All four stayed out.
    #[test]
    fn nothing_reaches_for_what_radix_published() {
        for list in [CONTENT, SUB_CONTENT] {
            assert!(!list.contains("--radix-"), "{list}");
            assert!(!list.contains("animate-out"), "{list}");
        }
        // The available height is asked of the browser instead.
        assert!(AVAILABLE_HEIGHT.contains("dvh"), "{AVAILABLE_HEIGHT}");
    }
}
