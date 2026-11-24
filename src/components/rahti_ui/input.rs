// rahti-ui: input
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Input — a port of shadcn/ui's `input`.
//!
//! The class string is shadcn's, copied rather than reinterpreted, so the
//! rendered field is pixel-identical to the React one under the same Tailwind
//! theme. What changes is how the props get there.
//!
//! shadcn ships two spellings of this component. The one ported here is the
//! utility-class version — the same one [`Button`](super::button) was ported
//! from. Its Base UI sibling moves the bulk of the styling behind a single
//! `cn-input` class defined in a stylesheet, which would take both the
//! `.class(…)` override and the layer audit below out of reach: neither a
//! class list nor `twMerge` can resolve a conflict against a rule they cannot
//! see.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Input placeholder="Email" />
//! <Input type="password" name="password" required />
//! <Input value="ada@example.com" disabled />
//! <Input invalid placeholder="Email" />
//! <Input placeholder="Search" oninput={setQuery(target.value)} />
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is this component's default.
//!
//! `r#type` is the one prop that cannot be spelled the way JSX spells it. A
//! prop names a Rust parameter, `type` is a Rust keyword, and `r#type` is what
//! Rust calls a keyword used as a name. The rendered attribute is `type`.
//!
//! ## The field is UI. The state is the caller's.
//!
//! `oninput={…}` and `value={…}` are native attributes, and this component
//! does nothing with them but put them on the `<input>` — the same way a React
//! Input spreads `onChange` onto its element without reading it.
//!
//! A literal `value="…"` is an *uncontrolled* default; `value={query}` is a
//! *controlled* binding PulsePoint keeps in step with the state named in it.
//! Write one or the other and do not switch between them after mount — that
//! rule is the runtime's, not this component's.
//!
//! ## A field wired to a page's state
//!
//! Through the tag, the way [`Button`](super::button) takes a click. PulsePoint
//! owns `value`, `checked` and the events that maintain them, and a component's
//! root attributes are evaluated where the tag was written, so they read and
//! write the calling page's state.
//!
//! So `<Input value={query} oninput={setQuery(target.value)} />` is a
//! controlled field — and so is one written between another component's tags,
//! because children read the block that wrote them.
//!
//! Two other spellings do the same, and the demo page shows both:
//!
//! - [`InputProps::render`], interpolated into the page's own block — the
//!   field is then in the page's scope, and `value`, `oninput` and
//!   `aria-invalid` all resolve;
//! - [`input_variants`] on a native `<input>`, for the same reason.
//!
//! Neither is needed for a binding; they are for a field assembled in Rust, or
//! an element this component does not render.
//!
//! ## When the tag is not enough
//!
//! **[`InputProps::render`], interpolated — for a field built in Rust.**
//!
//! ```ignore
//! @{input()
//!     .placeholder("Email")
//!     .on("input", crate::components::rahti_ui::attrs::ui_handler!({setEmail(target.value)}))
//!     .bind("aria-invalid", crate::components::rahti_ui::attrs::ui_expr!(aria-invalid={!valid}))
//!     .render()}
//! ```
//!
//! `render` returns the element and nothing around it, so interpolating it
//! puts the field in the calling block's scope, where bindings resolve.
//! [`InputProps`] is the same value the tag builds internally.
//!
//! **[`input_variants`] — the class list alone.**
//!
//! ```ignore
//! <textarea class=@{input_variants("min-h-16 py-2")}></textarea>
//! ```
//!
//! For when the element has to be something this component does not render.
//! shadcn has no `inputVariants` export of its own; this is the same escape
//! hatch `buttonVariants` is, under the name that matches it.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// Everything an input is.
///
/// shadcn writes this as three arguments to `cn()` — the field, the focus
/// ring, the invalid state — and the split is editorial rather than
/// functional: nothing in any of the three contradicts anything in the other
/// two, so concatenating them is exactly what merging them produces. They are
/// joined here for the same reason [`Button`](super::button)'s layers are kept
/// conflict-free: a field styled by the server stays styled with JavaScript
/// switched off. `audit::no_utility_contradicts_another` holds us to it.
///
/// Unlike a button, an input has no variants and no sizes, so this is the
/// whole of it — there is no second layer to disagree with.
const BASE: &str = "h-9 w-full min-w-0 rounded-md border border-input bg-transparent \
                    px-3 py-1 text-base shadow-xs transition-[color,box-shadow] \
                    outline-none selection:bg-primary selection:text-primary-foreground \
                    file:inline-flex file:h-7 file:border-0 file:bg-transparent \
                    file:text-sm file:font-medium file:text-foreground \
                    placeholder:text-muted-foreground disabled:pointer-events-none \
                    disabled:cursor-not-allowed disabled:opacity-50 md:text-sm \
                    dark:bg-input/30 focus-visible:border-ring focus-visible:ring-[3px] \
                    focus-visible:ring-ring/50 aria-invalid:border-destructive \
                    aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40";

/// The `type` values HTML gives `<input>`.
///
/// Not a restriction — the value is escaped like any other attribute and
/// nothing here is unsafe. It is a typo check: a browser silently treats
/// `type="emial"` as `type="text"`, and a field that quietly stops validating
/// email addresses is worth a `debug_assert` in development.
const KINDS: [&str; 22] = [
    "button",
    "checkbox",
    "color",
    "date",
    "datetime-local",
    "email",
    "file",
    "hidden",
    "image",
    "month",
    "number",
    "password",
    "radio",
    "range",
    "reset",
    "search",
    "submit",
    "tel",
    "text",
    "time",
    "url",
    "week",
];

/// The class list shadcn's `cn(…, className)` returns: the field's own
/// utilities, then the caller's, merged so the caller's win.
///
/// Reach for this when the element has to belong to the page — a `<textarea>`
/// styled to match, or a field whose handler reads the page's state.
pub fn input_variants(class: &str) -> String {
    cn(&[BASE, class])
}

/// An input's props, with shadcn's defaults already in place.
///
/// Build one with [`input`] and narrow it down a method at a time. Nothing
/// here is required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose, and resolving that is the browser's job.
/// Everything else an input carries — `type`, `name`, `placeholder`, `value` —
/// is an ordinary attribute, and lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputProps {
    class: String,
    attrs: Attrs,
}

/// A default input's props — the start of every chain.
pub fn input() -> InputProps {
    InputProps::default()
}

impl InputProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the field's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// `type="…"`. Left unset by default, as in shadcn — an `<input>` with no
    /// `type` is a text field, and that is HTML's rule rather than this
    /// component's to restate.
    pub fn kind(self, kind: &str) -> Self {
        debug_assert!(
            kind.is_empty() || KINDS.contains(&kind),
            "rahti-ui: `{kind}` is not an input type. A browser reads an unknown \
             one as `text`."
        );
        if kind.is_empty() {
            self
        } else {
            self.attr("type", kind)
        }
    }

    // Type shorthands, so a call site reads like the React one.
    pub fn email(self) -> Self {
        self.kind("email")
    }
    pub fn password(self) -> Self {
        self.kind("password")
    }
    pub fn search(self) -> Self {
        self.kind("search")
    }
    pub fn number(self) -> Self {
        self.kind("number")
    }
    pub fn tel(self) -> Self {
        self.kind("tel")
    }
    pub fn url(self) -> Self {
        self.kind("url")
    }
    pub fn file(self) -> Self {
        self.kind("file")
    }

    pub fn name(self, name: &str) -> Self {
        self.attr("name", name)
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    pub fn placeholder(self, placeholder: &str) -> Self {
        self.attr("placeholder", placeholder)
    }

    /// An uncontrolled default. A field PulsePoint keeps in step with state
    /// wants [`bind`](Self::bind) — `bind("value", "query")` — instead; see
    /// the module docs.
    pub fn value(self, value: &str) -> Self {
        self.attr("value", value)
    }

    /// `false` leaves the attribute off rather than writing `disabled="false"`,
    /// which HTML would read as disabled.
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

    pub fn readonly(self, readonly: bool) -> Self {
        if readonly {
            self.flag("readonly")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` — which is what the `aria-invalid:` utilities in
    /// [`BASE`] paint the field with. The styling and the accessible state are
    /// the same fact, spelled once.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.attr("aria-invalid", "true")
        } else {
            self
        }
    }

    /// Any attribute, rendered on the server: `attr("autocomplete", "email")`.
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
    /// `bind("value", "query")`.
    ///
    /// Through the [`Input`] tag the same binding is a `{…}` prop: it lands on
    /// the root, and a component's root attributes are evaluated where the tag
    /// was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("input", "setQuery(target.value)")`.
    ///
    /// The `on` prefix is optional — `on("input", …)` and `on("oninput", …)`
    /// are the same attribute. Use native lowercase event names, not the JSX
    /// spellings.
    ///
    /// Resolved when the event fires, which is late enough that it reaches
    /// the calling page's state through either form.
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

    /// The class list this input renders with, the caller's own included.
    ///
    /// A join, not a merge: [`BASE`] contradicts itself nowhere, so this is
    /// exact for everything except a caller's `.class(…)` override, which is
    /// the one thing that can conflict. That is resolved in the browser — see
    /// [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[BASE, &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Bindings run from the client registry compiled by `html!` and retain their caller scope.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order: the component's own
    /// first, the caller's spread last so it wins.
    ///
    /// Handed to `html!` as `@{..attrs}` — see [`render`](Self::render) —
    /// which is where names are validated and values escaped.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", "input");

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the field is styled on arrival,
        // JavaScript or no JavaScript. See the cost named in [`Self::render`].
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[BASE, &self.class]),
            )
        };

        // The caller's own, last, so a `{...props}` spread wins — the same
        // last-wins rule `AttrList` applies when it writes the tag.
        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the field in that block's scope,
    /// which is what a handler or a binding written here needs.
    ///
    /// `<input>` is void, so this takes no children and renders no closing
    /// tag. The attribute set is spread onto it with `@{..attrs}`, which
    /// validates every name and escapes every value.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. [`Input`] adds one when it needs to
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

/// A text field.
///
/// ```ignore
/// <Input placeholder="Email" />
/// <Input type="password" name="password" required />
/// <Input value="ada@example.com" disabled />
/// <Input invalid placeholder="Email" />
/// <Input class="w-64" placeholder="Narrow" />
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every one
/// is optional. `r#type` is the exception it cannot make: a prop names a Rust
/// parameter, and `type` is a keyword. The rendered attribute is `type`, and
/// an unrecognized value trips a `debug_assert` so a typo is loud in
/// development and harmless in production.
///
/// Native events are written as native events and land on the `<input>`
/// untouched, and so are attribute bindings — a component's root attributes are
/// evaluated where the tag was written, so a controlled field reads and writes
/// the page's state:
///
/// ```ignore
/// // Reads and writes the page's `query`.
/// <Input value={query} oninput={setQuery(target.value)} />
/// ```
///
/// [`InputProps::render`] and [`input_variants`] remain for a field built in
/// Rust or an element this component does not render. See the module docs.
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with no
/// `{…}` in them, and names a Rust identifier cannot spell:
///
/// ```ignore
/// <Input attrs=@{Attrs::new().set("autocomplete", "email").set("maxlength", "64")} />
/// ```
///
/// The fragment root is what carries the component's identity — a
/// `#[component]` has to end in `html!`, and the element itself comes back
/// from [`InputProps::render`] already built. That identity is also a
/// PulsePoint scope, so a field bound to a page's state wants
/// [`InputProps::render`] instead; see the module docs.
#[component]
// A component's parameters are its props, and eleven is what a text field
// has. The lint is measuring a function; the call site is a tag, where each
// of these is named and optional.
#[allow(clippy::too_many_arguments)]
pub fn Input(
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
    let mut props = InputProps::default()
        .class(class)
        .kind(r#type)
        .disabled(disabled)
        .required(required)
        .invalid(invalid);

    // An unwritten prop is an empty string, and an empty `name` is not the
    // same request as no `name` at all — so each of these is written only when
    // the caller said something.
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

    // `oninput={…}`, `value={…}` and friends. `html!` hands every `{…}` prop
    // over as the expression the author wrote, and it goes onto the input
    // unchanged — the component decides nothing about it, exactly as a React
    // Input spreads `onChange` onto its `<input>` without reading it. Applied
    // last, so a controlled `value={…}` replaces an uncontrolled `value="…"`
    // rather than sitting beside it.
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
        let input = props.render();
        html! {
            <>
                @{input}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{input}</>`. A fragment root has no
        // element to carry this block's name, so it is delimited by
        // `<!--pp:id-->…<!--/pp-->` comments, and at hydration the runtime
        // turns that range into a real `<pp-fragment>` — which a CSS
        // combinator sees even though layout does not. Written as this block's
        // own root, the name lands on the `<input>` and the field stays a real
        // sibling of whatever surrounds it. See [`Label`](super::label).
        let attrs = props.attr_list();
        html! {
            <input @{..attrs} />
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Input, InputProps, input, input_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_input_carries_shadcns_defaults() {
        let markup = input().render().into_test_string();

        assert!(markup.starts_with("<input "), "{markup}");
        assert!(markup.contains(r#"data-slot="input""#));
        assert!(markup.contains("h-9"));
        assert!(markup.contains("border-input"));
        // No `type`, as in shadcn: an `<input>` with none is a text field.
        assert!(!markup.contains("type="), "{markup}");
        // Void: no closing tag.
        assert!(!markup.contains("</input>"), "{markup}");
    }

    #[test]
    fn a_type_is_written_as_the_type_attribute() {
        let markup = input().password().render().into_test_string();
        assert!(markup.contains(r#"type="password""#), "{markup}");

        let markup = input().kind("email").render().into_test_string();
        assert!(markup.contains(r#"type="email""#), "{markup}");
    }

    /// A typo is a release-mode fallback and a debug-mode panic. A browser
    /// reads an unknown type as `text` without saying so.
    #[test]
    #[should_panic(expected = "not an input type")]
    #[cfg(debug_assertions)]
    fn an_unknown_type_is_loud_in_development() {
        input().kind("emial");
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = input()
            .disabled(true)
            .required(true)
            .invalid(true)
            .render()
            .into_test_string();

        assert!(markup.contains(r#" disabled="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
        assert!(markup.contains(r#"aria-invalid="true""#), "{markup}");

        let markup = input()
            .disabled(false)
            .required(false)
            .invalid(false)
            .render()
            .into_test_string();

        // Attributes, not the `disabled:` and `aria-invalid:` utilities that
        // are in the class list either way — which is the whole point of
        // those two prefixes.
        assert!(!markup.contains(r#" disabled="""#), "{markup}");
        assert!(!markup.contains(r#" required="""#), "{markup}");
        assert!(!markup.contains(r#"aria-invalid=""#), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = input()
            .class("w-64 border-emerald-600")
            .render()
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("border-emerald-600"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn an_input_with_no_override_needs_no_runtime() {
        let props = input().email().placeholder("Email");
        let markup = props.render().into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="h-9"#), "{markup}");

        assert!(!input().class("w-64").needs_scope());
        assert!(
            input()
                .on(
                    "input",
                    crate::components::rahti_ui::attrs::ui_handler!({ setQuery(target.value) })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = input()
            .on(
                "input",
                crate::components::rahti_ui::attrs::ui_handler!({ setQuery(target.value) }),
            )
            .render()
            .into_test_string();

        assert!(
            markup.contains(r#"pp-oninput="setQuery(target.value)""#),
            "{markup}"
        );
    }

    /// A literal `value` is an uncontrolled default; a bound one is
    /// controlled. Writing both leaves one attribute, not two.
    #[test]
    fn a_bound_value_replaces_a_literal_one() {
        let markup = input()
            .value("ada@example.com")
            .bind(
                "value",
                crate::components::rahti_ui::attrs::ui_expr!(value = { email }),
            )
            .render()
            .into_test_string();

        assert!(markup.contains(r#"value="{email}""#), "{markup}");
        assert!(!markup.contains("ada@example.com"), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = input()
            .attr("data-slot", "mine")
            .attr("autocomplete", "email")
            .render()
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"autocomplete="email""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = input()
            .attrs(Attrs::new().unset("data-slot").set("maxlength", "64"))
            .render()
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains(r#"maxlength="64""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = input()
            .placeholder(r#"a" pp-onfocus="steal()"#)
            .render()
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn input_variants_is_the_components_own_class_list() {
        assert_eq!(
            input_variants("min-h-16 py-2"),
            input().class("min-h-16 py-2").class_list()
        );
    }

    /// The call form behind `<Input placeholder="Email" />`. A tag fills the
    /// props it names and takes these defaults for the rest.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Input(
            "",
            "email",
            "email",
            "",
            "Email",
            "",
            false,
            true,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(markup.contains(r#"type="email""#), "{markup}");
        assert!(markup.contains(r#"name="email""#), "{markup}");
        assert!(markup.contains(r#"placeholder="Email""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Input />` compiles to now
    /// that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_input() {
        let markup = Input(
            <&str>::default(),
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

        assert!(markup.contains(r#"data-slot="input""#), "{markup}");
        assert!(markup.contains("h-9"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(r#"placeholder="""#), "{markup}");
        assert!(!markup.contains(r#"value="""#), "{markup}");
        assert!(!markup.contains(" disabled="), "{markup}");
    }

    /// What `<Input oninput={setQuery(target.value)} />` compiles to. The
    /// expression arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_input() {
        let markup = Input(
            "",
            "",
            "",
            "",
            "Search",
            "",
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[
                ("oninput", "setQuery(target.value)"),
                ("value", "query"),
            ]),
            Attrs::new(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-oninput="setQuery(target.value)""#),
            "{markup}"
        );
        assert!(markup.contains(r#"value="{query}""#), "{markup}");
    }

    /// `InputProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_field() {
        assert_eq!(InputProps::new(), InputProps::default());
        assert_eq!(
            input().render().into_test_string(),
            InputProps::new().render().into_test_string()
        );
    }
}

/// The layer-independence check behind [`BASE`].
///
/// An input has one layer, so this is a smaller claim than
/// [`Button`](super::button)'s — but it is the same claim, and it is what lets
/// [`InputProps::class_list`] be a join rather than a merge.
#[cfg(test)]
mod audit {
    use super::BASE;
    use crate::components::rahti_ui::utils::audit::clashes;

    #[test]
    fn no_utility_contradicts_another() {
        let found = clashes(BASE);

        assert!(
            found.is_empty(),
            "{} clashes:\n{}",
            found.len(),
            found.join("\n")
        );
    }
}
