// rahti-ui: kbd
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.3.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Kbd — a port of shadcn/ui's `kbd`.
//!
//! A key, drawn the way a keyboard draws it. Two parts, one class string each,
//! no script — the smallest component here after [`Skeleton`](super::skeleton).
//!
//! ## The registry is a spelling behind
//!
//! Eighth in a row, and the difference is one variant written two ways:
//!
//! ```text
//! new-york-v4/kbd.json                    the docs page
//! ─────────────────────────────────────   ──────────────────────────────
//! [[data-slot=tooltip-content]_&]:…       in-data-[slot=tooltip-content]:…
//! ```
//!
//! They compile to the same selector — "when an ancestor is a tooltip's
//! content" — and `in-*` is Tailwind v4's own spelling for it. The page's is
//! used here, as it has been for the last seven.
//!
//! ## It knows when it is inside a tooltip
//!
//! That is the only interesting thing in the file. A key drawn on the page is
//! `bg-muted` with muted text; the same key inside a
//! [`Tooltip`](super::tooltip) — which is a dark surface — would disappear, so
//! three utilities invert it there. Nothing has to be passed: the key asks
//! whether it has a tooltip ancestor.
//!
//! ## The parts
//!
//! ```html
//! <kbd data-slot="kbd" class="pointer-events-none inline-flex h-5 …">⌘</kbd>
//! <kbd data-slot="kbd-group" class="inline-flex items-center gap-1">…</kbd>
//! ```
//!
//! Both are `<kbd>`, including the group — which is shadcn's, and is why
//! [`KbdGroup`] renders one rather than the `<div>` its React signature names.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// One key. shadcn's string, with the docs page's `in-*` spelling.
///
/// `pointer-events-none` because a key is a label, not a control: it should
/// never eat a click meant for the field or the button it sits in.
pub const BASE: &str = "pointer-events-none inline-flex h-5 w-fit min-w-5 items-center \
                        justify-center gap-1 rounded-sm bg-muted px-1 font-sans text-xs \
                        font-medium text-muted-foreground select-none \
                        in-data-[slot=tooltip-content]:bg-background/20 \
                        in-data-[slot=tooltip-content]:text-background \
                        dark:in-data-[slot=tooltip-content]:bg-background/10 \
                        [&_svg:not([class*='size-'])]:size-3";

/// Several keys pressed together. shadcn's, unchanged.
pub const GROUP: &str = "inline-flex items-center gap-1";

/// Which of the two a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Key,
    Group,
}

impl Part {
    const fn slot(self) -> &'static str {
        match self {
            Self::Key => "kbd",
            Self::Group => "kbd-group",
        }
    }

    const fn classes(self) -> &'static str {
        match self {
            Self::Key => BASE,
            Self::Group => GROUP,
        }
    }
}

/// The class list shadcn's `cn("pointer-events-none inline-flex …", className)`
/// returns.
pub fn kbd_variants(class: &str) -> String {
    cn(&[BASE, class])
}

/// [`KbdGroup`]'s class list, the caller's own included.
pub fn kbd_group_variants(class: &str) -> String {
    cn(&[GROUP, class])
}

/// A Kbd part's props.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KbdProps {
    part: Part,
    class: String,
    attrs: Attrs,
}

/// A default [`Kbd`]'s props.
pub fn kbd() -> KbdProps {
    KbdProps::default()
}

/// A default [`KbdGroup`]'s props.
pub fn kbd_group() -> KbdProps {
    KbdProps {
        part: Part::Group,
        ..KbdProps::default()
    }
}

impl KbdProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the part's own, `className`-style.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// The part's own id.
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

    /// Everything in `attrs`, laid over what is already set.
    pub fn attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = merge(std::mem::take(&mut self.attrs), &attrs);
        self
    }

    /// The class list this part renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[self.part.classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());

        let own = self.part.classes();
        base = if self.class.trim().is_empty() {
            base.set("class", own)
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[own, &self.class]),
            )
        };

        merge(base, &self.attrs)
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// One key.
///
/// ```ignore
/// <Kbd>"/"</Kbd>
/// <Kbd>"⌘"</Kbd>
/// ```
///
/// `pointer-events-none`, because a key is a label rather than a control — one
/// sitting in a search field must not eat the click that was meant to focus it.
///
/// It inverts itself inside a [`Tooltip`](super::tooltip) without being told:
/// `in-data-[slot=tooltip-content]:` asks whether it has a tooltip ancestor,
/// and a muted key on a dark surface would otherwise vanish.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Kbd(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let mut props = kbd().class(class);
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
    let attrs = props.attr_list();

    html! {
        <kbd @{..attrs}>
            <slot />
        </kbd>
    }
}

/// Several keys pressed together.
///
/// ```ignore
/// <KbdGroup><Kbd>"⌘"</Kbd><Kbd>"K"</Kbd></KbdGroup>
/// ```
///
/// A `<kbd>` holding `<kbd>`s, which is shadcn's and is also what HTML says a
/// chord is.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn KbdGroup(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let mut props = kbd_group().class(class);
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
    let attrs = props.attr_list();

    html! {
        <kbd @{..attrs}>
            <slot />
        </kbd>
    }
}

#[cfg(test)]
mod tests {
    use super::{BASE, GROUP, Kbd, KbdGroup, kbd, kbd_group_variants, kbd_variants};
    use crate::components::rahti_ui::attrs::Attrs;
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    fn key() -> String {
        Kbd(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("/"),
        )
        .into_test_string()
    }

    #[test]
    fn a_key_is_a_kbd_element() {
        let markup = key();

        assert!(markup.starts_with("<kbd "), "{markup}");
        assert!(markup.contains(r#"data-slot="kbd""#), "{markup}");
        assert!(markup.ends_with("/</kbd>"), "{markup}");
    }

    /// A group is a `<kbd>` too, which is shadcn's and is also what HTML says a
    /// chord is.
    #[test]
    fn a_group_is_also_a_kbd() {
        let markup = KbdGroup(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.starts_with("<kbd "), "{markup}");
        assert!(markup.contains(r#"data-slot="kbd-group""#), "{markup}");
        assert_eq!(GROUP, "inline-flex items-center gap-1");
    }

    /// A key is a label, not a control — it must not eat a click meant for the
    /// field it sits in.
    #[test]
    fn a_key_takes_no_pointer_events() {
        assert!(BASE.contains("pointer-events-none"), "{BASE}");
        assert!(BASE.contains("select-none"), "{BASE}");
    }

    /// It inverts itself inside a tooltip without being told, which is the only
    /// interesting thing in the file.
    #[test]
    fn a_key_knows_when_it_is_in_a_tooltip() {
        for utility in [
            "in-data-[slot=tooltip-content]:bg-background/20",
            "in-data-[slot=tooltip-content]:text-background",
            "dark:in-data-[slot=tooltip-content]:bg-background/10",
        ] {
            assert!(BASE.contains(utility), "{utility} is missing from {BASE}");
        }
        // The registry's older spelling of the same selector did not cross.
        assert!(!BASE.contains("[[data-slot=tooltip-content]_&]"), "{BASE}");
    }

    #[test]
    fn the_class_list_is_the_variants_helper() {
        assert_eq!(kbd_variants(""), BASE);
        assert_eq!(kbd_group_variants(""), GROUP);
        assert_eq!(kbd().class_list(), BASE);
        assert!(!key().contains("twMerge("), "{}", key());
        assert!(
            Kbd(
                "bg-accent",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::default()
            )
            .into_test_string()
            .contains("bg-accent"),
        );
    }
}

#[cfg(test)]
mod audit {
    use super::{BASE, GROUP};
    use crate::components::rahti_ui::utils::audit::clashes;

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered.
    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [("BASE", BASE), ("GROUP", GROUP)] {
            let found = clashes(list);

            assert!(
                found.is_empty(),
                "{name}: {} clashes:\n{}",
                found.len(),
                found.join("\n")
            );
        }
    }
}
