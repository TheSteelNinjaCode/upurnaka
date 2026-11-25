// rahti-ui: switch
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Switch — a port of shadcn/ui's `switch`.
//!
//! shadcn's Switch is two elements: a Radix `SwitchPrimitive.Root` that draws
//! the track, and a `SwitchPrimitive.Thumb` inside it that slides. This is one
//! element, because the element it is — `<input type="checkbox" role="switch">`
//! — is void and can hold no thumb.
//!
//! ## Why this is a native checkbox
//!
//! The same bet [`Checkbox`](super::checkbox) makes, and the same reasoning: a
//! Radix Root is a `<button role="switch">`, and a button does not toggle
//! itself. A native checkbox toggles, submits, focuses, takes the space bar, and
//! is read correctly with no runtime at all.
//!
//! `role="switch"` is what makes it a switch rather than a checkbox to a screen
//! reader — "on"/"off" instead of "checked"/"unchecked". It is Radix's own role,
//! written here on the element HTML already gives the behaviour to.
//!
//! ## Four deltas, and one of them is the whole component
//!
//! 1. **The Thumb becomes a background image that moves.** shadcn's Thumb is a
//!    `<span>` translated by `translate-x-[calc(100%-2px)]`. An `<input>` is
//!    void, so the thumb here is a circle drawn with `radial-gradient` at
//!    `bg-left`, moved to `bg-right` on `checked:`. shadcn's `transition-all` on
//!    the track animates it, exactly as it animates the colour.
//!
//!    The geometry is not an approximation of shadcn's, it is the same
//!    arithmetic. `border border-transparent` makes the positioning area the
//!    padding box, one pixel inside the track. `bg-left` is therefore where the
//!    Thumb's `translate-x-0` puts it, and `bg-right` — 30px content minus a
//!    16px thumb — is 14px, which is what `calc(100% - 2px)` evaluates to.
//!
//!    Two consequences follow. The thumb is a background, so it reads
//!    `var(--background)` and its two `dark:` colours as tokens rather than
//!    baked hex — the trick [`RadioGroup`](super::radio_group)'s dot uses and
//!    [`Checkbox`](super::checkbox)'s tick cannot. And it is a gradient rather
//!    than a `::before`, which would have modelled shadcn's translate more
//!    directly: a pseudo-element on a replaced element is undefined by the
//!    spec, and every other icon in this library is already a background.
//! 2. **`data-[state=checked]:` becomes `checked:`, and
//!    `data-[state=unchecked]:` becomes `not-checked:`.** Radix keeps its state
//!    in a data attribute because a `<button>` has none. A native checkbox has
//!    `:checked`, and Tailwind's `not-*` is the other half of it. Written as a
//!    bare `bg-input` instead, the unchecked colour would be in a specificity
//!    race with `dark:` that the stylesheet's order decides; `not-checked:`
//!    states the condition and there is no race.
//! 3. **`appearance-none` is added.** shadcn does not need it — it is styling a
//!    `<button>`. A checkbox draws its own box, and none of shadcn's track has
//!    anywhere to sit until that is gone.
//! 4. **Two utilities are dropped**: `items-center`, which centres a child there
//!    is none of, and `group/switch`, which existed so the Thumb could read the
//!    Root's size. There is no Thumb to tell, so the two `bg-[size:…]` utilities
//!    read `data-size` directly.
//!
//! Everything else — every colour, every ring, every `dark:` — is shadcn's,
//! copied rather than reinterpreted. Including what is *not* there: shadcn's
//! switch has no `aria-invalid` utilities, so neither does this, and there is no
//! `invalid` prop to write one with.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Switch />
//! <Switch id="airplane" name="airplane" checked />
//! <Switch size="sm" />
//! <Switch disabled />
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is this component's default.
//!
//! ## `peer` is there for the Label
//!
//! shadcn's class list opens with `peer`, and it is not decoration: it is what
//! lets a [`Label`](super::label) written beside the switch read its disabled
//! state through `peer-disabled:opacity-50`. `peer-disabled:` compiles to
//! `.peer:disabled ~ &`, a combinator, so it only reaches a real sibling — which
//! is why both components render as one literal element rather than as
//! `<>@{element}</>`. See the note at the bottom of [`Switch`], and the same one
//! in [`Checkbox`](super::checkbox).
//!
//! ## The switch is UI. The state is the caller's.
//!
//! `onchange={…}` and `checked={…}` are native attributes, and this component
//! does nothing with them but put them on the `<input>`. As
//! [`Input`](super::input) describes, a component's root attributes are
//! evaluated where the tag was written, so a tag with `checked={airplane}` is
//! one the page drives. A bound control renders as one literal element, so
//! `peer` still reaches across.
//!
//! ## When the tag is not enough
//!
//! **[`SwitchProps::render`], interpolated — for a switch built in Rust, or a
//! class override that `peer` must reach across.**
//!
//! ```ignore
//! @{switch_props()
//!     .id("airplane")
//!     .bind("checked", crate::components::rahti_ui::attrs::ui_expr!(checked={airplane}))
//!     .on("change", crate::components::rahti_ui::attrs::ui_handler!({setAirplane(target.checked)}))
//!     .render()}
//! ```
//!
//! **[`switch_variants`] — the class list alone.**
//!
//! ```ignore
//! <input type="checkbox" role="switch" data-size="default"
//!        class=@{switch_variants(SwitchSize::Default, "")} />
//! ```

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// The track: everything the switch is, minus the thumb.
///
/// shadcn's Root string, with the two dropped utilities named in the module docs
/// left out and `appearance-none` and `bg-no-repeat` added.
///
/// `data-[size=default]:h-[1.15rem]` and friends are shadcn's own way of
/// spelling the two sizes — a data attribute the class list reads, not a second
/// layer of classes — so [`SwitchSize`] writes `data-size` and every size
/// utility stays here under its own prefix. Nothing to merge.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const BASE: &str = "peer inline-flex shrink-0 appearance-none rounded-full \
                    border border-transparent bg-no-repeat shadow-xs transition-all outline-none \
                    focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 \
                    disabled:cursor-not-allowed disabled:opacity-50 \
                    data-[size=default]:h-[1.15rem] data-[size=default]:w-8 \
                    data-[size=sm]:h-3.5 data-[size=sm]:w-6 \
                    checked:bg-primary not-checked:bg-input \
                    dark:not-checked:bg-input/80";

/// The thumb: shadcn's `<SwitchPrimitive.Thumb>`, as a background instead of an
/// element.
///
/// A circle filling its own tile, so one gradient serves both sizes — `50%` of a
/// 16px tile is an 8px radius and `50%` of a 12px one is 6px. The half-pixel
/// feather at the edge is what a `border-radius` would have given for free.
///
/// Three of them, because the thumb is three colours: `bg-background` in light,
/// and shadcn's two `dark:` overrides. Unlike [`Checkbox`](super::checkbox)'s
/// tick these stay tokens — a gradient can read a CSS variable where a data URI
/// cannot — so a theme change moves the thumb's colour with it.
///
/// `bg-left` and `checked:bg-right` are the movement, and shadcn's
/// `transition-all` on [`BASE`] is what makes it a slide rather than a jump.
///
/// One rule governs how these are spelled: **one utility per source line**,
/// because Tailwind scans source text for class names and would not find one
/// broken across two lines by a `\` continuation.
const THUMB: &str = "bg-left checked:bg-right \
                     data-[size=default]:bg-[size:1rem_1rem] data-[size=sm]:bg-[size:0.75rem_0.75rem] \
                     bg-[radial-gradient(circle_at_center,var(--background)_0_calc(50%_-_0.5px),transparent_50%)] \
                     dark:not-checked:bg-[radial-gradient(circle_at_center,var(--foreground)_0_calc(50%_-_0.5px),transparent_50%)] \
                     dark:checked:bg-[radial-gradient(circle_at_center,var(--primary-foreground)_0_calc(50%_-_0.5px),transparent_50%)]";

/// Every utility a switch carries, before the caller's own.
fn component_classes() -> String {
    cn(&[BASE, THUMB])
}

/// How big it is.
///
/// shadcn's two sizes, kept the way shadcn keeps them: a `data-size` attribute
/// that [`BASE`] and [`THUMB`] read, rather than a class layer of its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchSize {
    #[default]
    Default,
    Sm,
}

impl SwitchSize {
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
                    "rahti-ui: `{other}` is not a Switch size. One of: default, sm."
                );
                Self::Default
            }
        }
    }
}

/// The class list shadcn's `cn(…, className)` returns: the switch's own
/// utilities, then the caller's, merged so the caller's win.
///
/// The size is not in here — it is `data-size`, which the caller has to write
/// alongside — so it is taken as an argument only to make that impossible to
/// forget. Reach for this when the element has to belong to the page: a switch
/// whose handler reads the page's state, or one that has to be a real `peer` of
/// a label beside it.
pub fn switch_variants(size: SwitchSize, class: &str) -> String {
    let _ = size;
    cn(&[&component_classes(), class])
}

/// A switch's props, with shadcn's defaults already in place.
///
/// Build one with [`switch_props`] and narrow it down a method at a time.
/// Nothing here is required.
///
/// Only `class` is a field of its own besides the size: it is the one thing that
/// can conflict with what the component chose, and resolving that is the
/// browser's job. Everything else a switch carries — `name`, `value`, `checked`
/// — is an ordinary attribute, and lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SwitchProps {
    class: String,
    size: SwitchSize,
    attrs: Attrs,
}

/// A default switch's props — the start of every chain.
///
/// Named `switch_props` rather than `switch` for the reason
/// [`select_props`](super::select::select_props) is: a bare `switch` reads like
/// the element, and this returns props.
pub fn switch_props() -> SwitchProps {
    SwitchProps::default()
}

impl SwitchProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the switch's own, `className`-style: a conflicting
    /// utility written here wins.
    ///
    /// A utility carrying a variant prefix is only replaced by one carrying the
    /// same prefix — the sizes here are all `data-[size=…]:`, so an override
    /// meant to resize the track has to be prefixed too.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// `default` or `sm`, written as `data-size` for the class list to read.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    pub fn name(self, name: &str) -> Self {
        self.attr("name", name)
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// What this switch posts when it is on. A checkbox with no `value` submits
    /// `on`, which is HTML's rule rather than this component's to restate — and
    /// which reads well enough for a switch that it is worth leaving alone.
    pub fn value(self, value: &str) -> Self {
        self.attr("value", value)
    }

    /// An uncontrolled default. A switch PulsePoint keeps in step with state
    /// wants [`bind`](Self::bind) — `bind("checked", "airplane")` — instead; see
    /// the module docs.
    ///
    /// `false` leaves the attribute off rather than writing `checked="false"`,
    /// which HTML would read as checked.
    pub fn checked(self, checked: bool) -> Self {
        if checked { self.flag("checked") } else { self }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.flag("disabled")
        } else {
            self
        }
    }

    pub fn required(self, required: bool) -> Self {
        if required {
            self.flag("required")
        } else {
            self
        }
    }

    /// Any attribute, rendered on the server: `attr("form", "settings")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("autofocus")`.
    ///
    /// `name=""` is how HTML spells a present boolean attribute, and it is what
    /// `html!` itself emits for `autofocus=@{true}`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("checked", "airplane")`.
    ///
    /// Through the [`Switch`] tag the same binding is a `{…}` prop: it lands on
    /// the root, and a component's root attributes are evaluated where the tag
    /// was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("change", "setAirplane(target.checked)")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings; a switch reports itself through `target.checked`, not
    /// `target.value`.
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

    /// The class list this switch renders with, the caller's own included.
    ///
    /// A join, not a merge: the two layers contradict each other nowhere, so
    /// this is exact for everything except a caller's `.class(…)` override,
    /// which is the one thing that can conflict. That is resolved in the
    /// browser — see [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[&component_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Bindings run from the client registry compiled by `html!` and retain their caller scope.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order: the component's own first,
    /// the caller's spread last so it wins.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new()
            .set("type", "checkbox")
            .set("role", "switch")
            .set("data-slot", "switch")
            .set("data-size", self.size.slug());

        // Static class overrides compose data; they do not resolve conflicts.
        // Supply an explicit compiled `class={twMerge(...)}` binding when
        // overriding utilities from the component defaults.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&component_classes(), &self.class]),
            )
        };

        // The caller's own, last, so a `{...props}` spread wins — the same
        // last-wins rule `AttrList` applies when it writes the tag.
        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the switch in that block's scope,
    /// which is what a handler or a binding written here needs — and makes it a
    /// real sibling of anything beside it, which is what `peer-` needs. See the
    /// module docs.
    ///
    /// `<input>` is void, so this takes no children and renders no closing tag.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    pub fn render(&self) -> Html {
        let attrs = self.attr_list();
        html! {
            <input @{..attrs} />
        }
    }
}

/// A switch.
///
/// ```ignore
/// <Switch />
/// <Switch id="airplane" name="airplane" checked />
/// <Switch size="sm" />
/// <Switch disabled />
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every one is
/// optional — `html!` closes a props literal with `..Default::default()`, so an
/// unwritten prop is this component's default.
///
/// Native events are written as native events and land on the `<input>`
/// untouched. So does an attribute binding, which reads the state of the page
/// that wrote the tag:
///
/// ```ignore
/// // Reads and writes the page's `airplane`.
/// <Switch checked={airplane} onchange={setAirplane(target.checked)} />
/// ```
///
/// A bound switch renders as one literal element with no script beside it, so a
/// [`Label`](super::label) beside it still reads it through `peer`.
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with no
/// `{…}` in them, and names a Rust identifier cannot spell.
#[component]
// A component's parameters are its props, and nine is what a switch has. The
// lint is measuring a function; the call site is a tag, where each of these is
// named and optional.
#[allow(clippy::too_many_arguments)]
pub fn Switch(
    class: &str,
    size: &str,
    name: &str,
    id: &str,
    value: &str,
    checked: bool,
    disabled: bool,
    required: bool,
    bindings: Bindings,
    attrs: Attrs,
) -> Html {
    let mut props = SwitchProps::default()
        .class(class)
        .size(SwitchSize::parse(size))
        .checked(checked)
        .disabled(disabled)
        .required(required);

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

    // `onchange={…}`, `checked={…}` and friends. Applied last, so a controlled
    // `checked={…}` replaces an uncontrolled `checked` rather than sitting
    // beside it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    // Forwarded bindings retain the caller's scope. Only precompiled
    // bindings supplied through the builder attrs need a local scope when
    // no caller bindings are present; a static class override creates none.
    if props.needs_scope() && bindings.is_empty() {
        let switch = props.render();
        html! {
            <>
                @{switch}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{switch}</>`. A fragment root has no
        // element to carry this block's name, so it is delimited by
        // `<!--pp:id-->…<!--/pp-->` comments — and at hydration the runtime
        // turns that comment range into a real `<pp-fragment>`. Comments are not
        // elements and a wrapper is, so `peer` would stop reaching a Label
        // beside the switch the moment PulsePoint mounted. See the same note in
        // [`Checkbox`](super::checkbox).
        let attrs = props.attr_list();
        html! {
            <input @{..attrs} />
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Switch, SwitchProps, SwitchSize, switch_props, switch_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;

    #[test]
    fn a_default_switch_carries_shadcns_defaults() {
        let markup = switch_props().render().into_test_string();

        assert!(markup.starts_with("<input "), "{markup}");
        assert!(markup.contains(r#"type="checkbox""#), "{markup}");
        assert!(markup.contains(r#"data-slot="switch""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains("rounded-full"), "{markup}");
        assert!(markup.contains("data-[size=default]:w-8"), "{markup}");
        // Void: no closing tag.
        assert!(!markup.contains("</input>"), "{markup}");
    }

    /// The four deltas from shadcn's Radix version, as assertions rather than
    /// prose. Each one is a decision the module docs argue for.
    #[test]
    fn the_port_swaps_the_primitive_for_the_native_control() {
        let markup = switch_props().render().into_test_string();

        // 1. The Thumb is a background image that moves, so there is no thumb
        //    element to select.
        assert!(markup.contains("bg-[radial-gradient("), "{markup}");
        assert!(markup.contains("checked:bg-right"), "{markup}");
        assert!(!markup.contains("switch-thumb"), "{markup}");
        // 2. Radix's data attributes become the native selectors.
        assert!(markup.contains("checked:bg-primary"), "{markup}");
        assert!(markup.contains("not-checked:bg-input"), "{markup}");
        assert!(!markup.contains("data-[state="), "{markup}");
        // 3. The UA's own box is turned off so the track can show.
        assert!(markup.contains("appearance-none"), "{markup}");
        // 4. What existed only to talk to the Thumb is gone.
        assert!(!markup.contains("group/switch"), "{markup}");
    }

    /// A switch is a switch to a screen reader, not a checkbox — which is the
    /// one thing the native element does not say for itself.
    #[test]
    fn the_role_is_the_one_radix_writes() {
        assert!(
            switch_props()
                .render()
                .into_test_string()
                .contains(r#"role="switch""#)
        );
    }

    /// The thumb reads theme tokens rather than copies of them, which is what a
    /// data URI could not do. See the module docs.
    #[test]
    fn the_thumb_is_a_gradient_that_reads_its_tokens() {
        let markup = switch_props().render().into_test_string();

        assert!(markup.contains("var(--background)"), "{markup}");
        assert!(markup.contains("var(--foreground)"), "{markup}");
        assert!(markup.contains("var(--primary-foreground)"), "{markup}");
        // No hex anywhere: nothing here is a copy of what a token is worth.
        assert!(!markup.contains("%23"), "{markup}");
    }

    /// `peer` is shadcn's, and it is the whole reason a Label beside the switch
    /// can dim with it.
    #[test]
    fn the_class_list_opens_with_peer() {
        assert!(switch_props().class_list().starts_with("peer "));
    }

    #[test]
    fn the_size_is_a_data_attribute_the_class_list_reads() {
        let markup = switch_props()
            .size(SwitchSize::Sm)
            .render()
            .into_test_string();

        assert!(markup.contains(r#"data-size="sm""#), "{markup}");
        // Every size utility is always in the class list, under its own prefix.
        assert!(markup.contains("data-[size=sm]:w-6"), "{markup}");
        assert!(markup.contains("data-[size=default]:w-8"), "{markup}");
        assert!(
            markup.contains("data-[size=sm]:bg-[size:0.75rem_0.75rem]"),
            "{markup}"
        );

        assert_eq!(SwitchSize::parse(""), SwitchSize::Default);
        assert_eq!(SwitchSize::parse("sm"), SwitchSize::Sm);
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = switch_props()
            .checked(true)
            .disabled(true)
            .required(true)
            .render()
            .into_test_string();

        assert!(markup.contains(r#" checked="""#), "{markup}");
        assert!(markup.contains(r#" disabled="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");

        let markup = switch_props()
            .checked(false)
            .disabled(false)
            .required(false)
            .render()
            .into_test_string();

        // Attributes, not the `checked:` and `disabled:` utilities that are in
        // the class list either way — which is the whole point of those
        // prefixes.
        assert!(!markup.contains(r#" checked="""#), "{markup}");
        assert!(!markup.contains(r#" disabled="""#), "{markup}");
        assert!(!markup.contains(r#" required="""#), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = switch_props()
            .class("data-[size=default]:w-12")
            .render()
            .into_test_string();

        assert!(!markup.contains(r#"class="{twMerge(&quot;"#), "{markup}");
        assert!(markup.contains("data-[size=default]:w-12"), "{markup}");
        assert!(markup.contains("data-[size=default]:w-8"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_switch_with_no_override_needs_no_runtime() {
        let props = switch_props().id("airplane").name("airplane");
        let markup = props.render().into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="peer"#), "{markup}");

        assert!(!switch_props().class("w-12").needs_scope());
        assert!(
            switch_props()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({
                        setAirplane(target.checked)
                    })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = switch_props()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setAirplane(target.checked) }),
            )
            .render()
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setAirplane(target.checked)""#),
            "{markup}"
        );
    }

    /// A literal `checked` is an uncontrolled default; a bound one is
    /// controlled. Writing both leaves one attribute, not two.
    #[test]
    fn a_bound_checked_replaces_a_literal_one() {
        let markup = switch_props()
            .checked(true)
            .bind(
                "checked",
                crate::components::rahti_ui::attrs::ui_expr!(checked = { airplane }),
            )
            .render()
            .into_test_string();

        assert!(markup.contains(r#"checked="{airplane}""#), "{markup}");
        assert!(!markup.contains(r#" checked="""#), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = switch_props()
            .attr("data-slot", "mine")
            .attr("form", "settings")
            .render()
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"form="settings""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = switch_props()
            .attrs(
                Attrs::new()
                    .unset("role")
                    .set("aria-label", "Airplane mode"),
            )
            .render()
            .into_test_string();

        assert!(!markup.contains("role="), "{markup}");
        assert!(markup.contains(r#"aria-label="Airplane mode""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = switch_props()
            .value(r#"a" pp-onfocus="steal()"#)
            .render()
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn switch_variants_is_the_components_own_class_list() {
        assert_eq!(
            switch_variants(SwitchSize::Default, "w-12"),
            switch_props().class("w-12").class_list()
        );
    }

    /// The call form behind `<Switch id="airplane" name="airplane" checked />`.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Switch(
            "",
            "sm",
            "airplane",
            "airplane",
            "on",
            true,
            false,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(markup.contains(r#"name="airplane""#), "{markup}");
        assert!(markup.contains(r#"id="airplane""#), "{markup}");
        assert!(markup.contains(r#"value="on""#), "{markup}");
        assert!(markup.contains(r#"data-size="sm""#), "{markup}");
        assert!(markup.contains(r#" checked="""#), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Switch />` compiles to now
    /// that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_switch() {
        let markup = Switch(
            <&str>::default(),
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
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="switch""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains(r#"role="switch""#), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(r#"value="""#), "{markup}");
        assert!(!markup.contains(" checked="), "{markup}");
        assert!(!markup.contains(" disabled="), "{markup}");
    }

    /// What `<Switch onchange={setAirplane(target.checked)} />` compiles to. The
    /// expression arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_switch() {
        let markup = Switch(
            "",
            "",
            "",
            "airplane",
            "",
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[
                ("onchange", "setAirplane(target.checked)"),
                ("checked", "airplane"),
            ]),
            Attrs::new(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setAirplane(target.checked)""#),
            "{markup}"
        );
        assert!(markup.contains(r#"checked="{airplane}""#), "{markup}");
    }

    /// `SwitchProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_switch() {
        assert_eq!(SwitchProps::new(), SwitchProps::default());
        assert_eq!(
            switch_props().render().into_test_string(),
            SwitchProps::new().render().into_test_string()
        );
    }
}

/// The layer-independence check behind [`BASE`].
///
/// Two layers — the track and the thumb — and the claim is
/// [`Button`](super::button)'s: no two utilities in a rendered class list
/// compete, so joining them is what merging them would give.
///
/// This component leans on that harder than any before it, because the thumb and
/// the track are one element's background. Five `bg-` utilities share the
/// unprefixed and `checked:` prefixes, and none of them is another's answer:
/// a colour, a repeat, an image, a position and a size are five properties
/// wearing one prefix. `utils::audit::family` reads them the way tailwind-merge
/// does, and `utils` tests that reading directly.
#[cfg(test)]
mod audit {
    use super::{BASE, THUMB, component_classes};
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
        for (name, layer) in [("BASE", BASE), ("THUMB", THUMB)] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
