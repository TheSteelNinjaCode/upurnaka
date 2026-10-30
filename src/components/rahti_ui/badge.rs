// rahti-ui: badge
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Badge — a port of shadcn/ui's `badge`.
//!
//! A small pill with a word in it. Like [`Alert`](super::alert), it has no
//! script, no state and nothing to mount — everything it does is a class list
//! reading the children it was handed.
//!
//! ## The registry is stale here too
//!
//! Second time in a row, and for the same reason: `new-york-v4/badge.json` is a
//! version behind the page it is supposed to describe.
//!
//! ```text
//! new-york-v4/badge.json            the docs page
//! ───────────────────────────────   ────────────────────────────────────────
//! rounded-full                      rounded-4xl
//! (no height)                       h-5
//! transition-[color,box-shadow]     transition-all
//! [&>svg]:size-3                    [&>svg]:size-3!  (important)
//! [a&]:hover:…  at /90              [a]:hover:…      at /80
//! —                                 group/badge
//! —                                 has-data-[icon=inline-start]:pl-1.5
//! —                                 has-data-[icon=inline-end]:pr-1.5
//! six variants, `link` among them   five, and no `link` demo anywhere
//! ```
//!
//! So the strings are the docs page's, read off the rendered markup exactly as
//! [`Alert`](super::alert)'s were. Two consequences worth naming:
//!
//! - **No `link` variant.** The registry has one, but its string is written in
//!   the *old* spelling (`[a&]`, `/90`), and the page demos it nowhere — so
//!   there is nothing current to copy and guessing one would be inventing
//!   shadcn's design rather than porting it. [`BadgeVariant`] has five.
//! - **Physical padding, not logical.** The page ships the component twice: its
//!   LTR demos render `pr-1.5`/`pl-1.5` and its RTL section renders
//!   `pe-1.5`/`ps-1.5`. 44 occurrences to 6. The dominant spelling is the one
//!   here; an RTL page can swap them with a `class` override.
//!
//! ## `data-icon` is on the icon, and the badge reads it
//!
//! The interesting selector, and the same trick [`Alert`](super::alert) plays
//! with its action:
//!
//! ```text
//! has-data-[icon=inline-start]:pl-1.5     an icon on the left  -> trim the left
//! has-data-[icon=inline-end]:pr-1.5       an icon on the right -> trim the right
//! ```
//!
//! A round pill with `px-2` on both sides looks lopsided once an icon replaces
//! text on one side, because a glyph has no side bearing. shadcn's answer is not
//! a prop but an attribute on the **icon**, which the badge notices with
//! `:has()`. [`badge_icon`] is the sugar for writing it.
//!
//! ## A badge that is a link is an `<a>`
//!
//! shadcn's newest spelling is Base UI's `render={<a href="…">…</a>}`, which
//! replaced Radix's `asChild`. Neither crosses — this library's answer to both
//! has always been to render the element itself, so a [`href`](BadgeProps::href)
//! makes the badge an `<a data-slot="badge">`.
//!
//! That is also what the variants' `[a]:hover:` utilities are aimed at, and
//! whether they land is a question about Tailwind rather than about this port:
//! see [`BadgeVariant::classes`].
//!
//! ## The parts
//!
//! ```html
//! <span data-slot="badge" data-variant="secondary" class="group/badge inline-flex h-5 …">
//!   <svg data-icon="inline-start" …/>
//!   Verified
//! </span>
//! ```

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// The pill, before a variant. shadcn's string, less the two utilities a variant
/// has to be able to answer — see [`no_layer_contradicts_another`].
///
/// [`no_layer_contradicts_another`]: audit::no_layer_contradicts_another
///
/// `group/badge` is a named group nothing in this file reads yet; it is shadcn's
/// and it is what a page would hook its own `group-*` utilities onto.
/// `[&>svg]:size-3!` is important on purpose — an icon component that ships its
/// own size class would otherwise win.
pub const BASE: &str = "group/badge inline-flex h-5 w-fit shrink-0 items-center justify-center \
                        gap-1 overflow-hidden rounded-4xl border px-2 py-0.5 text-xs \
                        font-medium whitespace-nowrap transition-all \
                        focus-visible:border-ring focus-visible:ring-[3px] \
                        has-data-[icon=inline-end]:pr-1.5 \
                        has-data-[icon=inline-start]:pl-1.5 \
                        aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                        dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none \
                        [&>svg]:size-3!";

/// The border colour and focus ring four of the five variants share.
///
/// In shadcn's file these two sit in the base string and tailwind-merge takes
/// them back out for the variants that disagree — `outline` wants a real border
/// and `destructive` wants a red ring. There is no merge here, so a layer may
/// not answer a question another layer answers: they move down into the variants
/// that actually want them. Same rendered list, and
/// [`audit`](audit::no_layer_contradicts_another) is what holds it there.
pub const PLAIN: &str = "border-transparent focus-visible:ring-ring/50";

/// The filled pill. shadcn's, unchanged.
pub const DEFAULT: &str = "bg-primary text-primary-foreground [a]:hover:bg-primary/80";

/// The quieter fill. shadcn's, unchanged.
pub const SECONDARY: &str = "bg-secondary text-secondary-foreground [a]:hover:bg-secondary/80";

/// The one that has gone wrong. shadcn's, unchanged — and note it is a tinted
/// pill with red text rather than a solid red one, which is the same choice
/// [`Alert`](super::alert)'s destructive variant makes.
pub const DESTRUCTIVE: &str = "border-transparent bg-destructive/10 text-destructive \
                               focus-visible:ring-destructive/20 dark:bg-destructive/20 \
                               dark:focus-visible:ring-destructive/40 \
                               [a]:hover:bg-destructive/20";

/// The outlined pill — the one variant that wants a real border. shadcn's,
/// unchanged.
pub const OUTLINE: &str = "border-border focus-visible:ring-ring/50 text-foreground \
                           [a]:hover:bg-muted [a]:hover:text-muted-foreground";

/// No fill until you point at it. shadcn's, unchanged.
///
/// The only variant whose hover is unconditional rather than `[a]:`-gated — a
/// ghost badge reacts to the pointer whether or not it is a link, which is
/// shadcn's choice and reads like an oversight until you notice a ghost pill has
/// nothing else to show it is there.
pub const GHOST: &str = "hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50";

/// The badge's tone — shadcn's `variant`.
///
/// Five, not the registry's six: `link` is in the stale file only, written in a
/// spelling the current base string no longer uses, and demoed nowhere. See the
/// module docs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

impl BadgeVariant {
    /// The utilities this variant lays over [`BASE`].
    ///
    /// Four of the five carry [`PLAIN`] with them; `destructive` brings its own
    /// border and ring instead.
    ///
    /// ## About `[a]:hover:`
    ///
    /// shadcn's newest strings spell the link hover `[a]:hover:bg-primary/80`,
    /// where the older registry spelled it `[a&]:hover:`. The missing `&` looks
    /// like it should matter — an arbitrary variant with no `&` is usually read
    /// as a *descendant* — and it does not. Measured in this project's own
    /// compiled stylesheet:
    ///
    /// ```css
    /// .\[a\]\:hover\:bg-primary\/80:is(a):hover { background-color: var(--primary); }
    /// ```
    ///
    /// Tailwind v4 turns a bare element selector into `:is(a)` on the element
    /// itself, so the new spelling means exactly what the old one did: *this
    /// badge, when it is a link*. Confirmed in the browser too — an
    /// [`href`](BadgeProps::href) badge matches that selector's `:is(a)` half
    /// and a plain one does not.
    ///
    /// A page that wants a hover regardless of being a link can reach for
    /// `ghost`, whose hover is the one that is not gated.
    pub fn classes(self) -> String {
        match self {
            Self::Default => cn(&[PLAIN, DEFAULT]),
            Self::Secondary => cn(&[PLAIN, SECONDARY]),
            Self::Destructive => DESTRUCTIVE.to_string(),
            Self::Outline => OUTLINE.to_string(),
            Self::Ghost => cn(&[PLAIN, GHOST]),
        }
    }

    /// The name written into `data-variant`, and the one shadcn's `variant` prop
    /// takes.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Secondary => "secondary",
            Self::Destructive => "destructive",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `variant="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "secondary" => Self::Secondary,
            "destructive" => Self::Destructive,
            "outline" => Self::Outline,
            "ghost" => Self::Ghost,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Badge variant. \
                     One of: default, secondary, destructive, outline, ghost. \
                     (shadcn's stale registry also lists `link`; see the module docs.)"
                );
                Self::Default
            }
        }
    }
}

/// Which side of the label an icon sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSide {
    Start,
    End,
}

impl IconSide {
    /// The value written into `data-icon`, which [`BASE`] reads with `:has()`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Start => "inline-start",
            Self::End => "inline-end",
        }
    }
}

/// The attribute an icon inside a badge wants.
///
/// ```ignore
/// <Badge variant="secondary">
///     <BadgeCheck attrs=@{(badge_icon(IconSide::Start)).data()} />
///     "Verified"
/// </Badge>
/// ```
///
/// Sugar over `data-icon="inline-start"`, which is what shadcn writes by hand
/// and what [`BASE`]'s `has-data-[icon=inline-start]:pl-1.5` looks for. The
/// attribute goes on the **icon**, not on the badge: the badge finds it with
/// `:has()` and trims the padding on that side, because a glyph needs less room
/// beside it than a word does.
pub fn badge_icon(side: IconSide) -> Attrs {
    Attrs::new().set("data-icon", side.slug())
}

/// The class list shadcn's `cn(badgeVariants({ variant }), className)` returns.
pub fn badge_variants(variant: BadgeVariant, class: &str) -> String {
    cn(&[BASE, &variant.classes(), class])
}

/// A Badge's props, with shadcn's defaults already in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadgeProps {
    variant: BadgeVariant,
    href: String,
    class: String,
    attrs: Attrs,
}

impl Default for BadgeProps {
    fn default() -> Self {
        Self {
            variant: BadgeVariant::Default,
            href: String::new(),
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

/// A default [`Badge`]'s props.
pub fn badge() -> BadgeProps {
    BadgeProps::default()
}

impl BadgeProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the badge's own, `className`-style.
    ///
    /// This is how shadcn's colour demo is written — five differently coloured
    /// pills, and not one of them is a variant.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// The badge's tone.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Make this badge a link.
    ///
    /// It renders as `<a data-slot="badge">`, which is what shadcn's
    /// `render={<a href="…">}` produces and what the variants' `[a]:hover:`
    /// utilities are written for.
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = href.into();
        self
    }

    /// The badge's own id.
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
    ///
    /// Not `data-icon` on a child: [`BASE`] asks `has-data-[icon=…]` for it, and
    /// a bound value is literal text until the runtime mounts — so the padding
    /// would not trim until then.
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

    /// The badge's own utilities, before the caller's.
    fn own_classes(&self) -> String {
        cn(&[BASE, &self.variant.classes()])
    }

    /// The class list this badge renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[&self.own_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Never for its own sake — this component has no script and no state.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new()
            .set("data-slot", "badge")
            .set("data-variant", self.variant.slug());

        if !self.href.trim().is_empty() {
            base = base.set("href", self.href.trim());
        }

        let own = self.own_classes();
        base = if self.class.trim().is_empty() {
            base.set("class", own)
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&own, &self.class]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The rendered element — a `<span>`, or an `<a>` when there is an `href`.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();

        if self.href.trim().is_empty() {
            html! {
                <span @{..attrs}>@{children}</span>
            }
        } else {
            html! {
                <a @{..attrs}>@{children}</a>
            }
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// A small pill with a word in it.
///
/// ```ignore
/// <Badge>"Badge"</Badge>
/// <Badge variant="secondary">"Secondary"</Badge>
/// <Badge variant="outline">"Outline"</Badge>
/// <Badge href="/changelog">"What's new"</Badge>
///
/// <Badge variant="secondary">
///     <BadgeCheck attrs=@{(badge_icon(IconSide::Start)).data()} />
///     "Verified"
/// </Badge>
/// ```
///
/// A `<span>`, or an `<a>` when you give it an `href` — which is this library's
/// answer to both `asChild` and its replacement, `render`.
///
/// An icon goes in as a child with `data-icon` on it ([`badge_icon`] writes the
/// attribute), and the badge trims its padding on that side. There is no `icon`
/// prop and no side prop; `:has()` does it.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Badge(
    class: &str,
    variant: &str,
    href: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = badge()
        .class(class)
        .variant(BadgeVariant::parse(variant))
        .href(href);
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

    if href.trim().is_empty() {
        html! {
            <span @{..attrs}>
                <slot />
            </span>
        }
    } else {
        html! {
            <a @{..attrs}>
                <slot />
            </a>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BASE, Badge, BadgeProps, BadgeVariant, DEFAULT, DESTRUCTIVE, GHOST, IconSide, OUTLINE,
        PLAIN, SECONDARY, badge, badge_icon, badge_variants,
    };
    use crate::components::rahti_ui::attrs::Attrs;
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::components::rahti_ui::utils::cn;
    use crate::rahti::Html;

    fn pill(variant: &str) -> String {
        Badge(
            "",
            variant,
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Badge"),
        )
        .into_test_string()
    }

    #[test]
    fn a_badge_is_a_span_and_says_which_variant_it_is() {
        let markup = pill("secondary");

        assert!(markup.starts_with("<span "), "{markup}");
        assert!(markup.contains(r#"data-slot="badge""#), "{markup}");
        assert!(markup.contains(r#"data-variant="secondary""#), "{markup}");
    }

    /// With an `href` it is an `<a>`, which is what this library does instead of
    /// `asChild` and instead of its replacement, `render`.
    #[test]
    fn a_badge_with_an_href_is_an_anchor() {
        let link = Badge(
            "",
            "",
            "/changelog",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("What's new"),
        )
        .into_test_string();

        assert!(link.starts_with("<a "), "{link}");
        assert!(link.contains(r#"href="/changelog""#), "{link}");
        assert!(link.contains(r#"data-slot="badge""#), "{link}");
        // …and the variants' hover utilities are the ones written for it.
        assert!(DEFAULT.contains("[a]:hover:bg-primary/80"), "{DEFAULT}");
    }

    /// The padding trim is a question about a child, not a prop — and the
    /// attribute goes on the icon.
    #[test]
    fn an_icon_trims_the_padding_on_its_own_side() {
        let list = badge_variants(BadgeVariant::Default, "");

        assert!(
            list.contains("has-data-[icon=inline-start]:pl-1.5"),
            "{list}"
        );
        assert!(list.contains("has-data-[icon=inline-end]:pr-1.5"), "{list}");
        assert!(list.contains("px-2"), "{list}");

        // The sugar writes exactly what shadcn writes by hand.
        assert_eq!(
            badge_icon(IconSide::Start),
            Attrs::new().set("data-icon", "inline-start")
        );
        assert_eq!(
            badge_icon(IconSide::End),
            Attrs::new().set("data-icon", "inline-end")
        );
        assert_eq!(IconSide::Start.slug(), "inline-start");
    }

    /// The two utilities that moved out of shadcn's base string, and why.
    ///
    /// shadcn puts `border-transparent` and `focus-visible:ring-ring/50` in the
    /// base and lets tailwind-merge take them back out again for `outline` and
    /// `destructive`. There is no merge here, so they live in the four variants
    /// that actually want them — the rendered list is the same either way.
    #[test]
    fn the_border_and_ring_belong_to_the_variants() {
        assert!(!BASE.contains("border-transparent"), "{BASE}");
        assert!(!BASE.contains("focus-visible:ring-ring/50"), "{BASE}");
        assert!(BASE.contains(" border "), "{BASE}");

        for variant in [
            BadgeVariant::Default,
            BadgeVariant::Secondary,
            BadgeVariant::Ghost,
        ] {
            let list = variant.classes();
            assert!(list.contains("border-transparent"), "{list}");
            assert!(list.contains("focus-visible:ring-ring/50"), "{list}");
        }
        // `outline` wants a real border…
        assert!(OUTLINE.contains("border-border"), "{OUTLINE}");
        assert!(!OUTLINE.contains("border-transparent"), "{OUTLINE}");
        // …and `destructive` wants a red ring.
        assert!(
            DESTRUCTIVE.contains("focus-visible:ring-destructive/20"),
            "{DESTRUCTIVE}"
        );
        assert!(
            !DESTRUCTIVE.contains("focus-visible:ring-ring/50"),
            "{DESTRUCTIVE}"
        );
    }

    /// Five variants, and the sixth is deliberately absent — see the module
    /// docs.
    #[test]
    fn there_are_five_variants_and_no_link() {
        for name in ["default", "secondary", "destructive", "outline", "ghost"] {
            assert_eq!(BadgeVariant::parse(name).slug(), name);
        }
        assert_eq!(BadgeVariant::parse(""), BadgeVariant::Default);
        assert_eq!(BadgeProps::default().variant, BadgeVariant::Default);
    }

    /// The layout numbers the stale registry has not caught up with.
    #[test]
    fn the_shape_is_the_docs_pages() {
        assert!(BASE.contains("rounded-4xl"), "{BASE}");
        assert!(!BASE.contains("rounded-full"), "{BASE}");
        assert!(BASE.contains("h-5"), "{BASE}");
        assert!(BASE.contains("transition-all"), "{BASE}");
        assert!(!BASE.contains("transition-[color,box-shadow]"), "{BASE}");
        // Important on purpose: an icon that ships its own size class would
        // otherwise win.
        assert!(BASE.contains("[&>svg]:size-3!"), "{BASE}");
        assert!(BASE.contains("group/badge"), "{BASE}");
    }

    /// A destructive badge is a tinted pill with red text, not a solid red one —
    /// the same choice the Alert makes.
    #[test]
    fn destructive_is_a_tint_rather_than_a_fill() {
        assert!(DESTRUCTIVE.contains("bg-destructive/10"), "{DESTRUCTIVE}");
        assert!(DESTRUCTIVE.contains("text-destructive"), "{DESTRUCTIVE}");
        assert!(!DESTRUCTIVE.contains("text-white"), "{DESTRUCTIVE}");
    }

    /// The ghost badge is the one whose hover is not gated on being a link.
    #[test]
    fn only_ghost_hovers_without_being_a_link() {
        assert!(GHOST.contains("hover:bg-muted"), "{GHOST}");
        assert!(!GHOST.contains("[a]:hover:"), "{GHOST}");
        for list in [DEFAULT, SECONDARY, OUTLINE, DESTRUCTIVE] {
            assert!(list.contains("[a]:hover:"), "{list}");
        }
    }

    /// shadcn's colour demo is a `class` and nothing else.
    #[test]
    fn a_caller_can_recolour_a_badge_without_a_variant() {
        let blue = Badge(
            "bg-blue-50 text-blue-700",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Blue"),
        )
        .into_test_string();

        assert!(!blue.contains("{twMerge("), "{blue}");
        assert!(blue.contains("class=\""), "{blue}");
        assert!(blue.contains("text-blue-700"), "{blue}");
    }

    #[test]
    fn a_plain_badge_needs_no_scope() {
        assert!(!badge().needs_scope());
        assert!(!badge().class("rounded-md").needs_scope());
        assert!(!pill("").contains("<script"), "{}", pill(""));
    }

    #[test]
    fn the_class_list_is_the_variants_helper() {
        assert_eq!(
            badge_variants(BadgeVariant::Outline, "mt-1"),
            cn(&[BASE, OUTLINE, "mt-1"])
        );
        assert_eq!(
            badge_variants(BadgeVariant::Default, ""),
            cn(&[BASE, PLAIN, DEFAULT])
        );
        assert_eq!(
            BadgeProps::default().class_list(),
            badge_variants(BadgeVariant::Default, "")
        );
    }
}

#[cfg(test)]
mod audit {
    use super::{BASE, BadgeVariant};
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered.
    ///
    /// This is the test that decided where `border-transparent` and
    /// `focus-visible:ring-ring/50` live: shadcn keeps them in the base and lets
    /// tailwind-merge remove them for `outline` and `destructive`, which a join
    /// cannot do.
    #[test]
    fn no_layer_contradicts_another() {
        for variant in [
            BadgeVariant::Default,
            BadgeVariant::Secondary,
            BadgeVariant::Destructive,
            BadgeVariant::Outline,
            BadgeVariant::Ghost,
        ] {
            let list = cn(&[BASE, &variant.classes()]);
            let found = clashes(&list);

            assert!(
                found.is_empty(),
                "{}: {} clashes:\n{}",
                variant.slug(),
                found.len(),
                found.join("\n")
            );
        }
    }

    /// Nothing here reaches for a custom property a library would have had to
    /// publish — there is no library behind a badge.
    #[test]
    fn nothing_reaches_for_what_a_library_published() {
        for variant in [BadgeVariant::Default, BadgeVariant::Destructive] {
            let list = cn(&[BASE, &variant.classes()]);
            assert!(!list.contains("--radix-"), "{list}");
            assert!(!list.contains("cn-"), "{list}");
        }
    }
}
