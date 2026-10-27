// rahti-ui: table
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.3.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Table — a port of shadcn/ui's `table`.
//!
//! Nine parts, no variants, no state machine and not one line of JavaScript.
//! A Table is the plain HTML table elements with a class list on each, and
//! everything it does it does with CSS. What makes it worth a file of its own
//! is not the styling — that is thirty-odd utilities in total — but the two
//! things a Rust port has to get right that React never had to think about:
//! **the wrapper element `Table` renders but does not expose**, and **what the
//! HTML parser does to anything that is not a table element inside a table**.
//!
//! ## Which shadcn table this is
//!
//! Both sources agree here, nearly. `new-york-v4/table.json` and the source
//! block the Base UI docs page publishes as `components/ui/table.tsx` carry the
//! same nine strings, with one exception:
//!
//! ```text
//! registry   …[&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]
//! docs page  …[&:has([role=checkbox])]:pr-0
//! ```
//!
//! The registry nudges a checkbox down two pixels inside a head or a cell; the
//! page has dropped that and lets `align-middle` do the centring. **The page
//! wins**, for the reason every other file in this library gives — it is what a
//! reader is comparing against — and this one is a deliberate upstream removal
//! rather than a stale string, because the two files are otherwise identical
//! token for token. See [`HEAD`] and [`CELL`].
//!
//! There is one more thing the rendered page shows that neither source block
//! does, and it is worth naming because it will matter later. Among the demos
//! on the page, some heads and cells render `text-start` and `[&:has(…)]:pe-0`
//! where the published source says `text-left` and `[&:has(…)]:pr-0` — shadcn
//! is part-way through a move to logical properties. The published source is
//! what someone copying off that page gets today, so that is what this file
//! copies; when the migration lands, the change here is two tokens.
//!
//! ## The container is a second element, and it belongs to the caller
//!
//! `Table` does not render a `<table>`. It renders:
//!
//! ```text
//! <div data-slot="table-container" class="relative w-full overflow-x-auto">
//!   <table data-slot="table" class="w-full caption-bottom text-sm">…</table>
//! </div>
//! ```
//!
//! That wrapper is why a wide table scrolls sideways instead of pushing the
//! page out, and in React it is unreachable: `className` and `{...props}` both
//! land on the `<table>`, so nothing a call site writes can touch the `<div>`.
//! Usually that costs nothing. It costs something the moment you want a sticky
//! header, because `position: sticky` needs a scroll container with a height,
//! and the only element that could carry `max-h-96 overflow-y-auto` is the one
//! the component hid.
//!
//! So this port keeps shadcn's default exactly — `class` goes on the `<table>`,
//! like `className` — and adds one prop that React has no spelling for:
//!
//! ```ignore
//! <Table container_class="max-h-96 overflow-y-auto">…</Table>
//! ```
//!
//! [`table_container_variants`] is the same thing for a page that would rather
//! write the two elements itself. This is the only invented prop in the file,
//! and it is invented because the alternative is telling a caller their sticky
//! header is impossible.
//!
//! ## Why no part of this component puts a `<script>` in a `<table>`
//!
//! Rust composes class layers with `utils::cn`. Use frontend `twMerge`
//! bindings when utilities conflict.
//!
//! `<script>` happens to be one of the few things that survives — "in table",
//! "in table body" and "in row" all route a `<script>` start tag through the
//! "in head" rules, so it is inserted where it was written rather than fostered
//! out. That is a real guarantee and this file relies on it for
//! [`TableHeader`], [`TableBody`], [`TableFooter`] and [`TableRow`].
//!
//! [`Table`] itself does not need to rely on it, and does not: its root is the
//! container `<div>`, so its scope marker goes there, after the `</table>` and
//! outside the table entirely.
//!
//! ## The `<table>` never carries a binding, and that is not a style choice
//!
//! [`BOUND_TABLE_IS_DESTROYED`] has the measurement. In short: a `<table>` with
//! any PulsePoint binding on it, holding any nested component scope, is deleted
//! from the DOM when the runtime mounts — its sections survive, re-parented and
//! reordered into the container, and the table stops being a table. A `<Table
//! class="max-w-md">` around a `<TableCell class="font-medium">` is enough, and
//! that is an entirely ordinary thing to write.
//!
//! So this component takes the one rule that makes it immune, and two things
//! follow from it:
//!
//! Rust composes class layers with `utils::cn`. Use frontend `twMerge`
//! bindings when utilities conflict.
//!
//! Both are deviations from what the rest of this library does, and both are
//! reversible in one commit the day the runtime patches a `<table>` without
//! deleting it. `tests::no_attribute_of_a_table_is_ever_a_binding` is the guard
//! that would have to be removed with them.
//!
//! ## The caption goes first and displays last
//!
//! `<caption>` is only valid as the *first* child of `<table>`, and
//! `caption-bottom` on the table is what then paints it underneath. Both halves
//! are needed and they point in opposite directions, which is exactly the kind
//! of thing that reads as a bug six months later:
//!
//! ```ignore
//! <Table>
//!     <TableCaption>"A list of your recent invoices."</TableCaption>
//!     <TableHeader>…</TableHeader>
//!     <TableBody>…</TableBody>
//! </Table>
//! ```
//!
//! Write it anywhere else and the parser moves it for you, silently.
//!
//! ## What this port adds for accessibility, and why
//!
//! shadcn renders a bare `<th>`. A bare `<th>` is ambiguous — a screen reader
//! has to guess whether it heads a column or a row — and the fix is one
//! attribute, so [`TableHead`] takes `scope` and [`TableCell`] may render as a
//! `<th>` for the row-header case:
//!
//! ```ignore
//! <TableHead scope="col">"Invoice"</TableHead>
//! …
//! <TableCell element="th" scope="row">"INV001"</TableCell>
//! ```
//!
//! Neither is a default. shadcn's demos omit both, and a table whose first
//! column is data rather than a label should keep omitting them — `scope="row"`
//! on a plain cell is a worse lie than no scope at all.
//!
//! The `th` cell also carries [`ROW_HEADER`], which is not decoration: a `<th>`
//! arrives from the user agent centred and bold where a `<td>` does not, and
//! [`CELL`] answers neither. Without the reset a row header sits centred and
//! bold in a column of left-aligned normal cells — measured at `center`/`700`
//! against the `start`/`400` beside it, which is what the reset was written
//! for.
//!
//! ## The one state hook
//!
//! [`ROW`] carries `data-[state=selected]:bg-muted` and
//! `has-aria-expanded:bg-muted/50`. There is no selection logic anywhere in
//! shadcn's table and there is none here — the attribute is the interface, and
//! a page drives it either from the server ([`TableProps::selected`]) or from
//! PulsePoint:
//!
//! ```ignore
//! table_row().bind("data-state", crate::components::rahti_ui::attrs::ui_expr!(data-state={picked === id ? "selected" : "idle"}))
//! ```
//!
//! Bind `data-state`, never `selected` — the runtime rewrites the name of an
//! attribute it takes for a boolean, and `selected` is one of those.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// The scroll container, and the only element here shadcn does not let a caller
/// reach.
///
/// `overflow-x-auto` is the whole point: a table is the one block element that
/// refuses to shrink below its content, so without this a wide table widens the
/// page instead of scrolling inside it. `relative` is there so a caller can
/// absolutely position something against the container — a sticky shadow, a
/// loading overlay — without introducing a wrapper of their own.
///
/// `w-full` and not `max-w-full`, because the container is what gives the
/// `w-full` on the table something to be full *of*.
pub const CONTAINER: &str = "relative w-full overflow-x-auto";

/// The table. `caption-bottom` is the half of the caption rule that is not
/// about document order — see the module docs.
///
/// `text-sm` here rather than on each cell is what makes a table read as one
/// block of text; a caller wanting a denser table changes it once.
pub const TABLE: &str = "w-full caption-bottom text-sm";

/// The head. One descendant selector, and it is deliberately `[&_tr]` rather
/// than `[&>tr]`: a head with a grouped header — two rows, or a row inside
/// something else — still gets its rule.
pub const HEADER: &str = "[&_tr]:border-b";

/// The body, whose only job is to cancel the last row's border so the table
/// does not end in a line that has nothing under it.
///
/// `[&_tr:last-child]` and not `[&>tr:last-child]`, matching [`HEADER`].
pub const BODY: &str = "[&_tr:last-child]:border-0";

/// The foot: a totals row, told apart from the body by a top border and a
/// filled background rather than by weight alone.
///
/// `[&>tr]:last:border-b-0` is the one string in this file whose selector is
/// worth reading twice — it is `&>tr` scoped by `last:`, so it cancels the
/// bottom border of the final row *of the footer*, the same edge case [`BODY`]
/// handles from the other side.
pub const FOOTER: &str = "border-t bg-muted/50 font-medium [&>tr]:last:border-b-0";

/// A row: a bottom border, a hover fill, and two state hooks.
///
/// `has-aria-expanded:bg-muted/50` keeps a row that has opened something — a
/// disclosure inside a cell — looking hovered while it is open.
/// `data-[state=selected]:bg-muted` is the selection hook, and it is only an
/// attribute; see the module docs.
///
/// `transition-colors` is why the hover fades rather than snaps, and it is also
/// the reason a selected row's colour animates in when a binding flips.
pub const ROW: &str = "border-b transition-colors hover:bg-muted/50 \
                       has-aria-expanded:bg-muted/50 data-[state=selected]:bg-muted";

/// A header cell.
///
/// `h-10` gives the header row a fixed height whatever is in it, `text-left`
/// undoes the browser's centring of `<th>`, and `text-foreground` undoes the
/// muting a `<thead>` would otherwise inherit in some themes.
/// `whitespace-nowrap` is a real opinion and worth knowing about: a long header
/// widens the table and scrolls rather than wrapping.
///
/// `[&:has([role=checkbox])]:pr-0` drops the right padding of a cell holding a
/// select-all checkbox. The registry follows it with
/// `[&>[role=checkbox]]:translate-y-[2px]`; the docs page has dropped that and
/// so has this — see the module docs.
pub const HEAD: &str = "h-10 px-2 text-left align-middle font-medium whitespace-nowrap \
                        text-foreground [&:has([role=checkbox])]:pr-0";

/// A body cell. [`HEAD`] without the height, the weight or the alignment reset,
/// and with even padding instead of horizontal-only.
pub const CELL: &str = "p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0";

/// What a [`TableCell`] rendered as a `<th>` adds to [`CELL`].
///
/// shadcn has no row-header case, so it has no string for this, and leaving it
/// empty is not neutral — it is wrong. A bare `<th>` comes with two user-agent
/// declarations a `<td>` does not have, `text-align: center` and `font-weight:
/// bold`, and [`CELL`] answers neither. A row header written as
/// `<TableCell element="th">` would then sit centred and bold in a column of
/// left-aligned normal-weight cells: measured at `center`/`700` against the
/// `start`/`400` beside it.
///
/// So the `th` cell resets both, which is the promise the docs on
/// [`CELL_ELEMENTS`] make — a row header keeps the *cell's* look, not the
/// head's. [`HEAD`] carries `text-left` for exactly the same reason one step
/// up.
///
/// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
pub const ROW_HEADER: &str = "text-left font-normal";

/// The caption. `mt-4` is a *top* margin on an element that renders at the
/// bottom, which only makes sense once you know `caption-bottom` moved it.
pub const CAPTION: &str = "mt-4 text-sm text-muted-foreground";

/// The measured runtime defect this component is built around, kept as a
/// constant so that the rule it forces is greppable from the code that obeys
/// it.
///
/// Measured in the browser, 2026-08-22, against
/// `public/js/pp-reactive-v2.min.js`:
///
/// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
///
/// It is specific to `<table>`. A bound `<thead>`, `<tbody>` or `<tr>` with the
/// same nested scope inside survives, and so does an *unbound* `<table>` — the
/// four cases were run side by side. Nor is it a parser problem: parsing the
/// served HTML with `DOMParser` yields all eight tables intact, markers and
/// all, so nothing here is foster parenting.
///
/// Since a Table with a `class` and a cell with a `class` is the single most
/// ordinary thing anyone writes, the component cannot wait for a runtime fix.
/// It obeys one rule instead, and the tests hold it: **no attribute of the
/// rendered `<table>` is ever a binding.** See the module docs for what that
/// costs.
pub const BOUND_TABLE_IS_DESTROYED: &str =
    "a PulsePoint binding on a <table> with a nested scope inside destroys the element";

const CONTAINER_ELEMENTS: [&str; 1] = ["div"];
const TABLE_ELEMENTS: [&str; 1] = ["table"];
const HEADER_ELEMENTS: [&str; 1] = ["thead"];
const BODY_ELEMENTS: [&str; 1] = ["tbody"];
const FOOTER_ELEMENTS: [&str; 1] = ["tfoot"];
const ROW_ELEMENTS: [&str; 1] = ["tr"];
const HEAD_ELEMENTS: [&str; 1] = ["th"];

/// A [`TableCell`]'s tags. `th` is the row-header case — the first cell of a
/// row, naming what the rest of the row is about — and it takes `scope="row"`
/// with it or it is not worth writing. See the module docs.
const CELL_ELEMENTS: [&str; 2] = ["td", "th"];

const CAPTION_ELEMENTS: [&str; 1] = ["caption"];

/// Which of the nine a set of props is for.
///
/// A private enum rather than nine structs, for the reason [`Card`](super::card)
/// and [`Field`](super::field) give: the parts differ in a `data-slot`, a class
/// list and a tag, and not at all in how `class`, `attrs` and bindings work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    Container,
    #[default]
    Table,
    Header,
    Body,
    Footer,
    Row,
    Head,
    Cell,
    Caption,
}

impl Part {
    /// The `data-slot` this part carries.
    ///
    /// None of these are load-bearing the way `card-action`'s is — nothing in
    /// this file selects on a `data-slot`. They are for a page's own selectors
    /// and for anyone reading the DOM, and they match shadcn's exactly so that
    /// a stylesheet written against a React table works against this one.
    const fn slot(self) -> &'static str {
        match self {
            Self::Container => "table-container",
            Self::Table => "table",
            Self::Header => "table-header",
            Self::Body => "table-body",
            Self::Footer => "table-footer",
            Self::Row => "table-row",
            Self::Head => "table-head",
            Self::Cell => "table-cell",
            Self::Caption => "table-caption",
        }
    }

    /// This part's own utilities, before the caller's.
    const fn classes(self) -> &'static str {
        match self {
            Self::Container => CONTAINER,
            Self::Table => TABLE,
            Self::Header => HEADER,
            Self::Body => BODY,
            Self::Footer => FOOTER,
            Self::Row => ROW,
            Self::Head => HEAD,
            Self::Cell => CELL,
            Self::Caption => CAPTION,
        }
    }

    /// The tags this part is allowed to render as.
    ///
    /// Closed rather than open because `html!` writes a tag name literally: the
    /// element is chosen by a `match` in [`TableProps::render`], and this is the
    /// same list read back, so an unknown tag is refused where it is set rather
    /// than falling through to the default at render time.
    const fn elements(self) -> &'static [&'static str] {
        match self {
            Self::Container => &CONTAINER_ELEMENTS,
            Self::Table => &TABLE_ELEMENTS,
            Self::Header => &HEADER_ELEMENTS,
            Self::Body => &BODY_ELEMENTS,
            Self::Footer => &FOOTER_ELEMENTS,
            Self::Row => &ROW_ELEMENTS,
            Self::Head => &HEAD_ELEMENTS,
            Self::Cell => &CELL_ELEMENTS,
            Self::Caption => &CAPTION_ELEMENTS,
        }
    }

    /// The tag this part renders as unless told otherwise — the first of
    /// [`elements`](Self::elements), which for eight of the nine is the only
    /// one.
    const fn default_element(self) -> &'static str {
        self.elements()[0]
    }
}

/// The class list shadcn's `cn("w-full caption-bottom text-sm", className)`
/// returns for the `<table>` itself.
///
/// Reach for this — and for the eight below — when the element has to belong to
/// the page rather than to a component: a row whose class list a page computes,
/// or a table built inside the page's own `html!` block so that its cells can
/// read the page's PulsePoint state. Write the matching `data-slot` alongside
/// it.
pub fn table_variants(class: &str) -> String {
    cn(&[TABLE, class])
}

/// The scroll container's class list, the caller's own included.
///
/// The other half of the pair — a page writing its own container is writing the
/// element React never exposed, so this is the one variants helper here that
/// has no `className` equivalent upstream:
///
/// ```ignore
/// <div class=@{table_container_variants("max-h-96 overflow-y-auto")}
///      data-slot="table-container">
///     <table class=@{table_variants("")} data-slot="table">…</table>
/// </div>
/// ```
pub fn table_container_variants(class: &str) -> String {
    cn(&[CONTAINER, class])
}

/// [`TableHeader`]'s class list, the caller's own included.
pub fn table_header_variants(class: &str) -> String {
    cn(&[HEADER, class])
}

/// [`TableBody`]'s class list, the caller's own included.
pub fn table_body_variants(class: &str) -> String {
    cn(&[BODY, class])
}

/// [`TableFooter`]'s class list, the caller's own included.
pub fn table_footer_variants(class: &str) -> String {
    cn(&[FOOTER, class])
}

/// [`TableRow`]'s class list, the caller's own included.
pub fn table_row_variants(class: &str) -> String {
    cn(&[ROW, class])
}

/// [`TableHead`]'s class list, the caller's own included.
pub fn table_head_variants(class: &str) -> String {
    cn(&[HEAD, class])
}

/// [`TableCell`]'s class list, the caller's own included.
pub fn table_cell_variants(class: &str) -> String {
    cn(&[CELL, class])
}

/// A row-header cell's class list — [`table_cell_variants`] plus the two
/// resets a `<th>` needs. See [`ROW_HEADER`].
pub fn table_row_header_variants(class: &str) -> String {
    cn(&[CELL, ROW_HEADER, class])
}

/// [`TableCaption`]'s class list, the caller's own included.
pub fn table_caption_variants(class: &str) -> String {
    cn(&[CAPTION, class])
}

/// One Table part's props, with shadcn's defaults already in place.
///
/// Build one with [`table`], [`table_container`], [`table_header`],
/// [`table_body`], [`table_footer`], [`table_row`], [`table_head`],
/// [`table_cell`] or [`table_caption`] — each picks the part, and everything
/// after it is the same chain. Nothing here is required.
///
/// `class`, `element` and `container_class` are fields of their own. `class` is
/// the one thing that can conflict with what the component chose, and resolving
/// that is the browser's job; `element` decides which tag is written and so
/// cannot be an attribute; `container_class` belongs to a *different element*
/// than every other prop on the list, which is why it could not be one either.
/// Everything else — `id`, `colspan`, `scope`, `aria-*` — is an ordinary
/// attribute and lives in `attrs` with the caller's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableProps {
    part: Part,
    class: String,
    container_class: String,
    element: &'static str,
    attrs: Attrs,
    container_attrs: Attrs,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            part: Part::Table,
            class: String::new(),
            container_class: String::new(),
            element: Part::Table.default_element(),
            attrs: Attrs::new(),
            container_attrs: Attrs::new(),
        }
    }
}

/// A default [`Table`]'s props — the container and the `<table>` inside it.
pub fn table() -> TableProps {
    TableProps::of(Part::Table)
}

/// A bare scroll container's props, for a page rendering the `<table>` itself.
pub fn table_container() -> TableProps {
    TableProps::of(Part::Container)
}

/// A default [`TableHeader`]'s props.
pub fn table_header() -> TableProps {
    TableProps::of(Part::Header)
}

/// A default [`TableBody`]'s props.
pub fn table_body() -> TableProps {
    TableProps::of(Part::Body)
}

/// A default [`TableFooter`]'s props.
pub fn table_footer() -> TableProps {
    TableProps::of(Part::Footer)
}

/// A default [`TableRow`]'s props.
pub fn table_row() -> TableProps {
    TableProps::of(Part::Row)
}

/// A default [`TableHead`]'s props.
pub fn table_head() -> TableProps {
    TableProps::of(Part::Head)
}

/// A default [`TableCell`]'s props.
pub fn table_cell() -> TableProps {
    TableProps::of(Part::Cell)
}

/// A default [`TableCaption`]'s props.
pub fn table_caption() -> TableProps {
    TableProps::of(Part::Caption)
}

impl TableProps {
    fn of(part: Part) -> Self {
        Self {
            part,
            element: part.default_element(),
            ..Self::default()
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// The `className` override, on the part's own element.
    ///
    /// On a [`table`] this is the `<table>`, matching shadcn — the container
    /// takes [`container_class`](Self::container_class).
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// The scroll container's `class`, on a [`table`].
    ///
    /// The prop React has no spelling for. `max-h-96 overflow-y-auto` here is
    /// what makes `sticky top-0` on a [`TableHead`] work; see the module docs.
    /// Ignored on every other part, which have no container.
    pub fn container_class(mut self, class: impl Into<String>) -> Self {
        self.container_class = class.into();
        self
    }

    /// The tag to render as.
    ///
    /// Only [`table_cell`] has a choice — `td` or `th`. Anything else is
    /// refused here rather than at render time.
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

    /// The part's own id.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// What a header cell heads: `"col"`, `"row"`, `"colgroup"`, `"rowgroup"`.
    ///
    /// Not a default — see the module docs on why an unqualified `<th>` is
    /// sometimes the honest answer.
    pub fn scope(self, scope: &str) -> Self {
        self.attr("scope", scope)
    }

    /// How many columns a head or a cell spans. `0` leaves the attribute off.
    pub fn col_span(self, span: u32) -> Self {
        if span == 0 {
            self
        } else {
            self.attr("colspan", span.to_string())
        }
    }

    /// How many rows a head or a cell spans. `0` leaves the attribute off.
    pub fn row_span(self, span: u32) -> Self {
        if span == 0 {
            self
        } else {
            self.attr("rowspan", span.to_string())
        }
    }

    /// A row's selected state, decided on the server.
    ///
    /// `data-state="selected"` is the whole of it — [`ROW`] does the rest. A
    /// page whose selection lives in the browser binds the same attribute
    /// instead; see the module docs.
    pub fn selected(self, selected: bool) -> Self {
        if selected {
            self.attr("data-state", "selected")
        } else {
            self.unset("data-state")
        }
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

    /// An attribute on the scroll container, on a [`table`].
    pub fn container_attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.container_attrs = std::mem::take(&mut self.container_attrs).set(name, value);
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

    /// Everything in `attrs`, laid over the scroll container's.
    pub fn container_attrs(mut self, attrs: Attrs) -> Self {
        self.container_attrs = merge(std::mem::take(&mut self.container_attrs), &attrs);
        self
    }

    /// This part's own utilities, before the caller's.
    ///
    /// Identical to [`Part::classes`] except for a [`table_cell`] rendering as
    /// a `<th>`, which adds [`ROW_HEADER`] to undo the two user-agent rules a
    /// `<th>` brings and a `<td>` does not.
    fn own_classes(&self) -> String {
        if self.part == Part::Cell && self.element == "th" {
            cn(&[CELL, ROW_HEADER])
        } else {
            self.part.classes().to_string()
        }
    }

    /// The class list this part renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[&self.own_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// A [`table`]'s own `class` is the exception: it is joined on the server,
    /// so it asks for nothing. The container's `class` and the caller's
    /// bindings still count — they end up on the container, which is the same
    /// rendered root.
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
            || self
                .container_attrs
                .iter()
                .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes.
    ///
    /// For every part but [`Table`] this is the library's usual arrangement: a
    /// `data-slot`, a class list that becomes `twMerge(…)` when the caller
    /// overrode it, and the caller's attributes last.
    ///
    /// The `<table>` is the exception, and [`BOUND_TABLE_IS_DESTROYED`] says
    /// why: its class is joined on the server and every bound attribute is
    /// lifted off onto the container, so that nothing here can ever be a
    /// binding.
    fn attr_list(&self) -> Attrs {
        let base = Attrs::new().set("data-slot", self.part.slot());

        if self.part == Part::Table {
            let base = base.set("class", self.class_list());
            return merge(base, &self.static_attrs());
        }

        let base = with_class(base, &self.own_classes(), &self.class);

        // The caller's own, last, so a spread of the caller's props wins — the
        // same last-wins rule `AttrList` applies when it writes the tag.
        merge(base, &self.attrs)
    }

    /// The scroll container's attributes, on a [`table`].
    ///
    /// This is where a caller's bindings end up. The container is the
    /// component's root, which is where every other component in this library
    /// puts them anyway — and unlike the `<table>` it is an ordinary `<div>`
    /// that the runtime patches without incident.
    fn container_attr_list(&self) -> Attrs {
        let base = Attrs::new().set("data-slot", Part::Container.slot());
        let base = with_class(base, CONTAINER, &self.container_class);
        let base = merge(base, &self.container_attrs);

        merge(base, &self.bound_attrs())
    }

    /// The caller's attributes that are safe on a `<table>` — everything whose
    /// value is not a PulsePoint expression.
    fn static_attrs(&self) -> Attrs {
        self.attrs.only(|name| !self.attrs.is_bound(name))
    }

    /// The caller's attributes that a `<table>` cannot carry, for the container
    /// to take instead — its bindings, still live.
    fn bound_attrs(&self) -> Attrs {
        self.attrs.only(|name| self.attrs.is_bound(name))
    }

    /// The rendered element, or — for a [`table`] — the container and the
    /// `<table>` inside it.
    ///
    /// This carries no `<script>`. A `render` is for an element the *page*
    /// owns, written into the page's own `html!` block, and a scope marker
    /// there would cut the page's PulsePoint state off from its own cells. The
    /// components below add one; see the module docs for why that is a real
    /// question inside a table and not a formality.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();

        match self.part {
            Part::Table => {
                let container = self.container_attr_list();
                html! {
                    <div @{..container}>
                        <table @{..attrs}>
                            <slot />
                        </table>
                    </div>
                }
            }
            Part::Container => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
            Part::Header => html! {
                <thead @{..attrs}>
                    <slot />
                </thead>
            },
            Part::Body => html! {
                <tbody @{..attrs}>
                    <slot />
                </tbody>
            },
            Part::Footer => html! {
                <tfoot @{..attrs}>
                    <slot />
                </tfoot>
            },
            Part::Row => html! {
                <tr @{..attrs}>
                    <slot />
                </tr>
            },
            Part::Head => html! {
                <th @{..attrs}>
                    <slot />
                </th>
            },
            Part::Caption => html! {
                <caption @{..attrs}>
                    <slot />
                </caption>
            },
            Part::Cell => match self.element {
                "th" => html! {
                    <th @{..attrs}>
                        <slot />
                    </th>
                },
                _ => html! {
                    <td @{..attrs}>
                        <slot />
                    </td>
                },
            },
        }
    }
}

/// `class`, joined on the server when there is nothing to resolve and handed to
/// `twMerge` when there is.
///
/// With no caller override the attribute stays a finished class list and the
/// part is styled on arrival, JavaScript or no JavaScript. With one, the merge
/// is the browser's — there is no Rust tailwind-merge — which is why every part
/// that takes a `class` also asks for a scope.
fn with_class(base: Attrs, own: &str, caller: &str) -> Attrs {
    if caller.trim().is_empty() {
        base.set("class", own)
    } else {
        base.set(
            "class",
            crate::components::rahti_ui::utils::cn(&[own, caller]),
        )
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// The tag a caller asked for, if the part answers to it.
fn pick_element(part: Part, element: &str) -> Option<&'static str> {
    part.elements().iter().copied().find(|tag| *tag == element)
}

/// The lines every part below repeats: class, id, the caller's attributes, then
/// the caller's bindings last so a binding replaces a server value of the same
/// name.
fn assembled(
    props: TableProps,
    class: &str,
    id: &str,
    attrs: Attrs,
    bindings: Bindings,
) -> TableProps {
    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all — so it is written only when the caller said
    // something.
    let mut props = props.class(class);
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
    props
}

/// A scroll container and the table inside it.
///
/// ```ignore
/// <Table>
///     <TableCaption>"A list of your recent invoices."</TableCaption>
///     <TableHeader>
///         <TableRow><TableHead scope="col">"Invoice"</TableHead></TableRow>
///     </TableHeader>
///     <TableBody>
///         <TableRow><TableCell>"INV001"</TableCell></TableRow>
///     </TableBody>
/// </Table>
/// ```
///
/// `class` lands on the `<table>`, as `className` does upstream.
/// `container_class` lands on the `<div>` around it, which is where a height
/// goes if the header is to stick. The caption, if there is one, goes first.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Table(
    class: &str,
    container_class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = assembled(
        table().container_class(container_class),
        class,
        id,
        attrs,
        bindings,
    );

    let scope = props.needs_scope();
    let container = props.container_attr_list();
    let attrs = props.attr_list();

    // The scope marker sits in the container, after the `</table>` — the one
    // place in this component where it is outside a table's insertion mode
    // altogether, and so the one part of this file that needs no argument about
    // foster parenting.
    if scope {
        html! {
            <div @{..container}>
                <table @{..attrs}>
                    <slot />
                </table>
                <script>r##""##</script>
            </div>
        }
    } else {
        html! {
            <div @{..container}>
                <table @{..attrs}>
                    <slot />
                </table>
            </div>
        }
    }
}

/// The scroll container on its own, for a page writing the `<table>` itself.
///
/// Rare, and the reason it exists is the scope rule: a `<Table>` is a component
/// and so a PulsePoint scope of its own, so a table whose cells read the page's
/// state has to be built in the page's block. This is that page's wrapper.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableContainer(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = assembled(table_container(), class, id, attrs, bindings);
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <slot />
                <script>r##""##</script>
            </div>
        }
    } else {
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// A `<thead>`. Its rows get a bottom border; nothing else here is styled.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableHeader(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = assembled(table_header(), class, id, attrs, bindings);
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // Inside the `<thead>`, and safe there: "in table body" routes a `<script>`
    // start tag through the "in head" rules, so it is inserted where it was
    // written rather than fostered out ahead of the table. See the module docs.
    if scope {
        html! {
            <thead @{..attrs}>
                <slot />
                <script>r##""##</script>
            </thead>
        }
    } else {
        html! {
            <thead @{..attrs}>
                <slot />
            </thead>
        }
    }
}

/// A `<tbody>`. Cancels the last row's border, and nothing else.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableBody(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let props = assembled(table_body(), class, id, attrs, bindings);
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <tbody @{..attrs}>
                <slot />
                <script>r##""##</script>
            </tbody>
        }
    } else {
        html! {
            <tbody @{..attrs}>
                <slot />
            </tbody>
        }
    }
}

/// A `<tfoot>` — the totals row, set apart by a top border and a fill.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableFooter(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = assembled(table_footer(), class, id, attrs, bindings);
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <tfoot @{..attrs}>
                <slot />
                <script>r##""##</script>
            </tfoot>
        }
    } else {
        html! {
            <tfoot @{..attrs}>
                <slot />
            </tfoot>
        }
    }
}

/// A `<tr>`.
///
/// `selected` writes `data-state="selected"`, which is the only state this
/// component has and the only thing [`ROW`] is waiting for. A page whose
/// selection lives in the browser binds the attribute instead — bind
/// `data-state`, not `selected`.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableRow(
    class: &str,
    id: &str,
    selected: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let base = if selected {
        table_row().selected(true)
    } else {
        table_row()
    };
    let props = assembled(base, class, id, attrs, bindings);
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    // Inside the `<tr>`, and safe there for the same reason as in a `<thead>`:
    // "in row" defers a `<script>` to "in table", which defers it to "in head".
    if scope {
        html! {
            <tr @{..attrs}>
                <slot />
                <script>r##""##</script>
            </tr>
        }
    } else {
        html! {
            <tr @{..attrs}>
                <slot />
            </tr>
        }
    }
}

/// A `<th>` in the header.
///
/// `scope="col"` is not written unless asked for, and it should be asked for
/// whenever the head really does name a column — see the module docs.
/// `colspan`/`rowspan` of `0` leave the attribute off.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableHead(
    class: &str,
    id: &str,
    scope: &str,
    colspan: u32,
    rowspan: u32,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut base = table_head().col_span(colspan).row_span(rowspan);
    if !scope.is_empty() {
        base = base.scope(scope);
    }

    let props = assembled(base, class, id, attrs, bindings);
    let needs = props.needs_scope();
    let attrs = props.attr_list();

    if needs {
        html! {
            <th @{..attrs}>
                <slot />
                <script>r##""##</script>
            </th>
        }
    } else {
        html! {
            <th @{..attrs}>
                <slot />
            </th>
        }
    }
}

/// A `<td>`, or a `<th>` when `element="th"`.
///
/// The `th` case is the row header — the first cell of a row, naming the rest
/// of it — and it wants `scope="row"` alongside. `colspan`/`rowspan` of `0`
/// leave the attribute off, which is what makes `<TableCell colspan={3}>` in a
/// footer read the way shadcn's demo does.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableCell(
    class: &str,
    element: &str,
    id: &str,
    scope: &str,
    colspan: u32,
    rowspan: u32,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut base = table_cell().col_span(colspan).row_span(rowspan);

    if let Some(tag) = pick_element(Part::Cell, element) {
        base = base.element(tag);
    }
    if !scope.is_empty() {
        base = base.scope(scope);
    }

    let props = assembled(base, class, id, attrs, bindings);
    let needs = props.needs_scope();
    let attrs = props.attr_list();

    match (props.element, needs) {
        ("th", false) => html! {
            <th @{..attrs}>
                <slot />
            </th>
        },
        ("th", true) => html! {
            <th @{..attrs}>
                <slot />
                <script>r##""##</script>
            </th>
        },
        (_, false) => html! {
            <td @{..attrs}>
                <slot />
            </td>
        },
        (_, true) => html! {
            <td @{..attrs}>
                <slot />
                <script>r##""##</script>
            </td>
        },
    }
}

/// A `<caption>`. Write it as the **first** child of [`Table`]; `caption-bottom`
/// is what then paints it underneath. See the module docs.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn TableCaption(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = assembled(table_caption(), class, id, attrs, bindings);
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <caption @{..attrs}>
                <slot />
                <script>r##""##</script>
            </caption>
        }
    } else {
        html! {
            <caption @{..attrs}>
                <slot />
            </caption>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BODY, CAPTION, CELL, CONTAINER, FOOTER, HEAD, HEADER, ROW, ROW_HEADER, TABLE, Table,
        TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow, table,
        table_cell, table_cell_variants, table_head_variants, table_row, table_row_header_variants,
        table_row_variants,
    };
    use crate::components::rahti_ui::attrs::Attrs;
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    fn plain_table() -> String {
        Table(
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string()
    }

    fn row(class: &str, selected: bool) -> String {
        TableRow(
            class,
            "",
            selected,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string()
    }

    fn cell(element: &str, scope: &str, colspan: u32) -> String {
        TableCell(
            "",
            element,
            "",
            scope,
            colspan,
            0,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string()
    }

    /// A Table is two elements, not one, and the class lists are on opposite
    /// sides of the boundary — the container's own, the table's own.
    #[test]
    fn a_table_is_a_container_and_a_table() {
        let markup = plain_table();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(
            markup.contains(r#"data-slot="table-container""#),
            "{markup}"
        );
        assert!(
            markup.contains(r#"class="relative w-full overflow-x-auto""#),
            "{markup}"
        );
        assert!(markup.contains("<table "), "{markup}");
        assert!(markup.contains(r#"data-slot="table""#), "{markup}");
        assert!(
            markup.contains(r#"class="w-full caption-bottom text-sm""#),
            "{markup}"
        );
        assert!(markup.ends_with("</div>"), "{markup}");
        // Nothing needs a scope, so nothing gets one.
        assert!(!markup.contains("<script"), "{markup}");
    }

    /// `class` lands on the `<table>`; `container_class` lands on the `<div>`.
    /// Getting these the wrong way round is the one mistake this component's
    /// shape makes easy, so it is asserted rather than assumed.
    #[test]
    fn the_two_class_props_land_on_different_elements() {
        let markup = Table(
            "text-xs",
            "max-h-96",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();
        let split = markup.find("<table").expect("a table");
        let container = &markup[..split];
        let table_tag = &markup[split..];

        assert!(container.contains("max-h-96"), "{container}");
        assert!(!container.contains("text-xs"), "{container}");
        assert!(table_tag.contains("text-xs"), "{table_tag}");
        assert!(!table_tag.contains("max-h-96"), "{table_tag}");
    }

    /// The scope marker for a Table is outside the `<table>` entirely — after
    /// the closing tag, inside the container. See the module docs on foster
    /// parenting.
    #[test]
    fn a_tables_scope_marker_is_never_inside_the_table() {
        let markup = Table(
            "",
            "max-h-96",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(!markup.contains("<script"), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
    }

    /// The rule the whole component is built around. See
    /// [`BOUND_TABLE_IS_DESTROYED`] for the measurement behind it: a `<table>`
    /// carrying any binding, with a nested scope inside, is deleted by the
    /// runtime on mount.
    ///
    /// Every route a binding could take to the `<table>` is tried here — the
    /// `class` prop, a `{…}` prop, and a bound value smuggled in through
    /// `attrs` — and none of them may reach it.
    #[test]
    fn no_attribute_of_a_table_is_ever_a_binding() {
        let cases = [
            Table(
                "max-w-md",
                "",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::default(),
            )
            .into_test_string(),
            Table(
                "",
                "",
                "",
                crate::rahti::Bindings::__from_html(&[("onclick", "setPicked(1)")]),
                Attrs::new(),
                Html::default(),
            )
            .into_test_string(),
            Table(
                "",
                "",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new().set("data-hits", "{hits}"),
                Html::default(),
            )
            .into_test_string(),
            table()
                .class("max-w-md")
                .on(
                    "click",
                    crate::components::rahti_ui::attrs::ui_handler!({ setPicked(1) }),
                )
                .render(Html::default())
                .into_test_string(),
        ];

        for markup in cases {
            let open = markup.find("<table").expect("a table");
            let tag = &markup[open..markup[open..].find('>').expect("an open tag") + open];

            assert!(!tag.contains('{'), "binding on the <table>: {tag}");
        }
    }

    /// …and the two places those bindings went instead: the class joined on the
    /// server, the handler moved out to the container.
    #[test]
    fn a_tables_class_is_joined_and_its_bindings_move_to_the_container() {
        let markup = Table(
            "max-w-md",
            "",
            "",
            crate::rahti::Bindings::__from_html(&[("onclick", "setPicked(1)")]),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();
        let open = markup.find("<table").expect("a table");
        let container = &markup[..open];
        let table_tag = &markup[open..];

        // Joined, not merged — the whole list is there, finished, on the server.
        assert!(
            table_tag.contains(r#"class="w-full caption-bottom text-sm max-w-md""#),
            "{table_tag}"
        );
        assert!(!table_tag.contains("twMerge("), "{table_tag}");
        // The handler is on the root, where every other component puts one.
        assert!(
            container.contains(r#"pp-onclick="setPicked(1)""#),
            "{container}"
        );
        assert!(!table_tag.contains("onclick"), "{table_tag}");
    }

    /// A static attribute is not a binding, so it stays on the `<table>` where
    /// shadcn spreads it. Only expressions move.
    #[test]
    fn a_static_attribute_stays_on_the_table() {
        let markup = Table(
            "",
            "",
            "invoices",
            crate::rahti::Bindings::default(),
            Attrs::new().set("aria-describedby", "note"),
            Html::default(),
        )
        .into_test_string();
        let table_tag = &markup[markup.find("<table").expect("a table")..];

        assert!(table_tag.contains(r#"id="invoices""#), "{table_tag}");
        assert!(
            table_tag.contains(r#"aria-describedby="note""#),
            "{table_tag}"
        );
    }

    /// …but for the parts whose root *is* a table element, it is inside, and
    /// that is only safe because the parser routes a `<script>` through the
    /// "in head" rules. Assert the placement so a future edit cannot quietly
    /// move it somewhere the parser would foster it out.
    #[test]
    fn a_table_parts_scope_marker_is_inside_its_own_element() {
        for markup in [
            TableHeader(
                "sticky",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::default(),
            )
            .into_test_string(),
            TableBody(
                "divide-y",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::default(),
            )
            .into_test_string(),
            TableFooter(
                "font-bold",
                "",
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::default(),
            )
            .into_test_string(),
            row("cursor-pointer", false),
        ] {
            assert!(!markup.contains("<script"), "{markup}");
            assert!(markup.contains("pp-component="), "{markup}");
        }
    }

    /// The nine strings, exactly as the docs page publishes them.
    #[test]
    fn the_class_lists_are_the_docs_pages() {
        assert_eq!(CONTAINER, "relative w-full overflow-x-auto");
        assert_eq!(TABLE, "w-full caption-bottom text-sm");
        assert_eq!(HEADER, "[&_tr]:border-b");
        assert_eq!(BODY, "[&_tr:last-child]:border-0");
        assert_eq!(
            FOOTER,
            "border-t bg-muted/50 font-medium [&>tr]:last:border-b-0"
        );
        assert_eq!(
            ROW,
            "border-b transition-colors hover:bg-muted/50 has-aria-expanded:bg-muted/50 \
             data-[state=selected]:bg-muted"
        );
        assert_eq!(
            HEAD,
            "h-10 px-2 text-left align-middle font-medium whitespace-nowrap text-foreground \
             [&:has([role=checkbox])]:pr-0"
        );
        assert_eq!(
            CELL,
            "p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0"
        );
        assert_eq!(CAPTION, "mt-4 text-sm text-muted-foreground");
    }

    /// The one token the registry has and the page does not. Named here so that
    /// putting it back is a decision rather than an accident.
    #[test]
    fn the_checkbox_nudge_the_registry_still_ships_is_not_here() {
        assert!(!HEAD.contains("translate-y-"), "{HEAD}");
        assert!(!CELL.contains("translate-y-"), "{CELL}");
        // …while the padding rule it sat beside is kept.
        assert!(HEAD.contains("[&:has([role=checkbox])]:pr-0"), "{HEAD}");
        assert!(CELL.contains("[&:has([role=checkbox])]:pr-0"), "{CELL}");
    }

    /// The logical-property spelling the rendered demos are moving to, which
    /// the published source has not adopted. If this test starts failing,
    /// upstream finished the migration.
    #[test]
    fn the_published_source_is_still_physical_not_logical() {
        assert!(
            HEAD.contains("text-left") && !HEAD.contains("text-start"),
            "{HEAD}"
        );
        assert!(
            !HEAD.contains(":pe-0") && !CELL.contains(":pe-0"),
            "{HEAD} / {CELL}"
        );
    }

    /// Selection is an attribute and nothing else — no script, no class of its
    /// own, just the hook `ROW` is already waiting for.
    #[test]
    fn a_selected_row_is_one_attribute() {
        let selected = row("", true);
        let idle = row("", false);

        assert!(selected.contains(r#"data-state="selected""#), "{selected}");
        assert!(!selected.contains("<script"), "{selected}");
        assert!(!idle.contains("data-state"), "{idle}");
        assert!(ROW.contains("data-[state=selected]:bg-muted"), "{ROW}");
    }

    /// A page driving selection from the browser binds the same attribute.
    #[test]
    fn selection_can_come_from_the_browser_instead() {
        let markup = table_row()
            .bind("data-state", crate::components::rahti_ui::attrs::ui_expr!(data-state={picked === 1 ? "selected" : "idle"}))
            .render(Html::default())
            .into_test_string();

        assert!(markup.contains("data-state=\"{picked === 1 ?"), "{markup}");
        // A `render` never carries a scope marker — the page's block owns it.
        assert!(!markup.contains("<script"), "{markup}");
    }

    /// A cell is a `td` unless asked for a `th`, and an unknown tag is refused
    /// rather than silently honoured.
    #[test]
    fn a_cell_is_a_td_unless_it_is_a_row_header() {
        let plain = cell("", "", 0);
        let header = cell("th", "row", 0);
        let bogus = cell("span", "", 0);

        assert!(plain.starts_with("<td "), "{plain}");
        assert!(header.starts_with("<th "), "{header}");
        assert!(header.contains(r#"scope="row""#), "{header}");
        // …and it keeps the cell's class list, not the head's.
        assert!(header.contains("p-2 align-middle"), "{header}");
        assert!(!header.contains("h-10"), "{header}");
        // An unknown tag falls back to the default rather than being written.
        assert!(bogus.starts_with("<td "), "{bogus}");
    }

    /// A `<th>` arrives with `text-align: center` and `font-weight: bold` from
    /// the user agent, and a `<td>` does not. `CELL` answers neither, so a row
    /// header would sit centred and bold in a column of left-aligned normal
    /// cells — measured at `center`/`700` in the browser before this was added.
    #[test]
    fn a_row_header_undoes_what_a_th_brings_with_it() {
        let header = cell("th", "row", 0);
        let plain = cell("", "", 0);

        assert!(header.contains("text-left"), "{header}");
        assert!(header.contains("font-normal"), "{header}");
        // …and a `<td>` gets neither, because it needs neither.
        assert!(!plain.contains("text-left"), "{plain}");
        assert!(!plain.contains("font-normal"), "{plain}");

        assert_eq!(table_cell_variants(""), CELL);
        assert_eq!(
            table_row_header_variants(""),
            format!("{CELL} {ROW_HEADER}")
        );
    }

    /// The reset is part of the layer the caller's class is merged *against*,
    /// so `font-medium` — which is what shadcn's own demos pass on that cell —
    /// still beats it in the browser rather than landing beside a stale
    /// `font-normal` the merge never saw.
    #[test]
    fn a_row_headers_reset_is_inside_the_merge() {
        let markup = TableCell(
            "font-medium",
            "th",
            "",
            "row",
            0,
            0,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("font-medium"), "{markup}");
    }

    /// `colspan` of `0` is "no attribute", which is what lets one prop serve
    /// both the footer's spanning cell and every ordinary one.
    #[test]
    fn a_span_of_zero_writes_no_attribute() {
        let none = cell("", "", 0);
        let three = cell("", "", 3);

        assert!(!none.contains("colspan"), "{none}");
        assert!(three.contains(r#"colspan="3""#), "{three}");
        assert_eq!(table_cell().col_span(0), table_cell());
    }

    /// A head writes no `scope` unless asked. shadcn's demos do not, and the
    /// module docs argue why that stays the default.
    #[test]
    fn a_head_is_unqualified_until_told_otherwise() {
        let bare = TableHead(
            "",
            "",
            "",
            0,
            0,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();
        let scoped = TableHead(
            "",
            "",
            "col",
            0,
            0,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(!bare.contains("scope="), "{bare}");
        assert!(scoped.contains(r#"scope="col""#), "{scoped}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn a_caller_class_is_composed_for_the_frontend() {
        let overridden = row("bg-red-50", false);
        let _plain = row("", false);

        assert!(!overridden.contains("{twMerge("), "{overridden}");
        assert!(overridden.contains("class=\""), "{overridden}");
        assert!(overridden.contains("bg-red-50"), "{overridden}");
    }

    /// The caption is a `<caption>` and carries a top margin, which only reads
    /// correctly next to the table's `caption-bottom`.
    #[test]
    fn the_caption_is_pushed_down_from_a_table_that_moved_it() {
        let markup = TableCaption(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.starts_with("<caption "), "{markup}");
        assert!(markup.contains("mt-4"), "{markup}");
        assert!(TABLE.contains("caption-bottom"), "{TABLE}");
    }

    /// The container is reachable from `render` too, for a page that builds the
    /// table in its own block.
    #[test]
    fn a_page_can_render_the_pair_itself() {
        let markup = table()
            .container_class("rounded-md border")
            .render(Html::default())
            .into_test_string();

        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("rounded-md border"), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
    }

    #[test]
    fn a_plain_part_needs_no_scope() {
        assert!(!table().needs_scope());
        assert!(!table_row().needs_scope());
        // …but a container class does, because it is `twMerge`'s to resolve.
        assert!(!table().container_class("max-h-96").needs_scope());
        assert!(!table_row().class("bg-red-50").needs_scope());
    }

    #[test]
    fn the_caller_can_still_pass_attributes() {
        let markup = TableRow(
            "",
            "invoice-1",
            false,
            crate::rahti::Bindings::default(),
            Attrs::new().set("aria-label", "Invoice INV001"),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains(r#"id="invoice-1""#), "{markup}");
        assert!(
            markup.contains(r#"aria-label="Invoice INV001""#),
            "{markup}"
        );
    }
}

#[cfg(test)]
mod audit {
    use super::{BODY, CAPTION, CELL, CONTAINER, FOOTER, HEAD, HEADER, ROW, ROW_HEADER, TABLE};
    use crate::components::rahti_ui::utils::{audit::clashes, cn};

    /// Every class list this file owns, for the two checks below. The row
    /// header is the joined layer rather than the fragment, because the join is
    /// what actually reaches an element.
    const LISTS: [&str; 9] = [
        CONTAINER, TABLE, HEADER, BODY, FOOTER, ROW, HEAD, CELL, CAPTION,
    ];

    /// No layer contradicts itself. There is only one layer per part here — no
    /// variants, no sizes — so this is the whole of the join.
    #[test]
    fn no_utility_contradicts_another() {
        for list in LISTS {
            let found = clashes(list);

            assert!(
                found.is_empty(),
                "{} clashes in `{list}`:\n{}",
                found.len(),
                found.join("\n")
            );
        }
    }

    /// The row-header reset is added to `CELL`, so the joined layer is what has
    /// to be free of contradictions — `text-left` and `font-normal` must not
    /// answer a question `CELL` already answered.
    #[test]
    fn a_row_header_layer_does_not_contradict_the_cell_it_extends() {
        let joined = cn(&[CELL, ROW_HEADER]);
        let found = clashes(&joined);

        assert!(
            found.is_empty(),
            "{} clashes in `{joined}`:
{}",
            found.len(),
            found.join(
                "
"
            )
        );
    }

    /// Nothing here reaches for a custom property or a marker class, because
    /// there is no library behind a table — the elements are the primitives.
    #[test]
    fn nothing_reaches_for_what_a_library_published() {
        for list in LISTS {
            assert!(!list.contains("--radix-"), "{list}");
            assert!(!list.contains("cn-"), "{list}");
        }
    }
}
