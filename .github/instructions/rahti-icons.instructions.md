---
applyTo: "**/*.rs"
---

# Icons in this project

Icons are Rahti components installed by `cargo rahti-icons`. They live in `src/components/rahti_icons/`,
one file per icon, and are generated — do not edit them. The runtime behind them is
the `rahti-icons` crate.

## Using one

```rust
use crate::components::rahti_icons::user::User;

html! {
    <button class="inline-flex items-center gap-2">
        <User class="size-4" />
        "Profile"
    </button>
}
```

The file is named in snake_case and the component in PascalCase, both derived from
the catalog name: `bot-off` is `crate::components::rahti_icons::bot_off::BotOff`. `rahti-icons.json` lists what is
installed, with the module path for each.

## Props

Every prop is optional. Values are Rust expressions, so numbers are written as
strings — `size="20"`, not `size=20`.

| Prop | Does |
| --- | --- |
| `class` | Adds classes to the icon's own |
| `size` | Sets `width` and `height` together |
| `width`, `height` | Sets one of them |
| `stroke_width` | Sets `stroke-width` |
| `color` | Paints the icon, on whichever of `stroke`/`fill` it uses |
| `fill` | Sets `fill` |
| `label` | The accessible name; without one the icon is `aria-hidden` |
| `attrs` | `rahti_icons::Attrs` — any other attribute, and removals |

```rust
<User size="20" color="#4f46e5" label="Your profile" />
<User class="size-4" attrs=@{Attrs::new().set("id", "avatar").unset("stroke-width")} />
<User class="size-4" onclick={openMenu()} />
```

An icon renders one `<svg>` element and nothing around it, so `button > svg` and
`[&>svg]:size-4` reach it.

## Adding one

```text
cargo rahti-icons add <icon>          # one, or several
cargo rahti-icons list --search arrow # what the catalog has
cargo rahti-icons update              # re-fetch what is installed
cargo rahti-icons remove <icon>
```

Do not hand-write a file into `src/components/rahti_icons/`, and do not paste an SVG into a component to
avoid installing one: `cargo rahti-icons update` rewrites that directory.
