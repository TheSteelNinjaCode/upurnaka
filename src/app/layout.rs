use crate::rahti::{Html, Metadata, html};

/// Site-wide metadata defaults. A nested layout or page only repeats the
/// fields it needs to override.
pub fn metadata() -> Metadata {
    Metadata::new()
        .default_title("upurnaka")
        .title_template("{title} | upurnaka")
        .description("upurnaka, built with Rahti")
}

/// The root layout: the document shell every page renders inside.
///
/// `children` is the page — or the next layout down — already rendered.
/// `<slot />` is where it lands.
///
/// The slot is also where Rahti puts this layout's identity. `html!` names
/// every convention file for the client runtime, and the single parent here is
/// `<html>` — naming that would make the whole document a component to
/// re-render, `<head>` and all. So a document is named around its slot
/// instead: the rendered page carries `<template pp-component="…">` there,
/// derived from this file's path. Nothing to write, and nothing to keep in
/// step.
pub fn layout(children: Html) -> Html {
    html! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <link rel="icon" href="/favicon.ico" type="image/x-icon" />
                <link href="/css/styles.css" rel="stylesheet" />
                <script type="module" src="/js/main.js"></script>
            </head>
            <body>
                <slot />
            </body>
        </html>
    }
}
