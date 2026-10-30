// rahti-ui: alert-dialog
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.5.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Alert Dialog — a port of shadcn/ui's `alert-dialog`.
//!
//! A modal that interrupts to ask something, and will not be waved away. Same
//! machinery as [`Dialog`](super::dialog) — a `<dialog>` opened with
//! `showModal()` — and the differences are all about refusing to be dismissed by
//! accident.
//!
//! ## What makes it an *alert* dialog
//!
//! | | [`Dialog`](super::dialog) | here |
//! | --- | --- | --- |
//! | `role` | `dialog`, implied by `showModal()` | **`alertdialog`**, written |
//! | Click the backdrop | closes | **does nothing** |
//! | Escape | closes | closes |
//! | Corner `X` | drawn unless hidden | **none, ever** |
//! | Focus on open | the first focusable thing | **[`AlertDialogCancel`]** |
//! | Buttons | whatever you write | [`AlertDialogAction`] and [`AlertDialogCancel`] |
//!
//! The backdrop row is the one that matters. A Dialog is a surface you can step
//! away from; an alert dialog is a question, and clicking beside a question is
//! not an answer. So this component simply does not register the handler its
//! sibling does — there is no prop for it, because a dismissible alert dialog is
//! a Dialog.
//!
//! Escape *is* still honoured, and deliberately: it is the platform's
//! documented way out of a modal, a keyboard user has no other, and it is
//! unambiguous in a way a stray click is not. It takes the same animated path as
//! a [`AlertDialogCancel`] press.
//!
//! ## What `showModal()` gives away for free
//!
//! The same list [`Sheet`](super::sheet) and [`Dialog`](super::dialog) work
//! through: the top layer instead of `AlertDialogPortal`, `::backdrop` instead
//! of `AlertDialogOverlay`, and the focus trap, the inert background and the
//! `Escape` key from the browser. Both of those exports dissolve — a page
//! selecting on `[data-slot=alert-dialog-overlay]` will find nothing.
//!
//! ## The two buttons are Buttons
//!
//! shadcn wraps its Action and Cancel primitives in a `<Button asChild>`, so
//! their look is `buttonVariants` and not a class list of their own. This port
//! does the same by calling [`button_variants`](super::button::button_variants)
//! — `default` for the action, `outline` for the cancel, both overridable. A
//! destructive confirmation is `variant="destructive"` on the action, which is
//! shadcn's own example.
//!
//! Both close the dialog. That is Radix's behaviour too, and it is why an
//! action that must *not* close — one that starts a request and waits — belongs
//! in a [`Dialog`](super::dialog) instead.
//!
//! ## `size` is a data attribute, and the layout reads it
//!
//! `data-size` plus `group/alert-dialog-content` is the whole of it, and three
//! parts select on it rather than taking a prop:
//!
//! ```text
//! data-[size=sm]:max-w-xs                                     the box
//! data-[size=default]:sm:max-w-lg                             the box
//! group-data-[size=sm]/…:grid-cols-2                          the footer
//! sm:group-data-[size=default]/…:place-items-start            the header
//! sm:group-data-[size=default]/…:row-span-2                   the media
//! ```
//!
//! A `sm` alert dialog stays centred at every width and puts its two buttons in
//! an even two-column row; a `default` one goes left-aligned from `sm` up. No
//! part is told which it is in.
//!
//! ## The media slot changes the header's grid
//!
//! [`AlertDialogHeader`] is a grid whose row template depends on whether it
//! contains an [`AlertDialogMedia`] — `has-data-[slot=alert-dialog-media]:` — and
//! at `sm` on a `default` box the icon moves *beside* the text instead of above
//! it, with the title taking `col-start-2`. Nothing is passed down; every part
//! of that is a selector, which is why the media has to be a real child of the
//! header.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <AlertDialog>
//!     <AlertDialogTrigger class=@{button_variants(ButtonVariant::Outline, ButtonSize::Default, "")}>
//!         "Delete account"
//!     </AlertDialogTrigger>
//!     <AlertDialogContent>
//!         <AlertDialogHeader>
//!             <AlertDialogTitle>"Are you absolutely sure?"</AlertDialogTitle>
//!             <AlertDialogDescription>
//!                 "This cannot be undone."
//!             </AlertDialogDescription>
//!         </AlertDialogHeader>
//!         <AlertDialogFooter>
//!             <AlertDialogCancel>"Cancel"</AlertDialogCancel>
//!             <AlertDialogAction variant="destructive">"Delete"</AlertDialogAction>
//!         </AlertDialogFooter>
//!     </AlertDialogContent>
//! </AlertDialog>
//! ```
//!
//! Give it an [`AlertDialogTitle`]. The script points `aria-labelledby` at it,
//! and a modal with no accessible name is announced as nothing.

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::button::{ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::utils::{cn, js_string};
use crate::rahti::{Html, Json, component, html};

/// The root: `display: contents`, and the reason it needs to exist at all.
///
/// Radix's `AlertDialog.Root` renders nothing. This has to render something — a
/// script has to live somewhere, and it has to be able to find its own trigger
/// and box — so it renders a box and then takes the box back out of the layout.
pub const ROOT: &str = "contents";

/// The overlay, as the pseudo-element the browser already draws.
///
/// shadcn's `AlertDialogOverlay` is `fixed inset-0 z-50 bg-black/50` plus a
/// fade. A modal `<dialog>` paints `::backdrop` behind itself with no help, so
/// the position, the inset and the z-index are all gone and what is left is the
/// colour and the fade. Same choice, same reasoning, as
/// [`Dialog`](super::dialog)'s.
pub const BACKDROP: &str = "backdrop:bg-black/50 backdrop:opacity-0 \
                            backdrop:transition-opacity backdrop:duration-200 \
                            open:backdrop:opacity-100";

/// What the UA stylesheet does to a `<dialog>` that shadcn's class list never
/// had to answer, because Radix renders a plain `<div>`.
///
/// Identical to [`Dialog`](super::dialog)'s and argued there at length:
/// `not-open:hidden` because the box's `grid` is an author rule that beats the
/// UA's `display: none`, and `right-auto bottom-auto` because the UA pins a
/// modal on all four insets where shadcn writes only two — which LTR papers over
/// and RTL does not.
///
/// The UA's `max-height` and `overflow` on `dialog:modal` are deliberately left
/// alone, so a long alert dialog scrolls inside itself rather than running off
/// the screen.
pub const DIALOG_RESET: &str = "not-open:hidden right-auto bottom-auto";

/// The box. shadcn's string, unchanged.
///
/// `group/alert-dialog-content` is the name the header, the footer, the title
/// and the media all read `data-size` through — see the module docs.
pub const CONTENT: &str = "group/alert-dialog-content fixed top-[50%] left-[50%] z-50 grid \
                           w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] \
                           gap-4 rounded-lg border bg-background p-6 shadow-lg duration-200 \
                           data-[size=sm]:max-w-xs \
                           data-[state=closed]:animate-out data-[state=closed]:fade-out-0 \
                           data-[state=closed]:zoom-out-95 \
                           data-[state=open]:animate-in data-[state=open]:fade-in-0 \
                           data-[state=open]:zoom-in-95 \
                           data-[size=default]:sm:max-w-lg";

/// The titled block at the top. shadcn's string, unchanged.
///
/// A grid rather than a flex column, because it has to place an
/// [`AlertDialogMedia`] beside the text at one size and above it at the other.
/// Centred on a phone and at `size="sm"`; left-aligned from `sm` up on a
/// `default` box.
pub const HEADER: &str = "grid grid-rows-[auto_1fr] place-items-center gap-1.5 text-center \
                          has-data-[slot=alert-dialog-media]:grid-rows-[auto_auto_1fr] \
                          has-data-[slot=alert-dialog-media]:gap-x-6 \
                          sm:group-data-[size=default]/alert-dialog-content:place-items-start \
                          sm:group-data-[size=default]/alert-dialog-content:text-left \
                          sm:group-data-[size=default]/alert-dialog-content:has-data-[slot=alert-dialog-media]:grid-rows-[auto_1fr]";

/// The buttons at the bottom. shadcn's string, unchanged.
///
/// `flex-col-reverse` under `sm`, so the action written *last* ends up on top
/// when the buttons stack and on the right when they sit in a row. At
/// `size="sm"` it becomes an even two-column grid at every width instead, which
/// is what makes a small alert dialog read as a pair of equal choices.
pub const FOOTER: &str = "flex flex-col-reverse gap-2 \
                          group-data-[size=sm]/alert-dialog-content:grid \
                          group-data-[size=sm]/alert-dialog-content:grid-cols-2 \
                          sm:flex-row sm:justify-end";

/// The question. shadcn's string, unchanged.
///
/// The long variant is what moves it beside a media icon rather than under it,
/// on a `default` box from `sm` up.
pub const TITLE: &str = "text-lg font-semibold \
                         sm:group-data-[size=default]/alert-dialog-content:group-has-data-[slot=alert-dialog-media]/alert-dialog-content:col-start-2";

/// The line under it, and what `aria-describedby` names.
pub const DESCRIPTION: &str = "text-sm text-muted-foreground";

/// An icon or an image above the question. shadcn's string, unchanged.
///
/// `*:[svg:not([class*='size-'])]:size-8` sizes whatever is put inside it,
/// unless the icon says otherwise.
pub const MEDIA: &str = "mb-2 inline-flex size-16 items-center justify-center rounded-md \
                         bg-muted \
                         sm:group-data-[size=default]/alert-dialog-content:row-span-2 \
                         *:[svg:not([class*='size-'])]:size-8";

/// How long the script waits for the exit animation before it really closes.
///
/// shadcn's `duration-200`, as a number.
pub const EXIT_MS: u32 = 200;

/// How wide the box is, and how three other parts arrange themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertDialogSize {
    #[default]
    Default,
    Sm,
}

impl AlertDialogSize {
    /// The name shadcn writes into `data-size`.
    ///
    /// There is no class layer for a size here — every utility that reads it
    /// does so through `data-[size=…]` or the content's group, so the attribute
    /// *is* the variant.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `size="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "sm" => Self::Sm,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not an AlertDialog size. One of: default, sm."
                );
                Self::Default
            }
        }
    }
}

/// Which of the ten a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Trigger,
    Content,
    Header,
    Footer,
    Title,
    Description,
    Media,
    Action,
    Cancel,
}

impl Part {
    /// The `data-slot` this part carries.
    ///
    /// Four are load-bearing. The script finds its box, its trigger and its two
    /// buttons by these strings, and the header's grid selects on the media's —
    /// so renaming one either turns the dialog inert or quietly changes its
    /// layout.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "alert-dialog",
            Self::Trigger => "alert-dialog-trigger",
            Self::Content => "alert-dialog-content",
            Self::Header => "alert-dialog-header",
            Self::Footer => "alert-dialog-footer",
            Self::Title => "alert-dialog-title",
            Self::Description => "alert-dialog-description",
            Self::Media => "alert-dialog-media",
            Self::Action => "alert-dialog-action",
            Self::Cancel => "alert-dialog-cancel",
        }
    }

    /// The element this part renders as.
    const fn element(self) -> &'static str {
        match self {
            Self::Trigger | Self::Action | Self::Cancel => "button",
            Self::Content => "dialog",
            Self::Title => "h2",
            Self::Description => "p",
            Self::Root | Self::Header | Self::Footer | Self::Media => "div",
        }
    }

    /// Is this part one of the two buttons, whose look is a Button's?
    const fn is_button(self) -> bool {
        matches!(self, Self::Action | Self::Cancel)
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part, variant: ButtonVariant, size: ButtonSize) -> String {
    match part {
        Part::Root => ROOT.to_string(),
        // shadcn's trigger is the bare primitive whose `asChild` hands the look
        // to a Button, and `button_variants(…)` at the call site is how that
        // crosses — the same arrangement as [`Dialog`](super::dialog)'s.
        Part::Trigger => String::new(),
        Part::Content => cn(&[DIALOG_RESET, CONTENT, BACKDROP]),
        Part::Header => HEADER.to_string(),
        Part::Footer => FOOTER.to_string(),
        Part::Title => TITLE.to_string(),
        Part::Description => DESCRIPTION.to_string(),
        Part::Media => MEDIA.to_string(),
        // These two *are* Buttons — shadcn wraps them in `<Button asChild>`, so
        // their class list is `buttonVariants` rather than one of their own.
        Part::Action | Part::Cancel => button_variants(variant, size, ""),
    }
}

/// The class list shadcn's `cn("group/alert-dialog-content fixed …", className)`
/// returns for the box.
///
/// Write `data-state`, `data-size` and `role="alertdialog"` alongside it, and
/// write it on a `<dialog>`: half of [`DIALOG_RESET`] is about that element's UA
/// rules, and every part inside reads `data-size` through the content's group.
pub fn alert_dialog_content_variants(class: &str) -> String {
    cn(&[
        &part_classes(Part::Content, ButtonVariant::Default, ButtonSize::Default),
        class,
    ])
}

/// [`AlertDialogHeader`]'s class list, the caller's own included.
pub fn alert_dialog_header_variants(class: &str) -> String {
    cn(&[HEADER, class])
}

/// [`AlertDialogFooter`]'s class list, the caller's own included.
pub fn alert_dialog_footer_variants(class: &str) -> String {
    cn(&[FOOTER, class])
}

/// [`AlertDialogTitle`]'s class list, the caller's own included.
pub fn alert_dialog_title_variants(class: &str) -> String {
    cn(&[TITLE, class])
}

/// [`AlertDialogDescription`]'s class list, the caller's own included.
pub fn alert_dialog_description_variants(class: &str) -> String {
    cn(&[DESCRIPTION, class])
}

/// [`AlertDialogMedia`]'s class list, the caller's own included.
///
/// Write `data-slot="alert-dialog-media"` alongside it, or the header's grid
/// never notices it and the icon is laid out as ordinary content.
pub fn alert_dialog_media_variants(class: &str) -> String {
    cn(&[MEDIA, class])
}

/// [`AlertDialogAction`]'s class list — a Button's, the caller's own included.
pub fn alert_dialog_action_variants(
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> String {
    button_variants(variant, size, class)
}

/// [`AlertDialogCancel`]'s class list — a Button's, the caller's own included.
pub fn alert_dialog_cancel_variants(
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> String {
    button_variants(variant, size, class)
}

/// One Alert Dialog part's props, with shadcn's defaults already in place.
///
/// Build one with [`alert_dialog`], [`alert_dialog_trigger`],
/// [`alert_dialog_content`], [`alert_dialog_header`], [`alert_dialog_footer`],
/// [`alert_dialog_title`], [`alert_dialog_description`], [`alert_dialog_media`],
/// [`alert_dialog_action`] or [`alert_dialog_cancel`] — each picks the part, and
/// everything after it is the same chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertDialogProps {
    part: Part,
    size: AlertDialogSize,
    variant: ButtonVariant,
    button_size: ButtonSize,
    class: String,
    attrs: Attrs,
}

impl Default for AlertDialogProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            size: AlertDialogSize::Default,
            variant: ButtonVariant::Default,
            button_size: ButtonSize::Default,
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

/// A default [`AlertDialog`]'s props — the start of every chain.
pub fn alert_dialog() -> AlertDialogProps {
    AlertDialogProps::of(Part::Root)
}

/// A default [`AlertDialogTrigger`]'s props.
pub fn alert_dialog_trigger() -> AlertDialogProps {
    AlertDialogProps::of(Part::Trigger)
}

/// A default [`AlertDialogContent`]'s props.
pub fn alert_dialog_content() -> AlertDialogProps {
    AlertDialogProps::of(Part::Content)
}

/// A default [`AlertDialogHeader`]'s props.
pub fn alert_dialog_header() -> AlertDialogProps {
    AlertDialogProps::of(Part::Header)
}

/// A default [`AlertDialogFooter`]'s props.
pub fn alert_dialog_footer() -> AlertDialogProps {
    AlertDialogProps::of(Part::Footer)
}

/// A default [`AlertDialogTitle`]'s props.
pub fn alert_dialog_title() -> AlertDialogProps {
    AlertDialogProps::of(Part::Title)
}

/// A default [`AlertDialogDescription`]'s props.
pub fn alert_dialog_description() -> AlertDialogProps {
    AlertDialogProps::of(Part::Description)
}

/// A default [`AlertDialogMedia`]'s props.
pub fn alert_dialog_media() -> AlertDialogProps {
    AlertDialogProps::of(Part::Media)
}

/// A default [`AlertDialogAction`]'s props — a `default` Button, as shadcn's is.
pub fn alert_dialog_action() -> AlertDialogProps {
    AlertDialogProps::of(Part::Action)
}

/// A default [`AlertDialogCancel`]'s props — an `outline` Button, as shadcn's
/// is.
pub fn alert_dialog_cancel() -> AlertDialogProps {
    AlertDialogProps {
        variant: ButtonVariant::Outline,
        ..AlertDialogProps::of(Part::Cancel)
    }
}

impl AlertDialogProps {
    fn of(part: Part) -> Self {
        Self {
            part,
            ..Self::default()
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Classes laid over the part's own, `className`-style.
    ///
    /// On a trigger this is the *whole* class list — shadcn's is a bare
    /// primitive whose `asChild` hands the look to a Button.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// How wide the box is, and how the header, footer, title and media arrange
    /// themselves inside it. The box's, and nothing else's — every other part
    /// reads it through the group.
    pub fn size(mut self, size: AlertDialogSize) -> Self {
        debug_assert!(
            self.part == Part::Content,
            "rahti-ui: only an AlertDialogContent has a size. \
             The other parts read `data-size` through its group."
        );
        self.size = size;
        self
    }

    /// `size="sm"` — a narrow box with two even columns of buttons.
    pub fn sm(self) -> Self {
        self.size(AlertDialogSize::Sm)
    }

    /// Which Button the action or the cancel looks like.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        debug_assert!(
            self.part.is_button(),
            "rahti-ui: only an AlertDialogAction or AlertDialogCancel has a Button variant."
        );
        self.variant = variant;
        self
    }

    /// `variant="destructive"` — shadcn's own spelling for a confirmation that
    /// deletes something.
    pub fn destructive(self) -> Self {
        self.variant(ButtonVariant::Destructive)
    }

    /// How big the action or the cancel is, as a Button size.
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        debug_assert!(
            self.part.is_button(),
            "rahti-ui: only an AlertDialogAction or AlertDialogCancel has a Button size."
        );
        self.button_size = size;
        self
    }

    /// The part's own id — for a page's own selector.
    ///
    /// Not needed for `aria-labelledby` or `aria-describedby`: the script points
    /// the box at its title and description at mount, from an id it derives with
    /// `pp.id()`, because pairing them needs both halves.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Any attribute, rendered on the server.
    pub fn attr(mut self, name: &str, value: impl Into<String>) -> Self {
        self.attrs = std::mem::take(&mut self.attrs).set(name, value);
        self
    }

    /// A bare boolean attribute: `flag("disabled")`.
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
    /// `data-state` and `open` both belong to the script here — it writes
    /// `closed`, waits out the exit animation and only then calls
    /// `dialog.close()` — so binding either is a fight the script wins.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression.
    ///
    /// A handler here runs *beside* the script's own rather than instead of it,
    /// which is what makes `on("click", …)` on an [`AlertDialogAction`] the way
    /// to actually do the thing being confirmed: the page's handler runs, and
    /// the script closes the dialog.
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

    /// This part's own utilities, before the caller's.
    fn own_classes(&self) -> String {
        part_classes(self.part, self.variant, self.button_size)
    }

    /// The class list this part renders with, the caller's own included.
    pub fn class_list(&self) -> String {
        cn(&[&self.own_classes(), &self.class])
    }

    /// Does anything here need a mounted PulsePoint scope?
    ///
    /// A root always does; it *is* the script.
    pub fn needs_scope(&self) -> bool {
        matches!(self.part, Part::Root)
            || self
                .attrs
                .iter()
                .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());

        base = match self.part {
            Part::Trigger | Part::Action | Part::Cancel => base.set("type", "button"),
            Part::Content => base
                // The one Radix writes and `showModal()` does not: a modal
                // `<dialog>` is announced as a `dialog`, and this is a question
                // that has to be announced as one.
                .set("role", "alertdialog")
                .set("data-size", self.size.slug())
                // The resting state, and the only one the server can know. A
                // `<dialog>` with no `open` attribute is `display: none` by UA
                // rule, so this is correct with or without the runtime.
                .set("data-state", "closed"),
            _ => base,
        };

        let own = self.own_classes();
        base = if self.class.trim().is_empty() {
            match own.is_empty() {
                true => base,
                false => base.set("class", own),
            }
        } else if own.is_empty() {
            // A trigger has no layer of its own, so the caller's class list is
            // already finished — no merge, and no `{twMerge(…)}` that would be
            // literal text until the block mounts.
            base.set("class", self.class.trim())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&own, &self.class]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The rendered element, with no boundary around it and no `<script>` in it.
    ///
    /// The root has no `render`: a root *is* its script, a script written in a
    /// method's `html!` is a block of its own, and the `pp-ref` that script needs
    /// would never resolve there. Use the [`AlertDialog`] tag.
    pub fn render(&self, children: Html) -> Html {
        debug_assert!(
            !matches!(self.part, Part::Root),
            "rahti-ui: an AlertDialog root is its script, and a script in a \
             method's `html!` is a block of its own. Use the <AlertDialog> tag."
        );

        let attrs = self.attr_list();

        match self.part.element() {
            "button" => html! {
                <button @{..attrs}>@{children}</button>
            },
            "h2" => html! {
                <h2 @{..attrs}>@{children}</h2>
            },
            "p" => html! {
                <p @{..attrs}>@{children}</p>
            },
            "dialog" => html! {
                <dialog @{..attrs}>@{children}</dialog>
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

/// A question that has to be answered, and the trigger that asks it.
///
/// ```ignore
/// <AlertDialog>
///     <AlertDialogTrigger class=@{button_variants(ButtonVariant::Outline, ButtonSize::Default, "")}>
///         "Delete account"
///     </AlertDialogTrigger>
///     <AlertDialogContent>
///         <AlertDialogHeader>
///             <AlertDialogTitle>"Are you absolutely sure?"</AlertDialogTitle>
///             <AlertDialogDescription>"This cannot be undone."</AlertDialogDescription>
///         </AlertDialogHeader>
///         <AlertDialogFooter>
///             <AlertDialogCancel>"Cancel"</AlertDialogCancel>
///             <AlertDialogAction variant="destructive">"Delete"</AlertDialogAction>
///         </AlertDialogFooter>
///     </AlertDialogContent>
/// </AlertDialog>
/// ```
///
/// This element is `display: contents`, so it is not in the layout — the trigger
/// inside it sits exactly where it was written.
///
/// ## What the script does, and what it deliberately does not
///
/// It opens the box, closes it from either button, locks the body's scroll,
/// points `aria-labelledby` and `aria-describedby` at the title and description,
/// focuses the [`AlertDialogCancel`] when it opens, and holds the box on screen
/// for the length of its exit animation.
///
/// What it does **not** register is a backdrop-click handler. That is the whole
/// difference from [`Dialog`](super::dialog): clicking beside a question is not
/// an answer. `Escape` is still honoured — it is the platform's documented way
/// out and a keyboard user's only one — and takes the same animated path as a
/// cancel.
///
/// None of its state is `pp.state`, for the reason
/// [`ScrollArea`](super::scroll_area) gives: a state change re-renders the block
/// and hands the attribute reconciler a chance to put the server's
/// `data-state="closed"` back over the one the script just wrote.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialog(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog().class(class), id, attrs, bindings);
    let root = props.attr_list();
    let exit = EXIT_MS;
    let open_state = Json("open");
    let closed_state = Json("closed");

    html! {
        <div @{..root}>
            <slot />
            <span data-slot="alert-dialog-anchor" hidden=@{true} pp-ref={anchorRef}></span>
            <script>r##"
                const EXIT_MS = @{exit};
                const OPEN = @{open_state};
                const CLOSED = @{closed_state};

                const anchorRef = pp.ref(null);
                const uid = pp.id();

                pp.effect(() => {
                    const anchor = anchorRef.current;
                    if (!anchor) return;

                    const root = anchor.closest("[data-slot=alert-dialog]");
                    if (!root) return;

                    // Everything belonging to *this* alert dialog, so one
                    // written inside another's content keeps its own parts.
                    const own = (slot) =>
                        Array.from(root.querySelectorAll("[data-slot=" + slot + "]"))
                            .filter((el) => el.closest("[data-slot=alert-dialog]") === root);

                    const box = own("alert-dialog-content")[0];
                    if (!box) return;
                    const triggers = own("alert-dialog-trigger");
                    const cancels = own("alert-dialog-cancel");
                    const actions = own("alert-dialog-action");

                    // The accessible name and description. Both need an id on
                    // the other half, which is why neither part could write it.
                    const title = box.querySelector("[data-slot=alert-dialog-title]");
                    const description = box.querySelector("[data-slot=alert-dialog-description]");
                    if (title) {
                        if (!title.id) title.id = uid + "-title";
                        box.setAttribute("aria-labelledby", title.id);
                    }
                    if (description) {
                        if (!description.id) description.id = uid + "-description";
                        box.setAttribute("aria-describedby", description.id);
                    }

                    let timer = 0;

                    // A count on `<body>` rather than a value each dialog saves
                    // and puts back, because these nest and because a Dialog may
                    // already hold the lock. Same two keys `dialog.rs` uses, so
                    // the two components share one count instead of fighting.
                    const LOCKS = "rahtiScrollLocks";
                    const SAVED = "rahtiScrollOverflow";

                    const lock = () => {
                        const body = document.body;
                        const n = (Number(body.dataset[LOCKS]) || 0) + 1;
                        body.dataset[LOCKS] = n;
                        if (n > 1) return;
                        body.dataset[SAVED] = body.style.overflow;
                        body.style.overflow = "hidden";
                    };

                    const unlock = () => {
                        const body = document.body;
                        const n = (Number(body.dataset[LOCKS]) || 0) - 1;
                        if (n > 0) {
                            body.dataset[LOCKS] = n;
                            return;
                        }
                        delete body.dataset[LOCKS];
                        body.style.overflow = body.dataset[SAVED] || "";
                        delete body.dataset[SAVED];
                    };

                    const open = () => {
                        if (box.open) return;
                        clearTimeout(timer);
                        box.setAttribute("data-state", OPEN);
                        box.showModal();
                        lock();
                        // Radix puts focus on the cancel, so the safe answer is
                        // the one under the fingers. `showModal` has already
                        // focused something by now, so this is a correction
                        // rather than the first move.
                        const safe = cancels[0] || actions[0];
                        if (safe) safe.focus();
                    };

                    const finish = () => {
                        clearTimeout(timer);
                        timer = 0;
                        if (box.open) box.close();
                    };

                    // `dialog.close()` hides the element the instant it is
                    // called, so an exit animation has to run *before* it: write
                    // `closed`, let `zoom-out-95` play, and close when it ends.
                    // `animationend` is the accurate signal and the timer is the
                    // net for a browser that never fires one.
                    const close = () => {
                        if (!box.open) return;
                        if (box.getAttribute("data-state") === CLOSED) return;
                        box.setAttribute("data-state", CLOSED);
                        clearTimeout(timer);
                        timer = setTimeout(finish, EXIT_MS + 50);
                    };

                    const animated = (event) => {
                        if (event.target !== box) return;
                        if (box.getAttribute("data-state") !== CLOSED) return;
                        finish();
                    };

                    // Escape is the browser's, and it would close the dialog
                    // outright — so it is intercepted and sent down the animated
                    // path instead. It is honoured rather than swallowed: it is
                    // the platform's documented way out of a modal, and the only
                    // one a keyboard user has.
                    const cancel = (event) => {
                        event.preventDefault();
                        close();
                    };

                    const restore = () => {
                        unlock();
                        box.setAttribute("data-state", CLOSED);
                        // One of these written inside another is a DOM
                        // descendant, so closing this box would only hide it —
                        // it would stay `open`, keep its lock, and still be
                        // there next time.
                        const nested = box.querySelectorAll(
                            "dialog[data-slot=alert-dialog-content],dialog[data-slot=dialog-content]"
                        );
                        for (const inner of nested) if (inner.open) inner.close();
                    };

                    // No backdrop-click handler, and that is the component.
                    // `Dialog` registers one; an alert dialog must not, because
                    // clicking beside a question is not an answer to it.

                    for (const trigger of triggers) trigger.addEventListener("click", open);
                    for (const button of cancels) button.addEventListener("click", close);
                    for (const button of actions) button.addEventListener("click", close);
                    box.addEventListener("cancel", cancel);
                    box.addEventListener("close", restore);
                    box.addEventListener("animationend", animated);

                    return () => {
                        clearTimeout(timer);
                        if (box.open) box.close();
                        for (const trigger of triggers) trigger.removeEventListener("click", open);
                        for (const button of cancels) button.removeEventListener("click", close);
                        for (const button of actions) button.removeEventListener("click", close);
                        box.removeEventListener("cancel", cancel);
                        box.removeEventListener("close", restore);
                        box.removeEventListener("animationend", animated);
                    };
                }, []);
            "##</script>
        </div>
    }
}

/// The thing that asks the question.
///
/// shadcn's trigger has no class list of its own — its `asChild` was always
/// meant to hand the look to a Button — so `button_variants(…)` here is how that
/// crosses.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogTrigger(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog_trigger().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <button @{..attrs}>
            <slot />
        </button>
    }
}

/// The box itself — a `<dialog role="alertdialog">`.
///
/// ```ignore
/// <AlertDialogContent>…</AlertDialogContent>
/// <AlertDialogContent size="sm">…</AlertDialogContent>
/// ```
///
/// It carries `data-state="closed"` from the server and a `<dialog>` with no
/// `open` attribute is `display: none` by UA rule, so a page with no JavaScript
/// shows nothing rather than a box stuck open.
///
/// There is no corner `X` and no `hide_close`: the two buttons in the footer are
/// the only ways out besides `Escape`, which is the point of the component.
/// `size` is written as `data-size` and four other parts read it through this
/// element's group.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogContent(
    class: &str,
    size: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        alert_dialog_content()
            .class(class)
            .size(AlertDialogSize::parse(size)),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    html! {
        <dialog @{..attrs}>
            <slot />
        </dialog>
    }
}

/// The titled block at the top.
///
/// A grid, because it has to place an [`AlertDialogMedia`] above the text on a
/// phone and beside it on a wide `default` box. The media has to be a real child
/// for either to happen — the header selects on it with `has-data-[slot=…]`.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogHeader(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog_header().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// The buttons at the bottom.
///
/// Write the [`AlertDialogCancel`] first and the [`AlertDialogAction`] last: the
/// row is `flex-col-reverse` under `sm`, so last ends up on top when they stack
/// and on the right when they sit in a row. At `size="sm"` it is an even
/// two-column grid instead, and the same order reads left-to-right.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogFooter(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog_footer().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// The question, and an `<h2>`.
///
/// The script points the box's `aria-labelledby` at it at mount. Write one on
/// every alert dialog: a modal with no accessible name is announced as nothing.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogTitle(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog_title().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <h2 @{..attrs}>
            <slot />
        </h2>
    }
}

/// What the answer means, and what `aria-describedby` names.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogDescription(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog_description().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <p @{..attrs}>
            <slot />
        </p>
    }
}

/// An icon or an image above the question.
///
/// ```ignore
/// <AlertDialogHeader>
///     <AlertDialogMedia><TriangleAlert /></AlertDialogMedia>
///     <AlertDialogTitle>"Delete this file?"</AlertDialogTitle>
///     <AlertDialogDescription>"It cannot be recovered."</AlertDialogDescription>
/// </AlertDialogHeader>
/// ```
///
/// Write it *inside* the header and before the title. Everything it does to the
/// layout, the header does by selecting on it, so a media box put anywhere else
/// is just a grey square.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogMedia(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(alert_dialog_media().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// The button that answers yes.
///
/// ```ignore
/// <AlertDialogAction>"Continue"</AlertDialogAction>
/// <AlertDialogAction variant="destructive">"Delete"</AlertDialogAction>
/// ```
///
/// A `default` Button unless said otherwise. It closes the dialog, as Radix's
/// does — so the thing being confirmed goes on a handler beside that:
/// `onclick={remove(id)}`, which runs and *then* the box closes.
///
/// An action that must keep the dialog open while it waits for a server belongs
/// in a [`Dialog`](super::dialog); this component is for a question with an
/// immediate answer.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogAction(
    class: &str,
    variant: &str,
    size: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        alert_dialog_action()
            .class(class)
            .variant(ButtonVariant::parse(variant))
            .button_size(ButtonSize::parse(size)),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    html! {
        <button @{..attrs}>
            <slot />
        </button>
    }
}

/// The button that answers no.
///
/// An `outline` Button unless said otherwise, and the one the script focuses
/// when the box opens — the safe answer should be the one under the fingers.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AlertDialogCancel(
    class: &str,
    variant: &str,
    size: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let mut props = alert_dialog_cancel().class(class);

    // An unwritten `variant` is an empty string, and `outline` is this part's
    // default rather than `ButtonVariant`'s — so the prop is only read when the
    // caller actually wrote one.
    if !variant.is_empty() {
        props = props.variant(ButtonVariant::parse(variant));
    }
    props = props.button_size(ButtonSize::parse(size));

    let props = build(props, id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <button @{..attrs}>
            <slot />
        </button>
    }
}

/// The three lines every part above repeats: an optional id, the caller's
/// spread, and the caller's bindings, in that order.
fn build(
    mut props: AlertDialogProps,
    id: &str,
    attrs: Attrs,
    bindings: Bindings,
) -> AlertDialogProps {
    // An unwritten prop is an empty string, and an empty `id` is not the same
    // request as no `id` at all.
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

#[cfg(test)]
mod tests {
    use super::{
        AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
        AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogMedia,
        AlertDialogProps, AlertDialogSize, AlertDialogTitle, AlertDialogTrigger, Attrs, BACKDROP,
        CONTENT, DESCRIPTION, DIALOG_RESET, EXIT_MS, FOOTER, HEADER, MEDIA, ROOT, TITLE,
        alert_dialog, alert_dialog_action, alert_dialog_cancel, alert_dialog_content,
        alert_dialog_content_variants, alert_dialog_description, alert_dialog_footer,
        alert_dialog_header, alert_dialog_header_variants, alert_dialog_media,
        alert_dialog_media_variants, alert_dialog_title, alert_dialog_trigger,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    fn root() -> String {
        AlertDialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string()
    }

    #[test]
    fn the_root_is_the_script_and_is_not_in_the_layout() {
        let markup = root();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"data-slot="alert-dialog""#), "{markup}");
        assert!(markup.contains(r#"class="contents""#), "{markup}");
        assert!(markup.contains("showModal()"), "{markup}");
        assert!(
            markup.contains(r#"data-slot="alert-dialog-anchor""#),
            "{markup}"
        );
        assert_eq!(ROOT, "contents");
    }

    /// The one attribute `showModal()` does not imply, and the reason this
    /// component exists apart from [`Dialog`].
    #[test]
    fn the_box_is_an_alertdialog_that_starts_closed() {
        let markup = AlertDialogContent(
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("body"),
        )
        .into_test_string();

        assert!(markup.starts_with("<dialog "), "{markup}");
        assert!(markup.contains(r#"role="alertdialog""#), "{markup}");
        assert!(
            markup.contains(r#"data-slot="alert-dialog-content""#),
            "{markup}"
        );
        assert!(markup.contains(r#"data-state="closed""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");

        let tag = &markup[..markup.find('>').expect("an open tag")];
        assert!(
            !tag.split_whitespace()
                .any(|token| token == "open" || token.starts_with("open=")),
            "{tag}"
        );
    }

    /// The whole difference from [`Dialog`]: clicking beside a question is not
    /// an answer to it.
    #[test]
    fn a_backdrop_click_is_not_an_answer() {
        let markup = root();

        // `dialog.rs` registers exactly this, and this component must not.
        assert!(
            !markup.contains(r#"addEventListener("click", outside)"#),
            "{markup}"
        );
        assert!(!markup.contains("getBoundingClientRect"), "{markup}");

        // Escape is still honoured, and takes the animated path.
        assert!(
            markup.contains(r#"addEventListener("cancel", cancel)"#),
            "{markup}"
        );
        assert!(markup.contains("event.preventDefault()"), "{markup}");
    }

    /// There is no corner `X` at all — the two buttons and `Escape` are the ways
    /// out, which is what an alert dialog means.
    #[test]
    fn there_is_no_corner_close_button() {
        let markup = AlertDialogContent(
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("body"),
        )
        .into_test_string();

        assert!(!markup.contains("absolute top-4 right-4"), "{markup}");
        assert!(!markup.contains("sr-only"), "{markup}");
    }

    /// Radix puts focus on the cancel, so the safe answer is under the fingers.
    #[test]
    fn the_cancel_takes_focus_when_it_opens() {
        let markup = root();

        assert!(
            markup.contains("const safe = cancels[0] || actions[0]"),
            "{markup}"
        );
        assert!(markup.contains("safe.focus()"), "{markup}");
    }

    /// Both buttons close, as Radix's do.
    #[test]
    fn both_buttons_close_the_box() {
        let markup = root();

        assert!(markup.contains("for (const button of cancels)"), "{markup}");
        assert!(markup.contains("for (const button of actions)"), "{markup}");
        assert!(
            markup
                .matches(r#"addEventListener("click", close)"#)
                .count()
                >= 2,
            "{markup}"
        );
    }

    /// The two buttons are Buttons — shadcn wraps them in `<Button asChild>`, so
    /// their look is `buttonVariants` and not a class list of their own.
    #[test]
    fn the_action_and_the_cancel_are_buttons() {
        let action = AlertDialogAction(
            "",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Continue"),
        )
        .into_test_string();

        assert!(action.starts_with("<button "), "{action}");
        assert!(action.contains(r#"type="button""#), "{action}");
        assert!(
            action.contains(r#"data-slot="alert-dialog-action""#),
            "{action}"
        );
        // The `default` Button: a filled primary.
        assert!(action.contains("bg-primary"), "{action}");

        let cancel = AlertDialogCancel(
            "",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Cancel"),
        )
        .into_test_string();

        assert!(
            cancel.contains(r#"data-slot="alert-dialog-cancel""#),
            "{cancel}"
        );
        // The `outline` Button, which is this part's own default.
        assert!(cancel.contains("border-input"), "{cancel}");
        assert!(!cancel.contains("bg-primary"), "{cancel}");

        // shadcn's destructive confirmation.
        let danger = AlertDialogAction(
            "",
            "destructive",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Delete"),
        )
        .into_test_string();
        assert!(danger.contains("bg-destructive"), "{danger}");
    }

    /// `size` is written once and read by four other parts through the content's
    /// group — none of them takes a prop for it.
    #[test]
    fn the_size_is_one_attribute_that_four_parts_read() {
        let small = AlertDialogContent(
            "",
            "sm",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(small.contains(r#"data-size="sm""#), "{small}");

        let classes = alert_dialog_content_variants("");
        assert!(classes.contains("group/alert-dialog-content"), "{classes}");
        assert!(classes.contains("data-[size=sm]:max-w-xs"), "{classes}");
        assert!(
            classes.contains("data-[size=default]:sm:max-w-lg"),
            "{classes}"
        );

        assert!(
            FOOTER.contains("group-data-[size=sm]/alert-dialog-content:grid-cols-2"),
            "{FOOTER}"
        );
        assert!(
            HEADER.contains("sm:group-data-[size=default]/alert-dialog-content:text-left"),
            "{HEADER}"
        );
        assert!(
            MEDIA.contains("sm:group-data-[size=default]/alert-dialog-content:row-span-2"),
            "{MEDIA}"
        );
        assert!(
            TITLE.contains(
                "group-has-data-[slot=alert-dialog-media]/alert-dialog-content:col-start-2"
            ),
            "{TITLE}"
        );

        assert_eq!(AlertDialogSize::parse(""), AlertDialogSize::Default);
        assert_eq!(AlertDialogSize::parse("sm"), AlertDialogSize::Sm);
        assert_eq!(AlertDialogSize::Sm.slug(), "sm");
    }

    /// The header's grid changes shape when it holds a media box, and it finds
    /// one by `data-slot` rather than by being told.
    #[test]
    fn the_header_finds_its_media_by_slot() {
        assert!(
            HEADER.contains("has-data-[slot=alert-dialog-media]:grid-rows-[auto_auto_1fr]"),
            "{HEADER}"
        );

        let media = AlertDialogMedia(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(
            media.contains(r#"data-slot="alert-dialog-media""#),
            "{media}"
        );
        assert!(media.contains("size-16"), "{media}");
        assert!(alert_dialog_media_variants("").contains("*:[svg:not([class*='size-'])]:size-8"));
        assert_eq!(alert_dialog_header_variants(""), HEADER);
    }

    /// Radix's Overlay is a fixed div; here it is the pseudo-element the browser
    /// already paints.
    #[test]
    fn the_overlay_is_the_backdrop() {
        let classes = alert_dialog_content_variants("");

        assert!(classes.contains("backdrop:bg-black/50"), "{classes}");
        assert!(classes.contains("open:backdrop:opacity-100"), "{classes}");
        assert!(!classes.contains("alert-dialog-overlay"), "{classes}");
        assert!(!BACKDROP.contains("fixed"), "{BACKDROP}");
    }

    #[test]
    fn the_dialog_reset_answers_the_ua_stylesheet() {
        let classes = alert_dialog_content_variants("");

        assert!(classes.contains("not-open:hidden"), "{classes}");
        assert!(classes.contains("grid"), "{classes}");
        assert!(classes.contains("right-auto"), "{classes}");
        assert!(classes.contains("bottom-auto"), "{classes}");
        // Left alone on purpose, so a long question scrolls instead of running
        // off the screen.
        assert!(!classes.contains("max-h-none"), "{classes}");
        assert_eq!(DIALOG_RESET, "not-open:hidden right-auto bottom-auto");
    }

    #[test]
    fn the_animations_read_the_state_the_script_owns() {
        assert!(
            CONTENT.contains("data-[state=open]:zoom-in-95"),
            "{CONTENT}"
        );
        assert!(
            CONTENT.contains("data-[state=closed]:zoom-out-95"),
            "{CONTENT}"
        );
        assert!(CONTENT.contains("duration-200"), "{CONTENT}");
        assert_eq!(EXIT_MS, 200);
        assert!(root().contains("const EXIT_MS = 200"), "{}", root());
    }

    /// The scroll lock is the same count `dialog.rs` keeps, so the two share it
    /// rather than fighting over `<body>`.
    #[test]
    fn the_scroll_lock_is_the_one_dialog_uses() {
        let markup = root();

        assert!(markup.contains("rahtiScrollLocks"), "{markup}");
        assert!(markup.contains("rahtiScrollOverflow"), "{markup}");
        assert!(!markup.contains("ppScrollLocks"), "{markup}");
    }

    #[test]
    fn the_script_wires_the_accessible_name() {
        let markup = root();

        assert!(markup.contains("aria-labelledby"), "{markup}");
        assert!(markup.contains("aria-describedby"), "{markup}");
        assert!(markup.contains("pp.id()"), "{markup}");
    }

    #[test]
    fn the_title_is_a_heading_and_the_description_is_a_paragraph() {
        let title = AlertDialogTitle(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Sure?"),
        )
        .into_test_string();
        assert!(title.starts_with("<h2 "), "{title}");
        assert!(
            title.contains(r#"data-slot="alert-dialog-title""#),
            "{title}"
        );

        let description = AlertDialogDescription(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Why"),
        )
        .into_test_string();
        assert!(description.starts_with("<p "), "{description}");
        assert!(description.contains(DESCRIPTION), "{description}");
    }

    #[test]
    fn a_trigger_wears_whatever_the_call_site_gives_it() {
        let markup = AlertDialogTrigger(
            "inline-flex rounded-md border",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Delete"),
        )
        .into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(
            markup.contains(r#"class="inline-flex rounded-md border""#),
            "{markup}"
        );
        assert!(!markup.contains("twMerge"), "{markup}");

        let bare = AlertDialogTrigger(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(!bare.contains("class="), "{bare}");
    }

    #[test]
    fn an_override_hands_the_boxs_class_list_on_the_server() {
        let markup = AlertDialogContent(
            "sm:max-w-2xl",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(!markup.contains("{twMerge("), "{markup}");
        assert!(markup.contains("class=\""), "{markup}");
        assert!(markup.contains("sm:max-w-2xl"), "{markup}");
    }

    #[test]
    fn a_root_always_needs_a_scope_and_a_plain_part_does_not() {
        assert!(alert_dialog().needs_scope());
        assert!(!alert_dialog_header().needs_scope());
        assert!(!alert_dialog_header().class("pb-2").needs_scope());
        assert!(
            alert_dialog_content()
                .bind(
                    "data-x",
                    crate::components::rahti_ui::attrs::ui_expr!(data - x = { y })
                )
                .needs_scope()
        );
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = alert_dialog_content()
            .attr("role", "dialog")
            .attr("aria-label", "Confirm")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"role="dialog""#), "{markup}");
        assert!(markup.contains(r#"aria-label="Confirm""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = alert_dialog_trigger()
            .attrs(Attrs::new().unset("type"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("type="), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = alert_dialog_title()
            .attr("aria-label", r#"a" pp-onfocus="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn render_is_one_element_and_never_a_scope() {
        for markup in [
            alert_dialog_content()
                .render(Html::empty())
                .into_test_string(),
            alert_dialog_header()
                .class("pb-2")
                .render(Html::empty())
                .into_test_string(),
            alert_dialog_action()
                .render(Html::empty())
                .into_test_string(),
            alert_dialog_cancel()
                .render(Html::empty())
                .into_test_string(),
            alert_dialog_media()
                .render(Html::empty())
                .into_test_string(),
        ] {
            assert!(markup.starts_with('<'), "{markup}");
            assert!(!markup.contains("<script"), "{markup}");
        }
    }

    #[test]
    #[should_panic(expected = "an AlertDialog root is its script")]
    fn a_root_refuses_to_render_itself() {
        let _ = alert_dialog().render(Html::empty());
    }

    /// Every prop left unwritten, which is what `<AlertDialogContent>` compiles
    /// to now that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_part() {
        let markup = AlertDialogContent(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-state="closed""#), "{markup}");
        assert!(markup.contains(r#"data-size="default""#), "{markup}");
        assert!(markup.contains(r#"role="alertdialog""#), "{markup}");

        // An unwritten `variant` leaves the cancel its own `outline` default
        // rather than falling through to `ButtonVariant`'s `default`.
        let cancel = AlertDialogCancel(
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            <&str>::default(),
            crate::rahti::Bindings::default(),
            Attrs::default(),
            Html::empty(),
        )
        .into_test_string();
        assert!(cancel.contains("border-input"), "{cancel}");
        assert!(!cancel.contains("bg-primary"), "{cancel}");
    }

    #[test]
    fn props_defaults_are_a_plain_root() {
        assert_eq!(AlertDialogProps::new(), AlertDialogProps::default());
        assert_eq!(alert_dialog(), AlertDialogProps::default());
    }

    /// A whole alert dialog's worth of markup, in the call form the tags use.
    #[test]
    fn the_components_wrap_the_same_elements() {
        let header = AlertDialogHeader(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::concat([
                AlertDialogTitle(
                    "",
                    "",
                    crate::rahti::Bindings::default(),
                    Attrs::new(),
                    Html::escape("Are you sure?"),
                ),
                AlertDialogDescription(
                    "",
                    "",
                    crate::rahti::Bindings::default(),
                    Attrs::new(),
                    Html::escape("This cannot be undone."),
                ),
            ]),
        );

        let footer = AlertDialogFooter(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::concat([
                AlertDialogCancel(
                    "",
                    "",
                    "",
                    "",
                    crate::rahti::Bindings::default(),
                    Attrs::new(),
                    Html::escape("Cancel"),
                ),
                AlertDialogAction(
                    "",
                    "destructive",
                    "",
                    "",
                    crate::rahti::Bindings::default(),
                    Attrs::new(),
                    Html::escape("Delete"),
                ),
            ]),
        );

        let content = AlertDialogContent(
            "",
            "sm",
            "confirm",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::concat([header, footer]),
        );

        let markup = AlertDialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            content,
        )
        .into_test_string();

        assert!(markup.contains(r#"id="confirm""#), "{markup}");
        assert!(markup.contains(r#"data-size="sm""#), "{markup}");
        assert!(markup.contains(">Are you sure?</h2>"), "{markup}");
        assert!(markup.contains(">This cannot be undone.</p>"), "{markup}");
        assert!(markup.contains(">Cancel</button>"), "{markup}");
        assert!(markup.contains(">Delete</button>"), "{markup}");
        // Cancel first, action last — the order the footer's reversal expects.
        let cancel = markup.find(r#"data-slot="alert-dialog-cancel""#).unwrap();
        let action = markup.find(r#"data-slot="alert-dialog-action""#).unwrap();
        assert!(cancel < action, "{markup}");
    }
}

/// The layer-independence check behind [`CONTENT`] and [`DIALOG_RESET`].
///
/// A component here builds its class list by *joining* layers rather than
/// merging them, which is only exact if no two utilities in the result compete.
/// [`DIALOG_RESET`] is laid *under* shadcn's own string, so this is what says it
/// sits beside it rather than beating anything in it.
#[cfg(test)]
mod audit {
    use super::{
        BACKDROP, CONTENT, DESCRIPTION, DIALOG_RESET, FOOTER, HEADER, MEDIA, Part, ROOT, TITLE,
        part_classes,
    };
    use crate::components::rahti_ui::button::{ButtonSize, ButtonVariant};
    use crate::components::rahti_ui::utils::audit::clashes;

    const PARTS: [Part; 10] = [
        Part::Root,
        Part::Trigger,
        Part::Content,
        Part::Header,
        Part::Footer,
        Part::Title,
        Part::Description,
        Part::Media,
        Part::Action,
        Part::Cancel,
    ];

    const VARIANTS: [ButtonVariant; 6] = [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ];

    /// Every part, and for the two that are Buttons, every variant of one.
    #[test]
    fn no_utility_contradicts_another() {
        for part in PARTS {
            for variant in VARIANTS {
                let classes = part_classes(part, variant, ButtonSize::Default);
                let found = clashes(&classes);

                assert!(
                    found.is_empty(),
                    "{:?}/{:?}: {} clashes:\n{}",
                    part,
                    variant,
                    found.len(),
                    found.join("\n")
                );
            }
        }
    }

    #[test]
    fn no_layer_contradicts_itself() {
        for (name, layer) in [
            ("ROOT", ROOT),
            ("BACKDROP", BACKDROP),
            ("DIALOG_RESET", DIALOG_RESET),
            ("CONTENT", CONTENT),
            ("HEADER", HEADER),
            ("FOOTER", FOOTER),
            ("TITLE", TITLE),
            ("DESCRIPTION", DESCRIPTION),
            ("MEDIA", MEDIA),
        ] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
