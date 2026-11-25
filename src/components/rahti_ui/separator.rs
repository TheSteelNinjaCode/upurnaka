// rahti-ui: separator
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Separator — a port of shadcn/ui's `separator`.
//!
//! One element, one class string, and two attributes decided from two props.
//! It is the smallest component in this library, and the interesting thing
//! about it is how much of the React version turns out to be nothing at all
//! once the markup is written on the server.
//!
//! shadcn's registry Separator wraps `radix-ui`'s `Separator.Root`, which is a
//! `"use client"` component with a dependency behind it. Everything that
//! primitive does is this:
//!
//! ```text
//! data-orientation = orientation
//! decorative       ? role="none"
//!                  : role="separator" + aria-orientation (vertical only)
//! ```
//!
//! No state, no effect, no event. So the port is the attributes, and the
//! JavaScript that computed them does not cross.
//!
//! ## Which shadcn separator this is
//!
//! Both of shadcn's styles spell this one in utilities, which is unusual — the
//! Base UI style normally hides its design behind `cn-*` marker classes an
//! external stylesheet defines (see [`Card`](super::card)). Here the two differ
//! in one declaration:
//!
//! ```text
//! registry   …data-[orientation=vertical]:h-full      data-[orientation=vertical]:w-px
//! base ui    …data-vertical:self-stretch              data-vertical:w-px
//! ```
//!
//! **This port took the Base UI half, and that was wrong.** The argument for it
//! was that `h-full` is `height: 100%` and resolves against a parent that may
//! have no height, where `self-stretch` fills the cross axis of any flex parent.
//! That much is true. What it missed is what happens when the *caller* sets a
//! height, which is the commonest thing anyone does to a vertical separator:
//!
//! ```ignore
//! <Separator orientation="vertical" class="mr-2 h-4" />   // shadcn's own sidebar header
//! ```
//!
//! `align-self: stretch` on an item whose cross size is definite does not
//! stretch it — per the flexbox spec it behaves as `flex-start`. So the rule
//! pinned itself to the top of a 56px header instead of centring in it, which is
//! a visible stray line and how this was found.
//!
//! Rust composes class layers with `utils::cn`. Use frontend `twMerge`
//! bindings when utilities conflict.
//!
//! What `h-full` gives up is the case the Base UI spelling was chosen for: a
//! `flex items-center` parent with no height of its own and no height on the
//! separator. There the rule collapses. The module already said a vertical
//! separator wants a flex or grid parent *or* an explicit height; now it wants
//! one of those two rather than either.
//!
//! [`FieldSeparator`](super::field::FieldSeparator) is unaffected — it draws a
//! horizontal rule and positions it absolutely — and
//! [`ButtonGroupSeparator`](super::button_group::ButtonGroupSeparator) keeps a
//! copy of its own, where `self-stretch` is paired with `data-vertical:h-auto`
//! and so never meets a definite height.
//!
//! `data-horizontal:` and `data-vertical:` are shadcn's `@custom-variant`s, not
//! Tailwind's, and `src/app/globals.css` defines them. Without those
//! definitions every utility in [`BASE`] misses and the separator draws
//! nothing.
//!
//! ## `decorative` is spelled `announced`
//!
//! shadcn's prop is `decorative`, and it defaults to `true` — a rule between
//! two blocks is usually a picture of a division that the heading structure
//! already states, and announcing it again is noise. `role="none"` is how ARIA
//! spells that.
//!
//! The prop could not keep its name. A Rahti tag closes its props literal with
//! `..Default::default()`, so an unwritten `bool` prop is `false` — and a
//! `decorative` prop would therefore default to *not* decorative, silently
//! inverting shadcn's default on every `<Separator />` written. Renaming the
//! prop is the smaller divergence than flipping what it does:
//!
//! ```ignore
//! <Separator />                      // role="none"       — shadcn's default
//! <Separator announced=@{true} />    // role="separator"
//! ```
//!
//! [`SeparatorProps`] keeps both words: [`decorative`](SeparatorProps::decorative)
//! takes shadcn's boolean, [`announced`](SeparatorProps::announced) is the
//! shorthand for the other one.
//!
//! `aria-orientation` is written only on a vertical announced separator, which
//! is Radix's own rule: `horizontal` is the implicit value of the attribute for
//! `role="separator"`, so writing it says nothing.
//!
//! ## `element="hr"`, and the line that is not the one you think
//!
//! HTML already has this element. An `<hr>` is a separator to every screen
//! reader without a `role`, and to every reading-mode and print stylesheet
//! besides — so a horizontal rule between two runs of content is one of the few
//! places where the port can be more correct than the thing it is porting.
//!
//! It costs one utility, for a reason that hides itself. Tailwind's preflight
//! gives `hr` `border-top-width: 1px`, and this project's `globals.css` gives
//! every element `border-border` as its border colour. Boxes are `border-box`
//! here, so on an `<hr>` carrying [`BASE`]'s `h-px` that one-pixel border *is*
//! the whole element — and a border paints over the background, so the line an
//! `<hr>` draws is its border and `bg-border` is never seen.
//!
//! Which looks perfectly correct, because the two are the same colour. It stops
//! looking correct the moment anyone changes one: `.class("bg-primary")` on an
//! `<hr>` paints a background under an opaque grey border and the rule stays
//! grey. Give the rule any height above a pixel and you see both at once — a
//! grey hairline along the top of whatever colour was asked for.
//!
//! [`HR_RESET`] is the `border-0` that hands the line back to `bg-border`, so
//! an `<hr>` behaves like the `<div>` it is standing in for. It is added
//! automatically: [`SeparatorProps::class_list`] answers with it whenever the
//! element is an `hr`.
//!
//! An `<hr>` is a paragraph-level thematic break and cannot be vertical. That
//! combination trips a `debug_assert` rather than rendering something that lies
//! about itself.
//!
//! ## Which separator to reach for
//!
//! Three in this library, and they are not interchangeable:
//!
//! - **[`Separator`]** — a rule between two things on a page.
//! - **[`FieldSeparator`](super::field::FieldSeparator)** — a rule between two
//!   runs of a form, with an optional word sitting in a gap punched through it.
//! - **[`ButtonGroupSeparator`](super::button_group::ButtonGroupSeparator)** —
//!   a rule *inside* a welded strip of controls, in the buttons' border colour
//!   rather than the page's, and vertical unless said otherwise.
//!
//! The last two are their families' own parts and take their families'
//! defaults. This one is the plain case.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Separator />
//! <Separator class="my-4" />
//! <Separator orientation="vertical" />
//! <Separator element="hr" class="my-8" />
//! <Separator announced=@{true} orientation="vertical" />
//! ```
//!
//! Every prop is optional. `horizontal` unless said otherwise, and decorative
//! unless said otherwise, which is shadcn twice over.
//!
//! ## What crosses the boundary, and what does not
//!
//! Same rule as everywhere else in this library. `onclick={…}` on the tag lands
//! on the element untouched and resolves when it fires, late enough to reach
//! the calling page's state. So does an attribute *binding* —
//! `hidden={collapsed}` — because a component's root attributes are evaluated
//! where the tag was written.
//!
//! **[`SeparatorProps::render`], interpolated** — for a separator built in
//! Rust:
//!
//! ```ignore
//! @{separator().bind("hidden", crate::components::rahti_ui::attrs::ui_expr!(hidden={collapsed})).render()}
//! ```
//!
//! `hidden` is the one attribute worth binding here, and it is the one the
//! browser runtime handles specially on purpose: a binding whose name ends in
//! an HTML boolean-attribute word becomes a boolean attribute, present when
//! truthy and gone when falsy. That is exactly what `hidden` wants. It is also
//! why an invented `data-open` or `data-selected` must not be bound anywhere in
//! this library — same rewrite, silently wrong name.
//!
//! **[`separator_variants`] — the class list alone**, for an element the page
//! owns outright:
//!
//! ```ignore
//! <div class=@{separator_variants("my-4")} data-slot="separator"
//!      data-orientation="horizontal" role="none"></div>
//! ```
//!
//! Write `data-orientation` alongside it. Every utility in [`BASE`] is behind
//! `data-horizontal:` or `data-vertical:`, so without that attribute the class
//! list styles nothing at all.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// Everything a separator is.
///
/// One layer, like [`Label`](super::label)'s — there are no variants and no
/// sizes, and the orientation is not a layer either: both halves are in this
/// string at once and `data-orientation` decides which of them applies.
///
/// `shrink-0` is what keeps a separator a line rather than the first thing a
/// cramped flex row gives up. `bg-border` is the colour, and it is the whole
/// reason this constant is public — a
/// [`ButtonGroupSeparator`](super::button_group::ButtonGroupSeparator) takes
/// the same rule in the buttons' colour instead, and joins its layers rather
/// than merging them, so it carries its own copy without this one's background.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
pub const BASE: &str = "shrink-0 bg-border \
                        data-horizontal:h-px data-horizontal:w-full \
                        data-vertical:w-px data-vertical:self-stretch";

/// The one utility an `<hr>` needs that a `<div>` does not.
///
/// Tailwind's preflight gives `hr` `border-top-width: 1px`, and
/// `src/app/globals.css` gives every element `border-border` as its border
/// colour. [`BASE`] overrides the preflight's height and paints its own line
/// with `bg-border`, but nothing in it touches that border — and a border
/// paints over a background, so without this the line an `<hr>` draws is the
/// border's rather than the background's.
///
/// The two are the same colour by default, which is what makes it worth a
/// constant and a paragraph instead of a shrug: nothing looks wrong until a
/// caller changes the colour, and then `bg-primary` paints a background nobody
/// can see. Above a pixel of height both show at once, the border as a hairline
/// along the top.
///
/// Added by [`SeparatorProps::class_list`] whenever the element is an `hr`, so
/// nothing at a call site has to know about it. A page writing its own `<hr>`
/// with [`separator_variants`] does have to: join it in by hand.
pub const HR_RESET: &str = "border-0";

/// The utility in [`BASE`] that gives a vertical rule its length.
///
/// One, and it is `self-stretch` rather than the registry's `h-full`, which was
/// measured rather than argued:
///
/// | parent | `h-full` | `self-stretch` |
/// | --- | --- | --- |
/// | flex, definite height | ✓ | ✓ |
/// | flex, auto height | **0** | ✓ |
/// | block, definite height | ✓ | does nothing |
///
/// Carrying both is worse than either. `height: 100%` against an auto-height
/// flex parent resolves to *zero*, and a zero height is a **definite** cross
/// size — which is exactly the condition that stops `align-self: stretch`
/// applying. The one utility poisons the other in the one case the other exists
/// for.
///
/// It is dropped the moment a caller sets a height — see [`sets_own_height`],
/// and the module docs for what that cost to find.
const VERTICAL_HEIGHT: [&str; 1] = ["data-vertical:self-stretch"];

/// Does this caller's class list set a height of its own?
///
/// It matters because [`VERTICAL_HEIGHT`] and a caller's `h-4` are the same
/// question asked twice, and nothing downstream resolves it. `align-self:
/// stretch` on an item whose cross size is definite does not stretch it — per
/// the flexbox spec it behaves as `flex-start` — so the rule pins itself to the
/// top of its parent instead of centring in it. And `twMerge` will not step in:
/// it only merges utilities carrying the *same* variant, and a `data-vertical:`
/// one and a bare `h-4` carry different ones.
///
/// So the losing utility is dropped before the join rather than beaten after it,
/// which is the third time this library has had to do that — see
/// [`Resizable`](super::resizable) and [`Sidebar`](super::sidebar) — and it is
/// what makes shadcn's own call site work:
///
/// ```ignore
/// <Separator orientation="vertical" class="mr-2 h-4" />
/// ```
///
/// A bare height counts, and so does one scoped to a vertical separator —
/// `data-[orientation=vertical]:h-4` is shadcn's current spelling of the same
/// line. A height behind any other variant does not: a caller writing `md:h-6`
/// is asking a different question, and the rule stays to answer the base one.
/// See [`is_own_height`].
fn sets_own_height(class: &str) -> bool {
    class.split_whitespace().any(is_own_height)
}

/// Is this one utility a height asked at the same scope the rule answers?
///
/// Two spellings count. A bare `h-4` is shadcn's sidebar header. An
/// orientation-scoped `data-[orientation=vertical]:h-4` is the same question
/// written more precisely, and it is what shadcn's current `sidebar-07` block
/// ships; the registry's own `h-full` carries that prefix too, so a caller
/// matching it is answering exactly the case [`VERTICAL_HEIGHT`] exists for.
/// Both have to count, because a prefixed one that did not would leave
/// `self-stretch` in place and pin the rule to the top of its parent — the
/// failure the module docs open with, arrived at by a different route.
///
/// Everything else does not. `md:h-6` is a height at a breakpoint and says
/// nothing about the base case, so the rule stays to answer it.
fn is_own_height(utility: &str) -> bool {
    let height = match utility.rsplit_once(':') {
        None => utility,
        Some((prefix, height)) => match VERTICAL_VARIANTS.contains(&prefix) {
            true => height,
            false => return false,
        },
    };

    height.starts_with("h-")
}

/// The variants that scope a utility to a vertical separator and nothing else.
///
/// Both spellings, because `BASE` writes the short one and the registry writes
/// the long one, and a caller copying either is asking the same thing.
const VERTICAL_VARIANTS: [&str; 2] = ["data-vertical", "data-[orientation=vertical]"];

/// The tags a [`Separator`] will answer to.
///
/// `div` is shadcn's, through Radix's `Primitive.div`. `hr` is HTML's own
/// separator and needs no `role` to be one — see the module docs, and the
/// utility it costs.
const ELEMENTS: [&str; 2] = ["div", "hr"];

/// A rule across, or a rule down.
///
/// The default is [`Horizontal`](Self::Horizontal), which is shadcn's. Note
/// that a [`ButtonGroupSeparator`](super::button_group::ButtonGroupSeparator)'s
/// default is the other one, because that separator divides a strip that is a
/// row unless said otherwise.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl SeparatorOrientation {
    /// The name shadcn writes into `data-orientation`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `orientation="…"` prop.
    ///
    /// An empty string is the unwritten prop and means `fallback`. Anything
    /// else unrecognized is a typo: it falls back too, so a page is never
    /// broken by one, and trips a `debug_assert` so it does not go unnoticed in
    /// development.
    pub fn parse_or(name: &str, fallback: Self) -> Self {
        match name {
            "" => fallback,
            "horizontal" => Self::Horizontal,
            "vertical" => Self::Vertical,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Separator orientation. \
                     One of: horizontal, vertical."
                );
                fallback
            }
        }
    }

    /// [`parse_or`](Self::parse_or) with shadcn's default.
    pub fn parse(name: &str) -> Self {
        Self::parse_or(name, Self::Horizontal)
    }
}

/// The class list shadcn's `cn("shrink-0 bg-border …", className)` returns.
///
/// Reach for this when the element has to belong to the page — a separator
/// whose class list a page computes, or one that has to sit in the page's own
/// scope. Write `data-orientation` alongside it: every utility in [`BASE`] is
/// behind an orientation variant, and without the attribute none of them apply.
///
/// This is the `<div>` list. An `<hr>` needs [`HR_RESET`] joined in as well —
/// [`SeparatorProps::class_list`] does that for you, and this does not, because
/// it does not know what element it is going on.
pub fn separator_variants(class: &str) -> String {
    let base = match sets_own_height(class) {
        true => BASE
            .split_whitespace()
            .filter(|utility| !VERTICAL_HEIGHT.contains(utility))
            .collect::<Vec<_>>()
            .join(" "),
        false => BASE.to_string(),
    };

    cn(&[&base, class])
}

/// A separator's props, with shadcn's defaults already in place.
///
/// Build one with [`separator`] and narrow it down a method at a time. Nothing
/// here is required.
///
/// Only `class` and `element` are fields of their own: `class` is the one thing
/// that can conflict with what the component chose, and resolving that is the
/// browser's job; `element` decides which tag is written and so cannot be an
/// attribute. Everything else a separator carries — `id`, `title` — is an
/// ordinary attribute and lives in `attrs` with the caller's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeparatorProps {
    orientation: SeparatorOrientation,
    decorative: bool,
    class: String,
    element: &'static str,
    attrs: Attrs,
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            // shadcn's `decorative = true`, and the reason the tag's prop is
            // spelled `announced` instead. See the module docs.
            decorative: true,
            class: String::new(),
            element: "div",
            attrs: Attrs::new(),
        }
    }
}

/// A default separator's props — the start of every chain.
pub fn separator() -> SeparatorProps {
    SeparatorProps::default()
}

impl SeparatorProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the separator's own, `className`-style: a conflicting
    /// utility written here wins.
    ///
    /// Almost always a margin — `my-4` on a horizontal rule, `mx-2` on a
    /// vertical one. The line itself is one pixel and there is nothing in it to
    /// restyle.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Which way the rule runs.
    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SeparatorOrientation::Horizontal)
    }

    /// A rule down rather than across.
    ///
    /// Needs a parent with a height, or a height of its own.
    /// `data-vertical:h-full` is `height: 100%`, which resolves against the
    /// parent and collapses to nothing when the parent has none — so a vertical
    /// separator in a plain auto-height block is invisible, and that is a layout
    /// question rather than a bug in the class list.
    ///
    /// The usual answer is the caller's: `class="h-4"` on the separator, which
    /// beats `h-full` in the browser's merge and needs nothing from the parent.
    /// The module docs explain why that spelling is the one this component
    /// carries.
    pub fn vertical(self) -> Self {
        self.orientation(SeparatorOrientation::Vertical)
    }

    /// shadcn's `decorative`, with shadcn's meaning: `true` renders
    /// `role="none"` and the separator is invisible to a screen reader.
    ///
    /// Defaults to `true`, because a rule is usually a picture of a division
    /// the heading structure already states.
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    /// The other one: `role="separator"`, and a screen reader announces it.
    ///
    /// Reach for it when the rule is the *only* thing saying two runs of
    /// content are separate — a list of items with no headings between them, or
    /// a toolbar's groups.
    pub fn announced(self) -> Self {
        self.decorative(false)
    }

    /// Render as something other than `<div>` — shadcn's `asChild`, without the
    /// child. A tag outside [`ELEMENTS`] is ignored.
    pub fn element(mut self, tag: &'static str) -> Self {
        debug_assert!(
            ELEMENTS.contains(&tag),
            "rahti-ui: `{tag}` is not a tag a Separator renders as. One of: {}.",
            ELEMENTS.join(", "),
        );
        if ELEMENTS.contains(&tag) {
            self.element = tag;
        }
        self
    }

    /// Render as HTML's own separator — `<hr>`, a thematic break.
    ///
    /// Horizontal only: an `<hr>` is a paragraph-level break and a vertical one
    /// would be an element saying something the page does not mean. The pair
    /// trips a `debug_assert` in [`class_list`](Self::class_list).
    pub fn rule(self) -> Self {
        self.element("hr")
    }

    /// The separator's own id — for an `aria-labelledby` or a page's own
    /// selector.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server: `attr("title", "End of section")`.
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
    /// `bind("hidden", "collapsed")`.
    ///
    /// Through the [`Separator`] tag the same binding is a `{…}` prop: it lands
    /// on the root, and a component's root attributes are evaluated where the
    /// tag was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    ///
    /// `hidden` is the one worth binding on a separator, and it works the way
    /// you want because the runtime rewrites any binding whose attribute name
    /// ends in an HTML boolean-attribute word into a boolean attribute. That is
    /// the intended behaviour here and a trap elsewhere: never bind an invented
    /// `data-open` or `data-selected`, which are caught by the same rule and
    /// have their names truncated when the expression is falsy.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("click", "collapse()")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings. Rarely what you want on a one-pixel line — a separator is
    /// not a control, and a click target this size is not one either.
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
        self.attrs = merge(std::mem::take(&mut self.attrs), &attrs);
        self
    }

    /// This separator's own utilities, before the caller's — [`BASE`], plus
    /// [`HR_RESET`] when the element is an `<hr>`.
    fn own_classes(&self) -> String {
        debug_assert!(
            !(self.element == "hr" && self.orientation == SeparatorOrientation::Vertical),
            "rahti-ui: an <hr> is a thematic break and cannot be vertical. \
             Use the default <div> element for a vertical Separator."
        );

        // A caller's own height answers the same question as
        // `data-vertical:h-full`, and nothing downstream will resolve the two —
        // see `sets_own_height`.
        let base = match sets_own_height(&self.class) {
            true => BASE
                .split_whitespace()
                .filter(|utility| !VERTICAL_HEIGHT.contains(utility))
                .collect::<Vec<_>>()
                .join(" "),
            false => BASE.to_string(),
        };

        if self.element == "hr" {
            cn(&[&base, HR_RESET])
        } else {
            base
        }
    }

    /// The class list this separator renders with, the caller's own included.
    ///
    /// A join, not a merge: [`BASE`] contradicts itself nowhere and [`HR_RESET`]
    /// contradicts nothing in it, so this is exact for everything except a
    /// caller's `.class(…)` override — which is the one thing that can conflict,
    /// and is resolved in the browser. See [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[&self.own_classes(), &self.class])
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
            .set("data-slot", "separator")
            .set("data-orientation", self.orientation.slug());

        // Radix's `Separator.Root`, exactly. `role="none"` takes a decorative
        // rule out of the accessibility tree; the other branch puts it in, and
        // names the orientation only when it is not the one `role="separator"`
        // already implies.
        base = if self.decorative {
            base.set("role", "none")
        } else {
            let base = base.set("role", "separator");
            match self.orientation {
                SeparatorOrientation::Vertical => base.set("aria-orientation", "vertical"),
                SeparatorOrientation::Horizontal => base,
            }
        };

        // Static class overrides compose data; they do not resolve conflicts.
        // Supply an explicit compiled `class={twMerge(...)}` binding when
        // overriding utilities from the component defaults.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&self.own_classes(), &self.class]),
            )
        };

        // The caller's own, last, so a `{...props}` spread wins — the same
        // last-wins rule `AttrList` applies when it writes the tag.
        merge(base, &self.attrs)
    }

    /// The rendered element, with no boundary around it and no `<script>` in
    /// it.
    ///
    /// The absence of the script is the point, and it is what makes this the
    /// method to reach for when a page drives the separator. A block with a
    /// script is *mounted as its own scope*, so a separator rendered with one
    /// would compile a page's `bind("hidden", "collapsed")` against a scope that
    /// has no `collapsed`. The binding does not error at build time and the
    /// element simply never updates.
    ///
    /// Takes no children. A separator has none in either style — the word
    /// sitting in a gap through the rule belongs to
    /// [`FieldSeparator`](super::field::FieldSeparator), which is a different
    /// component for that reason.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    pub fn render(&self) -> Html {
        let attrs = self.attr_list();

        match self.element {
            "hr" => html! {
                <hr @{..attrs} />
            },
            _ => html! {
                <div @{..attrs}></div>
            },
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// A rule between two things.
///
/// ```ignore
/// <Separator />
/// <Separator class="my-4" />
/// <Separator element="hr" class="my-8" />
///
/// <div class="flex h-5 items-center gap-4 text-sm">
///     "Blog"
///     <Separator orientation="vertical" />
///     "Docs"
///     <Separator orientation="vertical" />
///     "Source"
/// </div>
/// ```
///
/// `horizontal` unless said otherwise, and decorative unless said otherwise —
/// both are shadcn's defaults. A vertical separator needs a flex or grid parent
/// to stretch across; see [`SeparatorProps::vertical`].
///
/// `announced=@{true}` is shadcn's `decorative={false}`, renamed because an
/// unwritten Rahti `bool` prop is `false` and keeping the name would have
/// flipped the default. The module docs say more.
///
/// Native events are written as native events and land on the element
/// untouched. So does an attribute binding: a component's root attributes are
/// evaluated where the tag was written, so `hidden={collapsed}` on the tag
/// reads the page's `collapsed`.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Separator(
    class: &str,
    orientation: &str,
    element: &str,
    id: &str,
    announced: bool,
    bindings: Bindings,
    attrs: Attrs,
) -> Html {
    let mut props = separator()
        .class(class)
        .orientation(SeparatorOrientation::parse(orientation))
        .decorative(!announced);

    // `element` arrives borrowed from the call site and [`ELEMENTS`] is
    // `&'static`, so the tag is matched back to its own entry rather than
    // leaked. An unrecognized one — including the empty string an unwritten
    // prop is — leaves the default in place.
    if let Some(tag) = ELEMENTS.iter().find(|known| **known == element) {
        props = props.element(tag);
    }
    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all.
    if !id.is_empty() {
        props = props.id(id);
    }

    props = props.attrs(attrs);

    // `onclick={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the element unchanged — the
    // component decides nothing about it. Applied last, so a binding replaces a
    // server value of the same name rather than sitting beside it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // A `class` override needs a mounted block for its `{twMerge(…)}`. A
    // `<div>` can hold the script itself and stay one literal element, which
    // keeps it a real sibling of whatever it divides, and its root attributes
    // — a caller's `{…}` props among them — are still evaluated where the tag
    // was written. An `<hr>` is void and cannot, so a class override with
    // nothing of the caller's pays for a fragment — invisible to layout (the
    // boundary is `display: contents`, so the rule is still a flex item of its
    // parent) and visible to a `+` or `:last-child` selector a page might have
    // written around it. An `<hr>` with a caller's binding renders script-less
    // instead, so it is compiled in the caller's scope, where that state lives.
    match (props.element, scope) {
        ("hr", true) if bindings.is_empty() => html! {
            <>
                <hr @{..attrs} />
                <script>r##""##</script>
            </>
        },
        ("hr", _) => html! {
            <hr @{..attrs} />
        },
        (_, false) => html! {
            <div @{..attrs}></div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <script>r##""##</script>
            </div>
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, Separator, SeparatorOrientation, SeparatorProps, separator, separator_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;

    #[test]
    fn a_default_separator_carries_shadcns_defaults() {
        let markup = separator().render().into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.ends_with("></div>"), "{markup}");
        assert!(markup.contains(r#"data-slot="separator""#), "{markup}");
        assert!(
            markup.contains(r#"data-orientation="horizontal""#),
            "{markup}"
        );
        // `decorative = true` is shadcn's default, and `role="none"` is it.
        assert!(markup.contains(r#"role="none""#), "{markup}");
        assert!(!markup.contains("aria-orientation"), "{markup}");
        assert!(markup.contains("bg-border"), "{markup}");
        assert!(markup.contains("shrink-0"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// Both halves of the class list are always carried; `data-orientation`
    /// decides which of them applies. Nothing about the string changes with the
    /// orientation.
    #[test]
    fn the_orientation_is_an_attribute_and_not_a_layer() {
        let horizontal = separator();
        let vertical = separator().vertical();

        assert_eq!(horizontal.class_list(), vertical.class_list());

        let markup = vertical.render().into_test_string();
        assert!(
            markup.contains(r#"data-orientation="vertical""#),
            "{markup}"
        );
        assert!(markup.contains("data-horizontal:h-px"), "{markup}");
        assert!(markup.contains("data-vertical:w-px"), "{markup}");
        // The registry's spelling rather than the Base UI style's
        // `self-stretch`, which pinned a caller-sized rule to the top of its
        // flex parent. The module docs argue it.
        assert!(markup.contains("data-vertical:self-stretch"), "{markup}");
    }

    /// Radix's rule: `role="separator"` when announced, and `aria-orientation`
    /// only for the value the role does not already imply.
    #[test]
    fn an_announced_separator_takes_radixs_semantics() {
        let horizontal = separator().announced().render().into_test_string();
        assert!(horizontal.contains(r#"role="separator""#), "{horizontal}");
        assert!(!horizontal.contains("aria-orientation"), "{horizontal}");

        let vertical = separator()
            .announced()
            .vertical()
            .render()
            .into_test_string();
        assert!(vertical.contains(r#"role="separator""#), "{vertical}");
        assert!(
            vertical.contains(r#"aria-orientation="vertical""#),
            "{vertical}"
        );
    }

    /// `decorative` is shadcn's word and its meaning, whichever way it is set.
    #[test]
    fn decorative_and_announced_are_the_same_switch() {
        assert_eq!(
            separator().decorative(false).render().into_test_string(),
            separator().announced().render().into_test_string()
        );
        assert_eq!(
            separator().decorative(true).render().into_test_string(),
            separator().render().into_test_string()
        );
    }

    /// The preflight would draw a second line, so an `<hr>` gets `border-0`
    /// without the call site asking.
    #[test]
    fn an_hr_resets_the_preflights_border() {
        let markup = separator().rule().render().into_test_string();

        assert!(markup.starts_with("<hr "), "{markup}");
        assert!(markup.ends_with("/>") || markup.ends_with(">"), "{markup}");
        assert!(markup.contains("border-0"), "{markup}");
        assert!(markup.contains("bg-border"), "{markup}");

        // The `<div>` never carries it — there is nothing there to reset.
        assert!(!separator().render().into_test_string().contains("border-0"));
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = separator().class("my-4").render().into_test_string();

        assert!(!markup.contains(r#"class="{twMerge(&quot;"#), "{markup}");
        assert!(markup.contains("my-4"), "{markup}");
        assert!(markup.contains("bg-border"), "{markup}");
    }

    /// The `hr` reset has to survive an override too — it is part of what the
    /// component chose, not part of what the caller wrote.
    #[test]
    fn an_overridden_hr_still_hands_over_its_reset() {
        let markup = separator().rule().class("my-8").render().into_test_string();

        assert!(markup.contains("border-0"), "{markup}");
        assert!(markup.contains("my-8"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_separator_with_no_override_needs_no_runtime() {
        let props = separator().id("end-of-intro");
        let markup = props.render().into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="shrink-0"#), "{markup}");

        assert!(!separator().class("my-4").needs_scope());
        assert!(
            separator()
                .bind(
                    "hidden",
                    crate::components::rahti_ui::attrs::ui_expr!(hidden = { collapsed })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = separator()
            .on(
                "click",
                crate::components::rahti_ui::attrs::ui_handler!({ collapse() }),
            )
            .render()
            .into_test_string();

        assert!(markup.contains(r#"pp-onclick="collapse()""#), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = separator()
            .attr("role", "presentation")
            .attr("title", "End of section")
            .render()
            .into_test_string();

        assert!(markup.contains(r#"role="presentation""#), "{markup}");
        assert!(markup.contains(r#"title="End of section""#), "{markup}");
        assert!(!markup.contains(r#"role="none""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = separator()
            .attrs(Attrs::new().unset("data-slot").set("title", "Break"))
            .render()
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains(r#"title="Break""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = separator()
            .id(r#"a" pp-onclick="steal()"#)
            .render()
            .into_test_string();

        assert!(!markup.contains("pp-onclick=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn separator_variants_is_the_components_own_class_list() {
        assert_eq!(
            separator_variants("my-4"),
            separator().class("my-4").class_list()
        );
        // The `hr` list is the one it cannot answer for — see its docs.
        assert_ne!(
            separator_variants(""),
            separator().rule().class("").class_list()
        );
    }

    #[test]
    fn an_orientation_is_parsed_from_the_prop() {
        assert_eq!(
            SeparatorOrientation::parse("vertical"),
            SeparatorOrientation::Vertical
        );
        assert_eq!(
            SeparatorOrientation::parse(""),
            SeparatorOrientation::Horizontal
        );
        assert_eq!(
            SeparatorOrientation::parse_or("", SeparatorOrientation::Vertical),
            SeparatorOrientation::Vertical
        );
    }

    /// The call form behind `<Separator orientation="vertical" />`. A tag fills
    /// the props it names and takes these defaults for the rest.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Separator(
            "",
            "vertical",
            "",
            "",
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"data-orientation="vertical""#),
            "{markup}"
        );
        assert!(markup.contains(r#"role="none""#), "{markup}");

        let rule = Separator(
            "",
            "",
            "hr",
            "",
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();
        assert!(rule.starts_with("<hr "), "{rule}");
        assert!(rule.contains("border-0"), "{rule}");
    }

    /// `announced=@{true}` is shadcn's `decorative={false}`. The rename is the
    /// whole reason this test exists.
    #[test]
    fn the_announced_prop_is_shadcns_decorative_inverted() {
        let quiet = Separator(
            "",
            "",
            "",
            "",
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();
        let loud = Separator(
            "",
            "",
            "",
            "",
            true,
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(quiet.contains(r#"role="none""#), "{quiet}");
        assert!(loud.contains(r#"role="separator""#), "{loud}");
    }

    /// Every prop left unwritten, which is what `<Separator />` compiles to now
    /// that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_separator() {
        let markup = Separator(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            bool::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="separator""#), "{markup}");
        assert!(
            markup.contains(r#"data-orientation="horizontal""#),
            "{markup}"
        );
        assert!(markup.contains(r#"role="none""#), "{markup}");
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// What `<Separator hidden={collapsed} />` compiles to. The expression
    /// arrives as source and goes onto the element unread — and the block gets
    /// the `<script>` that makes it mountable.
    #[test]
    fn native_bindings_land_on_the_separator() {
        let markup = Separator(
            "",
            "",
            "",
            "",
            false,
            crate::rahti::Bindings::__from_html(&[("hidden", "collapsed")]),
            Attrs::new(),
        )
        .into_test_string();

        assert!(markup.contains(r#"hidden="{collapsed}""#), "{markup}");
        assert!(markup.contains("<script"), "{markup}");
        // One literal element, script inside: still a real sibling of whatever
        // it divides.
        assert!(markup.starts_with("<div "), "{markup}");
    }

    /// `SeparatorProps` is `Default`, which is what lets a tag omit every prop.
    /// A caller's own height and `data-vertical:h-full` are the same question,
    /// and nothing downstream resolves them — so the component's answer is
    /// dropped before the join.
    ///
    /// Measured in a browser: without this, a `class="mr-2 h-4"` separator
    /// filled a 56px header instead of centring a 16px rule in it.
    #[test]
    fn a_callers_height_takes_the_rule_off_the_component() {
        let plain = separator_variants("");
        let sized = separator_variants("mr-2 h-4");

        assert!(plain.contains("data-vertical:self-stretch"), "{plain}");
        assert!(!sized.contains("self-stretch"), "{sized}");
        assert!(sized.contains("h-4"), "{sized}");
        // The rest of the rule is untouched.
        assert!(sized.contains("data-vertical:w-px"), "{sized}");
        assert!(sized.contains("bg-border"), "{sized}");

        // A prefixed height is a different question and leaves ours alone.
        let breakpoint = separator_variants("md:h-6");
        assert!(
            breakpoint.contains("data-vertical:self-stretch"),
            "{breakpoint}"
        );
    }

    /// The other spelling of the same line.
    ///
    /// shadcn's current `sidebar-07` writes the height behind the orientation
    /// variant rather than bare. It is the same question, so it has to take the
    /// same utility off — a scoped height that left `self-stretch` standing
    /// would pin the rule to the top of the header exactly as a bare one did.
    #[test]
    fn an_orientation_scoped_height_counts_as_the_callers_own() {
        for class in [
            "mr-2 data-[orientation=vertical]:h-4",
            "mr-2 data-vertical:h-4",
        ] {
            let sized = separator_variants(class);

            assert!(!sized.contains("self-stretch"), "{class}: {sized}");
            assert!(sized.contains("h-4"), "{class}: {sized}");
            assert!(sized.contains("data-vertical:w-px"), "{class}: {sized}");
        }

        // Still only a height, and still only at that scope.
        assert!(
            separator_variants("data-[orientation=vertical]:w-1")
                .contains("data-vertical:self-stretch")
        );
        assert!(separator_variants("md:h-6").contains("data-vertical:self-stretch"));
        assert!(separator_variants("data-horizontal:h-1").contains("data-vertical:self-stretch"));
    }

    #[test]
    fn props_default_is_a_plain_separator() {
        assert_eq!(SeparatorProps::new(), SeparatorProps::default());
        assert_eq!(
            separator().render().into_test_string(),
            SeparatorProps::new().render().into_test_string()
        );
    }
}

/// The layer-independence check behind [`BASE`] and [`HR_RESET`].
///
/// One layer for a `<div>` and two for an `<hr>`, so this is the same claim
/// [`Label`](super::label) makes and a smaller one than
/// [`Button`](super::button)'s — but it is what lets
/// [`SeparatorProps::class_list`] be a join rather than a merge.
///
/// The case worth naming is `data-horizontal:w-full` beside
/// `data-vertical:w-px`: one family, two answers, and no conflict, because the
/// variant prefixes are different and the two never apply at once. The audit
/// reads them the way Tailwind does and lets them stand.
#[cfg(test)]
mod audit {
    use super::{BASE, HR_RESET};
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("BASE", BASE.to_string()),
            ("BASE + HR_RESET", cn(&[BASE, HR_RESET])),
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
}
