//! A person, drawn as initials on a disc, with an optional presence badge.

use crate::components::rahti_ui::avatar::{Avatar as UiAvatar, AvatarFallback};
use crate::rahti::{Html, component, html};

/// The signed-in person's saved photo, with the standard initials fallback.
#[component]
pub fn ProfileAvatar(size: &str, presence: &str) -> Html {
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();
    let photo = if user.has_picture {
        html! {
            <img
                src="/profile-picture"
                alt=""
                class="absolute inset-0 h-full w-full rounded-full object-cover"
            />
        }
    } else {
        Html::empty()
    };
    let frame = match size {
        "lg" => "relative inline-flex h-10 w-10 shrink-0",
        "xl" => "relative inline-flex h-28 w-28 shrink-0",
        _ => "relative inline-flex h-8 w-8 shrink-0",
    };
    html! {
        <span class=@{frame}>
            <Avatar initials=@{initials.as_str()} size=@{size} presence=@{presence} />
            @{photo}
        </span>
    }
}

/// Initials on a disc.
///
/// `size` is one of `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"`; `presence` is one
/// of `"online"`, `"away"`, `"busy"`, `"offline"`, or `"none"` for a disc with
/// no badge at all.
#[component]
pub fn Avatar(initials: &str, size: &str, presence: &str) -> Html {
    let (frame, text, badge) = match size {
        "xs" => ("h-6 w-6", "text-[10px]!", "h-2 w-2 border"),
        "sm" => ("h-7 w-7", "text-[11px]!", "h-2.5 w-2.5 border-2"),
        "lg" => ("h-10 w-10", "text-sm!", "h-3 w-3 border-2"),
        "xl" => ("h-28 w-28", "text-4xl!", "h-6 w-6 border-4"),
        _ => ("h-8 w-8", "text-xs!", "h-2.5 w-2.5 border-2"),
    };

    let dot = match presence {
        "away" => "bg-away",
        "busy" => "bg-busy",
        "offline" => "bg-offline",
        _ => "bg-online",
    };

    let frame_class = format!("relative inline-flex shrink-0 {frame}");
    let disc_class = format!(
        "flex h-full w-full items-center justify-center rounded-full bg-avatar! font-semibold tracking-wide text-avatar-ink! {text}"
    );
    let badge_class =
        format!("absolute -bottom-0.5 -right-0.5 z-10 rounded-full border-chrome {badge} {dot}");

    html! {
        <span class=@{frame_class}>
            <UiAvatar class="size-full!">
                <AvatarFallback class=@{disc_class.as_str()}>@{initials}</AvatarFallback>
            </UiAvatar>
            <span class=@{badge_class} hidden=@{presence == "none"}></span>
        </span>
    }
}
