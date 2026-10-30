// rahti-ui: toggle-group
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Toggle Group — a port of shadcn/ui's `toggle-group`.
//!
//! shadcn's ToggleGroup is a Radix Root with `type="single" | "multiple"`, a
//! React context handing `variant`, `size` and `spacing` down to its items, and
//! roving-focus code so the arrow keys move between them.
//!
//! Here `type` is not a mode. It is the input type.
//!
//! | `type` | the input inside each item | what the browser then does |
//! | --- | --- | --- |
//! | `multiple` | `<input type="checkbox">` | any number on at once |
//! | `single` | `<input type="radio">`, one shared `name` | one at a time, arrow keys move it |
//!
//! That is the whole of the port's argument. A single-select toggle group *is* a
//! radio group wearing shadcn's paint: one selection, arrow keys, one tab stop
//! for the set, a value in the form post, and the right announcement — none of
//! it written, where Radix implements every line of it. A multiple-select group
//! is a row of [`Toggle`](super::toggle)s, which this library already had.
//!
//! ## The context has to be written down
//!
//! shadcn's Root puts `variant`, `size` and `spacing` in a React context and its
//! items read them. Children here arrive already rendered, so a group cannot
//! reach into them — the same wall [`RadioGroup`](super::radio_group) and
//! [`Select`](super::select) hit, and the same answer: what the context carried,
//! each item carries.
//!
//! Writing it three times per item would be miserable, so
//! [`ToggleGroupProps::item`] hands out an item with the group's own `type`,
//! `name`, `variant`, `size` and `spacing` already set:
//!
//! ```ignore
//! let group = toggle_group().single("view").variant(ToggleVariant::Outline);
//!
//! @{group.render(Html::concat([
//!     group.item().value("list").pressed(true).render(Html::escape("List")),
//!     group.item().value("grid").render(Html::escape("Grid")),
//! ]))}
//! ```
//!
//! The tags spell it out instead, which is what a tag is for:
//!
//! ```ignore
//! <ToggleGroup r#type="single" name="view" variant="outline">
//!     <ToggleGroupItem r#type="single" name="view" variant="outline" value="list" pressed>
//!         "List"
//!     </ToggleGroupItem>
//! </ToggleGroup>
//! ```
//!
//! ## Two layers a group takes back
//!
//! shadcn's item is `toggleVariants({variant, size})` with
//! `w-auto min-w-0 shrink-0 px-3` layered over it, and `twMerge` resolves the
//! two collisions: `min-w-0` beats the size's `min-w-9`, and `px-3` beats its
//! `px-2`.
//!
//! This library joins its layers rather than merging them, so the collision is
//! removed rather than resolved: an item takes [`ToggleSize::height`] — the only
//! part of the size that survives — and the group's own utilities supply the
//! rest. Same rendered class list, and `audit::no_utility_contradicts_another`
//! can prove there is nothing left to decide.
//!
//! ## `first:` and `last:` need real children
//!
//! At `spacing = 0` the items are joined into one strip, and the corners are
//! `first:rounded-l-md last:rounded-r-md` with `border-l-0` on all but the
//! first. Those are structural selectors: they match a real first and last child
//! of the group, so **an item must render as one element**, never as a fragment.
//!
//! [`Checkbox`](super::checkbox) and [`Label`](super::label) hit this with
//! `peer-` and answered it by keeping their no-script branch a literal element.
//! That is not enough here, because an item written as a [`ToggleGroupItem`] tag
//! and carrying a binding also needs a `<script>`, and a script beside the label
//! would itself be a child of the group — so `:last-child` would match the
//! script and the last item would keep a square right edge.
//!
//! So the tag's script goes *inside* the `<label>`, where it is still in the
//! block and is no longer a sibling. `<script>` is metadata content and a label
//! takes phrasing content, so this is valid HTML rather than a trick, and a
//! script is `display: none` so the label's `inline-flex` never sees it.
//!
//! [`ToggleGroupItemProps::render`] carries no script at all, which is the same
//! rule every `render` in this library follows and is worth stating because
//! getting it wrong is quiet: a block with a script is *mounted as its own
//! scope*, so an item rendered with one would compile a page's
//! `bind("checked", "view === 'list'")` against a scope that has no `view`. The
//! binding does not error at build time and the control simply never updates.
//!
//! ## What is not ported
//!
//! `spacing` is shadcn's, and so is the dead utility that comes with it:
//! `data-[spacing=default]:data-[variant=outline]:shadow-xs` on the Root never
//! matches, because `spacing` is a number and `data-spacing` is therefore never
//! the string `default`. It is carried here unchanged rather than quietly
//! dropped — this library copies shadcn's strings, including the ones shadcn is
//! not using.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding, lift_bindings};
use crate::components::rahti_ui::toggle::{ToggleSize, ToggleVariant};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, component, html};

/// The group: a strip, or a row with gaps in it.
///
/// shadcn's Root string, unchanged. `--gap` is an inline custom property the
/// server writes from `spacing`, which is how shadcn passes it too.
///
/// `audit::no_utility_contradicts_another` holds the join exact.
const GROUP_BASE: &str = "group/toggle-group flex w-fit items-center rounded-md \
                          gap-[--spacing(var(--gap))] \
                          data-[spacing=default]:data-[variant=outline]:shadow-xs";

/// What a group lays over one [`Toggle`](super::toggle)'s base and variant.
///
/// shadcn's second `cn` argument, plus its third, with two changes the module
/// docs argue: `focus:`/`focus-visible:` become `has-[…]` questions about the
/// item's input, and the size's `min-w-*` and `px-*` are not here to be beaten
/// because they were never joined in.
///
/// `z-10` works on a static element here because the group is a flex container,
/// and a flex item is painted as though it were positioned.
const ITEM_EXTRA: &str = "w-auto min-w-0 shrink-0 px-3 \
                          has-[:focus]:z-10 has-[:focus-visible]:z-10 \
                          data-[spacing=0]:rounded-none data-[spacing=0]:shadow-none \
                          data-[spacing=0]:first:rounded-l-md \
                          data-[spacing=0]:last:rounded-r-md \
                          data-[spacing=0]:data-[variant=outline]:border-l-0 \
                          data-[spacing=0]:data-[variant=outline]:first:border-l";

/// How the item's input is hidden without being taken away — the same one
/// [`Toggle`](super::toggle) uses, and for the same reason.
const INPUT: &str = "sr-only";

/// One at a time, or any number at once.
///
/// Not a mode this component implements: it is which input the items render, and
/// the browser does the rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleGroupType {
    /// `<input type="checkbox">` per item. Each carries its own `name`.
    #[default]
    Multiple,
    /// `<input type="radio">` per item, all sharing the group's `name`. One
    /// selection, and the arrow keys move it.
    Single,
}

impl ToggleGroupType {
    /// The `type` attribute each item's input gets.
    pub const fn input_type(self) -> &'static str {
        match self {
            Self::Multiple => "checkbox",
            Self::Single => "radio",
        }
    }

    /// What the group element is to assistive technology.
    ///
    /// A set of radios is a `radiogroup`, which is the association that makes a
    /// screen reader read "1 of 3" as the arrow keys move. A set of checkboxes
    /// is a plain `group`, which is what Radix uses for both.
    pub const fn role(self) -> &'static str {
        match self {
            Self::Multiple => "group",
            Self::Single => "radiogroup",
        }
    }

    /// The name shadcn writes in `type="…"`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Multiple => "multiple",
            Self::Single => "single",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `type="…"` prop.
    ///
    /// An empty string is the default rather than an error, because that is what
    /// an unwritten prop arrives as.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "multiple" => Self::Multiple,
            "single" => Self::Single,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not a ToggleGroup type. One of: single, multiple."
                );
                Self::Multiple
            }
        }
    }
}

/// Every utility the group carries, before the caller's own.
fn group_classes() -> String {
    GROUP_BASE.to_string()
}

/// Every utility one item carries, before the caller's own.
///
/// The toggle's base, its variant, the *height* of its size, and the group's own
/// layer. See the module docs for why the rest of the size is not here.
fn item_classes(variant: ToggleVariant, size: ToggleSize) -> String {
    cn(&[
        crate::components::rahti_ui::toggle::BASE,
        variant.class(),
        size.height(),
        ITEM_EXTRA,
    ])
}

/// The class list shadcn's `cn("group/toggle-group …", className)` returns.
pub fn toggle_group_variants(class: &str) -> String {
    cn(&[GROUP_BASE, class])
}

/// The class list shadcn's item builds out of `toggleVariants` and its own two
/// layers.
///
/// Reach for it when the element has to belong to the page — but note that the
/// class list alone is not a whole item: the `has-[…]:` utilities are asking
/// about an input, and the `data-[spacing=…]` ones about an attribute, so a
/// hand-written `<label>` has to carry both.
pub fn toggle_group_item_variants(variant: ToggleVariant, size: ToggleSize, class: &str) -> String {
    cn(&[&item_classes(variant, size), class])
}

/// A group's props, with shadcn's defaults already in place.
///
/// Build one with [`toggle_group`] and narrow it down a method at a time. The
/// group is also the factory for its items — see [`item`](Self::item), which is
/// this port's answer to shadcn's React context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToggleGroupProps {
    class: String,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: u32,
    kind: ToggleGroupType,
    name: String,
    attrs: Attrs,
}

/// A default group's props — the start of every chain.
pub fn toggle_group() -> ToggleGroupProps {
    ToggleGroupProps::default()
}

impl ToggleGroupProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the group's own, `className`-style.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// `0` joins the items into one strip, which is shadcn's default and where
    /// all the `first:`/`last:` corner work happens. Anything else is a gap in
    /// Tailwind's spacing scale, and each item keeps its own corners and border.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// One at a time, under this `name` — radios, and the browser's arrow keys.
    ///
    /// The name is required rather than optional because it is what makes the
    /// radios one group; without it they are separate controls that each latch
    /// on. See [`RadioGroup`](super::radio_group), which says the same thing at
    /// more length.
    pub fn single(mut self, name: &str) -> Self {
        self.kind = ToggleGroupType::Single;
        self.name = name.to_string();
        self
    }

    /// Any number at once — checkboxes. Each item posts under its own name, so
    /// the group needs none.
    pub fn multiple(mut self) -> Self {
        self.kind = ToggleGroupType::Multiple;
        self
    }

    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// `aria-labelledby="…"` — the id of the heading that names this set. A
    /// `role="group"` or `radiogroup` with no accessible name is a set of
    /// controls nobody is told the purpose of.
    pub fn labelled_by(self, id: &str) -> Self {
        self.attr("aria-labelledby", id)
    }

    /// An item carrying this group's `type`, `name`, `variant`, `size` and
    /// `spacing` — shadcn's React context, handed over rather than provided.
    ///
    /// Takes `&self` so a group can be built once and asked for as many items as
    /// it has.
    pub fn item(&self) -> ToggleGroupItemProps {
        ToggleGroupItemProps {
            class: String::new(),
            variant: self.variant,
            size: self.size,
            spacing: self.spacing,
            kind: self.kind,
            attrs: Attrs::new(),
            input_attrs: match self.kind {
                ToggleGroupType::Single => Attrs::new().set("name", self.name.clone()),
                ToggleGroupType::Multiple => Attrs::new(),
            },
        }
    }

    /// Any attribute on the group: `attr("aria-label", "Text alignment")`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute on the group.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off the group: `unset("role")`.
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

    /// An event handler, as a PulsePoint expression:
    /// `on("change", "setView(target.value)")`.
    ///
    /// `change` bubbles from the inputs inside, so one handler here serves the
    /// whole group — which is the nearest thing this port has to Radix's
    /// `onValueChange`.
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

    /// The class list this group renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[GROUP_BASE, &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order: the component's own first,
    /// the caller's spread last so it wins.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new()
            .set("role", self.kind.role())
            .set("data-slot", "toggle-group")
            .set("data-variant", self.variant.slug())
            .set("data-size", self.size.slug())
            .set("data-spacing", self.spacing.to_string())
            // `--gap` is what `gap-[--spacing(var(--gap))]` reads, and shadcn
            // writes it as an inline style too. Before the caller's attributes,
            // so a `style` of theirs replaces it rather than being dropped.
            .set("style", format!("--gap:{}", self.spacing));

        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&group_classes(), &self.class]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The rendered group, with no boundary around it.
    ///
    /// The children have to be the items and nothing else: the corner utilities
    /// are `:first-child` and `:last-child` selectors, and anything else in here
    /// takes a corner with it.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// One item's props.
///
/// Built by [`ToggleGroupProps::item`] rather than on its own, so the five
/// things shadcn's context carried arrive already set. [`toggle_group_item`]
/// exists for the call site that means to set them itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToggleGroupItemProps {
    class: String,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: u32,
    kind: ToggleGroupType,
    attrs: Attrs,
    input_attrs: Attrs,
}

/// A default item's props, with none of the group's context in them.
pub fn toggle_group_item() -> ToggleGroupItemProps {
    ToggleGroupItemProps::default()
}

impl ToggleGroupItemProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the item's own, `className`-style.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// Has to match the group's, since it is what the corner utilities read.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Which input this item renders. Has to match the group's.
    pub fn kind(mut self, kind: ToggleGroupType) -> Self {
        self.kind = kind;
        self
    }

    /// The control's name, on the input.
    ///
    /// In a `single` group this is the group's name and every item repeats it —
    /// that is what makes them one set of radios. In a `multiple` group each
    /// item has its own.
    pub fn name(self, name: &str) -> Self {
        self.input_attr("name", name)
    }

    pub fn id(self, id: &str) -> Self {
        self.input_attr("id", id)
    }

    /// What this item posts when it is on — Radix's item `value`, which is
    /// exactly what HTML calls it too.
    pub fn value(self, value: &str) -> Self {
        self.input_attr("value", value)
    }

    /// On, to start with — Radix's `defaultValue` for this item, written as the
    /// `checked` HTML has.
    pub fn pressed(self, pressed: bool) -> Self {
        if pressed {
            self.input_flag("checked")
        } else {
            self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        if disabled {
            self.input_flag("disabled")
        } else {
            self
        }
    }

    /// `aria-invalid="true"` on the input, which the
    /// `has-[[aria-invalid=true]]:` utilities in the toggle's base paint.
    pub fn invalid(self, invalid: bool) -> Self {
        if invalid {
            self.input_attr("aria-invalid", "true")
        } else {
            self
        }
    }

    /// An attribute on the `<label>`.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute on the `<label>`.
    pub fn flag(self, name: &str) -> Self {
        self.attr(name, "")
    }

    /// Take an attribute off the `<label>`.
    pub fn unset(mut self, name: &str) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).unset(name);
        self
    }

    /// An attribute on the `<input>`.
    pub fn input_attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.input_attrs = std::mem::take(&mut self.input_attrs).set(name, value);
        self
    }

    /// A bare boolean attribute on the `<input>`.
    pub fn input_flag(self, name: &str) -> Self {
        self.input_attr(name, "")
    }

    /// Take an attribute off the `<input>`.
    pub fn input_unset(mut self, name: &str) -> Self {
        self.input_attrs = std::mem::take(&mut self.input_attrs).unset(name);
        self
    }

    /// An attribute the browser runtime keeps up to date, on the input.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.input_attrs = std::mem::take(&mut this.input_attrs).bind(name, expr);
        this
    }

    /// An event handler on the input, as a PulsePoint expression.
    pub fn on(self, event: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let name = if event.starts_with("on") {
            event.to_string()
        } else {
            format!("on{event}")
        };
        self.bind(&name, expr)
    }

    /// Everything in `attrs`, laid over the label's.
    pub fn attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = merge(std::mem::take(&mut self.attrs), &attrs);
        self
    }

    /// Everything in `attrs`, laid over the input's.
    pub fn input_attrs(mut self, attrs: Attrs) -> Self {
        self.input_attrs = merge(std::mem::take(&mut self.input_attrs), &attrs);
        self
    }

    /// The class list this item's label renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[&item_classes(self.variant, self.size), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .chain(self.input_attrs.iter())
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    fn label_attr_list(&self) -> Attrs {
        let mut base = Attrs::new()
            .set("data-slot", "toggle-group-item")
            .set("data-variant", self.variant.slug())
            .set("data-size", self.size.slug())
            .set("data-spacing", self.spacing.to_string());

        base = if self.class.trim().is_empty() {
            base.set("class", self.class_list())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[
                    &item_classes(self.variant, self.size),
                    &self.class,
                ]),
            )
        };

        merge(base, &self.attrs)
    }

    fn input_attr_list(&self) -> Attrs {
        let base = Attrs::new()
            .set("type", self.kind.input_type())
            .set("data-slot", "toggle-group-item-input")
            .set("class", INPUT);

        merge(base, &self.input_attrs)
    }

    /// The rendered item: one `<label>`, and no script.
    ///
    /// The absence of the script is the point, and it is what makes this the
    /// method to reach for when a page drives the group. Every `render` in this
    /// library returns markup with no `<script>` in it, because a block that has
    /// one is *mounted as its own scope* — and a scope of its own is exactly
    /// what a binding on this element must not have. Interpolated into the
    /// page's block, these bindings compile against the page's state; given a
    /// script here they would compile against nothing and the page's `view`
    /// would be undefined.
    ///
    /// The [`ToggleGroupItem`] tag is the other path, and it is the one that
    /// needs a script — it puts it *inside* the label rather than beside it,
    /// because the group's corner utilities are `:first-child` and
    /// `:last-child` selectors and a sibling script would take a corner.
    pub fn render(&self, children: Html) -> Html {
        let label = self.label_attr_list();
        let input = self.input_attr_list();

        html! {
            <label @{..label}>
                <input @{..input} />
                <slot />
            </label>
        }
    }
}

/// One attribute set laid over another, last-wins, with `None` taking an
/// attribute away.
fn merge(base: Attrs, over: &Attrs) -> Attrs {
    base.merge(over)
}

/// A set of toggles, joined into a strip.
///
/// ```ignore
/// <ToggleGroup r#type="single" name="align" variant="outline">
///     <ToggleGroupItem r#type="single" name="align" variant="outline" value="left" pressed>
///         @{align_left()}
///     </ToggleGroupItem>
///     <ToggleGroupItem r#type="single" name="align" variant="outline" value="center">
///         @{align_center()}
///     </ToggleGroupItem>
/// </ToggleGroup>
/// ```
///
/// `type` decides what the items are: `single` makes them radios sharing
/// `name`, `multiple` makes them checkboxes. The browser does the rest — see the
/// module docs, which also say why each item repeats what shadcn's React context
/// would have handed it, and why [`ToggleGroupProps::item`] is the shorter way
/// to say the same thing.
///
/// What sits between the tags has to be the items and nothing else: the corner
/// utilities are `:first-child` and `:last-child` selectors.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn ToggleGroup(
    class: &str,
    r#type: &str,
    name: &str,
    variant: &str,
    size: &str,
    spacing: u32,
    id: &str,
    aria_labelledby: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let kind = ToggleGroupType::parse(r#type);

    let mut props = ToggleGroupProps::default()
        .class(class)
        .variant(ToggleVariant::parse(variant))
        .size(ToggleSize::parse(size))
        .spacing(spacing);

    props = match kind {
        ToggleGroupType::Single => props.single(name),
        ToggleGroupType::Multiple => props.multiple(),
    };

    if !id.is_empty() {
        props = props.id(id);
    }
    if !aria_labelledby.is_empty() {
        props = props.labelled_by(aria_labelledby);
    }

    props = props.attrs(attrs);

    for (name, expr) in bindings.iter() {
        props = props.bind(
            name,
            crate::components::rahti_ui::attrs::ClientBinding::forwarded(name, expr),
        );
    }

    // A caller's `{…}` props are the caller's expressions. Rendered without a
    // `<script>`, this block is compiled in its parent's scope, so they read
    // the caller's state the way a React prop would. Only a `class` override
    // with nothing of the caller's in it gets an empty script, so its
    // `{twMerge(…)}` resolves even where the page mounts no scope.
    if props.needs_scope() && bindings.is_empty() {
        let group = props.render(children);
        html! {
            <>
                @{group}
                <script>r##"
                    // Present so this block is mounted. See `needs_scope`.
                "##</script>
            </>
        }
    } else {
        let attrs = props.attr_list();
        html! {
            <div @{..attrs}>
                <slot />
            </div>
        }
    }
}

/// One item in a toggle group.
///
/// ```ignore
/// <ToggleGroupItem r#type="single" name="align" value="left" pressed>"Left"</ToggleGroupItem>
/// <ToggleGroupItem value="bold" variant="outline">@{bold_icon()}</ToggleGroupItem>
/// ```
///
/// Every prop shadcn's React context would have supplied — `type`, `name`,
/// `variant`, `size`, `spacing` — is written here instead, because children
/// arrive already rendered and a group cannot reach into them.
/// [`ToggleGroupProps::item`] is the builder that fills them in for you.
///
/// Renders one `<label>` around a hidden input, always — including when a
/// binding needs a script, which goes inside the label rather than beside it.
/// The group's corners depend on it.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn ToggleGroupItem(
    class: &str,
    r#type: &str,
    name: &str,
    id: &str,
    value: &str,
    variant: &str,
    size: &str,
    spacing: u32,
    pressed: bool,
    disabled: bool,
    invalid: bool,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = ToggleGroupItemProps::default()
        .class(class)
        .kind(ToggleGroupType::parse(r#type))
        .variant(ToggleVariant::parse(variant))
        .size(ToggleSize::parse(size))
        .spacing(spacing)
        .pressed(pressed)
        .disabled(disabled)
        .invalid(invalid);

    if !name.is_empty() {
        props = props.name(name);
    }
    if !id.is_empty() {
        props = props.id(id);
    }
    if !value.is_empty() {
        props = props.value(value);
    }

    props = props.attrs(attrs);

    // The input sits below the label, where the runtime evaluates in this
    // component's scope rather than the caller's, so a caller's bindings are
    // lifted: handlers onto the label, values onto it as props the input reads
    // back through `pp.props`. See [`lift_bindings`].
    let lifted = !bindings.is_empty();
    if lifted {
        let split = lift_bindings(bindings);
        props = props.attrs(split.root);
        props = props.input_attrs(split.inner);
    }

    // One literal `<label>` either way, unlike every other component in this
    // library: the script a binding or a class override needs goes inside it
    // rather than beside it,
    // because a sibling would be a child of the group and take a corner. A
    // `#[component]` has to end in `html!`, so the two branches are written out
    // here rather than deferred to [`ToggleGroupItemProps::render`], which does
    // the same thing.
    let label = props.label_attr_list();
    let input = props.input_attr_list();

    if lifted || props.needs_scope() {
        html! {
            <label @{..label}>
                <input @{..input} />
                <slot />
                <script>r##"
                    // Present so this block is mounted. See the module docs.
                "##</script>
            </label>
        }
    } else {
        html! {
            <label @{..label}>
                <input @{..input} />
                <slot />
            </label>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Attrs, ToggleGroup, ToggleGroupItem, ToggleGroupItemProps, ToggleGroupProps,
        ToggleGroupType, ToggleSize, ToggleVariant, toggle_group, toggle_group_item,
        toggle_group_item_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn a_default_group_carries_shadcns_defaults() {
        let markup = toggle_group().render(Html::empty()).into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"data-slot="toggle-group""#), "{markup}");
        assert!(markup.contains(r#"data-variant="default""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains(r#"data-spacing="0""#), "{markup}");
        assert!(markup.contains(r#"style="--gap:0""#), "{markup}");
        assert!(markup.contains("group/toggle-group"), "{markup}");
        assert!(markup.ends_with("</div>"), "{markup}");
    }

    /// The whole of the port: `type` is not a mode, it is the input type.
    #[test]
    fn the_type_is_the_input_type() {
        let group = toggle_group().single("view");
        let markup = group
            .item()
            .value("list")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"type="radio""#), "{markup}");
        assert!(markup.contains(r#"name="view""#), "{markup}");
        assert!(
            group
                .render(Html::empty())
                .into_test_string()
                .contains(r#"role="radiogroup""#)
        );

        let group = toggle_group().multiple();
        let markup = group
            .item()
            .value("bold")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"type="checkbox""#), "{markup}");
        // A multiple group's items each carry their own name, so none is given.
        assert!(!markup.contains("name="), "{markup}");
        assert!(
            group
                .render(Html::empty())
                .into_test_string()
                .contains(r#"role="group""#)
        );
    }

    /// shadcn's React context, handed over rather than provided.
    #[test]
    fn item_inherits_the_groups_context() {
        let group = toggle_group()
            .single("align")
            .variant(ToggleVariant::Outline)
            .size(ToggleSize::Lg)
            .spacing(2);

        let markup = group
            .item()
            .value("left")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-variant="outline""#), "{markup}");
        assert!(markup.contains(r#"data-size="lg""#), "{markup}");
        assert!(markup.contains(r#"data-spacing="2""#), "{markup}");
        assert!(markup.contains("border-input"), "{markup}");
        assert!(markup.contains("h-10"), "{markup}");
        assert!(markup.contains(r#"name="align""#), "{markup}");

        // And the same group can be asked for as many items as it has.
        assert_eq!(
            group
                .item()
                .value("a")
                .render(Html::empty())
                .into_test_string(),
            group
                .item()
                .value("a")
                .render(Html::empty())
                .into_test_string()
        );
    }

    /// The two layers shadcn resolves with `twMerge` and this port removes
    /// instead: an item keeps the height of its size and nothing else.
    #[test]
    fn an_item_takes_the_height_and_leaves_the_rest_of_the_size() {
        let classes = toggle_group_item_variants(ToggleVariant::Default, ToggleSize::Default, "");

        // shadcn's merged answer.
        assert!(classes.contains("h-9"), "{classes}");
        assert!(classes.contains("min-w-0"), "{classes}");
        assert!(classes.contains("px-3"), "{classes}");
        // Never joined in, so never in need of beating.
        assert!(!classes.contains("min-w-9"), "{classes}");
        assert!(!classes.contains("px-2"), "{classes}");
    }

    /// The corner utilities are structural selectors, so they have to be able
    /// to see a real first and last child.
    #[test]
    fn the_corner_utilities_are_carried_by_every_item() {
        let classes = toggle_group_item_variants(ToggleVariant::Outline, ToggleSize::Default, "");

        assert!(
            classes.contains("data-[spacing=0]:rounded-none"),
            "{classes}"
        );
        assert!(
            classes.contains("data-[spacing=0]:first:rounded-l-md"),
            "{classes}"
        );
        assert!(
            classes.contains("data-[spacing=0]:last:rounded-r-md"),
            "{classes}"
        );
        assert!(
            classes.contains("data-[spacing=0]:data-[variant=outline]:border-l-0"),
            "{classes}"
        );
    }

    /// An item is one element, and `render` never carries a script — a block
    /// with one is mounted as its own scope, and a binding written by a page
    /// has to compile against the page's.
    #[test]
    fn render_is_one_element_and_never_a_scope() {
        for markup in [
            toggle_group_item()
                .value("a")
                .render(Html::escape("A"))
                .into_test_string(),
            toggle_group_item()
                .value("a")
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ setView(target.value) }),
                )
                .render(Html::escape("A"))
                .into_test_string(),
        ] {
            assert!(markup.starts_with("<label "), "{markup}");
            assert!(markup.ends_with("</label>"), "{markup}");
            assert!(!markup.contains("<script"), "{markup}");
        }
    }

    /// The tag is the other path, and it is the one that needs a script — put
    /// inside the label, because a sibling would be a child of the group and
    /// take a corner.
    #[test]
    fn the_tags_script_lives_inside_the_label() {
        let bound = ToggleGroupItem(
            "",
            "single",
            "view",
            "",
            "list",
            "",
            "",
            0,
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[("onchange", "setView(target.value)")]),
            Attrs::new(),
            Html::escape("List"),
        )
        .into_test_string();

        assert!(bound.starts_with("<label "), "{bound}");
        assert!(bound.ends_with("</label>"), "{bound}");
        assert!(bound.contains("<script"), "{bound}");
        assert!(
            bound.find("<script").unwrap() < bound.find("</label>").unwrap(),
            "{bound}"
        );

        // And a plain one is still one element with nothing in it.
        let plain = ToggleGroupItem(
            "",
            "single",
            "view",
            "",
            "list",
            "",
            "",
            0,
            false,
            false,
            false,
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("List"),
        )
        .into_test_string();
        assert!(!plain.contains("<script"), "{plain}");
    }

    #[test]
    fn the_control_props_land_on_the_input() {
        let markup = toggle_group_item()
            .name("align")
            .id("align-left")
            .value("left")
            .pressed(true)
            .disabled(true)
            .invalid(true)
            .render(Html::empty())
            .into_test_string();

        let input = markup.split("<input ").nth(1).unwrap();
        assert!(input.contains(r#"name="align""#), "{markup}");
        assert!(input.contains(r#"id="align-left""#), "{markup}");
        assert!(input.contains(r#"value="left""#), "{markup}");
        assert!(input.contains(r#" checked="""#), "{markup}");
        assert!(input.contains(r#" disabled="""#), "{markup}");
        assert!(input.contains(r#"aria-invalid="true""#), "{markup}");
    }

    #[test]
    fn spacing_reaches_both_the_gap_and_the_corners() {
        let markup = toggle_group()
            .spacing(2)
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"style="--gap:2""#), "{markup}");
        assert!(markup.contains(r#"data-spacing="2""#), "{markup}");
        // The corner utilities are keyed to `0`, so at 2 they simply do not fire
        // and each item keeps its own rounding.
        let item = toggle_group()
            .spacing(2)
            .item()
            .render(Html::empty())
            .into_test_string();
        assert!(item.contains(r#"data-spacing="2""#), "{item}");
        assert!(item.contains("data-[spacing=0]:rounded-none"), "{item}");
    }

    /// `change` bubbles, so one handler on the group serves every item — the
    /// nearest thing this port has to Radix's `onValueChange`.
    #[test]
    fn a_handler_on_the_group_is_a_binding() {
        let markup = toggle_group()
            .on(
                "change",
                crate::components::rahti_ui::attrs::ui_handler!({ setView(target.value) }),
            )
            .render(Html::empty())
            .into_test_string();

        assert!(
            markup.contains(r#"pp-onchange="setView(target.value)""#),
            "{markup}"
        );
        assert!(
            toggle_group()
                .on(
                    "change",
                    crate::components::rahti_ui::attrs::ui_handler!({ x() })
                )
                .needs_scope()
        );
    }

    #[test]
    fn an_override_is_composed_as_class_data() {
        let markup = toggle_group()
            .class("w-full")
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("w-full"), "{markup}");
    }

    #[test]
    fn a_group_with_no_override_needs_no_runtime() {
        let props = toggle_group().single("view");

        assert!(!props.needs_scope());
        assert!(!props.item().value("a").needs_scope());
        assert!(!toggle_group().class("w-full").needs_scope());
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = toggle_group()
            .attr("data-slot", "mine")
            .attr("aria-label", "View")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-slot="mine""#), "{markup}");
        assert!(markup.contains(r#"aria-label="View""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = toggle_group()
            .attrs(Attrs::new().unset("role").set("aria-label", "View"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("role="), "{markup}");
        assert!(markup.contains(r#"aria-label="View""#), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = toggle_group_item()
            .value(r#"a" pp-onfocus="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    /// The call form behind the tags, and a whole group's worth of markup.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let items = Html::concat([
            ToggleGroupItem(
                "",
                "single",
                "align",
                "",
                "left",
                "outline",
                "",
                0,
                true,
                false,
                false,
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::escape("Left"),
            ),
            ToggleGroupItem(
                "",
                "single",
                "align",
                "",
                "center",
                "outline",
                "",
                0,
                false,
                false,
                false,
                crate::rahti::Bindings::default(),
                Attrs::new(),
                Html::escape("Center"),
            ),
        ]);

        let markup = ToggleGroup(
            "",
            "single",
            "align",
            "outline",
            "",
            0,
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            items,
        )
        .into_test_string();

        assert!(markup.contains(r#"role="radiogroup""#), "{markup}");
        assert_eq!(markup.matches(r#"type="radio""#).count(), 2, "{markup}");
        assert_eq!(markup.matches(r#"name="align""#).count(), 2, "{markup}");
        assert!(markup.contains(r#"value="left""#), "{markup}");
        assert!(markup.contains(">Left</label>"), "{markup}");
        // Exactly two children, so `:first-child` and `:last-child` mean what
        // the corner utilities need them to mean.
        assert_eq!(markup.matches("<label ").count(), 2, "{markup}");
    }

    /// Every prop left unwritten, which is what `<ToggleGroup>` compiles to now
    /// that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_group() {
        let markup = ToggleGroup(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            u32::default(),
            <&str>::default(),
            <&str>::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-slot="toggle-group""#), "{markup}");
        assert!(markup.contains(r#"role="group""#), "{markup}");
        assert!(markup.contains(r#"data-spacing="0""#), "{markup}");
    }

    #[test]
    fn types_round_trip_through_their_slugs() {
        assert_eq!(ToggleGroupType::parse(""), ToggleGroupType::Multiple);
        assert_eq!(ToggleGroupType::parse("single"), ToggleGroupType::Single);
        assert_eq!(ToggleGroupType::Single.input_type(), "radio");
        assert_eq!(ToggleGroupType::Single.role(), "radiogroup");
        assert_eq!(ToggleGroupType::Multiple.input_type(), "checkbox");
    }

    #[test]
    fn props_defaults_are_a_plain_group_and_a_plain_item() {
        assert_eq!(ToggleGroupProps::new(), ToggleGroupProps::default());
        assert_eq!(ToggleGroupItemProps::new(), ToggleGroupItemProps::default());
    }

    /// `<ToggleGroupItem checked={view === "list"} pp-onchange={setView(target.value)}>`:
    /// the input is below the label, so the handler goes onto the label and the
    /// value onto it as a prop the input reads back through `pp.props`.
    #[test]
    fn a_bound_items_bindings_are_lifted_onto_the_label() {
        let markup = ToggleGroupItem(
            "",
            "single",
            "view",
            "",
            "list",
            "",
            "",
            0,
            false,
            false,
            false,
            crate::rahti::Bindings::__from_html(&[
                ("checked", "view === \"list\""),
                ("onchange", "setView(target.value)"),
            ]),
            Attrs::new(),
            Html::escape("List"),
        )
        .into_test_string();

        let label = &markup[..markup.find("<input").unwrap()];
        let input = &markup[markup.find("<input").unwrap()..];
        assert!(
            label.contains(r#"pp-onchange="setView(target.value)""#),
            "{markup}"
        );
        assert!(label.contains("checked="), "{markup}");
        assert!(
            input.contains(r#"checked="{pp.props.checked}""#),
            "{markup}"
        );
        assert!(markup.ends_with("</script></label>"), "{markup}");
    }
}

/// The layer-independence check behind [`ITEM_EXTRA`].
///
/// This is the component the check was most needed for. shadcn's item layers
/// `min-w-0` and `px-3` over a size that already set `min-w-9` and `px-2`, and
/// leaves `twMerge` to sort it out — which this library cannot do on the server.
/// The audit is what says the collision is there, and [`item_classes`] is what
/// removes it by taking only [`ToggleSize::height`].
#[cfg(test)]
mod audit {
    use super::{GROUP_BASE, ITEM_EXTRA, ToggleSize, ToggleVariant, item_classes};
    use crate::components::rahti_ui::utils::audit::clashes;

    const VARIANTS: [ToggleVariant; 2] = [ToggleVariant::Default, ToggleVariant::Outline];
    const SIZES: [ToggleSize; 3] = [ToggleSize::Default, ToggleSize::Sm, ToggleSize::Lg];

    #[test]
    fn no_utility_contradicts_another() {
        let found = clashes(GROUP_BASE);
        assert!(found.is_empty(), "the group: {}", found.join("\n"));

        for variant in VARIANTS {
            for size in SIZES {
                let found = clashes(&item_classes(variant, size));

                assert!(
                    found.is_empty(),
                    "{}/{}: {} clashes:\n{}",
                    variant.slug(),
                    size.slug(),
                    found.len(),
                    found.join("\n")
                );
            }
        }
    }

    #[test]
    fn no_layer_contradicts_itself() {
        for (name, layer) in [("GROUP_BASE", GROUP_BASE), ("ITEM_EXTRA", ITEM_EXTRA)] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
