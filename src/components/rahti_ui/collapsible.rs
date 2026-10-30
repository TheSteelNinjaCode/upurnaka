// rahti-ui: collapsible
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Collapsible — a port of shadcn/ui's `collapsible`.
//!
//! The first component in this library with **no design to copy**. Both of
//! shadcn's styles ship the same three functions, and between them they contain
//! one `data-slot` each and not a single class:
//!
//! ```jsx
//! function Collapsible({ ...props }) {
//!   return <CollapsiblePrimitive.Root data-slot="collapsible" {...props} />
//! }
//! ```
//!
//! There is no `cn()` call, no `cva`, no marker class, nothing for the registry
//! and the Base UI style to disagree about. shadcn's Collapsible *is* the
//! primitive — Radix's in one style, Base UI's in the other — and everything it
//! does is behaviour.
//!
//! Which makes the only interesting question the one every other file in this
//! library answered by copying a string: what should this be made of?
//!
//! ## It is `<details>`
//!
//! HTML has this widget. A `<details>` with a `<summary>` is a disclosure, and
//! it is a disclosure to every screen reader, every keyboard, every print
//! stylesheet, and — since 2024 — every browser's find-in-page, which opens a
//! closed one to show a match inside it. None of that is written here.
//!
//! Compare what the two primitives are for. Radix's Collapsible is ~200 lines
//! that maintain an `open` boolean, mirror it into `data-state` and
//! `aria-expanded`, point `aria-controls` at a generated id, measure the panel
//! to publish `--radix-collapsible-content-height`, and keep the content
//! mounted-but-hidden so a transition can run. Take away the height custom
//! property — which is only there because CSS could not animate to `auto` when
//! Radix was written — and the rest is a description of `<details>`.
//!
//! ```text
//! Radix                                   here
//! ─────────────────────────────────────   ──────────────────────────────────
//! Root         state, context             <details>, and the `open` attribute
//! Trigger      button, aria-expanded,     <summary>
//!              aria-controls, onClick
//! Content      hidden, data-state,        everything after the <summary>
//!              --radix-…-content-height
//! open / defaultOpen                      `open`
//! onOpenChange                            the native `toggle` event
//! ```
//!
//! **The Collapsible works with the runtime absent.** It is the first component
//! here that is not merely *drawn* without JavaScript but fully operable without
//! it — [`Slider`](super::slider) is the only other one, and for the same
//! reason. That is the whole argument. The alternative port is a `<div>`, a
//! `<button>` and a script, and a page served without its script would then show
//! a trigger that does nothing over content that is either always there or never
//! there. Every other component in this library degrades; that one would break.
//!
//! ## What that costs, exactly
//!
//! Three things, and the third is the one to read.
//!
//! **1. The trigger is a `<summary>`, not a `<button>`.** shadcn's demo writes
//! `<CollapsibleTrigger asChild><Button variant="ghost" size="icon">`, and a
//! `<button>` inside a `<summary>` is interactive content inside interactive
//! content — two tab stops where the user sees one. So the trigger *is* the
//! summary, and it takes the button's class list instead of the button:
//!
//! ```ignore
//! <CollapsibleTrigger class=@{button_variants(ButtonVariant::Ghost, ButtonSize::Icon, "size-8")}>
//!     <ChevronsUpDown class=@{TRIGGER_ICON} />
//!     <span class="sr-only">"Toggle"</span>
//! </CollapsibleTrigger>
//! ```
//!
//! **2. There is no `data-state`.** Radix writes `data-state="open"` on all
//! three parts and shadcn's own examples select on it. Writing it here would be
//! a lie the moment the user clicked, because nothing would be updating it — the
//! state lives in the `open` attribute, which is where the browser keeps it.
//! Select on that instead. [`TRIGGER_ICON`] is the recipe, and it is worth
//! reading once: `[[open]>summary_&]:rotate-180` turns a chevron when *its own*
//! collapsible is open, and the `>` is what stops a nested one reading its
//! parent's state. Same rule, and same reason, as
//! [`Resizable`](super::resizable)'s handle.
//!
//! **3. The summary has to come first.** `<summary>` is the first child of
//! `<details>` and everything after it is the disclosed content — so the free
//! composition Radix allows does not survive. shadcn's starred-repositories demo
//! puts the trigger inside a header `<div>` and then an always-visible row
//! *between* the trigger and the content; that shape cannot be written as a
//! `<details>`. What it becomes is the header row as the summary, with the
//! always-visible row inside it:
//!
//! ```ignore
//! <Collapsible class="flex w-[350px] flex-col gap-2">
//!     <CollapsibleTrigger class="flex flex-col gap-2">
//!         <div class="flex items-center justify-between gap-4 px-4">…</div>
//!         <div class="rounded-md border px-4 py-2 font-mono text-sm">"@radix-ui/primitives"</div>
//!     </CollapsibleTrigger>
//!     <CollapsibleContent class="flex flex-col gap-2">…</CollapsibleContent>
//! </Collapsible>
//! ```
//!
//! Which is arguably the better widget — the whole header becomes the target
//! rather than an eight-pixel icon — but it is a different call site, and it is
//! the one place this port asks a page to be rewritten rather than translated.
//!
//! ## Controlled, and the reason it needs no script
//!
//! `open` is a real HTML boolean attribute and `toggle` is a real native event,
//! so Radix's `open` / `onOpenChange` pair needs nothing invented. Which half
//! you can reach depends on how the Collapsible is written, and the difference
//! is measured rather than assumed:
//!
//! **Through the tag — the value, one way.**
//!
//! ```ignore
//! <Collapsible open={expanded}>
//! ```
//!
//! A component's root is a PulsePoint boundary, and a boundary reads a `{…}`
//! attribute as a *value expression* evaluated in the parent's scope. That is
//! the page's scope, so `expanded` is found and the disclosure follows it.
//! `open` is also one of the names the runtime rewrites into a true boolean
//! attribute — present when truthy, gone when falsy — which is exactly what a
//! controlled `<details>` wants and is *not* what an invented `data-open` would
//! get. The collapsible still opens on a click; the page just does not hear
//! about it.
//!
//! **Through [`render`](CollapsibleProps::render) — the value and the event.**
//!
//! ```ignore
//! @{collapsible()
//!     .bind("open", crate::components::rahti_ui::attrs::ui_expr!(open={expanded}))
//!     .on("toggle", crate::components::rahti_ui::attrs::ui_handler!({typeof setExpanded === "function" && setExpanded(target.open)}))
//!     .render(body)}
//! ```
//!
//! `ontoggle={…}` cannot go on the tag. The runtime attaches a native `on*`
//! handler to ordinary elements only; on a boundary the same attribute is read
//! as a value expression like any other and evaluated during render, where there
//! is no event and no `target` — it logs `Template Expression Failed:
//! "setExpanded(target.open)"` and the handler never fires.
//! [`render`](CollapsibleProps::render) returns the `<details>` with no boundary
//! around it, so interpolating it into the page's own block puts both the
//! binding and the handler in that block's scope and both resolve. It is the
//! same route [`Switch`](super::switch)'s page takes for the same reason, and
//! the `/components/collapsible` demo has one of each side by side.
//!
//! **A bound `open` needs a guarded `ontoggle`.** The server writes a binding
//! as the literal text `open="{expanded}"`, and an `open` attribute carrying
//! any value is an open `<details>` to the parser — so a controlled
//! disclosure is *always* open when the document is parsed, whatever the state
//! says. HTML fires `toggle` at load for a `<details>` it found open, and at
//! that point `ontoggle` is still an ordinary inline handler the browser runs
//! in global scope, where the page's state does not exist. Guard the call:
//!
//! ```ignore
//! .on("toggle", crate::components::rahti_ui::attrs::ui_handler!({typeof setExpanded === "function" && setExpanded(target.open)}))
//! ```
//!
//! `typeof` is the one operator that does not throw on a name that was never
//! declared, and `&&` short-circuits before `target` is read, so the load-time
//! run is a no-op. After mount the runtime compiles the same expression in the
//! page's scope and the call goes through. Seeding the state `true` is worth
//! doing alongside it: the parser's guess is then already right, so the
//! disclosure does not flash open on its way to closed.
//!
//! ## `name`, and the accordion that is already here
//!
//! `<details name="faq">` makes a set of collapsibles exclusive: opening one
//! closes its siblings, with no script and no shared state.
//! [`CollapsibleProps::name`] is that attribute, and it is why the Accordion
//! that comes after this one is mostly a class list.
//!
//! ## The parts
//!
//! ```html
//! <details data-slot="collapsible" open>
//!   <summary data-slot="collapsible-trigger" class="cursor-pointer list-none …">…</summary>
//!   <div data-slot="collapsible-content">…</div>
//! </details>
//! ```
//!
//! Two utilities on the trigger and none anywhere else, which is as close to
//! shadcn's nothing as a native disclosure gets: `list-none` and its WebKit twin
//! take off the UA's triangle, because shadcn's trigger is a button and has no
//! marker to speak of. [`TRIGGER`] says the rest.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Collapsible>
//!     <CollapsibleTrigger>"Can I use this in my project?"</CollapsibleTrigger>
//!     <CollapsibleContent>"Yes. Free to use for personal and commercial projects."</CollapsibleContent>
//! </Collapsible>
//!
//! <Collapsible open>…</Collapsible>
//! <Collapsible name="faq">…</Collapsible>
//! <Collapsible animated>…</Collapsible>
//! ```
//!
//! `open` is Radix's `defaultOpen` when nothing binds it and Radix's `open` when
//! something does, which is the same distinction React draws and the same one
//! HTML has always drawn.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// The trigger: the two utilities a `<summary>` needs to stop looking like one.
///
/// shadcn's trigger is a `<button>` with no class list at all, so there is
/// nothing here to copy and everything here is this port's. All of it is the
/// difference between the element shadcn used and the element this uses:
///
/// - `list-none` and `[&::-webkit-details-marker]:hidden` take off the UA's
///   disclosure triangle. Two spellings because they are two different
///   mechanisms — a `::marker` on the summary's list-item box in Firefox and
///   modern Chrome, and a separate `::-webkit-details-marker` pseudo-element in
///   Safari. Neither one alone clears both.
/// - `cursor-pointer` is what a `<button>` conveys and a `<summary>` does not:
///   the UA stylesheet gives it `cursor: default`. The same call
///   [`Questionnaire`](super::questionnaire)'s choice makes for its label.
///
/// A trigger that *should* draw a marker can take these off with
/// `.class("list-item")`, but the shadcn thing to do is put an icon inside it —
/// see [`TRIGGER_ICON`].
pub const TRIGGER: &str = "cursor-pointer list-none [&::-webkit-details-marker]:hidden";

/// The trigger layer a `disabled` collapsible takes in place of the pointer
/// half of [`TRIGGER`].
///
/// Radix's `disabled` stops the trigger working and says nothing about how it
/// looks, so the look is borrowed from [`Button`](super::button)'s own
/// `disabled:pointer-events-none disabled:opacity-50` rather than invented.
///
/// `pointer-events-none` is doing real work here and not only styling: a
/// `<summary>` has no `disabled` attribute to honour, so blocking the click is
/// how the toggle is actually prevented. The keyboard half is `tabindex="-1"`,
/// written alongside it — without that, `Enter` on a focused summary still
/// opens it.
pub const TRIGGER_DISABLED: &str = "pointer-events-none opacity-50";

/// An icon inside the trigger, turned a half-turn when the collapsible is open.
///
/// Radix publishes `data-state="open"` for this. There is no `data-state` here
/// — the state is the `open` attribute, where the browser keeps it — so the icon
/// reads it off its own `<details>`:
///
/// ```text
/// [[open]>summary_&]:rotate-180
///   [open]     the details, open
///   > summary  its own trigger, and not a nested one's
///   _&         this icon, anywhere inside that trigger
/// ```
///
/// The `>` is the whole point, and it is the same rule
/// [`Resizable`](super::resizable)'s handle is built on: a descendant
/// combinator would turn an inner collapsible's chevron whenever the *outer* one
/// happened to be open. A child combinator asks only about the disclosure the
/// icon actually belongs to.
///
/// ```ignore
/// <CollapsibleTrigger>
///     "Details"
///     <ChevronDown class=@{TRIGGER_ICON} />
/// </CollapsibleTrigger>
/// ```
pub const TRIGGER_ICON: &str =
    "size-4 shrink-0 transition-transform duration-200 [[open]>summary_&]:rotate-180";

/// The height transition, opted into with [`CollapsibleProps::animated`].
///
/// shadcn ships no animation for this component — `tw-animate-css` carries
/// `animate-collapsible-down`, but those keyframes end at
/// `var(--radix-collapsible-content-height, auto)` and there is no Radix here to
/// publish it. So this is the CSS-only equivalent, and it is four declarations
/// on the `<details>` itself rather than on the content:
///
/// - `interpolate-size: allow-keywords` is what lets a `height` transition end
///   at `auto`. It inherits, which is why it belongs on the root.
/// - `::details-content` is the browser's own box around everything after the
///   summary — the element Radix had to fake by keeping hidden content mounted.
///   It goes from `h-0 overflow-hidden` to `h-auto` on `open`.
/// - The `transition` carries `content-visibility` with `allow-discrete`, so the
///   content stays rendered for the length of the close rather than vanishing on
///   the first frame.
///
/// **This is progressive enhancement and nothing is guaranteed.** A browser
/// without `interpolate-size` or `::details-content` ignores every one of these
/// and the collapsible snaps open exactly as it does now — which is the
/// unanimated behaviour, and is why this is a prop rather than a default.
pub const ANIMATED: &str = "[interpolate-size:allow-keywords] \
                            [&::details-content]:h-0 \
                            [&::details-content]:overflow-hidden \
                            [&::details-content]:[transition:height_200ms_ease-out,content-visibility_200ms_ease-out_allow-discrete] \
                            open:[&::details-content]:h-auto";

/// Which of the three a set of props is for.
///
/// A private enum rather than three structs, because the parts differ only in
/// the element they render, the `data-slot` they carry and which of the fields
/// below they read — and not in how `class`, `attrs`, bindings and the scope
/// rule work. Same arrangement as [`Card`](super::card) and
/// [`Resizable`](super::resizable), for the same reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Trigger,
    Content,
}

impl Part {
    /// The `data-slot` this part carries — shadcn's three, unchanged.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "collapsible",
            Self::Trigger => "collapsible-trigger",
            Self::Content => "collapsible-content",
        }
    }

    /// The element this part renders as.
    ///
    /// Not a prop. Radix's parts are all `Primitive.div` and take an `asChild`;
    /// these three are a disclosure, its summary, and the box after it, and
    /// swapping any of them for something else stops it being one.
    const fn element(self) -> &'static str {
        match self {
            Self::Root => "details",
            Self::Trigger => "summary",
            Self::Content => "div",
        }
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part, disabled: bool, animated: bool) -> String {
    match part {
        Part::Root if animated => ANIMATED.to_string(),
        // shadcn's own: nothing at all.
        Part::Root | Part::Content => String::new(),
        Part::Trigger if disabled => cn(&[TRIGGER, TRIGGER_DISABLED]),
        Part::Trigger => TRIGGER.to_string(),
    }
}

/// The class list a [`Collapsible`] renders with, the caller's own included.
///
/// Empty unless `animated`, because shadcn's root has no class list. Reach for
/// it when the element has to belong to the page — but write it on a
/// `<details>`, because everything about this component is that element.
pub fn collapsible_variants(animated: bool, class: &str) -> String {
    cn(&[&part_classes(Part::Root, false, animated), class])
}

/// [`CollapsibleTrigger`]'s class list, the caller's own included.
///
/// `disabled` picks the second layer. Write it on a `<summary>`: `list-none`
/// clears a marker that only a summary has, and `cursor-pointer` corrects a UA
/// default that only a summary carries.
pub fn collapsible_trigger_variants(disabled: bool, class: &str) -> String {
    cn(&[&part_classes(Part::Trigger, disabled, false), class])
}

/// [`CollapsibleContent`]'s class list, the caller's own included.
///
/// The caller's own and nothing else — shadcn's content has no class list, and
/// the hiding is the browser's. Kept for symmetry with the other two, and so a
/// page reaching for it does not have to know that.
pub fn collapsible_content_variants(class: &str) -> String {
    cn(&[class])
}

/// One Collapsible part's props, with shadcn's defaults already in place.
///
/// Build one with [`collapsible`], [`collapsible_trigger`] or
/// [`collapsible_content`] — each picks the part, and everything after it is the
/// same chain. Nothing here is required.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleProps {
    part: Part,
    open: bool,
    name: String,
    animated: bool,
    disabled: bool,
    class: String,
    attrs: Attrs,
}

impl Default for CollapsibleProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            // Radix's `defaultOpen`, and HTML's: closed.
            open: false,
            name: String::new(),
            animated: false,
            disabled: false,
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

/// A default [`Collapsible`]'s props — the start of every chain.
pub fn collapsible() -> CollapsibleProps {
    CollapsibleProps::of(Part::Root)
}

/// A default [`CollapsibleTrigger`]'s props.
pub fn collapsible_trigger() -> CollapsibleProps {
    CollapsibleProps::of(Part::Trigger)
}

/// A default [`CollapsibleContent`]'s props.
pub fn collapsible_content() -> CollapsibleProps {
    CollapsibleProps::of(Part::Content)
}

impl CollapsibleProps {
    fn of(part: Part) -> Self {
        Self {
            part,
            ..Self::default()
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the part's own, `className`-style: a conflicting
    /// utility written here wins.
    ///
    /// This is where a Collapsible gets everything it looks like, because it
    /// starts with nothing — shadcn's three parts have no class lists at all.
    /// The trigger is the usual place: `button_variants(…)` on it is how
    /// shadcn's `asChild` Button demo crosses.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Start open — Radix's `defaultOpen`, and HTML's `open`.
    ///
    /// Written by the server, so an open collapsible is open in the first byte
    /// rather than after a hydration. Bind it and it becomes Radix's controlled
    /// `open` instead; see [`bind`](Self::bind).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Make a set of collapsibles exclusive — HTML's `<details name="…">`.
    ///
    /// Every `<details>` sharing a name is a radio group: opening one closes the
    /// others. There is no Radix equivalent on this component, because Radix
    /// spells it by reaching for a different component entirely — this is the
    /// Accordion behaviour, and here it is an attribute.
    ///
    /// The name is scoped to the document, so two unrelated groups on one page
    /// need two names.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Transition the height on open and close, where the browser can.
    ///
    /// Off by default, because shadcn ships no animation for this component and
    /// because the CSS behind it is recent enough that a page should say it
    /// means it. [`ANIMATED`] spells out what it does and what happens where it
    /// is not supported — which is nothing, and the collapsible snaps.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// A trigger that is drawn and does not toggle — Radix's `disabled`.
    ///
    /// A `<summary>` has no `disabled` attribute, so this is two things at once:
    /// [`TRIGGER_DISABLED`]'s `pointer-events-none` blocks the click, and
    /// `tabindex="-1"` takes the keyboard away. `aria-disabled` says so.
    ///
    /// Radix puts `disabled` on the *root* and the trigger reads it through
    /// context. Here it belongs to the trigger, because the trigger is the only
    /// part that can act on it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The part's own id — for a page's own selector, or an `aria-labelledby`.
    ///
    /// Not needed for `aria-controls`: a `<summary>` is wired to its own
    /// disclosure by the browser, which is one of the things this port stops
    /// having to generate.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server: `attr("title", "Show more")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("open")`.
    ///
    /// `name=""` is how HTML spells a present boolean attribute, and it is what
    /// `html!` itself emits for `open=@{true}`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off, whatever set it: `unset("tabindex")`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("open", "expanded")`.
    ///
    /// `open` is the one worth binding, and it is the one the runtime handles
    /// specially on purpose: a binding whose attribute name is an HTML boolean
    /// word becomes a boolean attribute, present when the expression is truthy
    /// and gone when it is falsy. That is exactly what a controlled `<details>`
    /// wants. It is also the same rewrite that makes an invented `data-open`
    /// silently wrong, so bind `open` and nothing else of that shape here.
    ///
    /// Written on the [`Collapsible`] tag, `open={expanded}` works — a boundary
    /// evaluates a `{…}` attribute in the parent's scope, which is the page's.
    /// An event is the half that does not survive the tag: see [`on`](Self::on),
    /// and reach for [`render`](Self::render) when the page needs to hear about
    /// a change as well as cause one.
    ///
    /// A bound `open` is the literal text `open="{expanded}"` in the server's
    /// markup, and any `open` attribute opens the element — so the parser
    /// always finds this `<details>` open and HTML fires `toggle` at load.
    /// Guard anything passed to [`on`](Self::on) beside this, and seed the
    /// bound state `true` so the disclosure does not flash open on its way to
    /// closed.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("toggle", "setExpanded(target.open)")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings. The event worth listening for here is `toggle` on the
    /// root, which fires after the state has changed and carries it on
    /// `target.open` — Radix calls the same thing `onOpenChange`.
    ///
    /// **This needs [`render`](Self::render), not the tag.** The runtime binds a
    /// native `on*` handler on ordinary elements only. On a component's root —
    /// which is a boundary — the same attribute is read as a value expression
    /// and evaluated during render, where there is no event to read a `target`
    /// off; the console says `Template Expression Failed` and nothing is ever
    /// bound. Interpolate the rendered element into the page's own block
    /// instead, and the handler resolves there with everything else it names.
    ///
    /// Guard a handler that names page state. The attribute is live inline
    /// JavaScript until the runtime takes it off, and on a `<details>` whose
    /// `open` is bound the parser always finds the element open, so HTML fires
    /// `toggle` at load and runs the expression once in global scope:
    /// `typeof setExpanded === 'function' && setExpanded(target.open)`.
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

    /// This part's own utilities, before the caller's.
    fn own_classes(&self) -> String {
        part_classes(self.part, self.disabled, self.animated)
    }

    /// The class list this part renders with, the caller's own included.
    ///
    /// A join, not a merge: no layer here contradicts another, so this is exact
    /// for everything except a caller's `.class(…)` override — which is the one
    /// thing that can conflict, and is resolved in the browser.
    pub fn class_list(&self) -> String {
        cn(&[&self.own_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Bindings run from the client registry compiled by `html!` and retain their caller scope.
    ///
    /// Note what is *not* in this list: `open`. A Collapsible works without a
    /// scope because the browser is the one changing it, and a page's
    /// `open={expanded}` is compiled in the page's own block rather than in one
    /// of this component's — so it needs a script somewhere above, not here.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order: the component's own first,
    /// the caller's spread last so it wins.
    ///
    /// Handed to `html!` as `@{..attrs}`, which is where names are validated and
    /// values escaped.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());

        base = match self.part {
            Part::Root => {
                let mut root = base;
                if self.open {
                    root = root.set("open", "");
                }
                if !self.name.trim().is_empty() {
                    root = root.set("name", self.name.trim());
                }
                root
            }

            // A `<summary>` has no `disabled` attribute, so the two halves of
            // stopping it are written by hand: the pointer half is a utility in
            // `TRIGGER_DISABLED`, and this is the keyboard half.
            Part::Trigger if self.disabled => {
                base.set("aria-disabled", "true").set("tabindex", "-1")
            }

            Part::Trigger | Part::Content => base,
        };

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the part is drawn on arrival.
        //
        // The empty case is real here in a way it is nowhere else in this
        // library: shadcn's root and content have no class list at all, so a
        // `<Collapsible>` with nothing written on it would otherwise render a
        // `class=""` that says nothing.
        let own = self.own_classes();
        base = if self.class.trim().is_empty() {
            match own.is_empty() {
                true => base,
                false => base.set("class", own),
            }
        } else if own.is_empty() {
            base.set("class", self.class.trim())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&own, &self.class]),
            )
        };

        // The caller's own, last, so a `{...props}` spread wins — the same
        // last-wins rule `AttrList` applies when it writes the tag.
        merge(base, &self.attrs)
    }

    /// The rendered element, with no boundary around it and no `<script>` in it.
    ///
    /// Every part of this component renders this way — there is no script
    /// anywhere in the file — so this is the same markup the tags produce, and
    /// it is the way in for a page building a list of disclosures with
    /// `Html::concat`.
    ///
    /// The absence of a script is what makes a page's own `open={…}` work
    /// through a tag: a block with a script is mounted as its own scope, and a
    /// binding written on this element would then be compiled there instead of
    /// in the page. See [`Collapsible`].
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();

        match self.part.element() {
            "details" => html! {
                <details @{..attrs}>@{children}</details>
            },
            "summary" => html! {
                <summary @{..attrs}>@{children}</summary>
            },
            _ => html! {
                <div @{..attrs}>@{children}</div>
            },
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// A disclosure: a trigger, and a region it shows and hides.
///
/// ```ignore
/// <Collapsible>
///     <CollapsibleTrigger>"Can I use this in my project?"</CollapsibleTrigger>
///     <CollapsibleContent>
///         "Yes. Free to use for personal and commercial projects."
///     </CollapsibleContent>
/// </Collapsible>
/// ```
///
/// This is a `<details>`, and the trigger is its `<summary>`. It opens and
/// closes with no JavaScript at all — the only component in this library other
/// than [`Slider`](super::slider) that is fully operable with the runtime
/// absent, and the reason the module docs argue for the element at length.
///
/// The summary has to be the first thing inside. Anything written between the
/// trigger and the content is *part of* the content, because that is what
/// `<details>` means; shadcn's demo with an always-visible row after the trigger
/// is the one call site that has to be rearranged rather than translated.
///
/// ## Open, and staying open
///
/// `open` alone is Radix's `defaultOpen` — the server writes the attribute and
/// the browser owns it from then on. `open={…}` is Radix's controlled `open`,
/// and it works through the tag because a boundary evaluates a `{…}` attribute
/// in the *calling page's* scope:
///
/// ```ignore
/// <Collapsible open={expanded}>
/// ```
///
/// That is one way only. `ontoggle={…}` is not an event handler on a boundary —
/// it is read as a value expression, evaluated during render, and fails on the
/// `target` that is not there. A page that has to hear about the change as well
/// as cause it wants [`CollapsibleProps::render`] interpolated into its own
/// block, which is what the module docs and the demo page both work through.
///
/// `name` makes a set of them exclusive, which is `<details name="…">` and needs
/// nothing else.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Collapsible(
    class: &str,
    open: bool,
    name: &str,
    animated: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = collapsible()
        .class(class)
        .open(open)
        .name(name)
        .animated(animated);

    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all.
    if !id.is_empty() {
        props = props.id(id);
    }

    props = props.attrs(attrs);

    // `open={…}` lands here. Applied last, so a binding replaces a server value
    // of the same name rather than sitting beside it — which is what lets
    // `open={expanded}` win over a written `open`.
    //
    // An `ontoggle={…}` lands here too and will not work: this element is a
    // boundary, and a boundary reads every `{…}` attribute as a value rather
    // than binding a native handler. `CollapsibleProps::on` says so, and
    // `render` is the way round it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    let attrs = props.attr_list();

    // No `<script>`, ever, and that is load-bearing rather than an omission.
    // Every other component in this library adds an empty script when a binding
    // is present, so that the block mounts and the binding compiles. Here the
    // bindings that matter belong to the *page* — a script would raise a scope
    // between them and the state they name, and `open={expanded}` would start
    // looking for `expanded` in here, where there is none.
    html! {
        <details @{..attrs}>
            <slot />
        </details>
    }
}

/// The thing that is clicked — a `<summary>`.
///
/// ```ignore
/// <CollapsibleTrigger>"Show more"</CollapsibleTrigger>
///
/// <CollapsibleTrigger class="flex items-center justify-between gap-4">
///     <h4 class="text-sm font-semibold">"Starred repositories"</h4>
///     <ChevronsUpDown class=@{TRIGGER_ICON} />
/// </CollapsibleTrigger>
/// ```
///
/// It has to be the first thing inside its [`Collapsible`], and the browser
/// wires it to the disclosure itself — there is no `aria-expanded` and no
/// `aria-controls` to write, and no id to generate for them to point at.
///
/// shadcn's `asChild` Button trigger crosses as a class list rather than as a
/// nested element, because a `<button>` inside a `<summary>` is two tab stops
/// where the user sees one:
///
/// ```ignore
/// <CollapsibleTrigger class=@{button_variants(ButtonVariant::Ghost, ButtonSize::Icon, "size-8")}>
///     <ChevronsUpDown />
///     <span class="sr-only">"Toggle"</span>
/// </CollapsibleTrigger>
/// ```
///
/// `disabled` is Radix's, moved from the root to the part that can act on it: it
/// blocks the pointer with a utility and the keyboard with `tabindex="-1"`,
/// because a `<summary>` has no `disabled` attribute of its own.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CollapsibleTrigger(
    class: &str,
    disabled: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = collapsible_trigger().class(class).disabled(disabled);

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

    // No script here either, and for the same reason as the root: `toggle`
    // fires on the `<details>`, so a page listening for it writes the binding
    // there, and a scope raised around the summary would be one more boundary
    // between that expression and the page it belongs to.
    html! {
        <summary @{..attrs}>
            <slot />
        </summary>
    }
}

/// The region that is shown and hidden.
///
/// ```ignore
/// <CollapsibleContent class="flex flex-col gap-2">…</CollapsibleContent>
/// ```
///
/// A plain `<div>` with a `data-slot` and no class list of its own, which is
/// exactly what shadcn's is. The hiding is not written anywhere: it is what
/// `<details>` does to everything after its summary, which is also why this part
/// carries no `hidden`, no `data-state` and no height custom property.
///
/// It does not have to be the only thing after the trigger — two of these, or a
/// `<div>` written by hand, are all disclosed together. It is a name for the
/// region rather than a mechanism.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CollapsibleContent(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = collapsible_content().class(class);

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
        <div @{..attrs}>
            <slot />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ANIMATED, Attrs, Collapsible, CollapsibleContent, CollapsibleProps, CollapsibleTrigger,
        TRIGGER, TRIGGER_DISABLED, collapsible, collapsible_content, collapsible_content_variants,
        collapsible_trigger, collapsible_trigger_variants, collapsible_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    fn root(class: &str, open: bool, name: &str, animated: bool) -> String {
        Collapsible(
            class,
            open,
            name,
            animated,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("inside"),
        )
        .into_test_string()
    }

    fn trigger(class: &str, disabled: bool) -> String {
        CollapsibleTrigger(
            class,
            disabled,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Toggle"),
        )
        .into_test_string()
    }

    /// The whole component: a native disclosure, and nothing else.
    #[test]
    fn a_default_collapsible_is_a_details() {
        let markup = root("", false, "", false);

        assert!(markup.starts_with("<details "), "{markup}");
        assert!(markup.contains(r#"data-slot="collapsible""#), "{markup}");
        assert!(markup.contains("inside"), "{markup}");
        assert!(markup.ends_with("</details>"), "{markup}");
    }

    /// The point of the port. A script would mount a scope, and a scope would
    /// swallow the page's `open={…}`.
    #[test]
    fn no_part_ever_ships_a_script() {
        let parts = [
            root("flex flex-col gap-2", true, "faq", true),
            trigger("font-semibold", true),
            CollapsibleContent(
                "px-4",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::default(),
            )
            .into_test_string(),
        ];

        for markup in parts {
            assert!(!markup.contains("<script"), "{markup}");
        }
    }

    #[test]
    fn a_trigger_is_a_summary_with_its_marker_taken_off() {
        let markup = trigger("", false);

        assert!(markup.starts_with("<summary "), "{markup}");
        assert!(
            markup.contains(r#"data-slot="collapsible-trigger""#),
            "{markup}"
        );
        assert!(markup.contains("list-none"), "{markup}");
        assert!(markup.contains("cursor-pointer"), "{markup}");
    }

    #[test]
    fn the_content_is_a_div_and_carries_nothing_of_its_own() {
        let markup = CollapsibleContent(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("body"),
        )
        .into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(
            markup.contains(r#"data-slot="collapsible-content""#),
            "{markup}"
        );
        assert!(markup.contains("body"), "{markup}");
        // shadcn's content has no class list, so neither does this — and an
        // empty `class=""` would be an attribute saying nothing.
        assert!(!markup.contains("class="), "{markup}");
    }

    /// `open` is HTML's own boolean attribute, written by the server — so an
    /// open collapsible is open in the first byte.
    #[test]
    fn open_is_the_attribute_and_not_a_state_word() {
        let closed = root("", false, "", false);
        let opened = root("", true, "", false);

        assert!(!closed.contains("open"), "{closed}");
        assert!(opened.contains(r#"open="""#), "{opened}");

        // Radix writes `data-state`; there is nothing here to keep it true.
        assert!(!opened.contains("data-state"), "{opened}");
    }

    #[test]
    fn a_name_makes_a_set_of_them_exclusive() {
        let markup = root("", false, "faq", false);

        assert!(markup.contains(r#"name="faq""#), "{markup}");
        assert!(!root("", false, "", false).contains("name="), "{markup}");
    }

    #[test]
    fn animation_is_opted_into_and_lands_on_the_root() {
        let plain = root("", false, "", false);
        let animated = root("", false, "", true);

        assert!(!plain.contains("details-content"), "{plain}");
        assert!(animated.contains("interpolate-size"), "{animated}");
        assert!(animated.contains("details-content"), "{animated}");
    }

    /// A `<summary>` has no `disabled` attribute, so both halves are written.
    #[test]
    fn a_disabled_trigger_loses_the_pointer_and_the_keyboard() {
        let markup = trigger("", true);

        assert!(markup.contains("pointer-events-none"), "{markup}");
        assert!(markup.contains(r#"tabindex="-1""#), "{markup}");
        assert!(markup.contains(r#"aria-disabled="true""#), "{markup}");
    }

    /// The one binding this component is built around. A boundary evaluates a
    /// `{…}` attribute in the parent's scope, so this reaches the page's state.
    #[test]
    fn open_can_be_bound_through_the_tag() {
        let markup = Collapsible(
            "",
            false,
            "",
            false,
            "",
            crate::rahti::Bindings::__from_html(&[("open", "expanded")]),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"open="{expanded}""#), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
    }

    /// The other half does not survive the tag, and the way round it is
    /// `render` — the same route the Switch's page takes.
    ///
    /// A native `on*` handler is only bound on an ordinary element. On a
    /// component's root, which is a boundary, the runtime reads the same
    /// attribute as a value expression and evaluates it during render, where
    /// there is no `target` — measured in the browser, not assumed.
    #[test]
    fn a_toggle_handler_wants_render_rather_than_the_tag() {
        let rendered = collapsible()
            .bind(
                "open",
                crate::components::rahti_ui::attrs::ui_expr!(open = { expanded }),
            )
            .on(
                "toggle",
                crate::components::rahti_ui::attrs::ui_handler!({ setExpanded(target.open) }),
            )
            .render(Html::default())
            .into_test_string();

        assert!(rendered.starts_with("<details "), "{rendered}");
        assert!(rendered.contains(r#"open="{expanded}""#), "{rendered}");
        assert!(
            rendered.contains(r#"pp-ontoggle="setExpanded(target.open)""#),
            "{rendered}"
        );
        // The whole point: no boundary, so this lands in the block it is
        // interpolated into.
        assert!(!rendered.contains("pp-component"), "{rendered}");
        assert!(!rendered.contains("<script"), "{rendered}");
    }

    /// A bound `open` renders as `open="{expanded}"`, which is an *open*
    /// `<details>` to the parser whatever the expression says — and HTML fires
    /// `toggle` at load for a disclosure it found open. `ontoggle` is still a
    /// plain inline handler at that point, run in global scope, so a handler
    /// that names page state has to survive being called before there is any.
    ///
    /// This is the shape the docs teach and the `/components/collapsible` demo
    /// uses. `typeof` does not throw on an undeclared name, and `&&`
    /// short-circuits before `target` is read.
    #[test]
    fn a_bound_open_wants_its_toggle_handler_guarded() {
        let expr = crate::components::rahti_ui::attrs::ui_handler!({typeof setExpanded === "function" && setExpanded(target.open)});
        let rendered = collapsible()
            .bind(
                "open",
                crate::components::rahti_ui::attrs::ui_expr!(open = { expanded }),
            )
            .on("toggle", expr)
            .render(Html::default())
            .into_test_string();

        // The guard reaches the attribute intact, quotes and all.
        assert!(
            rendered.contains("typeof setExpanded === &quot;function&quot;"),
            "{rendered}"
        );
        // And the element really is open to the parser, which is the reason
        // the guard is there.
        assert!(rendered.contains(r#"open="{expanded}""#), "{rendered}");
    }

    /// A binding wins over the server's own attribute of the same name, rather
    /// than being written beside it.
    #[test]
    fn a_bound_open_replaces_a_written_one() {
        let markup = Collapsible(
            "",
            true,
            "",
            false,
            "",
            crate::rahti::Bindings::__from_html(&[("open", "expanded")]),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"open="{expanded}""#), "{markup}");
        assert!(!markup.contains(r#"open="""#), "{markup}");
    }

    /// shadcn's `asChild` Button trigger, as a class list. The override is the
    /// one thing that can conflict, so it becomes a merge the browser resolves.
    #[test]
    fn an_override_on_the_trigger_becomes_a_merge() {
        let markup = trigger("inline-flex size-8 items-center", false);

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("size-8"), "{markup}");
    }

    /// The root has no class list of its own, so an override there is the whole
    /// answer and needs no merge at all.
    #[test]
    fn an_override_on_the_root_needs_no_merge() {
        let markup = root("flex w-[350px] flex-col gap-2", false, "", false);

        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(
            markup.contains(r#"class="flex w-[350px] flex-col gap-2""#),
            "{markup}"
        );
    }

    /// …unless it is animated, which is the one case the root owns utilities.
    #[test]
    fn an_override_on_an_animated_root_does_need_one() {
        let markup = root("flex flex-col gap-2", false, "", true);

        assert!(!markup.contains("{twMerge("), "{markup}");
    }

    /// `{...props}` is the last word, as it is in shadcn.
    #[test]
    fn the_callers_attributes_win() {
        let markup = CollapsibleTrigger(
            "",
            true,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new().set("aria-disabled", "false").unset("tabindex"),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"aria-disabled="false""#), "{markup}");
        assert!(!markup.contains("tabindex"), "{markup}");
    }

    #[test]
    fn the_variants_helpers_are_the_lists_the_components_render() {
        assert_eq!(collapsible_variants(false, ""), "");
        assert_eq!(collapsible_variants(false, "gap-2"), "gap-2");
        assert_eq!(collapsible_variants(true, ""), ANIMATED);
        assert_eq!(collapsible_trigger_variants(false, ""), TRIGGER);
        assert!(collapsible_trigger_variants(true, "").contains(TRIGGER_DISABLED));
        assert_eq!(collapsible_content_variants("px-4"), "px-4");
    }

    /// Every part renders the same outside a tag as inside one, because none of
    /// them has a script to leave out.
    #[test]
    fn the_parts_render_outside_a_tag() {
        let one = collapsible()
            .open(true)
            .name("faq")
            .render(Html::escape("body"))
            .into_test_string();

        assert!(one.starts_with("<details "), "{one}");
        assert!(one.contains(r#"open="""#), "{one}");

        let two = collapsible_trigger()
            .render(Html::escape("Toggle"))
            .into_test_string();
        assert!(two.starts_with("<summary "), "{two}");

        let three = collapsible_content()
            .render(Html::default())
            .into_test_string();
        assert!(three.starts_with("<div "), "{three}");
    }

    /// A scope is needed for an override or a binding, and for nothing else —
    /// but note that nothing in this file ever acts on the answer. It is here
    /// because a page reaching for `render` may need to know.
    #[test]
    fn a_scope_is_only_ever_wanted_by_the_caller() {
        assert!(!collapsible().needs_scope());
        assert!(!collapsible().open(true).name("faq").needs_scope());
        assert!(!collapsible().class("gap-2").needs_scope());
        assert!(
            collapsible()
                .bind(
                    "open",
                    crate::components::rahti_ui::attrs::ui_expr!(open = { expanded })
                )
                .needs_scope()
        );
    }

    /// Every prop left unwritten, which is what `<Collapsible />` compiles to.
    #[test]
    fn the_unwritten_props_are_shadcns_defaults() {
        let props = CollapsibleProps::default();

        assert!(!props.open);
        assert!(!props.animated);
        assert!(!props.disabled);
        assert!(props.name.is_empty());
    }
}

#[cfg(test)]
mod audit {
    use super::{ANIMATED, TRIGGER, TRIGGER_DISABLED, TRIGGER_ICON};
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered. The two trigger layers
    /// are checked as the pair they are actually rendered in.
    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("TRIGGER", TRIGGER.to_string()),
            (
                "TRIGGER + TRIGGER_DISABLED",
                cn(&[TRIGGER, TRIGGER_DISABLED]),
            ),
            ("TRIGGER_ICON", TRIGGER_ICON.to_string()),
            ("ANIMATED", ANIMATED.to_string()),
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

    /// There is no `data-state` in this component, and there must not be: the
    /// state is the `open` attribute, and nothing here would keep a copy true.
    #[test]
    fn nothing_reads_a_state_word() {
        for list in [TRIGGER, TRIGGER_DISABLED, TRIGGER_ICON, ANIMATED] {
            assert!(!list.contains("data-state"), "{list}");
        }
    }

    /// The icon turns for its own disclosure and not for an ancestor's — the
    /// same child-combinator rule the Resizable's handle is built on.
    #[test]
    fn the_icon_reads_its_own_details_and_not_an_ancestors() {
        assert!(
            TRIGGER_ICON.contains("[[open]>summary_&]:rotate-180"),
            "{TRIGGER_ICON}"
        );
        assert!(!TRIGGER_ICON.contains("[[open]_"), "{TRIGGER_ICON}");
    }
}
