// rahti-ui: field
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Field — a port of shadcn/ui's Base UI `field`.
//!
//! Ten parts, one family. shadcn's Field is not a control: it is the layout
//! and the wiring *around* a control, so that a label, the control, a hint,
//! and an error read as one thing to a person and as one group to a screen
//! reader. The class strings here are shadcn's Base UI ones, copied rather
//! than reinterpreted, so a Field renders pixel-identically to the React one
//! under the same Tailwind theme.
//!
//! | Tag | Element | What it is |
//! | --- | --- | --- |
//! | [`FieldSet`] | `<fieldset>` | A named run of fields |
//! | [`FieldLegend`] | `<legend>` | That run's title |
//! | [`FieldGroup`] | `<div>` | The stack a set's fields live in |
//! | [`Field`] | `<div role="group">` | One field: label, control, hint, error |
//! | [`FieldContent`] | `<div>` | Label and hint as one column, beside a control |
//! | [`FieldLabel`] | `<label>` | The control's name |
//! | [`FieldTitle`] | `<div>` | A label's look without a label's meaning |
//! | [`FieldDescription`] | `<p>` | The hint under, or beside, the control |
//! | [`FieldSeparator`] | `<div>` | A rule between sections, with optional text |
//! | [`FieldError`] | `<div role="alert">` | What went wrong |
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <FieldGroup>
//!     <Field>
//!         <FieldLabel r#for="email">"Email"</FieldLabel>
//!         @{input().r#type("email").id("email").render()}
//!         <FieldDescription>"We only use it to sign you in."</FieldDescription>
//!     </Field>
//! </FieldGroup>
//! ```
//!
//! Every prop is optional — `html!` closes a props literal with
//! `..Default::default()`, so an unwritten prop is this component's default.
//! `r#for` on a [`FieldLabel`] is the one prop that cannot be spelled the way
//! JSX spells it, for the same reason [`Label`](super::label)'s cannot: a prop
//! names a Rust parameter and `for` is a keyword. The rendered attribute is
//! `for`.
//!
//! ## The layout is written in selectors, so the markup shape is the API
//!
//! Almost every utility in this family reaches out of its own element. A
//! Field's `*:w-full` sizes its **direct children**; a description's
//! `nth-last-2:-mt-1` asks where it sits **among its siblings**; a label's
//! `has-[>[data-slot=field]]:border` draws a card only when a whole Field is
//! nested **directly inside** it; a set's `has-[>[data-slot=radio-group]]:gap-3`
//! tightens the stack when what it holds is a radio group.
//!
//! That is why nesting is not decoration here. `<Field>` inside `<FieldGroup>`
//! inside `<FieldSet>` is what the class lists are written against, and a
//! wrapper div slipped between two of them silently turns rules off.
//!
//! It is also why this component does something [`Label`](super::label) does
//! not have to. When a caller writes a `class` override or a binding, a
//! rahti-ui component normally renders `<>@{element}<script/></>` — and a
//! fragment root is delimited by comments that the runtime raises into a real
//! `<pp-fragment>` at mount. A `<pp-fragment>` is `display: contents`, which is
//! invisible to layout and to a descendant selector but not to `>`: through
//! one, `*:w-full` styles the wrapper instead of the label inside it, and the
//! label loses its width the moment PulsePoint mounts.
//!
//! So a Field part is **always one literal element**, and the `<script>` that
//! makes a block mountable goes *inside* it rather than beside it — first
//! child everywhere except [`FieldSet`], where `<legend>` has to come first, so
//! it goes last.
//!
//! A `<script>` is `display: none`, so it is not a flex item, it takes no gap,
//! and `*:w-full` on it means nothing. The one thing it could cost is a
//! position — `nth-last-2:` and `last:` in [`DESCRIPTION`] count siblings — and
//! it does not, because the runtime takes the script out of the DOM once the
//! block is mounted. So it is a served-markup detail: measured on
//! `/components/field`, eight parts ship one and none of them still has one
//! after hydration.
//!
//! ## Put the width on a wrapper, not on the part that holds the repeats
//!
//! One constraint worth knowing before the first form, because it is invisible
//! until the page hydrates and it is the runtime's rather than this
//! component's.
//!
//! A `class` override makes a part's block *mounted* (see [`FieldProps::render`]
//! for why), and a mounted block re-renders what its `<slot />` holds. Adjacent
//! children that carry the same `pp-component` — which two `<Field>` tags always
//! do, since the name is the function's — are mis-keyed by that re-render: one
//! is duplicated and the last is dropped. It survives the server perfectly and
//! goes wrong in the browser.
//!
//! ```ignore
//! // Wrong: the group is mounted, and the fields under it are repeats.
//! <FieldGroup class="max-w-sm">
//!     <Field>…</Field>
//!     <Field>…</Field>
//! </FieldGroup>
//!
//! // Right: the group stays inert markup and the width belongs to the box
//! // around it — which a `w-full` FieldGroup fills anyway.
//! <div class="max-w-sm">
//!     <FieldGroup>
//!         <Field>…</Field>
//!         <Field>…</Field>
//!     </FieldGroup>
//! </div>
//! ```
//!
//! This is nothing to do with `data-slot` or with the class lists: any repeated
//! component tag under any mounted block behaves the same way. It shows up here
//! first because a layout wrapper with a width on it is the most natural thing
//! to write. `orientation="responsive"` is safe under the second form too — the
//! container query measures the FieldGroup, and a `w-full` group inside a
//! `max-w-xs` box is `max-w-xs` wide.
//!
//! An override on a part with *one* child, or on a part whose children are all
//! different components, is unaffected. So is a group whose repeats are
//! separated by something else — a [`FieldSeparator`] between two Fields is
//! enough.
//!
//! ## What this port changes, and why
//!
//! Three of shadcn's selectors ask Base UI questions that a server-rendered
//! native control cannot answer, and each is translated the way the rest of
//! this library translates them — see [`Checkbox`](super::checkbox), which
//! reads `data-[state=checked]:` as `checked:`.
//!
//! 1. **`has-data-checked:` becomes `has-[:checked]:`.** Base UI marks a
//!    checked box with a `data-checked` attribute; a native
//!    `<input type="checkbox">` answers `:checked`. Same declarations, same
//!    highlight on a selected choice card, native selector.
//! 2. **`[&>[role=checkbox],[role=radio]]:mt-px` becomes
//!    `[&>input[type=checkbox],&>input[type=radio]]:mt-px`.** Radix renders a
//!    `<button role="checkbox">`; this library renders the input HTML already
//!    has. The optical nudge is the same one.
//! 3. **`data-horizontal:` is a `@custom-variant`, and `src/app/globals.css`
//!    now carries shadcn's.** It is not Tailwind's: shadcn's Base UI style
//!    defines `data-horizontal` as `&:where([data-orientation="horizontal"])`
//!    and writes its components in the short spelling. Two class lists here
//!    read it — [`SEPARATOR_RULE`], where without the definition the rule
//!    renders as a ten-pixel bar instead of a one-pixel line, and
//!    [`DESCRIPTION`]. `data-vertical` comes with it, because the same class
//!    list writes both.
//!
//! `group-has-data-horizontal/field:text-balance` is worth one more sentence,
//! because it is easy to read as a rule about *this* field. `group-has-` asks
//! whether the group **contains** something horizontal, and a Field carries its
//! orientation on itself — so a plain horizontal field's own description is not
//! balanced. It fires for a description inside a Field that holds another,
//! horizontal Field, which is exactly when it fires in shadcn's own output.
//!
//! [`FieldSeparator`] draws its rule inline rather than through a
//! [`Separator`](super::separator::Separator) tag, because the rule is only
//! half of what that part is — the other half is a chip of text painted over
//! the middle of it. [`SEPARATOR_RULE`] is where the rule comes from, and it is
//! now [`separator::BASE`](super::separator::BASE) under its old name: kept
//! whole and alone before the port so there would be one place to hand over,
//! and this is the handover.
//!
//! ## What crosses the boundary
//!
//! Same rule as [`Button`](super::button) and [`Label`](super::label).
//! `onclick={…}` on a tag lands on the element untouched and is resolved when
//! it fires, late enough to reach the calling page's state. So does an
//! attribute *binding* — `hidden={!valid}` — because a component's root
//! attributes are evaluated where the tag was written.
//!
//! ## When the tag is not enough
//!
//! **[`FieldProps::render`], interpolated — for a field built in Rust.**
//!
//! ```ignore
//! @{field().bind("data-invalid", crate::components::rahti_ui::attrs::ui_expr!(data-invalid={!valid})).render(children)}
//! ```
//!
//! **The `*_variants` functions — the class list alone.** For a part the page
//! writes out itself — the markup is then the page's, with nothing between it
//! and the page's state.
//!
//! ```ignore
//! <div role="group" data-slot="field" data-orientation="vertical"
//!      class=@{field_variants(FieldOrientation::Vertical, "")}>
//!     <p data-slot="field-description" class=@{field_description_variants("")}>
//!         {password.length} " of 8 characters"
//!     </p>
//! </div>
//! ```
//!
//! A `data-slot` written by hand is not decoration when you do this: it is what
//! the rest of the family's selectors look for.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// A `<fieldset>`'s own utilities.
///
/// The two `has-` rules are why a set of checkboxes sits tighter than a set of
/// text inputs: a group of boxes is already a stack, so the outer gap comes
/// down. `data-slot="radio-group"` is what [`RadioGroup`](super::radio_group)
/// renders, so the second one fires in this library today; there is no
/// checkbox-group component yet and the first is carried for when there is.
const SET: &str = "flex flex-col gap-4 \
                   has-[>[data-slot=checkbox-group]]:gap-3 \
                   has-[>[data-slot=radio-group]]:gap-3";

/// A `<legend>`'s own utilities.
///
/// Both sizes are always in the list and `data-variant` picks between them —
/// shadcn's own arrangement, and the same one [`Select`](super::select) uses
/// for its `data-size`. Nothing has to be recomputed to switch, which is what
/// lets the variant be an attribute a binding can change.
const LEGEND: &str = "mb-1.5 font-medium \
                      data-[variant=label]:text-sm \
                      data-[variant=legend]:text-base";

/// A [`FieldGroup`]'s own utilities.
///
/// `@container/field-group` is the whole point of this element: it is the
/// container [`FieldOrientation::Responsive`] measures itself against, so a
/// responsive field laid out in a narrow column stays stacked and the same
/// field in a wide one goes side by side. Without a FieldGroup above it,
/// `@md/field-group:` never fires and a responsive field is a vertical one.
///
/// `*:data-[slot=field-group]:gap-4` is a nested group: a stack inside a stack
/// breathes less than the outer one.
const GROUP: &str = "group/field-group @container/field-group flex w-full flex-col gap-5 \
                     data-[slot=checkbox-group]:gap-3 \
                     *:data-[slot=field-group]:gap-4";

/// Everything a [`Field`] is before its orientation.
///
/// `group/field` is what the rest of the family names when it wants to know
/// something about the field around it — a label dims with
/// `group-data-[disabled=true]/field:opacity-50`, and a description asks
/// `group-has-…/field:`. `data-[invalid=true]:text-destructive` turns the whole
/// group red at once, which is why the invalid state is written on the Field
/// and not on each part.
const BASE: &str = "group/field flex w-full gap-2 data-[invalid=true]:text-destructive";

/// Label above control — the default, and what a form on a phone wants.
///
/// `*:w-full` is what makes the control fill the column, and
/// `[&>.sr-only]:w-auto` is the exception: a visually hidden label must not be
/// stretched, because `sr-only` sizes itself to nothing on purpose.
const VERTICAL: &str = "flex-col *:w-full [&>.sr-only]:w-auto";

/// Label beside control — for a switch, a checkbox, or a short answer.
///
/// `*:data-[slot=field-label]:flex-auto` gives the label the leftover width, so
/// the control sits at the end of the row. The `has-` pair is the checkbox
/// case: when the field holds a [`FieldContent`] the row is taller than one
/// line, so it aligns to the top and the box gets a one-pixel nudge to sit on
/// the first line's optical centre.
///
/// The nudge names native inputs. shadcn names `[role=checkbox]` and
/// `[role=radio]`, which is what Radix's `<button>` primitives carry; this
/// library renders `<input type="checkbox">`, and the module docs say why.
const HORIZONTAL: &str = "flex-row items-center \
                          has-[>[data-slot=field-content]]:items-start \
                          *:data-[slot=field-label]:flex-auto \
                          has-[>[data-slot=field-content]]:[&>input[type=checkbox],&>input[type=radio]]:mt-px";

/// Vertical until the [`FieldGroup`] around it is wide enough, then horizontal.
///
/// Every `@md/field-group:` here is measured against the *container*, not the
/// viewport, so the same field is stacked in a sidebar and side by side in a
/// page — without either knowing where it was placed. That only works inside a
/// [`FieldGroup`]; see [`GROUP`].
const RESPONSIVE: &str = "flex-col *:w-full \
                          @md/field-group:flex-row @md/field-group:items-center \
                          @md/field-group:*:w-auto \
                          @md/field-group:has-[>[data-slot=field-content]]:items-start \
                          @md/field-group:*:data-[slot=field-label]:flex-auto \
                          [&>.sr-only]:w-auto \
                          @md/field-group:has-[>[data-slot=field-content]]:[&>input[type=checkbox],&>input[type=radio]]:mt-px";

/// A [`FieldContent`]'s own utilities.
///
/// `flex-1` is what makes it take the row's leftover width in a horizontal
/// field, and `gap-0.5` is the tight stack a label and its hint want when they
/// are one block beside a control rather than two rows above it.
const CONTENT: &str = "group/field-content flex flex-1 flex-col gap-0.5 leading-snug";

/// [`Label`](super::label)'s class list, minus the three utilities
/// [`FIELD_LABEL`] replaces.
///
/// shadcn composes a FieldLabel out of `<Label className={…}>`, and `cn` runs
/// tailwind-merge over the result: `flex`, `gap-2` and `leading-none` are
/// written twice, and the second spelling wins. There is no Rust merge to do
/// that with, so the merge is done here, once, and `the_label_merge_is_shadcns`
/// proves it against [`label_variants`](super::label::label_variants) rather
/// than trusting the transcription.
const LABEL_KEPT: &str = "items-center text-sm font-medium select-none \
                          group-data-[disabled=true]:pointer-events-none \
                          group-data-[disabled=true]:opacity-50 \
                          peer-disabled:cursor-not-allowed peer-disabled:opacity-50";

/// What a [`FieldLabel`] adds on top of a plain label.
///
/// Two things beyond the tighter leading. `group-data-[disabled=true]/field:`
/// dims the label when the *field* is disabled, which is the pairing shadcn
/// prefers over the `peer-` one a bare label carries — it works with the label
/// written first, and `peer-` does not.
///
/// The rest is the **choice card**: a whole [`Field`] nested directly inside a
/// label turns that label into a bordered, rounded, full-width block, and it
/// highlights when the control inside is checked. `has-[:checked]:` is where
/// this port differs from shadcn's `has-data-checked:` — same rule, asked of a
/// native input. See the module docs.
const FIELD_LABEL: &str = "group/field-label peer/field-label flex w-fit gap-2 leading-snug \
                           group-data-[disabled=true]/field:opacity-50 \
                           has-[:checked]:border-primary/30 has-[:checked]:bg-primary/5 \
                           has-[>[data-slot=field]]:rounded-lg has-[>[data-slot=field]]:border \
                           *:data-[slot=field]:p-2.5 \
                           dark:has-[:checked]:border-primary/20 dark:has-[:checked]:bg-primary/10 \
                           has-[>[data-slot=field]]:w-full has-[>[data-slot=field]]:flex-col";

/// A [`FieldTitle`]'s own utilities — a label's look, on a `<div>`.
///
/// Deliberately not a `<label>`: inside a [`FieldContent`] whose control is
/// already named by something else, a second `<label>` would claim the control
/// and a screen reader would read the name twice. shadcn still stamps it
/// `data-slot="field-label"` so the family's selectors count it as one, and so
/// does this.
const TITLE: &str = "flex w-fit items-center gap-2 text-sm font-medium \
                     group-data-[disabled=true]/field:opacity-50";

/// A [`FieldDescription`]'s own utilities.
///
/// The three position rules are what keep a hint tucked against whatever it
/// explains without the call site measuring anything:
/// `[[data-variant=legend]+&]:-mt-1.5` pulls it up under a legend, `last:mt-0`
/// leaves it alone when it is the last thing in the field, and
/// `nth-last-2:-mt-1` tightens it when exactly one thing follows.
///
/// `group-has-data-horizontal/field:text-balance` balances the wrapping of a
/// description inside a field that *holds* a horizontal one — `group-has-` is a
/// question about descendants, and a Field carries its orientation on itself.
/// The short `data-horizontal` spelling is a `@custom-variant` in
/// `src/app/globals.css`; the module docs say where it came from.
const DESCRIPTION: &str = "text-left text-sm leading-normal font-normal text-muted-foreground \
                           group-has-data-horizontal/field:text-balance \
                           [[data-variant=legend]+&]:-mt-1.5 \
                           last:mt-0 nth-last-2:-mt-1 \
                           [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary";

/// A [`FieldSeparator`]'s own utilities — the box the rule and its text sit in.
///
/// `-my-2` is a negative margin that eats the stack's gap on both sides, so a
/// separator reads as a division between two runs rather than as a third item
/// in the list.
const SEPARATOR: &str = "relative -my-2 h-5 text-sm \
                         group-data-[variant=outline]/field-group:-mb-2";

/// shadcn's Separator, whole — [`separator::BASE`](super::separator::BASE),
/// under the name this file has always called it.
///
/// A [`FieldSeparator`] draws its own element rather than rendering a
/// [`Separator`](super::separator::Separator), because the rule is only half of
/// what that part is: the other half is a chip of text painted over the middle
/// of it, and the two have to agree about position and stacking. So the string
/// is borrowed and the element stays this file's.
///
/// It used to be a copy, kept unmixed with the positioning
/// [`SEPARATOR_RULE_POSITION`] adds so the port would have exactly one thing to
/// replace. This is that replacement, and the alias stays so nothing else here
/// had to move.
///
/// `data-horizontal:` is shadcn's `@custom-variant`, not Tailwind's, and
/// `src/app/globals.css` defines it. Without that definition the rule matches
/// nothing, `absolute inset-0 top-1/2` gives it half the box's height, and the
/// separator draws a ten-pixel bar instead of a line.
pub use super::separator::BASE as SEPARATOR_RULE;

/// What a [`FieldSeparator`] lays over [`SEPARATOR_RULE`]: the line, centred
/// across the box, behind the text.
const SEPARATOR_RULE_POSITION: &str = "absolute inset-0 top-1/2";

/// The chip a [`FieldSeparator`]'s text sits in.
///
/// `bg-background` is what punches the hole in the rule behind it — the line is
/// drawn edge to edge and this paints over the middle of it, which is why the
/// text has to be `relative` and the rule `absolute`.
const SEPARATOR_CONTENT: &str =
    "relative mx-auto block w-fit bg-background px-2 text-muted-foreground";

/// The rule a [`FieldSeparator`] stretches across itself.
///
/// A function rather than a constant because it is an [`Attrs`], and the two
/// places that draw one — [`FieldProps::render`] and the [`FieldSeparator`]
/// tag — have to agree on every attribute of it.
fn separator_rule() -> Attrs {
    Attrs::new()
        .set("data-slot", "separator")
        .set("data-orientation", "horizontal")
        .set("role", "separator")
        .set("aria-orientation", "horizontal")
        .set("class", cn(&[SEPARATOR_RULE, SEPARATOR_RULE_POSITION]))
}

/// The chip a [`FieldSeparator`]'s text sits in.
fn separator_chip() -> Attrs {
    Attrs::new()
        .set("data-slot", "field-separator-content")
        .set("class", SEPARATOR_CONTENT)
}

/// A [`FieldError`]'s own utilities.
const ERROR: &str = "text-sm font-normal text-destructive";

/// The list a [`FieldError`] builds when it is given more than one message.
const ERROR_LIST: &str = "ml-4 flex list-disc flex-col gap-1";

/// Which way a [`Field`] lays its parts out.
///
/// shadcn's three, kept the way shadcn keeps them: a class layer, plus a
/// `data-orientation` attribute the rest of the family can read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldOrientation {
    /// Label above control. The default, and what a narrow column wants.
    #[default]
    Vertical,
    /// Label beside control — a switch, a checkbox, a short answer.
    Horizontal,
    /// Vertical until the [`FieldGroup`] around it is wide enough. Needs one:
    /// the breakpoint is a container query, not a viewport one.
    Responsive,
}

impl FieldOrientation {
    /// The name shadcn writes into `data-orientation`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Responsive => "responsive",
        }
    }

    /// The class layer this orientation contributes.
    pub const fn layer(self) -> &'static str {
        match self {
            Self::Vertical => VERTICAL,
            Self::Horizontal => HORIZONTAL,
            Self::Responsive => RESPONSIVE,
        }
    }

    /// The [`slug`](Self::slug) read back, for the `orientation="…"` prop.
    ///
    /// An empty string is the default rather than an error, because that is
    /// what an unwritten prop arrives as. Anything else unrecognised is a
    /// mistake at the call site and trips a `debug_assert!` — the same rule
    /// [`SelectSize::parse`](super::select::SelectSize::parse) applies.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "vertical" => Self::Vertical,
            "horizontal" => Self::Horizontal,
            "responsive" => Self::Responsive,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a Field orientation. \
                     One of: vertical, horizontal, responsive."
                );
                Self::Vertical
            }
        }
    }
}

/// How big a [`FieldLegend`] reads.
///
/// Not a size prop: `legend` is a section heading and `label` is a legend that
/// has to sit at a label's weight, next to fields rather than above a whole
/// set. Both spellings are always in [`LEGEND`], and the attribute picks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldLegendVariant {
    /// `text-base` — a section heading.
    #[default]
    Legend,
    /// `text-sm` — a heading at a label's weight.
    Label,
}

impl FieldLegendVariant {
    /// The name shadcn writes into `data-variant`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Legend => "legend",
            Self::Label => "label",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `variant="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "legend" => Self::Legend,
            "label" => Self::Label,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a FieldLegend variant. One of: legend, label."
                );
                Self::Legend
            }
        }
    }
}

/// Where a part's mounting `<script>` goes when it needs one.
///
/// Inside the element either way — the module docs say why it is never a
/// sibling — and first unless the element is a `<fieldset>`, whose `<legend>`
/// has to be its first child.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptAt {
    First,
    Last,
}

/// Which of the ten a [`FieldProps`] is.
///
/// One props type serves the whole family, because the ten differ in three
/// facts and not in behaviour: the tag they render, the `data-slot` they carry,
/// and the class list they own. Keeping that a value rather than ten near
/// identical structs is what makes `class`, `attrs`, bindings and the scope
/// rule one implementation instead of ten.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Field,
    Set,
    Legend,
    Group,
    Content,
    Label,
    Title,
    Description,
    Separator,
    Error,
}

impl Part {
    /// The element this part renders.
    const fn tag(self) -> &'static str {
        match self {
            Self::Set => "fieldset",
            Self::Legend => "legend",
            Self::Label => "label",
            Self::Description => "p",
            _ => "div",
        }
    }

    /// The `data-slot` the rest of the family's selectors look for.
    ///
    /// [`Part::Title`] answers `field-label`, not `field-title`. That is
    /// shadcn's: a title *is* a label as far as the layout is concerned, and
    /// `*:data-[slot=field-label]:flex-auto` has to find it.
    const fn slot(self) -> &'static str {
        match self {
            Self::Field => "field",
            Self::Set => "field-set",
            Self::Legend => "field-legend",
            Self::Group => "field-group",
            Self::Content => "field-content",
            Self::Label | Self::Title => "field-label",
            Self::Description => "field-description",
            Self::Separator => "field-separator",
            Self::Error => "field-error",
        }
    }

    const fn script_at(self) -> ScriptAt {
        match self {
            Self::Set => ScriptAt::Last,
            _ => ScriptAt::First,
        }
    }
}

/// The class list a part renders with, before the caller's own.
///
/// A join, not a merge — [`no_layer_contradicts_another`](audit) holds every
/// one of these conflict-free, which is what lets [`cn`] be exact.
fn part_classes(part: Part, orientation: FieldOrientation) -> String {
    match part {
        Part::Field => cn(&[BASE, orientation.layer()]),
        Part::Set => SET.to_string(),
        Part::Legend => LEGEND.to_string(),
        Part::Group => GROUP.to_string(),
        Part::Content => CONTENT.to_string(),
        Part::Label => cn(&[LABEL_KEPT, FIELD_LABEL]),
        Part::Title => TITLE.to_string(),
        Part::Description => DESCRIPTION.to_string(),
        Part::Separator => SEPARATOR.to_string(),
        Part::Error => ERROR.to_string(),
    }
}

/// The class list shadcn's `cn(fieldVariants({ orientation }), className)`
/// returns.
///
/// Reach for this when the element has to belong to the page — a field whose
/// invalid state is the page's state, or whose children read it. Write the
/// `role="group"`, `data-slot="field"` and `data-orientation` alongside: the
/// family's other selectors look for them.
pub fn field_variants(orientation: FieldOrientation, class: &str) -> String {
    cn(&[&part_classes(Part::Field, orientation), class])
}

/// [`FieldSet`]'s class list, the caller's own included.
pub fn field_set_variants(class: &str) -> String {
    cn(&[SET, class])
}

/// [`FieldLegend`]'s class list, the caller's own included.
///
/// The variant is not in here — it is `data-variant`, which the caller has to
/// write alongside.
pub fn field_legend_variants(class: &str) -> String {
    cn(&[LEGEND, class])
}

/// [`FieldGroup`]'s class list, the caller's own included.
pub fn field_group_variants(class: &str) -> String {
    cn(&[GROUP, class])
}

/// [`FieldContent`]'s class list, the caller's own included.
pub fn field_content_variants(class: &str) -> String {
    cn(&[CONTENT, class])
}

/// [`FieldLabel`]'s class list, the caller's own included.
pub fn field_label_variants(class: &str) -> String {
    cn(&[
        &part_classes(Part::Label, FieldOrientation::Vertical),
        class,
    ])
}

/// [`FieldTitle`]'s class list, the caller's own included.
pub fn field_title_variants(class: &str) -> String {
    cn(&[TITLE, class])
}

/// [`FieldDescription`]'s class list, the caller's own included.
pub fn field_description_variants(class: &str) -> String {
    cn(&[DESCRIPTION, class])
}

/// [`FieldSeparator`]'s class list, the caller's own included — the box, not
/// the rule inside it.
pub fn field_separator_variants(class: &str) -> String {
    cn(&[SEPARATOR, class])
}

/// [`FieldError`]'s class list, the caller's own included.
pub fn field_error_variants(class: &str) -> String {
    cn(&[ERROR, class])
}

/// One Field part's props, with shadcn's defaults already in place.
///
/// Build one with [`field`], [`field_set`], [`field_label`] and the rest —
/// each picks the part, and everything after it is the same chain. Nothing
/// here is required.
///
/// Only `class` is a field of its own: it is the one thing that can conflict
/// with what the component chose, and resolving that is the browser's job.
/// Everything else a part carries — `id`, `for`, `data-invalid` — is an
/// ordinary attribute, and lives in `attrs` with the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldProps {
    part: Part,
    orientation: FieldOrientation,
    class: String,
    attrs: Attrs,
}

/// A default [`Field`]'s props — one field, laid out vertically.
pub fn field() -> FieldProps {
    FieldProps::of(Part::Field)
}

/// A default [`FieldSet`]'s props.
pub fn field_set() -> FieldProps {
    FieldProps::of(Part::Set)
}

/// A default [`FieldLegend`]'s props.
pub fn field_legend() -> FieldProps {
    FieldProps::of(Part::Legend)
}

/// A default [`FieldGroup`]'s props.
pub fn field_group() -> FieldProps {
    FieldProps::of(Part::Group)
}

/// A default [`FieldContent`]'s props.
pub fn field_content() -> FieldProps {
    FieldProps::of(Part::Content)
}

/// A default [`FieldLabel`]'s props.
pub fn field_label() -> FieldProps {
    FieldProps::of(Part::Label)
}

/// A default [`FieldTitle`]'s props.
pub fn field_title() -> FieldProps {
    FieldProps::of(Part::Title)
}

/// A default [`FieldDescription`]'s props.
pub fn field_description() -> FieldProps {
    FieldProps::of(Part::Description)
}

/// A default [`FieldSeparator`]'s props.
pub fn field_separator() -> FieldProps {
    FieldProps::of(Part::Separator)
}

/// A default [`FieldError`]'s props.
pub fn field_error() -> FieldProps {
    FieldProps::of(Part::Error)
}

impl FieldProps {
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

    /// Which way a [`Field`] lays its parts out.
    ///
    /// Only a Field has one. On any other part this is a mistake at the call
    /// site and trips a `debug_assert!` rather than changing anything.
    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        debug_assert!(
            self.part == Part::Field,
            "rahti-ui: only a Field has an orientation; `{}` does not.",
            self.part.slot()
        );
        self.orientation = orientation;
        self
    }

    /// A [`FieldLegend`]'s weight, as the `data-variant` [`LEGEND`] reads.
    pub fn variant(self, variant: FieldLegendVariant) -> Self {
        debug_assert!(
            self.part == Part::Legend,
            "rahti-ui: only a FieldLegend has a variant; `{}` does not.",
            self.part.slot()
        );
        self.attr("data-variant", variant.slug())
    }

    /// `data-invalid="true"` — what turns the whole group red.
    ///
    /// Written on the [`Field`] rather than on each part, because
    /// `data-[invalid=true]:text-destructive` in [`BASE`] colours everything
    /// inside it at once. The control still needs its own `aria-invalid`: this
    /// is what a reader sees, that is what a screen reader is told.
    ///
    /// `false` leaves the attribute off rather than writing
    /// `data-invalid="false"`, which the selector would not match but a
    /// stylesheet elsewhere might.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.attr("data-invalid", "true")
        } else {
            self
        }
    }

    /// `data-disabled="true"` — what dims a [`Field`]'s label.
    ///
    /// The `group-data-[disabled=true]/field:` rules a [`FieldLabel`] and a
    /// [`FieldTitle`] carry read this and nothing else. It is a presentation
    /// flag: the control inside still needs its own `disabled`, or it stays
    /// usable.
    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.attr("data-disabled", "true")
        } else {
            self
        }
    }

    /// `for="…"` — the id of the control a [`FieldLabel`] names.
    ///
    /// Named after JSX's `htmlFor` because `for` is a Rust keyword. The
    /// rendered attribute is `for`. Not the only way to associate a label: a
    /// control written *inside* it is named by it without either needing an id.
    pub fn html_for(self, id: &str) -> Self {
        self.attr("for", id)
    }

    /// The part's own id.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server: `attr("aria-live", "polite")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("disabled")`.
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
    /// Through a tag the same binding is a `{…}` prop: it lands on the root,
    /// and a component's root attributes are evaluated where the tag was
    /// written. This is its spelling for an element [`render`](Self::render)ed
    /// into a page's own block.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression:
    /// `on("click", "focusEmail()")`.
    ///
    /// The `on` prefix is optional. Use native lowercase event names, not the
    /// JSX spellings. Resolved when the event fires, which is late enough that
    /// it reaches the calling page's state either way.
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

    /// The class list this part renders with, the caller's own included.
    ///
    /// A join, not a merge: every layer here contradicts nothing else in it, so
    /// this is exact for everything except a caller's `.class(…)` override,
    /// which is the one thing that can conflict. That is resolved in the
    /// browser — see [`Self::render`].
    pub fn class_list(&self) -> String {
        cn(&[&part_classes(self.part, self.orientation), &self.class])
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
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new();

        // Ahead of `data-slot`, because a Field is a group before it is a
        // Field and `role` reads first in the served markup either way.
        if self.part == Part::Field {
            base = base.set("role", "group");
        }
        if self.part == Part::Error {
            base = base.set("role", "alert");
        }

        base = base.set("data-slot", self.part.slot());

        if self.part == Part::Field {
            base = base.set("data-orientation", self.orientation.slug());
        }
        // shadcn writes `data-variant` from a prop with a default, so a legend
        // that says nothing still carries one. `.variant(…)` overwrites this.
        if self.part == Part::Legend {
            base = base.set("data-variant", FieldLegendVariant::default().slug());
        }

        // A `.class(…)` override is the only thing here that can conflict with
        // what the component chose. The merge is `twMerge`'s, in the browser,
        // which is what the class attribute becomes when there is something to
        // resolve. With no override there is nothing to resolve, so `class`
        // stays a finished class list and the part is styled on arrival,
        // JavaScript or no JavaScript.
        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[
                    &part_classes(self.part, self.orientation),
                    &self.class,
                ]),
            )
        };

        base = base.merge(&self.attrs);

        base
    }

    /// The rendered element, with no boundary around it.
    ///
    /// Interpolating this into a block puts the part in that block's scope,
    /// which is what a binding written here needs. A `{…}` written among the
    /// *children*, though, is compiled where those children were written — so a
    /// part whose text is the page's state wants the matching `*_variants`
    /// function on native markup instead.
    ///
    /// ## What `.class(…)` costs
    ///
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    ///
    /// It also has to be in a block the runtime mounts, and a block is only
    /// mounted if it has a `<script>`. This adds one *inside* the element when
    /// it needs to, never beside it — the module docs say why, and what the one
    /// position it costs is.
    pub fn render(&self, children: Html) -> Html {
        if self.part == Part::Separator {
            return self.render_separator(children);
        }

        let attrs = self.attr_list();

        // Every arm below is one literal element, and the `<script>` is written
        // in this block rather than interpolated into it — a nested `html!`
        // would be a scope of its own, and only a script inside *this* block
        // makes *this* block mountable.
        match (self.part.tag(), self.needs_scope(), self.part.script_at()) {
            ("fieldset", false, _) => html! {
                <fieldset @{..attrs}>
                    <slot />
                </fieldset>
            },
            ("fieldset", true, _) => html! {
                <fieldset @{..attrs}>
                    <slot />
                    // Present so this block is mounted. See `needs_scope`.
                    // Last, because a `<legend>` has to be the first child.
                    <script>r##""##</script>
                </fieldset>
            },
            ("legend", false, _) => html! {
                <legend @{..attrs}>
                    <slot />
                </legend>
            },
            ("legend", true, _) => html! {
                <legend @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </legend>
            },
            ("label", false, _) => html! {
                <label @{..attrs}>
                    <slot />
                </label>
            },
            ("label", true, _) => html! {
                <label @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </label>
            },
            ("p", false, _) => html! {
                <p @{..attrs}>
                    <slot />
                </p>
            },
            ("p", true, _) => html! {
                <p @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </p>
            },
            (_, false, _) => html! {
                <div @{..attrs}>
                    <slot />
                </div>
            },
            (_, true, _) => html! {
                <div @{..attrs}>
                    <script>r##""##</script>
                    <slot />
                </div>
            },
        }
    }

    /// A separator is the one part with markup of its own: a rule stretched
    /// across the box, and — only when there is something to say — a chip
    /// painted over the middle of it.
    fn render_separator(&self, children: Html) -> Html {
        let content = !children.is_empty();

        // `data-content` is shadcn's, and it is written before the caller's
        // spread so `attrs` can still override it.
        let props = self
            .clone()
            .attr("data-content", if content { "true" } else { "false" });

        let attrs = props.attr_list();
        let rule = separator_rule();
        let chip = separator_chip();

        match (content, props.needs_scope()) {
            (false, false) => html! {
                <div @{..attrs}>
                    <div @{..rule}></div>
                </div>
            },
            (false, true) => html! {
                <div @{..attrs}>
                    <script>r##""##</script>
                    <div @{..rule}></div>
                </div>
            },
            (true, false) => html! {
                <div @{..attrs}>
                    <div @{..rule}></div>
                    <span @{..chip}>
                        <slot />
                    </span>
                </div>
            },
            (true, true) => html! {
                <div @{..attrs}>
                    <script>r##""##</script>
                    <div @{..rule}></div>
                    <span @{..chip}>
                        <slot />
                    </span>
                </div>
            },
        }
    }
}

/// What a [`FieldError`] puts inside itself: the children if there are any,
/// otherwise the messages, de-duplicated.
///
/// shadcn's `useMemo`, as a function. The rules are its: children win over
/// `errors`; one message renders as text; more than one renders as a bulleted
/// list; nothing renders as nothing, and the element itself is then not drawn
/// at all.
///
/// De-duplication is by message and keeps first-seen order — two validators
/// complaining about the same thing should say it once. Empty messages are
/// dropped rather than rendered as an empty bullet.
pub fn field_error_content(errors: &[&str], children: Html) -> Html {
    if !children.is_empty() {
        return children;
    }

    let mut unique: Vec<&str> = Vec::new();
    for message in errors {
        let message = message.trim();
        if !message.is_empty() && !unique.contains(&message) {
            unique.push(message);
        }
    }

    match unique.as_slice() {
        [] => Html::empty(),
        [only] => Html::escape(only),
        many => {
            let items = Html::concat(many.iter().map(|message| {
                html! {
                    <li>@{Html::escape(message)}</li>
                }
            }));
            html! {
                <ul class=@{ERROR_LIST}>@{items}</ul>
            }
        }
    }
}

/// One field: a label, a control, and whatever explains or corrects them.
///
/// ```ignore
/// <Field>
///     <FieldLabel r#for="email">"Email"</FieldLabel>
///     @{input().r#type("email").id("email").render()}
///     <FieldDescription>"We only use it to sign you in."</FieldDescription>
/// </Field>
///
/// <Field orientation="horizontal">
///     <FieldLabel r#for="alerts">"Email alerts"</FieldLabel>
///     @{switch_props().id("alerts").render()}
/// </Field>
/// ```
///
/// `role="group"` is what makes the parts one thing to a screen reader.
/// `invalid` turns everything inside red at once — the control still needs its
/// own `aria-invalid`, because that is the half assistive technology reads.
/// `disabled` is presentation only, and dims the label; the control needs its
/// own.
///
/// `orientation="responsive"` needs a [`FieldGroup`] above it. The breakpoint
/// is a container query measured against that element, so without one the field
/// stays vertical at every width.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Field(
    class: &str,
    orientation: &str,
    id: &str,
    invalid: bool,
    disabled: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field()
        .class(class)
        .orientation(FieldOrientation::parse(orientation))
        .invalid(invalid)
        .disabled(disabled);

    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all.
    if !id.is_empty() {
        props = props.id(id);
    }

    props = props.attrs(attrs);

    // `onclick={…}` and friends. `html!` hands every `{…}` prop over as the
    // expression the author wrote, and it goes onto the element unchanged.
    // Applied last, so a binding replaces a server value of the same name
    // rather than sitting beside it.
    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
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

/// A named run of fields.
///
/// ```ignore
/// <FieldSet>
///     <FieldLegend>"Billing address"</FieldLegend>
///     <FieldDescription>"Where the card statement goes."</FieldDescription>
///     <FieldGroup>…</FieldGroup>
/// </FieldSet>
/// ```
///
/// A real `<fieldset>`, so `disabled` on it disables every control inside —
/// HTML's own behaviour, not a class. Put the [`FieldLegend`] first: a legend
/// that is not a fieldset's first child is drawn as ordinary text.
#[component]
pub fn FieldSet(
    class: &str,
    id: &str,
    disabled: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field_set().class(class);

    if disabled {
        props = props.flag("disabled");
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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <fieldset @{..attrs}>
                <slot />
                <script>r##""##</script>
            </fieldset>
        }
    } else {
        html! {
            <fieldset @{..attrs}>
                <slot />
            </fieldset>
        }
    }
}

/// A [`FieldSet`]'s title.
///
/// ```ignore
/// <FieldLegend>"Billing address"</FieldLegend>
/// <FieldLegend variant="label">"Delivery window"</FieldLegend>
/// ```
///
/// `variant="label"` drops it to a label's size, for a set that sits among
/// fields rather than above a whole section. A [`FieldDescription`] written
/// straight after a legend tucks itself up under it — that is
/// `[[data-variant=legend]+&]:-mt-1.5` in [`DESCRIPTION`], and it needs the two
/// to be adjacent siblings.
#[component]
pub fn FieldLegend(
    class: &str,
    variant: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field_legend()
        .class(class)
        .variant(FieldLegendVariant::parse(variant));

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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <legend @{..attrs}>
                <script>r##""##</script>
                <slot />
            </legend>
        }
    } else {
        html! {
            <legend @{..attrs}>
                <slot />
            </legend>
        }
    }
}

/// The stack fields live in.
///
/// ```ignore
/// <FieldGroup>
///     <Field>…</Field>
///     <FieldSeparator />
///     <Field>…</Field>
/// </FieldGroup>
/// ```
///
/// Two jobs. It spaces the fields, and it is the container
/// `orientation="responsive"` measures itself against — so a responsive field
/// without one of these above it never turns horizontal, whatever the window
/// is doing.
#[component]
pub fn FieldGroup(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let mut props = field_group().class(class);

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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
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

/// A label and its hint as one block, beside a control.
///
/// ```ignore
/// <Field orientation="horizontal">
///     @{checkbox().id("terms").render()}
///     <FieldContent>
///         <FieldLabel r#for="terms">"Accept the terms"</FieldLabel>
///         <FieldDescription>"You can withdraw consent later."</FieldDescription>
///     </FieldContent>
/// </Field>
/// ```
///
/// This is what a horizontal field looks for: `has-[>[data-slot=field-content]]`
/// in [`HORIZONTAL`] switches the row from centred to top-aligned when it finds
/// one, and nudges the box down a pixel to sit on the first line.
#[component]
pub fn FieldContent(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field_content().class(class);

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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
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

/// The control's name.
///
/// ```ignore
/// <FieldLabel r#for="email">"Email"</FieldLabel>
/// ```
///
/// A [`Label`](super::label) with the field's own rules layered on: it dims
/// with the field rather than with a `peer` sibling, and a whole [`Field`]
/// nested directly inside it becomes a **choice card** — bordered, rounded,
/// full width, and highlighted while the control inside is checked:
///
/// ```ignore
/// <FieldLabel r#for="plan-pro">
///     <Field orientation="horizontal">
///         <FieldContent>
///             <FieldTitle>"Pro"</FieldTitle>
///             <FieldDescription>"Everything, and a seat for a friend."</FieldDescription>
///         </FieldContent>
///         @{radio_group_item().name("plan").value("pro").id("plan-pro").render()}
///     </Field>
/// </FieldLabel>
/// ```
///
/// The highlight is `has-[:checked]:` — a native `:checked` where shadcn asks
/// Base UI's `data-checked`. The card is `has-[>[data-slot=field]]:`, and the
/// `>` is not decoration: a div between the label and the Field turns it off.
///
/// `r#for` is spelled the way Rust makes it spellable; the rendered attribute
/// is `for`.
#[component]
pub fn FieldLabel(
    class: &str,
    r#for: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field_label().class(class);

    if !r#for.is_empty() {
        props = props.html_for(r#for);
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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <label @{..attrs}>
                <script>r##""##</script>
                <slot />
            </label>
        }
    } else {
        html! {
            <label @{..attrs}>
                <slot />
            </label>
        }
    }
}

/// A label's look without a label's meaning.
///
/// ```ignore
/// <FieldContent>
///     <FieldTitle>"Pro"</FieldTitle>
///     <FieldDescription>"Everything, and a seat for a friend."</FieldDescription>
/// </FieldContent>
/// ```
///
/// A `<div>`, not a `<label>`, and that is the whole point: inside a choice
/// card the [`FieldLabel`] already names the control, and a second `<label>`
/// would make a screen reader read the name twice. It still carries
/// `data-slot="field-label"`, because the layout counts it as one.
#[component]
pub fn FieldTitle(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let mut props = field_title().class(class);

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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
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

/// The hint under, or beside, the control.
///
/// ```ignore
/// <FieldDescription>"At least 8 characters."</FieldDescription>
/// <FieldDescription>"See the " <a href="/terms">"terms"</a> "."</FieldDescription>
/// ```
///
/// Position is in the class list rather than at the call site: it tucks up
/// under a [`FieldLegend`] it follows, and tightens when exactly one thing
/// comes after it. A link inside is underlined and turns primary on hover,
/// which is `[&>a]` and needs the anchor to be a direct child.
///
/// Point the control at it with `aria-describedby` and give it an `id` — the
/// styling is what a reader gets, and that is what a screen reader gets.
#[component]
pub fn FieldDescription(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field_description().class(class);

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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();

    if scope {
        html! {
            <p @{..attrs}>
                <script>r##""##</script>
                <slot />
            </p>
        }
    } else {
        html! {
            <p @{..attrs}>
                <slot />
            </p>
        }
    }
}

/// A rule between two runs of fields, with an optional word over it.
///
/// ```ignore
/// <FieldSeparator />
/// <FieldSeparator>"Or continue with"</FieldSeparator>
/// ```
///
/// The line is drawn edge to edge and the text is painted over the middle of it
/// in the page's background colour, which is how the gap in the rule appears
/// without anyone measuring the text. `data-content` says which of the two this
/// is, the way shadcn's does.
///
/// The rule itself is [`SEPARATOR_RULE`] inline rather than a
/// [`Separator`](super::separator::Separator) tag — the same class list, this
/// file's element, because a chip of text has to be painted over the middle of
/// it.
#[component]
pub fn FieldSeparator(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = field_separator().class(class);

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

    let content = !children.is_empty();
    // `data-content` is shadcn's, written before the caller's spread so an
    // `attrs` can still override it.
    props = props.attr("data-content", if content { "true" } else { "false" });

    let scope = props.needs_scope();
    let attrs = props.attr_list();
    let rule = separator_rule();
    let chip = separator_chip();

    // One literal element, and the chip has to come after the rule: it paints
    // over the middle of it. See the module docs for why the `<script>` is a
    // child rather than a sibling.
    match (content, scope) {
        (false, false) => html! {
            <div @{..attrs}>
                <div @{..rule}></div>
            </div>
        },
        (false, true) => html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <div @{..rule}></div>
            </div>
        },
        (true, false) => html! {
            <div @{..attrs}>
                <div @{..rule}></div>
                <span @{..chip}>
                    <slot />
                </span>
            </div>
        },
        (true, true) => html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <div @{..rule}></div>
                <span @{..chip}>
                    <slot />
                </span>
            </div>
        },
    }
}

/// What went wrong.
///
/// ```ignore
/// <FieldError>"Enter an email address."</FieldError>
/// <FieldError errors=@{&["Too short.", "Needs a digit."][..]} />
/// ```
///
/// `role="alert"` is what makes a screen reader announce it when it appears,
/// which is why an error with nothing to say renders **nothing at all** rather
/// than an empty box — an empty alert region is a thing that can announce
/// silence.
///
/// Children win over `errors`. One message renders as text, more than one as a
/// bulleted list, and duplicates are dropped — see [`field_error_content`],
/// which is shadcn's `useMemo` with the same rules.
///
/// Mark the [`Field`] `invalid` alongside: that is what colours the rest of the
/// group, and this only colours itself.
#[component]
pub fn FieldError(
    class: &str,
    id: &str,
    errors: &'static [&'static str],
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let content = field_error_content(errors, children);

    // shadcn returns `null`. An alert with nothing in it is worse than no
    // alert, so this returns nothing rather than an empty element — and the
    // `<script>` an override would have asked for is not written either, since
    // there is no element left for it to style.
    if content.is_empty() {
        return Html::empty();
    }

    let mut props = field_error().class(class);

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

    // One literal element, never `<>@{…}</>`: a fragment root is delimited by
    // comments the runtime raises into a `<pp-fragment>` at mount, and this
    // family's `>` and `nth-last` selectors cannot see through one. The
    // `<script>` a mounted block needs therefore goes inside the element. See
    // the module docs.
    let scope = props.needs_scope();
    let attrs = props.attr_list();
    let children = content;

    if scope {
        html! {
            <div @{..attrs}>
                <script>r##""##</script>
                <slot />
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

#[cfg(test)]
mod tests {
    use super::{
        Attrs, Field, FieldError, FieldLabel, FieldLegend, FieldLegendVariant, FieldOrientation,
        FieldProps, FieldSeparator, FieldSet, SEPARATOR_RULE, field, field_content,
        field_description, field_error, field_error_content, field_group, field_label,
        field_legend, field_separator, field_set, field_title, field_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_field_carries_shadcns_defaults() {
        let markup = field().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"role="group""#), "{markup}");
        assert!(markup.contains(r#"data-slot="field""#), "{markup}");
        assert!(
            markup.contains(r#"data-orientation="vertical""#),
            "{markup}"
        );
        assert!(markup.contains("group/field"), "{markup}");
        assert!(
            markup.contains("data-[invalid=true]:text-destructive"),
            "{markup}"
        );
        assert!(markup.contains("flex-col"), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
        // Nothing optional leaked an empty attribute.
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// Each of the ten renders the element shadcn renders, under the
    /// `data-slot` the rest of the family's selectors look for.
    #[test]
    fn every_part_is_the_element_shadcn_renders() {
        let cases: [(FieldProps, &str, &str); 9] = [
            (field(), "<div ", r#"data-slot="field""#),
            (field_set(), "<fieldset ", r#"data-slot="field-set""#),
            (field_legend(), "<legend ", r#"data-slot="field-legend""#),
            (field_group(), "<div ", r#"data-slot="field-group""#),
            (field_content(), "<div ", r#"data-slot="field-content""#),
            (field_label(), "<label ", r#"data-slot="field-label""#),
            // A title is a label to the layout, and shadcn says so too.
            (field_title(), "<div ", r#"data-slot="field-label""#),
            (
                field_description(),
                "<p ",
                r#"data-slot="field-description""#,
            ),
            (field_error(), "<div ", r#"data-slot="field-error""#),
        ];

        for (props, tag, slot) in cases {
            let markup = props.render(Html::empty()).into_test_string();
            assert!(markup.starts_with(tag), "{tag}: {markup}");
            assert!(markup.contains(slot), "{slot}: {markup}");
        }
    }

    /// Both `role`s shadcn writes, and the two elements that get one.
    #[test]
    fn the_roles_are_shadcns() {
        let group = field().render(Html::empty()).into_test_string();
        let alert = field_error()
            .render(Html::escape("Nope"))
            .into_test_string();
        let plain = field_group().render(Html::empty()).into_test_string();

        assert!(group.contains(r#"role="group""#), "{group}");
        assert!(alert.contains(r#"role="alert""#), "{alert}");
        assert!(!plain.contains("role="), "{plain}");
    }

    #[test]
    fn the_orientation_is_a_class_layer_and_an_attribute() {
        let horizontal = field()
            .orientation(FieldOrientation::Horizontal)
            .render(Html::empty())
            .into_test_string();

        assert!(
            horizontal.contains(r#"data-orientation="horizontal""#),
            "{horizontal}"
        );
        assert!(horizontal.contains("flex-row"), "{horizontal}");
        assert!(
            horizontal.contains("*:data-[slot=field-label]:flex-auto"),
            "{horizontal}"
        );
        // The vertical layer is gone, not merely outranked.
        assert!(!horizontal.contains(" flex-col"), "{horizontal}");

        let responsive = field()
            .orientation(FieldOrientation::Responsive)
            .render(Html::empty())
            .into_test_string();

        assert!(
            responsive.contains(r#"data-orientation="responsive""#),
            "{responsive}"
        );
        assert!(
            responsive.contains("@md/field-group:flex-row"),
            "{responsive}"
        );

        assert_eq!(FieldOrientation::parse(""), FieldOrientation::Vertical);
        assert_eq!(
            FieldOrientation::parse("horizontal"),
            FieldOrientation::Horizontal
        );
        assert_eq!(
            FieldOrientation::parse("responsive"),
            FieldOrientation::Responsive
        );
    }

    /// The three translations the module docs argue for, as assertions.
    #[test]
    fn the_port_asks_native_controls_native_questions() {
        // The class lists themselves: `&` and `>` are escaped on the way into
        // an attribute, and what is being checked here is the utility.
        let label = field_label().class_list();
        let horizontal = field()
            .orientation(FieldOrientation::Horizontal)
            .class_list();
        let description = field_description().class_list();

        // 1. Base UI's `data-checked` becomes the input's own `:checked`.
        assert!(label.contains("has-[:checked]:bg-primary/5"), "{label}");
        assert!(
            label.contains("dark:has-[:checked]:border-primary/20"),
            "{label}"
        );
        assert!(!label.contains("has-data-checked"), "{label}");

        // 2. Radix's `[role=checkbox]` becomes the input HTML already has.
        assert!(
            horizontal.contains("[&>input[type=checkbox],&>input[type=radio]]:mt-px"),
            "{horizontal}"
        );
        assert!(!horizontal.contains("[role=checkbox]"), "{horizontal}");

        // 3. And the short `data-horizontal` spelling, in both of the class
        //    lists that read it. `globals.css` is what defines it.
        assert!(
            description.contains("group-has-data-horizontal/field:text-balance"),
            "{description}"
        );
        assert!(
            SEPARATOR_RULE.contains("data-horizontal:h-px"),
            "{SEPARATOR_RULE}"
        );
    }

    /// A FieldLabel is a Label with a layer on top, and the layer's `flex`,
    /// `gap-2` and `leading-none` replacements are what a tailwind-merge would
    /// have resolved. This proves the merge rather than the transcription.
    #[test]
    fn the_label_merge_is_shadcns() {
        use crate::components::rahti_ui::label::label_variants;

        let merged = field_label().class_list();
        let label = label_variants("");
        let has = |list: &str, utility: &str| list.split_whitespace().any(|u| u == utility);

        // `flex` and `gap-2` are written on both sides and are the same
        // utility either way, so the merge is invisible in them: what it
        // decides is which `leading-` survives.
        for utility in label.split_whitespace() {
            assert_eq!(
                has(&merged, utility),
                utility != "leading-none",
                "`{utility}` in {merged}"
            );
        }

        assert!(has(&merged, "flex"), "{merged}");
        assert!(has(&merged, "gap-2"), "{merged}");
        assert!(has(&merged, "leading-snug"), "{merged}");

        // Written once each, not twice — a join that kept both copies would
        // still render, and would stop being shadcn's string.
        assert_eq!(merged.matches(" flex ").count(), 1, "{merged}");
        assert_eq!(merged.matches(" gap-2 ").count(), 1, "{merged}");
    }

    #[test]
    fn invalid_and_disabled_are_the_attributes_the_class_lists_read() {
        let markup = field()
            .invalid(true)
            .disabled(true)
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-invalid="true""#), "{markup}");
        assert!(markup.contains(r#"data-disabled="true""#), "{markup}");

        // `false` is the attribute's absence, not `="false"`.
        let plain = field()
            .invalid(false)
            .disabled(false)
            .render(Html::empty())
            .into_test_string();
        assert!(!plain.contains("data-invalid"), "{plain}");
        assert!(!plain.contains("data-disabled"), "{plain}");
    }

    #[test]
    fn a_legend_always_carries_a_variant() {
        let default = field_legend().render(Html::empty()).into_test_string();
        let label = field_legend()
            .variant(FieldLegendVariant::Label)
            .render(Html::empty())
            .into_test_string();

        assert!(default.contains(r#"data-variant="legend""#), "{default}");
        assert!(label.contains(r#"data-variant="label""#), "{label}");
        // Both sizes are always in the class list; the attribute picks.
        assert!(label.contains("data-[variant=legend]:text-base"), "{label}");
        assert!(label.contains("data-[variant=label]:text-sm"), "{label}");

        assert_eq!(FieldLegendVariant::parse(""), FieldLegendVariant::Legend);
        assert_eq!(
            FieldLegendVariant::parse("label"),
            FieldLegendVariant::Label
        );
    }

    /// The structural rule the module docs are built on: one literal element,
    /// never a fragment, whatever the caller asked for.
    #[test]
    fn an_override_never_wraps_the_element() {
        let markup = field()
            .class("max-w-xs")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
        assert!(!markup.contains("<!--pp:"), "{markup}");
    }

    /// And the script that makes it mountable goes inside — first, or last for
    /// a fieldset, whose `<legend>` has to come first.
    #[test]
    fn the_mounting_script_goes_inside_the_element() {
        // A marker the class attribute cannot also contain.
        let child = Html::from_raw("<b></b>".into());
        let field = field()
            .class("max-w-xs")
            .render(child.clone())
            .into_test_string();
        let set = field_set()
            .class("max-w-xs")
            .render(child)
            .into_test_string();

        assert!(!field.contains("<script>"), "{field}");

        assert!(!set.contains("<script>"), "{set}");
    }
    /// Class strings are composed on the server; frontend `twMerge` bindings resolve conflicts.
    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = field()
            .class("w-fit")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("w-fit"), "{markup}");
    }

    #[test]
    fn a_part_with_no_override_needs_no_runtime() {
        let props = field_group().id("signup");
        let markup = props.render(Html::empty()).into_test_string();

        assert!(!props.needs_scope());
        assert!(!markup.contains("twMerge"), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
        assert!(markup.contains(r#"class="group/field-group"#), "{markup}");

        assert!(!field_group().class("gap-2").needs_scope());
        assert!(
            field()
                .bind(
                    "data-invalid",
                    crate::components::rahti_ui::attrs::ui_expr!(data - invalid = { !valid })
                )
                .needs_scope()
        );
    }

    #[test]
    fn a_separator_draws_a_rule_and_only_a_chip_when_asked() {
        let plain = field_separator().render(Html::empty()).into_test_string();
        let titled = field_separator()
            .render(Html::escape("Or continue with"))
            .into_test_string();

        assert!(plain.contains(r#"data-content="false""#), "{plain}");
        assert!(plain.contains(r#"data-slot="separator""#), "{plain}");
        assert!(plain.contains("absolute inset-0 top-1/2"), "{plain}");
        assert!(!plain.contains("field-separator-content"), "{plain}");

        assert!(titled.contains(r#"data-content="true""#), "{titled}");
        assert!(
            titled.contains(r#"data-slot="field-separator-content""#),
            "{titled}"
        );
        assert!(titled.contains("Or continue with</span>"), "{titled}");
        // The chip paints over the rule, so it has to come after it.
        assert!(titled.find("data-slot=\"separator\"") < titled.find("field-separator-content"));
    }

    /// shadcn's `useMemo`, rule for rule.
    #[test]
    fn an_error_reads_its_messages_the_way_shadcn_does() {
        // Children win over `errors`.
        let children = field_error_content(&["ignored"], Html::escape("Told you"));
        assert_eq!(children.into_test_string(), "Told you");

        // One message is text.
        assert_eq!(
            field_error_content(&["Enter an email address."], Html::empty()).into_test_string(),
            "Enter an email address."
        );

        // More than one is a list, and duplicates are said once.
        let list = field_error_content(
            &["Too short.", "Needs a digit.", "Too short."],
            Html::empty(),
        )
        .into_test_string();
        assert!(list.starts_with("<ul "), "{list}");
        assert!(list.contains("list-disc"), "{list}");
        assert_eq!(list.matches("<li>").count(), 2, "{list}");

        // Nothing is nothing — including messages that are only whitespace.
        assert!(field_error_content(&[], Html::empty()).is_empty());
        assert!(field_error_content(&["", "  "], Html::empty()).is_empty());
    }

    /// An alert region with nothing in it can still announce. shadcn returns
    /// `null`; so does this.
    #[test]
    fn an_empty_error_renders_nothing_at_all() {
        let empty = FieldError(
            "",
            "",
            &[],
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert_eq!(empty, "");

        let said = FieldError(
            "",
            "email-error",
            &["Enter an email address."],
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(said.contains(r#"role="alert""#), "{said}");
        assert!(said.contains(r#"id="email-error""#), "{said}");
        assert!(said.contains("Enter an email address."), "{said}");
    }

    #[test]
    fn html_for_is_written_as_the_for_attribute() {
        let markup = field_label()
            .html_for("email")
            .render(Html::escape("Email"))
            .into_test_string();

        assert!(markup.contains(r#"for="email""#), "{markup}");
        assert!(!markup.contains("htmlFor"), "{markup}");
    }

    #[test]
    fn a_handler_is_serialized_as_a_pulsepoint_binding() {
        let markup = field_label()
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
        let markup = field()
            .attr("data-slot", "mine")
            .attr("aria-labelledby", "title")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"aria-labelledby="title""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = field()
            .attrs(Attrs::new().unset("role").set("aria-label", "Email"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("role="), "{markup}");
        assert!(markup.contains(r#"aria-label="Email""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = field_label()
            .html_for(r#"a" pp-onclick="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onclick=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn field_variants_is_the_components_own_class_list() {
        assert_eq!(
            field_variants(FieldOrientation::Horizontal, "max-w-xs"),
            field()
                .orientation(FieldOrientation::Horizontal)
                .class("max-w-xs")
                .class_list()
        );
    }

    /// The call form behind `<Field orientation="horizontal">…</Field>`.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let markup = Field(
            "",
            "horizontal",
            "email-field",
            true,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("x"),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"data-orientation="horizontal""#),
            "{markup}"
        );
        assert!(markup.contains(r#"data-invalid="true""#), "{markup}");
        assert!(markup.contains(r#"id="email-field""#), "{markup}");

        let set = FieldSet(
            "",
            "",
            true,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(set.starts_with("<fieldset "), "{set}");
        assert!(set.contains(r#"disabled="""#), "{set}");

        let legend = FieldLegend(
            "",
            "label",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Plan"),
        )
        .into_test_string();
        assert!(legend.contains(r#"data-variant="label""#), "{legend}");

        let separator = FieldSeparator(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Or"),
        )
        .into_test_string();
        assert!(separator.contains(r#"data-content="true""#), "{separator}");
    }

    /// Every prop left unwritten, which is what `<FieldLabel>"Email"</FieldLabel>`
    /// compiles to now that `html!` closes the props literal with
    /// `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_part() {
        let markup = FieldLabel(
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

        assert!(markup.contains(r#"data-slot="field-label""#), "{markup}");
        assert!(markup.contains("leading-snug"), "{markup}");
        assert!(markup.contains(">Email</label>"), "{markup}");
        assert!(!markup.contains(r#"for="""#), "{markup}");
        assert!(!markup.contains(r#"id="""#), "{markup}");
    }

    /// What `<Field data-invalid={!valid}>` compiles to. The expression arrives
    /// as source and goes onto the element unread.
    #[test]
    fn native_event_props_land_on_the_element() {
        let markup = Field(
            "",
            "",
            "",
            false,
            false,
            crate::rahti::Bindings::__from_html(&[("data-invalid", "!valid")]),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-invalid="{!valid}""#), "{markup}");
        // A binding is what makes the block need mounting.
        assert!(markup.contains("<script>"), "{markup}");
    }

    /// `FieldProps` is `Default`, which is what lets a tag omit every prop.
    #[test]
    fn props_default_is_a_plain_field() {
        assert_eq!(FieldProps::new(), FieldProps::default());
        assert_eq!(
            field().render(Html::empty()).into_test_string(),
            FieldProps::new().render(Html::empty()).into_test_string()
        );
    }
}

/// The layer-independence check behind every class string in this file.
///
/// A Field part builds its class list by *joining* layers rather than merging
/// them, which is only exact if no two utilities in the result compete. This is
/// what proves it, for all ten parts and all three orientations — the same
/// claim [`Button`](super::button) makes across its 48 variant×size pairs.
///
/// The interesting cases are the width ones. A vertical field writes `w-full`
/// on itself and `*:w-full` on its children; a responsive one adds
/// `@md/field-group:*:w-auto`. All three set a width and none of them clash,
/// because the variant prefix is what decides who is answering which question —
/// and the audit reads that the way Tailwind does.
#[cfg(test)]
mod audit {
    use super::{
        ERROR, ERROR_LIST, FieldOrientation, Part, SEPARATOR_CONTENT, SEPARATOR_RULE,
        SEPARATOR_RULE_POSITION, part_classes,
    };
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    const PARTS: [Part; 10] = [
        Part::Field,
        Part::Set,
        Part::Legend,
        Part::Group,
        Part::Content,
        Part::Label,
        Part::Title,
        Part::Description,
        Part::Separator,
        Part::Error,
    ];

    const ORIENTATIONS: [FieldOrientation; 3] = [
        FieldOrientation::Vertical,
        FieldOrientation::Horizontal,
        FieldOrientation::Responsive,
    ];

    #[test]
    fn no_layer_contradicts_another() {
        for part in PARTS {
            for orientation in ORIENTATIONS {
                let list = part_classes(part, orientation);
                let found = clashes(&list);

                assert!(
                    found.is_empty(),
                    "{} ({}): {} clashes:\n{}",
                    part.slot(),
                    orientation.slug(),
                    found.len(),
                    found.join("\n")
                );
            }
        }
    }

    /// The strings a part writes on markup it owns but does not take props
    /// for — a separator's rule and chip, an error's list.
    #[test]
    fn the_inner_markup_is_clash_free_too() {
        for list in [
            cn(&[SEPARATOR_RULE, SEPARATOR_RULE_POSITION]),
            SEPARATOR_CONTENT.to_string(),
            ERROR.to_string(),
            ERROR_LIST.to_string(),
        ] {
            let found = clashes(&list);
            assert!(found.is_empty(), "{} clashes in `{list}`", found.len());
        }
    }
}
