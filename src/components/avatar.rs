//! A person, drawn as initials on a disc, with an optional presence badge.

use crate::rahti::{Html, component, html};

/// Initials on a disc.
///
/// `size` is one of `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"`; `presence` is one
/// of `"online"`, `"away"`, `"busy"`, `"offline"`, or `"none"` for a disc with
/// no badge at all.
#[component]
pub fn Avatar(initials: &str, size: &str, presence: &str) -> Html {
    let (frame, text, badge) = match size {
        "xs" => ("h-6 w-6", "text-[10px]", "h-2 w-2 border"),
        "sm" => ("h-7 w-7", "text-[11px]", "h-2.5 w-2.5 border-2"),
        "lg" => ("h-10 w-10", "text-sm", "h-3 w-3 border-2"),
        "xl" => ("h-28 w-28", "text-4xl", "h-6 w-6 border-4"),
        _ => ("h-8 w-8", "text-xs", "h-2.5 w-2.5 border-2"),
    };

    let dot = match presence {
        "away" => "bg-away",
        "busy" => "bg-busy",
        "offline" => "bg-offline",
        _ => "bg-online",
    };

    let frame_class = format!("relative inline-flex shrink-0 {frame}");
    let disc_class = format!(
        "flex h-full w-full items-center justify-center rounded-full bg-avatar font-semibold tracking-wide text-avatar-ink {text}"
    );
    let badge_class =
        format!("absolute -bottom-0.5 -right-0.5 rounded-full border-chrome {badge} {dot}");

    html! {
        <span class=@{frame_class}>
            <span class=@{disc_class}>@{initials}</span>
            <span class=@{badge_class} hidden=@{presence == "none"}></span>
        </span>
    }
}
