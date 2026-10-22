//! The one place a glyph is drawn.
//!
//! Every icon in the shell is a 24x24 stroked outline, so a size and a colour
//! are the only things a caller decides: `class` carries both.

use crate::rahti::{Html, component, html};

/// A named outline icon.
///
/// `name` is matched against the closed list below. An unknown name renders an
/// empty frame rather than nothing, so a typo shows up as a gap in the layout
/// instead of a silently collapsed row.
#[component]
pub fn Icon(name: &str, class: &str) -> Html {
    let body = match name {
        "chat" => html! {
            <path
                d="M20 11.4c0 3.8-3.6 6.9-8 6.9a9.4 9.4 0 0 1-2.4-.3L5 19.7l1.3-3.4A6.6 6.6 0 0 1 4 11.4c0-3.8 3.6-6.9 8-6.9s8 3.1 8 6.9Z"
            />
        },
        "video" => html! {
            <g>
                <rect x="3" y="6" width="12.2" height="12" rx="2.6" />
                <path d="M15.2 10.4 20 7.9a.6.6 0 0 1 .9.5v7.2a.6.6 0 0 1-.9.5l-4.8-2.5Z" />
            </g>
        },
        "contacts" => html! {
            <g>
                <rect x="3" y="5" width="18" height="14" rx="2.6" />
                <circle cx="9.2" cy="10.6" r="2.2" />
                <path d="M6 15.9a3.7 3.7 0 0 1 6.4 0" />
                <path d="M15.2 9.6h3.4M15.2 13.2h3.4" />
            </g>
        },
        "copilot" => html! {
            <g>
                <defs>
                    <linearGradient id="ppCopilot" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#36c5f0" />
                        <stop offset="45%" stop-color="#7b61ff" />
                        <stop offset="100%" stop-color="#e95eb0" />
                    </linearGradient>
                </defs>
                <path
                    d="M12.6 4c3.2 0 4.9 1.9 5.9 4.6.8 2.4-.3 3.9-2.4 3.9h-3c-.9 0-1.3.4-1.6 1.3-.5 1.5-1.5 3.1-3.7 3.1-1.9 0-3.4-1.5-3.4-3.5 0-2.3 1.5-3.4 3.3-4 1-.4 1.4-.7 1.6-1.7C9.8 5.4 10.8 4 12.6 4Z"
                    fill="url(#ppCopilot)"
                    stroke="none"
                />
                <path
                    d="M18.4 15.4c0 1.9-1.4 3.4-3.3 3.9-1.2.3-2 .7-2.5 1.3"
                    stroke="url(#ppCopilot)"
                    stroke-width="1.8"
                />
            </g>
        },
        "communities" | "people" => html! {
            <g>
                <circle cx="9.2" cy="8.9" r="3" />
                <path d="M3.7 18.6a5.5 5.5 0 0 1 11 0" />
                <circle cx="17.2" cy="10.4" r="2.2" />
                <path d="M16.1 15a4.6 4.6 0 0 1 4.5 3.3" />
            </g>
        },
        "calendar" => html! {
            <g>
                <rect x="3.4" y="5.1" width="17.2" height="15.3" rx="2.6" />
                <path d="M3.4 9.6h17.2M8 3.4v3.3M16 3.4v3.3" />
            </g>
        },
        "calendar-day" => html! {
            <g>
                <rect x="3.4" y="5.1" width="17.2" height="15.3" rx="2.6" />
                <path d="M3.4 9.6h17.2M8 3.4v3.3M16 3.4v3.3" />
                <rect x="6.9" y="12" width="4.6" height="4.2" rx="1.1" />
            </g>
        },
        "bell" => html! {
            <g>
                <path d="M18 15.6V11a6 6 0 1 0-12 0v4.6L4.6 18h14.8L18 15.6Z" />
                <path d="M9.8 20.4a2.4 2.4 0 0 0 4.4 0" />
            </g>
        },
        "apps" => html! {
            <g>
                <path d="M8 3.3h8l4 5.2-8 12.2-8-12.2 4-5.2Z" />
                <path d="M4 8.5h16M8 3.3l4 17.4M16 3.3l-4 17.4" />
            </g>
        },
        "teams" => html! {
            <g>
                <rect
                    x="2.2"
                    y="2.2"
                    width="19.6"
                    height="19.6"
                    rx="5.2"
                    fill="#5b5fc7"
                    stroke="none"
                />
                <path d="M7.4 8.1h9.2M12 8.1v8.2" stroke="#ffffff" stroke-width="2.1" />
            </g>
        },
        "panel" => html! {
            <g>
                <rect x="3.1" y="5.1" width="17.8" height="13.8" rx="2.6" />
                <path d="M9.8 5.1v13.8" />
                <path d="M5.8 9.4h1.9M5.8 12h1.9" />
            </g>
        },
        "chevron-left" => html! {
            <path d="m14.4 5.8-6.2 6.2 6.2 6.2" />
        },
        "chevron-right" => html! {
            <path d="m9.6 5.8 6.2 6.2-6.2 6.2" />
        },
        "chevron-down" => html! {
            <path d="m6 9.4 6 6 6-6" />
        },
        "chevron-up" => html! {
            <path d="m6 14.6 6-6 6 6" />
        },
        "search" => html! {
            <g>
                <circle cx="11" cy="11" r="6.1" />
                <path d="m15.5 15.5 4.2 4.2" />
            </g>
        },
        "more" => html! {
            <g fill="currentColor" stroke="none">
                <circle cx="5.6" cy="12" r="1.45" />
                <circle cx="12" cy="12" r="1.45" />
                <circle cx="18.4" cy="12" r="1.45" />
            </g>
        },
        "minimize" => html! {
            <path d="M5.5 12h13" />
        },
        "maximize" => html! {
            <rect x="5.6" y="5.6" width="12.8" height="12.8" rx="1.6" />
        },
        "close" => html! {
            <path d="m6.2 6.2 11.6 11.6M17.8 6.2 6.2 17.8" />
        },
        "compose" => html! {
            <g>
                <path
                    d="M19 12.6V18a2.6 2.6 0 0 1-2.6 2.6H7.6A2.6 2.6 0 0 1 5 18V9.2a2.6 2.6 0 0 1 2.6-2.6h5.2"
                />
                <path d="m16.9 3.6 3.5 3.5-7.6 7.6-4.3.8.8-4.3 7.6-7.6Z" />
            </g>
        },
        "emoji" => html! {
            <g>
                <circle cx="12" cy="12" r="8" />
                <path d="M9.3 10.1h.02M14.7 10.1h.02" stroke-width="2.1" />
                <path d="M8.8 14a4 4 0 0 0 6.4 0" />
            </g>
        },
        "attach" => html! {
            <path
                d="M19.4 11.1 12 18.5a4.4 4.4 0 0 1-6.2-6.2l7.7-7.7a2.9 2.9 0 1 1 4.1 4.1l-7.7 7.7a1.5 1.5 0 0 1-2-2.1l7-7"
            />
        },
        "image" => html! {
            <g>
                <rect x="3.4" y="5.1" width="17.2" height="13.8" rx="2.6" />
                <circle cx="9" cy="10" r="1.6" />
                <path d="m4.2 17.4 5-5 4.4 4.4 2.6-2.6 4.1 4.1" />
            </g>
        },
        "format" => html! {
            <g>
                <path d="M4.6 7.3h8.2M8.7 7.3v9.4" />
                <path d="M13.8 12.6h5.6M16.6 12.6v4.1" />
            </g>
        },
        "plus" => html! {
            <path d="M12 5.8v12.4M5.8 12h12.4" />
        },
        "send" => html! {
            <path d="m4.2 11.8 15.6-6.6-6.6 15.6-2.5-6.5-6.5-2.5Z" />
        },
        "settings" => html! {
            <g>
                <circle cx="12" cy="12" r="6.2" />
                <circle cx="12" cy="12" r="2.6" />
                <path
                    d="M12 3.4v2.3M12 18.3v2.3M20.6 12h-2.3M5.7 12H3.4M17.95 6.05l-1.6 1.6M7.65 16.35l-1.6 1.6M17.95 17.95l-1.6-1.6M7.65 7.65l-1.6-1.6"
                />
            </g>
        },
        "filter" => html! {
            <path d="M4.4 6.6h15.2M7.4 12h9.2M10.4 17.4h3.2" />
        },
        "check-circle" => html! {
            <g>
                <circle cx="12" cy="12" r="8" />
                <path d="m8.4 12.1 2.5 2.5 4.7-5" />
            </g>
        },
        "check" => html! {
            <path d="m5.4 12.4 4.3 4.3 8.9-9.4" />
        },
        "person-add" => html! {
            <g>
                <circle cx="9.8" cy="8.8" r="3.2" />
                <path d="M4 19.2a5.9 5.9 0 0 1 11.6 0" />
                <path d="M18.6 8v5.2M16 10.6h5.2" />
            </g>
        },
        "link" => html! {
            <g>
                <path d="M10.6 13.4a3.6 3.6 0 0 0 5.1 0l2.4-2.4a3.6 3.6 0 0 0-5.1-5.1l-1.2 1.2" />
                <path d="M13.4 10.6a3.6 3.6 0 0 0-5.1 0l-2.4 2.4a3.6 3.6 0 0 0 5.1 5.1l1.2-1.2" />
            </g>
        },
        "hash" => {
            html! {
                <path d="M4.8 9.2h14.4M4.8 14.8h14.4M10.2 4.2 8.4 19.8M15.6 4.2l-1.8 15.6" />
            }
        }
        "sparkle" => html! {
            <path d="M12 3.6 13.8 9l5.4 1.8-5.4 1.8L12 18l-1.8-5.4L4.8 10.8 10.2 9 12 3.6Z" />
        },
        _ => html! {
            <path d="M0 0" />
        },
    };

    html! {
        <svg
            class=@{class}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            focusable="false"
        >
            @{body}
        </svg>
    }
}
