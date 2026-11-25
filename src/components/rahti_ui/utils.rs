// rahti-ui: utils
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.6.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! Class-list composition shared by the component corpus.
//!
//! Rust joins authored utility layers; conflict resolution belongs to the
//! existing frontend `twMerge` exposed by `public/js/main.js`.

// A component kit ships its whole API, and an application uses the part it
// needs. What this file leaves unused is that surface, not dead code, so the
// lint is off for this file alone; the application's own components keep it.
#![allow(dead_code)]

/// Join nonempty class layers, preserving first-seen order without duplicates.
/// This composes strings; it does not resolve Tailwind conflicts.
pub fn cn(parts: &[&str]) -> String {
    let mut classes = Vec::new();
    for class in parts.iter().flat_map(|part| part.split_whitespace()) {
        if !classes.contains(&class) {
            classes.push(class);
        }
    }
    classes.join(" ")
}

/// A Rust string as a double-quoted JavaScript literal, for an expression
/// that ends up inside an HTML attribute.
///
/// The class lists this quotes carry `'` (`[class*='size-']`) and `>`
/// (`has-[>svg]:`), so the quoting has to survive both the JavaScript
/// parser and the HTML one. Double quotes here leave the apostrophes alone,
/// and `AttrList` escapes the rest on the way into the attribute.
pub fn js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // A literal `</script` would end a surrounding script element,
            // and these strings are also seeded into `<script>` blocks.
            '<' => out.push_str("\\u003C"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The layer-independence check every component's class strings are held to.
///
/// A component here builds its class list by *joining* layers rather than
/// merging them (see [`cn`]), which is only exact if no two utilities in the
/// result compete. This is what proves that: splitting a utility into its
/// variant prefix and its conflict family is what a tailwind-merge does, and
/// this does as much of it as the strings in this library actually use.
///
/// Test-only. Each component owns the assertion — `input.rs` checks one list,
/// `button.rs` checks all 48 variant×size pairs — and this owns the reading.
#[cfg(test)]
pub mod audit {
    /// `dark:hover:bg-input/50` -> (`dark:hover`, `bg-input/50`).
    ///
    /// Splits on `:` only outside brackets, so
    /// `[&_svg:not([class*='size-'])]:size-4` keeps its arbitrary variant
    /// whole.
    pub fn split(class: &str) -> (String, &str) {
        let mut depth = 0i32;
        let mut last = None;
        for (i, c) in class.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                ':' if depth == 0 => last = Some(i),
                _ => {}
            }
        }
        match last {
            Some(i) => (class[..i].to_string(), &class[i + 1..]),
            None => (String::new(), class),
        }
    }

    /// The group a utility competes in, for the groups this library's strings
    /// touch. `None` means nothing here can conflict with it.
    ///
    /// Deliberately partial. A full conflict map is tailwind-merge's job and
    /// is handled by frontend `twMerge`; this only has to be complete enough to catch a
    /// clash the components could actually introduce.
    pub fn family(utility: &str) -> Option<&'static str> {
        let name = utility.split('/').next().unwrap_or(utility);
        // `text-[0.8rem]` is a font size and `text-[#fff]` is a colour, under
        // one prefix. Tailwind guesses from the value's shape and so does this:
        // a length starts with a digit or a dot, and anything else is a colour
        // unless it says otherwise. A Calendar's weekday writes the first kind
        // beside `text-muted-foreground`, which is the second.
        let is_arbitrary_text_size = name.starts_with("text-[")
            && (name["text-[".len()..].starts_with(|c: char| c.is_ascii_digit() || c == '.')
                || name.starts_with("text-[calc(")
                || name.starts_with("text-[length:"));
        let is_text_size = is_arbitrary_text_size
            || matches!(
                name,
                "text-xs" | "text-sm" | "text-base" | "text-lg" | "text-xl"
            );
        // `text-` is three properties wearing one prefix, and a Field's
        // description sets two of them at once: where the text sits and what
        // colour it is. tailwind-merge keeps alignment, wrapping and colour
        // apart — `text-left` and `text-muted-foreground` are not two answers
        // to the same question — and so does this.
        let is_text_align = matches!(
            name,
            "text-left" | "text-center" | "text-right" | "text-justify" | "text-start" | "text-end"
        );
        let is_text_wrap = matches!(
            name,
            "text-wrap" | "text-nowrap" | "text-balance" | "text-pretty"
        );
        // Exact names only. `flex` is a display and `flex-1`, `flex-col` and
        // `flex-wrap` are three other properties that merely start the same
        // way — an Input Group lays `flex` over a Button's `inline-flex` and
        // keeps `flex-1` on the control, and reading those as one family would
        // call a working class list broken.
        let is_display = matches!(
            name,
            "flex"
                | "inline-flex"
                | "grid"
                | "inline-grid"
                | "block"
                | "inline-block"
                | "inline"
                | "hidden"
                | "contents"
                | "flow-root"
                | "table"
                | "list-item"
        );
        // `justify-items-` and `justify-self-` are their own properties, so
        // this is exact too.
        let is_justify = matches!(
            name,
            "justify-start"
                | "justify-end"
                | "justify-center"
                | "justify-between"
                | "justify-around"
                | "justify-evenly"
                | "justify-normal"
                | "justify-stretch"
        );
        let is_items = matches!(
            name,
            "items-start" | "items-end" | "items-center" | "items-baseline" | "items-stretch"
        );
        Some(match () {
            _ if is_display => "display",
            _ if is_justify => "justify-content",
            _ if is_items => "align-items",
            _ if name.starts_with("gap-") => "gap",
            _ if is_text_size => "text-size",
            _ if is_text_align => "text-align",
            _ if is_text_wrap => "text-wrap",
            _ if name.starts_with("text-") => "text-color",
            // Split the same way `ring-` is, and for the same reason: a
            // Resizable's handle writes `ring-offset-1` and
            // `ring-offset-background` together, which are a width and a colour
            // rather than two answers to one question. Only the numeric one can
            // be told apart by shape.
            _ if name.starts_with("ring-offset-") && name[12..].parse::<u32>().is_ok() => {
                "ring-offset-width"
            }
            _ if name.starts_with("ring-offset-") => "ring-offset-color",
            _ if name == "ring" || name.starts_with("ring-[") => "ring-width",
            _ if name.starts_with("ring-") && name[5..].parse::<u32>().is_ok() => "ring-width",
            _ if name.starts_with("ring-") => "ring-color",
            _ if name.starts_with("size-") => "size",
            // Before `w-`/`h-`, which these are not a longer spelling of.
            _ if name.starts_with("min-w-") => "min-width",
            _ if name.starts_with("min-h-") => "min-height",
            _ if name.starts_with("h-") => "height",
            _ if name.starts_with("w-") => "width",
            _ if name.starts_with("px-") => "padding-x",
            _ if name.starts_with("py-") => "padding-y",
            // A select splits shadcn's `px-3` into `pl-3 pr-9` to leave room
            // for the chevron, so these two are worth telling apart.
            _ if name.starts_with("pl-") => "padding-left",
            _ if name.starts_with("pr-") => "padding-right",
            _ if name.starts_with("rounded") => rounded_side(name),
            // `bg-` is four properties wearing one prefix, and a checkbox uses
            // three of them at once: a colour, an image, and where that image
            // sits. tailwind-merge keeps them apart — `bg-primary` and
            // `bg-[url(…)]` are not two answers to the same question — and so
            // does this. The image test reads `bg-[` rather than the whole
            // value because `name` has already been cut at the first `/`, and a
            // data URI is full of those.
            // A radio's dot is a `bg-[radial-gradient(…)]`: an image written as
            // an arbitrary value with no `image:` hint in front of it, because
            // a gradient function is one of the things Tailwind can recognise
            // on sight. This has to recognise it too, or the dot reads as a
            // second background *colour*.
            _ if name.starts_with("bg-[url(")
                || name.starts_with("bg-[image:")
                || name.starts_with("bg-[radial-gradient(")
                || name.starts_with("bg-[linear-gradient(")
                || name.starts_with("bg-[conic-gradient(")
                || name.starts_with("bg-[repeating-radial-gradient(")
                || name.starts_with("bg-[repeating-linear-gradient(")
                || name.starts_with("bg-[repeating-conic-gradient(") =>
            {
                "background-image"
            }
            _ if name.starts_with("bg-linear-")
                || name.starts_with("bg-radial")
                || name.starts_with("bg-conic")
                || name == "bg-none" =>
            {
                "background-image"
            }
            _ if matches!(
                name,
                "bg-repeat"
                    | "bg-no-repeat"
                    | "bg-repeat-x"
                    | "bg-repeat-y"
                    | "bg-repeat-round"
                    | "bg-repeat-space"
            ) =>
            {
                "background-repeat"
            }
            _ if name.starts_with("bg-[size:")
                || matches!(name, "bg-auto" | "bg-cover" | "bg-contain") =>
            {
                "background-size"
            }
            // A fifth property under the same prefix, and the one a Combobox's
            // chips box needs: `bg-clip-padding` decides where the background
            // is painted, not what colour it is, so it sits beside
            // `bg-transparent` rather than replacing it.
            _ if name.starts_with("bg-clip-") => "background-clip",
            _ if name.starts_with("bg-[position:")
                || matches!(
                    name,
                    "bg-center"
                        | "bg-top"
                        | "bg-bottom"
                        | "bg-left"
                        | "bg-right"
                        | "bg-left-top"
                        | "bg-left-bottom"
                        | "bg-right-top"
                        | "bg-right-bottom"
                ) =>
            {
                "background-position"
            }
            _ if name.starts_with("bg-") => "background",
            // `border`, `border-0`, `border-2` are widths; `border-input` is a
            // colour. Only the numeric ones can be told apart by shape.
            _ if name == "border" => "border-width",
            _ if name.starts_with("border-") && name[7..].parse::<u32>().is_ok() => "border-width",
            // `border-l-0` is a left width and `border-l` is one too; a Toggle
            // Group writes both to join its items into a strip. Without this
            // they would read as border *colours* and could be called a clash
            // with `border-input`, which sets something else entirely.
            _ if border_side(name).is_some() => border_side(name).unwrap(),
            // A fourth property under the same prefix. A Chart's dashed tooltip
            // swatch writes `border-(--color-border)`, `border-[1.5px]` and
            // `border-dashed` in one list — a colour, a width and a *style* —
            // and reading the last two as colours would call shadcn's own
            // string broken.
            _ if matches!(
                name,
                "border-solid"
                    | "border-dashed"
                    | "border-dotted"
                    | "border-double"
                    | "border-hidden"
                    | "border-none"
            ) =>
            {
                "border-style"
            }
            // `border-[1.5px]` is a width and `border-[#fff]` is a colour, under
            // one spelling. Told apart by shape, the same way `outline-[…]` and
            // `text-[…]` are: a length starts with a digit or a dot.
            _ if name.starts_with("border-[")
                && name["border-[".len()..]
                    .starts_with(|c: char| c.is_ascii_digit() || c == '.') =>
            {
                "border-width"
            }
            _ if name.starts_with("border-") => "border-color",
            _ if name.starts_with("shadow-") => "shadow",
            _ if name.starts_with("opacity-") => "opacity",
            _ if name.starts_with("cursor-") => "cursor",
            _ if name.starts_with("pointer-events-") => "pointer-events",
            // `outline-` is three properties under one prefix, and a Tabs
            // trigger writes two of them at once: `focus-visible:outline-1` is a
            // width and `focus-visible:outline-ring` is a colour. Split the same
            // way `ring-` and `ring-offset-` are, and for the same reason —
            // reading them as one family calls a working class list broken.
            _ if matches!(
                name,
                "outline-none"
                    | "outline-hidden"
                    | "outline-solid"
                    | "outline-dashed"
                    | "outline-dotted"
                    | "outline-double"
            ) =>
            {
                "outline-style"
            }
            _ if name.starts_with("outline-offset-") => "outline-offset",
            _ if name == "outline" => "outline-width",
            _ if name.starts_with("outline-") && name[8..].parse::<u32>().is_ok() => {
                "outline-width"
            }
            // `outline-[2px]` is a width and `outline-[#fff]` is a colour, under
            // one spelling. Tailwind guesses from the value's shape and so does
            // this: a length starts with a digit or a dot.
            _ if name.starts_with("outline-[")
                && name["outline-[".len()..]
                    .starts_with(|c: char| c.is_ascii_digit() || c == '.') =>
            {
                "outline-width"
            }
            _ if name.starts_with("outline") => "outline-color",
            _ if name.starts_with("transition") => "transition",
            _ => return None,
        })
    }

    /// `rounded`, `rounded-r`, `rounded-tl` — which corners a radius sets.
    ///
    /// tailwind-merge keeps the corner groups apart, so `rounded-md` and
    /// `rounded-r-md` are not two answers to one question: the first rounds
    /// every corner and the second re-states the right pair. A Calendar's day
    /// writes exactly that pair under one variant, which is what made this
    /// worth splitting.
    fn rounded_side(name: &str) -> &'static str {
        let rest = name.strip_prefix("rounded").unwrap_or("");
        let rest = rest.strip_prefix('-').unwrap_or(rest);
        match rest.split('-').next().unwrap_or("") {
            "s" => "rounded-s",
            "e" => "rounded-e",
            "t" => "rounded-t",
            "r" => "rounded-r",
            "b" => "rounded-b",
            "l" => "rounded-l",
            "ss" => "rounded-ss",
            "se" => "rounded-se",
            "es" => "rounded-es",
            "ee" => "rounded-ee",
            "tl" => "rounded-tl",
            "tr" => "rounded-tr",
            "br" => "rounded-br",
            "bl" => "rounded-bl",
            // `rounded`, `rounded-md`, `rounded-[3px]` — every corner.
            _ => "rounded",
        }
    }

    /// `border-l`, `border-l-0`, `border-x-2` — a width on one side or one axis.
    ///
    /// `None` for anything else, including `border-l-red-500`, which is a colour
    /// on a side and not something this library writes.
    fn border_side(name: &str) -> Option<&'static str> {
        let rest = name.strip_prefix("border-")?;
        let (side, width) = match rest.split_once('-') {
            Some((side, width)) => (side, Some(width)),
            None => (rest, None),
        };
        let family = match side {
            "t" => "border-top-width",
            "r" => "border-right-width",
            "b" => "border-bottom-width",
            "l" => "border-left-width",
            "x" => "border-x-width",
            "y" => "border-y-width",
            _ => return None,
        };
        match width {
            None => Some(family),
            Some(width) if width.parse::<u32>().is_ok() => Some(family),
            // `border-l-red-500` is a colour, and `family` says so below.
            Some(_) => None,
        }
    }

    /// Every utility in `list` that competes with an earlier one: same variant
    /// prefix, same conflict family. An empty answer means joining the layers
    /// is exactly what merging them would have produced.
    pub fn clashes(list: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen: Vec<(String, &str, &str)> = Vec::new();

        for class in list.split_whitespace() {
            let (prefix, utility) = split(class);
            let Some(family) = family(utility) else {
                continue;
            };
            match seen.iter().find(|(p, f, _)| *p == prefix && *f == family) {
                Some((_, _, first)) => {
                    out.push(format!("`{first}` and `{class}` both set {family}"));
                }
                None => seen.push((prefix, family, class)),
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::audit::{clashes, family, split};
    use super::{cn, js_string};

    #[test]
    fn joins_in_first_seen_order() {
        assert_eq!(cn(&["px-4 py-2", "rounded-md"]), "px-4 py-2 rounded-md");
    }

    #[test]
    fn drops_empties_and_repeats() {
        assert_eq!(cn(&["", "  ", "h-9", "h-9 px-4"]), "h-9 px-4");
        assert_eq!(cn(&[]), "");
    }

    /// The thing [`cn`] deliberately does not do. Left as a test so the
    /// contract is written down rather than assumed.
    #[test]
    fn conflicting_utilities_are_left_for_the_frontend() {
        assert_eq!(cn(&["bg-primary", "bg-red-500"]), "bg-primary bg-red-500");
    }

    #[test]
    fn quotes_a_class_list_without_touching_its_apostrophes() {
        assert_eq!(
            js_string("[&_svg:not([class*='size-'])]:size-4"),
            r#""[&_svg:not([class*='size-'])]:size-4""#
        );
    }

    #[test]
    fn quotes_cannot_break_out_of_the_literal() {
        assert_eq!(js_string(r#"a" + evil() + "b"#), r#""a\" + evil() + \"b""#);
    }

    /// These strings are also seeded into `<script>` blocks, where a raw
    /// `<` could end the element early.
    #[test]
    fn a_closing_script_tag_cannot_survive() {
        let quoted = js_string("</script>");
        assert!(!quoted.contains('<'), "{quoted}");
        assert_eq!(quoted, "\"\\u003C/script>\"");
    }

    /// The audit is only worth anything if it can fail, and only safe if it
    /// does not fire on utilities that merely look alike.
    #[test]
    fn the_audit_reads_a_utility_the_way_tailwind_does() {
        let (prefix, utility) = split("dark:hover:bg-input/50");
        assert_eq!(prefix, "dark:hover");
        assert_eq!(family(utility), Some("background"));

        let (prefix, utility) = split("[&_svg:not([class*='size-'])]:size-4");
        assert_eq!(prefix, "[&_svg:not([class*='size-'])]");
        assert_eq!(family(utility), Some("size"));

        // What shadcn's own base and `xs` size did to each other.
        assert_eq!(family("gap-2"), family("gap-1"));
        assert_eq!(family("text-sm"), family("text-xs"));
        // And what it does not confuse.
        assert_ne!(family("text-sm"), family("text-primary"));
        // A Field description writes `text-left` beside `text-muted-foreground`
        // and `text-sm`, and none of the three is an answer to the other two.
        assert_ne!(family("text-left"), family("text-primary"));
        assert_ne!(family("text-left"), family("text-sm"));
        assert_ne!(family("text-balance"), family("text-primary"));
        assert!(clashes("text-left text-sm text-muted-foreground").is_empty());
        // An Input Group's button lays `flex` over a Button's `inline-flex`,
        // and its control keeps `flex-1` beside both. Only the first pair is a
        // question about the same property.
        assert_eq!(family("inline-flex"), family("flex"));
        assert_eq!(family("flex"), Some("display"));
        assert_eq!(family("flex-1"), None);
        assert_eq!(family("flex-col"), None);
        assert_eq!(family("justify-center"), family("justify-start"));
        assert_ne!(family("justify-center"), family("items-center"));
        assert_eq!(clashes("inline-flex flex").len(), 1);
        assert_eq!(clashes("justify-center justify-start").len(), 1);
        assert!(clashes("flex flex-col flex-1 items-center justify-center").is_empty());
        assert_ne!(family("ring-[3px]"), family("ring-ring/50"));
        // A Calendar's weekday: an arbitrary length is a size, and a colour is
        // still a colour.
        assert_eq!(family("text-[0.8rem]"), family("text-sm"));
        assert_ne!(family("text-[0.8rem]"), family("text-muted-foreground"));
        assert_eq!(family("text-[#fff]"), family("text-muted-foreground"));
        assert!(clashes("text-[0.8rem] font-normal text-muted-foreground").is_empty());
        // A Calendar's day writes both under one variant, and tailwind-merge
        // keeps the corner groups apart — every corner, then the right pair.
        assert_ne!(family("rounded-md"), family("rounded-r-md"));
        assert_ne!(family("rounded-l-md"), family("rounded-r-md"));
        assert_eq!(family("rounded-md"), family("rounded-none"));
        assert_eq!(family("rounded"), family("rounded-[3px]"));
        assert!(clashes("rounded-md rounded-r-md").is_empty());
        assert_eq!(clashes("rounded-md rounded-none").len(), 1);
        assert_ne!(family("border-0"), family("border-input"));
        // A Toggle Group's joined strip: a width on one side, not a colour.
        assert_eq!(family("border-l-0"), Some("border-left-width"));
        assert_eq!(family("border-l"), Some("border-left-width"));
        assert_ne!(family("border-l-0"), family("border-input"));
        assert_ne!(family("border-l-0"), family("border-r-0"));
        // A colour on a side is still a colour.
        assert_eq!(family("border-l-red-500"), Some("border-color"));
        assert_ne!(family("min-w-0"), family("w-full"));
    }

    /// `bg-` is four properties, and a checkbox sets three of them at once —
    /// the colour of the box, the tick drawn over it, and where that tick
    /// sits. Reading them as one family would call a working class list
    /// broken.
    #[test]
    fn the_audit_reads_bg_the_way_tailwind_does() {
        assert_eq!(family("bg-primary"), Some("background"));
        assert_eq!(family("bg-input/30"), Some("background"));
        assert_eq!(family("bg-no-repeat"), Some("background-repeat"));
        assert_eq!(family("bg-center"), Some("background-position"));
        assert_eq!(family("bg-cover"), Some("background-size"));

        // A data URI is full of slashes, and `family` cuts at the first one
        // looking for an opacity modifier — the image test has to survive that.
        let tick = "bg-[url('data:image/svg+xml,%3csvg/%3e')]";
        assert_eq!(family(tick), Some("background-image"));
        assert_ne!(family(tick), family("bg-primary"));

        // A switch puts five of them on one element: the track's colour, the
        // thumb's image, and the size, position and repeat that place it.
        assert_eq!(family("bg-[size:1rem_1rem]"), Some("background-size"));
        assert_eq!(
            family("bg-[position:right_center]"),
            Some("background-position")
        );
        assert!(
            clashes(
                "bg-input bg-no-repeat bg-left bg-[size:1rem_1rem] \
                 bg-[radial-gradient(circle,var(--background)_0,transparent_50%)]"
            )
            .is_empty()
        );

        // An OTP field's dividers, which repeat.
        let dividers = "bg-[repeating-linear-gradient(to_right,transparent_0_1px,red_1px_2px)]";
        assert_eq!(family(dividers), Some("background-image"));
        assert!(clashes(&format!("bg-transparent {dividers}")).is_empty());

        // A radio's dot, which is an image without saying so.
        let dot = "bg-[radial-gradient(circle_at_center,var(--primary)_0,transparent_4px)]";
        assert_eq!(family(dot), Some("background-image"));
        assert_ne!(family(dot), family("bg-input/30"));

        // A tick drawn over a filled box is not a clash; two fills would be.
        assert!(clashes("checked:bg-primary checked:bg-[url('data:image/svg+xml,x')]").is_empty());
        assert_eq!(clashes("checked:bg-primary checked:bg-red-500").len(), 1);

        // A Combobox's chips box says where the background is painted as well
        // as what it is, and neither answers the other.
        assert_eq!(family("bg-clip-padding"), Some("background-clip"));
        assert_ne!(family("bg-clip-padding"), family("bg-transparent"));
        assert!(clashes("bg-transparent bg-clip-padding dark:bg-input/30").is_empty());
    }

    #[test]
    fn a_clash_is_reported_and_a_prefix_difference_is_not() {
        assert!(clashes("h-9 px-4 text-sm").is_empty());
        // Same family, same prefix — Tailwind's stylesheet order decides.
        assert_eq!(clashes("bg-primary bg-emerald-600").len(), 1);
        // Same family, different prefix — both apply, in different states.
        assert!(clashes("bg-primary hover:bg-primary/90").is_empty());
        assert!(clashes("text-base md:text-sm").is_empty());
    }
}

/// Inspect the compiled scripts belonging to rendered instances in unit tests.
/// The browser still receives only inert registry references and JSON data.
#[cfg(test)]
pub trait TestHtml {
    fn into_test_string(self) -> String;
}
#[cfg(test)]
impl TestHtml for crate::rahti::Html {
    fn into_test_string(self) -> String {
        let mut markup = normalized_test_bindings(self.into_string());
        let mut cursor = 0;
        while let Some(at) = markup[cursor..].find("<script pp-script=\"") {
            let start = cursor + at;
            let Some(end) = markup[start..].find("</script>").map(|n| start + n + 9) else {
                break;
            };
            let tag = &markup[start..end];
            let key = tag
                .split("pp-script=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            let code = crate::rahti::inventory::iter::<crate::rahti::client::ClientCode>
                .into_iter()
                .find(|entry| entry.key == key)
                .expect("compiled script registry entry");
            let data = tag
                .split("pp-data=\"")
                .nth(1)
                .and_then(|tail| tail.split('"').next())
                .unwrap_or("[]")
                .replace("&quot;", "\"")
                .replace("&#39;", "'")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&");
            let values = script_values(&data);
            let mut script = code.js.to_string();
            for (index, value) in values.iter().enumerate() {
                script = script.replace(&format!("__pp_server[{index}]"), value);
            }
            let replacement = format!("<script>{script}</script>");
            markup.replace_range(start..end, &replacement);
            cursor = start + replacement.len();
        }
        markup
    }
}

#[cfg(test)]
fn script_values(data: &str) -> Vec<&str> {
    let data = data
        .trim()
        .strip_prefix('[')
        .unwrap()
        .strip_suffix(']')
        .unwrap();
    if data.trim().is_empty() {
        return Vec::new();
    }
    let mut values = Vec::new();
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    let mut start = 0;
    for (at, ch) in data.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                values.push(data[start..at].trim());
                start = at + 1;
            }
            _ => {}
        }
    }
    values.push(data[start..].trim());
    values
}

/// Structural assertions ignore formatter-added padding around binding source.
/// Runtime HTML and registry lookups retain the original source byte for byte.
#[cfg(test)]
fn normalized_test_bindings(mut markup: String) -> String {
    let mut cursor = 0;
    while let Some(at) = markup[cursor..].find("=\"") {
        let start = cursor + at + 2;
        let Some(end) = markup[start..].find('"').map(|n| start + n) else {
            break;
        };
        let name_start = markup[..start - 2]
            .rfind(|c: char| c.is_whitespace() || c == '<')
            .map_or(0, |n| n + 1);
        let name = &markup[name_start..start - 2];
        let value = &markup[start..end];
        let replacement = if name.starts_with("pp-on") {
            Some(value.trim().to_string())
        } else {
            value
                .strip_prefix('{')
                .and_then(|v| v.strip_suffix('}'))
                .map(|v| format!("{{{}}}", v.trim()))
        };
        if let Some(replacement) = replacement {
            markup.replace_range(start..end, &replacement);
            cursor = start + replacement.len() + 1;
        } else {
            cursor = end + 1;
        }
    }
    markup
}
