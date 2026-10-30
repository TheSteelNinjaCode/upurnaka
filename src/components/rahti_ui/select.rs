// rahti-ui: select
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Select — a port of shadcn/ui's `select`.
//!
//! shadcn ships ten exports here. This file ships five, and the difference is
//! the whole story of the port: a native `<select>` *is* Radix's Root, Trigger,
//! Value, Portal, Content, Viewport and both ScrollButtons, and the browser
//! renders the part of it that shadcn's ten exports exist to build.
//!
//! | shadcn | here | |
//! | --- | --- | --- |
//! | `Select` + `SelectTrigger` + `SelectValue` | [`Select`] | one `<select>` |
//! | `SelectContent` + `SelectViewport` + `SelectPortal` | — | the browser's popup |
//! | `SelectItem` | [`SelectItem`] | `<option>` |
//! | `SelectGroup` + `SelectLabel` | [`SelectGroup`] | `<optgroup label="…">` |
//! | `SelectSeparator` | [`SelectSeparator`] | `<hr>` |
//! | `SelectScrollUpButton` / `SelectScrollDownButton` | — | the browser scrolls |
//! | `<SelectValue placeholder="…" />` | [`SelectPlaceholder`] | a first `<option>` |
//!
//! ## What the trade is
//!
//! This is the first component where the native bet costs something a caller can
//! see, so it is worth stating both halves plainly.
//!
//! **What is given up.** The open popup is the browser's, and it cannot be
//! styled. There is no `focus:bg-accent` on the hovered item, no check mark
//! beside the chosen one, no `rounded-md border bg-popover shadow-md` around the
//! list, no open/close animation, and no icons inside an option. shadcn's
//! `SelectContent` and `SelectItem` class lists have no native home and are not
//! ported — an `<option>` here carries `data-slot` and nothing else.
//!
//! **What is gained.** On a phone the popup is the OS picker rather than a div
//! that has to be told about touch, momentum and the keyboard inset. Typeahead
//! works. The list cannot be clipped by an ancestor's `overflow`, needs no
//! portal, no z-index, no scroll lock and no focus trap. It renders with no
//! JavaScript, and it is read correctly by every screen reader without an ARIA
//! attribute being written. The dark popup comes from `color-scheme: dark`,
//! which `globals.css` already sets.
//!
//! The closed control — the part shadcn calls the Trigger, and the only part
//! anyone can style — is pixel-identical to shadcn's, because it is shadcn's
//! class list.
//!
//! **When this is the wrong component.** If the list itself has to be styled —
//! icons per row, a check mark, a description under each label, a search box —
//! then the answer is not a `<select>` in either library. That is a Dropdown
//! Menu or a Combobox: a button and a popover, built out of elements that can
//! hold anything, and both are their own components.
//!
//! ## The deltas, spelled out
//!
//! 1. **`appearance-none`, and the chevron becomes a background image.** shadcn
//!    renders a `<ChevronDownIcon className="size-4 opacity-50" />` inside the
//!    Trigger. A `<select>` may contain only options, so the chevron is lucide's
//!    `chevron-down` drawn as a `background-image` — same path, same 16px, same
//!    half opacity — and the UA's own arrow is turned off. `pr-9` is the room it
//!    needs: shadcn's `px-3` plus `gap-2` plus a 16px icon.
//! 2. **`data-[placeholder]:` becomes `has-[option[data-placeholder]:checked]:`.**
//!    Radix marks the Trigger when its Value is empty. A `<select>` has no such
//!    attribute, but it does have the option that is currently chosen, and CSS
//!    can ask about it. [`SelectPlaceholder`] writes the attribute the selector
//!    looks for. Same declaration, native question.
//! 3. **Four groups of utilities are dropped**, each because the element they
//!    select cannot exist here: `flex items-center justify-between gap-2` (a
//!    replaced element lays out its own text), `*:data-[slot=select-value]:*`
//!    (there is no Value child), `[&_svg]:*` (there are no svg children), and
//!    every class on Content, Item, Label and Separator.
//! 4. **The selection lives on the option**, as `selected`, not on the group as
//!    `value` — the same move [`RadioGroup`](super::radio_group) makes, and for
//!    the same reason: children arrive already rendered, so a parent cannot
//!    reach into them.
//!
//! Everything else — every colour, every ring, every `dark:` — is shadcn's,
//! copied rather than reinterpreted.
//!
//! ## The call site is the JSX one, one element shallower
//!
//! ```ignore
//! <Select name="fruit">
//!     <SelectPlaceholder>"Select a fruit"</SelectPlaceholder>
//!     <SelectGroup label="Fruits">
//!         <SelectItem value="apple">"Apple"</SelectItem>
//!         <SelectItem value="banana" selected>"Banana"</SelectItem>
//!         <SelectItem value="durian" disabled>"Durian"</SelectItem>
//!     </SelectGroup>
//! </Select>
//! ```
//!
//! There is no Trigger and no Content to nest: shadcn needs them because the
//! button and the list are two elements in two places, and here they are one
//! element the browser draws twice.
//!
//! ## The control is UI. The state is the caller's.
//!
//! `onchange={…}` and `value={…}` are native attributes, and this component
//! does nothing with them but put them on the `<select>`. As
//! [`Input`](super::input) describes, a component's root attributes are
//! evaluated where the tag was written, so `<Select value={fruit}
//! onchange={setFruit(target.value)}>` is a select the page drives.
//!
//! ## When the tag is not enough
//!
//! **[`SelectProps::render`], interpolated — for a select built in Rust.**
//!
//! ```ignore
//! @{select_props()
//!     .name("fruit")
//!     .bind("value", crate::components::rahti_ui::attrs::ui_expr!(value={fruit}))
//!     .on("change", crate::components::rahti_ui::attrs::ui_handler!({setFruit(target.value)}))
//!     .render(options)}
//! ```
//!
//! **[`select_variants`] — the class list alone**, for a `<select>` whose
//! options are the page's state, since a `{…}` written among children is
//! compiled in the block it was written in:
//!
//! ```ignore
//! <select class=@{select_variants(SelectSize::Default, "")} data-size="default">
//!     <template pp-for="f in fruits"><option key="{f}" value="{f}">{f}</option></template>
//! </select>
//! ```

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// Everything the closed control is, minus the chevron.
///
/// shadcn's Trigger string, with the four dropped groups named in the module
/// docs left out and `appearance-none`, `bg-no-repeat` and `pr-9` added.
///
/// `data-[size=default]:h-9 data-[size=sm]:h-8` is shadcn's own way of spelling
/// the two heights — the size is a data attribute the class list reads, not a
/// second layer of classes — so [`SelectSize`] writes `data-size` and both
/// utilities stay here under different prefixes. Nothing to merge.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const BASE: &str = "w-fit appearance-none rounded-md border border-input bg-transparent \
                    bg-no-repeat py-2 pr-9 pl-3 text-sm whitespace-nowrap shadow-xs \
                    transition-[color,box-shadow] outline-none \
                    focus-visible:border-ring focus-visible:ring-[3px] \
                    focus-visible:ring-ring/50 \
                    disabled:cursor-not-allowed disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                    has-[option[data-placeholder]:checked]:text-muted-foreground \
                    data-[size=default]:h-9 data-[size=sm]:h-8 \
                    dark:bg-input/30 dark:hover:bg-input/50 \
                    dark:aria-invalid:ring-destructive/40";

/// The chevron: lucide's `chevron-down`, the icon shadcn's Trigger renders, as
/// an image instead of an element.
///
/// The path, the 24-unit box and the stroke width are lucide's; `width`/`height`
/// are baked into the SVG at 16px so shadcn's `size-4` needs no
/// `background-size` utility to go with it, and `opacity='0.5'` is its
/// `opacity-50`.
///
/// Two of them, because a data URI cannot read a CSS variable — the same tax
/// [`Checkbox`](super::checkbox)'s tick pays, and the reason
/// [`RadioGroup`](super::radio_group)'s dot is a gradient instead. A chevron is
/// not a gradient, so `--foreground` is spelled out for each theme.
///
/// The two rules that govern how these are written, both learned by breaking
/// them: **one utility per source line**, because Tailwind scans source text and
/// would not find a class name broken across a `\` continuation; and **spaces
/// are `%20`, not `_`**, because Tailwind's underscore-to-space substitution is
/// switched off inside `url()`.
const CHEVRON: &str = "bg-[position:right_0.75rem_center] \
                       bg-[url('data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2716%27%20height=%2716%27%20viewBox=%270%200%2024%2024%27%20fill=%27none%27%20stroke=%27%230a0a0a%27%20stroke-width=%272%27%20stroke-linecap=%27round%27%20stroke-linejoin=%27round%27%20opacity=%270.5%27%3e%3cpath%20d=%27m6%209%206%206%206-6%27/%3e%3c/svg%3e')] \
                       dark:bg-[url('data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2716%27%20height=%2716%27%20viewBox=%270%200%2024%2024%27%20fill=%27none%27%20stroke=%27%23fafafa%27%20stroke-width=%272%27%20stroke-linecap=%27round%27%20stroke-linejoin=%27round%27%20opacity=%270.5%27%3e%3cpath%20d=%27m6%209%206%206%206-6%27/%3e%3c/svg%3e')]";

/// Every utility the control carries, before the caller's own.
fn component_classes() -> String {
    cn(&[BASE, CHEVRON])
}

/// How tall the closed control is.
///
/// shadcn's two sizes, kept the way shadcn keeps them: a `data-size` attribute
/// that [`BASE`] reads, rather than a class layer of its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    #[default]
    Default,
    Sm,
}

impl SelectSize {
    /// The name shadcn writes into `data-size`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `size="…"` prop.
    ///
    /// An empty string is the default rather than an error, because that is what
    /// an unwritten prop arrives as. Anything else unrecognised is a mistake at
    /// the call site and trips a `debug_assert!` — the same rule
    /// [`ButtonSize::parse`](super::button::ButtonSize::parse) applies.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "sm" => Self::Sm,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Select size. One of: default, sm."
                );
                Self::Default
            }
        }
    }
}

/// The class list shadcn's `cn(…, className)` returns: the control's own
/// utilities, then the caller's, merged so the caller's win.
///
/// The size is not in here — it is `data-size`, which the caller has to write
/// alongside — so it is taken as an argument only to make that impossible to
/// forget. Reach for this when the element has to belong to the page: a select
/// whose options are the page's state, or whose handler reads it.
pub fn select_variants(size: SelectSize, class: &str) -> String {
    let _ = size;
    cn(&[&component_classes(), class])
}

/// A select's props, with shadcn's defaults already in place.
///
/// Build one with [`select_props`] and narrow it down a method at a time.
/// Nothing here is required.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectProps {
    class: String,
    size: SelectSize,
    attrs: Attrs,
}

/// A default select's props — the start of every chain.
///
/// Named `select_props` rather than `select` because `select` is what the
/// rendered element is called, and a helper that returns props is not it.
pub fn select_props() -> SelectProps {
    SelectProps::default()
}

impl SelectProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the control's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// `default` or `sm`, written as `data-size` for [`BASE`] to read.
    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    pub fn name(self, name: &str) -> Self {
        self.attr("name", name)
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.flag("disabled")
        } else {
            self
        }
    }

    /// Required — which, with a [`SelectPlaceholder`] first in the list, is what
    /// stops a form being submitted before a real choice is made. The
    /// placeholder's value is empty, and an empty value fails `required`.
    pub fn required(self, required: bool) -> Self {
        if required {
            self.flag("required")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` — which is what the `aria-invalid:` utilities in
    /// [`BASE`] paint the control with. The styling and the accessible state are
    /// the same fact, spelled once.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.attr("aria-invalid", "true")
        } else {
            self
        }
    }

    /// Any attribute, rendered on the server: `attr("autocomplete", "country")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("multiple")`.
    ///
    /// `multiple` and `size="4"` are how HTML asks for a list box instead of a
    /// dropdown. Both work, and neither is a shadcn prop, because Radix's Select
    /// has no such thing — but the heights in [`BASE`] are written for the closed
    /// control, so a list box wants an override to let it grow:
    ///
    /// ```ignore
    /// select_props().flag("multiple").class("data-[size=default]:h-auto")
    /// ```
    ///
    /// `data-[size=default]:h-auto`, not `h-auto`. A merge only replaces a
    /// utility with one carrying the same variant prefix, and the height here is
    /// prefixed; a plain `h-auto` would sit beside it and lose.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("value", "fruit")`.
    ///
    /// Through the [`Select`] tag the same binding is a `{…}` prop: it lands on
    /// the root, and a component's root attributes are evaluated where the tag
    /// was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("change", "setFruit(target.value)")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings; a select reports itself through `target.value`.
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

    /// The class list this select renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[&component_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order: the component's own first,
    /// the caller's spread last so it wins.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new()
            .set("data-slot", "select")
            .set("data-size", self.size.slug());

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser.
        // With no override there is nothing to resolve, so `class` stays a
        // finished class list and the control is styled on arrival, JavaScript
        // or no JavaScript. See the cost named in [`Self::render`].
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&component_classes(), &self.class]),
            )
        };

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the control in that block's scope,
    /// which is what a handler or a binding written here needs. A `{…}` written
    /// among the *options*, though, is compiled where those options were
    /// written — so a select whose list is the page's state wants
    /// [`select_variants`] on a native `<select>` instead.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();
        html! {
            <select @{..attrs}>
                <slot />
            </select>
        }
    }
}

/// One option's props.
///
/// An `<option>` has no class list of its own here: shadcn's `SelectItem`
/// string styles a div in a popup this port does not build, and none of it
/// applies to an option the browser draws. `class` is still forwarded, for the
/// little an option does accept.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectItemProps {
    class: String,
    attrs: Attrs,
}

/// A default option's props — the start of every chain.
pub fn select_item() -> SelectItemProps {
    SelectItemProps::default()
}

impl SelectItemProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes on the `<option>`. Unlike every other component here this is not
    /// laid over anything — an option has no utilities of its own — so it is
    /// written straight onto the element and never becomes a `twMerge` binding.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// What this option posts when it is chosen. An option with no `value`
    /// posts its own text, which is HTML's rule rather than this component's to
    /// restate.
    pub fn value(self, value: &str) -> Self {
        self.attr("value", value)
    }

    /// The option the control starts on — Radix's `value` on the Root, moved to
    /// where HTML keeps it.
    ///
    /// Written on more than one option, the last wins, which is what lets a
    /// [`SelectPlaceholder`] be `selected` and still be replaced by a real
    /// choice further down the list.
    pub fn selected(self, selected: bool) -> Self {
        if selected {
            self.flag("selected")
        } else {
            self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.flag("disabled")
        } else {
            self
        }
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("hidden")`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("selected", "fruit === 'apple'")`.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// Everything in `attrs`, laid over what is already set.
    pub fn attrs(mut self, attrs: Attrs) -> Self {
        let mut merged = std::mem::take(&mut self.attrs);
        merged = merged.merge(&attrs);
        self.attrs = merged;
        self
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// No `class` clause, unlike the other props types: an option's `class` is
    /// written literally and never becomes a binding.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", "select-item");

        if !self.class.trim().is_empty() {
            base = base.set("class", self.class.clone());
        }

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered `<option>`, with no boundary around it.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();
        html! {
            <option @{..attrs}>
                <slot />
            </option>
        }
    }
}

/// A select, the closed control and its list in one element.
///
/// ```ignore
/// <Select name="fruit">
///     <SelectPlaceholder>"Select a fruit"</SelectPlaceholder>
///     <SelectItem value="apple">"Apple"</SelectItem>
///     <SelectItem value="banana" selected>"Banana"</SelectItem>
/// </Select>
/// <Select size="sm" disabled>…</Select>
/// <Select invalid required>…</Select>
/// ```
///
/// The props are shadcn's Trigger's, spelled the way its JSX spells them, and
/// every one is optional — `html!` closes a props literal with
/// `..Default::default()`. There is no Trigger and no Content to nest: what
/// sits between the tags is the option list.
///
/// Native events are written as native events and land on the `<select>`
/// untouched. So does an attribute binding, which reads the state of the page
/// that wrote the tag:
///
/// ```ignore
/// // Reads and writes the page's `fruit`.
/// <Select value={fruit} onchange={setFruit(target.value)}>…</Select>
/// ```
///
/// [`SelectProps::render`] and [`select_variants`] remain for a select built in
/// Rust or an element this component does not render.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Select(
    class: &str,
    size: &str,
    name: &str,
    id: &str,
    disabled: bool,
    required: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = SelectProps::default()
        .class(class)
        .size(SelectSize::parse(size))
        .disabled(disabled)
        .required(required)
        .invalid(invalid);

    // An unwritten prop is an empty string, and an empty `name` is not the same
    // request as no `name` at all — so each of these is written only when the
    // caller said something.
    if !name.is_empty() {
        props = props.name(name);
    }
    if !id.is_empty() {
        props = props.id(id);
    }

    props = props.attrs(attrs);

    // `onchange={…}`, `value={…}` and friends. `html!` hands every `{…}` prop
    // over as the expression the author wrote, and it goes onto the select
    // unchanged — the component decides nothing about it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    // A caller's `{…}` props are the caller's expressions. Rendered without a
    // `<script>`, this block is compiled in its parent's scope, so they read
    // the caller's state the way a React prop would. Only a `class` override
    // with nothing of the caller's in it gets an empty script, so its
    // `{twMerge(…)}` resolves even where the page mounts no scope.
    if props.needs_scope() && bindings.is_empty() {
        let control = props.render(children);
        html! {
            <>
                @{control}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        let attrs = props.attr_list();
        html! {
            <select @{..attrs}>
                <slot />
            </select>
        }
    }
}

/// One option in the list.
///
/// ```ignore
/// <SelectItem value="apple">"Apple"</SelectItem>
/// <SelectItem value="banana" selected>"Banana"</SelectItem>
/// <SelectItem value="durian" disabled>"Durian"</SelectItem>
/// ```
///
/// `selected` is Radix's `value` on the Root, moved to where HTML keeps it —
/// see the module docs, because it is one of the two places this port changes
/// the shape of the API rather than the element under it.
///
/// What sits between the tags is the option's text, and text is all it can be:
/// an `<option>` renders as a string in a list the browser draws, so markup
/// inside it is ignored. An icon per row is the thing this component cannot do,
/// and the module docs say what to reach for instead.
#[component]
// A component's parameters are its props, and eight is what an option has. The
// lint is measuring a function; the call site is a tag, where each of these is
// named and optional.
#[allow(clippy::too_many_arguments)]
pub fn SelectItem(
    class: &str,
    value: &str,
    id: &str,
    selected: bool,
    disabled: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = SelectItemProps::default()
        .class(class)
        .selected(selected)
        .disabled(disabled);

    // Every option has a value, as Radix's does. `value=""` is a real choice,
    // the one a `<Select value={…}>` lands on when its state is empty, and a tag
    // cannot tell it from an unwritten prop, so both write the attribute. An
    // option without one would answer to its text and leave that select blank.
    // A `{…}` value is in `bindings` and is not written twice.
    if !value.is_empty() || !bindings.iter().any(|(name, _)| name == "value") {
        props = props.value(value);
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

    // A caller's `{…}` props are the caller's expressions. Rendered without a
    // `<script>`, this block is compiled in its parent's scope, so they read
    // the caller's state the way a React prop would. Only a `class` override
    // with nothing of the caller's in it gets an empty script, so its
    // `{twMerge(…)}` resolves even where the page mounts no scope.
    if props.needs_scope() && bindings.is_empty() {
        let item = props.render(children);
        html! {
            <>
                @{item}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        let attrs = props.attr_list();
        html! {
            <option @{..attrs}>
                <slot />
            </option>
        }
    }
}

/// The first option: what the control shows before a choice is made.
///
/// ```ignore
/// <SelectPlaceholder>"Select a fruit"</SelectPlaceholder>
/// ```
///
/// shadcn spells this `<SelectValue placeholder="Select a fruit" />`, inside the
/// Trigger. A `<select>` has no Value element — it shows whichever option is
/// chosen — so a placeholder is an option like any other, with four attributes
/// that make it behave like one:
///
/// - `selected`, so it is what shows first. Written on the first option, a
///   `selected` further down still wins, which is how a form re-rendered with a
///   previous answer keeps it.
/// - `value=""`, so `required` on the [`Select`] rejects it.
/// - `disabled`, so it cannot be chosen again once a real option has been.
/// - `data-placeholder`, which is what
///   `has-[option[data-placeholder]:checked]:text-muted-foreground` in [`BASE`]
///   looks for. Radix marks the Trigger; this marks the option and lets CSS ask.
///
/// `hidden` keeps it out of the open list, the way shadcn's placeholder is not
/// one of the items.
#[component]
pub fn SelectPlaceholder(class: &str, attrs: Attrs, children: Html) -> Html {
    let mut props = SelectItemProps::default()
        .class(class)
        .value("")
        .selected(true)
        .disabled(true)
        .flag("hidden")
        .attr("data-placeholder", "")
        .unset("data-slot")
        .attr("data-slot", "select-placeholder");

    props = props.attrs(attrs);

    let attrs = props.attr_list();
    html! {
        <option @{..attrs}>
            <slot />
        </option>
    }
}

/// A titled run of options — shadcn's `SelectGroup` and `SelectLabel` in one
/// element.
///
/// ```ignore
/// <SelectGroup label="Fruits">
///     <SelectItem value="apple">"Apple"</SelectItem>
/// </SelectGroup>
/// ```
///
/// HTML has no separate label element inside a select: an `<optgroup>` carries
/// its title as the `label` attribute, and the browser draws it. shadcn's
/// `SelectLabel` class list — `px-2 py-1.5 text-xs text-muted-foreground` —
/// styles a div in a popup this port does not build, so it is not ported.
#[component]
pub fn SelectGroup(class: &str, label: &str, disabled: bool, attrs: Attrs, children: Html) -> Html {
    let mut base = Attrs::new()
        .set("data-slot", "select-group")
        .set("label", label);

    if disabled {
        base = base.set("disabled", "");
    }
    if !class.trim().is_empty() {
        base = base.set("class", class);
    }

    base = base.merge(&attrs);

    html! {
        <optgroup @{..base}>
            <slot />
        </optgroup>
    }
}

/// A rule between two runs of options.
///
/// ```ignore
/// <SelectSeparator />
/// ```
///
/// An `<hr>` inside a `<select>` is HTML, and recent Chrome, Safari and Firefox
/// draw it. Older browsers ignore it — which is the right failure, since a
/// separator carries no meaning a reader would miss. shadcn's class list here
/// (`-mx-1 my-1 h-px bg-border`) styles a div in a popup this port does not
/// build, so it is not ported: the rule is the browser's.
#[component]
pub fn SelectSeparator(class: &str, attrs: Attrs) -> Html {
    let mut base = Attrs::new().set("data-slot", "select-separator");

    if !class.trim().is_empty() {
        base = base.set("class", class);
    }

    base = base.merge(&attrs);

    html! {
        <hr @{..base} />
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, Select, SelectItem, SelectItemProps, SelectPlaceholder, SelectProps, SelectSize,
        select_item, select_props, select_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_select_carries_shadcns_defaults() {
        let markup = select_props().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<select "), "{markup}");
        assert!(markup.contains(r#"data-slot="select""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains("rounded-md"), "{markup}");
        assert!(markup.contains("border-input"), "{markup}");
        assert!(markup.contains("data-[size=default]:h-9"), "{markup}");
        assert!(markup.ends_with("</select>"), "{markup}");
    }

    /// The deltas from shadcn's Radix version, as assertions rather than prose.
    /// Each one is a decision the module docs argue for.
    #[test]
    fn the_port_swaps_the_primitive_for_the_native_control() {
        let markup = select_props().render(Html::empty()).into_test_string();

        // 1. The UA's own dropdown is turned off, and the chevron is an image.
        assert!(markup.contains("appearance-none"), "{markup}");
        assert!(markup.contains("bg-[url("), "{markup}");
        // 2. Radix's Trigger attribute becomes a question about the option.
        assert!(
            markup.contains("has-[option[data-placeholder]:checked]:text-muted-foreground"),
            "{markup}"
        );
        assert!(!markup.contains("data-[placeholder]:"), "{markup}");
        // 3. What cannot exist here is not carried.
        assert!(!markup.contains("select-value"), "{markup}");
        assert!(!markup.contains("[&_svg]"), "{markup}");
    }

    /// Both themes are spelled out, because a data URI cannot read a variable.
    /// The Radio Group's dot escapes this; a chevron cannot.
    #[test]
    fn the_chevron_is_drawn_once_per_theme() {
        let markup = select_props().render(Html::empty()).into_test_string();

        assert!(markup.contains("%230a0a0a"), "{markup}");
        assert!(markup.contains("dark:bg-[url("), "{markup}");
        assert!(markup.contains("%23fafafa"), "{markup}");
    }

    #[test]
    fn the_size_is_a_data_attribute_the_class_list_reads() {
        let markup = select_props()
            .size(SelectSize::Sm)
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-size="sm""#), "{markup}");
        // Both heights are always in the class list, under different prefixes.
        assert!(markup.contains("data-[size=sm]:h-8"), "{markup}");
        assert!(markup.contains("data-[size=default]:h-9"), "{markup}");

        assert_eq!(SelectSize::parse(""), SelectSize::Default);
        assert_eq!(SelectSize::parse("sm"), SelectSize::Sm);
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = select_props()
            .disabled(true)
            .required(true)
            .invalid(true)
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#" disabled="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
        assert!(markup.contains(r#"aria-invalid="true""#), "{markup}");

        let markup = select_props()
            .disabled(false)
            .required(false)
            .invalid(false)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains(r#" disabled="""#), "{markup}");
        assert!(!markup.contains(r#" required="""#), "{markup}");
        assert!(!markup.contains(r#"aria-invalid=""#), "{markup}");
    }

    /// An option carries no class list of its own — shadcn's styles a popup this
    /// port does not build — but it is still an element with attributes.
    #[test]
    fn an_option_is_an_option_and_not_a_styled_div() {
        let markup = select_item()
            .value("apple")
            .selected(true)
            .render(Html::escape("Apple"))
            .into_test_string();

        assert!(markup.starts_with("<option "), "{markup}");
        assert!(markup.contains(r#"data-slot="select-item""#), "{markup}");
        assert!(markup.contains(r#"value="apple""#), "{markup}");
        assert!(markup.contains(r#" selected="""#), "{markup}");
        assert!(markup.ends_with(">Apple</option>"), "{markup}");
        // Nothing from shadcn's item string, because none of it would apply.
        assert!(!markup.contains("focus:bg-accent"), "{markup}");
        assert!(!markup.contains("class="), "{markup}");
    }

    /// The four attributes that make an option behave like Radix's placeholder.
    #[test]
    fn the_placeholder_is_an_option_that_marks_itself() {
        let markup =
            SelectPlaceholder("", Attrs::new(), Html::escape("Select a fruit")).into_test_string();

        assert!(markup.contains(r#"value="""#), "{markup}");
        assert!(markup.contains(r#" selected="""#), "{markup}");
        assert!(markup.contains(r#" disabled="""#), "{markup}");
        assert!(markup.contains(r#" hidden="""#), "{markup}");
        assert!(markup.contains(r#"data-placeholder="""#), "{markup}");
        assert!(
            markup.contains(r#"data-slot="select-placeholder""#),
            "{markup}"
        );
        assert!(markup.ends_with(">Select a fruit</option>"), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = select_props()
            .class("w-full")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("w-full"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_select_with_no_override_needs_no_runtime() {
        let props = select_props().name("fruit");
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="w-fit"#), "{markup}");

        assert!(!select_props().class("w-full").needs_scope());
        assert!(
            select_props()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ setFruit(target.value) })
                )
                .needs_scope()
        );
        // An option's class is literal, so it never needs a scope on its own.
        assert!(!select_item().class("font-mono").needs_scope());
        assert!(
            select_item()
                .bind(
                    "selected",
                    crate::components::rahti_ui::attrs::ui_expr!(selected={fruit === "apple"})
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = select_props()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setFruit(target.value) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setFruit(target.value)""#),
            "{markup}"
        );
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = select_props()
            .attr("data-slot", "mine")
            .attr("autocomplete", "country")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"autocomplete="country""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = select_props()
            .attrs(Attrs::new().unset("data-slot").set("aria-label", "Fruit"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains(r#"aria-label="Fruit""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = select_item()
            .value(r#"a" pp-onfocus="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn select_variants_is_the_components_own_class_list() {
        assert_eq!(
            select_variants(SelectSize::Default, "w-full"),
            select_props().class("w-full").class_list()
        );
    }

    /// The call form behind the tags, and the shape of a whole list.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let options = Html::concat([
            SelectPlaceholder("", Attrs::new(), Html::escape("Select a fruit")),
            SelectItem(
                "",
                "apple",
                "",
                false,
                false,
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::escape("Apple"),
            ),
        ]);

        let markup = Select(
            "",
            "sm",
            "fruit",
            "fruit",
            false,
            true,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            options,
        )
        .into_test_string();

        assert!(markup.contains(r#"name="fruit""#), "{markup}");
        assert!(markup.contains(r#"data-size="sm""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
        assert!(markup.contains("Select a fruit</option>"), "{markup}");
        assert!(markup.contains(r#"value="apple""#), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Select>…</Select>` compiles to
    /// now that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_select() {
        let markup = Select(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            bool::default(),
            bool::default(),
            bool::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="select""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(" disabled="), "{markup}");
    }

    /// What `<Select onchange={setFruit(target.value)}>` compiles to. The
    /// expression arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_select() {
        let markup = Select(
            "",
            "",
            "fruit",
            "",
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[("onchange", "setFruit(target.value)")]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setFruit(target.value)""#),
            "{markup}"
        );
    }

    /// Both props types are `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_defaults_are_a_plain_select_and_a_plain_option() {
        assert_eq!(SelectProps::new(), SelectProps::default());
        assert_eq!(SelectItemProps::new(), SelectItemProps::default());
        assert_eq!(
            select_props().render(Html::empty()).into_test_string(),
            SelectProps::new().render(Html::empty()).into_test_string()
        );
    }

    /// `<SelectItem value="">` is the choice an empty `<Select value={…}>` shows.
    /// Dropping the attribute leaves that select with nothing selected.
    #[test]
    fn an_empty_option_value_is_written() {
        let markup = SelectItem(
            "",
            "",
            "",
            false,
            false,
            crate::rahti::Bindings::__from_html(&[]),
            Attrs::new(),
            Html::escape("Default"),
        )
        .into_test_string();
        assert!(markup.contains(r#"value="""#), "{markup}");
    }

    /// A bound option keeps one `value`: the binding.
    #[test]
    fn a_bound_option_value_is_not_written_twice() {
        let markup = SelectItem(
            "",
            "",
            "",
            false,
            false,
            crate::rahti::Bindings::__from_html(&[("value", "device.id")]),
            Attrs::new(),
            Html::escape("Device"),
        )
        .into_test_string();
        assert_eq!(markup.matches(" value=").count(), 1, "{markup}");
    }

    /// `<Select value={fruit} onchange={setFruit(target.value)}>`: the
    /// `<select>` is the root, so the bindings stay on it and the block renders
    /// script-less, compiled where the tag was written.
    #[test]
    fn a_bound_select_is_one_scriptless_element() {
        let markup = Select(
            "",
            "",
            "",
            "",
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[
                ("value", "fruit"),
                ("onchange", "setFruit(target.value)"),
            ]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.starts_with("<select "), "{markup}");
        assert!(markup.contains(r#"value="{fruit}""#), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
    }
}

/// The layer-independence check behind [`BASE`].
///
/// Two layers — the control and the chevron — and the claim is
/// [`Button`](super::button)'s: no two utilities in a rendered class list
/// compete, so joining them is what merging them would give.
///
/// The interesting pairs are the `bg-`s, and there are four of them here:
/// `bg-transparent` is a colour, `bg-no-repeat` a repeat, the chevron an image,
/// and its position a position. `utils::audit::family` reads `bg-` as the four
/// properties it actually is, which is what keeps this honest — and the two
/// `bg-[url(…)]`s sit under different prefixes.
#[cfg(test)]
mod audit {
    use super::{BASE, CHEVRON, component_classes};
    use crate::components::rahti_ui::utils::audit::clashes;

    #[test]
    fn no_utility_contradicts_another() {
        let found = clashes(&component_classes());

        assert!(
            found.is_empty(),
            "{} clashes:\n{}",
            found.len(),
            found.join("\n")
        );
    }

    /// Each layer is also clean on its own, so a failure above points at the
    /// join rather than at one string.
    #[test]
    fn no_layer_contradicts_itself() {
        for (name, layer) in [("BASE", BASE), ("CHEVRON", CHEVRON)] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
