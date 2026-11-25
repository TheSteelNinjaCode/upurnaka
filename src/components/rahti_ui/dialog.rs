// rahti-ui: dialog
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Dialog — a port of shadcn/ui's `dialog`.
//!
//! A modal box in the middle of the screen. shadcn's is Radix's Dialog; this one
//! is a `<dialog>`, which is the element HTML added for exactly this — and which
//! is why [`Sheet`](super::sheet) exists in the shape it does. This component is
//! that component's twin: same machinery, same script, different geometry.
//!
//! ## What `showModal()` gives away for free
//!
//! ```text
//! Radix                                    here
//! ──────────────────────────────────────   ──────────────────────────────────
//! Portal — escape the stacking context     the top layer, which is above
//!                                          everything by definition
//! Overlay — a fixed div at z-50            `::backdrop`, and the `backdrop:`
//!                                          utilities that paint it
//! focus trap + focus restore               the browser's
//! Escape to dismiss                        the `cancel` event
//! aria-modal, role=dialog                  implied by `showModal()`
//! background made inert                    the browser's
//! ```
//!
//! Two of shadcn's ten exports dissolve into that. `DialogPortal` becomes
//! nothing, because a modal `<dialog>` is already in the top layer and cannot be
//! covered. `DialogOverlay` becomes [`BACKDROP`], because `::backdrop` *is* the
//! overlay. Neither has a `data-slot` any more, and a page selecting on
//! `[data-slot=dialog-overlay]` will find nothing — this paragraph is the
//! reason.
//!
//! **This still needs the runtime.** `showModal()` is a method, and HTML has no
//! declarative way to call it. So the dialog is drawn and styled by the server
//! and opened by a script. Without JavaScript the trigger does nothing and the
//! box stays shut, which is the correct resting state rather than a degradation:
//! a `<dialog>` with no `open` attribute is `display: none` by UA rule.
//!
//! ## `data-state` is real here
//!
//! Kept, and the script owns it. That matters because it is what shadcn's exit
//! animation reads — `data-[state=closed]:zoom-out-95` has to run *while the box
//! is still on screen* — so the script writes `closed`, waits for the animation,
//! and only then calls `dialog.close()`. A `<dialog>` that closed immediately
//! would vanish rather than fade.
//!
//! ## What this port adds, and why
//!
//! [`DIALOG_RESET`] — the cost of using the element HTML actually has instead of
//! a `<div>` pretending to be it. Preflight already clears the UA's `margin`,
//! `padding` and `border` through its `*, ::backdrop` rule; three things it does
//! not clear are argued at the constant. It is the same shape of fix as
//! [`Sheet`](super::sheet)'s and as [`Separator`](super::separator)'s
//! `HR_RESET`.
//!
//! ## The root renders `display: contents`
//!
//! Radix's `Dialog.Root` renders no element at all — it is a context provider,
//! and the trigger it wraps lays out exactly where it was written. There is no
//! context here, so [`Dialog`] has to be a real element for its script to live
//! in and for the script to find its trigger and its box. [`ROOT`] is `contents`,
//! which takes that element back out of the layout.
//!
//! ## Where this differs from [`Sheet`](super::sheet)
//!
//! Deliberately, and in one place beyond the geometry. shadcn's `DialogClose` is
//! the bare primitive with a `data-slot` and **no class list**; the `X` in the
//! corner is a *second*, internal close button that carries [`CLOSE`]. This port
//! keeps those two apart, so a [`DialogClose`] written in a [`DialogFooter`] is
//! an ordinary button that takes whatever `class` the call site gives it —
//! usually `button_variants(…)` — rather than inheriting the corner's
//! `absolute top-4 right-4`.
//!
//! ## The parts
//!
//! ```html
//! <div data-slot="dialog" class="contents">
//!   <button data-slot="dialog-trigger">Open</button>
//!   <dialog data-slot="dialog-content" data-state="closed" class="fixed top-[50%] …">
//!     <div data-slot="dialog-header">
//!       <h2 data-slot="dialog-title">…</h2>
//!       <p data-slot="dialog-description">…</p>
//!     </div>
//!     <div data-slot="dialog-footer">…</div>
//!     <button data-slot="dialog-close" class="absolute top-4 right-4 …">…</button>
//!   </dialog>
//!   <span data-slot="dialog-anchor" hidden></span>
//!   <script>…</script>
//! </div>
//! ```
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Dialog>
//!     <DialogTrigger class=@{button_variants(ButtonVariant::Outline, ButtonSize::Default, "")}>
//!         "Edit profile"
//!     </DialogTrigger>
//!     <DialogContent>
//!         <DialogHeader>
//!             <DialogTitle>"Edit profile"</DialogTitle>
//!             <DialogDescription>"Make changes here. Save when you are done."</DialogDescription>
//!         </DialogHeader>
//!         @{fields}
//!         <DialogFooter>
//!             <DialogClose class=@{button_variants(ButtonVariant::Outline, ButtonSize::Default, "")}>
//!                 "Cancel"
//!             </DialogClose>
//!             <Button>"Save"</Button>
//!         </DialogFooter>
//!     </DialogContent>
//! </Dialog>
//! ```
//!
//! shadcn's `asChild` Button trigger crosses as a class list rather than as a
//! nested element, the same way [`Sheet`](super::sheet)'s and
//! [`Collapsible`](super::collapsible)'s do — a `<button>` inside a `<button>`
//! is not markup.
//!
//! Give every dialog a [`DialogTitle`]. The script points `aria-labelledby` at
//! it, and a modal with no accessible name is announced as nothing.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_icons::x::X;
use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::button::{ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, Json, component, html};

/// The root: `display: contents`, and the reason it needs to exist at all.
///
/// Radix's `Dialog.Root` renders nothing. This has to render something — a
/// script has to live somewhere, and it has to be able to find its own trigger
/// and box — so it renders a box and then takes the box back out of the layout.
/// A `contents` element generates no box of its own and its children become
/// children of its parent for layout purposes, so a `<DialogTrigger>` inside a
/// flex row is still an item of that row.
pub const ROOT: &str = "contents";

/// The overlay, as the pseudo-element the browser already draws.
///
/// shadcn's `DialogOverlay` is `fixed inset-0 z-50 bg-black/50` plus a fade. A
/// modal `<dialog>` paints `::backdrop` behind itself with no help, so the
/// position, the inset and the z-index are all gone and what is left is the
/// colour and the fade.
///
/// `backdrop:` is Tailwind's variant for that pseudo-element. The fade is a
/// transition rather than shadcn's `animate-in`/`fade-out-0` keyframes, because
/// those set custom properties on the animating element and `::backdrop`
/// inherits them unevenly across engines — a transition on one property is the
/// part that works everywhere. Same choice, same reasoning, as
/// [`Sheet`](super::sheet)'s.
pub const BACKDROP: &str = "backdrop:bg-black/50 backdrop:opacity-0 \
                            backdrop:transition-opacity backdrop:duration-200 \
                            open:backdrop:opacity-100";

/// What the UA stylesheet does to a `<dialog>` that shadcn's class list never
/// had to answer, because Radix renders a plain `<div>`.
///
/// Preflight clears `margin: auto`, `padding: 1em` and `border: solid` already,
/// through its `*, ::backdrop` rule. These are what is left:
///
/// - **`not-open:hidden`.** The UA hides a closed dialog with
///   `dialog:not([open]) { display: none }`, and [`CONTENT`]'s `grid` is an
///   author rule, so it *wins* — a closed dialog would paint a stray box in the
///   middle of every page it is on. `:not(…)` carries its argument's
///   specificity, so this beats `grid` whichever order the two land in.
/// - **`right-auto bottom-auto`.** The UA pins a modal dialog on all four
///   insets (`inset-inline-start/end`, `inset-block-start/end`), and shadcn
///   writes only `top-[50%] left-[50%]`. In LTR the browser resolves the
///   over-constraint by dropping the ones this clears, so nothing visibly
///   breaks — but in RTL `inset-inline-start: 0` *is* `right: 0`, and it is
///   `left-[50%]` that gets dropped instead. Clearing both makes the geometry
///   the same in either direction rather than relying on which edge the browser
///   decides to ignore.
///
/// What is deliberately *not* reset is the UA's `max-height` and `overflow` on
/// `dialog:modal`. Those cap a modal at roughly the viewport and let it scroll
/// inside, which is what a centred box wants — shadcn's `<div>` had neither, and
/// a dialog taller than the screen simply ran off it. [`Sheet`](super::sheet)
/// clears them because a drawer is `h-full` and the cap left a sliver of
/// backdrop at the edges; a Dialog is the opposite case.
pub const DIALOG_RESET: &str = "not-open:hidden right-auto bottom-auto";

/// The box. shadcn's string, unchanged.
///
/// `grid` rather than a flex column, which is shadcn's own choice and the thing
/// a scrolling dialog builds on: `grid-rows-[auto_1fr_auto]` over this is how a
/// header and a footer stay put while the middle scrolls.
///
/// `duration-200` is what [`EXIT_MS`] is a copy of.
pub const CONTENT: &str = "fixed top-[50%] left-[50%] z-50 grid w-full \
                           max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] \
                           gap-4 rounded-lg border bg-background p-6 shadow-lg duration-200 \
                           outline-none \
                           data-[state=closed]:animate-out data-[state=closed]:fade-out-0 \
                           data-[state=closed]:zoom-out-95 \
                           data-[state=open]:animate-in data-[state=open]:fade-in-0 \
                           data-[state=open]:zoom-in-95 sm:max-w-lg";

/// The `X` in the corner. shadcn's string, unchanged.
///
/// This belongs to the *internal* close button only. A [`DialogClose`] a call
/// site writes does not get it — see the module docs for why that split is
/// deliberate here and not in [`Sheet`](super::sheet).
pub const CLOSE: &str = "absolute top-4 right-4 rounded-xs opacity-70 ring-offset-background \
                         transition-opacity hover:opacity-100 focus:ring-2 focus:ring-ring \
                         focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none \
                         data-[state=open]:bg-accent data-[state=open]:text-muted-foreground \
                         [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                         [&_svg:not([class*='size-'])]:size-4";

/// A titled block at the top. shadcn's string, unchanged.
///
/// Centred on a phone and left-aligned from `sm` up, which is shadcn's own
/// asymmetry rather than an accident.
pub const HEADER: &str = "flex flex-col gap-2 text-center sm:text-left";

/// A row of buttons at the bottom. shadcn's string, unchanged.
///
/// `flex-col-reverse` on a phone and `flex-row` from `sm` up: the primary action
/// is written last and so ends up on *top* when the buttons stack, and on the
/// right when they sit in a row. Both are the conventional place for it.
pub const FOOTER: &str = "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end";

/// The dialog's accessible name. The script points `aria-labelledby` at it.
pub const TITLE: &str = "text-lg leading-none font-semibold";

/// The line under the title, and what `aria-describedby` names.
pub const DESCRIPTION: &str = "text-sm text-muted-foreground";

/// How long the script waits for the exit animation before it really closes.
///
/// shadcn's `duration-200`, as a number. The script prefers the `animationend`
/// event and falls back to this, so a browser that never fires one — or a user
/// with `prefers-reduced-motion` and no animation at all — still ends up with a
/// closed dialog.
pub const EXIT_MS: u32 = 200;

/// Which of the eight a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Trigger,
    Content,
    Close,
    Header,
    Footer,
    Title,
    Description,
}

impl Part {
    /// The `data-slot` this part carries.
    ///
    /// The first four are load-bearing: the script finds its box, its triggers
    /// and its close buttons by these strings rather than through a context, so
    /// renaming one turns a dialog inert.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "dialog",
            Self::Trigger => "dialog-trigger",
            Self::Content => "dialog-content",
            Self::Close => "dialog-close",
            Self::Header => "dialog-header",
            Self::Footer => "dialog-footer",
            Self::Title => "dialog-title",
            Self::Description => "dialog-description",
        }
    }

    /// The element this part renders as.
    const fn element(self) -> &'static str {
        match self {
            Self::Trigger | Self::Close => "button",
            Self::Content => "dialog",
            Self::Title => "h2",
            Self::Description => "p",
            Self::Root | Self::Header | Self::Footer => "div",
        }
    }

    /// This part's own utilities, before the caller's.
    ///
    /// A trigger and a close button have none — shadcn's are bare primitives
    /// whose `asChild` was always meant to hand the look to a Button, and
    /// `button_variants(…)` at the call site is how that crosses.
    const fn classes(self) -> &'static str {
        match self {
            Self::Root => ROOT,
            Self::Trigger | Self::Close => "",
            Self::Content => CONTENT,
            Self::Header => HEADER,
            Self::Footer => FOOTER,
            Self::Title => TITLE,
            Self::Description => DESCRIPTION,
        }
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part) -> String {
    match part {
        Part::Content => cn(&[DIALOG_RESET, CONTENT, BACKDROP]),
        other => other.classes().to_string(),
    }
}

/// The class list shadcn's `cn("fixed top-[50%] …", className)` returns for the
/// box itself.
///
/// Write `data-state` alongside it and keep it true — every animation utility in
/// [`CONTENT`] reads it, and the exit animation depends on it still saying
/// `closed` while the box is on screen. And write it on a `<dialog>`: half of
/// [`DIALOG_RESET`] is about that element's UA rules and the rest of the class
/// list assumes the top layer.
pub fn dialog_content_variants(class: &str) -> String {
    cn(&[&part_classes(Part::Content), class])
}

/// The corner `X`'s class list, the caller's own included.
///
/// Not a [`DialogClose`]'s — that part has no class list of its own. This is
/// here for a page that draws its own dismiss in the same corner.
pub fn dialog_close_variants(class: &str) -> String {
    cn(&[CLOSE, class])
}

/// [`DialogHeader`]'s class list, the caller's own included.
pub fn dialog_header_variants(class: &str) -> String {
    cn(&[HEADER, class])
}

/// [`DialogFooter`]'s class list, the caller's own included.
pub fn dialog_footer_variants(class: &str) -> String {
    cn(&[FOOTER, class])
}

/// [`DialogTitle`]'s class list, the caller's own included.
pub fn dialog_title_variants(class: &str) -> String {
    cn(&[TITLE, class])
}

/// [`DialogDescription`]'s class list, the caller's own included.
pub fn dialog_description_variants(class: &str) -> String {
    cn(&[DESCRIPTION, class])
}

/// One Dialog part's props, with shadcn's defaults already in place.
///
/// Build one with [`dialog`], [`dialog_trigger`], [`dialog_content`],
/// [`dialog_close`], [`dialog_header`], [`dialog_footer`], [`dialog_title`] or
/// [`dialog_description`] — each picks the part, and everything after it is the
/// same chain. Nothing here is required.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogProps {
    part: Part,
    hide_close: bool,
    close_button: bool,
    class: String,
    attrs: Attrs,
}

impl Default for DialogProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            hide_close: false,
            close_button: false,
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

/// A default [`Dialog`]'s props — the start of every chain.
pub fn dialog() -> DialogProps {
    DialogProps::of(Part::Root)
}

/// A default [`DialogTrigger`]'s props.
pub fn dialog_trigger() -> DialogProps {
    DialogProps::of(Part::Trigger)
}

/// A default [`DialogContent`]'s props.
pub fn dialog_content() -> DialogProps {
    DialogProps::of(Part::Content)
}

/// A default [`DialogClose`]'s props.
pub fn dialog_close() -> DialogProps {
    DialogProps::of(Part::Close)
}

/// A default [`DialogHeader`]'s props.
pub fn dialog_header() -> DialogProps {
    DialogProps::of(Part::Header)
}

/// A default [`DialogFooter`]'s props.
pub fn dialog_footer() -> DialogProps {
    DialogProps::of(Part::Footer)
}

/// A default [`DialogTitle`]'s props.
pub fn dialog_title() -> DialogProps {
    DialogProps::of(Part::Title)
}

/// A default [`DialogDescription`]'s props.
pub fn dialog_description() -> DialogProps {
    DialogProps::of(Part::Description)
}

impl DialogProps {
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
    /// On a trigger or a [`DialogClose`] this is the *whole* class list — both
    /// are bare primitives in shadcn, because their `asChild` was always meant
    /// to hand the look to a Button. `button_variants(…)` here is how that
    /// crosses.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// Leave the `X` out of the corner — shadcn's `showCloseButton={false}` on
    /// the content.
    ///
    /// Spelled the other way round because an unwritten Rahti `bool` prop is
    /// `false`, and a `close_button` prop on the content would therefore have
    /// taken the `X` off every dialog written. [`Separator`](super::separator)'s
    /// `announced` and [`Sheet`](super::sheet)'s `hide_close` are the same
    /// trick.
    ///
    /// Leaving it out is only reasonable when the dialog holds its own dismiss —
    /// a [`DialogClose`] in the footer, say. Escape and a backdrop click still
    /// work either way, because both are the browser's.
    pub fn hide_close(mut self, hide: bool) -> Self {
        debug_assert!(
            self.part == Part::Content,
            "rahti-ui: only a DialogContent draws the corner X."
        );
        self.hide_close = hide;
        self
    }

    /// Put shadcn's outline "Close" button at the end of the footer — its
    /// `showCloseButton` on the *footer*, which defaults to `false` there.
    ///
    /// The opposite spelling to [`hide_close`](Self::hide_close), and for the
    /// same reason: an unwritten Rahti `bool` is `false`, which is already the
    /// default shadcn gives this one. Two names for two opposite defaults rather
    /// than one name that is wrong on one of the two parts.
    pub fn close_button(mut self, show: bool) -> Self {
        debug_assert!(
            self.part == Part::Footer,
            "rahti-ui: only a DialogFooter draws the trailing Close button."
        );
        self.close_button = show;
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

    /// A bare boolean attribute: `flag("open")`.
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
    /// `dialog.close()` — so binding either is a fight the script wins. Read
    /// them instead.
    pub fn bind(self, name: &str, expr: crate::components::rahti_ui::attrs::ClientBinding) -> Self {
        let mut this = self;
        this.attrs = std::mem::take(&mut this.attrs).bind(name, expr);
        this
    }

    /// An event handler, as a PulsePoint expression.
    ///
    /// A handler here runs *beside* the script's own rather than instead of it:
    /// the script listens on the same elements and stops nothing, so a trigger's
    /// `onclick` fires and the dialog still opens.
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
        part_classes(self.part)
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
            Part::Trigger | Part::Close => base.set("type", "button"),
            // The resting state, and the only one the server can know. A dialog
            // is closed until something opens it, and a `<dialog>` with no
            // `open` attribute is `display: none` by UA rule — so this is
            // correct with or without the runtime.
            Part::Content => base.set("data-state", "closed"),
            _ => base,
        };

        let own = self.own_classes();
        base = if self.class.trim().is_empty() {
            match own.is_empty() {
                true => base,
                false => base.set("class", own),
            }
        } else if own.is_empty() {
            // A trigger and a close button have no layer of their own, so the
            // caller's class list is already finished — no merge, and no
            // `{twMerge(…)}` that would be literal text until the block mounts.
            base.set("class", self.class.trim())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&own, &self.class]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The `X` shadcn puts in the corner, or nothing.
    ///
    /// Written here rather than left to the call site because it is *inside* the
    /// content element and a caller has no way to put it there. Its class list
    /// is [`CLOSE`] and a [`DialogClose`]'s is not — see the module docs.
    fn corner_close(&self) -> Html {
        if self.hide_close {
            return Html::default();
        }

        html! {
            <button type="button" data-slot="dialog-close" class=@{CLOSE}>
                <X />
                <span class="sr-only">"Close"</span>
            </button>
        }
    }

    /// shadcn's outline "Close" at the end of the footer, or nothing.
    fn trailing_close(&self) -> Html {
        if !self.close_button {
            return Html::default();
        }

        let classes = button_variants(ButtonVariant::Outline, ButtonSize::Default, "");

        html! {
            <button type="button" data-slot="dialog-close" class=@{classes}>"Close"</button>
        }
    }

    /// The rendered element, with no boundary around it and no `<script>` in it.
    ///
    /// The root has no `render`: a root *is* its script, a script written in a
    /// method's `html!` is a block of its own, and the `pp-ref` that script needs
    /// would never resolve there. Use the [`Dialog`] tag.
    pub fn render(&self, children: Html) -> Html {
        debug_assert!(
            !matches!(self.part, Part::Root),
            "rahti-ui: a Dialog root is its script, and a script in a method's \
             `html!` is a block of its own. Use the <Dialog> tag."
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
            "dialog" => {
                let close = self.corner_close();
                html! {
                    <dialog @{..attrs}>@{children}@{close}</dialog>
                }
            }
            _ if self.part == Part::Footer => {
                let close = self.trailing_close();
                html! {
                    <div @{..attrs}>@{children}@{close}</div>
                }
            }
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

/// A modal box, and the trigger that opens it.
///
/// ```ignore
/// <Dialog>
///     <DialogTrigger class=@{button_variants(ButtonVariant::Outline, ButtonSize::Default, "")}>
///         "Open"
///     </DialogTrigger>
///     <DialogContent>
///         <DialogHeader>
///             <DialogTitle>"Edit profile"</DialogTitle>
///             <DialogDescription>"Make changes here."</DialogDescription>
///         </DialogHeader>
///     </DialogContent>
/// </Dialog>
/// ```
///
/// This element is `display: contents`, so it is not in the layout — the trigger
/// inside it sits exactly where it was written. It exists because the script has
/// to live somewhere and has to be able to find its own parts, which Radix does
/// with a context that has no equivalent here.
///
/// ## What the script does, and what the browser does
///
/// Almost all of it is the browser's. `showModal()` puts the box in the top
/// layer, paints `::backdrop`, traps focus, restores it on close, makes the rest
/// of the page inert and closes on `Escape`. The script opens it, closes it on a
/// backdrop click, locks the body's scroll while it is open, points
/// `aria-labelledby` and `aria-describedby` at the title and description, and —
/// the one genuinely fiddly part — holds the box on screen for the length of its
/// exit animation before letting the dialog actually close.
///
/// A dialog written inside another dialog's content keeps its own parts: the
/// script only claims elements whose nearest `[data-slot=dialog]` is this root.
///
/// None of its state is `pp.state`, for the reason
/// [`ScrollArea`](super::scroll_area) gives: a state change re-renders the block
/// and hands the attribute reconciler a chance to put the server's
/// `data-state="closed"` back over the one the script just wrote.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Dialog(class: &str, id: &str, bindings: Bindings, attrs: Attrs, children: Html) -> Html {
    let mut props = dialog().class(class);

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

    let root = props.attr_list();
    let exit = EXIT_MS;
    let open_state = Json("open");
    let closed_state = Json("closed");

    html! {
        <div @{..root}>
            <slot />
            <span data-slot="dialog-anchor" hidden=@{true} pp-ref={anchorRef}></span>
            <script>r##"
                const EXIT_MS = @{exit};
                const OPEN = @{open_state};
                const CLOSED = @{closed_state};

                const anchorRef = pp.ref(null);
                const uid = pp.id();

                pp.effect(() => {
                    const anchor = anchorRef.current;
                    if (!anchor) return;

                    const root = anchor.closest("[data-slot=dialog]");
                    if (!root) return;

                    // Everything belonging to *this* dialog, so a dialog written
                    // inside another dialog's content keeps its own parts.
                    const own = (slot) =>
                        Array.from(root.querySelectorAll("[data-slot=" + slot + "]"))
                            .filter((el) => el.closest("[data-slot=dialog]") === root);

                    const box = own("dialog-content")[0];
                    if (!box) return;
                    const triggers = own("dialog-trigger");
                    const closers = own("dialog-close");

                    // The accessible name and description. Both need an id on
                    // the other half, which is why neither part could write it.
                    const title = box.querySelector("[data-slot=dialog-title]");
                    const description = box.querySelector("[data-slot=dialog-description]");
                    if (title) {
                        if (!title.id) title.id = uid + "-title";
                        box.setAttribute("aria-labelledby", title.id);
                    }
                    if (description) {
                        if (!description.id) description.id = uid + "-description";
                        box.setAttribute("aria-describedby", description.id);
                    }

                    let timer = 0;

                    // The browser makes the page inert but does not stop it
                    // scrolling behind the dialog, so the lock is ours. It is a
                    // *count* on `<body>` rather than a value each dialog saves
                    // and puts back, because dialogs nest: a per-dialog snapshot
                    // has the inner one restoring `hidden` — which was the
                    // outer's doing — and the page never scrolls again.
                    // Measured, and this is the shape that survives it.
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
                        // `showModal()` moves focus to the first focusable
                        // element, and on a text field the browser leaves the
                        // caret at position 0 — before the value. Radix ends up
                        // with the caret after it, which is also where a person
                        // continues typing. Only where the selection API
                        // applies: `setSelectionRange` throws on types that do
                        // not carry a selection (`email`, `number`).
                        const field = document.activeElement;
                        if (
                            field &&
                            box.contains(field) &&
                            (field.tagName === "TEXTAREA" ||
                                (field.tagName === "INPUT" &&
                                    /^(text|search|url|tel|password)$/.test(field.type)))
                        ) {
                            const end = field.value.length;
                            try { field.setSelectionRange(end, end); } catch (_) {}
                        }
                    };

                    const finish = () => {
                        clearTimeout(timer);
                        timer = 0;
                        if (box.open) box.close();
                    };

                    // The one genuinely fiddly part. `dialog.close()` hides the
                    // element the instant it is called, so an exit animation has
                    // to run *before* it: write `closed`, let
                    // `data-[state=closed]:zoom-out-95` play, and close when it
                    // ends. `animationend` is the accurate signal and the timer
                    // is the net for a browser that never fires one —
                    // `prefers-reduced-motion` with no animation at all, say.
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
                    // path instead.
                    const cancel = (event) => {
                        // A file input also emits a bubbling `cancel` when its
                        // native picker is dismissed. Only this dialog's own
                        // cancellation should close the dialog.
                        if (event.target !== box) return;
                        event.preventDefault();
                        close();
                    };

                    const restore = () => {
                        unlock();
                        // `close` is queued, not dispatched by `close()` itself,
                        // so a dialog reopened right after it closed receives the
                        // old one late. Marking that open box `closed` would let
                        // its opening `animationend` finish it off.
                        if (box.open) return;
                        box.setAttribute("data-state", CLOSED);
                        // A dialog written inside this one is a DOM descendant,
                        // so closing this box only *hides* it: it stays `open`,
                        // stays in the top layer, keeps its scroll lock, and is
                        // still there the next time this box is opened. Closing
                        // it here fires its own `close`, which unlocks and
                        // cascades to anything nested inside it.
                        const nested = box.querySelectorAll("dialog[data-slot=dialog-content]");
                        for (const inner of nested) if (inner.open) inner.close();
                    };

                    // A click that lands outside the box is a click on
                    // `::backdrop`, which targets the dialog itself.
                    //
                    // Both halves of this test are load-bearing. The target
                    // check is what keeps a *nested* dialog from closing this
                    // one: a click anywhere inside an inner dialog bubbles up
                    // to this element too, and the inner box — or its own
                    // backdrop — is very often outside this box's rectangle.
                    // The rectangle check is what the target check cannot do
                    // on its own: this element's padding and its `gap-4` are
                    // part of the dialog, so a click there targets `box` and is
                    // still inside.
                    const outside = (event) => {
                        if (event.target !== box) return;
                        const rect = box.getBoundingClientRect();
                        const inside =
                            event.clientX >= rect.left &&
                            event.clientX <= rect.right &&
                            event.clientY >= rect.top &&
                            event.clientY <= rect.bottom;
                        if (!inside) close();
                    };

                    for (const trigger of triggers) trigger.addEventListener("click", open);
                    for (const closer of closers) closer.addEventListener("click", close);
                    box.addEventListener("cancel", cancel);
                    box.addEventListener("close", restore);
                    box.addEventListener("animationend", animated);
                    box.addEventListener("click", outside);

                    return () => {
                        clearTimeout(timer);
                        if (box.open) box.close();
                        for (const trigger of triggers) trigger.removeEventListener("click", open);
                        for (const closer of closers) closer.removeEventListener("click", close);
                        box.removeEventListener("cancel", cancel);
                        box.removeEventListener("close", restore);
                        box.removeEventListener("animationend", animated);
                        box.removeEventListener("click", outside);
                    };
                }, []);
            "##</script>
        </div>
    }
}

/// The thing that opens the dialog.
///
/// shadcn's trigger has no class list of its own — its `asChild` was always
/// meant to hand the look to a Button — so `button_variants(…)` here is how that
/// crosses, the same way [`Sheet`](super::sheet)'s does.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogTrigger(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(dialog_trigger().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <button @{..attrs}>
            <slot />
        </button>
    }
}

/// The box itself — a `<dialog>`.
///
/// ```ignore
/// <DialogContent>…</DialogContent>
/// <DialogContent hide_close>…</DialogContent>
/// <DialogContent class="sm:max-w-2xl">…</DialogContent>
/// ```
///
/// It carries `data-state="closed"` from the server and a `<dialog>` with no
/// `open` attribute is `display: none` by UA rule, so a page with no JavaScript
/// shows nothing rather than a box stuck open. Give it a [`DialogTitle`]: the
/// script points `aria-labelledby` at it, and a modal with no accessible name is
/// announced as nothing.
///
/// The `X` in the corner is drawn unless `hide_close`. Escape and a backdrop
/// click work either way.
///
/// It is a `grid` with a `gap-4`, which is what a scrolling dialog builds on:
/// `class="grid-rows-[auto_1fr_auto] max-h-[80vh]"` with `overflow-y-auto` and
/// `min-h-0` on the middle region keeps the header and footer still.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogContent(
    class: &str,
    hide_close: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dialog_content().class(class).hide_close(hide_close),
        id,
        attrs,
        bindings,
    );

    let close = props.corner_close();
    let attrs = props.attr_list();

    html! {
        <dialog @{..attrs}>
            <slot />
            @{close}
        </dialog>
    }
}

/// A button inside the dialog that closes it.
///
/// ```ignore
/// <DialogClose class=@{button_variants(ButtonVariant::Outline, ButtonSize::Default, "")}>
///     "Cancel"
/// </DialogClose>
/// ```
///
/// No class list of its own — shadcn's is the bare primitive, and the corner `X`
/// is a different button that happens to share the `data-slot`. Give it
/// `button_variants(…)`, or it renders unstyled.
///
/// Takes the animated path, the same as the corner `X` and `Escape` — a footer
/// button that called `dialog.close()` outright would make the box vanish rather
/// than fade.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogClose(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(dialog_close().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <button @{..attrs}>
            <slot />
        </button>
    }
}

/// A titled block at the top.
///
/// Centred on a phone and left-aligned from `sm` up, which is shadcn's.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogHeader(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(dialog_header().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// A row of buttons at the bottom.
///
/// ```ignore
/// <DialogFooter>
///     <DialogClose class=@{outline}>"Cancel"</DialogClose>
///     <Button>"Save"</Button>
/// </DialogFooter>
///
/// <DialogFooter close_button />
/// ```
///
/// Write the primary action *last*: the row is `flex-col-reverse` on a phone and
/// `flex-row` from `sm` up, so last ends up on top when the buttons stack and on
/// the right when they sit in a row.
///
/// `close_button` appends shadcn's outline "Close" — its `showCloseButton` on
/// the footer, which is `false` by default there and so keeps the same spelling
/// in Rahti. On the *content* the same idea is spelled `hide_close`, because
/// that one defaults to `true`.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogFooter(
    class: &str,
    close_button: bool,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        dialog_footer().class(class).close_button(close_button),
        id,
        attrs,
        bindings,
    );

    let close = props.trailing_close();
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
            @{close}
        </div>
    }
}

/// The dialog's accessible name, and an `<h2>`.
///
/// Radix's Title is a `<h2>` and so is this. The script points the dialog's
/// `aria-labelledby` at it at mount.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogTitle(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(dialog_title().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <h2 @{..attrs}>
            <slot />
        </h2>
    }
}

/// The line under the title, and what `aria-describedby` names.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn DialogDescription(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(dialog_description().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <p @{..attrs}>
            <slot />
        </p>
    }
}

/// The three lines every part above repeats: an optional id, the caller's
/// spread, and the caller's bindings, in that order.
fn build(mut props: DialogProps, id: &str, attrs: Attrs, bindings: Bindings) -> DialogProps {
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
        Attrs, BACKDROP, CLOSE, CONTENT, DESCRIPTION, DIALOG_RESET, Dialog, DialogClose,
        DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogProps, DialogTitle,
        DialogTrigger, EXIT_MS, FOOTER, HEADER, ROOT, TITLE, dialog, dialog_close,
        dialog_close_variants, dialog_content, dialog_content_variants, dialog_description,
        dialog_description_variants, dialog_footer, dialog_footer_variants, dialog_header,
        dialog_header_variants, dialog_title, dialog_title_variants, dialog_trigger,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    fn content(hide_close: bool) -> String {
        DialogContent(
            "",
            hide_close,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("body"),
        )
        .into_test_string()
    }

    #[test]
    fn the_root_is_the_script_and_is_not_in_the_layout() {
        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.starts_with("<div "), "{markup}");
        assert!(markup.contains(r#"data-slot="dialog""#), "{markup}");
        assert!(markup.contains(r#"class="contents""#), "{markup}");
        assert!(markup.contains("<script"), "{markup}");
        assert!(markup.contains("showModal()"), "{markup}");
        // The anchor the script's `pp-ref` resolves against.
        assert!(markup.contains(r#"data-slot="dialog-anchor""#), "{markup}");
        assert_eq!(ROOT, "contents");
    }

    /// The resting state, and the one the server can know. A `<dialog>` with no
    /// `open` attribute is `display: none` by UA rule, so a page with no
    /// JavaScript shows nothing rather than a box stuck open.
    #[test]
    fn the_box_is_a_closed_dialog_element() {
        let markup = content(false);

        assert!(markup.starts_with("<dialog "), "{markup}");
        assert!(markup.contains(r#"data-slot="dialog-content""#), "{markup}");
        assert!(markup.contains(r#"data-state="closed""#), "{markup}");
        assert!(markup.ends_with("</dialog>"), "{markup}");

        // No `open` attribute. Read off the tag's attribute *names* rather than
        // searched for in the string: `not-open:hidden` and
        // `open:backdrop:opacity-100` both put the word in the class list, and a
        // substring test would find those instead.
        let tag = &markup[..markup.find('>').expect("an open tag")];
        assert!(
            !tag.split_whitespace()
                .any(|token| token == "open" || token.starts_with("open=")),
            "{tag}"
        );
    }

    /// The two things the UA stylesheet does that shadcn's class list never had
    /// to answer.
    #[test]
    fn the_dialog_reset_answers_the_ua_stylesheet() {
        let classes = dialog_content_variants("");

        // A closed dialog would otherwise paint: `grid` is an author rule and
        // beats the UA's `display: none`.
        assert!(classes.contains("not-open:hidden"), "{classes}");
        assert!(classes.contains("grid"), "{classes}");
        // The UA pins all four insets; shadcn writes only two.
        assert!(classes.contains("right-auto"), "{classes}");
        assert!(classes.contains("bottom-auto"), "{classes}");
        // And what is deliberately left alone — a modal's own cap is what lets a
        // tall dialog scroll instead of running off the screen.
        assert!(!classes.contains("max-h-none"), "{classes}");
        assert_eq!(DIALOG_RESET, "not-open:hidden right-auto bottom-auto");
    }

    /// Radix's Overlay is a fixed div; here it is the pseudo-element the browser
    /// already paints, so the position, the inset and the z-index are all gone.
    #[test]
    fn the_overlay_is_the_backdrop() {
        let classes = dialog_content_variants("");

        assert!(classes.contains("backdrop:bg-black/50"), "{classes}");
        assert!(classes.contains("open:backdrop:opacity-100"), "{classes}");
        // Dissolved, and with it the slot a page might have selected on.
        assert!(!classes.contains("dialog-overlay"), "{classes}");
        assert!(!BACKDROP.contains("fixed"), "{BACKDROP}");
        assert!(!BACKDROP.contains("inset-0"), "{BACKDROP}");
    }

    /// The exit animation reads `data-state`, and the script's close path is
    /// built around holding it on screen long enough for that to play.
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
        assert!(
            CONTENT.contains("data-[state=closed]:fade-out-0"),
            "{CONTENT}"
        );
        // `EXIT_MS` is `duration-200` as a number, and the two have to agree.
        assert!(CONTENT.contains("duration-200"), "{CONTENT}");
        assert_eq!(EXIT_MS, 200);

        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(markup.contains("const EXIT_MS = 200"), "{markup}");
    }

    /// shadcn's `showCloseButton` on the content, spelled the other way round
    /// because an unwritten Rahti `bool` is `false`.
    #[test]
    fn the_corner_x_is_drawn_unless_it_is_hidden() {
        let drawn = content(false);
        assert!(drawn.contains(r#"data-slot="dialog-close""#), "{drawn}");
        assert!(drawn.contains("absolute top-4 right-4"), "{drawn}");
        assert!(drawn.contains(r#"class="sr-only""#), "{drawn}");
        assert!(drawn.contains("Close"), "{drawn}");

        let hidden = content(true);
        assert!(!hidden.contains(r#"data-slot="dialog-close""#), "{hidden}");
        assert!(!hidden.contains("absolute top-4"), "{hidden}");
    }

    /// The one place this port differs from [`Sheet`] beyond the geometry: the
    /// corner `X` owns [`CLOSE`] and a caller's `DialogClose` does not.
    #[test]
    fn a_written_close_button_is_not_the_corner_x() {
        let markup = DialogClose(
            "inline-flex h-9 px-4",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Cancel"),
        )
        .into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(markup.contains(r#"type="button""#), "{markup}");
        assert!(markup.contains(r#"data-slot="dialog-close""#), "{markup}");
        assert!(
            markup.contains(r#"class="inline-flex h-9 px-4""#),
            "{markup}"
        );
        // Not positioned into the corner, and not handed to `twMerge` either —
        // there is no layer of its own for a caller's classes to conflict with.
        assert!(!markup.contains("absolute"), "{markup}");
        assert!(!markup.contains("twMerge"), "{markup}");

        // The corner's own string is still reachable for a page that draws one.
        assert!(dialog_close_variants("").contains("absolute top-4 right-4"));
    }

    /// A trigger has no class list of its own either, for the same reason.
    #[test]
    fn a_trigger_wears_whatever_the_call_site_gives_it() {
        let markup = DialogTrigger(
            "inline-flex items-center rounded-md border",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Open"),
        )
        .into_test_string();

        assert!(markup.starts_with("<button "), "{markup}");
        assert!(markup.contains(r#"type="button""#), "{markup}");
        assert!(markup.contains(r#"data-slot="dialog-trigger""#), "{markup}");
        assert!(
            markup.contains(r#"class="inline-flex items-center rounded-md border""#),
            "{markup}"
        );
        assert!(!markup.contains("twMerge"), "{markup}");

        // And an unstyled one carries no `class` at all rather than an empty one.
        let bare = DialogTrigger(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();
        assert!(!bare.contains("class="), "{bare}");
    }

    /// shadcn's `showCloseButton` on the *footer*, which defaults to `false`
    /// there — so this one keeps the positive spelling.
    #[test]
    fn a_footer_can_append_shadcns_close_button() {
        let plain = DialogFooter(
            "",
            false,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("body"),
        )
        .into_test_string();
        assert!(plain.contains(r#"data-slot="dialog-footer""#), "{plain}");
        assert!(!plain.contains(r#"data-slot="dialog-close""#), "{plain}");

        let with_close = DialogFooter(
            "",
            true,
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("body"),
        )
        .into_test_string();
        assert!(
            with_close.contains(r#"data-slot="dialog-close""#),
            "{with_close}"
        );
        assert!(with_close.contains(">Close<"), "{with_close}");
        // It is a Button, and it is the outline one.
        assert!(with_close.contains("border-input"), "{with_close}");
        // Appended after the children, so it is the last thing in the row.
        let body = with_close.find("body").unwrap();
        let close = with_close.find(r#"data-slot="dialog-close""#).unwrap();
        assert!(body < close, "{with_close}");
    }

    #[test]
    fn the_header_and_footer_carry_shadcns_own_asymmetry() {
        assert_eq!(HEADER, "flex flex-col gap-2 text-center sm:text-left");
        assert_eq!(
            FOOTER,
            "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"
        );

        let markup = DialogHeader(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("head"),
        )
        .into_test_string();
        assert!(markup.contains("text-center"), "{markup}");
        assert!(markup.contains("sm:text-left"), "{markup}");
    }

    #[test]
    fn the_title_is_a_heading_and_the_description_is_a_paragraph() {
        let title = DialogTitle(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("Edit"),
        )
        .into_test_string();
        assert!(title.starts_with("<h2 "), "{title}");
        assert!(title.contains(r#"data-slot="dialog-title""#), "{title}");
        assert!(title.contains(TITLE), "{title}");

        let description = DialogDescription(
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

    /// The script pairs the box with its title and description by id, because
    /// neither part can write an attribute that names the other.
    #[test]
    fn the_script_wires_the_accessible_name() {
        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains("aria-labelledby"), "{markup}");
        assert!(markup.contains("aria-describedby"), "{markup}");
        assert!(markup.contains("pp.id()"), "{markup}");
    }

    /// A dialog inside another dialog's content keeps its own parts.
    #[test]
    fn the_script_claims_only_its_own_parts() {
        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"closest("[data-slot=dialog]") === root"#),
            "{markup}"
        );
    }

    /// A click inside a nested dialog bubbles to the outer `<dialog>` as well,
    /// and the inner box is very often outside the outer one's rectangle — so
    /// the backdrop test has to check the target before it measures anything.
    /// Measured in the browser: without the guard, dismissing the inner dialog
    /// closed the outer one with it.
    #[test]
    fn a_backdrop_click_is_this_boxs_own() {
        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(
            markup.contains("if (event.target !== box) return;"),
            "{markup}"
        );
        // And still measures, because this element's own padding targets it too.
        assert!(markup.contains("getBoundingClientRect()"), "{markup}");
    }

    /// The scroll lock is a count on `<body>`, not a value each dialog saves and
    /// puts back.
    ///
    /// Measured: with a per-dialog snapshot, an inner dialog restores the
    /// `hidden` the *outer* one set, and the page never scrolls again. The count
    /// is what makes the nested case come back to zero.
    #[test]
    fn the_scroll_lock_counts_rather_than_snapshots() {
        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains("rahtiScrollLocks"), "{markup}");
        assert!(markup.contains("const lock = ()"), "{markup}");
        assert!(markup.contains("const unlock = ()"), "{markup}");
        // Not PulsePoint's namespace: `data-pp-*` is the runtime's to write.
        assert!(!markup.contains("ppScrollLocks"), "{markup}");
    }

    /// Closing a dialog only *hides* one written inside it — it stays `open`,
    /// stays in the top layer and keeps its lock. Measured, and this is what
    /// closes it.
    #[test]
    fn closing_a_dialog_closes_the_ones_inside_it() {
        let markup = Dialog(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::empty(),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"box.querySelectorAll("dialog[data-slot=dialog-content]")"#),
            "{markup}"
        );
        assert!(
            markup.contains("if (inner.open) inner.close();"),
            "{markup}"
        );
    }

    #[test]
    fn an_override_hands_the_boxs_class_list_on_the_server() {
        let markup = DialogContent(
            "sm:max-w-2xl",
            false,
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
        assert!(dialog().needs_scope());
        assert!(!dialog_header().needs_scope());
        assert!(!dialog_header().class("pb-2").needs_scope());
        assert!(
            dialog_content()
                .bind(
                    "data-x",
                    crate::components::rahti_ui::attrs::ui_expr!(data - x = { y })
                )
                .needs_scope()
        );
    }

    #[test]
    fn caller_attributes_win_over_the_components_own() {
        let markup = dialog_content()
            .attr("data-state", "open")
            .attr("aria-label", "Editor")
            .render(Html::empty())
            .into_test_string();

        assert!(markup.contains(r#"data-state="open""#), "{markup}");
        assert!(markup.contains(r#"aria-label="Editor""#), "{markup}");
    }

    #[test]
    fn a_forwarded_attrs_can_take_an_attribute_away() {
        let markup = dialog_trigger()
            .attrs(Attrs::new().unset("type"))
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("type="), "{markup}");
    }

    #[test]
    fn a_caller_value_cannot_break_out_of_the_quotes() {
        let markup = dialog_title()
            .attr("aria-label", r#"a" pp-onfocus="steal()"#)
            .render(Html::empty())
            .into_test_string();

        assert!(!markup.contains("pp-onfocus=\""), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    /// `render` never carries a script — the root is the exception and has none,
    /// which is what the `debug_assert` says.
    #[test]
    fn render_is_one_element_and_never_a_scope() {
        for markup in [
            dialog_content().render(Html::empty()).into_test_string(),
            dialog_header()
                .class("pb-2")
                .render(Html::empty())
                .into_test_string(),
            dialog_close()
                .class("px-4")
                .render(Html::empty())
                .into_test_string(),
            dialog_footer().render(Html::empty()).into_test_string(),
        ] {
            assert!(markup.starts_with('<'), "{markup}");
            assert!(!markup.contains("<script"), "{markup}");
        }
    }

    #[test]
    #[should_panic(expected = "a Dialog root is its script")]
    fn a_root_refuses_to_render_itself() {
        let _ = dialog().render(Html::empty());
    }

    /// Every prop left unwritten, which is what `<DialogContent>` compiles to
    /// now that `html!` closes the props literal with `..Default::default()`.
    #[test]
    fn all_props_omitted_is_a_default_part() {
        let markup = DialogContent(
            <&str>::default(),
            bool::default(),
            <&str>::default(),
            crate::rahti::Bindings::__from_html(
                <&'static [(&'static str, &'static str)]>::default(),
            ),
            Attrs::default(),
            Html::empty(),
        )
        .into_test_string();

        assert!(markup.contains(r#"data-state="closed""#), "{markup}");
        // `hide_close` defaults to `false`, so the X is drawn — shadcn's default.
        assert!(markup.contains("absolute top-4 right-4"), "{markup}");

        let markup = DialogFooter(
            <&str>::default(),
            bool::default(),
            <&str>::default(),
            crate::rahti::Bindings::default(),
            Attrs::default(),
            Html::empty(),
        )
        .into_test_string();
        // `close_button` defaults to `false` — also shadcn's default.
        assert!(!markup.contains(r#"data-slot="dialog-close""#), "{markup}");
    }

    /// The class list a page reaches for when the element has to be its own —
    /// the same one the component would have rendered.
    #[test]
    fn the_variants_helpers_return_what_the_components_render() {
        assert_eq!(
            dialog_content_variants("sm:max-w-2xl"),
            dialog_content().class("sm:max-w-2xl").class_list()
        );
        assert_eq!(
            dialog_header_variants("pb-2"),
            dialog_header().class("pb-2").class_list()
        );
        assert_eq!(
            dialog_footer_variants("pt-2"),
            dialog_footer().class("pt-2").class_list()
        );
        assert_eq!(
            dialog_title_variants("text-xl"),
            dialog_title().class("text-xl").class_list()
        );
        assert_eq!(
            dialog_description_variants("text-balance"),
            dialog_description().class("text-balance").class_list()
        );
        assert_eq!(CLOSE, dialog_close_variants(""));
    }

    #[test]
    fn props_defaults_are_a_plain_root() {
        assert_eq!(DialogProps::new(), DialogProps::default());
        assert_eq!(dialog(), DialogProps::default());
    }
}

/// The layer-independence check behind [`CONTENT`] and [`DIALOG_RESET`].
///
/// A component here builds its class list by *joining* layers rather than
/// merging them, which is only exact if no two utilities in the result compete.
/// The box is the one that needed watching: [`DIALOG_RESET`] is laid *under*
/// shadcn's own string, and a blanket `inset-auto` there would have beaten
/// `top-[50%]` rather than sitting beside it. This is what says it does not.
#[cfg(test)]
mod audit {
    use super::{
        BACKDROP, CLOSE, CONTENT, DESCRIPTION, DIALOG_RESET, FOOTER, HEADER, Part, ROOT, TITLE,
        part_classes,
    };
    use crate::components::rahti_ui::utils::audit::clashes;

    const PARTS: [Part; 8] = [
        Part::Root,
        Part::Trigger,
        Part::Content,
        Part::Close,
        Part::Header,
        Part::Footer,
        Part::Title,
        Part::Description,
    ];

    #[test]
    fn no_utility_contradicts_another() {
        for part in PARTS {
            let classes = part_classes(part);
            let found = clashes(&classes);

            assert!(
                found.is_empty(),
                "{:?}: {} clashes:\n{}",
                part,
                found.len(),
                found.join("\n")
            );
        }
    }

    #[test]
    fn no_layer_contradicts_itself() {
        for (name, layer) in [
            ("ROOT", ROOT),
            ("BACKDROP", BACKDROP),
            ("DIALOG_RESET", DIALOG_RESET),
            ("CONTENT", CONTENT),
            ("CLOSE", CLOSE),
            ("HEADER", HEADER),
            ("FOOTER", FOOTER),
            ("TITLE", TITLE),
            ("DESCRIPTION", DESCRIPTION),
        ] {
            let found = clashes(layer);
            assert!(found.is_empty(), "{name}: {}", found.join("\n"));
        }
    }
}
