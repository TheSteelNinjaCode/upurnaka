// rahti-ui: input-group
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.3.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Input Group — a port of shadcn/ui's Base UI `input-group`.
//!
//! Six parts. An Input Group is the border and the focus ring lifted off a
//! control and put around a *row*, so an icon, a prefix, a unit, or a button
//! can sit inside the field rather than beside it. The control keeps the
//! typing and loses its own chrome; the group draws everything the eye reads
//! as "the field".
//!
//! | Tag | Element | What it is |
//! | --- | --- | --- |
//! | [`InputGroup`] | `<div role="group">` | The bordered row |
//! | [`InputGroupInput`] | `<input>` | The control, stripped of its chrome |
//! | [`InputGroupTextarea`] | `<textarea>` | The same, for many lines |
//! | [`InputGroupAddon`] | `<div role="group">` | What sits beside the control |
//! | [`InputGroupButton`] | `<button>` | A button sized to fit inside |
//! | [`InputGroupText`] | `<span>` | Muted text inside an addon |
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <InputGroup>
//!     <InputGroupInput placeholder="Search…" />
//!     <InputGroupAddon>@{search_icon}</InputGroupAddon>
//! </InputGroup>
//! ```
//!
//! **Write the addon after the control**, whichever side it appears on. That
//! is shadcn's instruction and it is about the keyboard, not the layout: Tab
//! follows DOM order, and a `<button>` written before the input would take the
//! focus a user was reaching for. Which side an addon *draws* on is
//! [`align`](InputGroupAlign) — `order-first` and `order-last` in the class
//! list do the moving.
//!
//! Use `inline-start` / `inline-end` with an [`InputGroupInput`], and
//! `block-start` / `block-end` with an [`InputGroupTextarea`]: a block-aligned
//! addon turns the group into a column (`has-[>[data-align=block-end]]:flex-col`)
//! and takes the full width.
//!
//! ## The state is asked for, not passed down
//!
//! Nothing here threads a prop. Every state the group draws is a question CSS
//! asks about its own contents:
//!
//! - **Focus.** `has-[[data-slot=input-group-control]:focus-visible]:ring-3` —
//!   the *group* rings when the control inside is focused, which is why the
//!   control's own ring is turned off (`focus-visible:ring-0`).
//! - **Invalid.** `has-[[data-slot][aria-invalid=true]]:border-destructive`.
//!   Mark the control `aria-invalid`; the group reddens.
//! - **Disabled.** `has-disabled:bg-input/50`. Disable the control; the group
//!   dims.
//! - **Shape.** `has-[>textarea]:h-auto` and `has-[>[data-align=block-end]]:flex-col`
//!   — the row becomes a column because of what it was given, not because it
//!   was told.
//!
//! Every one of those is a `has-[>…]` or a descendant question about **direct
//! children**, which makes the markup shape the API. See the note on overrides
//! below, and [`Field`](super::field), whose family has the same property.
//!
//! ## What this port changes, and why
//!
//! 1. **The group is `h-9 rounded-md`, where shadcn writes `h-8 rounded-lg`.**
//!    Those two numbers are shadcn's *Input's* height and radius, and this
//!    library's [`Input`](super::input) is `h-9 rounded-md` — it is a port of
//!    the earlier shadcn Input, and every control here matches it. A group at
//!    shadcn's 32px would be the only 32px control in the library and would sit
//!    a pixel-and-a-half short of a Button beside it. Everything else — the
//!    addon paddings, the negative margins, the button metrics, the ring — is
//!    shadcn's string, unchanged.
//! 2. **The addon's click-to-focus is a native `onclick`, not a binding.**
//!    shadcn attaches a React handler that focuses the input unless the click
//!    landed on a button. That is what `cursor-text` on the addon promises, and
//!    it is three lines of DOM. Written as a plain inline handler it costs an
//!    addon no mounted scope, works before hydration, and is taken over by the
//!    runtime after it — see [`FOCUS_CONTROL`], which says what that handover
//!    looks like and what was measured.
//! 3. **[`InputGroupButton`]'s `sm` is this library's small Button.** shadcn's
//!    size layer spells `sm` as the empty string, meaning "whatever the Button
//!    already is" — and their Button's default is `h-8 px-2.5`. Ours is `h-9`,
//!    so `sm` is written out as the `h-8` button this library actually has. The
//!    other three sizes are shadcn's layer verbatim.
//!
//! ## An override must not become a wrapper
//!
//! Same rule as [`Field`](super::field), for the same reason and with one extra
//! wrinkle. A `class` override makes a block mountable, a mountable block needs
//! a `<script>`, and a `<script>` written *beside* the element makes the block's
//! root a fragment — which the runtime raises into a `<pp-fragment>` that
//! `has-[>textarea]` and `[&>input]:pl-1.5` cannot see through.
//!
//! So the four parts that can hold one — [`InputGroup`], [`InputGroupAddon`],
//! [`InputGroupButton`], [`InputGroupText`] — put the script *inside* the
//! element, as its first child. A `<script>` is `display: none`, so it is not a
//! flex item and takes no gap, and the runtime removes it once the block is
//! mounted.
//!
//! [`InputGroupInput`] and [`InputGroupTextarea`] cannot: an `<input>` is void,
//! and anything inside a `<textarea>` is its *value*. Those two fall back to the
//! fragment the rest of the library uses, and an override on them therefore
//! costs the group's `>` rules. Reach for
//! [`input_group_input_variants`] / [`input_group_textarea_variants`] on a
//! native element instead:
//!
//! ```ignore
//! <InputGroup>
//!     <textarea
//!         class=@{input_group_textarea_variants("font-mono text-sm")}
//!         data-slot="input-group-control"
//!         rows="3"
//!     ></textarea>
//!     <InputGroupAddon align="block-end">"Markdown supported"</InputGroupAddon>
//! </InputGroup>
//! ```
//!
//! `data-slot="input-group-control"` written by hand is not decoration there:
//! it is what the group's focus and invalid rules look for.
//!
//! ## A click crosses the boundary; a binding does not
//!
//! Same rule as [`Button`](super::button). `onclick={…}` on a tag lands on the
//! element untouched and is resolved when it fires. An attribute *binding* is
//! compiled against the scope its element sits in, which through a tag is this
//! component's — so a group wired to the page's state wants
//! [`InputGroupProps::render`] interpolated into the page's own block, or the
//! matching `*_variants` function on native markup.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::button::ButtonVariant;
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// The bordered row.
///
/// shadcn's string with two numbers changed — `h-8` to `h-9` and `rounded-lg`
/// to `rounded-md`, which are this library's control height and radius. The
/// module docs argue for it.
///
/// Almost everything else is a question about the children. The four
/// `has-[>[data-align=…]]:[&>input]:p…` rules are the subtle ones: they trim
/// the input's own padding on whichever side an addon showed up, so the text
/// starts just past the icon rather than a full field-padding away from it.
///
/// `in-data-[slot=combobox-content]:` is carried verbatim. There is no Combobox
/// in this library yet; when there is, the rule that stops a group inside a
/// popup from drawing its own focus ring is already here.
const GROUP: &str = "group/input-group relative flex h-9 w-full min-w-0 items-center \
                     rounded-md border border-input transition-colors outline-none \
                     in-data-[slot=combobox-content]:focus-within:border-inherit \
                     in-data-[slot=combobox-content]:focus-within:ring-0 \
                     has-disabled:bg-input/50 has-disabled:opacity-50 \
                     has-[[data-slot=input-group-control]:focus-visible]:border-ring \
                     has-[[data-slot=input-group-control]:focus-visible]:ring-3 \
                     has-[[data-slot=input-group-control]:focus-visible]:ring-ring/50 \
                     has-[[data-slot][aria-invalid=true]]:border-destructive \
                     has-[[data-slot][aria-invalid=true]]:ring-3 \
                     has-[[data-slot][aria-invalid=true]]:ring-destructive/20 \
                     has-[>[data-align=block-end]]:h-auto \
                     has-[>[data-align=block-end]]:flex-col \
                     has-[>[data-align=block-start]]:h-auto \
                     has-[>[data-align=block-start]]:flex-col \
                     has-[>textarea]:h-auto \
                     dark:bg-input/30 dark:has-disabled:bg-input/80 \
                     dark:has-[[data-slot][aria-invalid=true]]:ring-destructive/40 \
                     has-[>[data-align=block-end]]:[&>input]:pt-3 \
                     has-[>[data-align=block-start]]:[&>input]:pb-3 \
                     has-[>[data-align=inline-end]]:[&>input]:pr-1.5 \
                     has-[>[data-align=inline-start]]:[&>input]:pl-1.5";

/// Everything an addon is before its alignment.
///
/// `cursor-text` is the promise [`FOCUS_CONTROL`] keeps: the addon looks like
/// part of the field, so clicking it should put the caret in the field.
///
/// `justify-center` is **not** here, though shadcn's base has it — the two
/// block alignments replace it with `justify-start`, and a layer that
/// contradicts another is a layer this library cannot join. It lives in the two
/// inline alignments instead, which is where it survives shadcn's own merge.
const ADDON: &str = "flex h-auto cursor-text items-center gap-2 py-1.5 text-sm font-medium \
                     text-muted-foreground select-none \
                     group-data-[disabled=true]/input-group:opacity-50 \
                     [&>kbd]:rounded-[calc(var(--radius)-5px)] \
                     [&>svg:not([class*='size-'])]:size-4";

/// Before the control, on the inline axis — an icon, a currency, a protocol.
///
/// The negative margins are the fitting: a button inside an addon carries its
/// own padding, and `ml-[-0.3rem]` pulls that back so the button's *edge*, not
/// its padding, lines up with where an icon would have been.
const INLINE_START: &str = "justify-center order-first pl-2 \
                            has-[>button]:ml-[-0.3rem] has-[>kbd]:ml-[-0.15rem]";

/// After the control, on the inline axis — a unit, a clear button, a shortcut.
const INLINE_END: &str = "justify-center order-last pr-2 \
                          has-[>button]:mr-[-0.3rem] has-[>kbd]:mr-[-0.15rem]";

/// Above the control, full width. Turns the group into a column.
///
/// `[.border-b]:pb-2` is a rule about a class the *caller* writes: add
/// `border-b` to the addon and it gets the padding a divider needs. shadcn's
/// idea, kept.
const BLOCK_START: &str = "order-first w-full justify-start px-2.5 pt-2 \
                           group-has-[>input]/input-group:pt-2 [.border-b]:pb-2";

/// Below the control, full width — a toolbar under a textarea.
const BLOCK_END: &str = "order-last w-full justify-start px-2.5 pb-2 \
                         group-has-[>input]/input-group:pb-2 [.border-t]:pt-2";

/// What an addon does when it is clicked: put the caret in the field.
///
/// A native inline handler rather than a PulsePoint binding, which is what
/// keeps an addon free of a mounted scope: it carries no `{…}`, so
/// [`needs_scope`](InputGroupProps::needs_scope) stays false and no `<script>`
/// is written.
///
/// `this` in an inline handler is the element the attribute is on, which is
/// shadcn's `e.currentTarget`. The early return is theirs too: a click that
/// landed on a button was meant for the button.
///
/// Deliberately brace-free — a `{` with a `}` after it is what
/// [`is_binding`](super::attrs::is_binding) looks for.
///
/// **The runtime takes it over.** PulsePoint claims every `on*` attribute it
/// finds: on hydration the attribute is gone from the element and the handler
/// is a listener instead. That is a handover, not a loss — measured on
/// `/components/input-group`, all three of shadcn's behaviours hold after
/// mount: clicking the addon focuses the control, clicking a button inside it
/// does not, and a textarea's addon does neither. Before hydration the
/// browser's own inline handler does the same job, so the served page is not
/// waiting on JavaScript to become clickable.
///
/// It queries `input`, exactly as shadcn does — which means an addon beside an
/// [`InputGroupTextarea`] is not clickable-to-focus in either library.
const FOCUS_CONTROL: &str =
    "if(event.target.closest('button'))return;this.parentElement?.querySelector('input')?.focus()";

/// Muted text inside an addon.
const TEXT: &str = "flex items-center gap-2 text-sm text-muted-foreground \
                    [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4";

/// [`Button`](super::button)'s class list, minus the two utilities the input
/// group replaces.
///
/// `inline-flex` goes because the group's button layer says `flex`, and
/// `rounded-md` goes because the radius is per size here — the two `xs` sizes
/// are tighter than a normal button. shadcn composes a `<Button>` and lets
/// `cn` resolve both; there is no Rust merge to do that with, so the merge is
/// done here and `the_button_merge_is_shadcns` proves it against
/// [`button_variants`](super::button::button_variants) rather than trusting
/// the transcription.
const BUTTON_KEPT: &str = "shrink-0 items-center justify-center font-medium whitespace-nowrap \
                           transition-all outline-none focus-visible:border-ring \
                           focus-visible:ring-[3px] disabled:pointer-events-none \
                           disabled:opacity-50 aria-invalid:border-destructive \
                           aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 \
                           [&_svg]:pointer-events-none [&_svg]:shrink-0";

/// What every input-group button adds on top of a plain one, whatever its size.
///
/// `shadow-none` is the reason [`variant_kept`] exists: an `Outline` button
/// carries `shadow-xs`, and a button sitting *inside* a field must not cast a
/// shadow onto it.
const BUTTON_SHARED: &str = "flex text-sm shadow-none";

/// The default: a small button that fits a `h-9` field with room to spare.
const BUTTON_XS: &str = "h-6 gap-1 rounded-[calc(var(--radius)-3px)] px-1.5 \
                         [&_svg:not([class*='size-'])]:size-4 \
                         [&>svg:not([class*='size-'])]:size-3.5";

/// This library's `h-8` button, which is what shadcn's empty `sm` layer means.
const BUTTON_SM: &str = "h-8 gap-2 rounded-md px-2.5 [&_svg:not([class*='size-'])]:size-4";

/// Square, for a button that is only an icon.
const BUTTON_ICON_XS: &str = "size-6 gap-2 rounded-[calc(var(--radius)-3px)] p-0 \
                              has-[>svg]:p-0 [&_svg:not([class*='size-'])]:size-4";

/// The same, one step up.
const BUTTON_ICON_SM: &str = "size-8 gap-2 rounded-md p-0 has-[>svg]:p-0 \
                              [&_svg:not([class*='size-'])]:size-4";

/// [`Input`](super::input)'s class list, minus what [`CONTROL`] replaces.
///
/// Five utilities go, and each of them is chrome the *group* now draws:
/// `rounded-md`, `border`, `shadow-xs`, `focus-visible:ring-[3px]` and
/// `dark:bg-input/30`. What stays is everything about the text — the height,
/// the padding, the placeholder, the file button, the selection colour.
const INPUT_KEPT: &str = "w-full min-w-0 border-input bg-transparent px-3 py-1 text-base \
                          transition-[color,box-shadow] outline-none selection:bg-primary \
                          selection:text-primary-foreground file:inline-flex file:h-7 \
                          file:border-0 file:bg-transparent file:text-sm file:font-medium \
                          file:text-foreground placeholder:text-muted-foreground \
                          disabled:pointer-events-none disabled:cursor-not-allowed \
                          disabled:opacity-50 md:text-sm focus-visible:border-ring \
                          focus-visible:ring-ring/50 aria-invalid:border-destructive \
                          aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40";

/// What a control gives up to sit inside a group: its border, its rounding, its
/// shadow and its ring.
///
/// `focus-visible:ring-0` is the one worth reading twice. The control still
/// takes focus and still shows it — but the ring is drawn by the group, around
/// the whole row, so the control turning its own off is what makes the two look
/// like one field.
/// `h-full` rather than the [`Input`](super::input)'s own `h-9`, and that is a
/// fix rather than a preference.
///
/// Measured in the browser: the group is `h-9` *and draws the border*, so its
/// content box is 36px less two border widths — 34.67px at a 1.5 device pixel
/// ratio. A control that keeps its own `h-9` is therefore 1.33px taller than the
/// box it sits in, and its square, `rounded-none` corners hang over the group's
/// rounded ones. Transparent, that is invisible; the moment the border darkens
/// on focus the overhang paints past the curve and reads as a straight vertical
/// line at each end of the field.
///
/// `h-full` makes the control exactly the content box. shadcn has the same
/// overhang — its control inherits `h-9` from its Input and its group is `h-9`
/// too — so this is one of the few places this port is deliberately not a copy.
/// The field is still 36px tall; only the control now respects the border.
///
/// A textarea does not want this: the group goes `h-auto` around one, so
/// [`TEXTAREA_CONTROL`] keeps its own sizing.
const CONTROL: &str = "h-full flex-1 rounded-none border-0 bg-transparent shadow-none ring-0 \
                       focus-visible:ring-0 disabled:bg-transparent aria-invalid:ring-0 \
                       dark:bg-transparent dark:disabled:bg-transparent";

/// [`Textarea`](super::textarea)'s class list, minus the same five.
const TEXTAREA_KEPT: &str = "flex field-sizing-content min-h-16 w-full border-input \
                             bg-transparent px-3 py-2 text-base transition-[color,box-shadow] \
                             outline-none placeholder:text-muted-foreground \
                             focus-visible:border-ring focus-visible:ring-ring/50 \
                             disabled:cursor-not-allowed disabled:opacity-50 \
                             aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                             md:text-sm dark:aria-invalid:ring-destructive/40";

/// [`CONTROL`], plus the one thing a textarea needs and an input does not.
///
/// `resize-none` because the group draws the border: a corner grip would drag
/// the textarea out of the box that is supposed to contain it.
const TEXTAREA_CONTROL: &str = "flex-1 resize-none rounded-none border-0 bg-transparent py-2 \
                                shadow-none ring-0 focus-visible:ring-0 disabled:bg-transparent \
                                aria-invalid:ring-0 dark:bg-transparent dark:disabled:bg-transparent";

/// Which side of the control an addon draws on.
///
/// Written into `data-align`, which is what the group reads to decide whether
/// it is a row or a column. The name is the CSS logical one: `inline` is the
/// text direction, `block` is down the page, so this is right in a
/// right-to-left layout without a second spelling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputGroupAlign {
    /// Before the control, on the text axis. The default.
    #[default]
    InlineStart,
    /// After the control, on the text axis.
    InlineEnd,
    /// Above the control, full width. Turns the group into a column.
    BlockStart,
    /// Below the control, full width. Turns the group into a column.
    BlockEnd,
}

impl InputGroupAlign {
    /// The name shadcn writes into `data-align`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
            Self::BlockStart => "block-start",
            Self::BlockEnd => "block-end",
        }
    }

    /// The class layer this alignment contributes.
    pub const fn layer(self) -> &'static str {
        match self {
            Self::InlineStart => INLINE_START,
            Self::InlineEnd => INLINE_END,
            Self::BlockStart => BLOCK_START,
            Self::BlockEnd => BLOCK_END,
        }
    }

    /// The [`slug`](Self::slug) read back, for the `align="…"` prop.
    ///
    /// An empty string is the default rather than an error, because that is
    /// what an unwritten prop arrives as.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "inline-start" => Self::InlineStart,
            "inline-end" => Self::InlineEnd,
            "block-start" => Self::BlockStart,
            "block-end" => Self::BlockEnd,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not an InputGroupAddon align. One of: \
                     inline-start, inline-end, block-start, block-end."
                );
                Self::InlineStart
            }
        }
    }
}

/// How big a button inside a group is.
///
/// Four, not the Button's eight: a button that has to fit inside a field has
/// only so much room, and shadcn narrows the choice for the same reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputGroupButtonSize {
    /// `h-6`, tight radius. The default.
    #[default]
    Xs,
    /// `h-8` — this library's small button. See the module docs.
    Sm,
    /// Square `size-6`, no padding.
    IconXs,
    /// Square `size-8`, no padding.
    IconSm,
}

impl InputGroupButtonSize {
    /// The name shadcn writes into `data-size`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::IconXs => "icon-xs",
            Self::IconSm => "icon-sm",
        }
    }

    /// The class layer this size contributes.
    pub const fn layer(self) -> &'static str {
        match self {
            Self::Xs => BUTTON_XS,
            Self::Sm => BUTTON_SM,
            Self::IconXs => BUTTON_ICON_XS,
            Self::IconSm => BUTTON_ICON_SM,
        }
    }

    /// The [`slug`](Self::slug) read back, for the `size="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "xs" => Self::Xs,
            "sm" => Self::Sm,
            "icon-xs" => Self::IconXs,
            "icon-sm" => Self::IconSm,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not an InputGroupButton size. One of: \
                     xs, sm, icon-xs, icon-sm."
                );
                Self::Xs
            }
        }
    }
}

/// A [`ButtonVariant`]'s utilities, minus the shadow [`BUTTON_SHARED`] turns
/// off.
///
/// Only `Outline` carries one. A button inside a field must not cast a shadow
/// onto the field, and `shadow-xs` beside `shadow-none` is a contradiction this
/// library resolves here rather than in the browser.
fn variant_kept(variant: ButtonVariant) -> String {
    variant
        .class()
        .split_whitespace()
        .filter(|utility| *utility != "shadow-xs")
        .collect::<Vec<_>>()
        .join(" ")
}

/// Which of the six a [`InputGroupProps`] is.
///
/// One props type serves them all, because the six differ in the tag they
/// render, the `data-slot` they carry and the class list they own — and not in
/// how `class`, `attrs`, bindings and the scope rule work. Same arrangement as
/// [`Field`](super::field), for the same reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Group,
    Addon,
    Button,
    Text,
    Input,
    Textarea,
}

impl Part {
    /// The element this part renders.
    const fn tag(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Text => "span",
            Self::Input => "input",
            Self::Textarea => "textarea",
            _ => "div",
        }
    }

    /// The `data-slot` the group's own selectors look for, if any.
    ///
    /// [`Part::Text`] has none, which is shadcn's: a `<span>` of muted text is
    /// not a thing the group needs to find, and `[data-slot]` appears in the
    /// group's invalid rule as a *presence* test.
    const fn slot(self) -> Option<&'static str> {
        match self {
            Self::Group => Some("input-group"),
            Self::Addon => Some("input-group-addon"),
            Self::Button => Some("button"),
            Self::Text => None,
            Self::Input | Self::Textarea => Some("input-group-control"),
        }
    }

    /// Can a `<script>` live inside this element?
    ///
    /// No for the two controls, and for two different reasons: an `<input>` is
    /// void, and anything inside a `<textarea>` is its value. Those fall back
    /// to a fragment, which the module docs say costs the group's `>` rules.
    const fn holds_script(self) -> bool {
        !matches!(self, Self::Input | Self::Textarea)
    }
}

/// The class list a part renders with, before the caller's own.
///
/// A join, not a merge — [`no_layer_contradicts_another`](audit) holds every
/// one of these conflict-free, across all four alignments, all four button
/// sizes and all six button variants.
fn part_classes(
    part: Part,
    align: InputGroupAlign,
    size: InputGroupButtonSize,
    variant: ButtonVariant,
) -> String {
    match part {
        Part::Group => GROUP.to_string(),
        Part::Addon => cn(&[ADDON, align.layer()]),
        Part::Button => cn(&[
            BUTTON_KEPT,
            &variant_kept(variant),
            BUTTON_SHARED,
            size.layer(),
        ]),
        Part::Text => TEXT.to_string(),
        Part::Input => cn(&[INPUT_KEPT, CONTROL]),
        Part::Textarea => cn(&[TEXTAREA_KEPT, TEXTAREA_CONTROL]),
    }
}

/// [`InputGroup`]'s class list, the caller's own included.
///
/// Reach for this when the element has to belong to the page. Write the
/// `role="group"` and `data-slot="input-group"` alongside — the parts inside
/// ask for them.
pub fn input_group_variants(class: &str) -> String {
    cn(&[GROUP, class])
}

/// [`InputGroupAddon`]'s class list for one alignment, the caller's own
/// included.
///
/// The alignment is not only a class layer: `data-align` has to be written
/// alongside, because that is what the group reads to become a column.
pub fn input_group_addon_variants(align: InputGroupAlign, class: &str) -> String {
    cn(&[
        &part_classes(
            Part::Addon,
            align,
            InputGroupButtonSize::Xs,
            ButtonVariant::Ghost,
        ),
        class,
    ])
}

/// [`InputGroupButton`]'s class list, the caller's own included.
pub fn input_group_button_variants(
    variant: ButtonVariant,
    size: InputGroupButtonSize,
    class: &str,
) -> String {
    cn(&[
        &part_classes(Part::Button, InputGroupAlign::InlineStart, size, variant),
        class,
    ])
}

/// [`InputGroupText`]'s class list, the caller's own included.
pub fn input_group_text_variants(class: &str) -> String {
    cn(&[TEXT, class])
}

/// [`InputGroupInput`]'s class list, the caller's own included.
///
/// The escape hatch the module docs point at: a native `<input>` carrying this
/// and `data-slot="input-group-control"` is an InputGroupInput in every way the
/// group can see, and it stays one literal element whatever the override.
pub fn input_group_input_variants(class: &str) -> String {
    cn(&[
        &part_classes(
            Part::Input,
            InputGroupAlign::InlineStart,
            InputGroupButtonSize::Xs,
            ButtonVariant::Ghost,
        ),
        class,
    ])
}

/// [`InputGroupTextarea`]'s class list, the caller's own included.
pub fn input_group_textarea_variants(class: &str) -> String {
    cn(&[
        &part_classes(
            Part::Textarea,
            InputGroupAlign::InlineStart,
            InputGroupButtonSize::Xs,
            ButtonVariant::Ghost,
        ),
        class,
    ])
}

/// One Input Group part's props, with shadcn's defaults already in place.
///
/// Build one with [`input_group`], [`input_group_addon`] and the rest — each
/// picks the part, and everything after it is the same chain. Nothing here is
/// required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose. Everything else a part carries — `id`,
/// `placeholder`, `aria-invalid` — is an ordinary attribute and lives in
/// `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputGroupProps {
    part: Part,
    align: InputGroupAlign,
    size: InputGroupButtonSize,
    variant: ButtonVariant,
    class: String,
    attrs: Attrs,
}

/// A default [`InputGroup`]'s props — the start of every chain.
pub fn input_group() -> InputGroupProps {
    InputGroupProps::of(Part::Group)
}

/// A default [`InputGroupAddon`]'s props.
pub fn input_group_addon() -> InputGroupProps {
    InputGroupProps::of(Part::Addon)
}

/// A default [`InputGroupButton`]'s props — `ghost`, `xs`, `type="button"`.
pub fn input_group_button() -> InputGroupProps {
    InputGroupProps::of(Part::Button)
}

/// A default [`InputGroupText`]'s props.
pub fn input_group_text() -> InputGroupProps {
    InputGroupProps::of(Part::Text)
}

/// A default [`InputGroupInput`]'s props.
pub fn input_group_input() -> InputGroupProps {
    InputGroupProps::of(Part::Input)
}

/// A default [`InputGroupTextarea`]'s props.
pub fn input_group_textarea() -> InputGroupProps {
    InputGroupProps::of(Part::Textarea)
}

impl InputGroupProps {
    fn of(part: Part) -> Self {
        Self {
            part,
            // shadcn's Button default is `default`; an input-group button's is
            // `ghost`, because it sits on the field's own background.
            variant: ButtonVariant::Ghost,
            ..Self::default()
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the part's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Which side of the control an [`InputGroupAddon`] draws on.
    pub fn align(mut self, align: InputGroupAlign) -> Self {
        debug_assert!(
            self.part == Part::Addon,
            "rahti-ui: only an InputGroupAddon has an align."
        );
        self.align = align;
        self
    }

    /// How big an [`InputGroupButton`] is.
    pub fn size(mut self, size: InputGroupButtonSize) -> Self {
        debug_assert!(
            self.part == Part::Button,
            "rahti-ui: only an InputGroupButton has a size."
        );
        self.size = size;
        self
    }

    /// How loud an [`InputGroupButton`] is. `Ghost` unless said otherwise.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        debug_assert!(
            self.part == Part::Button,
            "rahti-ui: only an InputGroupButton has a variant."
        );
        self.variant = variant;
        self
    }

    /// `name="…"` — for a control that a form posts.
    pub fn name(self, name: &str) -> Self {
        self.attr("name", name)
    }

    /// The part's own id.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// `placeholder="…"`, for the two controls.
    pub fn placeholder(self, placeholder: &str) -> Self {
        self.attr("placeholder", placeholder)
    }

    /// `value="…"` — the server's answer, and the browser's starting point.
    pub fn value(self, value: &str) -> Self {
        self.attr("value", value)
    }

    /// `type="…"` on an [`InputGroupInput`], or on an [`InputGroupButton`]
    /// where it decides whether a click submits the form around it.
    ///
    /// Named `kind` because `type` is a Rust keyword, as
    /// [`InputProps::kind`](super::input::InputProps::kind) is.
    pub fn kind(self, kind: &str) -> Self {
        self.attr("type", kind)
    }

    /// `disabled` — which the group reads with `has-disabled:` and dims itself
    /// for. `false` leaves the attribute off rather than writing
    /// `disabled="false"`, which HTML would read as disabled.
    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.flag("disabled")
        } else {
            self
        }
    }

    /// `required`.
    pub fn required(self, required: bool) -> Self {
        if required {
            self.flag("required")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` — what reddens the whole group.
    ///
    /// Written on the *control*, not on the group: the group asks
    /// `has-[[data-slot][aria-invalid=true]]`, and this is the half a screen
    /// reader is told as well as the half a reader sees.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.attr("aria-invalid", "true")
        } else {
            self
        }
    }

    /// Any attribute, rendered on the server: `attr("maxlength", "64")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("readonly")`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("onclick")`.
    ///
    /// The one worth knowing about is an addon's: `unset("onclick")` gives up
    /// [`FOCUS_CONTROL`], for an addon whose contents handle their own clicks.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("value", "query")`.
    ///
    /// Through a tag the same binding is a `{…}` prop: it lands on the root,
    /// and a component's root attributes are evaluated where the tag was
    /// written. This is its spelling for an element [`render`](Self::render)ed
    /// into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("input", "setQuery(target.value)")`.
    ///
    /// Use native lowercase event names, not the JSX spellings. On an addon
    /// this replaces [`FOCUS_CONTROL`] rather than sitting beside it — an
    /// attribute is written once.
    pub fn on(self, event: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let name = if event.starts_with("on") {
            event.to_string()
        } else {
            format!("on{event}")
        };
        self.bind(&name, expr)
    }

    /// Everything in `attrs`, laid over what is already set — shadcn's
    /// `{...props}`, and the last word on all of it.
    pub fn attrs(mut self, attrs: Attrs) -> Self {
        let mut merged = std::mem::take(&mut self.attrs);
        merged = merged.merge(&attrs);
        self.attrs = merged;
        self
    }

    /// The class list this part renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[
            &part_classes(self.part, self.align, self.size, self.variant),
            &self.class,
        ])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// [`FOCUS_CONTROL`] deliberately does not count: it carries no `{…}`, so
    /// [`is_binding`](super::attrs::is_binding) reads it as the ordinary
    /// attribute it is, and an addon costs the runtime nothing.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order: the component's own first,
    /// the caller's spread last so it wins.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new();

        if matches!(self.part, Part::Group | Part::Addon) {
            base = base.set("role", "group");
        }
        if let Some(slot) = self.part.slot() {
            base = base.set("data-slot", slot);
        }
        if self.part == Part::Addon {
            base = base
                .set("data-align", self.align.slug())
                // shadcn's React handler, as the inline one HTML already has.
                .bind("onclick", crate::components::rahti_ui::attrs::ui_handler!({if(event.target.closest("button"))return;this.parentElement?.querySelector("input")?.focus()}));
        }
        if self.part == Part::Button {
            base = base
                .set("data-variant", self.variant.slug())
                .set("data-size", self.size.slug())
                // Before the caller's spread, so `.kind("submit")` still wins.
                // Without it a button inside a form submits it, which is never
                // what a clear-field or reveal-password button meant.
                .set("type", "button");
        }

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the part is styled on arrival,
        // JavaScript or no JavaScript.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[
                    &part_classes(self.part, self.align, self.size, self.variant),
                    &self.class,
                ]),
            )
        };

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the part in that block's scope,
    /// which is what a binding written here needs.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. Four of the six parts take one inside
    /// the element; the two controls cannot, and pay a fragment for it. The
    /// module docs say what that costs and what to reach for instead.
    pub fn render(&self, children: Html) -> Html {
        let scope = self.needs_scope();
        let attrs = self.attr_list();

        match (self.part, scope) {
            (Part::Group, false) => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
            (Part::Group, true) => html! {
                <div @{..attrs}>
                    // Present so this block is mounted, and inside the element
                    // rather than beside it. See the module docs.
                    <script>r##""##</script>
                    <slot />
                </div>
            },
            (Part::Addon, false) => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
            (Part::Addon, true) => html! {
                <div @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </div>
            },
            (Part::Button, false) => html! {
                <button @{..attrs}>
                    <slot />
                </button>
            },
            (Part::Button, true) => html! {
                <button @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </button>
            },
            (Part::Text, false) => html! {
                <span @{..attrs}>
                    <slot />
                </span>
            },
            (Part::Text, true) => html! {
                <span @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </span>
            },
            (Part::Input, false) => html! {
                <input @{..attrs} />
            },
            (Part::Input, true) => {
                // An `<input>` is void, so the script cannot go inside it. See
                // the module docs for what the fragment costs.
                let control = html! {
                    <input @{..attrs} />
                };
                html! {
                    <>
                        @{control}
                        <script>r##""##</script>
                    </>
                }
            }
            (Part::Textarea, false) => html! {
                <textarea @{..attrs}>
                    <slot />
                </textarea>
            },
            (Part::Textarea, true) => {
                // Anything inside a `<textarea>` is its value, so a script in
                // there would be text the user has to delete.
                let control = html! {
                    <textarea @{..attrs}>
                        <slot />
                    </textarea>
                };
                html! {
                    <>
                        @{control}
                        <script>r##""##</script>
                    </>
                }
            }
        }
    }
}

/// The bordered row an Input Group draws.
///
/// ```ignore
/// <InputGroup>
///     <InputGroupInput placeholder="Search…" />
///     <InputGroupAddon>@{search_icon}</InputGroupAddon>
/// </InputGroup>
/// ```
///
/// Write the addon **after** the control, whichever side it draws on — Tab
/// follows the DOM, and `align` moves the paint. Everything the group shows
/// about state — focus, invalid, disabled, and whether it is a row or a column
/// — it works out from what it was given. Nothing is passed down.
#[component]
pub fn InputGroup(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let mut props = input_group().class(class);

    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all.
    if !id.is_empty() {
        props = props.id(id);
    }

    props = props.attrs(attrs);

    // `onclick={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the element unchanged.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
            </div>
        }
    } else {
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// What sits beside the control: an icon, a label, a unit, a button, a
/// shortcut.
///
/// ```ignore
/// <InputGroupAddon>@{search_icon}</InputGroupAddon>
/// <InputGroupAddon align="inline-end"><InputGroupText>"USD"</InputGroupText></InputGroupAddon>
/// <InputGroupAddon align="block-end">…a toolbar under a textarea…</InputGroupAddon>
/// ```
///
/// `inline-start` and `inline-end` belong with an [`InputGroupInput`];
/// `block-start` and `block-end` with an [`InputGroupTextarea`], and they turn
/// the group into a column.
///
/// Clicking one puts the caret in the field — that is [`FOCUS_CONTROL`], a
/// plain inline handler, and it is what `cursor-text` promises. A click that
/// landed on a button is left to the button. Give it up with
/// `attrs=@{Attrs::new().unset("onclick")}`.
#[component]
pub fn InputGroupAddon(
    class: &str,
    align: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = input_group_addon()
        .class(class)
        .align(InputGroupAlign::parse(align));

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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
            </div>
        }
    } else {
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// A button sized to sit inside a field.
///
/// ```ignore
/// <InputGroupButton onclick={clear()}>"Clear"</InputGroupButton>
/// <InputGroupButton size="icon-xs" attrs=@{Attrs::new().set("aria-label", "Copy")}>
///     @{copy_icon}
/// </InputGroupButton>
/// ```
///
/// `ghost` and `xs` unless said otherwise, and `type="button"` always unless
/// the caller writes another — a clear-field button that submitted the form
/// around it would be a bug in every case.
///
/// Put it inside an [`InputGroupAddon`]: that is what the addon's
/// `has-[>button]:ml-[-0.3rem]` is fitting, and what its click handler steps
/// aside for.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn InputGroupButton(
    class: &str,
    variant: &str,
    size: &str,
    r#type: &str,
    id: &str,
    disabled: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = input_group_button()
        .class(class)
        .size(InputGroupButtonSize::parse(size))
        .disabled(disabled);

    // An unwritten prop arrives as `""`, and `ButtonVariant::parse("")` is
    // `Default` — a Button's default, not an input-group button's. Writing it
    // unconditionally would make every unstyled button primary. `size` and
    // `align` do not need this: their `parse("")` already is their default.
    if !variant.is_empty() {
        props = props.variant(ButtonVariant::parse(variant));
    }

    if !r#type.is_empty() {
        props = props.kind(r#type);
    }
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <button @{..attrs}>
                <script>r##""##</script>
                <slot />
            </button>
        }
    } else {
        html! {
            <button @{..attrs}>
                <slot />
            </button>
        }
    }
}

/// Muted text inside an addon — a unit, a prefix, a hint.
///
/// ```ignore
/// <InputGroupAddon align="inline-end"><InputGroupText>"USD"</InputGroupText></InputGroupAddon>
/// ```
///
/// No `data-slot`, which is shadcn's: the group has nothing to ask about a run
/// of text.
#[component]
pub fn InputGroupText(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = input_group_text().class(class);

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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <span @{..attrs}>
                <script>r##""##</script>
                <slot />
            </span>
        }
    } else {
        html! {
            <span @{..attrs}>
                <slot />
            </span>
        }
    }
}

/// The control, stripped of its own chrome.
///
/// ```ignore
/// <InputGroupInput placeholder="Search…" />
/// <InputGroupInput r#type="email" name="email" required invalid />
/// ```
///
/// An [`Input`](super::input) without its border, rounding, shadow or focus
/// ring — the group draws all four, around the whole row. What it keeps is
/// everything about the text.
///
/// `invalid` writes `aria-invalid="true"`, which is what reddens the group and
/// what a screen reader is told. `r#type` is spelled the way Rust makes it
/// spellable; the rendered attribute is `type`.
///
/// An override on this is the one case in the family that costs something —
/// see the module docs, and reach for [`input_group_input_variants`].
#[component]
#[allow(clippy::too_many_arguments)]
pub fn InputGroupInput(
    class: &str,
    r#type: &str,
    name: &str,
    id: &str,
    placeholder: &str,
    value: &str,
    disabled: bool,
    required: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
) -> Html {
    let mut props = input_group_input()
        .class(class)
        .disabled(disabled)
        .required(required)
        .invalid(invalid);

    if !r#type.is_empty() {
        props = props.kind(r#type);
    }
    if !name.is_empty() {
        props = props.name(name);
    }
    if !id.is_empty() {
        props = props.id(id);
    }
    if !placeholder.is_empty() {
        props = props.placeholder(placeholder);
    }
    if !value.is_empty() {
        props = props.value(value);
    }

    props = props.attrs(attrs);

    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // A caller's `{…}` props are the caller's expressions: rendered
    // script-less, the block is compiled in the caller's scope, where they
    // read its state. Only a class override with nothing of the caller's needs
    // a script for its `{twMerge(…)}` — and an `<input>` is void, so the script
    // cannot go inside it and the block has to be a fragment. The module docs
    // say what that costs.
    if scope && bindings.is_empty() {
        html! {
            <>
                <input @{..attrs} />
                <script>r##""##</script>
            </>
        }
    } else {
        html! {
            <input @{..attrs} />
        }
    }
}

/// The same, for many lines.
///
/// ```ignore
/// <InputGroup>
///     <InputGroupTextarea placeholder="Tell us what happened…" />
///     <InputGroupAddon align="block-end">
///         <InputGroupText>"Markdown supported"</InputGroupText>
///     </InputGroupAddon>
/// </InputGroup>
/// ```
///
/// `resize-none`, because the group draws the border and a corner grip would
/// drag the textarea out of it. The group grows instead —
/// `field-sizing-content` on the textarea and `has-[>textarea]:h-auto` on the
/// group, working together.
///
/// Pair it with a `block-start` or `block-end` addon. An inline one would sit
/// beside a box that is many lines tall.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn InputGroupTextarea(
    class: &str,
    name: &str,
    id: &str,
    placeholder: &str,
    rows: &str,
    disabled: bool,
    required: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = input_group_textarea()
        .class(class)
        .disabled(disabled)
        .required(required)
        .invalid(invalid);

    if !name.is_empty() {
        props = props.name(name);
    }
    if !id.is_empty() {
        props = props.id(id);
    }
    if !placeholder.is_empty() {
        props = props.placeholder(placeholder);
    }
    if !rows.is_empty() {
        props = props.attr("rows", rows);
    }

    props = props.attrs(attrs);

    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // As for InputGroupInput: a caller's binding renders script-less, so it is
    // compiled in the caller's scope. Anything inside a `<textarea>` is its
    // value, so a class override's script goes beside it and the block is a
    // fragment. See the module docs.
    if scope && bindings.is_empty() {
        html! {
            <>
                <textarea @{..attrs}>
                    <slot />
                </textarea>
                <script>r##""##</script>
            </>
        }
    } else {
        html! {
            <textarea @{..attrs}>
                <slot />
            </textarea>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, ButtonVariant, FOCUS_CONTROL, InputGroup, InputGroupAddon, InputGroupAlign,
        InputGroupButton, InputGroupButtonSize, InputGroupInput, InputGroupProps,
        InputGroupTextarea, input_group, input_group_addon, input_group_button,
        input_group_button_variants, input_group_input, input_group_text, input_group_textarea,
        input_group_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_group_carries_shadcns_defaults() {
        let markup = input_group().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"role="group""#), "{markup}");
        assert!(markup.contains(r#"data-slot="input-group""#), "{markup}");
        assert!(markup.contains("group/input-group"), "{markup}");
        assert!(markup.contains("border-input"), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// Each of the six renders the element shadcn renders, under the
    /// `data-slot` the group's own selectors look for.
    #[test]
    fn every_part_is_the_element_shadcn_renders() {
        let cases: [(InputGroupProps, &str, Option<&str>); 6] = [
            (input_group(), "<div ", Some(r#"data-slot="input-group""#)),
            (
                input_group_addon(),
                "<div ",
                Some(r#"data-slot="input-group-addon""#),
            ),
            (
                input_group_button(),
                "<button ",
                Some(r#"data-slot="button""#),
            ),
            // shadcn gives its text span no slot, and neither does this.
            (input_group_text(), "<span ", None),
            (
                input_group_input(),
                "<input ",
                Some(r#"data-slot="input-group-control""#),
            ),
            (
                input_group_textarea(),
                "<textarea ",
                Some(r#"data-slot="input-group-control""#),
            ),
        ];

        for (props, tag, slot) in cases {
            let markup = props.render(Html::empty()).into_test_string();
            assert!(markup.starts_with(tag), "{tag}: {markup}");
            match slot {
                Some(slot) => assert!(markup.contains(slot), "{slot}: {markup}"),
                None => assert!(!markup.contains("data-slot"), "{markup}"),
            }
        }
    }

    /// The group asks about its children rather than being told about them.
    /// These four selectors are the whole state model.
    #[test]
    fn the_group_reads_its_own_contents() {
        let markup = input_group().render(Html::empty()).into_test_string();

        assert!(
            markup.contains("has-[[data-slot=input-group-control]:focus-visible]:ring-3"),
            "{markup}"
        );
        assert!(
            markup.contains("has-[[data-slot][aria-invalid=true]]:border-destructive"),
            "{markup}"
        );
        assert!(markup.contains("has-disabled:bg-input/50"), "{markup}");
        assert!(markup.contains("has-[&gt;textarea]:h-auto"), "{markup}");
    }

    /// The one adaptation the module docs argue for: this library's control
    /// height and radius, not shadcn's Input's.
    #[test]
    fn the_group_is_this_librarys_control_size() {
        use crate::components::rahti_ui::input::input_variants;

        let group = input_group().class_list();
        let input = input_variants("");

        for metric in ["h-9", "rounded-md"] {
            assert!(group.split_whitespace().any(|u| u == metric), "{group}");
            assert!(
                input.split_whitespace().any(|u| u == metric),
                "the library moved: {input}"
            );
        }
        assert!(!group.contains("h-8 "), "{group}");
        assert!(!group.contains("rounded-lg"), "{group}");
    }

    #[test]
    fn an_addon_is_aligned_by_an_attribute_and_a_layer() {
        for (align, layer, attr) in [
            (InputGroupAlign::InlineStart, "order-first", "inline-start"),
            (InputGroupAlign::InlineEnd, "order-last", "inline-end"),
            (InputGroupAlign::BlockStart, "order-first", "block-start"),
            (InputGroupAlign::BlockEnd, "order-last", "block-end"),
        ] {
            let markup = input_group_addon()
                .align(align)
                .render(Html::empty())
                .into_test_string();

            assert!(
                markup.contains(&format!(r#"data-align="{attr}""#)),
                "{markup}"
            );
            assert!(markup.contains(layer), "{markup}");
        }

        // The two block alignments replace `justify-center`; the inline ones
        // are where it survives.
        let inline = input_group_addon().class_list();
        let block = input_group_addon()
            .align(InputGroupAlign::BlockStart)
            .class_list();
        assert!(inline.contains("justify-center"), "{inline}");
        assert!(!block.contains("justify-center"), "{block}");
        assert!(block.contains("justify-start"), "{block}");

        assert_eq!(InputGroupAlign::parse(""), InputGroupAlign::InlineStart);
        assert_eq!(
            InputGroupAlign::parse("block-end"),
            InputGroupAlign::BlockEnd
        );
    }

    /// shadcn's React handler, as the inline one HTML already has — and not a
    /// binding, so an addon never needs a scope for it.
    #[test]
    fn an_addon_focuses_the_control_without_the_runtime() {
        let props = input_group_addon();
        let markup = props.render(Html::empty()).into_test_string();

        assert!(markup.contains("pp-onclick="), "{markup}");
        assert!(markup.contains("closest("), "{markup}");
        assert!(!props.needs_scope());
        assert!(!FOCUS_CONTROL.contains('{'), "{FOCUS_CONTROL}");
        assert!(!markup.contains("<script"), "{markup}");

        // And it can be given up.
        let bare = input_group_addon()
            .attrs(Attrs::new().unset("onclick"))
            .render(Html::empty())
            .into_test_string();
        assert!(!bare.contains("onclick"), "{bare}");
    }

    /// A button inside a field must never submit the form around it by
    /// accident, and must never cast a shadow onto the field.
    #[test]
    fn a_button_is_a_ghost_that_does_not_submit() {
        let markup = input_group_button()
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"type="button""#), "{markup}");
        assert!(markup.contains(r#"data-variant="ghost""#), "{markup}");
        assert!(markup.contains(r#"data-size="xs""#), "{markup}");
        assert!(markup.contains("shadow-none"), "{markup}");

        // Through the tag too. An unwritten `variant` is `""`, and
        // `ButtonVariant::parse("")` is `Default` — writing it unconditionally
        // would make every unstyled button in every group primary.
        let tag = InputGroupButton(
            "",
            "",
            "",
            "",
            "",
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Apply"),
        )
        .into_test_string();
        assert!(tag.contains(r#"data-variant="ghost""#), "{tag}");
        assert!(!tag.contains("bg-primary"), "{tag}");
        assert!(tag.contains("hover:bg-accent"), "{tag}");

        // The caller can still ask for a submit button.
        let submit = input_group_button()
            .kind("submit")
            .render(Html::empty())
            .into_test_string();
        assert!(submit.contains(r#"type="submit""#), "{submit}");

        // An Outline button's own shadow is the one thing `shadow-none` had to
        // take away, and a join that kept both would be a contradiction.
        let outline = input_group_button()
            .variant(ButtonVariant::Outline)
            .class_list();
        assert!(outline.contains("shadow-none"), "{outline}");
        assert!(!outline.contains("shadow-xs"), "{outline}");
        assert!(outline.contains("bg-background"), "{outline}");
    }

    /// A Button with the input group's layer on top, and the `inline-flex` and
    /// `rounded-md` a tailwind-merge would have resolved.
    #[test]
    fn the_button_merge_is_shadcns() {
        use crate::components::rahti_ui::button::{ButtonSize, button_variants};

        let merged = input_group_button().class_list();
        let button = button_variants(ButtonVariant::Ghost, ButtonSize::Default, "");
        let has = |list: &str, utility: &str| list.split_whitespace().any(|u| u == utility);

        // `inline-flex` becomes `flex`, and the radius moves into the size.
        assert!(!has(&merged, "inline-flex"), "{merged}");
        assert!(has(&merged, "flex"), "{merged}");
        assert!(has(&button, "inline-flex"), "the library moved: {button}");
        assert!(has(&button, "rounded-md"), "the library moved: {button}");
        assert!(
            merged.contains("rounded-[calc(var(--radius)-3px)]"),
            "{merged}"
        );

        // Everything the Button contributes that is not chrome survives.
        for utility in ["shrink-0", "whitespace-nowrap", "focus-visible:border-ring"] {
            assert!(has(&merged, utility), "`{utility}` in {merged}");
        }

        // The `sm` size is this library's h-8 button, radius included.
        let sm = input_group_button()
            .size(InputGroupButtonSize::Sm)
            .class_list();
        assert!(has(&sm, "h-8"), "{sm}");
        assert!(has(&sm, "rounded-md"), "{sm}");

        assert_eq!(InputGroupButtonSize::parse(""), InputGroupButtonSize::Xs);
        assert_eq!(
            InputGroupButtonSize::parse("icon-sm"),
            InputGroupButtonSize::IconSm
        );
    }

    /// The control gives up exactly the chrome the group took over, and keeps
    /// everything about the text.
    #[test]
    fn a_control_gives_up_its_chrome_and_keeps_its_text() {
        use crate::components::rahti_ui::input::input_variants;

        let control = input_group_input().class_list();
        let input = input_variants("");
        let has = |list: &str, utility: &str| list.split_whitespace().any(|u| u == utility);

        for gone in [
            "border",
            "shadow-xs",
            "focus-visible:ring-[3px]",
            "dark:bg-input/30",
        ] {
            assert!(has(&input, gone), "the library moved: {input}");
            assert!(!has(&control, gone), "`{gone}` survived: {control}");
        }
        // `rounded-md` is replaced rather than dropped.
        assert!(!has(&control, "rounded-md"), "{control}");
        assert!(has(&control, "rounded-none"), "{control}");

        for kept in ["px-3", "placeholder:text-muted-foreground", "md:text-sm"] {
            assert!(has(&control, kept), "`{kept}` in {control}");
        }
        // The height is given up too, and replaced. See `CONTROL`: an `h-9`
        // control inside an `h-9` group that draws the border is taller than
        // the box it sits in, and its square corners hang over the rounded ones.
        assert!(
            !has(&control, "h-9"),
            "the control kept its own height: {control}"
        );
        assert!(has(&control, "h-full"), "{control}");
        // The ring is the group's now, but the ring *colour* is not a width.
        assert!(has(&control, "focus-visible:ring-0"), "{control}");
        assert!(has(&control, "focus-visible:ring-ring/50"), "{control}");

        // A textarea gives up the same five and adds the one of its own.
        let textarea = input_group_textarea().class_list();
        assert!(has(&textarea, "resize-none"), "{textarea}");
        assert!(has(&textarea, "field-sizing-content"), "{textarea}");
        assert!(!has(&textarea, "shadow-xs"), "{textarea}");
    }

    #[test]
    fn invalid_is_written_on_the_control_where_the_group_can_read_it() {
        let markup = input_group_input()
            .invalid(true)
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"aria-invalid="true""#), "{markup}");

        // `false` is the attribute's absence, not `="false"`. The class list
        // still says `aria-invalid:` — those are the utilities waiting for it.
        let plain = input_group_input()
            .invalid(false)
            .render(Html::empty())
            .into_test_string();
        assert!(!plain.contains(r#"aria-invalid=""#), "{plain}");
        assert!(plain.contains("aria-invalid:ring-0"), "{plain}");
    }

    /// The structural rule: the four parts that can hold a script are one
    /// literal element whatever the caller asked for.
    #[test]
    fn an_override_never_wraps_the_four_parts_that_can_hold_a_script() {
        for (props, open, close) in [
            (input_group().class("max-w-xs"), "<div ", "</div>"),
            (input_group_addon().class("pl-3"), "<div ", "</div>"),
            (
                input_group_button().class("text-primary"),
                "<button ",
                "</button>",
            ),
            (input_group_text().class("font-mono"), "<span ", "</span>"),
        ] {
            let markup = props.render(Html::empty()).into_test_string();
            assert!(markup.starts_with(open), "{markup}");
            assert!(markup.ends_with(close), "{markup}");
            assert!(!markup.contains("{twMerge("), "{markup}");
        }
    }

    /// And the two that cannot say so plainly rather than pretending.
    #[test]
    fn an_override_on_a_control_is_resolved_without_a_fragment() {
        let input = input_group_input().class("text-center");
        let textarea = input_group_textarea().class("font-mono");
        let input_markup = input.render(Html::empty()).into_test_string();
        let textarea_markup = textarea.render(Html::empty()).into_test_string();
        assert!(input_markup.starts_with("<input "), "{input_markup}");
        assert!(
            textarea_markup.starts_with("<textarea "),
            "{textarea_markup}"
        );
        assert!(input_markup.contains("text-center"), "{input_markup}");
        assert!(textarea_markup.contains("font-mono"), "{textarea_markup}");
    }

    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = input_group()
            .class("h-12")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("h-12"), "{markup}");
    }

    #[test]
    fn a_part_with_no_override_needs_no_runtime() {
        let props = input_group().id("search");
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
        assert!(markup.contains(r#"class="group/input-group"#), "{markup}");

        assert!(!input_group().class("max-w-xs").needs_scope());
        assert!(
            input_group_input()
                .bind(
                    "value",
                    crate::components::rahti_ui::attrs::ui_expr!(value = { query })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = input_group_input()
            .on(
                "input",
                crate::components::rahti_ui::attrs::ui_handler!({ setQuery(target.value) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"pp-oninput="setQuery(target.value)""#),
            "{markup}"
        );
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = input_group_button()
            .attr("data-slot", "mine")
            .attr("aria-label", "Copy")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"aria-label="Copy""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = input_group_input()
            .placeholder(r#"a" pp-onclick="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onclick=\"steal"), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn the_variants_functions_are_the_components_own_class_lists() {
        assert_eq!(
            input_group_variants("max-w-xs"),
            input_group().class("max-w-xs").class_list()
        );
        assert_eq!(
            input_group_button_variants(
                ButtonVariant::Outline,
                InputGroupButtonSize::IconSm,
                "ml-auto"
            ),
            input_group_button()
                .variant(ButtonVariant::Outline)
                .size(InputGroupButtonSize::IconSm)
                .class("ml-auto")
                .class_list()
        );
    }

    /// The call form behind `<InputGroupAddon align="inline-end">`.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let group = InputGroup(
            "",
            "search",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("x"),
        )
        .into_test_string();
        assert!(group.contains(r#"id="search""#), "{group}");

        let addon = InputGroupAddon(
            "",
            "inline-end",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("USD"),
        )
        .into_test_string();
        assert!(addon.contains(r#"data-align="inline-end""#), "{addon}");
        assert!(addon.contains("order-last"), "{addon}");

        let button = InputGroupButton(
            "",
            "outline",
            "icon-sm",
            "",
            "",
            true,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(button.contains(r#"data-size="icon-sm""#), "{button}");
        assert!(button.contains(r#"disabled="""#), "{button}");
        assert!(button.contains(r#"data-variant="outline""#), "{button}");

        let control = InputGroupInput(
            "",
            "email",
            "email",
            "",
            "you@example.com",
            "",
            false,
            true,
            true,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();
        assert!(control.contains(r#"type="email""#), "{control}");
        assert!(control.contains(r#"required="""#), "{control}");
        assert!(control.contains(r#"aria-invalid="true""#), "{control}");

        let area = InputGroupTextarea(
            "",
            "",
            "",
            "Tell us…",
            "3",
            false,
            false,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(area.contains(r#"rows="3""#), "{area}");
        assert!(area.starts_with("<textarea "), "{area}");
    }

    /// Every prop left unwritten, which is what `<InputGroupInput />` compiles
    /// to now that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_part() {
        let markup = InputGroupInput(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"data-slot="input-group-control""#),
            "{markup}"
        );
        assert!(markup.contains("rounded-none"), "{markup}");
        // Nothing optional leaked an empty attribute.
        for empty in [r#"type="""#, r#"name="""#, r#"id="""#, r#"placeholder="""#] {
            assert!(!markup.contains(empty), "{empty}: {markup}");
        }
    }

    /// `InputGroupProps` is `Default`, which is what lets a tag omit every
    /// prop — and the default part is the group itself.
    #[test]
    fn props_default_is_a_plain_group() {
        assert_eq!(InputGroupProps::new(), InputGroupProps::default());
        assert_eq!(
            InputGroupProps::default()
                .render(Html::empty())
                .into_test_string(),
            InputGroupProps::new()
                .render(Html::empty())
                .into_test_string()
        );
    }
}

/// The layer-independence check behind every class string in this file.
///
/// A part builds its class list by *joining* layers rather than merging them,
/// which is only exact if no two utilities in the result compete. This proves
/// it for all six parts, all four addon alignments, and all 24 button
/// variant×size pairs.
///
/// The button is where it earns its keep. Three layers meet there — a Button's
/// base, its variant, and the input group's own — and each of the three
/// contradictions between them (`inline-flex`/`flex`, `rounded-md`/the size's
/// radius, `shadow-xs`/`shadow-none`) is resolved by a constant or by
/// [`variant_kept`] rather than left for the browser.
#[cfg(test)]
mod audit {
    use super::{ButtonVariant, InputGroupAlign, InputGroupButtonSize, Part, part_classes};
    use crate::components::rahti_ui::utils::audit::clashes;

    const ALIGNS: [InputGroupAlign; 4] = [
        InputGroupAlign::InlineStart,
        InputGroupAlign::InlineEnd,
        InputGroupAlign::BlockStart,
        InputGroupAlign::BlockEnd,
    ];

    const SIZES: [InputGroupButtonSize; 4] = [
        InputGroupButtonSize::Xs,
        InputGroupButtonSize::Sm,
        InputGroupButtonSize::IconXs,
        InputGroupButtonSize::IconSm,
    ];

    const VARIANTS: [ButtonVariant; 6] = [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ];

    #[test]
    fn no_layer_contradicts_another() {
        for part in [Part::Group, Part::Text, Part::Input, Part::Textarea] {
            let list = part_classes(
                part,
                InputGroupAlign::InlineStart,
                InputGroupButtonSize::Xs,
                ButtonVariant::Ghost,
            );
            let found = clashes(&list);
            assert!(
                found.is_empty(),
                "{:?}: {} clashes:\n{}",
                part,
                found.len(),
                found.join("\n")
            );
        }

        for align in ALIGNS {
            let list = part_classes(
                Part::Addon,
                align,
                InputGroupButtonSize::Xs,
                ButtonVariant::Ghost,
            );
            let found = clashes(&list);
            assert!(
                found.is_empty(),
                "addon {}: {} clashes:\n{}",
                align.slug(),
                found.len(),
                found.join("\n")
            );
        }

        for size in SIZES {
            for variant in VARIANTS {
                let list = part_classes(Part::Button, InputGroupAlign::InlineStart, size, variant);
                let found = clashes(&list);
                assert!(
                    found.is_empty(),
                    "button {}×{}: {} clashes:\n{}",
                    variant.slug(),
                    size.slug(),
                    found.len(),
                    found.join("\n")
                );
            }
        }
    }
}
