// rahti-ui: checkbox
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Checkbox — a port of shadcn/ui's `checkbox`.
//!
//! The class string is shadcn's and the rendered box is the same box. What
//! changes is the element under it, and that is a bigger change than the other
//! components in this library needed — so it is written down here rather than
//! discovered later.
//!
//! ## Why this is a native `<input>`
//!
//! shadcn's Checkbox is a Radix `CheckboxPrimitive.Root`, which renders a
//! `<button role="checkbox" aria-checked data-state>` with an Indicator child,
//! plus a hidden `<input>` beside it so a form has something to post. The
//! visible control is a button, and a button does not check itself: every one
//! of those boxes needs JavaScript to tick.
//!
//! This library has spent three components on the opposite bet — [`Input`],
//! [`Textarea`] and [`Button`](super::button) all render the native element and
//! leave the primitive behind, so a control styled by the server stays styled
//! and *usable* with JavaScript switched off. A checkbox is where that bet
//! costs something, and it is still the right bet: an `<input type="checkbox">`
//! ticks, submits, focuses, and reads correctly to a screen reader with no
//! runtime at all.
//!
//! Three things follow, and they are the whole of the delta:
//!
//! 1. **`data-[state=checked]:` becomes `checked:`.** Radix keeps its state in
//!    a data attribute because a `<button>` has none; a native checkbox has
//!    `:checked`. Same three declarations, native selector.
//! 2. **`appearance-none` is added.** shadcn does not need it — it is styling a
//!    `<button>`. A checkbox draws its own box, and shadcn's `border`,
//!    `rounded-[4px]` and `bg-*` have nothing to sit on until the UA's box is
//!    gone.
//! 3. **The Indicator becomes a background image.** shadcn renders
//!    `<CheckIcon className="size-3.5" />` inside a
//!    `data-slot="checkbox-indicator"` element. An `<input>` is void — it can
//!    have no children — so the tick is lucide's `check` at 14px, drawn as a
//!    `background-image` that appears on `checked:`. It is the same path at the
//!    same size and the same stroke width; what it is not is an element, so
//!    there is no `checkbox-indicator` slot to select.
//!
//! Everything else — every colour, every ring, every `dark:` — is shadcn's,
//! copied rather than reinterpreted.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Checkbox />
//! <Checkbox id="terms" name="terms" checked />
//! <Checkbox disabled />
//! <Checkbox invalid />
//! <Checkbox indeterminate />
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is this component's default.
//!
//! ## `peer` is there for the Label
//!
//! shadcn's class list opens with `peer`, and it is not decoration: it is what
//! lets a [`Label`](super::label) written beside the box read its disabled
//! state through `peer-disabled:opacity-50`. The two components are built to
//! sit next to each other, and a plain tag is enough:
//!
//! ```ignore
//! <div class="flex items-center gap-2">
//!     <Checkbox id="terms" disabled />
//!     <Label for="terms">"Accept the terms"</Label>
//! </div>
//! ```
//!
//! `peer-disabled:` compiles to `.peer:disabled ~ &`, a combinator, so it only
//! reaches a real sibling — which is why both components render as one literal
//! element rather than as `<>@{element}</>`. A fragment root has no element to
//! carry its block's name, so `html!` delimits it with comments and the runtime
//! turns those into a `<pp-fragment>` at hydration; the box would then stop
//! being the label's sibling as soon as the page mounted. See the note at the
//! bottom of [`Checkbox`], and the same one in [`Label`](super::label).
//!
//! The `group-data-` route was never affected either way — a descendant
//! selector reaches through anything — and it is the one shadcn's own `Field`
//! uses.
//!
//! ## Indeterminate is an attribute here, not a property
//!
//! A native checkbox's third state lives in `HTMLInputElement.indeterminate`, a
//! DOM property with no attribute behind it: there is nothing a server can
//! write that turns it on. Since `appearance-none` has already replaced the
//! UA's rendering, nothing is lost by not setting it —
//! [`indeterminate`](CheckboxProps::indeterminate) writes
//! `data-indeterminate="true"` for the dash, and `aria-checked="mixed"` so
//! assistive technology is told the same thing the dash says.
//!
//! What it cannot do is stay true. A user clicking a mixed box makes it
//! checked, and the attribute the server wrote does not know. A tri-state box
//! that a user drives is state, and state is the caller's — bind it.
//!
//! ## The box is UI. The state is the caller's.
//!
//! `onchange={…}` and `checked={…}` are native attributes, and this component
//! does nothing with them but put them on the `<input>`. As [`Input`]
//! describes, a component's root attributes are evaluated where the tag was
//! written, so a tag with `checked={agreed}` is one the page drives. A bound
//! control renders as one literal element, so `peer` still reaches across.
//!
//! ## When the tag is not enough
//!
//! **[`CheckboxProps::render`], interpolated — for a box built in Rust, or a
//! class override that `peer` must reach across.**
//!
//! ```ignore
//! @{checkbox()
//!     .id("terms")
//!     .bind("checked", crate::components::rahti_ui::attrs::ui_expr!(checked={agreed}))
//!     .on("change", crate::components::rahti_ui::attrs::ui_handler!({setAgreed(target.checked)}))
//!     .render()}
//! ```
//!
//! **[`checkbox_variants`] — the class list alone.**
//!
//! ```ignore
//! <input type="checkbox" class=@{checkbox_variants("size-5")} />
//! ```
//!
//! shadcn has no `checkboxVariants` export of its own; this is the same escape
//! hatch [`button_variants`](super::button::button_variants) is, under the name
//! that matches it.
//!
//! [`Input`]: super::input
//! [`Textarea`]: super::textarea

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// Everything a checkbox is, minus the tick.
///
/// shadcn's string, with `data-[state=checked]:` read as `checked:` and
/// `appearance-none` added — see the module docs for why those two, and why
/// nothing else moved. One layer, like [`Input`](super::input)'s: no variants,
/// no sizes, nothing for a merge to resolve.
///
/// `bg-center bg-no-repeat` belongs to the tick rather than to the box, but it
/// is true of every tick this component can draw, so it is written once here
/// rather than three times below.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const BASE: &str = "peer size-4 shrink-0 appearance-none rounded-[4px] border border-input \
                    bg-center bg-no-repeat shadow-xs transition-shadow outline-none \
                    focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 \
                    disabled:cursor-not-allowed disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                    checked:border-primary checked:bg-primary checked:text-primary-foreground \
                    dark:bg-input/30 dark:aria-invalid:ring-destructive/40 dark:checked:bg-primary";

/// The tick: lucide's `check`, the icon shadcn's Indicator renders, as an
/// image instead of an element.
///
/// The path, the 24-unit box and the stroke width are lucide's; `width`/
/// `height` are baked into the SVG at 14px so `size-3.5` needs no
/// `background-size` utility to go with it.
///
/// Two of them, because a data URI cannot read a CSS variable.
/// `--primary-foreground` is near-white under `:root` and near-black under
/// `.dark`, so the stroke is spelled out for each — the one place in this
/// library where a `dark:` utility carries a colour rather than a token.
///
/// Written as arbitrary values rather than a rule in `globals.css` on purpose:
/// a stylesheet the class list cannot see is a rule `twMerge` cannot resolve,
/// which is the same objection `input.rs` raises to shadcn's Base UI variants.
///
/// Two rules govern how these are spelled, and both were learned by breaking
/// them:
///
/// - **One utility per source line.** Tailwind scans source text for class
///   names, and would not find one broken across two lines by a `\`
///   continuation.
/// - **Spaces are `%20`, not `_`.** An arbitrary value cannot contain
///   whitespace, and Tailwind's usual answer — write `_`, get a space — is
///   switched off inside `url()`, where underscores are common and meant
///   literally. A `_` here survives into the stylesheet as a `_`, which makes
///   the markup `<svg_xmlns=…`: not an SVG, and not a tick.
const CHECK: &str = "checked:bg-[url('data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2714%27%20height=%2714%27%20viewBox=%270%200%2024%2024%27%20fill=%27none%27%20stroke=%27%23fafafa%27%20stroke-width=%272%27%20stroke-linecap=%27round%27%20stroke-linejoin=%27round%27%3e%3cpath%20d=%27M20%206%209%2017l-5-5%27/%3e%3c/svg%3e')] \
                     dark:checked:bg-[url('data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2714%27%20height=%2714%27%20viewBox=%270%200%2024%2024%27%20fill=%27none%27%20stroke=%27%23262626%27%20stroke-width=%272%27%20stroke-linecap=%27round%27%20stroke-linejoin=%27round%27%3e%3cpath%20d=%27M20%206%209%2017l-5-5%27/%3e%3c/svg%3e')]";

/// The dash, for the third state: lucide's `minus`, drawn the same way as
/// [`CHECK`] and keyed off `data-indeterminate` rather than `:checked`,
/// because a native checkbox has no selector for it. See the module docs.
const INDETERMINATE: &str = "data-[indeterminate=true]:border-primary \
                             data-[indeterminate=true]:bg-primary \
                             data-[indeterminate=true]:text-primary-foreground \
                             data-[indeterminate=true]:bg-[url('data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2714%27%20height=%2714%27%20viewBox=%270%200%2024%2024%27%20fill=%27none%27%20stroke=%27%23fafafa%27%20stroke-width=%272%27%20stroke-linecap=%27round%27%20stroke-linejoin=%27round%27%3e%3cpath%20d=%27M5%2012h14%27/%3e%3c/svg%3e')] \
                             dark:data-[indeterminate=true]:bg-[url('data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2714%27%20height=%2714%27%20viewBox=%270%200%2024%2024%27%20fill=%27none%27%20stroke=%27%23262626%27%20stroke-width=%272%27%20stroke-linecap=%27round%27%20stroke-linejoin=%27round%27%3e%3cpath%20d=%27M5%2012h14%27/%3e%3c/svg%3e')]";

/// Every utility a checkbox carries, before the caller's own.
fn component_classes() -> String {
    cn(&[BASE, CHECK, INDETERMINATE])
}

/// The class list shadcn's `cn(…, className)` returns: the box's own
/// utilities, then the caller's, merged so the caller's win.
///
/// Reach for this when the element has to belong to the page — a box whose
/// handler reads the page's state, or one that has to be a real `peer` of a
/// label beside it.
pub fn checkbox_variants(class: &str) -> String {
    cn(&[&component_classes(), class])
}

/// A checkbox's props, with shadcn's defaults already in place.
///
/// Build one with [`checkbox`] and narrow it down a method at a time. Nothing
/// here is required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose, and resolving that is the browser's job.
/// Everything else a checkbox carries — `name`, `value`, `checked` — is an
/// ordinary attribute, and lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckboxProps {
    class: String,
    attrs: Attrs,
}

/// A default checkbox's props — the start of every chain.
pub fn checkbox() -> CheckboxProps {
    CheckboxProps::default()
}

impl CheckboxProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the box's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    pub fn name(self, name: &str) -> Self {
        self.attr("name", name)
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// What this box posts when it is ticked. A checkbox with no `value`
    /// submits `on`, which is HTML's rule rather than this component's to
    /// restate.
    pub fn value(self, value: &str) -> Self {
        self.attr("value", value)
    }

    /// An uncontrolled default. A box PulsePoint keeps in step with state
    /// wants [`bind`](Self::bind) — `bind("checked", "agreed")` — instead; see
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

    /// The third state: a dash instead of a tick.
    ///
    /// `data-indeterminate="true"` is what [`INDETERMINATE`] paints, and
    /// `aria-checked="mixed"` is the same fact told to assistive technology.
    /// The DOM property of that name is not set and cannot be — see the module
    /// docs — and neither attribute survives a user clicking the box.
    pub fn indeterminate(self, indeterminate: bool) -> Self {
        if indeterminate {
            self.attr("data-indeterminate", "true")
                .attr("aria-checked", "mixed")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` — which is what the `aria-invalid:` utilities in
    /// [`BASE`] paint the box with. The styling and the accessible state are
    /// the same fact, spelled once.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.attr("aria-invalid", "true")
        } else {
            self
        }
    }

    /// Any attribute, rendered on the server: `attr("form", "signup")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("autofocus")`.
    ///
    /// `name=""` is how HTML spells a present boolean attribute, and it is
    /// what `html!` itself emits for `autofocus=@{true}`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("checked", "agreed")`.
    ///
    /// Through the [`Checkbox`] tag the same binding is a `{…}` prop: it lands
    /// on the root, and a component's root attributes are evaluated where the
    /// tag was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("change", "setAgreed(target.checked)")`.
    ///
    /// The `on` prefix is optional — `on("change", …)` and `on("onchange", …)`
    /// are the same attribute. Use native lowercase event names, not the JSX
    /// spellings; a checkbox reports itself through `target.checked`, not
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

    /// The class list this checkbox renders with, the caller's own included.
    ///
    /// A join, not a merge: the three layers contradict each other nowhere, so
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
    ///
    /// Handed to `html!` as `@{..attrs}` — see [`render`](Self::render) — which
    /// is where names are validated and values escaped.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new()
            .set("type", "checkbox")
            .set("data-slot", "checkbox");

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the box is styled on arrival,
        // JavaScript or no JavaScript. See the cost named in [`Self::render`].
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
    /// Interpolating this into a block puts the box in that block's scope,
    /// which is what a handler or a binding written here needs — and makes it a
    /// real sibling of anything beside it, which is what `peer-` needs. See the
    /// module docs.
    ///
    /// `<input>` is void, so this takes no children and renders no closing tag.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. [`Checkbox`] adds one when it needs to
    /// ([`needs_scope`](Self::needs_scope)); markup interpolated into a page
    /// helper that has neither script nor state is never mounted, and an
    /// override there silently does nothing.
    pub fn render(&self) -> Html {
        let attrs = self.attr_list();
        html! {
            <input @{..attrs} />
        }
    }
}

/// A checkbox.
///
/// ```ignore
/// <Checkbox />
/// <Checkbox id="terms" name="terms" checked />
/// <Checkbox disabled />
/// <Checkbox invalid />
/// <Checkbox indeterminate />
/// <Checkbox class="size-5" />
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every one
/// is optional — `html!` closes a props literal with `..Default::default()`, so
/// an unwritten prop is this component's default.
///
/// Native events are written as native events and land on the `<input>`
/// untouched. So does an attribute binding, which reads the state of the page
/// that wrote the tag:
///
/// ```ignore
/// // Reads and writes the page's `agreed`.
/// <Checkbox checked={agreed} onchange={setAgreed(target.checked)} />
/// ```
///
/// A bound box renders as one literal element with no script beside it, so a
/// [`Label`](super::label) beside it still reads it through `peer`. See the
/// module docs.
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with no
/// `{…}` in them, and names a Rust identifier cannot spell:
///
/// ```ignore
/// <Checkbox attrs=@{Attrs::new().set("form", "signup").set("aria-label", "Terms")} />
/// ```
///
/// The fragment root is what carries the component's identity — a
/// `#[component]` has to end in `html!`, and the element itself comes back from
/// [`CheckboxProps::render`] already built.
#[component]
// A component's parameters are its props, and eleven is what a checkbox has.
// The lint is measuring a function; the call site is a tag, where each of these
// is named and optional.
#[allow(clippy::too_many_arguments)]
pub fn Checkbox(
    class: &str,
    name: &str,
    id: &str,
    value: &str,
    checked: bool,
    disabled: bool,
    required: bool,
    indeterminate: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
) -> Html {
    let mut props = CheckboxProps::default()
        .class(class)
        .checked(checked)
        .disabled(disabled)
        .required(required)
        .indeterminate(indeterminate)
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

    // `onchange={…}`, `checked={…}` and friends. `html!` hands every `{…}` prop
    // over as the expression the author wrote, and it goes onto the input
    // unchanged — the component decides nothing about it. Applied last, so a
    // controlled `checked={…}` replaces an uncontrolled `checked` rather than
    // sitting beside it.
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
        let checkbox = props.render();
        html! {
            <>
                @{checkbox}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{checkbox}</>`. A fragment root has no
        // element to carry this block's name, so it is delimited by
        // `<!--pp:id-->…<!--/pp-->` comments — and at hydration the runtime
        // turns that comment range into a real `<pp-fragment>`. Comments are
        // not elements and a wrapper is, so `peer` stops reaching a Label
        // beside the box the moment PulsePoint mounts. Written as this block's
        // own root, the name lands on the `<input>` and it stays a real
        // sibling. See [`Label`](super::label) for the same note.
        let attrs = props.attr_list();
        html! {
            <input @{..attrs} />
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Checkbox, CheckboxProps, checkbox, checkbox_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_checkbox_carries_shadcns_defaults() {
        let markup = checkbox().render().into_test_string();

        assert!(markup.starts_with("<input "), "{markup}");
        assert!(markup.contains(r#"type="checkbox""#), "{markup}");
        assert!(markup.contains(r#"data-slot="checkbox""#), "{markup}");
        assert!(markup.contains("size-4"), "{markup}");
        assert!(markup.contains("border-input"), "{markup}");
        assert!(markup.contains("rounded-[4px]"), "{markup}");
        // Void: no closing tag.
        assert!(!markup.contains("</input>"), "{markup}");
    }

    /// The three deltas from shadcn's Radix version, as assertions rather than
    /// prose. Each one is a decision the module docs argue for.
    #[test]
    fn the_port_swaps_the_primitive_for_the_native_control() {
        let markup = checkbox().render().into_test_string();

        // 1. Radix's data attribute becomes the native selector.
        assert!(markup.contains("checked:bg-primary"), "{markup}");
        assert!(!markup.contains("data-[state=checked]"), "{markup}");
        // 2. The UA's own box is turned off so shadcn's can show.
        assert!(markup.contains("appearance-none"), "{markup}");
        // 3. The Indicator is an image, so there is no indicator element.
        assert!(markup.contains("checked:bg-[url("), "{markup}");
        assert!(!markup.contains("checkbox-indicator"), "{markup}");
    }

    /// `peer` is shadcn's, and it is the whole reason a Label beside the box
    /// can dim with it.
    #[test]
    fn the_class_list_opens_with_peer() {
        assert!(checkbox().class_list().starts_with("peer "));
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = checkbox()
            .checked(true)
            .disabled(true)
            .required(true)
            .invalid(true)
            .render()
            .into_test_string();

        assert!(markup.contains(r#" checked="""#), "{markup}");
        assert!(markup.contains(r#" disabled="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
        assert!(markup.contains(r#"aria-invalid="true""#), "{markup}");

        let markup = checkbox()
            .checked(false)
            .disabled(false)
            .required(false)
            .invalid(false)
            .render()
            .into_test_string();

        // Attributes, not the `checked:`, `disabled:` and `aria-invalid:`
        // utilities that are in the class list either way — which is the whole
        // point of those three prefixes.
        assert!(!markup.contains(r#" checked="""#), "{markup}");
        assert!(!markup.contains(r#" disabled="""#), "{markup}");
        assert!(!markup.contains(r#" required="""#), "{markup}");
        assert!(!markup.contains(r#"aria-invalid=""#), "{markup}");
    }

    /// The third state is two attributes: one the stylesheet reads, one a
    /// screen reader does.
    #[test]
    fn indeterminate_is_an_attribute_and_an_aria_state() {
        let markup = checkbox().indeterminate(true).render().into_test_string();

        assert!(markup.contains(r#"data-indeterminate="true""#), "{markup}");
        assert!(markup.contains(r#"aria-checked="mixed""#), "{markup}");
        assert!(
            markup.contains("data-[indeterminate=true]:bg-primary"),
            "{markup}"
        );

        let markup = checkbox().indeterminate(false).render().into_test_string();
        assert!(!markup.contains(r#"data-indeterminate=""#), "{markup}");
        assert!(!markup.contains("aria-checked"), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = checkbox()
            .class("size-5 rounded-full")
            .render()
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("rounded-full"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_checkbox_with_no_override_needs_no_runtime() {
        let props = checkbox().id("terms").name("terms");
        let markup = props.render().into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="peer"#), "{markup}");

        assert!(!checkbox().class("size-5").needs_scope());
        assert!(
            checkbox()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ setAgreed(target.checked) })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = checkbox()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setAgreed(target.checked) }),
            )
            .render()
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setAgreed(target.checked)""#),
            "{markup}"
        );
    }

    /// A literal `checked` is an uncontrolled default; a bound one is
    /// controlled. Writing both leaves one attribute, not two.
    #[test]
    fn a_bound_checked_replaces_a_literal_one() {
        let markup = checkbox()
            .checked(true)
            .bind(
                "checked",
                crate::components::rahti_ui::attrs::ui_expr!(checked = { agreed }),
            )
            .render()
            .into_test_string();

        assert!(markup.contains(r#"checked="{agreed}""#), "{markup}");
        assert!(!markup.contains(r#" checked="""#), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = checkbox()
            .attr("data-slot", "mine")
            .attr("form", "signup")
            .render()
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"form="signup""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = checkbox()
            .attrs(Attrs::new().unset("data-slot").set("aria-label", "Terms"))
            .render()
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains(r#"aria-label="Terms""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = checkbox()
            .value(r#"a" pp-onfocus="steal()"#)
            .render()
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn checkbox_variants_is_the_components_own_class_list() {
        assert_eq!(
            checkbox_variants("size-5"),
            checkbox().class("size-5").class_list()
        );
    }

    /// The call form behind `<Checkbox id="terms" name="terms" checked />`. A
    /// tag fills the props it names and takes these defaults for the rest.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Checkbox(
            "",
            "terms",
            "terms",
            "accepted",
            true,
            false,
            true,
            false,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(markup.contains(r#"name="terms""#), "{markup}");
        assert!(markup.contains(r#"id="terms""#), "{markup}");
        assert!(markup.contains(r#"value="accepted""#), "{markup}");
        assert!(markup.contains(r#" checked="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Checkbox />` compiles to now
    /// that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_checkbox() {
        let markup = Checkbox(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            bool::default(),
            bool::default(),
            bool::default(),
            bool::default(),
            bool::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"type="checkbox""#), "{markup}");
        assert!(markup.contains(r#"data-slot="checkbox""#), "{markup}");
        assert!(markup.contains("size-4"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(r#"value="""#), "{markup}");
        assert!(!markup.contains(" checked="), "{markup}");
        assert!(!markup.contains(" disabled="), "{markup}");
    }

    /// What `<Checkbox onchange={setAgreed(target.checked)} />` compiles to.
    /// The expression arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_checkbox() {
        let markup = Checkbox(
            "",
            "",
            "terms",
            "",
            false,
            false,
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[
                ("onchange", "setAgreed(target.checked)"),
                ("checked", "agreed"),
            ]),
            Attrs::new(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setAgreed(target.checked)""#),
            "{markup}"
        );
        assert!(markup.contains(r#"checked="{agreed}""#), "{markup}");
    }

    /// `CheckboxProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_box() {
        assert_eq!(CheckboxProps::new(), CheckboxProps::default());
        assert_eq!(
            checkbox().render().into_test_string(),
            CheckboxProps::new().render().into_test_string()
        );
    }
}

/// The layer-independence check behind [`BASE`].
///
/// Three layers here rather than [`Input`](super::input)'s one — the box, the
/// tick, the dash — and the claim is [`Button`](super::button)'s: no two
/// utilities in a rendered class list compete, so joining them is what merging
/// them would give.
///
/// The interesting pairs are the `bg-`s. `checked:bg-primary` fills the box and
/// `checked:bg-[url(…)]` draws the tick over it, under the same prefix — not a
/// clash, because `bg-` is four properties and only one of them is the colour.
/// `utils::audit::family` was taught that distinction for this component, and
/// `utils` tests it directly.
#[cfg(test)]
mod audit {
    use super::{BASE, CHECK, INDETERMINATE, component_classes};
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
        for (name, layer) in [
            ("BASE", BASE),
            ("CHECK", CHECK),
            ("INDETERMINATE", INDETERMINATE),
        ] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
