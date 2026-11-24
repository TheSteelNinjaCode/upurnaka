// rahti-ui: label
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Label — a port of shadcn/ui's `label`.
//!
//! The class string is shadcn's, copied rather than reinterpreted, so the
//! rendered label is pixel-identical to the React one under the same Tailwind
//! theme. What changes is how the props get there.
//!
//! As with [`Input`](super::input) and [`Textarea`](super::textarea), the
//! version ported here is the utility-class one. Its Base UI sibling wraps a
//! primitive whose only job is the association a native `<label for="…">`
//! already makes — React needs the primitive because `htmlFor` has to be
//! threaded through a component tree; server-rendered markup does not.
//! What Radix's primitive adds on top of the element is one thing:
//! `onMouseDown` suppressing the text selection a double-click would otherwise
//! make. `select-none` in [`BASE`] is that, without the JavaScript.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Label for="email">"Your email address"</Label>
//! <Label for="terms" class="text-muted-foreground">"Accept the terms"</Label>
//! <Label>@{icon} "With an icon beside it"</Label>
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is this component's default.
//!
//! `r#for` is the one prop that cannot be spelled the way JSX spells it, for
//! the same reason [`Input`](super::input)'s `r#type` cannot. A prop names a
//! Rust parameter, `for` is a Rust keyword, and `r#for` is what Rust calls a
//! keyword used as a name. The rendered attribute is `for` — HTML's spelling,
//! not JSX's `htmlFor`.
//!
//! ## What a label is for
//!
//! Two things, and `for` is only one of them. A `<label for="email">` names the
//! control with `id="email"`; a `<label>` with the control *inside it* names
//! that control without either needing an id. Both are HTML's, both work here,
//! and the second is why this component takes children rather than a `text`
//! prop:
//!
//! ```ignore
//! <Label>
//!     "Email"
//!     @{input().email().name("email").render()}
//! </Label>
//! ```
//!
//! The association is what makes the label clickable — a click on it focuses
//! the control — and it is also what a screen reader reads. Neither happens
//! from styling alone, so a label with neither `for` nor a control inside it is
//! decoration with a label's class list on it.
//!
//! ## The disabled variants belong to something else
//!
//! Two of the utilities in [`BASE`] never fire on their own:
//!
//! - `peer-disabled:*` styles this label when a **sibling** carrying `peer` is
//!   disabled. Tailwind compiles it to `.peer:disabled ~ &`, so the control has
//!   to have `class="peer"` and has to come *before* the label in the markup.
//! - `group-data-[disabled=true]:*` styles it when an **ancestor** carrying
//!   `group` has `data-disabled="true"`. That is the one shadcn's `Field` uses,
//!   and the one that works with the label written first.
//!
//! Neither is something this component can arrange for itself — both read
//! markup it does not own. The demo page shows each.
//!
//! The `peer-` half needs the label to be a *real* sibling, which is worth a
//! note because it was briefly not. A `#[component]` body that ends in
//! `<>@{element}</>` has no element to carry the block's name, so `html!`
//! delimits it with `<!--pp:id-->…<!--/pp-->` comments instead — and at
//! hydration the runtime replaces that comment range with a `<pp-fragment>`
//! element. Comments are not elements and a wrapper is, so `.peer:disabled ~ &`
//! matched the wrapper and the rule stopped firing the moment PulsePoint
//! mounted.
//!
//! [`Label`] is written so that does not happen: with no override and no
//! binding it renders as one literal `<label>` carrying its own `pp-component`,
//! and the tag is a real sibling like any other element. The script branch — a
//! class override with no binding — still uses a fragment. `group-data-` was
//! never affected either way — a descendant selector reaches through anything.
//!
//! ## What crosses the boundary
//!
//! Same rule as [`Button`](super::button). `onclick={…}` on the tag lands on
//! the `<label>` untouched and is resolved when it fires, late enough to reach
//! the calling page's state. So does an attribute *binding* — `hidden={locked}`
//! — because a component's root attributes are evaluated where the tag was
//! written: it reads the page's `locked`.
//!
//! ## When the tag is not enough
//!
//! **[`LabelProps::render`], interpolated — for a label built in Rust.**
//!
//! ```ignore
//! @{label()
//!     .html_for("email")
//!     .bind("data-invalid", crate::components::rahti_ui::attrs::ui_expr!(data-invalid={!valid}))
//!     .render(Html::escape("Email"))}
//! ```
//!
//! `render` returns the element and nothing around it, so interpolating it puts
//! the label in the calling block's scope, where bindings resolve.
//!
//! **[`label_variants`] — the class list alone.**
//!
//! ```ignore
//! <label class=@{label_variants("")} for="email">
//!     "Email"
//!     <span class="text-xs font-normal">{email.length}</span>
//! </label>
//! ```
//!
//! For a label whose *content* is the page's state: a `{…}` written between the
//! tags is compiled in the block it is written in, and children handed to a
//! component tag are not. shadcn has no `labelVariants` export of its own; this
//! is the same escape hatch [`button_variants`](super::button::button_variants)
//! is, under the name that matches it.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// Everything a label is.
///
/// One layer, like [`Input`](super::input)'s and unlike
/// [`Button`](super::button)'s three — there are no variants and no sizes, so
/// there is nothing here for a merge to resolve.
///
/// `flex items-center gap-2` is what lets an icon sit beside the text without
/// the call site arranging it. `select-none` is Radix's `onMouseDown` handler
/// as a utility: a double-click on a label focuses its control, and without
/// this it would also select the label's text. `leading-none` keeps a label the
/// height of its text, which is what lets it sit flush against a field.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const BASE: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none \
                    group-data-[disabled=true]:pointer-events-none \
                    group-data-[disabled=true]:opacity-50 \
                    peer-disabled:cursor-not-allowed peer-disabled:opacity-50";

/// The class list shadcn's `cn(…, className)` returns: the label's own
/// utilities, then the caller's, merged so the caller's win.
///
/// Reach for this when the element has to belong to the page — a label whose
/// text is the page's state, or one wrapping markup that reads it.
pub fn label_variants(class: &str) -> String {
    cn(&[BASE, class])
}

/// A label's props, with shadcn's defaults already in place.
///
/// Build one with [`label`] and narrow it down a method at a time. Nothing here
/// is required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose, and resolving that is the browser's job.
/// Everything else a label carries — `for`, `id`, `title` — is an ordinary
/// attribute, and lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LabelProps {
    class: String,
    attrs: Attrs,
}

/// A default label's props — the start of every chain.
pub fn label() -> LabelProps {
    LabelProps::default()
}

impl LabelProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the label's own, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// `for="…"` — the id of the control this label names.
    ///
    /// Named after JSX's `htmlFor` because `for` is a Rust keyword and
    /// `fn r#for` reads worse than the attribute it writes. The rendered
    /// attribute is `for`.
    ///
    /// Not the only way to associate a label: a control written *inside* the
    /// label is named by it without either needing an id. See the module docs.
    pub fn html_for(self, id: &str) -> Self {
        self.attr("for", id)
    }

    /// The label's own id — for a control that points back with
    /// `aria-labelledby`, which is the association to reach for when the label
    /// names something that is not a form control.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server: `attr("title", "Required")`.
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

    /// Take an attribute off, whatever set it: `unset("data-slot")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("data-invalid", "!valid")`.
    ///
    /// Through the [`Label`] tag the same binding is a `{…}` prop: it lands on
    /// the root, and a component's root attributes are evaluated where the tag
    /// was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("click", "focusEmail()")`.
    ///
    /// The `on` prefix is optional — `on("click", …)` and `on("onclick", …)`
    /// are the same attribute. Use native lowercase event names, not the JSX
    /// spellings.
    ///
    /// Resolved when the event fires, which is late enough that it reaches the
    /// calling page's state through either form.
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

    /// The class list this label renders with, the caller's own included.
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

    /// The open tag's attributes, in shadcn's order: the component's own first,
    /// the caller's spread last so it wins.
    ///
    /// Handed to `html!` as `@{..attrs}` — see [`render`](Self::render) — which
    /// is where names are validated and values escaped.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", "label");

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the label is styled on arrival,
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
    /// Interpolating this into a block puts the label in that block's scope,
    /// which is what a binding written here needs. A `{…}` written among the
    /// *children*, though, is compiled where those children were written — so a
    /// label whose text is the page's state wants [`label_variants`] on a
    /// native `<label>` instead.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. [`Label`] adds one when it needs to
    /// ([`needs_scope`](Self::needs_scope)); markup interpolated into a page
    /// helper that has neither script nor state is never mounted, and an
    /// override there silently does nothing.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();
        html! {
            <label @{..attrs}>
                <slot />
            </label>
        }
    }
}

/// A label for a form control.
///
/// ```ignore
/// <Label for="email">"Your email address"</Label>
/// <Label for="terms" class="text-muted-foreground">"Accept the terms"</Label>
/// <Label>@{icon} "With an icon beside it"</Label>
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every one
/// is optional. `r#for` is the exception it cannot make: a prop names a Rust
/// parameter, and `for` is a keyword. The rendered attribute is `for`.
///
/// What sits between the tags is the label's content — text, an icon, a badge,
/// or the control itself. `flex items-center gap-2` in the class list is what
/// arranges more than one of those.
///
/// Native events are written as native events and land on the `<label>`
/// untouched. So does an attribute binding: a component's root attributes are
/// evaluated where the tag was written, so `for={target}` on the tag reads
/// the page's `target`. See the module docs.
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with no
/// `{…}` in them, and names a Rust identifier cannot spell:
///
/// ```ignore
/// <Label attrs=@{Attrs::new().set("title", "Required").unset("data-slot")}>"Email"</Label>
/// ```
///
/// The fragment root is what carries the component's identity — a
/// `#[component]` has to end in `html!`, and the element itself comes back from
/// [`LabelProps::render`] already built.
#[component]
pub fn Label(
    class: &str,
    r#for: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = LabelProps::default().class(class);

    // An unwritten prop is an empty string, and an empty `for` is not the same
    // request as no `for` at all — so each of these is written only when the
    // caller said something.
    if !r#for.is_empty() {
        props = props.html_for(r#for);
    }
    if !id.is_empty() {
        props = props.id(id);
    }

    props = props.attrs(attrs);

    // `onclick={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the label unchanged — the
    // component decides nothing about it, exactly as a React Label spreads
    // `onClick` onto its element without reading it. Applied last, so a binding
    // replaces a server value of the same name rather than sitting beside it.
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
        let label = props.render(children);
        html! {
            <>
                @{label}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{label}</>`, and the difference is
        // structural rather than stylistic.
        //
        // Every `html!` block is named for the client runtime. A block with a
        // single element root carries that name as an attribute on the root; a
        // block whose root is a fragment has no element to put it on, so it is
        // delimited by `<!--pp:id-->…<!--/pp-->` comments — and at hydration
        // the runtime replaces that comment range with a real `<pp-fragment>`
        // element.
        //
        // Comments are not elements and a `<pp-fragment>` is, so a fragment
        // root changes the DOM's shape the moment PulsePoint mounts. That is
        // invisible to layout (`display: contents`) and to a descendant
        // selector, but not to a combinator: `peer-disabled:` on a Label is
        // `.peer:disabled ~ &`, and through a wrapper it matches nothing.
        //
        // Writing the element as this block's own root puts the name on the
        // `<label>` and leaves it a real sibling of whatever precedes it.
        // Measured on `/components/label`: with a fragment root, a Label beside
        // a disabled peer stayed at full opacity.
        let attrs = props.attr_list();
        html! {
            <label @{..attrs}>
                <slot />
            </label>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Label, LabelProps, label, label_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_label_carries_shadcns_defaults() {
        let markup = label().render(Html::escape("Email")).into_test_string();

        assert!(markup.starts_with("<label "), "{markup}");
        assert!(markup.ends_with(">Email</label>"), "{markup}");
        assert!(markup.contains(r#"data-slot="label""#));
        assert!(markup.contains("text-sm"));
        assert!(markup.contains("font-medium"));
        assert!(markup.contains("select-none"));
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"for="""#), "{markup}");
    }

    /// The utilities that only fire because of markup this component does not
    /// own — a `peer` sibling and a `group` ancestor. They are in the class
    /// list either way, which is the whole point of those prefixes.
    #[test]
    fn the_disabled_variants_are_always_carried() {
        let markup = label().render(Html::empty()).into_test_string();

        assert!(markup.contains("peer-disabled:opacity-50"), "{markup}");
        assert!(
            markup.contains("group-data-[disabled=true]:opacity-50"),
            "{markup}"
        );
        assert!(
            markup.contains("group-data-[disabled=true]:pointer-events-none"),
            "{markup}"
        );
    }

    /// JSX writes `htmlFor`; HTML writes `for`. The builder is named after the
    /// first and renders the second.
    #[test]
    fn html_for_is_written_as_the_for_attribute() {
        let markup = label()
            .html_for("email")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"for="email""#), "{markup}");
        assert!(!markup.contains("htmlFor"), "{markup}");
    }

    /// A label's content is markup, not text: an icon, a badge, or the control
    /// itself all go between the tags.
    #[test]
    fn children_pass_through_as_markup() {
        let markup = label()
            .render(Html::from_raw("<svg></svg>Email".into()))
            .into_test_string();

        assert!(markup.contains("<svg></svg>Email</label>"), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = label()
            .class("text-xs text-muted-foreground")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("text-muted-foreground"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_label_with_no_override_needs_no_runtime() {
        let props = label().html_for("email");
        let markup = props.render(Html::escape("Email")).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="flex"#), "{markup}");

        assert!(!label().class("text-xs").needs_scope());
        assert!(
            label()
                .bind(
                    "data-invalid",
                    crate::components::rahti_ui::attrs::ui_expr!(data - invalid = { !valid })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = label()
            .on(
                "click",
                crate::components::rahti_ui::attrs::ui_handler!({ focusEmail() }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"pp-onclick="focusEmail()""#), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = label()
            .attr("data-slot", "mine")
            .attr("title", "Required")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"title="Required""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = label()
            .attrs(Attrs::new().unset("data-slot").set("title", "Required"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains(r#"title="Required""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = label()
            .html_for(r#"a" pp-onclick="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onclick=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn label_variants_is_the_components_own_class_list() {
        assert_eq!(
            label_variants("text-xs"),
            label().class("text-xs").class_list()
        );
    }

    /// The call form behind `<Label for="email">"Email"</Label>`. A tag fills
    /// the props it names and takes these defaults for the rest.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Label(
            "",
            "email",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Email"),
        )
        .into_test_string();

        assert!(markup.contains(r#"for="email""#), "{markup}");
        assert!(markup.contains(">Email</label>"), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Label>"Email"</Label>`
    /// compiles to now that `html!` closes the props literal with
    /// `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_label() {
        let markup = Label(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::escape("Email"),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="label""#), "{markup}");
        assert!(markup.contains("font-medium"), "{markup}");
        assert!(markup.contains(">Email</label>"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"for="""#), "{markup}");
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// What `<Label onclick={focusEmail()}>` compiles to. The expression
    /// arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_label() {
        let markup = Label(
            "",
            "email",
            "",
            crate::rahti::Bindings::__from_html(&[("onclick", "focusEmail()")]),
            Attrs::new(),
            Html::escape("Email"),
        )
        .into_test_string();

        assert!(markup.contains(r#"pp-onclick="focusEmail()""#), "{markup}");
        assert!(markup.contains(r#"for="email""#), "{markup}");
    }

    /// `LabelProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_label() {
        assert_eq!(LabelProps::new(), LabelProps::default());
        assert_eq!(
            label().render(Html::empty()).into_test_string(),
            LabelProps::new().render(Html::empty()).into_test_string()
        );
    }
}

/// The layer-independence check behind [`BASE`].
///
/// One layer, so this is the same claim [`Input`](super::input) makes and a
/// smaller one than [`Button`](super::button)'s — but it is what lets
/// [`LabelProps::class_list`] be a join rather than a merge.
///
/// The two `opacity-50`s are the interesting case: they share a family and
/// would clash, except that `peer-disabled` and `group-data-[disabled=true]`
/// are different variant prefixes, so both apply in different states. The audit
/// reads them the way Tailwind does and lets them stand.
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
