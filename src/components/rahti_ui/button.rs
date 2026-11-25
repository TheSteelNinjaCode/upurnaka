// rahti-ui: button
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Button — a port of shadcn/ui's `button`.
//!
//! The class strings are shadcn's, copied rather than reinterpreted, so the
//! rendered button is pixel-identical to the React one under the same
//! Tailwind theme. What changes is how the props get there.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Button>"Save"</Button>
//! <Button variant="outline" size="sm">"Cancel"</Button>
//! <Button variant="destructive" disabled>"Delete"</Button>
//! <Button variant="link" href="/docs">"Docs"</Button>
//! <Button variant="outline" onclick={setCount(count + 1)}>"Increment"</Button>
//! ```
//!
//! Every prop is optional, which a Rust struct literal does not normally
//! allow — a component tag is sugar over one, and a literal has no defaults.
//! `html!` closes a props literal with `..Default::default()`, so an
//! unwritten prop is the component's default and the tag reads the way
//! `<Button variant="outline">` reads in TSX. The trade is that a forgotten
//! prop is no longer a compile error, and every prop type must be `Default`.
//!
//! `variant` and `size` are strings for the same reason they are strings in
//! TypeScript: it is what reads well at the call site. [`ButtonVariant`] and
//! [`ButtonSize`] are still the real types underneath, and the strings are
//! their [`slug`](ButtonVariant::slug)s.
//!
//! ## The button is UI. The event is the caller's.
//!
//! `onclick={…}` is a native HTML event attribute, and this component does
//! nothing with it but put it on the `<button>` — the same way a React
//! Button spreads `onClick` onto its element without reading it. `html!`
//! hands every `{…}` prop over as the source the author wrote, and they
//! arrive together in `bindings`.
//!
//! Where the handler *runs* is the part worth knowing. The component is its
//! own PulsePoint scope, but an event expression is resolved when the event
//! fires, by which time it reaches the calling page's state — so
//! `onclick={setCount(count + 1)}` drives a counter declared in the page's
//! `<script>`, through the component, with nothing passed down.
//!
//! An attribute *binding* crosses the same way: `disabled={count === 0}` on the
//! tag lands on the `<button>`, and a component's root attributes are evaluated
//! where the tag was written, so it reads the page's `count`. A `{…}` written
//! between the tags reads the page too — children belong to the block that
//! wrote them.
//!
//! ## When the tag is not enough
//!
//! **[`ButtonProps::render`], interpolated — for a button built in Rust.**
//!
//! ```ignore
//! @{button().outline()
//!     .on("click", crate::components::rahti_ui::attrs::ui_handler!({setCount(count + 1)}))
//!     .bind("aria-pressed", crate::components::rahti_ui::attrs::ui_expr!(aria-pressed={count > 0}))
//!     .render(Html::escape("Increment"))}
//! ```
//!
//! `render` returns the element and nothing around it, so interpolating it
//! puts the button in the calling block's scope, where bindings resolve.
//! [`ButtonProps`] is the same value the tag builds internally.
//!
//! **[`ButtonProps::attrs`] — a set of attributes, forwarded.**
//!
//! ```ignore
//! <Button attrs=@{Attrs::new().set("form", "signup").unset("data-size")}>
//!     "Sign up"
//! </Button>
//! ```
//!
//! shadcn's `{...props}`, as a named prop. See `attrs.rs`.
//!
//! **[`button_variants`] — the class list alone.**
//!
//! ```ignore
//! <a href="/docs" class=@{button_variants(ButtonVariant::Link, ButtonSize::Sm, "")}>
//!     "Docs"
//! </a>
//! ```
//!
//! shadcn's own escape hatch, for when the element has to be something this
//! component does not render.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// Everything a button is, before variant and size choose between them.
///
/// ## Why this is not shadcn's base string verbatim
///
/// cva concatenates its three layers blindly and leaves `cn()` to sort out
/// the wreckage, so shadcn's base carries a `gap-2`, a `text-sm`, a
/// `focus-visible:ring-ring/50` and an svg `size-4` that four of the sizes
/// and one of the variants then have to override. Every one of the 48
/// variant×size pairs depends on a merge to render correctly — not for the
/// caller's sake, for its own.
///
/// We are not obliged to inherit that. The same utilities are distributed
/// here so that no layer contradicts another: whatever a size decides about
/// gap, text size or icon size, it decides alone, and the ring color belongs
/// to the variant. Concatenating the three is then exactly what merging them
/// would produce, which `audit::no_layer_contradicts_another` holds us to.
///
/// The rendered class list is unchanged. What changes is that producing it
/// no longer needs a conflict resolver, so a button is styled by the server
/// and stays styled with JavaScript switched off.
/// Every button's shared layer, public so that a component which has to spell
/// out its own size can still join onto it.
///
/// [`Pagination`](super::pagination) is the case: shadcn's `cn()` is
/// tailwind-merge, so its pagination controls simply pass `gap-1 px-2.5` and
/// let the merge beat [`ButtonSize::Default`]'s `gap-2 px-4`. This library
/// joins rather than merges, so the loser would be written beside the winner
/// and Tailwind's stylesheet order would decide — wrongly, as it happens.
/// Composing `BASE` + a variant + a size layer that never had the conflicting
/// tokens is the same answer with no merge required, which is the arrangement
/// [`ButtonSize::class`] already argues for.
pub const BASE: &str = "inline-flex shrink-0 items-center justify-center rounded-md \
                    font-medium whitespace-nowrap transition-all outline-none \
                    focus-visible:border-ring focus-visible:ring-[3px] \
                    disabled:pointer-events-none disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                    dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none \
                    [&_svg]:shrink-0";

/// How loud the button is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    /// The utilities this variant adds to [`BASE`].
    ///
    /// The focus ring color lives here rather than in the base, because
    /// `Destructive` wants its own and shadcn resolves that collision with a
    /// merge. See [`BASE`].
    pub const fn class(self) -> &'static str {
        match self {
            Self::Default => {
                "bg-primary text-primary-foreground hover:bg-primary/90 \
                 focus-visible:ring-ring/50"
            }
            Self::Destructive => {
                "bg-destructive text-white hover:bg-destructive/90 \
                 focus-visible:ring-destructive/20 dark:bg-destructive/60 \
                 dark:focus-visible:ring-destructive/40"
            }
            Self::Outline => {
                "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground \
                 dark:border-input dark:bg-input/30 dark:hover:bg-input/50 \
                 focus-visible:ring-ring/50"
            }
            Self::Secondary => {
                "bg-secondary text-secondary-foreground hover:bg-secondary/80 \
                 focus-visible:ring-ring/50"
            }
            Self::Ghost => {
                "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50 \
                 focus-visible:ring-ring/50"
            }
            Self::Link => {
                "text-primary underline-offset-4 hover:underline focus-visible:ring-ring/50"
            }
        }
    }

    /// The name shadcn writes into `data-variant`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
            Self::Outline => "outline",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Link => "link",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `variant="…"` prop.
    ///
    /// An empty string is the unwritten prop and means the default. Anything
    /// else unrecognized is a typo: it renders as the default so a page is
    /// never broken by one, and trips a `debug_assert` so it does not go
    /// unnoticed in development.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "destructive" => Self::Destructive,
            "outline" => Self::Outline,
            "secondary" => Self::Secondary,
            "ghost" => Self::Ghost,
            "link" => Self::Link,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Button variant. \
                     One of: default, destructive, outline, secondary, ghost, link."
                );
                Self::Default
            }
        }
    }
}

/// How big it is. The `Icon*` sizes are square and drop the label padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

impl ButtonSize {
    /// The utilities this size adds to [`BASE`].
    ///
    /// Gap, text size and icon size are spelled out on every size rather
    /// than inherited from the base and overridden by four of them. Same
    /// result, no merge required. See [`BASE`].
    pub const fn class(self) -> &'static str {
        match self {
            Self::Default => {
                "h-9 gap-2 px-4 py-2 text-sm has-[>svg]:px-3 \
                 [&_svg:not([class*='size-'])]:size-4"
            }
            Self::Xs => {
                "h-6 gap-1 px-2 text-xs has-[>svg]:px-1.5 \
                 [&_svg:not([class*='size-'])]:size-3"
            }
            Self::Sm => {
                "h-8 gap-1.5 px-3 text-sm has-[>svg]:px-2.5 \
                 [&_svg:not([class*='size-'])]:size-4"
            }
            Self::Lg => {
                "h-10 gap-2 px-6 text-sm has-[>svg]:px-4 \
                 [&_svg:not([class*='size-'])]:size-4"
            }
            Self::Icon => "size-9 gap-2 text-sm [&_svg:not([class*='size-'])]:size-4",
            Self::IconXs => "size-6 gap-1 text-xs [&_svg:not([class*='size-'])]:size-3",
            Self::IconSm => "size-8 gap-2 text-sm [&_svg:not([class*='size-'])]:size-4",
            Self::IconLg => "size-10 gap-2 text-sm [&_svg:not([class*='size-'])]:size-4",
        }
    }

    /// The name shadcn writes into `data-size`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Lg => "lg",
            Self::Icon => "icon",
            Self::IconXs => "icon-xs",
            Self::IconSm => "icon-sm",
            Self::IconLg => "icon-lg",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `size="…"` prop. Same
    /// rules as [`ButtonVariant::parse`].
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "xs" => Self::Xs,
            "sm" => Self::Sm,
            "lg" => Self::Lg,
            "icon" => Self::Icon,
            "icon-xs" => Self::IconXs,
            "icon-sm" => Self::IconSm,
            "icon-lg" => Self::IconLg,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Button size. \
                     One of: default, xs, sm, lg, icon, icon-xs, icon-sm, icon-lg."
                );
                Self::Default
            }
        }
    }
}

/// The class list shadcn's `buttonVariants({ variant, size, className })`
/// returns: base, variant, size, then the caller's own classes, merged so
/// the caller's win.
///
/// Reach for this when the element has to belong to the page — a link
/// styled as a button, or a button whose handler reads the page's state.
pub fn button_variants(variant: ButtonVariant, size: ButtonSize, class: &str) -> String {
    cn(&[BASE, variant.class(), size.class(), class])
}

/// The tags a button is allowed to render as.
///
/// Closed rather than open because `html!` writes a tag name literally: the
/// element is chosen by a `match` in [`ButtonProps::render`], and this is the
/// same list read back so `element("script")` is refused where it is set
/// rather than silently falling through to `<button>` at render time.
const ELEMENTS: [&str; 5] = ["button", "a", "label", "span", "div"];

/// A button's props, with shadcn's defaults already in place.
///
/// Build one with [`button`] and narrow it down a method at a time. Nothing
/// here is required.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonProps {
    variant: ButtonVariant,
    size: ButtonSize,
    class: String,
    element: &'static str,
    disabled: bool,
    attrs: Attrs,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            class: String::new(),
            element: "button",
            disabled: false,
            attrs: Attrs::new(),
        }
    }
}

/// A default button's props — the start of every chain.
pub fn button() -> ButtonProps {
    ButtonProps::default()
}

impl ButtonProps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Classes laid over the variant's, `className`-style: a conflicting
    /// utility written here wins.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Render as something other than `<button>` — shadcn's `asChild`,
    /// without the child. A tag outside [`ELEMENTS`] is ignored.
    ///
    /// An `<a>` has no `disabled`, so a disabled one is marked
    /// `aria-disabled` and made inert with `pointer-events-none` instead.
    pub fn element(mut self, tag: &'static str) -> Self {
        if ELEMENTS.contains(&tag) {
            self.element = tag;
        }
        self
    }

    /// Render as an `<a href="…">`.
    pub fn href(self, url: impl Into<String>) -> Self {
        self.element("a").attr("href", url)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `type="submit"`. Left unset by default, as in shadcn — a `<button>`
    /// in a form submits it unless told otherwise, and that is HTML's rule
    /// rather than this component's to override.
    pub fn submit(self) -> Self {
        self.attr("type", "submit")
    }

    pub fn reset(self) -> Self {
        self.attr("type", "reset")
    }

    /// `type="button"` — the opt-out for a button that sits in a form and
    /// must not submit it.
    pub fn plain(self) -> Self {
        self.attr("type", "button")
    }

    /// Any attribute, rendered on the server: `attr("aria-label", "Close")`.
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

    /// Take an attribute off, whatever set it: `unset("data-size")`.
    ///
    /// The one thing a value cannot express, and the reason a forwarded
    /// [`Attrs`] is richer than a list of pairs.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("aria-expanded", "open")`.
    ///
    /// Through the [`Button`] tag the same binding is a `{…}` prop: it lands on
    /// the root, and a component's root attributes are evaluated where the tag
    /// was written. This is its spelling for an element
    /// [`render`](Self::render)ed into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression: `on("click", "save()")`.
    ///
    /// The `on` prefix is optional — `on("click", …)` and `on("onclick", …)`
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

    // Variant shorthands, so a call site reads like the React one.
    pub fn default_variant(self) -> Self {
        self.variant(ButtonVariant::Default)
    }
    pub fn destructive(self) -> Self {
        self.variant(ButtonVariant::Destructive)
    }
    pub fn outline(self) -> Self {
        self.variant(ButtonVariant::Outline)
    }
    pub fn secondary(self) -> Self {
        self.variant(ButtonVariant::Secondary)
    }
    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }
    pub fn link(self) -> Self {
        self.variant(ButtonVariant::Link)
    }

    // Size shorthands.
    pub fn xs(self) -> Self {
        self.size(ButtonSize::Xs)
    }
    pub fn sm(self) -> Self {
        self.size(ButtonSize::Sm)
    }
    pub fn lg(self) -> Self {
        self.size(ButtonSize::Lg)
    }
    pub fn icon(self) -> Self {
        self.size(ButtonSize::Icon)
    }
    pub fn icon_xs(self) -> Self {
        self.size(ButtonSize::IconXs)
    }
    pub fn icon_sm(self) -> Self {
        self.size(ButtonSize::IconSm)
    }
    pub fn icon_lg(self) -> Self {
        self.size(ButtonSize::IconLg)
    }

    /// The class list this button renders with, the caller's own included.
    ///
    /// A join, not a merge: the three layers are built not to contradict
    /// each other (see [`BASE`]), so this is exact for everything except a
    /// caller's `.class(…)` override, which is the one thing that can
    /// conflict. Resolve conflicts with an explicit frontend `twMerge` binding — see [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[&self.component_classes(), &self.class])
    }

    /// The class list without the caller's override: what `twMerge` merges
    /// *into*.
    fn component_classes(&self) -> String {
        let disabled_on_anchor = if self.disabled && self.element != "button" {
            "pointer-events-none opacity-50"
        } else {
            ""
        };
        cn(&[
            BASE,
            self.variant.class(),
            self.size.class(),
            disabled_on_anchor,
        ])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// The script does not cost this button its caller's state. A click is
    /// resolved when it fires, and reaches the page's scope through the
    /// boundary either way — `/components/button` increments the same `count`
    /// through a component tag as through a native one. See [`Button`].
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
        let mut base = Attrs::new()
            .set("data-slot", "button")
            .set("data-variant", self.variant.slug())
            .set("data-size", self.size.slug());

        // Static class overrides compose data; they do not resolve conflicts.
        // Supply an explicit compiled `class={twMerge(...)}` binding when
        // overriding utilities from the component defaults.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&self.component_classes(), &self.class]),
            )
        };

        if self.disabled {
            base = if self.element == "button" {
                // `disabled=""` — HTML reads a boolean attribute by whether
                // it is there, and this is the spelling `html!` itself emits
                // for `disabled=@{true}`.
                base.set("disabled", "")
            } else {
                base.set("aria-disabled", "true")
            };
        }

        // The caller's own, last, so a `{...props}` spread wins — the same
        // last-wins rule `AttrList` applies when it writes the tag.
        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the button in that block's
    /// scope, which is what a handler or a binding written here needs.
    ///
    /// The element is written by `html!` with the attribute set spread onto
    /// it, which is how a Rahti component forwards attributes it cannot name:
    /// `@{..attrs}` validates every name, escapes every value, and lets a
    /// later attribute replace an earlier one in place. The tag itself is
    /// matched rather than interpolated because a tag name is markup — the
    /// arms are [`ELEMENTS`], spelled out.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. [`Button`] adds one when it needs to
    /// ([`needs_scope`](Self::needs_scope)); markup interpolated into a page
    /// helper that has neither script nor state is never mounted, and an
    /// override there silently does nothing.
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();

        match self.element {
            "a" => html! {
                <a @{..attrs}>
                    <slot />
                </a>
            },
            "label" => html! {
                <label @{..attrs}>
                    <slot />
                </label>
            },
            "span" => html! {
                <span @{..attrs}>
                    <slot />
                </span>
            },
            "div" => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
            _ => html! {
                <button @{..attrs}>
                    <slot />
                </button>
            },
        }
    }
}

/// A button.
///
/// ```ignore
/// <Button>"Save"</Button>
/// <Button variant="outline" size="sm">"Cancel"</Button>
/// <Button variant="destructive" disabled>"Delete"</Button>
/// <Button variant="link" href="/docs">"Docs"</Button>
/// <Button class="w-full">"Wide"</Button>
/// ```
///
/// The props are shadcn's, spelled the way its JSX spells them, and every
/// one is optional — `html!` closes a props literal with
/// `..Default::default()`, so an unwritten prop is this component's default.
/// `variant` and `size` are strings for the same reason they are strings in
/// TypeScript: it is what reads well at the call site. An unrecognized one
/// falls back to the default, and trips a `debug_assert` on the way so a
/// typo is loud in development and harmless in production.
///
/// Native events are written as native events, and land on the `<button>`
/// untouched:
///
/// ```ignore
/// <Button variant="outline" onclick={setCount(count + 1)}>"Increment"</Button>
/// <Button onclick={save()} onkeydown={maybeSubmit(event)}>"Save"</Button>
/// ```
///
/// `attrs` is the rest of the `{...props}` spread — server-side values with
/// no `{…}` in them, and names a Rust identifier cannot spell:
///
/// ```ignore
/// <Button attrs=@{Attrs::new().set("type", "submit").set("aria-label", "Sign up")}>
///     "Sign up"
/// </Button>
/// ```
///
/// The fragment root is what carries the component's identity — a
/// `#[component]` has to end in `html!`, and the element itself comes back
/// from [`ButtonProps::render`] already built. That identity is also a
/// PulsePoint scope, so a button bound to a page's state wants
/// [`ButtonProps::render`] instead; see the module docs.
#[component]
// A component's parameters are its props, and eight is what this one has.
// The lint is measuring a function; the call site is a tag, where each of
// these is named and optional.
#[allow(clippy::too_many_arguments)]
pub fn Button(
    variant: &str,
    size: &str,
    class: &str,
    href: &str,
    disabled: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = ButtonProps::default()
        .variant(ButtonVariant::parse(variant))
        .size(ButtonSize::parse(size))
        .class(class)
        .disabled(disabled)
        .attrs(attrs);

    // `onclick={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the button unchanged —
    // the component decides nothing about it, exactly as a React Button
    // spreads `onClick` onto its `<button>` without reading it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    if !href.is_empty() {
        props = props.href(href);
    }

    // Forwarded bindings retain the caller's scope. Only precompiled
    // bindings supplied through the builder attrs need a local scope when
    // no caller bindings are present; a static class override creates none.
    if props.needs_scope() && bindings.is_empty() {
        let button = props.render(children);
        html! {
            <>
                @{button}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        // One literal element, not `<>@{button}</>`, and the element is
        // matched here rather than interpolated for the same reason
        // [`ButtonProps::render`] matches it: `html!` writes a tag name
        // literally.
        //
        // A fragment root has no element to carry this block's name, so it is
        // delimited by `<!--pp:id-->…<!--/pp-->` comments — and at hydration
        // the runtime turns that comment range into a real `<pp-fragment>`.
        // `display: contents` hides that from layout but not from a
        // combinator, so a button inside one stops being a direct child or a
        // sibling of anything the moment PulsePoint mounts. Written as this
        // block's own root, the name lands on the element itself.
        let attrs = props.attr_list();
        match props.element {
            "a" => html! {
                <a @{..attrs}>
                    <slot />
                </a>
            },
            "label" => html! {
                <label @{..attrs}>
                    <slot />
                </label>
            },
            "span" => html! {
                <span @{..attrs}>
                    <slot />
                </span>
            },
            "div" => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
            _ => html! {
                <button @{..attrs}>
                    <slot />
                </button>
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::audit::{SIZES, VARIANTS};
    use super::{Attrs, Button, ButtonSize, ButtonVariant, button, button_variants};
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_button_carries_shadcns_defaults() {
        let markup = button().render(Html::escape("Save")).into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(markup.ends_with(">Save</button>"), "{markup}");
        assert!(markup.contains(r#"data-slot="button""#));
        assert!(markup.contains(r#"data-variant="default""#));
        assert!(markup.contains(r#"data-size="default""#));
        assert!(markup.contains("bg-primary"));
        assert!(markup.contains("h-9"));
    }

    #[test]
    fn variant_and_size_replace_the_defaults_they_conflict_with() {
        let markup = button()
            .outline()
            .sm()
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-variant="outline""#));
        assert!(!markup.contains("bg-primary"));
        assert!(markup.contains("hover:bg-accent"));
        assert!(markup.contains("h-8"));
        assert!(!markup.contains("h-9"));
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = button()
            .class("bg-emerald-600 w-full")
            .render(Html::empty())
            .into_test_string();

        // The component's own list and the override, as two arguments.
        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("w-full"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no
    /// binding, and no scope for the runtime to mount.
    #[test]
    fn a_button_with_no_override_needs_no_runtime() {
        let props = button().outline().sm();
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="inline-flex"#), "{markup}");

        assert!(!button().class("w-full").needs_scope());
        assert!(
            button()
                .on(
                    "click",
                    crate::components::rahti_ui::attrs::ui_handler!({ save() })
                )
                .needs_scope()
        );
    }

    /// `disabled=""` rather than a bare `disabled`: HTML reads a boolean
    /// attribute by whether it is there, and this is the spelling `html!`
    /// emits for `disabled=@{true}`.
    #[test]
    fn disabled_is_an_attribute_on_a_button_and_aria_on_a_link() {
        let markup = button()
            .disabled(true)
            .render(Html::empty())
            .into_test_string();
        assert!(markup.contains(r#" disabled="""#), "{markup}");

        let markup = button()
            .href("/docs")
            .disabled(true)
            .render(Html::empty())
            .into_test_string();
        assert!(markup.starts_with("<a "), "{markup}");
        assert!(markup.contains(r#"aria-disabled="true""#));
        assert!(markup.contains("pointer-events-none"));
        // The `disabled:` utilities in the class list are not the attribute.
        assert!(!markup.contains(r#" disabled="""#), "{markup}");
    }

    /// The thing a list of pairs cannot say, and the reason the forwarded
    /// prop is an `Attrs` rather than a `Vec<(&str, &str)>`.
    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = button()
            .attrs(Attrs::new().unset("data-size").set("form", "signup"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("data-size"), "{markup}");
        assert!(markup.contains(r#"data-variant="default""#), "{markup}");
        assert!(markup.contains(r#"form="signup""#), "{markup}");
    }

    /// Escaping is the framework's now — `AttrList` writes every value. This
    /// holds the property that mattered when it was ours.
    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = button()
            .attr("title", r#"a" pp-onmouseover="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onmouseover=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    /// A name markup cannot carry used to be dropped in silence. The
    /// framework is louder about it: a `debug_assert` in development, and the
    /// attribute left off in release.
    #[test]
    #[should_panic(expected = "not an attribute name markup can carry")]
    #[cfg(debug_assertions)]
    fn an_illegal_attribute_name_is_loud_in_development() {
        button()
            .attr("on click\" x=\"y", "boom")
            .render(Html::empty());
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = button()
            .attr("data-slot", "mine")
            .submit()
            .attr("aria-label", "Save & close")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#));
        assert!(markup.contains(r#"type="submit""#));
        assert!(markup.contains(r#"aria-label="Save &amp; close""#));
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = button()
            .on(
                "click",
                crate::components::rahti_ui::attrs::ui_handler!({ setOpen(!open) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onclick="setOpen(!open)""#),
            "{markup}"
        );
    }

    #[test]
    fn children_pass_through_as_markup() {
        let markup = button()
            .render(Html::from_raw("<svg></svg>Send".into()))
            .into_test_string();

        assert!(markup.contains("<svg></svg>Send</button>"), "{markup}");
    }

    #[test]
    fn an_unknown_element_is_refused() {
        let markup = button()
            .element("script")
            .render(Html::empty())
            .into_test_string();
        assert!(markup.starts_with("<button "), "{markup}");
    }

    #[test]
    fn button_variants_is_the_components_own_class_list() {
        assert_eq!(
            button_variants(ButtonVariant::Ghost, ButtonSize::Lg, "w-full"),
            button().ghost().lg().class("w-full").class_list()
        );
    }

    /// The call form behind `<Button variant="secondary">"Hi"</Button>`.
    /// A tag fills the props it names and takes these defaults for the rest.
    #[test]
    fn the_component_wraps_the_same_element() {
        let markup = Button(
            "secondary",
            "",
            "",
            "",
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Hi"),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-variant="secondary""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains(">Hi</button>"), "{markup}");
    }

    /// Every prop left unwritten, which is what `<Button>"Save"</Button>`
    /// compiles to now that `html!` closes the props literal with
    /// `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_button() {
        let markup = Button(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            bool::default(),
            crate::rahti::Bindings::__from_html(<&[(&str, &str)]>::default()),
            Attrs::default(),
            Html::escape("Save"),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-variant="default""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains("bg-primary"), "{markup}");
        assert!(markup.contains(">Save</button>"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"href="""#), "{markup}");
        assert!(!markup.contains(" disabled>"), "{markup}");
    }

    /// What `<Button onclick={setCount(count + 1)}>` compiles to. The
    /// expression arrives as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_button() {
        let markup = Button(
            "outline",
            "",
            "",
            "",
            false,
            crate::rahti::Bindings::__from_html(&[
                ("onclick", "setCount(count + 1)"),
                ("onkeydown", "maybeSubmit(event)"),
            ]),
            Attrs::new(),
            Html::escape("Increment"),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"pp-onclick="setCount(count + 1)""#),
            "{markup}"
        );
        assert!(
            markup.contains(r#"pp-onkeydown="maybeSubmit(event)""#),
            "{markup}"
        );
        // Byte-identical to the same handler written on a native element,
        // which is what lets the runtime treat it as one.
        assert!(markup.contains(r#"data-variant="outline""#));
    }

    #[test]
    fn variant_and_size_parse_from_the_strings_the_tag_writes() {
        assert_eq!(ButtonVariant::parse(""), ButtonVariant::Default);
        assert_eq!(ButtonVariant::parse("outline"), ButtonVariant::Outline);
        assert_eq!(ButtonSize::parse(""), ButtonSize::Default);
        assert_eq!(ButtonSize::parse("icon-lg"), ButtonSize::IconLg);

        // Every slug round-trips, so the prop and `data-*` never disagree.
        for variant in VARIANTS {
            assert_eq!(ButtonVariant::parse(variant.slug()), variant);
        }
        for size in SIZES {
            assert_eq!(ButtonSize::parse(size.slug()), size);
        }
    }

    /// A typo is a release-mode fallback and a debug-mode panic. This checks
    /// the fallback; the `debug_assert` is what a developer sees.
    #[test]
    #[cfg(not(debug_assertions))]
    fn an_unknown_variant_falls_back() {
        assert_eq!(ButtonVariant::parse("outlien"), ButtonVariant::Default);
    }

    #[test]
    #[should_panic(expected = "not a Button variant")]
    #[cfg(debug_assertions)]
    fn an_unknown_variant_is_loud_in_development() {
        ButtonVariant::parse("outlien");
    }
}

/// The layer-independence check behind [`BASE`].
///
/// The reading of a utility — its variant prefix, its conflict family — lives
/// in `utils::audit`, shared with every other component. What is here is the
/// claim this file makes and the data it is checked over.
#[cfg(test)]
mod audit {
    use super::*;
    use crate::components::rahti_ui::utils::audit::clashes;

    pub(super) const VARIANTS: [ButtonVariant; 6] = [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ];

    pub(super) const SIZES: [ButtonSize; 8] = [
        ButtonSize::Default,
        ButtonSize::Xs,
        ButtonSize::Sm,
        ButtonSize::Lg,
        ButtonSize::Icon,
        ButtonSize::IconXs,
        ButtonSize::IconSm,
        ButtonSize::IconLg,
    ];

    /// The claim [`BASE`] makes: no two utilities in a rendered class list
    /// compete, so joining the layers is what merging them would give.
    ///
    /// This held for 0 of 48 combinations with shadcn's strings as written,
    /// and holds for 48 of 48 with them redistributed. Checked against
    /// `tailwind_fuse` before that dependency was dropped; this is the
    /// standing guard.
    #[test]
    fn no_layer_contradicts_another() {
        let mut found: Vec<String> = Vec::new();

        for variant in VARIANTS {
            for size in SIZES {
                let list = cn(&[BASE, variant.class(), size.class()]);
                found.extend(
                    clashes(&list)
                        .into_iter()
                        .map(|clash| format!("{}/{}: {clash}", variant.slug(), size.slug())),
                );
            }
        }

        assert!(
            found.is_empty(),
            "{} clashes:\n{}",
            found.len(),
            found.join("\n")
        );
    }
}
