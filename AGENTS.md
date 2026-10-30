# upurnaka — Agent Guide

This is a Rahti application: a server-rendered Rust web framework with
file-system routing under `src/app/`, typed HTML through `html!`, and
browser reactivity supplied by the bundled PulsePoint runtime. This file
is written for coding agents and the people working beside them. Read it
first, and read the matching convention document before working in its
area.

This file is the scaffold's until you edit it: `cargo rahti upgrade`
refreshes an unedited copy as the framework evolves, and leaves an
edited one alone.

## Applying an edit

Nothing you edit is real until the running server is replaced, and you
own that step. After every batch of edits:

1. `cargo rahti restart`. A compile error leaves the old server running
and says so. Do not start a second server, and never force-kill one.
2. Load every affected route in a browser with a full page load, and
exercise what changed.
3. `cargo rahti status`, or MCP `server_status`. It says whether the
server is running the current source, which routes loaded under this
boot with their browser errors, and how background jobs are doing. A
route it does not list was never opened, so it proves nothing.

End with what you applied: "Restarted, loaded `/todo`, saved a row;
browser clean." If you have no browser, say so and give the exact URL.

## Background jobs for agents

`cargo rahti generate job <name> --params id:i64` adds a `#[job]` to
`src/jobs.rs`. Call it as `crate::jobs::<name>::enqueue(id).await?`, after
the write it follows commits; a wrong name or argument is a compile
error. `jobs` in `.rahti/manifest.json`, or MCP `list_jobs`, gives each
job's parameters, policy and exact call. Retryable jobs run more than
once, so make them idempotent. Payloads are ids, never secrets. See
`docs/conventions/background-jobs.md`.

## RPC work for agents

Run `cargo check` after changing RPC declarations. Read `rpcContracts`
in `.rahti/manifest.json`, or optional MCP `list_rpcs`, before writing
the caller. Match scope, file and URLs; inspect parameters, response
types and checking limits. `schemaStatus: "unresolved"` requires
reading the Rust/Serde type. Use `pp.rpc` beside its `#[rpc]`; run
compiler and router/browser checks. Dynamic values and response-field
accesses are not proved by compilation. See `docs/conventions/rpc-and-uploads.md`.

## Required Reading

| Work area | Document |
| --- | --- |
| Pages, layouts, route groups, dynamic routes, APIs, errors, 404s | `docs/conventions/routing.md` |
| `html!`, interpolation, escaping, components, children | `docs/conventions/rendering-and-components.md` |
| Page titles, descriptions, social cards, canonical URLs, robots | `docs/conventions/metadata-and-seo.md` |
| Static HTML builds, dynamic export paths, output safety | `docs/conventions/static-generation.md` |
| Cache stores, page caching, revalidation, backend adapters | `docs/conventions/caching.md` |
| Locales, Fluent catalogs, fallback, localized routes, language direction | `docs/conventions/localization.md` |
| Typed input rules, field errors, HTTP 422 validation failures | `docs/conventions/validation.md` |
| PulsePoint markup: not JSX, the closed directive and API list, bindings, events, forms, conditionals, lists, props | `docs/conventions/pulsepoint.md` |
| Every `pp.*` hook with an example: `pp.event`, effects, context, refs, portals, error boundaries, external stores | `docs/conventions/pulsepoint-hooks.md` |
| Slow inputs or lists, re-renders, keyed rows, identity, splitting a large page | `docs/conventions/pulsepoint-performance.md` |
| WebGPU, Canvas/WebGL, workers, WebAssembly, media, third-party libraries, games and widgets (DOM ownership) | `docs/conventions/pulsepoint-web-apis.md` |
| SPA navigation, scroll, loading regions, `pp.redirect` | `docs/conventions/pulsepoint-navigation.md` |
| Browser/backend development diagnostics and `.rahti/dev.log` | `docs/conventions/diagnostics.md` |
| `#[rpc]`, errors, component RPCs, streams, files | `docs/conventions/rpc-and-uploads.md` |
| WebSockets: `#[socket]`, `pp.socket` | `docs/conventions/websockets.md` |
| Sessions, route protection, `#[rpc(auth)]` | `docs/conventions/authentication.md` |
| Abilities, resource policies, 401/403 decisions | `docs/conventions/authorization.md` |
| CSP, browser security headers, HSTS, policy overrides | `docs/conventions/security-middleware.md` |
| Request budgets, client IPs, proxy trust, HTTP 429 | `docs/conventions/rate-limiting.md` |
| Application middleware, request/response order, short circuits | `docs/conventions/request-pipeline.md` |
| Request IDs, JSON logs, metrics, liveness and readiness | `docs/conventions/production-observability.md` |
| Background jobs, retries, queue bounds, graceful drain | `docs/conventions/background-jobs.md` |
| Intervals, local/UTC daily and one-shot task dispatch | `docs/conventions/task-scheduling.md` |
| Application timezone, UTC storage, calendar-day bounds | `docs/conventions/application-timezone.md` |
| Error privacy, page boundaries, Problem Details, RPC failures | `docs/conventions/error-handling.md` |
| SeaORM entities, migrations, the connection, database tests | `docs/conventions/database.md` |
| Escaping, CSRF, redirects, upload trust | `docs/conventions/security.md` |
| `rahti.config.json`, CSS engines, frontend `twMerge`, static files, the CLI | `docs/conventions/configuration-and-tooling.md` |
| Where tests live, driving the router, RPC/auth/database tests | `docs/conventions/testing.md` |
| Optional Windows/Android packaging, packaged paths, the loopback server | `docs/conventions/native-packaging.md` |

## Non-Negotiable Conventions

1. Do not edit generated files by hand: `src/routes.rs`,
    `src/components/mod.rs`, `src/models/mod.rs`, `src/migrations/mod.rs`,
    and `.rahti/manifest.json`. They are regenerated by `rahti-build` on
    every Cargo build.
2. Routes live under `src/app/`. A page is `page.rs`; a layout is
    `layout.rs`; an error boundary is `error.rs`; a navigation loading
    region is `loading.rs`; the root 404 is `not-found.rs`; an API
    endpoint is `route.rs`. A segment cannot hold both `page.rs` and
    `route.rs`. `_private` directories are ignored by the router.
3. A template is markup written as Rust tokens, `html! { … }`, and only
    a `<script>` or `<style>` body is a string, always written
    `<script>r##"…"##</script>` — JavaScript and CSS exactly as
    written, `"#fff"` included; `html!` refuses fewer `#`s.
    Text is a quoted string, `<p>"Hello"</p>`; a literal attribute is
    quoted, `class="card"`; a binding is braced, `hidden={!open}`; a
    handler is a binding, `onclick={save()}`; a server value is
    `href=@{url}`. A binding is JavaScript the Rust lexer must accept —
    double-quoted strings, no template literals — so longer logic is a
    named function in the script. Compute values in Rust above the
    template and interpolate names. `html!` has one authored root: a
    literal element, a single component tag, or a `<>…</>` fragment. The
    root layout is the exception: it writes a literal `<html>` root,
    optionally preceded by a doctype.
4. Server values use `@{rust_expression}`. PulsePoint browser
    expressions use `{javascript_expression}`. Never interchange the two.
5. Runtime values are
    escaped, braces included (`&#123;`), so no value can open a PulsePoint
    expression; use `Html::from_raw` only for explicitly trusted markup,
    whose braces are not escaped. Client code is written in `html!` and
    nowhere else — it is compiled there into the client registry, and the
    runtime runs nothing else. A component forwards its tag's `{…}` props
    through a `bindings: Bindings` parameter; an `Attrs` value is always
    data, and `Attrs::merge` and `Attrs::only` compose sets.
6. Components are PascalCase functions marked `#[component]`, return
    `Html`, and take props as normal Rust arguments. In markup they are
    called as tags — `<Card title="…">…</Card>` — and children are an
    explicit `Html` argument rendered with `<slot />`.
7. A reactive block and its `<script>` must be self-contained. Children
    written between component tags are the exception: a `{…}` binding
    there reads the writing block's state — `html!` ships such children
    as an owner-scoped `<template pp-owner="…">`, the React children
    semantics. Fragment-rooted (`<>…</>`) and `@{expr}`-built children
    still do not cross the boundary.
8. A page-owned `#[rpc]` lives beside its caller in that `page.rs`; a
    component-owned `#[rpc]` lives in the component file. Parameters are
    owned, deserializable types (`String`, not `&str`); return values
    must serialize; use `rahti::Result<T>` for failures.
9. Static route metadata is `pub fn metadata() -> Metadata` in a page
    or layout, inherited root to leaf. Request-specific metadata is
    returned with `Page::new(content, metadata)`. Rahti writes the
    managed SEO tags; do not duplicate them in the root layout.
10. `cargo rahti build --static` renders the real application router.
    Static pages are automatic; a dynamic page exports exact concrete
    URLs from `pub async fn static_paths()`. Keep `dist/` generated and
    do not remove the export ownership marker by hand.
11. `rahti::cache` stores derived application data, never authority.
    Cache keys include every user, tenant, locale, permission, and input
    dimension that affects a value. Install a shared adapter during
    application startup; its `clear()` removes only the namespace that
    application owns.
12. Page caching is an explicit `pub fn cache() -> PageCache` export on
    a public, deterministic `page.rs`. It caches anonymous complete-document
    responses only; do not use it when cookies, locale headers, experiments
    or other request state changes the HTML. Call `revalidate_path` or
    `revalidate_tag` only after the authoritative write succeeds.
13. Deserialize into owned Rust types first, then use `rahti::Validator`
    for business rules at the handler boundary before side effects.
    Validation failures are HTTP 422 with named field messages; malformed
    or mistyped RPC payloads remain HTTP 400. Database constraints remain
    authoritative for invariants that concurrent requests can race.
14. The generated router applies `rahti::security_headers` once over every
    response including public assets, probes, and failures, immediately
    inside production observability.
    Existing response headers win. Customize the baseline only through
    `rahti::security::configure` during startup. `script-src` is
    `'self'`, with `require-trusted-types-for 'script'; trusted-types
    pulsepoint`: PulsePoint runs only the client registry `html!`
    compiled. Name a vendor SDK's exact host, add `'wasm-unsafe-eval'`
    only for WebAssembly, and never add `https:`, `'unsafe-inline'` or
    `'unsafe-eval'`. Never enable HSTS `includeSubDomains` unless every
    subdomain is HTTPS.
15. The generated router applies the process-local `rahti::rate_limit`
    outside application/auth/CSRF work and inside security headers. The
    default is 200 dynamic requests per client IP per minute; real public
    files, `OPTIONS`, `/health`, `/ready`, and development tooling are
    exempt. Never
    trust `X-Forwarded-For` unless an inaccessible trusted proxy replaces
    it, and do not mistake this bounded per-process safety net for
    distributed DDoS protection.
16. Install application middleware once with
    `rahti::request_pipeline::configure(...)` before building the router.
    It runs in registration order around matched pages, APIs, and RPCs,
    after auth and inside CSRF, rate limiting, and security headers. A
    short circuit skips layouts, caching, body parsing, and the handler;
    public files and framework endpoints never enter this chain.
17. Production observability is the generated router's outermost layer.
    It assigns or validates `X-Request-Id`, emits secret-free JSONL request
    records to stderr, and updates fixed-cardinality process metrics.
    `/health` and `/ready` are reserved public framework routes outside
    sessions and application middleware; readiness checks are named,
    side-effect-free, concurrent, and timeout-bounded. Configure
    `rahti::observability` before building the router, never log credentials
    or bodies, and remember that response duration does not measure a
    streaming body's lifetime.
18. Background jobs are typed, named application work running on Rahti's
    bounded process-local queue. Keep a `JobQueue` handle in application-owned
    state, enqueue only after the authoritative transaction commits, and
    make retryable handlers idempotent. Payloads are neither durable nor
    encrypted; use an external broker when losing queued work on process
    failure is unacceptable. Every host calls
    `rahti::jobs::finish_shutdown()` after HTTP graceful shutdown.
19. Task scheduling submits registered background jobs; it never creates
    a second execution path. Recurring schedules prevent overlap by
    default, coalesce missed ticks, and skip instead of waiting when the
    bounded queue is full. Timers and uniqueness are process-local, so
    use one designated scheduler replica or external durable coordination
    when a fleet must produce one occurrence.
    `rahti::jobs::finish_shutdown()` stops schedulers before queues.
20. `APP_TIMEZONE` is resolved and installed once during startup, defaults
    to UTC, and must be a valid IANA name. It controls calendar display,
    `today`, local-day query bounds, and `Scheduler::daily`; it never
    controls stored instants, sessions, cache TTLs, request budgets,
    retry delays, or signatures. Store instants in UTC and query local
    dates with `rahti::time::day_bounds_utc`.
21. Localization is opt-in through `rahti.config.json`. Fluent catalogs
    live under `<i18n.dir>/<locale>/`, are validated and embedded at build
    time, and are read through request-task-local `rahti::i18n` state. A
    root `[locale]` or `[lang]` segment is authoritative and rejects
    unsupported values. Translation results are escaped text, not trusted
    markup, and localized links keep an explicit locale prefix.
22. Expected 4xx errors may carry public messages; unexpected 5xx errors,
    ordinary errors crossing `?`, and panics expose diagnostics only when
    development mode is active. Use `Error::public` or
    `with_public_message` only for deliberately safe server-error text.
    Page boundaries receive the safe message and optional request id;
    unbounded page/API failures use `application/problem+json`, while RPCs
    keep their JSON wire. Framework error responses are `no-store`, and
    public details never replace reserved error or request-id fields.
23. PulsePoint markup is HTML with `{…}` bindings, never JSX: a binding
    renders text, a condition is `hidden={…}` on the element, a list is
    `<template pp-for="item in items">` with a `key` on the row, and a
    form submit sends `Object.fromEntries(new FormData(event.currentTarget))`
    through `pp.rpc`. Bind first-party events with `on…={…}`; never wire
    your own UI with ids, `querySelector`, `addEventListener` or
    `innerHTML`. The directive, hook and utility lists in
    `docs/conventions/pulsepoint.md` are closed, and `html!` refuses JSX
    markup in a binding, JSX attribute names (`className`, `onClick`, …),
    `pp-for` off a `<template>`, directives that do not exist
    (`pp-if`, `pp-show`, `pp-model`, …), a quoted handler, a template
    written as a string, a script body written as tokens, and a server
    value inside client code (`onclick=@{…}`, `@{…}` within a `{…}`
    binding), naming the form to write. A script reads a server value as
    data: `const id = @{Json(&id)};`.
24. Do not write PulsePoint's runtime-managed DOM attributes by hand. In
    particular, do not invent `data-pp-*`, `pp-owner`, `pp-ref-owner`,
    `pp-script`, `pp-data`, `pp-on*` or `pp-loop-scope`, and write no root-layout marker — `html!` injects
    them.
25. `public/js/pp-reactive-v2.min.js` is a shipped framework asset.
    Never hand-edit it; `cargo rahti upgrade` replaces it.
26. A component script is plain browser JavaScript, so every web API is
    available to it directly: WebGPU (`navigator.gpu`), Canvas/WebGL, Web
    Workers, WebAssembly, Web Audio, WebRTC, IndexedDB, and the device APIs.
    PulsePoint wraps none of them; do not invent `pp.gpu` or similar. The
    handle goes in `pp.ref`, a `pp.effect` acquires, feeds, and releases
    it, and data arrives through `#[rpc]`/`pp.rpc`. See
    `docs/conventions/pulsepoint-web-apis.md`.
27. A `<script>` body is JavaScript exactly as written — comments,
    single quotes and template literals included — with `@{expr}`
    interpolated. A function handed to a library or a long-lived listener is
    wrapped in `pp.event(…)` so it reads current state, never a
    hand-kept "latest" ref. A library mounts into an empty element of its
    own, `<div pp-ref="{host}"></div>`; re-renders leave its children and the
    attributes it sets alone.
28. A `#[socket]` function lives beside the page or component whose
    script opens it, takes a final `socket: rahti::ws::Socket`
    parameter, and receives its other arguments as the connection's
    first frame. `#[socket(auth)]` refuses the handshake with a 401
    when there is no session.
29. Authentication is a signed session cookie and a route policy;
    authorization is an application-defined gate of named abilities and
    typed resource policies. Neither infers roles, and there are no OAuth
    providers. Both policies are application-owned (`src/auth.rs`) and
    installed once from `initialize_application` with
    `rahti::auth::configure(auth::settings())` and
    `rahti::authorization::configure(auth::authorization())`, before the router is
    built. `#[rpc(auth)]` guards the call and `private_routes` guards
    the page; an ability guards the action/resource. They do not
    substitute for each other, so enforce every applicable boundary.
30. Auth reads exactly three environment values — `AUTH_SECRET`,
    `AUTH_COOKIE_NAME`, and the optional `SESSION_LIFETIME_HOURS` —
    generated per project and living in the git-ignored `.env`, never in
    `rahti.config.json`. The session payload is signed, not encrypted:
    sign in an id and what the UI needs, never a hash or a token.
31. Development browser warnings/errors and Rahti-handled backend
    failures are written as JSON Lines to `.rahti/dev.log`. Read it when
    diagnosing frontend behavior; never commit it.
32. Tests live in `src/tests/`, included from `src/lib.rs`, and run
    with `cargo test`. They drive the generated router in-process — no
    server, no port. A feature lands with its tests: a page renders, an
    `#[rpc]` answers and refuses a malformed payload, a private route
    redirects without a session. Before testing private routes, install
    the auth policy in `setup()` exactly as `initialize_application`
    does. What only a browser shows — a binding updating, a window
    opening, a library's DOM surviving a re-render — gets a browser
    test with `rahti::browser` (the `browser-test` feature, in
    `[dev-dependencies]`), ending with `browser.errors()` empty. See
    `docs/conventions/testing.md`.
33. The database is SeaORM and belongs to this application.
    `src/models/` holds one flat file per table; `src/migrations/`
    holds one flat file per change, named `mYYYYMMDD_NNNNNN_<what>.rs`
    because filename order is application order. The migration is the
    schema — an entity neither creates nor validates a table. Reach
    the connection with `crate::db::db()`; `db::migrate()` is never
    automatic.
34. A connection string is a credential. It lives in `DATABASE_URL`
    via the ignored `.env`, never in `rahti.config.json`, which
    records only which backend the project is on.
35. Tests reach the database the way `setup()` in `src/tests/support.rs`
    shows: a *file* SQLite database, never `:memory:`, deleted on
    the way in, with `db::migrate()` run against it. An in-memory
    database vanishes between `#[tokio::test]` runtimes, and the
    symptom is tests that pass alone and fail together.
36. Tailwind class conflicts use the scaffold's browser `twMerge`, not
    a Rust dependency or `pp.twMerge`. Check `css.engine`, the helper
    asset, and `public/js/main.js`: Tailwind scaffolds publish it before
    `pp.mount()`; plain CSS scaffolds do not. Rust helpers join class
    data; `Attrs::merge` merges attributes, not Tailwind utilities.
    Static class attributes are not automatically merged. Call `twMerge`
    explicitly in compiled bindings or scripts when utilities conflict;
    pass server strings through script `@{...}`, never formatted code.
    Do not add `tw_merge`, npm tooling, or another merger for this
    behavior. `twMerge` does not generate CSS. Read the Tailwind class
    merging agent contract in `docs/conventions/configuration-and-tooling.md`.

## Commands

```text
cargo run              # regenerate routes and serve
cargo dev              # cargo run, restarted on every edit (needs cargo-watch)
cargo check            # verify without running
cargo test             # this application's tests
cargo fmt              # format ordinary Rust
cargo rahti fmt        # format authored html! markup
cargo rahti fmt --check # verify markup formatting without writing
cargo rahti build --static # render the site into dist/
cargo rahti upgrade    # refresh unedited scaffold files, this guide included
```

After editing markup, run `cargo fmt` then `cargo rahti fmt` on the changed
files. Use four-space nesting and separate lines for nested elements;
keep visible spaces inside quoted text. The formatter preserves client
and server expressions, attribute order, and script/style raw strings.
See the formatting rules in `docs/conventions/configuration-and-tooling.md`.
