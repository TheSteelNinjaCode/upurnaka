// rahti-ui: card
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.3.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Card — a port of shadcn/ui's `card`.
//!
//! Seven parts, no variants, no state, and not one line of JavaScript: a Card
//! is a bordered surface and six regions inside it, and everything it does it
//! does with CSS. That makes it the smallest component in this library by
//! behaviour and one of the fussiest by layout, because the two parts that
//! matter — [`CardHeader`] and [`CardAction`] — talk to each other through a
//! grid neither of them fully owns.
//!
//! ## Which shadcn card this is
//!
//! shadcn ships the card twice. Its Base UI style writes the design into
//! `cn-card`, `cn-card-header`, `cn-card-title` … marker classes that an
//! external stylesheet defines, so copying it here would copy names with no
//! rules behind them. The published registry
//! (`https://ui.shadcn.com/r/styles/new-york-v4/card.json`) spells the same
//! card in utilities, and that is what this file copies — the same choice
//! [`ButtonGroup`](super::button_group) and [`Field`](super::field) made.
//!
//! One thing is lost with the marker classes, and it is worth naming rather
//! than pretending away: the Base UI card takes `size="default" | "sm"`, which
//! is a `data-size` attribute the `cn-card` stylesheet reads to tighten every
//! gap and every padding at once. There is no utility spelling of it in the
//! registry style, so there is none here. A tighter card is
//! `.class("gap-4 py-4")` on the card and `.class("px-4")` on its parts —
//! written out, because that is what the size prop was hiding.
//!
//! ## The header is a grid, and the action is placed in it
//!
//! This is the whole of the component's cleverness, and the only way to get it
//! wrong:
//!
//! ```text
//! CardHeader   grid auto-rows-min grid-rows-[auto_auto] items-start gap-2
//!              has-data-[slot=card-action]:grid-cols-[1fr_auto]
//! CardAction   col-start-2 row-span-2 row-start-1 self-start justify-self-end
//! ```
//!
//! The header is one column until it *contains* something carrying
//! `data-slot="card-action"`, at which point it becomes two — and the action
//! puts itself in the second one, spanning the title's row and the
//! description's. Tailwind compiles the trigger to `:has([data-slot=…])`, so
//! the action only has to be a descendant for the column to appear.
//!
//! Being *placed* in that column is the stricter half: `col-start-2` is a grid
//! property, and it only applies to a grid item. A [`CardAction`] therefore has
//! to be a direct child of the header — which it is, including when it carries
//! a binding, because every part in this file keeps its `<script>` *inside* its
//! own element rather than beside it.
//!
//! That is the opposite of what [`ButtonGroup`](super::button_group) had to do,
//! and the reason is worth writing down. A group welds its children with
//! `:first-child` / `:last-child`, and those count every element, `display:
//! none` or not — so a `<script>` in there would be counted. A card counts
//! nothing. A `<script>` is `display: none`, so it is neither a flex item of
//! the card nor a grid item of the header, and putting it inside costs the
//! layout nothing while keeping every part a real child of its parent.
//!
//! The other consequence of the same rule: when a page renders its own action
//! element, `data-slot="card-action"` is what makes the header find it.
//! `unset("data-slot")` on a [`CardAction`] gives the second column up.
//!
//! ## The two border variants are about the element itself
//!
//! [`HEADER`] ends in `[.border-b]:pb-6` and [`FOOTER`] in `[.border-t]:pt-6`.
//! Neither is a descendant or a parent selector — Tailwind compiles them to
//! `:is(.border-b)` and `:is(.border-t)` on the element carrying them, so they
//! fire when *this* header or *this* footer is given the border and never
//! otherwise:
//!
//! ```ignore
//! <CardHeader class="border-b">…</CardHeader>   // the rule, and its padding
//! <CardFooter class="border-t">…</CardFooter>
//! ```
//!
//! Rust composes class layers with `utils::cn`. Use frontend `twMerge`
//! bindings when utilities conflict.
//!
//! ## Twenty-four pixels of asymmetry, on purpose
//!
//! The card pads *vertically* (`py-6`) and every region pads *horizontally*
//! (`px-6`). Nothing pads both. That is what lets a region bleed to the card's
//! edge by dropping one utility — an image or a table in a [`CardContent`]
//! with `.class("px-0")` runs the full width, and the card's own top and bottom
//! padding is untouched. It is also why a bare `<div>` written between two card
//! parts is unpadded and looks broken: the horizontal padding was never on the
//! card.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Card class="w-full max-w-sm">
//!     <CardHeader>
//!         <CardTitle>"Sign in"</CardTitle>
//!         <CardDescription>"Use the email you signed up with."</CardDescription>
//!         <CardAction><Button variant="link">"Sign up"</Button></CardAction>
//!     </CardHeader>
//!     <CardContent>
//!         @{field}
//!     </CardContent>
//!     <CardFooter class="flex-col gap-2">
//!         <Button class="w-full">"Sign in"</Button>
//!     </CardFooter>
//! </Card>
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is that part's default.
//!
//! `element` is shadcn's `asChild` without the child, and each part has its own
//! short list of tags it will answer to. It earns its place here more than in
//! most components: shadcn's card is seven `<div>`s, and a card is very often
//! an `<article>` with a real heading in it. `<CardTitle element="h3">` is a
//! heading to a screen reader and the same `leading-none font-semibold` to
//! everyone else.
//!
//! ## What crosses the boundary, and what does not
//!
//! Same rule as everywhere else in this library. `onclick={…}` on a tag lands
//! on the element untouched and resolves when it fires, late enough to reach
//! the calling page's state. So does an attribute *binding* —
//! `hidden={collapsed}` — because a component's root attributes are evaluated
//! where the tag was written.
//!
//! Children follow the same rule, which matters here because a card is mostly
//! children. A `{count}` written between `<CardTitle>` tags is compiled in the
//! block it was *written* in — the page's, which is what you want. What does
//! not work is a page reaching *into* a part it did not render. When the card's
//! content is the page's state, build the parts with the `card*()` props and
//! render them into the page's own block; every `render` in this file carries
//! no `<script>` precisely so the whole card stays one scope.
//!
//! ## When the tag is not enough
//!
//! **[`CardProps::render`], interpolated — for a card a page assembles in
//! Rust.**
//!
//! ```ignore
//! @{card().bind("data-state", crate::components::rahti_ui::attrs::ui_expr!(data-state={picked === 1 ? "picked" : "idle"})).render(Html::concat([
//!     card_header().render(card_title().render(Html::escape("Plan"))),
//!     card_content().render(Html::from_raw("{price}".to_string())),
//! ]))}
//! ```
//!
//! **[`card_variants`] and its six siblings — the class list alone**, for an
//! element the page has to own outright.
//!
//! ```ignore
//! <div class=@{card_variants("w-72")} data-slot="card">…</div>
//! ```
//!
//! Write the `data-slot` alongside it whenever the part is a [`CardAction`] —
//! that attribute is not decoration there, it is the selector the header's
//! second column depends on.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// The surface itself.
///
/// `py-6` and no `px-` is deliberate — see the module docs. `bg-card` and
/// `text-card-foreground` are the only place in this file that names a colour,
/// which is why a card reads correctly in dark mode without a `dark:`
/// anywhere: the two variables are redefined, not the utilities.
///
/// `gap-6` is what separates header, content and footer, so a region left out
/// costs nothing — there is no margin anywhere to collapse or to cancel.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const CARD: &str = "flex flex-col gap-6 rounded-xl border bg-card py-6 \
                    text-card-foreground shadow-sm";

/// The header, which is a grid rather than a stack.
///
/// Two rows always (`grid-rows-[auto_auto]`, sized to content by
/// `auto-rows-min`), and a second column only when there is a [`CardAction`]
/// inside to fill it. `items-start` keeps the title from stretching down into
/// the description's row when there is no description.
///
/// `@container/card-header` names the header as a container, so a caller can
/// write `@md/card-header:…` on anything inside it. shadcn's own cards do not
/// use it; it costs nothing to leave available.
///
/// `[.border-b]:pb-6` fires on the header itself, and only when the header is
/// given `border-b`. See the module docs.
const HEADER: &str = "@container/card-header grid auto-rows-min grid-rows-[auto_auto] \
                      items-start gap-2 px-6 \
                      has-data-[slot=card-action]:grid-cols-[1fr_auto] \
                      [.border-b]:pb-6";

/// The title. Two utilities, and the interesting one is `leading-none`: a card
/// title sits directly above a description with `gap-2` between them, and a
/// line box taller than the text would make that gap read as bigger than the
/// one it is measured against.
const TITLE: &str = "leading-none font-semibold";

/// The description — one step down in size, one step back in colour. Also what
/// the header's second row is for, which is why an action spans two rows
/// whether or not a description is there to fill the lower one.
const DESCRIPTION: &str = "text-sm text-muted-foreground";

/// Where an action is placed in the header's grid.
///
/// Every utility here is a grid property, so all of them need the action to be
/// a grid item — a direct child of the header. `justify-self-end` is what pins
/// it to the right edge of a column that is only as wide as it is.
const ACTION: &str = "col-start-2 row-span-2 row-start-1 self-start justify-self-end";

/// The body. One utility, and the module docs explain why it is a horizontal
/// one: `.class("px-0")` is how content bleeds to the card's edge.
const CONTENT: &str = "px-6";
/// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
///
/// `[.border-t]:pt-6` is [`HEADER`]'s rule, turned upside down.
const FOOTER: &str = "flex items-center px-6 [.border-t]:pt-6";

/// The tags a [`Card`] will answer to.
///
/// `article` is the one worth reaching for: a card that is a self-contained
/// piece of content — a post, a product, a plan — is an `<article>`, and a
/// screen reader can then be told there are six of them. `li` is for a card
/// that is an item of a real list, `section` for one that is a labelled region
/// of the page.
const CARD_ELEMENTS: [&str; 4] = ["div", "article", "section", "li"];

/// A [`CardHeader`]'s tags. `header` is only correct inside a sectioning
/// element, which is to say inside a Card rendered as an `article` or a
/// `section` — in a plain `<div>` card it would be the *page's* header.
const HEADER_ELEMENTS: [&str; 2] = ["div", "header"];

/// A [`CardTitle`]'s tags.
///
/// shadcn renders a `<div>` and leaves the heading level to the application,
/// which is the right default and the wrong outcome if nobody ever changes it:
/// a card title is almost always a heading, and a `<div>` is not one at any
/// level. Pick the level from the page's outline — `h2` under the page's `h1`,
/// `h3` under a section's `h2`.
const TITLE_ELEMENTS: [&str; 6] = ["div", "h1", "h2", "h3", "h4", "span"];

/// A [`CardDescription`]'s tags. `p` for a description that is a sentence,
/// which is most of them.
const DESCRIPTION_ELEMENTS: [&str; 3] = ["div", "p", "span"];

/// A [`CardAction`]'s tags. Both are neutral boxes — the action itself is a
/// [`Button`](super::button) inside, not this element.
const ACTION_ELEMENTS: [&str; 2] = ["div", "span"];

/// A [`CardContent`]'s tags.
const CONTENT_ELEMENTS: [&str; 2] = ["div", "section"];

/// A [`CardFooter`]'s tags. `footer` under the same condition `header` is —
/// inside a sectioning element, or it belongs to the page.
const FOOTER_ELEMENTS: [&str; 2] = ["div", "footer"];

/// The element every part renders as unless told otherwise. shadcn's is the
/// same one seven times.
const DEFAULT_ELEMENT: &str = "div";

/// Which of the seven a set of props is for.
///
/// A private enum rather than seven structs, because the parts differ only in
/// the `data-slot` they carry, the class list they own, and the tags they will
/// answer to — and not in how `class`, `element`, `attrs`, bindings and the
/// scope rule work. Same arrangement as [`Field`](super::field),
/// [`InputGroup`](super::input_group) and [`ButtonGroup`](super::button_group),
/// for the same reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Card,
    Header,
    Title,
    Description,
    Action,
    Content,
    Footer,
}

impl Part {
    /// The `data-slot` this part carries.
    ///
    /// [`Action`](Self::Action)'s is load-bearing:
    /// `has-data-[slot=card-action]:grid-cols-[1fr_auto]` on the header is how
    /// the second column comes to exist. The other six are for a page's own
    /// selectors, and for anyone reading the DOM.
    const fn slot(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Header => "card-header",
            Self::Title => "card-title",
            Self::Description => "card-description",
            Self::Action => "card-action",
            Self::Content => "card-content",
            Self::Footer => "card-footer",
        }
    }

    /// This part's own utilities, before the caller's.
    const fn classes(self) -> &'static str {
        match self {
            Self::Card => CARD,
            Self::Header => HEADER,
            Self::Title => TITLE,
            Self::Description => DESCRIPTION,
            Self::Action => ACTION,
            Self::Content => CONTENT,
            Self::Footer => FOOTER,
        }
    }

    /// The tags this part is allowed to render as.
    ///
    /// Closed rather than open because `html!` writes a tag name literally: the
    /// element is chosen by a `match` in [`CardProps::render`], and this is the
    /// same list read back, so an unknown tag is refused where it is set rather
    /// than silently falling through to `<div>` at render time.
    const fn elements(self) -> &'static [&'static str] {
        match self {
            Self::Card => &CARD_ELEMENTS,
            Self::Header => &HEADER_ELEMENTS,
            Self::Title => &TITLE_ELEMENTS,
            Self::Description => &DESCRIPTION_ELEMENTS,
            Self::Action => &ACTION_ELEMENTS,
            Self::Content => &CONTENT_ELEMENTS,
            Self::Footer => &FOOTER_ELEMENTS,
        }
    }
}

/// The class list shadcn's `cn("flex flex-col …", className)` returns for the
/// card itself.
///
/// Reach for this — and for the six below — when the element has to belong to
/// the page rather than to a component: a card whose content is the page's
/// state, or one whose class list a page computes. Write `data-slot="card"`
/// alongside it.
pub fn card_variants(class: &str) -> String {
    cn(&[CARD, class])
}

/// [`CardHeader`]'s class list, the caller's own included.
///
/// Write `data-slot="card-header"` alongside it. Nothing selects on it today —
/// the header finds its action, not the other way round — but a page's own
/// selectors will, and it is what every other part in a card carries.
pub fn card_header_variants(class: &str) -> String {
    cn(&[HEADER, class])
}

/// [`CardTitle`]'s class list, the caller's own included.
pub fn card_title_variants(class: &str) -> String {
    cn(&[TITLE, class])
}

/// [`CardDescription`]'s class list, the caller's own included.
pub fn card_description_variants(class: &str) -> String {
    cn(&[DESCRIPTION, class])
}

/// [`CardAction`]'s class list, the caller's own included.
///
/// `data-slot="card-action"` is not optional on the element this goes on: the
/// header grows its second column with `:has([data-slot=card-action])`, and
/// without the attribute these utilities place the action in a column that was
/// never created.
pub fn card_action_variants(class: &str) -> String {
    cn(&[ACTION, class])
}

/// [`CardContent`]'s class list, the caller's own included.
pub fn card_content_variants(class: &str) -> String {
    cn(&[CONTENT, class])
}

/// [`CardFooter`]'s class list, the caller's own included.
pub fn card_footer_variants(class: &str) -> String {
    cn(&[FOOTER, class])
}

/// One Card part's props, with shadcn's defaults already in place.
///
/// Build one with [`card`], [`card_header`], [`card_title`],
/// [`card_description`], [`card_action`], [`card_content`] or [`card_footer`] —
/// each picks the part, and everything after it is the same chain. Nothing here
/// is required.
///
/// Only `class` and `element` are fields of their own. `class` is the one thing
/// that can conflict with what the component chose, and resolving that is the
/// browser's job; `element` decides which tag is written and so cannot be an
/// attribute. Everything else a part carries — `id`, `aria-labelledby`,
/// `title` — is an ordinary attribute and lives in `attrs` with the caller's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardProps {
    part: Part,
    class: String,
    element: &'static str,
    attrs: Attrs,
}

impl Default for CardProps {
    fn default() -> Self {
        Self {
            part: Part::Card,
            class: String::new(),
            element: DEFAULT_ELEMENT,
            attrs: Attrs::new(),
        }
    }
}

/// A default [`Card`]'s props — the start of every chain.
pub fn card() -> CardProps {
    CardProps::of(Part::Card)
}

/// A default [`CardHeader`]'s props.
pub fn card_header() -> CardProps {
    CardProps::of(Part::Header)
}

/// A default [`CardTitle`]'s props.
pub fn card_title() -> CardProps {
    CardProps::of(Part::Title)
}

/// A default [`CardDescription`]'s props.
pub fn card_description() -> CardProps {
    CardProps::of(Part::Description)
}

/// A default [`CardAction`]'s props.
pub fn card_action() -> CardProps {
    CardProps::of(Part::Action)
}

/// A default [`CardContent`]'s props.
pub fn card_content() -> CardProps {
    CardProps::of(Part::Content)
}

/// A default [`CardFooter`]'s props.
pub fn card_footer() -> CardProps {
    CardProps::of(Part::Footer)
}

impl CardProps {
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
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Render as something other than `<div>` — shadcn's `asChild`, without the
    /// child. A tag outside this part's own list is ignored.
    ///
    /// Each part has its own list, because the sets do not overlap: a Card may
    /// be an `<article>` and a title may be an `<h3>`, and neither is a sensible
    /// answer for the other. See [`CARD_ELEMENTS`] and its siblings.
    pub fn element(mut self, tag: &'static str) -> Self {
        debug_assert!(
            self.part.elements().contains(&tag),
            "rahti-ui: `{tag}` is not a tag a {} renders as. One of: {}.",
            self.part.slot(),
            self.part.elements().join(", "),
        );
        if self.part.elements().contains(&tag) {
            self.element = tag;
        }
        self
    }

    /// The part's id — what an `aria-labelledby` elsewhere points at, and what
    /// a Card points at to be named by its own title.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// `aria-labelledby="…"` — the id of the [`CardTitle`] that names this
    /// card.
    ///
    /// Worth writing on a Card rendered as an `<article>` or a `<section>`:
    /// both are announced as landmarks-of-a-kind, and one with no accessible
    /// name is announced as a region nobody is told the purpose of. Give the
    /// title an [`id`](Self::id) and point at it.
    pub fn labelled_by(self, id: &str) -> Self {
        self.attr("aria-labelledby", id)
    }

    /// Any attribute, rendered on the server: `attr("title", "Draft")`.
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
    ///
    /// The one worth knowing about is a [`CardAction`]'s: without
    /// `data-slot="card-action"` the header never grows the column the action
    /// places itself in, and the action lands wherever the grid's auto-placement
    /// puts it.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date:
    /// `bind("data-state", "picked === id ? \"picked\" : \"idle\"")`.
    ///
    /// Through a tag the same binding is a `{…}` prop: it lands on the root,
    /// and a component's root attributes are evaluated where the tag was
    /// written. This is its spelling for an element [`render`](Self::render)ed
    /// into a page's own block.
    ///
    /// Mind the attribute *name*. The browser runtime rewrites any binding
    /// whose name ends in an HTML boolean-attribute word — `data-selected`,
    /// `data-open`, `data-disabled`, `aria-selected` — into a boolean
    /// attribute, and truncates the name to `data-` when the expression is
    /// falsy. The server HTML is correct and the damage happens at hydration,
    /// so it only shows in the browser. `data-state="{…}"` with a string value
    /// is the spelling that survives, and it is Radix's own convention.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression: `on("click", "pick(1)")`.
    ///
    /// The `on` prefix is optional — `on("click", …)` and `on("onclick", …)`
    /// are the same attribute. Use native lowercase event names, not the JSX
    /// spellings.
    ///
    /// Resolved when the event fires, which is late enough that it reaches the
    /// calling page's state through either form. `click` bubbles, so one handler
    /// on the card serves everything inside it.
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

    /// The class list this part renders with, the caller's own included.
    ///
    /// A join, not a merge: no layer here contradicts another — `audit` proves
    /// it for all seven — so this is exact for everything except a caller's
    /// `.class(…)` override, which is the one thing that can conflict. That is
    /// resolved in the browser; see [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[self.part.classes(), &self.class])
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
        let mut base = Attrs::new().set("data-slot", self.part.slot());

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the part is styled on arrival,
        // JavaScript or no JavaScript. See the cost named in [`Self::render`].
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[self.part.classes(), &self.class]),
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
    /// method to reach for when a page drives the card. A block with a script is
    /// *mounted as its own scope*, so a card rendered with one would compile a
    /// page's `bind("data-state", "picked === id ? …")` against a scope that has
    /// no `picked`. The binding does not error at build time and the element
    /// simply never updates.
    ///
    /// Interpolating this into a block also puts the part in that block's scope,
    /// which is what a binding written here needs. A `{…}` written among the
    /// *children*, though, is compiled where those children were written — see
    /// the module docs.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();

        match self.element {
            "article" => html! {
                <article @{..attrs}>
                    <slot />
                </article>
            },
            "section" => html! {
                <section @{..attrs}>
                    <slot />
                </section>
            },
            "li" => html! {
                <li @{..attrs}>
                    <slot />
                </li>
            },
            "header" => html! {
                <header @{..attrs}>
                    <slot />
                </header>
            },
            "footer" => html! {
                <footer @{..attrs}>
                    <slot />
                </footer>
            },
            "h1" => html! {
                <h1 @{..attrs}>
                    <slot />
                </h1>
            },
            "h2" => html! {
                <h2 @{..attrs}>
                    <slot />
                </h2>
            },
            "h3" => html! {
                <h3 @{..attrs}>
                    <slot />
                </h3>
            },
            "h4" => html! {
                <h4 @{..attrs}>
                    <slot />
                </h4>
            },
            "span" => html! {
                <span @{..attrs}>
                    <slot />
                </span>
            },
            "p" => html! {
                <p @{..attrs}>
                    <slot />
                </p>
            },
            _ => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// The tag a part renders as, matched back to its own `&'static` entry.
///
/// `element` arrives borrowed from a call site and the lists are `&'static`, so
/// the tag is looked up rather than leaked. An unrecognized one — including the
/// empty string an unwritten prop is — leaves the default in place.
fn pick_element(part: Part, element: &str) -> Option<&'static str> {
    part.elements().iter().copied().find(|tag| *tag == element)
}

/// A bordered surface: a header, a body, a footer, and a border around all
/// three.
///
/// ```ignore
/// <Card class="w-full max-w-sm">
///     <CardHeader>
///         <CardTitle>"Monthly report"</CardTitle>
///         <CardDescription>"For the period ending 30 June."</CardDescription>
///     </CardHeader>
///     <CardContent>@{table}</CardContent>
///     <CardFooter><Button>"Download"</Button></CardFooter>
/// </Card>
/// ```
///
/// Nothing in here is required and nothing is ordered: a card with only a
/// content region is a card, and `gap-6` means the ones left out cost nothing.
///
/// `element="article"` for a card that is a self-contained piece of content,
/// which is most of the ones that appear more than once on a page. Pair it with
/// `aria_labelledby` and an `id` on the title, or the article is announced
/// without a name.
///
/// The card pads top and bottom; its regions pad left and right. See the module
/// docs for what that buys.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Card(
    class: &str,
    element: &str,
    id: &str,
    aria_labelledby: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card().class(class);

    if let Some(tag) = pick_element(Part::Card, element) {
        props = props.element(tag);
    }
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

    // `onclick={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the element unchanged — the
    // component decides nothing about it, exactly as a React Card spreads
    // `onClick` onto its element without reading it. Applied last, so a binding
    // replaces a server value of the same name rather than sitting beside it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // The `<script>` goes *inside*, and every arm is one literal element. Both
    // halves matter: a script written in a nested `html!` would be a scope of
    // its own and would not mount this block, and a fragment root would put a
    // `<pp-fragment>` between this card and whatever contains it. A `<script>`
    // is `display: none`, so it is not a flex item and the card's `gap-6` never
    // sees it.
    match (props.element, scope) {
        ("article", false) => html! {
            <article @{..attrs}>
                <slot />
            </article>
        },
        ("article", true) => html! {
            <article @{..attrs}>
                <slot />
                <script>r##""##</script>
            </article>
        },
        ("section", false) => html! {
            <section @{..attrs}>
                <slot />
            </section>
        },
        ("section", true) => html! {
            <section @{..attrs}>
                <slot />
                <script>r##""##</script>
            </section>
        },
        ("li", false) => html! {
            <li @{..attrs}>
                <slot />
            </li>
        },
        ("li", true) => html! {
            <li @{..attrs}>
                <slot />
                <script>r##""##</script>
            </li>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

/// The card's top region: a title, a description, and an action beside them.
///
/// ```ignore
/// <CardHeader>
///     <CardTitle>"Sign in"</CardTitle>
///     <CardDescription>"Use the email you signed up with."</CardDescription>
///     <CardAction><Button variant="link">"Sign up"</Button></CardAction>
/// </CardHeader>
/// ```
///
/// A grid of two rows, and of two columns as soon as it contains a
/// [`CardAction`]. The action has to be a direct child to be placed in the
/// second column — see the module docs, which is also where the `border-b`
/// case is explained.
///
/// `element="header"` only inside a Card that is an `article` or a `section`.
/// In a `<div>` card, a `<header>` is the page's.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CardHeader(
    class: &str,
    element: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card_header().class(class);

    if let Some(tag) = pick_element(Part::Header, element) {
        props = props.element(tag);
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // Inside, for the same reason as everywhere else in this file — and here it
    // is load-bearing twice over: a script beside the header would leave a
    // `<pp-fragment>` between it and the card, and a script that was a grid item
    // would take a row from `grid-rows-[auto_auto]`. It is neither.
    match (props.element, scope) {
        ("header", false) => html! {
            <header @{..attrs}>
                <slot />
            </header>
        },
        ("header", true) => html! {
            <header @{..attrs}>
                <slot />
                <script>r##""##</script>
            </header>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

/// What the card is called.
///
/// ```ignore
/// <CardTitle>"Monthly report"</CardTitle>
/// <CardTitle element="h3" id="plan-pro">"Pro"</CardTitle>
/// ```
///
/// A `<div>` unless said otherwise, which is shadcn's default and rarely the
/// right one: give it the heading level the page's outline calls for. `id` is
/// what a Card's `aria_labelledby` points at.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CardTitle(
    class: &str,
    element: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card_title().class(class);

    if let Some(tag) = pick_element(Part::Title, element) {
        props = props.element(tag);
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    match (props.element, scope) {
        ("h1", false) => html! {
            <h1 @{..attrs}>
                <slot />
            </h1>
        },
        ("h1", true) => html! {
            <h1 @{..attrs}>
                <slot />
                <script>r##""##</script>
            </h1>
        },
        ("h2", false) => html! {
            <h2 @{..attrs}>
                <slot />
            </h2>
        },
        ("h2", true) => html! {
            <h2 @{..attrs}>
                <slot />
                <script>r##""##</script>
            </h2>
        },
        ("h3", false) => html! {
            <h3 @{..attrs}>
                <slot />
            </h3>
        },
        ("h3", true) => html! {
            <h3 @{..attrs}>
                <slot />
                <script>r##""##</script>
            </h3>
        },
        ("h4", false) => html! {
            <h4 @{..attrs}>
                <slot />
            </h4>
        },
        ("h4", true) => html! {
            <h4 @{..attrs}>
                <slot />
                <script>r##""##</script>
            </h4>
        },
        ("span", false) => html! {
            <span @{..attrs}>
                <slot />
            </span>
        },
        ("span", true) => html! {
            <span @{..attrs}>
                <slot />
                <script>r##""##</script>
            </span>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

/// The line under the title.
///
/// ```ignore
/// <CardDescription element="p">"Use the email you signed up with."</CardDescription>
/// ```
///
/// It fills the header's second row, which is the row a [`CardAction`] spans
/// whether the description is there or not.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CardDescription(
    class: &str,
    element: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card_description().class(class);

    if let Some(tag) = pick_element(Part::Description, element) {
        props = props.element(tag);
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    match (props.element, scope) {
        ("p", false) => html! {
            <p @{..attrs}>
                <slot />
            </p>
        },
        ("p", true) => html! {
            <p @{..attrs}>
                <slot />
                <script>r##""##</script>
            </p>
        },
        ("span", false) => html! {
            <span @{..attrs}>
                <slot />
            </span>
        },
        ("span", true) => html! {
            <span @{..attrs}>
                <slot />
                <script>r##""##</script>
            </span>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

/// The control in the header's top-right corner.
///
/// ```ignore
/// <CardHeader>
///     <CardTitle>"Sign in"</CardTitle>
///     <CardDescription>"Use the email you signed up with."</CardDescription>
///     <CardAction><Button variant="link">"Sign up"</Button></CardAction>
/// </CardHeader>
/// ```
///
/// Two things have to hold, and both are about markup rather than styling. It
/// has to carry `data-slot="card-action"`, which is what grows the header's
/// second column; and it has to be a **direct child** of the header, because
/// `col-start-2` only applies to a grid item. Written as this tag, both do —
/// including when it carries a binding, which is why the `<script>` goes
/// inside.
///
/// A page that renders its own action element with [`card_action_variants`]
/// has to write the `data-slot` itself.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CardAction(
    class: &str,
    element: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card_action().class(class);

    if let Some(tag) = pick_element(Part::Action, element) {
        props = props.element(tag);
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    match (props.element, scope) {
        ("span", false) => html! {
            <span @{..attrs}>
                <slot />
            </span>
        },
        ("span", true) => html! {
            <span @{..attrs}>
                <slot />
                <script>r##""##</script>
            </span>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

/// The card's body.
///
/// ```ignore
/// <CardContent>@{form}</CardContent>
/// <CardContent class="px-0">@{table}</CardContent>
/// ```
///
/// One utility — `px-6` — so `px-0` is the whole of how an image, a table or a
/// list bleeds to the card's edge without disturbing the padding above and
/// below it.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CardContent(
    class: &str,
    element: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card_content().class(class);

    if let Some(tag) = pick_element(Part::Content, element) {
        props = props.element(tag);
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    match (props.element, scope) {
        ("section", false) => html! {
            <section @{..attrs}>
                <slot />
            </section>
        },
        ("section", true) => html! {
            <section @{..attrs}>
                <slot />
                <script>r##""##</script>
            </section>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

/// The card's bottom region.
///
/// ```ignore
/// <CardFooter><Button class="w-full">"Sign in"</Button></CardFooter>
/// <CardFooter class="flex-col gap-2">…</CardFooter>
/// <CardFooter class="border-t">…</CardFooter>
/// ```
///
/// A row unless said otherwise. `border-t` gives it a rule and the padding that
/// goes with it — the module docs say what that costs through an override, and
/// what to reach for instead.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn CardFooter(
    class: &str,
    element: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = card_footer().class(class);

    if let Some(tag) = pick_element(Part::Footer, element) {
        props = props.element(tag);
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

    let scope = props.needs_scope();
    let attrs = props.attr_list();

    match (props.element, scope) {
        ("footer", false) => html! {
            <footer @{..attrs}>
                <slot />
            </footer>
        },
        ("footer", true) => html! {
            <footer @{..attrs}>
                <slot />
                <script>r##""##</script>
            </footer>
        },
        (_, false) => html! {
            <div @{..attrs}>
                <slot />
            </div>
        },
        (_, true) => html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardProps,
        CardTitle, card, card_action, card_action_variants, card_content, card_content_variants,
        card_description, card_description_variants, card_footer, card_footer_variants,
        card_header, card_header_variants, card_title, card_title_variants, card_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_card_carries_shadcns_defaults() {
        let markup = card().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.ends_with("></div>"), "{markup}");
        assert!(markup.contains(r#"data-slot="card""#), "{markup}");
        assert!(markup.contains("rounded-xl"), "{markup}");
        assert!(markup.contains("bg-card"), "{markup}");
        assert!(markup.contains("text-card-foreground"), "{markup}");
        assert!(markup.contains("shadow-sm"), "{markup}");
        assert!(markup.contains("py-6"), "{markup}");
        // The asymmetry: the card pads vertically and never horizontally.
        assert!(!markup.contains("px-6"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// Seven parts, seven slots. `card-action` is the one anything selects on;
    /// the rest are for a page's own.
    #[test]
    fn every_part_carries_its_slot() {
        for (props, slot) in [
            (card(), "card"),
            (card_header(), "card-header"),
            (card_title(), "card-title"),
            (card_description(), "card-description"),
            (card_action(), "card-action"),
            (card_content(), "card-content"),
            (card_footer(), "card-footer"),
        ] {
            let markup = props.render(Html::empty()).into_test_string();
            assert!(
                markup.contains(&format!(r#"data-slot="{slot}""#)),
                "{slot}: {markup}"
            );
        }
    }

    /// The header's second column and the action that fills it. Two class
    /// strings on two elements, and the `data-slot` between them is what makes
    /// them one layout.
    #[test]
    fn the_header_grid_and_the_action_agree() {
        let header = card_header().render(Html::empty()).into_test_string();
        let action = card_action().render(Html::empty()).into_test_string();

        assert!(
            header.contains("has-data-[slot=card-action]:grid-cols-[1fr_auto]"),
            "{header}"
        );
        assert!(header.contains("grid-rows-[auto_auto]"), "{header}");
        assert!(header.contains("auto-rows-min"), "{header}");

        assert!(action.contains(r#"data-slot="card-action""#), "{action}");
        assert!(action.contains("col-start-2"), "{action}");
        assert!(action.contains("row-span-2"), "{action}");
        assert!(action.contains("row-start-1"), "{action}");
        assert!(action.contains("justify-self-end"), "{action}");
    }

    /// `:is(.border-b)` on the element itself — carried always, fired only when
    /// the caller adds the border.
    #[test]
    fn the_border_variants_are_always_carried() {
        assert!(
            card_header()
                .render(Html::empty())
                .into_test_string()
                .contains("[.border-b]:pb-6")
        );
        assert!(
            card_footer()
                .render(Html::empty())
                .into_test_string()
                .contains("[.border-t]:pt-6")
        );
    }

    /// shadcn's `asChild`, without the child. The list is per part.
    #[test]
    fn a_part_renders_as_the_tag_it_was_given() {
        let article = card()
            .element("article")
            .render(Html::empty())
            .into_test_string();
        assert!(article.starts_with("<article "), "{article}");
        assert!(article.ends_with("></article>"), "{article}");

        let heading = card_title()
            .element("h3")
            .render(Html::escape("Pro"))
            .into_test_string();
        assert!(heading.starts_with("<h3 "), "{heading}");
        assert!(heading.ends_with(">Pro</h3>"), "{heading}");

        let paragraph = card_description()
            .element("p")
            .render(Html::empty())
            .into_test_string();
        assert!(paragraph.starts_with("<p "), "{paragraph}");

        let footer = card_footer().element("footer").render(Html::empty());
        assert!(footer.into_test_string().starts_with("<footer "));
    }

    /// A tag one part answers to is not a tag another does. The lists are
    /// closed, and an entry outside this part's own leaves the default alone.
    #[test]
    fn a_tag_from_another_parts_list_is_ignored() {
        // `h3` is a title's, not a card's.
        let markup = super::Card(
            "",
            "h3",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"data-slot="card""#), "{markup}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = card_footer()
            .class("flex-col gap-2")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("gap-2"), "{markup}");
    }

    /// No override, nothing to resolve: a plain served class list, no binding,
    /// and no scope for the runtime to mount.
    #[test]
    fn a_card_with_no_override_needs_no_runtime() {
        let props = card().id("plan");
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(markup.contains(r#"class="flex"#), "{markup}");

        assert!(!card().class("w-72").needs_scope());
        assert!(
            card()
                .bind(
                    "data-state",
                    crate::components::rahti_ui::attrs::ui_expr!(data-state={picked === 1})
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = card()
            .on(
                "click",
                crate::components::rahti_ui::attrs::ui_handler!({ pick(1) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"pp-onclick="pick(1)""#), "{markup}");
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = card_action()
            .attr("data-slot", "mine")
            .attr("title", "More")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"title="More""#), "{markup}");
    }

    /// Giving up the second column, on purpose. Written down because the
    /// header's `:has()` is the only thing in this file one element says about
    /// another.
    #[test]
    fn an_action_can_give_up_its_slot() {
        let markup = card_action()
            .attrs(Attrs::new().unset("data-slot"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("data-slot"), "{markup}");
        assert!(markup.contains("col-start-2"), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = card()
            .id(r#"a" pp-onclick="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onclick=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn the_variants_helpers_are_the_parts_own_class_lists() {
        assert_eq!(card_variants("w-72"), card().class("w-72").class_list());
        assert_eq!(
            card_header_variants("border-b"),
            card_header().class("border-b").class_list()
        );
        assert_eq!(
            card_title_variants("text-lg"),
            card_title().class("text-lg").class_list()
        );
        assert_eq!(
            card_description_variants("text-xs"),
            card_description().class("text-xs").class_list()
        );
        assert_eq!(
            card_action_variants(""),
            card_action().class("").class_list()
        );
        assert_eq!(
            card_content_variants("px-0"),
            card_content().class("px-0").class_list()
        );
        assert_eq!(
            card_footer_variants("border-t"),
            card_footer().class("border-t").class_list()
        );
    }

    /// The call form behind `<Card class="w-full max-w-sm">…</Card>`. A tag
    /// fills the props it names and takes these defaults for the rest.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let card_markup = Card(
            "",
            "article",
            "plan",
            "plan-title",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(card_markup.starts_with("<article "), "{card_markup}");
        assert!(card_markup.contains(r#"id="plan""#), "{card_markup}");
        assert!(
            card_markup.contains(r#"aria-labelledby="plan-title""#),
            "{card_markup}"
        );

        let header = CardHeader(
            "",
            "header",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(header.starts_with("<header "), "{header}");

        let title = CardTitle(
            "",
            "h2",
            "plan-title",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Pro"),
        )
        .into_test_string();
        assert!(title.contains(r#"id="plan-title""#), "{title}");
        assert!(title.ends_with(">Pro</h2>"), "{title}");

        let description = CardDescription(
            "",
            "p",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Everything, monthly."),
        )
        .into_test_string();
        assert!(
            description.ends_with(">Everything, monthly.</p>"),
            "{description}"
        );

        let action = CardAction(
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(action.contains(r#"data-slot="card-action""#), "{action}");

        let content = CardContent(
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(content.contains(r#"data-slot="card-content""#), "{content}");

        let footer = CardFooter(
            "",
            "footer",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(footer.starts_with("<footer "), "{footer}");
    }

    /// Every prop left unwritten, which is what `<Card>…</Card>` compiles to
    /// now that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_card() {
        let markup = Card(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::escape("Body"),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="card""#), "{markup}");
        assert!(markup.contains("rounded-xl"), "{markup}");
        assert!(markup.contains(">Body</div>"), "{markup}");
        assert!(!markup.contains(r#"id="""#), "{markup}");
        assert!(!markup.contains("aria-labelledby"), "{markup}");
    }

    /// What `<Card onclick={pick(1)}>` compiles to. The expression arrives as
    /// source and goes onto the element unread — and the block gets the
    /// `<script>` that makes it mountable.
    #[test]
    fn native_event_props_land_on_the_card() {
        let markup = Card(
            "",
            "",
            "",
            "",
            crate::rahti::Bindings::__from_html(&[("onclick", "pick(1)")]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"pp-onclick="pick(1)""#), "{markup}");
        assert!(markup.contains("<script"), "{markup}");
    }

    /// The script is *inside* the element, which is what keeps every part a
    /// real child of its parent — an action beside a `<pp-fragment>` would not
    /// be a grid item of the header, and `col-start-2` would do nothing.
    #[test]
    fn a_part_that_needs_a_scope_keeps_its_script_inside() {
        let markup = CardAction(
            "",
            "",
            "",
            crate::rahti::Bindings::__from_html(&[("onclick", "dismiss()")]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
        assert!(markup.contains("<script"), "{markup}");
        // The script is between the open and close tag, not after it.
        let close = markup.rfind("</div>").unwrap();
        let script = markup.find("<script").unwrap();
        assert!(script < close, "{markup}");
    }

    /// `CardProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_card() {
        assert_eq!(CardProps::new(), CardProps::default());
        assert_eq!(
            card().render(Html::empty()).into_test_string(),
            CardProps::new().render(Html::empty()).into_test_string()
        );
    }
}

/// The layer-independence check behind the seven class strings.
///
/// Every part in this file is one layer, so each of these is the same claim
/// [`Label`](super::label) makes and a smaller one than
/// [`Button`](super::button)'s — but it is what lets [`CardProps::class_list`]
/// be a join rather than a merge.
///
/// The cases worth naming are the header's and the footer's. `px-6` sits beside
/// `[.border-b]:pb-6` and `[.border-t]:pt-6`, which are different properties
/// under different variant prefixes; and `grid-cols-[1fr_auto]` is under
/// `has-data-[slot=card-action]` while `grid-rows-[auto_auto]` is unprefixed.
/// The audit reads them the way Tailwind does and lets them stand.
#[cfg(test)]
mod audit {
    use super::{ACTION, CARD, CONTENT, DESCRIPTION, FOOTER, HEADER, TITLE};
    use crate::components::rahti_ui::utils::audit::clashes;

    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("CARD", CARD),
            ("HEADER", HEADER),
            ("TITLE", TITLE),
            ("DESCRIPTION", DESCRIPTION),
            ("ACTION", ACTION),
            ("CONTENT", CONTENT),
            ("FOOTER", FOOTER),
        ] {
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
