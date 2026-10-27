//! Attribute sets the application repeats on rahti-ui tags.
//!
//! A component tag takes only the props it names, so an attribute such as
//! `aria-label` travels through its `attrs` prop. These keep the common ones
//! to a single call at the tag.

use crate::components::rahti_ui::attrs::{Attrs, ClientBinding};

/// `aria-label` and a matching `title`, for a control that shows only an icon.
pub fn labelled(label: &str) -> Attrs {
    Attrs::new().set("aria-label", label).set("title", label)
}

/// `aria-label` alone, for a control whose tooltip would repeat visible text.
pub fn named(label: &str) -> Attrs {
    Attrs::new().set("aria-label", label)
}

/// `type="button"`, for a rahti-ui Button inside a form that does not submit
/// it. The Button writes no `type` of its own, and inside a `<form>` the HTML
/// default is `submit` — so a toolbar or picker button there would send the
/// form.
pub fn plain() -> Attrs {
    Attrs::new().set("type", "button")
}

/// `type="submit"`, for a rahti-ui Button that submits its form.
pub fn submit() -> Attrs {
    Attrs::new().set("type", "submit")
}

/// A control that exists only to be clicked from a script.
///
/// rahti-ui's Dialog and AlertDialog open from a trigger and close from a
/// close button; they have no `open` prop. A page that opens one from a menu
/// row, a keyboard shortcut or a successful save writes a hidden trigger (or
/// close) with a `<span pp-ref={…}>` inside it and clicks the span: the click
/// bubbles to the button, which is what the dialog listens on.
///
/// The ref is on a child element because a component tag cannot carry a
/// hyphenated binding such as `pp-ref`, and children written between
/// component tags are evaluated in the page's scope.
pub fn hidden() -> Attrs {
    Attrs::new().set("hidden", "").set("tabindex", "-1")
}

/// One client binding — `aria-pressed={on}`, `aria-label={name}` — as an
/// attribute set, for a rahti-ui tag.
///
/// A component tag's props are Rust identifiers, so an attribute with a `-` in
/// its name cannot be written on one; it travels in `attrs`, compiled at the
/// call site by rahti-ui's `ui_expr!`:
///
/// ```ignore
/// attrs=@{bound("aria-pressed", ui_expr!(aria-pressed={handRaised}))}
/// ```
///
/// Only for a tag that renders in the page's own scope. A binding carried this
/// way is not seen by `html!` as one of the page's, so inside the children of a
/// scripted rahti-ui component — a Dialog, a Popover, a DropdownMenu, an
/// InputGroup — it is evaluated in that component's scope and finds nothing.
/// There, put the binding on an element you write instead. And `ui_expr!` must
/// be written where the expression is: it reads the source from its span.
///
/// Not for `pp-ref`: a ref carried this way is never bound. Put
/// `<span class="contents" pp-ref={mark}>` inside the component and reach the
/// element from it (`mark.current?.closest("button")`).
pub fn bound(name: &str, binding: ClientBinding) -> Attrs {
    Attrs::new().bind(name, binding)
}
