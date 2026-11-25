// rahti-ui: textarea
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Textarea — a port of shadcn/ui's `textarea`.
//!
//! The class string is shadcn's, copied rather than reinterpreted, so the
//! rendered field is pixel-identical to the React one under the same Tailwind
//! theme. What changes is how the props get there.
//!
//! As with [`Input`](super::input), the version ported here is the
//! utility-class one rather than its Base UI sibling, which moves the bulk of
//! the styling behind a `cn-textarea` class defined in a stylesheet — out of
//! reach of both the `.class(…)` override and the layer audit below.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Textarea placeholder="Tell us about it" />
//! <Textarea name="bio" rows="6" required />
//! <Textarea invalid placeholder="Too short" />
//! <Textarea>"The text a textarea opens with is its content."</Textarea>
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is this component's default.
//!
//! ## Its value is its children
//!
//! This is the one place a textarea reads differently from an input. HTML puts
//! a textarea's initial value *between its tags* rather than in a `value`
//! attribute, so that is where this component puts it too — `children`, placed
//! with `<slot />`:
//!
//! ```ignore
//! <Textarea name="bio">@{user.bio}</Textarea>
//! ```
//!
//! Children are text. A textarea's content is parsed as character data, so
//! markup written there is not markup — `<b>` arrives as the four characters
//! a user typed, which is what escaping is for and what a form round-trips.
//!
//! ## Bind `value` for a field the page drives
//!
//! A textarea whose value is the page's state is *controlled*: bind `value` to
//! that state and write it back from the handler, so the field and the state
//! cannot disagree.
//!
//! ```ignore
//! @{textarea()
//!     .bind("value", crate::components::rahti_ui::attrs::ui_expr!(value={bio}))
//!     .on("input", crate::components::rahti_ui::attrs::ui_handler!({setBio(target.value)}))
//!     .render(Html::empty())}
//! ```
//!
//! An uncontrolled textarea — one nothing is bound to — keeps whatever is typed
//! into it, and static children are an initial value the runtime leaves alone
//! once a user edits over them. Both were broken until recently: a textarea's
//! value lives in its children, every render produces those children, and the
//! runtime applied them unconditionally, so an empty template erased the field
//! on every render of the block. It was fixed in PulsePoint's
//! `AttributeSyncManager`, which now follows a textarea's rendered content only
//! once it has seen that content move — which is what marks the value as one
//! the template owns.
//!
//! Verified against the bundled runtime on `/components/textarea`: with
//! re-renders driven by an unrelated field, an uncontrolled `<textarea>` kept
//! every character and a seeded one kept an edit made over its default. The
//! floor is `public/js/pp-reactive-v2.min.js` carrying that fix — an older
//! bundle needs `value` bound on every textarea in a block that re-renders.
//!
//! ## A field wired to a page's state
//!
//! Through the tag, as for [`Input`](super::input): a component's root
//! attributes are evaluated where the tag was written, so `<Textarea
//! value={draft} oninput={setDraft(target.value)} />` reads and writes the
//! page's `draft`.
//!
//! Two other spellings do the same:
//!
//! - [`TextareaProps::render`], interpolated into the page's own block — the
//!   field is then in the page's scope, and `value`, `oninput` and
//!   `aria-invalid` all resolve;
//! - [`textarea_variants`] on a native `<textarea>`, for the same reason.
//!
//! Neither is needed for a binding; they are for a field assembled in Rust, or
//! an element this component does not render.
//!
//! ## When the tag is not enough
//!
//! **[`TextareaProps::render`], interpolated — for a field built in Rust.**
//!
//! ```ignore
//! @{textarea()
//!     .placeholder("Tell us about it")
//!     .on("input", crate::components::rahti_ui::attrs::ui_handler!({setDraft(target.value)}))
//!     .bind("aria-invalid", crate::components::rahti_ui::attrs::ui_expr!(aria-invalid={draft.length > 280}))
//!     .render(Html::empty())}
//! ```
//!
//! **[`textarea_variants`] — the class list alone.**
//!
//! ```ignore
//! <textarea class=@{textarea_variants("font-mono")}></textarea>
//! ```

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// Everything a textarea is.
///
/// One layer, like [`Input`](super::input)'s and unlike
/// [`Button`](super::button)'s three — there are no variants and no sizes, so
/// there is nothing here for a merge to resolve. `field-sizing-content` is
/// what grows the box with what is typed into it, and `min-h-16` is the floor
/// it starts from. `audit::no_utility_contradicts_another` holds the join
/// exact.
const BASE: &str = "flex field-sizing-content min-h-16 w-full rounded-md border \
                    border-input bg-transparent px-3 py-2 text-base shadow-xs \
                    transition-[color,box-shadow] outline-none \
                    placeholder:text-muted-foreground focus-visible:border-ring \
                    focus-visible:ring-[3px] focus-visible:ring-ring/50 \
                    disabled:cursor-not-allowed disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                    md:text-sm dark:bg-input/30 dark:aria-invalid:ring-destructive/40";

/// The class list shadcn's `cn(…, className)` returns: the field's own
/// utilities, then the caller's, merged so the caller's win.
///
/// Reach for this when the element has to belong to the page — a field whose
/// handler reads the page's state.
pub fn textarea_variants(class: &str) -> String {
    cn(&[BASE, class])
}

/// A textarea's props, with shadcn's defaults already in place.
///
/// Build one with [`textarea`] and narrow it down a method at a time. Nothing
/// here is required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose, and resolving that is the browser's job.
/// Everything else — `name`, `placeholder`, `rows` — is an ordinary attribute,
/// and lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextareaProps {
    class: String,
    attrs: Attrs,
}

/// A default textarea's props — the start of every chain.
pub fn textarea() -> TextareaProps {
    TextareaProps::default()
}

impl TextareaProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the field's own, `className`-style: a conflicting
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

    pub fn placeholder(self, placeholder: &str) -> Self {
        self.attr("placeholder", placeholder)
    }

    /// How many lines tall it starts.
    ///
    /// A number written as a string, the same way every other prop that takes
    /// a `&str` is — a tag's props are Rust expressions, and `rows="6"` is
    /// what reads well there.
    ///
    /// `field-sizing-content` in [`BASE`] grows the box past this as soon as
    /// there is enough text, so this sets where it opens rather than where it
    /// stays.
    pub fn rows(self, rows: &str) -> Self {
        self.attr("rows", rows)
    }

    pub fn cols(self, cols: &str) -> Self {
        self.attr("cols", cols)
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

    /// Any attribute, rendered on the server: `attr("maxlength", "280")`.
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
    /// `bind("value", "draft")`.
    ///
    /// Through the [`Textarea`] tag the same binding is a `{…}` prop: it lands
    /// on the root, and a component's root attributes are evaluated where the
    /// tag was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("input", "setDraft(target.value)")`.
    ///
    /// The `on` prefix is optional — `on("input", …)` and `on("oninput", …)`
    /// are the same attribute. Use native lowercase event names, not the JSX
    /// spellings.
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

    /// The class list this textarea renders with, the caller's own included.
    ///
    /// A join, not a merge: [`BASE`] contradicts itself nowhere, so this is
    /// exact for everything except a caller's `.class(…)` override.
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
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", "textarea");

        // Static class overrides compose data; they do not resolve conflicts.
        // Supply an explicit compiled `class={twMerge(...)}` binding when
        // overriding utilities from the component defaults.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[BASE, &self.class]),
            )
        };

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the field in that block's scope,
    /// which is what a handler or a binding written here needs.
    ///
    /// `children` is the textarea's initial value — HTML's own spelling, and
    /// the reason this takes an argument where [`Input`](super::input)'s
    /// `render` takes none. It is escaped on the way in, so text a user typed
    /// round-trips rather than becoming markup.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. [`Textarea`] adds one when it needs to
    /// ([`needs_scope`](Self::needs_scope)); markup interpolated into a page
    /// helper that has neither script nor state is never mounted, and an
    /// override there silently does nothing.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();
        html! {
            <textarea @{..attrs}>
                <slot />
            </textarea>
        }
    }
}

/// A multi-line text field.
///
/// ```ignore
/// <Textarea placeholder="Tell us about it" />
/// <Textarea name="bio" rows="6" required />
/// <Textarea invalid placeholder="Too short" />
/// <Textarea class="font-mono">"Opens with this text."</Textarea>
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every one
/// is optional. What sits between the tags is the field's initial value, which
/// is where HTML keeps a textarea's — see the module docs.
///
/// A field this page drives wants [`TextareaProps::render`] interpolated into
/// the page's own block, or [`textarea_variants`] on a native `<textarea>`:
/// PulsePoint owns `value` and its events, and takes that ownership in the
/// component's scope when it is reached through this tag.
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with no
/// `{…}` in them, and names a Rust identifier cannot spell:
///
/// ```ignore
/// <Textarea attrs=@{Attrs::new().set("maxlength", "280").set("wrap", "hard")} />
/// ```
#[component]
// A component's parameters are its props, and eleven is what a textarea has.
// The lint is measuring a function; the call site is a tag, where each of
// these is named and optional.
#[allow(clippy::too_many_arguments)]
pub fn Textarea(
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
    let mut props = TextareaProps::default()
        .class(class)
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
    if !rows.is_empty() {
        props = props.rows(rows);
    }

    props = props.attrs(attrs);

    // `oninput={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the textarea unchanged.
    // Applied last, so a binding replaces a server value of the same name
    // rather than sitting beside it.
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
        let field = props.render(children);
        html! {
            <>
                @{field}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{field}</>`. A fragment root has no
        // element to carry this block's name, so it is delimited by
        // `<!--pp:id-->…<!--/pp-->` comments, and at hydration the runtime
        // turns that range into a real `<pp-fragment>` — which a CSS
        // combinator sees even though layout does not. Written as this block's
        // own root, the name lands on the `<textarea>` and the field stays a
        // real sibling of whatever surrounds it. See [`Label`](super::label).
        let attrs = props.attr_list();
        html! {
            <textarea @{..attrs}>
                <slot />
            </textarea>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Textarea, TextareaProps, textarea, textarea_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_textarea_carries_shadcns_defaults() {
        let markup = textarea().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<textarea "), "{markup}");
        assert!(markup.ends_with("></textarea>"), "{markup}");
        assert!(markup.contains(r#"data-slot="textarea""#));
        assert!(markup.contains("min-h-16"));
        assert!(markup.contains("field-sizing-content"));
        assert!(markup.contains("border-input"));
    }

    /// A textarea is not void: its value lives between the tags, which is
    /// where HTML keeps it and where `<slot />` puts it.
    #[test]
    fn children_are_the_initial_value() {
        let markup = textarea()
            .render(Html::escape("Opens with this."))
            .into_test_string();

        assert!(markup.contains(">Opens with this.</textarea>"), "{markup}");
    }

    /// A textarea's content is character data, so text a user typed comes back
    /// as text rather than as markup.
    #[test]
    fn children_are_escaped_rather_than_parsed() {
        let markup = textarea()
            .render(Html::escape("<b>not bold</b> & so on"))
            .into_test_string();

        assert!(markup.contains("&lt;b&gt;not bold&lt;/b&gt;"), "{markup}");
        assert!(markup.contains("&amp; so on"), "{markup}");
        assert!(!markup.contains("<b>"), "{markup}");
    }

    #[test]
    fn boolean_props_are_present_or_absent_rather_than_true_or_false() {
        let markup = textarea()
            .disabled(true)
            .required(true)
            .invalid(true)
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#" disabled="""#), "{markup}");
        assert!(markup.contains(r#" required="""#), "{markup}");
        assert!(markup.contains(r#"aria-invalid="true""#), "{markup}");

        let markup = textarea()
            .disabled(false)
            .required(false)
            .invalid(false)
            .render(Html::empty())
            .into_test_string();

        // Attributes, not the `disabled:` and `aria-invalid:` utilities that
        // are in the class list either way.
        assert!(!markup.contains(r#" disabled="""#), "{markup}");
        assert!(!markup.contains(r#" required="""#), "{markup}");
        assert!(!markup.contains(r#"aria-invalid=""#), "{markup}");
    }

    #[test]
    fn rows_is_written_as_the_rows_attribute() {
        let markup = textarea()
            .rows("6")
            .render(Html::empty())
            .into_test_string();
        assert!(markup.contains(r#"rows="6""#), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = textarea()
            .class("min-h-40 font-mono")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("font-mono"), "{markup}");
    }

    #[test]
    fn a_textarea_with_no_override_needs_no_runtime() {
        let props = textarea().placeholder("Notes").rows("4");
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="flex"#), "{markup}");

        assert!(!textarea().class("font-mono").needs_scope());
        assert!(
            textarea()
                .on(
                    "input",
                    crate::components::rahti_ui::attrs::ui_handler!({ setDraft(target.value) })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = textarea()
            .on(
                "input",
                crate::components::rahti_ui::attrs::ui_handler!({ setDraft(target.value) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"pp-oninput="setDraft(target.value)""#),
            "{markup}"
        );
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = textarea()
            .attr("data-slot", "mine")
            .attr("maxlength", "280")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"maxlength="280""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = textarea()
            .attrs(Attrs::new().unset("data-slot").set("wrap", "hard"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains(r#"wrap="hard""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = textarea()
            .placeholder(r#"a" pp-onfocus="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn textarea_variants_is_the_components_own_class_list() {
        assert_eq!(
            textarea_variants("font-mono"),
            textarea().class("font-mono").class_list()
        );
    }

    /// The call form behind `<Textarea name="bio" rows="6" required />`.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Textarea(
            "",
            "bio",
            "",
            "Tell us about it",
            "6",
            false,
            true,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"name="bio""#), "{markup}");
        assert!(markup.contains(r#"rows="6""#), "{markup}");
        assert!(
            markup.contains(r#"placeholder="Tell us about it""#),
            "{markup}"
        );
        assert!(markup.contains(r#" required="""#), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Textarea></Textarea>`
    /// compiles to.
    #[test]
    fn all_props_omitted_is_a_default_textarea() {
        let markup = Textarea(
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
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="textarea""#), "{markup}");
        assert!(markup.contains("min-h-16"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"name="""#), "{markup}");
        assert!(!markup.contains(r#"rows="""#), "{markup}");
        assert!(!markup.contains(r#"placeholder="""#), "{markup}");
        assert!(!markup.contains(" disabled="), "{markup}");
    }

    #[test]
    fn native_event_props_land_on_the_textarea() {
        let markup = Textarea(
            "",
            "",
            "",
            "Notes",
            "",
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[("oninput", "setDraft(target.value)")]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-oninput="setDraft(target.value)""#),
            "{markup}"
        );
    }

    #[test]
    fn props_default_is_a_plain_field() {
        assert_eq!(TextareaProps::new(), TextareaProps::default());
    }
}

/// The layer-independence check behind [`BASE`].
///
/// One layer, so this is the same claim [`Input`](super::input) makes and a
/// smaller one than [`Button`](super::button)'s — but it is what lets
/// [`TextareaProps::class_list`] be a join rather than a merge.
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
