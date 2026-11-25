// rahti-ui: avatar
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Avatar — a port of shadcn/ui's `avatar`.
//!
//! A round picture with something to show while it is not there. shadcn's is
//! Radix's Avatar, whose whole job is that second half: it loads the image in
//! JavaScript, tracks `idle`/`loading`/`loaded`/`error`, and renders the
//! fallback until the load succeeds.
//!
//! ## The fallback is a stacking order, not a state machine
//!
//! None of that crosses, because it does not have to. Put the fallback in the
//! box and lay the image over it:
//!
//! ```html
//! <span data-slot="avatar" class="relative overflow-hidden rounded-full …">
//!   <span data-slot="avatar-fallback" class="flex size-full …">CN</span>
//!   <img data-slot="avatar-image" class="absolute inset-0 aspect-square size-full" alt="" src="…" />
//! </span>
//! ```
//!
//! The image is positioned and the fallback is not, so the image paints above it
//! without a `z-index` — and when the image loads, that is all that shows. When
//! it fails, or when there is no `src` at all, an `<img>` with an empty `alt`
//! renders nothing at all and the fallback underneath is what is seen.
//!
//! So the states Radix tracks are the browser's: *loading* is the fallback still
//! showing, *loaded* is the image covering it, and *error* is the image
//! collapsing back to nothing. No script, no flash of the wrong one, and the
//! right thing is on screen in the first byte rather than after a load event.
//!
//! [`IMAGE_LAYER`] is the one utility that buys all of it.
//!
//! ## `alt` is the image's, and it is deliberately empty by default
//!
//! An avatar is nearly always beside the name it belongs to — in the
//! [`Sidebar`](super::sidebar)'s footer, in a menu label, next to a comment. A
//! picture that repeats the name it is sitting next to is noise to a screen
//! reader, so the image is decorative unless a page says otherwise with
//! [`alt`](AvatarProps::alt).
//!
//! It also has to stay empty for the fallback to work: a broken `<img>` with a
//! non-empty `alt` draws the alt text and a broken-image icon *over* the
//! fallback in most browsers, which is exactly what this arrangement is trying
//! to avoid. Give the avatar an `alt` only when the picture is the only thing
//! naming the person.
//!
//! ## Which shadcn avatar this is
//!
//! The registry. All six of its parts are here, including the three the older
//! shadcn avatar did not have — [`AvatarBadge`], [`AvatarGroup`] and
//! [`AvatarGroupCount`] — and every class string is shadcn's, with
//! [`IMAGE_LAYER`] the only addition.
//!
//! ## The call site is the JSX one
//!
//! ```ignore
//! <Avatar>
//!     <AvatarImage src="/shadcn.png" />
//!     <AvatarFallback>"CN"</AvatarFallback>
//! </Avatar>
//!
//! <Avatar size="lg"><AvatarFallback>"CN"</AvatarFallback></Avatar>
//! ```
//!
//! Write the fallback *before* the image, which is the order shadcn writes them
//! in anyway.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

use crate::components::rahti_ui::attrs::{Attrs, Bindings, is_binding};
use crate::components::rahti_ui::utils::cn;
use crate::rahti::{Html, component, html};

/// The round box. shadcn's string, unchanged.
///
/// `group/avatar` is what the fallback's text size and the badge's dimensions
/// read, so the size is set once here and three other parts follow it.
pub const ROOT: &str = "group/avatar relative flex size-8 shrink-0 overflow-hidden \
                        rounded-full select-none data-[size=lg]:size-10 data-[size=sm]:size-6";

/// The picture. shadcn's string, unchanged.
pub const IMAGE: &str = "aspect-square size-full";

/// The one utility this port adds, and the whole of what replaces Radix's load
/// tracking.
///
/// Positioning the image takes it out of flow and lays it over the fallback
/// underneath — a positioned element paints above a non-positioned sibling in
/// the same stacking context, so no `z-index` is needed and the two cannot fight.
///
/// When the image loads it covers the fallback. When it fails, or when no `src`
/// was given, an `<img>` with an empty `alt` renders nothing and the fallback is
/// simply what is there. Radix needs four load states to arrive at the same
/// picture.
pub const IMAGE_LAYER: &str = "absolute inset-0";

/// What shows until the picture does. shadcn's string, unchanged.
pub const FALLBACK: &str = "flex size-full items-center justify-center rounded-full \
                            bg-muted text-sm text-muted-foreground \
                            group-data-[size=sm]/avatar:text-xs";

/// A dot in the corner — presence, unread, whatever the page means by it.
pub const BADGE: &str = "absolute right-0 bottom-0 z-10 inline-flex items-center \
                         justify-center rounded-full bg-primary text-primary-foreground \
                         ring-2 ring-background select-none \
                         group-data-[size=sm]/avatar:size-2 \
                         group-data-[size=sm]/avatar:[&>svg]:hidden \
                         group-data-[size=default]/avatar:size-2.5 \
                         group-data-[size=default]/avatar:[&>svg]:size-2 \
                         group-data-[size=lg]/avatar:size-3 \
                         group-data-[size=lg]/avatar:[&>svg]:size-2";

/// A row of overlapping avatars.
pub const GROUP: &str = "group/avatar-group flex -space-x-2 \
                         *:data-[slot=avatar]:ring-2 *:data-[slot=avatar]:ring-background";

/// The `+3` at the end of a group, which sizes itself from the avatars beside it.
pub const GROUP_COUNT: &str = "relative flex size-8 shrink-0 items-center justify-center \
                               rounded-full bg-muted text-sm text-muted-foreground \
                               ring-2 ring-background \
                               group-has-data-[size=lg]/avatar-group:size-10 \
                               group-has-data-[size=sm]/avatar-group:size-6 \
                               [&>svg]:size-4 \
                               group-has-data-[size=lg]/avatar-group:[&>svg]:size-5 \
                               group-has-data-[size=sm]/avatar-group:[&>svg]:size-3";

/// How big the round box is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl AvatarSize {
    /// The name written into `data-size`.
    ///
    /// Load-bearing rather than decorative: [`ROOT`]'s own size reads it, and so
    /// do the fallback's text size and all three of the badge's dimensions,
    /// through `group-data-[size=…]/avatar:`.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }

    /// The [`slug`](Self::slug) read back, for the `size="…"` prop.
    pub fn parse(name: &str) -> Self {
        match name {
            "" | "default" => Self::Default,
            "sm" => Self::Sm,
            "lg" => Self::Lg,
            other => {
                debug_assert!(
                    false,
                    "rahti-ui: `{other}` is not an Avatar size. One of: default, sm, lg."
                );
                Self::Default
            }
        }
    }
}

/// Which of the six a set of props is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Part {
    #[default]
    Root,
    Image,
    Fallback,
    Badge,
    Group,
    GroupCount,
}

impl Part {
    /// The `data-slot` this part carries — shadcn's, unchanged.
    ///
    /// The root's is load-bearing: [`GROUP`] rings its children with
    /// `*:data-[slot=avatar]:ring-2`, which finds them by this string.
    const fn slot(self) -> &'static str {
        match self {
            Self::Root => "avatar",
            Self::Image => "avatar-image",
            Self::Fallback => "avatar-fallback",
            Self::Badge => "avatar-badge",
            Self::Group => "avatar-group",
            Self::GroupCount => "avatar-group-count",
        }
    }

    /// The element this part renders as.
    ///
    /// The root is a `<span>` rather than a `<div>`, which is Radix's own choice
    /// and matters here: an avatar's usual home is inside a `<button>` — a
    /// [`Sidebar`](super::sidebar) footer, a menu label — and a `<div>` there is
    /// not markup.
    const fn element(self) -> &'static str {
        match self {
            Self::Image => "img",
            Self::Root | Self::Fallback | Self::Badge => "span",
            Self::Group | Self::GroupCount => "div",
        }
    }
}

/// The class list a part renders with, before the caller's own.
fn part_classes(part: Part) -> String {
    match part {
        Part::Root => ROOT.to_string(),
        Part::Image => cn(&[IMAGE_LAYER, IMAGE]),
        Part::Fallback => FALLBACK.to_string(),
        Part::Badge => BADGE.to_string(),
        Part::Group => GROUP.to_string(),
        Part::GroupCount => GROUP_COUNT.to_string(),
    }
}

/// The class list an [`Avatar`] renders with, the caller's own included.
///
/// Write `data-size` alongside it: the root's own size reads it, and so do the
/// fallback and the badge through the group.
pub fn avatar_variants(class: &str) -> String {
    cn(&[ROOT, class])
}

/// [`AvatarImage`]'s class list, the caller's own included.
///
/// Includes [`IMAGE_LAYER`], which is what lays it over the fallback. A page
/// writing its own `<img>` needs that as much as the component does.
pub fn avatar_image_variants(class: &str) -> String {
    cn(&[IMAGE_LAYER, IMAGE, class])
}

/// [`AvatarFallback`]'s class list, the caller's own included.
pub fn avatar_fallback_variants(class: &str) -> String {
    cn(&[FALLBACK, class])
}

/// [`AvatarBadge`]'s class list, the caller's own included.
pub fn avatar_badge_variants(class: &str) -> String {
    cn(&[BADGE, class])
}

/// [`AvatarGroup`]'s class list, the caller's own included.
pub fn avatar_group_variants(class: &str) -> String {
    cn(&[GROUP, class])
}

/// [`AvatarGroupCount`]'s class list, the caller's own included.
pub fn avatar_group_count_variants(class: &str) -> String {
    cn(&[GROUP_COUNT, class])
}

/// One Avatar part's props, with shadcn's defaults already in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarProps {
    part: Part,
    size: AvatarSize,
    src: String,
    alt: String,
    class: String,
    attrs: Attrs,
}

impl Default for AvatarProps {
    fn default() -> Self {
        Self {
            part: Part::Root,
            size: AvatarSize::Default,
            src: String::new(),
            alt: String::new(),
            class: String::new(),
            attrs: Attrs::new(),
        }
    }
}

/// A default [`Avatar`]'s props — the start of every chain.
pub fn avatar() -> AvatarProps {
    AvatarProps::of(Part::Root)
}

/// A default [`AvatarImage`]'s props.
pub fn avatar_image() -> AvatarProps {
    AvatarProps::of(Part::Image)
}

/// A default [`AvatarFallback`]'s props.
pub fn avatar_fallback() -> AvatarProps {
    AvatarProps::of(Part::Fallback)
}

/// A default [`AvatarBadge`]'s props.
pub fn avatar_badge() -> AvatarProps {
    AvatarProps::of(Part::Badge)
}

/// A default [`AvatarGroup`]'s props.
pub fn avatar_group() -> AvatarProps {
    AvatarProps::of(Part::Group)
}

/// A default [`AvatarGroupCount`]'s props.
pub fn avatar_group_count() -> AvatarProps {
    AvatarProps::of(Part::GroupCount)
}

impl AvatarProps {
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
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    /// How big the round box is.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// The picture.
    pub fn src(mut self, src: impl Into<String>) -> Self {
        self.src = src.into();
        self
    }

    /// What a screen reader calls the picture. Empty by default, and usually
    /// right that way — see the module docs.
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    /// The part's own id.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
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

    /// An attribute the browser runtime keeps up to date.
    ///
    /// `src` is the one worth binding, and it is the one to guard: a nullish
    /// value leaves a *present* empty `src`, which re-requests the current
    /// document. Write `src={avatar ? avatar : placeholder}` rather than
    /// `src={avatar}`.
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
    /// No part of an Avatar ever ships a script — the fallback is a stacking
    /// order — so this only ever answers for a caller reaching for
    /// [`render`](Self::render).
    pub fn needs_scope(&self) -> bool {
        self.attrs
            .iter()
            .any(|(_, value)| value.is_some_and(is_binding))
    }

    /// The open tag's attributes, in shadcn's order.
    fn attr_list(&self) -> Attrs {
        let mut base = Attrs::new().set("data-slot", self.part.slot());

        base = match self.part {
            Part::Root => base.set("data-size", self.size.slug()),
            Part::Image => {
                let mut image = base
                    // Always written, and usually empty. A broken `<img>` with a
                    // non-empty `alt` draws its text over the fallback, which is
                    // the thing this arrangement exists to avoid.
                    .set("alt", self.alt.as_str());
                if !self.src.trim().is_empty() {
                    image = image.set("src", self.src.trim());
                }
                image
            }
            _ => base,
        };

        base = if self.class.trim().is_empty() {
            base.set("class", self.own_classes())
        } else {
            base.set(
                "class",
                crate::components::rahti_ui::utils::cn(&[&self.own_classes(), &self.class]),
            )
        };

        merge(base, &self.attrs)
    }

    /// The rendered element, with no boundary around it and no `<script>` in it.
    ///
    /// Every part renders this way — there is no script anywhere in the file —
    /// so this is the same markup the tags produce.
    pub fn render(&self, children: Html) -> Html {
        let attrs = self.attr_list();

        match self.part.element() {
            "img" => html! {
                <img @{..attrs} />
            },
            "span" => html! {
                <span @{..attrs}>@{children}</span>
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

/// The three lines every part repeats.
fn build(mut props: AvatarProps, id: &str, attrs: Attrs, bindings: Bindings) -> AvatarProps {
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

/// A round picture, and something to show while it is not there.
///
/// ```ignore
/// <Avatar>
///     <AvatarFallback>"CN"</AvatarFallback>
///     <AvatarImage src="/shadcn.png" />
/// </Avatar>
///
/// <Avatar size="lg" class="rounded-lg">…</Avatar>
/// ```
///
/// A `<span>`, because an avatar's usual home is inside a button. `size` is
/// written as `data-size` and three other parts read it through the group, so
/// the size is set once here.
///
/// No script: the fallback is underneath and the image is laid over it, which is
/// the whole of what Radix's four load states were tracking. The module docs
/// work through why that is enough.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn Avatar(
    class: &str,
    size: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(
        avatar().class(class).size(AvatarSize::parse(size)),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    html! {
        <span @{..attrs}>
            <slot />
        </span>
    }
}

/// The picture.
///
/// ```ignore
/// <AvatarImage src="/shadcn.png" />
/// <AvatarImage src="/shadcn.png" alt="Colm Tuite" />
/// ```
///
/// Laid over the fallback by [`IMAGE_LAYER`] rather than swapped in by a load
/// event. `alt` is empty unless given, which is both the accessible choice — the
/// name is nearly always beside the picture — and the working one: a broken
/// `<img>` with alt text draws it over the fallback.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AvatarImage(
    class: &str,
    src: &str,
    alt: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
) -> Html {
    let props = build(
        avatar_image().class(class).src(src).alt(alt),
        id,
        attrs,
        bindings,
    );
    let attrs = props.attr_list();

    html! {
        <img @{..attrs} />
    }
}

/// What shows until the picture does — usually two initials.
///
/// Write it *before* the image, which is the order shadcn writes it in anyway:
/// the image is positioned and paints over whatever came first.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AvatarFallback(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(avatar_fallback().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <span @{..attrs}>
            <slot />
        </span>
    }
}

/// A dot in the avatar's corner.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AvatarBadge(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(avatar_badge().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <span @{..attrs}>
            <slot />
        </span>
    }
}

/// A row of overlapping avatars.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AvatarGroup(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(avatar_group().class(class), id, attrs, bindings);
    let attrs = props.attr_list();

    html! {
        <div @{..attrs}>
            <slot />
        </div>
    }
}

/// The `+3` at the end of a group, sized from the avatars beside it.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn AvatarGroupCount(
    class: &str,
    id: &str,
    bindings: Bindings,
    attrs: Attrs,
    children: Html,
) -> Html {
    let props = build(avatar_group_count().class(class), id, attrs, bindings);
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
        Attrs, Avatar, AvatarFallback, AvatarGroup, AvatarImage, AvatarProps, AvatarSize, FALLBACK,
        IMAGE_LAYER, avatar, avatar_fallback, avatar_image, avatar_image_variants,
    };
    use crate::components::rahti_ui::utils::TestHtml as _;
    use crate::rahti::Html;

    #[test]
    fn the_root_is_a_span_carrying_its_size() {
        let markup = Avatar(
            "",
            "lg",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("CN"),
        )
        .into_test_string();

        assert!(markup.starts_with("<span "), "{markup}");
        assert!(markup.contains(r#"data-slot="avatar""#), "{markup}");
        assert!(markup.contains(r#"data-size="lg""#), "{markup}");
        assert!(markup.contains("group/avatar"), "{markup}");
    }

    /// The one utility that replaces Radix's four load states.
    #[test]
    fn the_image_is_laid_over_the_fallback() {
        let list = avatar_image_variants("");

        assert!(list.starts_with(IMAGE_LAYER), "{list}");
        assert!(list.contains("absolute"), "{list}");
        // Positioned over a non-positioned fallback, so no z-index is needed.
        assert!(!list.contains("z-"), "{list}");
        assert!(!FALLBACK.contains("absolute"), "{FALLBACK}");
    }

    /// Empty unless given — the working choice as much as the accessible one.
    #[test]
    fn the_alt_is_written_and_empty_by_default() {
        let bare = AvatarImage(
            "",
            "/a.png",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();
        let named = AvatarImage(
            "",
            "/a.png",
            "Colm Tuite",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(bare.starts_with("<img "), "{bare}");
        assert!(bare.contains(r#"alt="""#), "{bare}");
        assert!(bare.contains(r#"src="/a.png""#), "{bare}");
        assert!(named.contains(r#"alt="Colm Tuite""#), "{named}");
    }

    /// An empty `src` is a present, empty attribute, which re-requests the
    /// current document — so it is left off entirely instead.
    #[test]
    fn no_src_means_no_src_attribute() {
        let markup = AvatarImage(
            "",
            "",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
        )
        .into_test_string();

        assert!(!markup.contains("src="), "{markup}");
    }

    #[test]
    fn the_fallback_is_shadcns_and_reads_the_size_through_the_group() {
        let markup = AvatarFallback(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::escape("CN"),
        )
        .into_test_string();

        assert!(
            markup.contains(r#"data-slot="avatar-fallback""#),
            "{markup}"
        );
        assert!(
            markup.contains("group-data-[size=sm]/avatar:text-xs"),
            "{markup}"
        );
        assert!(markup.contains("CN"), "{markup}");
    }

    /// The group rings its avatars by their `data-slot`, so that string is
    /// load-bearing rather than decorative.
    #[test]
    fn the_group_finds_its_avatars_by_their_slot() {
        let markup = AvatarGroup(
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();

        assert!(markup.contains("*:data-[slot=avatar]:ring-2"), "{markup}");
    }

    /// No part of an Avatar ever ships a script.
    #[test]
    fn nothing_here_needs_a_scope() {
        assert!(!avatar().needs_scope());
        assert!(!avatar_image().src("/a.png").needs_scope());
        assert!(!avatar_fallback().needs_scope());
        assert!(!avatar().class("rounded-lg").needs_scope());

        let markup = Avatar(
            "rounded-lg",
            "",
            "",
            crate::rahti::Bindings::default(),
            Attrs::new(),
            Html::default(),
        )
        .into_test_string();
        assert!(!markup.contains("<script"), "{markup}");
    }

    #[test]
    fn an_unknown_size_falls_back_to_shadcns() {
        assert_eq!(AvatarSize::parse(""), AvatarSize::Default);
        assert_eq!(AvatarSize::parse("sm"), AvatarSize::Sm);
        assert_eq!(AvatarProps::default().size, AvatarSize::Default);
    }

    #[test]
    fn the_parts_render_outside_a_tag() {
        let markup = avatar()
            .size(AvatarSize::Sm)
            .render(Html::concat([
                avatar_fallback().render(Html::escape("CN")),
                avatar_image().src("/a.png").render(Html::default()),
            ]))
            .into_test_string();

        assert!(markup.contains(r#"data-size="sm""#), "{markup}");
        assert!(
            markup.contains(r#"data-slot="avatar-fallback""#),
            "{markup}"
        );
        assert!(markup.contains(r#"data-slot="avatar-image""#), "{markup}");
        assert!(!markup.contains("<script"), "{markup}");
    }
}

#[cfg(test)]
mod audit {
    use super::{BADGE, FALLBACK, GROUP, GROUP_COUNT, IMAGE, IMAGE_LAYER, ROOT};
    use crate::components::rahti_ui::utils::audit::clashes;
    use crate::components::rahti_ui::utils::cn;

    /// The class lists are joined rather than merged, so nothing in one may
    /// answer a question another has already answered.
    #[test]
    fn no_utility_contradicts_another() {
        for (name, list) in [
            ("ROOT", ROOT.to_string()),
            ("IMAGE_LAYER + IMAGE", cn(&[IMAGE_LAYER, IMAGE])),
            ("FALLBACK", FALLBACK.to_string()),
            ("BADGE", BADGE.to_string()),
            ("GROUP", GROUP.to_string()),
            ("GROUP_COUNT", GROUP_COUNT.to_string()),
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

    /// The stacking order the fallback depends on: the image positioned, the
    /// fallback not. Give the fallback a position and the image disappears
    /// behind it.
    #[test]
    fn only_the_image_is_positioned() {
        assert!(IMAGE_LAYER.contains("absolute"), "{IMAGE_LAYER}");
        assert!(!FALLBACK.contains("absolute"), "{FALLBACK}");
        assert!(!FALLBACK.contains("relative"), "{FALLBACK}");
        assert!(ROOT.contains("relative"), "{ROOT}");
    }
}
