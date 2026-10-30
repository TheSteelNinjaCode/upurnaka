// rahti-ui: radio-group
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Radio Group — a port of shadcn/ui's `radio-group`.
//!
//! shadcn ships this as one registry item holding two components, and so does
//! this file: [`RadioGroup`] is the box the options sit in, [`RadioGroupItem`]
//! is one option. The class strings are shadcn's. What changes is the element
//! under the item, and — because the whole primitive exists to do what `name`
//! already does — what the group is *for*.
//!
//! ## Why this is a native `<input type="radio">`
//!
//! shadcn's item is a Radix `RadioGroupPrimitive.Item`: a
//! `<button role="radio" aria-checked data-state>` with an Indicator child, a
//! hidden input beside it for the form, and a Root that owns `value`,
//! `onValueChange` and a roving `tabindex` so the arrow keys move between
//! options. Every one of those parts is JavaScript reimplementing something the
//! browser already ships.
//!
//! This library has spent four components on the opposite bet — [`Input`],
//! [`Textarea`], [`Button`](super::button) and [`Checkbox`](super::checkbox) all
//! render the native element — and a radio is where it pays best. Give two
//! `<input type="radio">` the same `name` and the browser gives you, for free:
//! one selection at a time, arrow keys that move it, `Tab` that enters the group
//! once rather than option by option, a value in the form post, and the right
//! thing read out by a screen reader. Radix's Root is that list, in JavaScript.
//!
//! Four things follow, and they are the whole of the delta:
//!
//! 1. **The Indicator becomes a background image.** shadcn renders a
//!    `<CircleIcon className="size-2 fill-primary">` inside a
//!    `data-slot="radio-group-indicator"` element. An `<input>` is void — it can
//!    have no children — so the dot is drawn as a `background-image` that
//!    appears on `checked:`. Same 8px, same colour; not an element, so there is
//!    no `radio-group-indicator` slot to select.
//! 2. **`appearance-none` is added.** shadcn does not need it — it is styling a
//!    `<button>`. A radio draws its own dial, and shadcn's `border`,
//!    `rounded-full` and `bg-*` have nothing to sit on until the UA's is gone.
//! 3. **`peer` is added.** shadcn's radio item does not carry it, but its Base
//!    UI sibling does, and for the reason [`Checkbox`](super::checkbox) carries
//!    it: a [`Label`](super::label) written beside the dial reads its disabled
//!    state through `peer-disabled:`. It costs one class and it is what makes
//!    the two components sit next to each other.
//! 4. **The group is a `<div role="radiogroup">` and nothing else.** It carries
//!    `grid gap-3` and an ARIA role. It does not own the selection, because
//!    `name` does.
//!
//! Everything else — every colour, every ring, every `dark:` — is shadcn's,
//! copied rather than reinterpreted.
//!
//! ## The dot is a gradient, not a data URI
//!
//! [`Checkbox`](super::checkbox) draws its tick as a data-URI SVG, and pays for
//! it twice: a data URI cannot read a CSS variable, so the stroke colour is
//! spelled out as a hex literal once for light and once for `dark:`.
//!
//! A dot needs no SVG. `radial-gradient` is a background image that *can* read a
//! variable, so [`DOT`] is one utility carrying `var(--primary)` — the token,
//! not a copy of what the token is worth today, and no `dark:` twin to keep in
//! step with it.
//!
//! ## The group does not own the selection
//!
//! This is the one place the port changes the shape of the API rather than the
//! element under it. In React:
//!
//! ```ignore
//! <RadioGroup value={plan} onValueChange={setPlan}>
//!     <RadioGroupItem value="free" />
//! ```
//!
//! Here the `name` is what groups them, and it goes on each item:
//!
//! ```ignore
//! <RadioGroup>
//!     <RadioGroupItem name="plan" value="free" id="plan-free" checked />
//!     <RadioGroupItem name="plan" value="pro" id="plan-pro" />
//! </RadioGroup>
//! ```
//!
//! A group with no `name` on its items is not a group — it is three radios that
//! all stay on once ticked, which is the one way to get this wrong. Everything
//! else about the group is presentation: [`RadioGroup`] renders a `<div>` and
//! could be one, and `role="radiogroup"` is there so assistive technology reads
//! the options as a set.
//!
//! `required` on any one item makes the whole group required — HTML's rule
//! rather than this component's to restate — and `disabled` is per item, since
//! a group cannot reach into children it was handed already rendered.
//!
//! ## The dial is UI. The state is the caller's.
//!
//! `onchange={…}` and `checked={…}` are native attributes, and this component
//! does nothing with them but put them on the `<input>`. As [`Input`]
//! describes, a component's root attributes are evaluated where the tag was
//! written, so a tag with `checked={plan}` is one the page drives. A bound
//! control renders as one literal element, so `peer` still reaches across.
//!
//! ## When the tag is not enough
//!
//! **[`RadioGroupItemProps::render`], interpolated — for an option built in
//! Rust, or a class override that `peer` must reach across.**
//!
//! ```ignore
//! @{radio_group_item()
//!     .name("plan")
//!     .value("pro")
//!     .id("plan-pro")
//!     .bind("checked", crate::components::rahti_ui::attrs::ui_expr!(checked={plan === "pro"}))
//!     .on("change", crate::components::rahti_ui::attrs::ui_handler!({setPlan(target.value)}))
//!     .render()}
//! ```
//!
//! **[`radio_group_item_variants`] — the class list alone.**
//!
//! ```ignore
//! <input type="radio" name="plan" value="pro" class=@{radio_group_item_variants("")} />
//! ```
//!
//! shadcn has no `radioGroupVariants` export of its own; these are the same
//! escape hatch [`button_variants`](super::button::button_variants) is, under
//! the names that match it.
//!
//! [`Input`]: super::input
//! [`Textarea`]: super::textarea

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// Everything the group is: a column of options with room between them.
///
/// shadcn's string, unchanged. One layer, so there is nothing here for a merge
/// to resolve.
const GROUP_BASE: &str = "grid gap-3";

/// Everything one option is, minus the dot.
///
/// shadcn's string, with `appearance-none` and `peer` added — see the module
/// docs for why those two, and why nothing else moved. Note what is *not* here:
/// no `checked:` anything. In shadcn's version a selected radio keeps its
/// border and its background and gains a dot, and that is the look this copies.
///
/// `text-primary` is vestigial under `appearance-none` — it is what shadcn's
/// `CircleIcon` inherited before it was overruled by `fill-primary` — and is
/// kept because dropping a utility is a change and this port is not making one.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const ITEM_BASE: &str = "peer aspect-square size-4 shrink-0 appearance-none rounded-full \
                         border border-input text-primary shadow-xs \
                         transition-[color,box-shadow] outline-none \
                         focus-visible:border-ring focus-visible:ring-[3px] \
                         focus-visible:ring-ring/50 \
                         disabled:cursor-not-allowed disabled:opacity-50 \
                         aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                         dark:bg-input/30 dark:aria-invalid:ring-destructive/40";

/// The dot: shadcn's `size-2 fill-primary` circle, as an image instead of an
/// element.
///
/// 8px across (`3.5px` solid, then a half-pixel fade to `4px`) and centred by
/// the gradient itself, so unlike [`Checkbox`](super::checkbox)'s tick it needs
/// no `bg-center bg-no-repeat` beside it — a gradient is already the size of the
/// box it paints.
///
/// One utility rather than the checkbox's two, and this is the reason: a
/// `radial-gradient` is a background image that can read a CSS variable, so the
/// colour stays `var(--primary)` instead of being spelled out as a hex literal
/// for `:root` and again for `.dark`. See the module docs.
///
/// The rule the checkbox learned the hard way still applies: **one utility per
/// source line**, because Tailwind scans source text for class names and would
/// not find one broken across two lines by a `\` continuation. Underscores are
/// spaces here — this is not `url()`, where that substitution is switched off.
const DOT: &str = "checked:bg-[radial-gradient(circle_at_center,var(--primary)_0,var(--primary)_3.5px,transparent_4px)]";

/// Every utility an option carries, before the caller's own.
fn item_classes() -> String {
    cn(&[ITEM_BASE, DOT])
}

/// The class list shadcn's `cn("grid gap-3", className)` returns.
///
/// Reach for this when the group element has to belong to the page — a
/// `<fieldset>` instead of a `<div>`, say, or a container whose own attributes
/// are bound.
pub fn radio_group_variants(class: &str) -> String {
    cn(&[GROUP_BASE, class])
}

/// The class list shadcn's `cn(…, className)` returns for one option: the
/// dial's own utilities, then the caller's, merged so the caller's win.
///
/// Reach for this when the element has to belong to the page — an option whose
/// handler reads the page's state, or one that has to be a real `peer` of a
/// label beside it.
pub fn radio_group_item_variants(class: &str) -> String {
    cn(&[&item_classes(), class])
}

/// A group's props, with shadcn's defaults already in place.
///
/// Build one with [`radio_group`] and narrow it down a method at a time.
/// Nothing here is required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose, and resolving that is the browser's job.
/// Everything else lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RadioGroupProps {
    class: String,
    attrs: Attrs,
}

/// A default group's props — the start of every chain.
pub fn radio_group() -> RadioGroupProps {
    RadioGroupProps::default()
}

impl RadioGroupProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the group's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// `aria-labelledby="…"` — the id of the heading that names this set.
    ///
    /// A `role="radiogroup"` with no accessible name is a set of options nobody
    /// is told the purpose of. This is one of the two ways to give it one; the
    /// other is a `<fieldset>` with a `<legend>`, which needs
    /// [`radio_group_variants`] rather than this component. See the demo page.
    pub fn labelled_by(self, id: &str) -> Self {
        self.attr("aria-labelledby", id)
    }

    /// Any attribute, rendered on the server: `attr("aria-label", "Plan")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("hidden")`.
    ///
    /// `name=""` is how HTML spells a present boolean attribute, and it is what
    /// `html!` itself emits for `hidden=@{true}`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("role")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("data-invalid", "!valid")`.
    ///
    /// Through the [`RadioGroup`] tag the same binding is a `{…}` prop: it
    /// lands on the root, and a component's root attributes are evaluated where
    /// the tag was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("change", "setPlan(target.value)")`.
    ///
    /// The `on` prefix is optional. `change` bubbles from the options inside,
    /// so one handler here can serve the whole group — which is the nearest
    /// thing this port has to Radix's `onValueChange`.
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

    /// The class list this group renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[GROUP_BASE, &self.class])
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
            .set("role", "radiogroup")
            .set("data-slot", "radio-group");

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser.
        // With no override there is nothing to resolve, so `class` stays a
        // finished class list and the group is laid out on arrival, JavaScript
        // or no JavaScript.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[GROUP_BASE, &self.class]),
            )
        };

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the group in that block's scope,
    /// which is what a handler or a binding written here needs. A `{…}` written
    /// among the *children*, though, is compiled where those children were
    /// written — so a group whose options are the page's state wants
    /// [`radio_group_variants`] on a native element instead.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// An option's props, with shadcn's defaults already in place.
///
/// Build one with [`radio_group_item`] and narrow it down a method at a time.
/// Nothing here is required by the type — but an option with no
/// [`name`](Self::name) is not in a group, and an option with no
/// [`value`](Self::value) posts `on`. See the module docs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RadioGroupItemProps {
    class: String,
    attrs: Attrs,
}

/// A default option's props — the start of every chain.
pub fn radio_group_item() -> RadioGroupItemProps {
    RadioGroupItemProps::default()
}

impl RadioGroupItemProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the dial's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// What makes this option one of a set.
    ///
    /// Radios sharing a `name` are the group: the browser lets one of them be
    /// on, moves between them with the arrow keys, and posts the chosen
    /// `value` under this name. Radix's Root is this attribute, reimplemented.
    pub fn name(self, name: &str) -> Self {
        self.attr("name", name)
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// What this option posts when it is chosen. A radio with no `value`
    /// submits `on`, which is HTML's rule rather than this component's to
    /// restate — and which makes a whole group indistinguishable, so write one.
    pub fn value(self, value: &str) -> Self {
        self.attr("value", value)
    }

    /// An uncontrolled default: the option the group starts on. An option
    /// PulsePoint keeps in step with state wants [`bind`](Self::bind) —
    /// `bind("checked", "plan === 'pro'")` — instead.
    ///
    /// `false` leaves the attribute off rather than writing `checked="false"`,
    /// which HTML would read as checked.
    pub fn checked(self, checked: bool) -> Self {
        if checked { self.flag("checked") } else { self }
    }

    /// Per option, not per group — a group is handed its children already
    /// rendered and cannot reach into them. See the module docs.
    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.flag("disabled")
        } else {
            self
        }
    }

    /// `required` on any one option makes the whole group required, which is
    /// HTML's rule rather than this component's to restate.
    pub fn required(self, required: bool) -> Self {
        if required {
            self.flag("required")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` — which is what the `aria-invalid:` utilities in
    /// [`ITEM_BASE`] paint the dial with. The styling and the accessible state
    /// are the same fact, spelled once.
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
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("checked", "plan === 'pro'")`.
    ///
    /// Through the [`RadioGroupItem`] tag the same binding is a `{…}` prop: it
    /// lands on the root, and a component's root attributes are evaluated where
    /// the tag was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("change", "setPlan(target.value)")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings; a radio reports itself through `target.value`, and only
    /// the one being turned *on* fires `change` — the one being turned off does
    /// not, which is why a group needs no bookkeeping to stay exclusive.
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

    /// The class list this option renders with, the caller's own included.
    ///
    /// A join, not a merge: the two layers contradict each other nowhere, so
    /// this is exact for everything except a caller's `.class(…)` override,
    /// which is resolved in the browser — see [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[&item_classes(), &self.class])
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
            .set("type", "radio")
            .set("data-slot", "radio-group-item");

        // See the note in `RadioGroupProps::attr_list` — and the cost named in
        // [`Self::render`], which is louder here than it is on the group.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&item_classes(), &self.class]),
            )
        };

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the dial in that block's scope,
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

/// A set of radio options.
///
/// ```ignore
/// <RadioGroup>
///     <RadioGroupItem name="plan" value="free" id="plan-free" checked />
///     <RadioGroupItem name="plan" value="pro" id="plan-pro" />
/// </RadioGroup>
/// ```
///
/// A `<div role="radiogroup">` with `grid gap-3` on it, and no more than that.
/// The selection belongs to the `name` the options share, not to this element —
/// see the module docs, because it is the one place this port changes the API
/// rather than the element under it.
///
/// `aria_labelledby` names the set for assistive technology. A `<fieldset>` with
/// a `<legend>` is the other way to do that, and wants
/// [`radio_group_variants`] rather than this tag.
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with no
/// `{…}` in them, and names a Rust identifier cannot spell.
#[component]
pub fn RadioGroup(
    class: &str,
    id: &str,
    aria_labelledby: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = RadioGroupProps::default().class(class);

    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all — so each of these is written only when the
    // caller said something.
    if !id.is_empty() {
        props = props.id(id);
    }
    if !aria_labelledby.is_empty() {
        props = props.labelled_by(aria_labelledby);
    }

    props = props.attrs(attrs);

    // `onchange={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the element unchanged.
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
        let group = props.render(children);
        html! {
            <>
                @{group}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        let attrs = props.attr_list();
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// One option in a radio group.
///
/// ```ignore
/// <RadioGroupItem name="plan" value="free" id="plan-free" />
/// <RadioGroupItem name="plan" value="pro" id="plan-pro" checked />
/// <RadioGroupItem name="plan" value="team" disabled />
/// <RadioGroupItem name="plan" value="free" invalid />
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, plus the `name`
/// its Root would have handed down. Every one is optional — `html!` closes a
/// props literal with `..Default::default()` — but an option with no `name` is
/// not in a group.
///
/// Native events are written as native events and land on the `<input>`
/// untouched. So does an attribute binding, which reads the state of the page
/// that wrote the tag:
///
/// ```ignore
/// // Reads and writes the page's `plan`.
/// <RadioGroupItem checked={plan === "pro"} onchange={setPlan(target.value)} />
/// ```
///
/// A bound option renders as one literal element with no script beside it, so a
/// [`Label`](super::label) beside it still reads it through `peer`.
#[component]
// A component's parameters are its props, and ten is what a radio has. The lint
// is measuring a function; the call site is a tag, where each of these is named
// and optional.
#[allow(clippy::too_many_arguments)]
pub fn RadioGroupItem(
    class: &str,
    name: &str,
    id: &str,
    value: &str,
    checked: bool,
    disabled: bool,
    required: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
) -> Html {
    let mut props = RadioGroupItemProps::default()
        .class(class)
        .checked(checked)
        .disabled(disabled)
        .required(required)
        .invalid(invalid);

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

    // Applied last, so a controlled `checked={…}` replaces an uncontrolled
    // `checked` rather than sitting beside it.
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
        let item = props.render();
        html! {
            <>
                @{item}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{item}</>`. A fragment root has no
        // element to carry this block's name, so it is delimited by
        // `<!--pp:id-->…<!--/pp-->` comments — and at hydration the runtime
        // turns that comment range into a real `<pp-fragment>`. Comments are not
        // elements and a wrapper is, so `peer` would stop reaching a Label
        // beside the dial the moment PulsePoint mounted. See the same note in
        // [`Checkbox`](super::checkbox).
        let attrs = props.attr_list();
        html! {
            <input @{..attrs} />
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, RadioGroup, RadioGroupItem, RadioGroupItemProps, RadioGroupProps, radio_group,
        radio_group_item, radio_group_item_variants, radio_group_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_group_carries_shadcns_defaults() {
        let markup = radio_group().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"data-slot="radio-group""#), "{markup}");
        assert!(markup.contains(r#"role="radiogroup""#), "{markup}");
        assert!(markup.contains("grid"), "{markup}");
        assert!(markup.contains("gap-3"), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
    }

    #[test]
    fn a_default_item_carries_shadcns_defaults() {
        let markup = radio_group_item().render().into_test_string();

        assert!(markup.starts_with("<input "), "{markup}");
        assert!(markup.contains(r#"type="radio""#), "{markup}");
        assert!(
            markup.contains(r#"data-slot="radio-group-item""#),
            "{markup}"
        );
        assert!(markup.contains("size-4"), "{markup}");
        assert!(markup.contains("rounded-full"), "{markup}");
        assert!(markup.contains("border-input"), "{markup}");
        // Void: no closing tag.
        assert!(!markup.contains("</input>"), "{markup}");
    }

    /// The four deltas from shadcn's Radix version, as assertions rather than
    /// prose. Each one is a decision the module docs argue for.
    #[test]
    fn the_port_swaps_the_primitive_for_the_native_control() {
        let markup = radio_group_item().render().into_test_string();

        // 1. The Indicator is a background image, so there is no indicator
        //    element to select.
        assert!(markup.contains("checked:bg-[radial-gradient("), "{markup}");
        assert!(!markup.contains("radio-group-indicator"), "{markup}");
        // 2. The UA's own dial is turned off so shadcn's can show.
        assert!(markup.contains("appearance-none"), "{markup}");
        // 3. `peer`, so a Label beside it can read its disabled state.
        assert!(radio_group_item().class_list().starts_with("peer "));
        // 4. The group is a div with a role, and owns no state.
        let group = radio_group().render(Html::empty()).into_test_string();
        assert!(group.contains(r#"role="radiogroup""#), "{group}");
        assert!(!group.contains("name="), "{group}");
    }

    /// The dot reads the theme token rather than a copy of it, which is what a
    /// data URI could not do. See the module docs.
    #[test]
    fn the_dot_is_a_gradient_that_reads_the_primary_token() {
        let markup = radio_group_item().render().into_test_string();

        assert!(markup.contains("var(--primary)"), "{markup}");
        // One utility, not one per theme.
        assert!(!markup.contains("dark:checked:bg-["), "{markup}");
    }

    /// `name` is the group. Without it, several radios are several independent
    /// controls that each stay on once ticked.
    #[test]
    fn the_name_is_what_groups_the_options() {
        let markup = radio_group_item()
            .name("plan")
            .value("pro")
            .render()
            .into_test_string();

        assert!(markup.contains(r#"name="plan""#), "{markup}");
        assert!(markup.contains(r#"value="pro""#), "{markup}");
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = radio_group_item()
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

        let markup = radio_group_item()
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
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = radio_group_item()
            .class("size-5")
            .render()
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("size-5"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn an_option_with_no_override_needs_no_runtime() {
        let props = radio_group_item().name("plan").value("pro");
        let markup = props.render().into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="peer"#), "{markup}");

        assert!(!radio_group_item().class("size-5").needs_scope());
        assert!(
            radio_group_item()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ setPlan(target.value) })
                )
                .needs_scope()
        );
        assert!(!radio_group().needs_scope());
        assert!(
            radio_group()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ setPlan(target.value) })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = radio_group_item()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setPlan(target.value) }),
            )
            .render()
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setPlan(target.value)""#),
            "{markup}"
        );

        // `change` bubbles, so one handler on the group serves every option in
        // it — the nearest thing this port has to Radix's `onValueChange`.
        let markup = radio_group()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setPlan(target.value) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setPlan(target.value)""#),
            "{markup}"
        );
    }

    /// A literal `checked` is an uncontrolled default; a bound one is
    /// controlled. Writing both leaves one attribute, not two.
    #[test]
    fn a_bound_checked_replaces_a_literal_one() {
        let markup = radio_group_item()
            .checked(true)
            .bind(
                "checked",
                crate::components::rahti_ui::attrs::ui_expr!(checked={plan === "pro"}),
            )
            .render()
            .into_test_string();

        assert!(
            markup.contains(r#"checked="{plan === &quot;pro&quot;}""#),
            "{markup}"
        );
        assert!(!markup.contains(r#" checked="""#), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = radio_group_item()
            .attr("data-slot", "mine")
            .attr("form", "signup")
            .render()
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"form="signup""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = radio_group()
            .attrs(Attrs::new().unset("role").set("aria-label", "Plan"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("role="), "{markup}");
        assert!(markup.contains(r#"aria-label="Plan""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = radio_group_item()
            .value(r#"a" pp-onfocus="steal()"#)
            .render()
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn the_variants_helpers_are_the_components_own_class_lists() {
        assert_eq!(
            radio_group_item_variants("size-5"),
            radio_group_item().class("size-5").class_list()
        );
        assert_eq!(
            radio_group_variants("gap-6"),
            radio_group().class("gap-6").class_list()
        );
    }

    /// The call form behind `<RadioGroupItem name="plan" value="pro" checked />`.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let markup = RadioGroupItem(
            "",
            "plan",
            "plan-pro",
            "pro",
            true,
            false,
            true,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(markup.contains(r#"name="plan""#), "{markup}");
        assert!(markup.contains(r#"id="plan-pro""#), "{markup}");
        assert!(markup.contains(r#"value="pro""#), "{markup}");
        assert!(markup.contains(r#" checked="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");

        let markup = RadioGroup(
            "",
            "",
            "plan-heading",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::from_raw("<span></span>".into()),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"aria-labelledby="plan-heading""#),
            "{markup}"
        );
        assert!(markup.contains("<span></span></div>"), "{markup}");
    }

    /// Every prop left unwritten, which is what `<RadioGroupItem />` compiles to
    /// now that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_option() {
        let markup = RadioGroupItem(
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
        )
        .into_test_string();

        assert!(markup.contains(r#"type="radio""#), "{markup}");
        assert!(
            markup.contains(r#"data-slot="radio-group-item""#),
            "{markup}"
        );
        assert!(markup.contains("size-4"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(r#"value="""#), "{markup}");
        assert!(!markup.contains(" checked="), "{markup}");
        assert!(!markup.contains(" disabled="), "{markup}");
    }

    /// What `<RadioGroupItem onchange={setPlan(target.value)} />` compiles to.
    /// The expression arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_option() {
        let markup = RadioGroupItem(
            "",
            "plan",
            "plan-pro",
            "pro",
            false,
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[("onchange", "setPlan(target.value)")]),
            Attrs::new(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setPlan(target.value)""#),
            "{markup}"
        );
    }

    /// Both props types are `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_defaults_are_a_plain_group_and_a_plain_option() {
        assert_eq!(RadioGroupProps::new(), RadioGroupProps::default());
        assert_eq!(RadioGroupItemProps::new(), RadioGroupItemProps::default());
        assert_eq!(
            radio_group_item().render().into_test_string(),
            RadioGroupItemProps::new().render().into_test_string()
        );
    }
}

/// The layer-independence check behind [`ITEM_BASE`].
///
/// Two layers on the option — the dial and the dot — and one on the group, and
/// the claim is [`Button`](super::button)'s: no two utilities in a rendered
/// class list compete, so joining them is what merging them would give.
///
/// The interesting pair is `dark:bg-input/30` against the dot. Both are `bg-`
/// and neither is the other's answer: one is a colour under `dark:`, the other
/// an image under `checked:`. `utils::audit::family` reads `bg-` as the four
/// properties it actually is, and the prefixes differ besides.
#[cfg(test)]
mod audit {
    use super::{DOT, GROUP_BASE, ITEM_BASE, item_classes};
    use crate::components::rahti_ui::utils::audit::clashes;

    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("the option", item_classes()),
            ("the group", GROUP_BASE.to_string()),
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
        for (name, layer) in [("ITEM_BASE", ITEM_BASE), ("DOT", DOT)] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
