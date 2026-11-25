// rahti-ui: spinner
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Spinner — a port of shadcn/ui's `spinner`.
//!
//! One icon that turns. shadcn's component is four lines: lucide's
//! `Loader2Icon`, `size-4 animate-spin`, and the two attributes that make it a
//! status rather than decoration.
//!
//! ```tsx
//! <Loader2Icon role="status" aria-label="Loading" className={cn("size-4 animate-spin", className)} />
//! ```
//!
//! ## The registry adds nothing this time — the page adds one attribute
//!
//! Fifth in a row, and the smallest gap of the five: `new-york-v4` renders no
//! `data-slot` at all, and every spinner on the docs page carries
//! `data-slot="spinner"`. The class list, the role and the label are identical.
//! Same rule as the last four — the page is what a reader compares against.
//!
//! ## It is an icon, so it is [`LoaderCircle`]
//!
//! This library has lucide's icons as components already, so the port is a thin
//! wrapper rather than a hand-written `<svg>`. Two things about
//! [`rahti_icons`](crate::components::rahti_icons) make that work cleanly, and
//! both are the icon crate's design rather than luck:
//!
//! 1. **Setting a `role` or an `aria-label` takes the decorative defaults off.**
//!    An icon is `aria-hidden="true" focusable="false"` unless something gives
//!    it an accessible name, so a spinner does not have to unset anything — it
//!    just says what it is.
//! 2. **A `class` is appended to the icon's own**, and a `{…}` inside it is
//!    substituted where it stands rather than replacing the attribute. That is
//!    what makes an override work here: see below.
//!
//! ## An override has to be merged, not joined
//!
//! Every one of shadcn's size demos passes a `size-*`, and `size-*` is exactly
//! what [`BASE`] already says — so unlike most components in this library, a
//! caller's class here almost always *conflicts*. A join would leave both sizes
//! in the list and let the stylesheet's order decide, which makes `size-8` win
//! and `size-3` lose. So this one merges.
//!
//! What that renders, measured rather than assumed:
//!
//! Rust composes class layers with `utils::cn`. Use frontend `twMerge`
//! bindings when utilities conflict.
//!
//! The merge carries only [`BASE`] and the caller's class. Lucide's two marker
//! classes are the icon's own and are prepended by it either way, so they do not
//! belong in the merge — putting them there writes them twice.
//!
//! ## The size is on the spinner, and that matters inside other components
//!
//! A [`Button`](super::button) sizes a bare icon with
//! `[&_svg:not([class*='size-'])]:size-4` — *unless the icon already has a size
//! class*. A spinner always does, so it keeps its own size wherever it is put,
//! which is what shadcn's button and badge demos rely on. A
//! [`Badge`](super::badge) is the exception and says so out loud: its
//! `[&>svg]:size-3!` is important, so a badge spinner is 12px whatever it asked
//! for.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_icons::loader_circle::LoaderCircle;
use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// The whole component. shadcn's string, unchanged.
pub const BASE: &str = "size-4 animate-spin";

/// The classes [`LoaderCircle`] is drawn with — its catalog entry's own. A
/// bound class replaces the icon's whole `class`, so it starts with these.
const ICON_CLASS: &str = "lucide lucide-loader-circle";

/// What a spinner tells a screen reader it is.
pub const ROLE: &str = "status";

/// What a spinner is called when nothing renames it — shadcn's `aria-label`.
pub const LABEL: &str = "Loading";

/// The class list shadcn's `cn("size-4 animate-spin", className)` returns.
pub fn spinner_variants(class: &str) -> String {
    cn(&[BASE, class])
}

/// A Spinner's props.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpinnerProps {
    label: String,
    class: String,
    attrs: Attrs,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            label: LABEL.to_string(),
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

/// A default [`Spinner`]'s props.
pub fn spinner() -> SpinnerProps {
    SpinnerProps::default()
}

impl SpinnerProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the spinner's own, `className`-style.
    ///
    /// Almost always a `size-*`, which is what [`BASE`] already says — so this
    /// is one of the few components here whose override normally has to be
    /// merged rather than joined.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// What a screen reader calls it.
    ///
    /// shadcn hardcodes `"Loading"`. A page that is waiting for something more
    /// specific can say so — "Uploading", "Saving" — and a page that has already
    /// said it in text beside the spinner can pass an empty label to make the
    /// icon decoration again, which is the icon crate's own rule.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// The spinner's own id.
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
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression.
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

    /// The class list this spinner renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        spinner_variants(&self.class)
    }

    /// Does anything here need a mounted PulsePoint scope?
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// Compose the icon's default classes and caller classes as data.
    /// Frontend `twMerge` bindings resolve conflicting utilities.
    fn icon_class(&self) -> String {
        crate::components::rahti_ui::utils::cn(&[BASE, &self.class])
    }

    /// The attributes laid on the icon, beyond its own.
    ///
    /// `role` and `aria-label` are what make it a status rather than decoration
    /// — and, because the icon crate drops `aria-hidden="true"` as soon as
    /// anything names the icon, they are also what stops it being hidden. An
    /// empty label leaves that alone, which is how a page makes a spinner
    /// decorative when the text beside it already says what is happening.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", "spinner");

        if !self.label.trim().is_empty() {
            base = base.set("role", ROLE).set("aria-label", self.label.trim());
        }

        merge(base, &self.attrs)
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// An icon that turns while something is happening.
///
/// ```ignore
/// <Spinner />
/// <Spinner class="size-8" />
/// <Spinner label="Uploading" />
/// ```
///
/// It announces itself: `role="status"` with `aria-label="Loading"`, which is
/// shadcn's. Put one beside text that already says what is happening and the
/// label is said twice — pass `label=""` there and it becomes decoration, which
/// is the icon crate's rule rather than an invention here.
///
/// The `size-4` it carries is its own, and it keeps it inside a
/// [`Button`](super::button) or an [`Item`](super::item), because those size a
/// bare icon only when it has no size class of its own. A
/// [`Badge`](super::badge) overrides it anyway — that one is `[&>svg]:size-3!`,
/// important on purpose.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Spinner(class: &str, label: &str, id: &str, bindings: Bindings, attrs: Attrs) -> Html {
    let mut props = spinner().class(class);
    // An empty `label` prop is "unwritten", so it keeps shadcn's default. A page
    // that wants no label at all writes `label=" "`… which trims to empty, so
    // the explicit way is `attrs.unset("aria-label")`.
    if !label.is_empty() {
        props = props.label(label);
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

    let icon_class = props.icon_class();
    let icon_attrs = props.attr_list();

    html! {
        <LoaderCircle
            class=@{icon_class.as_str()}
            attrs=@{(icon_attrs).data()}
            bindings=@{bindings}
        />
    }
}

#[cfg(test)]
mod tests {
    use super::{BASE, LABEL, ROLE, Spinner, spinner, spinner_variants};
    use crate::components::rahti_ui::attrs::Attrs;
    use crate::components::rahti_ui::utils::TestHtml as _;

    fn plain() -> String {
        Spinner("", "", "", crate::rahti::Bindings::default(), Attrs::new()).into_test_string()
    }

    fn sized(class: &str) -> String {
        Spinner(
            class,
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string()
    }

    /// The whole component: an icon, two utilities and the two attributes that
    /// make it a status.
    #[test]
    fn a_spinner_is_an_announced_icon() {
        let markup = plain();

        assert!(markup.starts_with("<svg "), "{markup}");
        assert!(markup.contains(r#"data-slot="spinner""#), "{markup}");
        assert!(markup.contains(r#"role="status""#), "{markup}");
        assert!(markup.contains(r#"aria-label="Loading""#), "{markup}");
        assert_eq!(BASE, "size-4 animate-spin");
        assert_eq!(ROLE, "status");
        assert_eq!(LABEL, "Loading");
    }

    /// Naming the icon is what takes its decorative defaults off — that is the
    /// icon crate's rule, and a spinner relies on it rather than unsetting
    /// anything.
    #[test]
    fn naming_it_stops_it_being_hidden() {
        let markup = plain();

        assert!(!markup.contains(r#"aria-hidden="true""#), "{markup}");
        assert!(!markup.contains(r#"focusable="false""#), "{markup}");
    }

    /// With no override the icon joins the class itself, so the server's list is
    /// final and lucide's markers survive.
    #[test]
    fn the_plain_spinner_is_finished_by_the_server() {
        let markup = plain();

        assert!(!markup.contains("twMerge("), "{markup}");
        assert!(
            markup.contains(r#"class="lucide lucide-loader-circle size-4 animate-spin""#),
            "{markup}"
        );
    }

    /// A `size-*` override contends with `BASE`'s own `size-4`, so it has to be
    /// merged rather than joined.
    ///
    /// The merge carries only `BASE` and the caller's class. Measured in the
    /// browser: the icon prepends its own classes and the runtime substitutes
    /// the `{…}` where it stands, so lucide's markers survive without the merge
    /// mentioning them — and mentioning them writes them twice.
    #[test]
    fn an_override_is_merged_without_restating_the_icons_markers() {
        let markup = sized("size-8");

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("size-8"), "{markup}");
    }

    /// A page whose text already says what is happening can make the spinner
    /// decoration again.
    #[test]
    fn an_unset_label_makes_it_decoration() {
        let markup = Spinner(
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new().unset("aria-label").unset("role"),
        )
        .into_test_string();

        assert!(!markup.contains("aria-label"), "{markup}");
        assert!(!markup.contains(r#"role="status""#), "{markup}");
    }

    /// A more specific label than shadcn's default.
    #[test]
    fn a_page_can_say_what_it_is_waiting_for() {
        let markup = Spinner(
            "",
            "Uploading",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(markup.contains(r#"aria-label="Uploading""#), "{markup}");
        assert!(!markup.contains(r#"aria-label="Loading""#), "{markup}");
    }

    /// The size is the spinner's own, which is what lets it survive being put
    /// inside a Button or an Item — both size a bare icon only when it has no
    /// size class.
    #[test]
    fn it_carries_its_own_size() {
        use crate::components::rahti_ui::button::{ButtonSize, ButtonVariant, button_variants};

        assert!(BASE.contains("size-4"), "{BASE}");
        // The selector the Button sizes a bare icon with — and the reason a
        // spinner, which always has a size class, is exempt from it.
        let button = button_variants(ButtonVariant::Default, ButtonSize::Default, "");
        assert!(
            button.contains("[&_svg:not([class*='size-'])]:size-4"),
            "{button}"
        );
        assert!(plain().contains("size-4"), "{}", plain());
    }

    #[test]
    fn the_class_list_is_the_variants_helper() {
        assert_eq!(spinner_variants(""), BASE);
        assert_eq!(spinner_variants("size-8"), "size-4 animate-spin size-8");
        assert_eq!(spinner().class_list(), BASE);
    }

    #[test]
    fn the_caller_can_still_pass_attributes() {
        let markup = Spinner(
            "",
            "",
            "upload-spinner",
            crate::rahti::Bindings::default(),
            Attrs::new().set("data-icon", "inline-start"),
        )
        .into_test_string();

        assert!(markup.contains(r#"id="upload-spinner""#), "{markup}");
        assert!(markup.contains(r#"data-icon="inline-start""#), "{markup}");
    }
}

#[cfg(test)]
mod audit {
    use super::BASE;
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// Two utilities cannot contradict each other, and this says so rather than
    /// assuming it.
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

    /// A caller's `size-*` is the one thing that *does* contradict `BASE`, which
    /// is why the frontend must resolve a size override.
    #[test]
    fn a_size_override_really_does_conflict() {
        let found = clashes(&cn(&[BASE, "size-8"]));

        assert!(
            !found.is_empty(),
            "the composed classes leave the size conflict for frontend twMerge"
        );
    }
}
