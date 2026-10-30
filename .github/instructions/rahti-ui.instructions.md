---
applyTo: "**/*.rs"
---

# rahti-ui components in this project

These are shadcn/ui components ported to Rahti and installed by `cargo rahti-ui`.
They live in `src/components/rahti_ui/`, one file per component. Unlike the icons in
`src/components/rahti_icons/`, they are **not generated** — the file is the component, and it is
this project's to read and to change.

`rahti-ui.json` lists what is installed, and for each one every `#[component]` it
declares. Read it before assuming a tag does not exist: `sidebar` alone declares
eleven of them.

## Using one

```rust
use crate::components::rahti_ui::button::Button;

html! {
    <Button variant="outline" size="sm" onclick={save()}>"Save"</Button>
}
```

Every prop is optional — `html!` closes a props literal with `..Default::default()`,
so a tag names what it changes and nothing else. Values are Rust expressions, so a
number is written as a string.

A `{…}` written on a component tag reads the state of the page that wrote it:

- An **event** — `onclick={setCount(count + 1)}` — lands on the element and runs
in the calling page's scope.
- An **attribute binding** — `disabled={count === 0}`, `value={query}` — lands on
the component's root, whose attributes are evaluated where the tag was written.
Where the bound element sits below the root (a Toggle's input inside its label),
the component lifts it: the handler onto the root, the value onto it as a prop
the inner element reads through `pp.props`.
- A `{…}` between the tags reads the page too: children belong to the block that
wrote them.

The props builder is the other spelling, for an element built in Rust:
`@{button().outline().bind("disabled", "count === 0").render(children)}`.

Anything the component does not name goes through `attrs`, which is what
`{...props}` spreads in the React original:

```rust
<Button attrs=@{Attrs::new().set("aria-label", "Sign up").unset("data-size")}>
    "Sign up"
</Button>
```

## Adding one

```text
cargo rahti-ui add <component>       # one, or several
cargo rahti-ui list --search menu    # what the catalog has
cargo rahti-ui update                # re-fetch what is installed
cargo rahti-ui remove <component>
```

`add` brings the whole graph: the components a component imports, and the icons it
imports, through `cargo rahti-icons`. Do not hand-write a file into `src/components/rahti_ui/` to
avoid installing one — a component that is not in `rahti-ui.json` is one nothing will
update.

## Editing one

Editing is the point, and `update` protects it: each file's hash as installed is
recorded in `rahti-ui.json`, and a file that no longer matches is left alone and
reported. `--force` overwrites it. So an edit is safe, and an edit is also the
thing that stops a component receiving upstream fixes — for a one-off variation,
prefer a wrapper component of this project's own over a change to the file.

## Styling

The class strings are shadcn's and name shadcn's theme, which `cargo rahti-ui`
installs into `src/app/globals.css` on the first `add`. Without it these components render
unstyled. `cargo rahti-ui theme` re-checks it.
