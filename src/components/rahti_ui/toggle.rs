// rahti-ui: toggle
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Toggle — a port of shadcn/ui's `toggle`.
//!
//! shadcn's Toggle is a Radix `TogglePrimitive.Root`: a `<button aria-pressed>`
//! carrying `data-state="on"`, and a `useState` behind it, because a button does
//! not stay pressed on its own. This is a `<label>` wrapping a hidden
//! `<input type="checkbox">`, which does.
//!
//! ## Why two elements rather than one
//!
//! Every other control in this library is one native element. A toggle cannot
//! be, and the reason is worth stating: the state has to live in a checkbox, and
//! a checkbox is void — it can hold no icon. So the checkbox holds the state and
//! the label holds the appearance, which is the arrangement HTML has for exactly
//! this and the reason a click on the label toggles the input.
//!
//! ```html
//! <label data-slot="toggle" class="…">
//!   <input type="checkbox" class="sr-only" …>
//!   <!-- the icon, the text, whatever was between the tags -->
//! </label>
//! ```
//!
//! What that buys is the usual bet, and here it is a large one: the toggle
//! toggles, keeps its state, takes the space bar, posts in a form, and is read
//! correctly, with no JavaScript at all. Radix's Root does none of those without
//! a `useState` above it.
//!
//! It also buys back something the [`Checkbox`](super::checkbox) port had to
//! give up. shadcn's `[&_svg]:size-4` and its friends style icon children, which
//! a void `<input>` cannot have — on a Checkbox they were dropped. Here the
//! visible element is a `<label>`, so all three come across untouched and an
//! icon inside a Toggle is sized the way shadcn sizes it.
//!
//! ## Four deltas
//!
//! 1. **`data-[state=on]:` becomes `has-[:checked]:`.** Radix keeps its state in
//!    a data attribute because a `<button>` has none. The label has no state
//!    either — but its input does, and `:has()` is how one element asks about
//!    another. Every state utility moves the same way: `disabled:` becomes
//!    `has-[:disabled]:`, `focus-visible:` becomes `has-[:focus-visible]:`, and
//!    `aria-invalid:` becomes `has-[[aria-invalid=true]]:`. The declarations are
//!    shadcn's; only the question changed.
//! 2. **`select-none` is added.** A `<button>`'s text cannot be selected by a
//!    double-click and a `<label>`'s can. One utility, and it is the same one
//!    [`Label`](super::label) carries for the same reason.
//! 3. **The hover utilities are re-layered.** shadcn puts
//!    `hover:bg-muted hover:text-muted-foreground` in the base and lets the
//!    `outline` variant's `hover:bg-accent hover:text-accent-foreground`
//!    overrule them through `twMerge`. This library joins its layers rather than
//!    merging them — see [`utils`](super::utils) — so the base carries neither
//!    and each variant carries its own. Same rendered class list, arrived at
//!    without a merge, and `audit::no_layer_contradicts_another` can prove it.
//! 4. **The accessible name of the state changes.** Radix's button is announced
//!    as a toggle button, pressed or not pressed; this is announced as a
//!    checkbox, checked or not. That is a real difference and it is the price of
//!    the state being real. `role="switch"` was available to
//!    [`Switch`](super::switch) because a switch *is* a two-state control;
//!    there is no role that turns a checkbox into a toggle button without
//!    taking its behaviour away with it.
//!
//! Everything else — every colour, every ring, every size, both variants — is
//! shadcn's, copied rather than reinterpreted.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Toggle>@{bold_icon()}</Toggle>
//! <Toggle variant="outline" size="lg" pressed>"Bold"</Toggle>
//! <Toggle disabled>@{italic_icon()}</Toggle>
//! ```
//!
//! `pressed` is Radix's own prop name, and it writes `checked` on the input —
//! the prop reads the way the JSX reads and the attribute reads the way HTML
//! does.
//!
//! ## Two elements, two attribute sets
//!
//! The visible element and the control are not the same element here, so the
//! builder says which one it means. [`attr`](ToggleProps::attr) and
//! [`attrs`](ToggleProps::attrs) reach the `<label>`, where `class` and
//! `data-slot` live. Everything about the *control* — `name`, `value`,
//! `pressed`, `disabled`, `required`, `invalid`, every binding and every handler
//! — reaches the `<input>`, because that is where a form reads it and where a
//! `change` event fires. [`input_attr`](ToggleProps::input_attr) is the escape
//! hatch for anything else that belongs there.
//!
//! ## The toggle is UI. The state is the caller's.
//!
//! `onchange={…}` and `checked={…}` land on the input untouched, and read and
//! write the page that wrote the tag. Through the tag, the bindings are lifted,
//! because the input sits below the root: a handler goes onto the root, where
//! the event bubbles up and runs in the caller's scope, and a value goes onto
//! it as a prop the input reads back through `pp.props`. Listen for `change`
//! rather than `click`: a click on the label reaches the root twice, once for
//! the label and once for the input it activates.
//!
//! ## When the tag is not enough
//!
//! **[`ToggleProps::render`], interpolated — for a toggle built in Rust.**
//!
//! ```ignore
//! @{toggle()
//!     .name("bold")
//!     .bind("checked", crate::components::rahti_ui::attrs::ui_expr!(checked={bold}))
//!     .on("change", crate::components::rahti_ui::attrs::ui_handler!({setBold(target.checked)}))
//!     .render(Html::escape("Bold"))}
//! ```
//!
//! **[`toggle_variants`] — the class list alone**, which is shadcn's own export
//! under shadcn's own name.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding, lift_bindings};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// Everything a toggle is, before its variant and its size.
///
/// shadcn's base string, with every state utility asked as a `:has()` question
/// about the input, `select-none` added, and the two `hover:` utilities moved
/// down into [`ToggleVariant::class`] — the module docs argue each.
///
/// The three `[&_svg]:` utilities are shadcn's, unchanged, and they work here
/// because the element carrying them is a `<label>` with children rather than a
/// void input.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
///
/// `pub(crate)` because [`ToggleGroup`](super::toggle_group) builds its item's
/// class list from these same layers minus the two a group overrides — it cannot
/// go through [`toggle_variants`] and then take utilities back out, since this
/// library joins its layers rather than merging them.
pub(crate) const BASE: &str = "inline-flex items-center justify-center gap-2 rounded-md text-sm \
                    font-medium whitespace-nowrap select-none \
                    transition-[color,box-shadow] outline-none \
                    has-[:focus-visible]:border-ring has-[:focus-visible]:ring-[3px] \
                    has-[:focus-visible]:ring-ring/50 \
                    has-[:disabled]:pointer-events-none has-[:disabled]:opacity-50 \
                    has-[[aria-invalid=true]]:border-destructive \
                    has-[[aria-invalid=true]]:ring-destructive/20 \
                    has-[:checked]:bg-accent has-[:checked]:text-accent-foreground \
                    dark:has-[[aria-invalid=true]]:ring-destructive/40 \
                    [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                    [&_svg:not([class*='size-'])]:size-4";

/// How the input is hidden without being taken away.
///
/// `sr-only` is Tailwind's own: off-screen to a sighted reader, still focusable,
/// still checkable, still in the form. The state has to stay a real checkbox for
/// any of this component's claims to hold, so it is hidden rather than replaced.
const INPUT: &str = "sr-only";

/// Which of shadcn's two a toggle is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

impl ToggleVariant {
    /// The utilities this variant adds to [`BASE`].
    ///
    /// Each variant carries its own `hover:`, rather than shadcn's arrangement
    /// where the base carries one pair and `outline` overrules it. Same
    /// rendered class list; no merge required. See the module docs.
    pub const fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-transparent hover:bg-muted hover:text-muted-foreground",
            Self::Outline => {
                "border border-input bg-transparent shadow-xs \
                 hover:bg-accent hover:text-accent-foreground"
            }
        }
    }

    /// The name shadcn writes in `variant="…"`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `variant="…"` prop.
    ///
    /// An empty string is the default rather than an error, because that is what
    /// an unwritten prop arrives as. Anything else unrecognised is a mistake at
    /// the call site and trips a `debug_assert!` — the same rule
    /// [`ButtonVariant::parse`](super::button::ButtonVariant::parse) applies.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "outline" => Self::Outline,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Toggle variant. One of: default, outline."
                );
                Self::Default
            }
        }
    }
}

/// How big it is. `min-w-*` rather than `w-*`, so a toggle holding one icon is
/// square and one holding a word grows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    /// The utilities this size adds to [`BASE`].
    pub const fn class(self) -> &'static str {
        match self {
            Self::Default => "h-9 min-w-9 px-2",
            Self::Sm => "h-8 min-w-8 px-1.5",
            Self::Lg => "h-10 min-w-10 px-2.5",
        }
    }

    /// The height alone — the only part of [`class`](Self::class) a
    /// [`ToggleGroup`](super::toggle_group) item keeps.
    ///
    /// A group overrides `min-w-*` with `min-w-0` and `px-*` with `px-3`, and
    /// shadcn resolves that with `twMerge`. This library joins its layers, so
    /// the group takes the height and leaves the other two behind instead —
    /// same rendered class list, nothing to resolve.
    ///
    /// `tests::the_height_is_the_front_of_the_size` holds the two in step.
    pub const fn height(self) -> &'static str {
        match self {
            Self::Default => "h-9",
            Self::Sm => "h-8",
            Self::Lg => "h-10",
        }
    }

    /// The name shadcn writes in `size="…"`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `size="…"` prop. Same rules
    /// as [`ToggleVariant::parse`].
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "sm" => Self::Sm,
            "lg" => Self::Lg,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Toggle size. One of: default, sm, lg."
                );
                Self::Default
            }
        }
    }
}

/// Every utility a toggle carries, before the caller's own.
fn component_classes(variant: ToggleVariant, size: ToggleSize) -> String {
    cn(&[BASE, variant.class(), size.class()])
}

/// The class list shadcn's `toggleVariants({ variant, size, className })`
/// returns.
///
/// shadcn's own export, under shadcn's own name and with its own signature.
/// Reach for it when the element has to belong to the page — but note that the
/// class list alone is not a whole toggle: the `has-[…]:` utilities are asking
/// about an input, and a `<label>` written by hand has to contain one.
pub fn toggle_variants(variant: ToggleVariant, size: ToggleSize, class: &str) -> String {
    cn(&[&component_classes(variant, size), class])
}

/// The class list the hidden input carries. Exported for the same reason
/// [`toggle_variants`] is: a hand-written toggle needs both halves.
pub fn toggle_input_variants(class: &str) -> String {
    cn(&[INPUT, class])
}

/// A toggle's props, with shadcn's defaults already in place.
///
/// Build one with [`toggle`] and narrow it down a method at a time. Nothing here
/// is required.
///
/// Two attribute sets, because a toggle is two elements: `attrs` is the
/// `<label>`'s and `input_attrs` is the `<input>`'s. The module docs say which
/// builder reaches which.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToggleProps {
    class: String,
    variant: ToggleVariant,
    size: ToggleSize,
    attrs: Attrs,
    input_attrs: Attrs,
}

/// A default toggle's props — the start of every chain.
pub fn toggle() -> ToggleProps {
    ToggleProps::default()
}

impl ToggleProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the label's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// The control's name, on the input — what a form posts this under.
    pub fn name(self, name: &str) -> Self {
        self.input_attr("name", name)
    }

    /// The control's id, on the input rather than the label, because it names
    /// the thing an `aria-describedby` or an external `<label for>` would want
    /// to point at.
    pub fn id(self, id: &str) -> Self {
        self.input_attr("id", id)
    }

    /// What this toggle posts when it is on. With no `value` a checkbox submits
    /// `on`, which is HTML's rule rather than this component's to restate.
    pub fn value(self, value: &str) -> Self {
        self.input_attr("value", value)
    }

    /// On, to start with — Radix's `defaultPressed`, written as the `checked`
    /// HTML has.
    ///
    /// `false` leaves the attribute off rather than writing `checked="false"`,
    /// which HTML would read as checked.
    pub fn pressed(self, pressed: bool) -> Self {
        if pressed {
            self.input_flag("checked")
        } else {
            self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.input_flag("disabled")
        } else {
            self
        }
    }

    pub fn required(self, required: bool) -> Self {
        if required {
            self.input_flag("required")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` on the input — which is what the
    /// `has-[[aria-invalid=true]]:` utilities in [`BASE`] paint the label with.
    /// The styling and the accessible state are the same fact, spelled once, and
    /// it is on the control rather than on the label because that is what a
    /// screen reader reads.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.input_attr("aria-invalid", "true")
        } else {
            self
        }
    }

    /// An attribute on the `<label>`: `attr("title", "Bold")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute on the `<label>`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off the `<label>`: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute on the `<input>`: `input_attr("form", "editor")`.
    pub fn input_attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.input_attrs = std::mem::take(&mut self.input_attrs).set(name, value);
        self
    }

    /// A bare boolean attribute on the `<input>`: `input_flag("autofocus")`.
    pub fn input_flag(self, name: &str) -> Self {
        self.input_attr(name, "")
    }

    /// Take an attribute off the `<input>`.
    pub fn input_unset(mut self, name: &str) -> Self {
        self.input_attrs = std::mem::take(&mut self.input_attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date, on the input:
    /// `bind("checked", "bold")`.
    ///
    /// This is the spelling for a toggle [`render`](Self::render)ed into a
    /// page's own block, where the input is in the page's scope. Through the
    /// [`Toggle`] tag the same binding is a `{…}` prop, lifted onto the label
    /// and read back by the input — see [`Toggle`].
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.input_attrs = std::mem::take(&mut this.input_attrs).bind(name, expr);
        this
    }

    /// An event handler on the input, as a PulsePoint expression:
    /// `on("change", "setBold(target.checked)")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings; a toggle reports itself through `target.checked`, since
    /// under the paint it is a checkbox.
    pub fn on(self, event: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let name = if event.starts_with("on") {
            event.to_string()
        } else {
            format!("on{event}")
        };
        self.bind(&name, expr)
    }

    /// Everything in `attrs`, laid over the label's — shadcn's `{...props}`, and
    /// the last word on all of it.
    pub fn attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = merge(std::mem::take(&mut self.attrs), &attrs);
        self
    }

    /// Everything in `attrs`, laid over the input's.
    pub fn input_attrs(mut self, attrs: Attrs) -> Self {
        self.input_attrs = merge(std::mem::take(&mut self.input_attrs), &attrs);
        self
    }

    /// The class list this toggle's label renders with, the caller's own
    /// included.
    ///
    /// A join, not a merge: the three layers contradict each other nowhere —
    /// which is what the re-layering in [`ToggleVariant::class`] bought — so
    /// this is exact for everything except a caller's `.class(…)` override. That
    /// is resolved in the browser; see [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[&component_classes(self.variant, self.size), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Bindings run from the client registry compiled by `html!` and retain their caller scope.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .chain(self.input_attrs.iter())
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The label's open tag, in shadcn's order: the component's own first, the
    /// caller's spread last so it wins.
    fn label_attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", "toggle");

        // Static class overrides compose data; they do not resolve conflicts.
        // Supply an explicit compiled `class={twMerge(...)}` binding when
        // overriding utilities from the component defaults.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[
                    &component_classes(self.variant, self.size),
                    &self.class,
                ]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The input's open tag.
    fn input_attr_list(&self) -> Attrs {
        let base = Attrs::new()
            .set("type", "checkbox")
            .set("data-slot", "toggle-input")
            .set("class", toggle_input_variants(""));

        merge(base, &self.input_attrs)
    }

    /// The rendered pair, with no boundary around them.
    ///
    /// Interpolating this into a block puts both elements in that block's scope,
    /// which is what a handler or a binding written here needs. A `{…}` written
    /// among the *children*, though, is compiled where those children were
    /// written — so a toggle whose label text is the page's state wants
    /// [`toggle_variants`] on a native `<label>` instead.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    pub fn render(&self, children: Html) -> Html {
        let label = self.label_attr_list();
        let input = self.input_attr_list();

        html! {
            <label @{..label}>
                <input @{..input} />
                <slot />
            </label>
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away — the same rule `AttrList` applies when it writes a tag.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// A toggle: a button that stays pressed.
///
/// ```ignore
/// <Toggle>@{bold_icon()}</Toggle>
/// <Toggle variant="outline" size="lg" pressed>"Bold"</Toggle>
/// <Toggle disabled>@{italic_icon()}</Toggle>
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every one is
/// optional — `html!` closes a props literal with `..Default::default()`. What
/// sits between the tags is what the toggle shows: an icon, a word, or both,
/// laid out by shadcn's `inline-flex items-center gap-2`.
///
/// `pressed` is Radix's own prop name and writes `checked` on the input. Every
/// other control prop lands there too — see the module docs, because a toggle is
/// two elements and the builder says which one it means.
///
/// Native events are written as native events and land on the `<input>` and
/// read the page that wrote the tag, and so do attribute bindings:
///
/// ```ignore
/// // Reads and writes the page's `bold`.
/// <Toggle checked={bold} onchange={setBold(target.checked)}>"Bold"</Toggle>
/// ```
///
/// Through the tag, the bindings are lifted, because the input sits below the
/// root: a handler goes onto the root, where the event bubbles up and runs in
/// the caller's scope, and a value goes onto it as a prop the input reads back
/// through `pp.props`. Listen for `change` rather than `click`, which reaches
/// the label twice.
///
/// `attrs` is the rest of the `{...props}` spread, and it reaches the label.
#[component]
// A component's parameters are its props, and twelve is what a toggle has. The
// lint is measuring a function; the call site is a tag, where each of these is
// named and optional.
#[allow(clippy::too_many_arguments)]
pub fn Toggle(
    class: &str,
    variant: &str,
    size: &str,
    name: &str,
    id: &str,
    value: &str,
    pressed: bool,
    disabled: bool,
    required: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = ToggleProps::default()
        .class(class)
        .variant(ToggleVariant::parse(variant))
        .size(ToggleSize::parse(size))
        .pressed(pressed)
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
    if !value.is_empty() {
        props = props.value(value);
    }

    props = props.attrs(attrs);

    // `onchange={…}`, `checked={…}` and friends. The input sits below the
    // label, where the runtime evaluates in this component's scope rather than
    // the caller's, so they are lifted: handlers onto the label, values onto it
    // as props the input reads back. Applied last, so a controlled
    // `checked={…}` replaces an uncontrolled `pressed` rather than sitting
    // beside it. See [`lift_bindings`].
    let lifted = !bindings.is_empty();
    if lifted {
        let split = lift_bindings(bindings);
        props = props.attrs(split.root).input_attrs(split.inner);
    }

    // A `class` override needs a mounted block for its `{twMerge(…)}`, and a
    // block with no `<script>` is never mounted. So a toggle with one gets an
    // empty script, and a toggle with none stays inert markup and costs the
    // runtime nothing.
    //
    // Unlike the Checkbox and the Switch there is no `peer` to protect here:
    // this component's label is its own wrapper, and nothing outside reaches
    // across it with a combinator. So the fragment is safe in both branches, and
    // the only reason to keep them apart is the script.
    if lifted {
        // The script gives this block the scope `pp.props` is read in, inside
        // the label so the label stays the block's root and carries the props.
        let label = props.label_attr_list();
        let input = props.input_attr_list();
        html! {
            <label @{..label}>
                <input @{..input} />
                <slot />
                <script>r##""##</script>
            </label>
        }
    } else if props.needs_scope() {
        let toggle = props.render(children);
        html! {
            <>
                @{toggle}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        let label = props.label_attr_list();
        let input = props.input_attr_list();
        html! {
            <label @{..label}>
                <input @{..input} />
                <slot />
            </label>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Toggle, ToggleProps, ToggleSize, ToggleVariant, toggle, toggle_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_toggle_carries_shadcns_defaults() {
        let markup = toggle().render(Html::escape("Bold")).into_test_string();

        assert!(markup.starts_with("<label "), "{markup}");
        assert!(markup.contains(r#"data-slot="toggle""#), "{markup}");
        assert!(markup.contains(r#"type="checkbox""#), "{markup}");
        assert!(markup.contains(r#"data-slot="toggle-input""#), "{markup}");
        assert!(markup.contains("inline-flex"), "{markup}");
        assert!(markup.contains("rounded-md"), "{markup}");
        // shadcn's default variant and size.
        assert!(markup.contains("bg-transparent"), "{markup}");
        assert!(markup.contains("h-9"), "{markup}");
        assert!(markup.ends_with("Bold</label>"), "{markup}");
    }

    /// The state lives in a checkbox and the appearance lives in a label, which
    /// is the whole shape of this port.
    #[test]
    fn the_state_is_a_checkbox_and_the_paint_is_a_label() {
        let markup = toggle().render(Html::empty()).into_test_string();

        // The input is inside the label, and hidden rather than replaced.
        assert!(markup.contains(r#"<label "#), "{markup}");
        assert!(markup.contains(r#"class="sr-only""#), "{markup}");
        // Radix's data attribute becomes a question the label asks its input.
        assert!(markup.contains("has-[:checked]:bg-accent"), "{markup}");
        assert!(markup.contains("has-[:disabled]:opacity-50"), "{markup}");
        assert!(
            markup.contains("has-[:focus-visible]:ring-[3px]"),
            "{markup}"
        );
        assert!(!markup.contains("data-[state=on]"), "{markup}");
        // A label's text can be selected and a button's cannot.
        assert!(markup.contains("select-none"), "{markup}");
    }

    /// What the Checkbox port had to give up and this one gets back: the
    /// visible element has children, so shadcn's icon utilities come across.
    #[test]
    fn the_icon_utilities_survive_because_a_label_has_children() {
        let classes = toggle().class_list();

        assert!(classes.contains("[&_svg]:pointer-events-none"), "{classes}");
        assert!(classes.contains("[&_svg]:shrink-0"), "{classes}");
        assert!(
            classes.contains("[&_svg:not([class*='size-'])]:size-4"),
            "{classes}"
        );
    }

    /// Markup between the tags is markup, not text — an icon, a word, or both.
    #[test]
    fn children_pass_through_into_the_label() {
        let markup = toggle()
            .render(Html::from_raw("<svg></svg>Bold".into()))
            .into_test_string();

        assert!(markup.contains("<svg></svg>Bold</label>"), "{markup}");
    }

    /// Both of shadcn's variants and all three sizes, and the re-layering that
    /// keeps the join exact.
    #[test]
    fn the_variants_and_sizes_are_shadcns() {
        assert!(
            toggle()
                .variant(ToggleVariant::Outline)
                .class_list()
                .contains("border-input")
        );
        assert!(
            toggle()
                .size(ToggleSize::Lg)
                .class_list()
                .contains("min-w-10")
        );
        assert!(toggle().size(ToggleSize::Sm).class_list().contains("h-8"));

        // Each variant carries its own hover, and only its own — shadcn keeps
        // one pair in the base and lets `outline` overrule it through twMerge.
        let default = toggle().class_list();
        assert!(default.contains("hover:bg-muted"), "{default}");
        assert!(!default.contains("hover:bg-accent"), "{default}");

        let outline = toggle().variant(ToggleVariant::Outline).class_list();
        assert!(outline.contains("hover:bg-accent"), "{outline}");
        assert!(!outline.contains("hover:bg-muted"), "{outline}");

        assert_eq!(ToggleVariant::parse(""), ToggleVariant::Default);
        assert_eq!(ToggleVariant::parse("outline"), ToggleVariant::Outline);
        assert_eq!(ToggleSize::parse("lg"), ToggleSize::Lg);
    }

    /// What a Toggle Group item keeps of a size, held in step with the whole of
    /// it. See [`ToggleSize::height`].
    #[test]
    fn the_height_is_the_front_of_the_size() {
        for size in [ToggleSize::Default, ToggleSize::Sm, ToggleSize::Lg] {
            let class = size.class();
            let height = size.height();

            assert!(class.starts_with(height), "{class} / {height}");
            // And it really is only the height: the other two are what a group
            // overrides.
            assert!(class[height.len()..].contains("min-w-"), "{class}");
            assert!(class[height.len()..].contains("px-"), "{class}");
        }
    }

    /// Every control attribute belongs to the input, not to the label.
    #[test]
    fn the_control_props_land_on_the_input() {
        let markup = toggle()
            .name("bold")
            .id("bold")
            .value("on")
            .pressed(true)
            .required(true)
            .invalid(true)
            .render(Html::empty())
            .into_test_string();

        let input = markup.split("<input ").nth(1).unwrap();
        assert!(input.contains(r#"name="bold""#), "{markup}");
        assert!(input.contains(r#"id="bold""#), "{markup}");
        assert!(input.contains(r#"value="on""#), "{markup}");
        assert!(input.contains(r#" checked="""#), "{markup}");
        assert!(input.contains(r#" required="""#), "{markup}");
        assert!(input.contains(r#"aria-invalid="true""#), "{markup}");
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = toggle()
            .pressed(false)
            .disabled(false)
            .required(false)
            .invalid(false)
            .render(Html::empty())
            .into_test_string();

        // Attributes, not the `has-[…]:` utilities that are in the class list
        // either way — which is the whole point of those prefixes. Asked of the
        // input alone, because the label's class list names `aria-invalid`
        // inside a selector and would answer for the wrong reason.
        let input = markup.split("<input ").nth(1).unwrap();
        assert!(!input.contains(r#" checked="""#), "{markup}");
        assert!(!input.contains(r#" disabled="""#), "{markup}");
        assert!(!input.contains(r#" required="""#), "{markup}");
        assert!(!input.contains("aria-invalid="), "{markup}");
        assert!(markup.contains("has-[:checked]:bg-accent"), "{markup}");
        assert!(
            markup.contains("has-[[aria-invalid=true]]:border-destructive"),
            "{markup}"
        );
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_hands_the_label_class_list_on_the_server() {
        let markup = toggle()
            .class("rounded-full")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("rounded-full"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_toggle_with_no_override_needs_no_runtime() {
        let props = toggle().name("bold");
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="inline-flex"#), "{markup}");

        assert!(!toggle().class("rounded-full").needs_scope());
        assert!(
            toggle()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ setBold(target.checked) })
                )
                .needs_scope()
        );
        // A binding on either element is enough.
        assert!(toggle().attr("data-live", "{now}").needs_scope());
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding_on_the_input() {
        let markup = toggle()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setBold(target.checked) }),
            )
            .render(Html::empty())
            .into_test_string();

        let input = markup.split("<input ").nth(1).unwrap();
        assert!(
            input.contains(r#"pp-onchange="setBold(target.checked)""#),
            "{markup}"
        );
    }

    /// A literal `pressed` is an uncontrolled default; a bound `checked` is
    /// controlled. Writing both leaves one attribute, not two.
    #[test]
    fn a_bound_checked_replaces_a_literal_pressed() {
        let markup = toggle()
            .pressed(true)
            .bind(
                "checked",
                crate::components::rahti_ui::attrs::ui_expr!(checked = { bold }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"checked="{bold}""#), "{markup}");
        assert!(!markup.contains(r#" checked="""#), "{markup}");
    }

    /// Two elements, two attribute sets, and each builder says which it means.
    #[test]
    fn the_two_attribute_sets_stay_apart() {
        let markup = toggle()
            .attr("title", "Bold")
            .input_attr("form", "editor")
            .render(Html::empty())
            .into_test_string();

        let (label, input) = markup.split_once("<input ").unwrap();
        assert!(label.contains(r#"title="Bold""#), "{markup}");
        assert!(!label.contains("form="), "{markup}");
        assert!(input.contains(r#"form="editor""#), "{markup}");
        assert!(!input.contains("title="), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = toggle()
            .attr("data-slot", "mine")
            .input_attr("data-slot", "mine-too")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"data-slot="mine-too""#), "{markup}");
        assert!(!markup.contains(r#"data-slot="toggle""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = toggle()
            .attrs(Attrs::new().unset("data-slot").set("title", "Bold"))
            .input_attrs(Attrs::new().set("aria-label", "Bold"))
            .render(Html::empty())
            .into_test_string();

        let (label, input) = markup.split_once("<input ").unwrap();
        assert!(!label.contains("data-slot"), "{markup}");
        assert!(label.contains(r#"title="Bold""#), "{markup}");
        assert!(input.contains(r#"aria-label="Bold""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = toggle()
            .value(r#"a" pp-onfocus="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn toggle_variants_is_the_components_own_class_list() {
        assert_eq!(
            toggle_variants(ToggleVariant::Outline, ToggleSize::Lg, "rounded-full"),
            toggle()
                .variant(ToggleVariant::Outline)
                .size(ToggleSize::Lg)
                .class("rounded-full")
                .class_list()
        );
    }

    /// The call form behind `<Toggle variant="outline" size="lg" pressed>`.
    #[test]
    fn the_component_wraps_the_same_elements() {
        let markup = Toggle(
            "",
            "outline",
            "lg",
            "bold",
            "bold",
            "on",
            true,
            false,
            false,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Bold"),
        )
        .into_test_string();

        assert!(markup.contains("border-input"), "{markup}");
        assert!(markup.contains("min-w-10"), "{markup}");
        assert!(markup.contains(r#"name="bold""#), "{markup}");
        assert!(markup.contains(r#" checked="""#), "{markup}");
        assert!(markup.ends_with("Bold</label>"), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Toggle>"Bold"</Toggle>`
    /// compiles to now that `html!` closes the props literal with
    /// `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_toggle() {
        let markup = Toggle(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            bool::default(),
            bool::default(),
            bool::default(),
            bool::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::escape("Bold"),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="toggle""#), "{markup}");
        assert!(markup.contains("h-9"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(r#"value="""#), "{markup}");
        assert!(!markup.contains(" checked="), "{markup}");
    }

    /// What `<Toggle checked={bold} onchange={setBold(target.checked)}>`
    /// compiles to. The input is below the label, so the caller's bindings
    /// are lifted: the handler onto the label, where `change` bubbles and runs
    /// in the caller's scope, and the value onto it as a prop the input reads
    /// back through `pp.props` in this component's own scope.
    #[test]
    fn caller_bindings_are_lifted_onto_the_label() {
        let markup = Toggle(
            "",
            "",
            "",
            "",
            "bold",
            "",
            false,
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[
                ("onchange", "setBold(target.checked)"),
                ("checked", "bold"),
            ]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        let label = &markup[..markup.find("<input").unwrap()];
        let input = &markup[markup.find("<input").unwrap()..];
        assert!(
            label.contains(r#"pp-onchange="setBold(target.checked)""#),
            "{markup}"
        );
        assert!(label.contains(r#"checked="{bold}""#), "{markup}");
        assert!(
            input.contains(r#"checked="{pp.props.checked}""#),
            "{markup}"
        );
        assert!(!input.contains("onchange"), "{markup}");
        // The scope `pp.props` is read in, inside the label.
        assert!(markup.ends_with("<script></script></label>"), "{markup}");
    }

    /// `ToggleProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_toggle() {
        assert_eq!(ToggleProps::new(), ToggleProps::default());
        assert_eq!(
            toggle().render(Html::empty()).into_test_string(),
            ToggleProps::new().render(Html::empty()).into_test_string()
        );
    }
}

/// The layer-independence check behind [`BASE`].
///
/// Three layers — the base, the variant, the size — and the claim is
/// [`Button`](super::button)'s: no two utilities in a rendered class list
/// compete, so joining them is what merging them would give.
///
/// All six combinations are checked, because the claim is about the join and not
/// about any one string. This is the component where that check earned its
/// keep: shadcn's own layering *does* clash here — the base's
/// `hover:bg-muted` against `outline`'s `hover:bg-accent` — and the audit is
/// what says so, which is why [`ToggleVariant::class`] carries the hover state
/// instead.
#[cfg(test)]
mod audit {
    use super::{BASE, ToggleSize, ToggleVariant, component_classes};
    use crate::components::rahti_ui::utils::audit::clashes;

    const VARIANTS: [ToggleVariant; 2] = [ToggleVariant::Default, ToggleVariant::Outline];
    const SIZES: [ToggleSize; 3] = [ToggleSize::Default, ToggleSize::Sm, ToggleSize::Lg];

    #[test]
    fn no_utility_contradicts_another() {
        for variant in VARIANTS {
            for size in SIZES {
                let found = clashes(&component_classes(variant, size));

                assert!(
                    found.is_empty(),
                    "{}/{}: {} clashes:\n{}",
                    variant.slug(),
                    size.slug(),
                    found.len(),
                    found.join("\n")
                );
            }
        }
    }

    /// Each layer is also clean on its own, so a failure above points at the
    /// join rather than at one string.
    #[test]
    fn no_layer_contradicts_itself() {
        let found = clashes(BASE);
        assert!(found.is_empty(), "BASE: {}", found.join("\n"));

        for variant in VARIANTS {
            let found = clashes(variant.class());
            assert!(found.is_empty(), "{}: {}", variant.slug(), found.join("\n"));
        }
        for size in SIZES {
            let found = clashes(size.class());
            assert!(found.is_empty(), "{}: {}", size.slug(), found.join("\n"));
        }
    }
}
