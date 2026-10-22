use crate::rahti::{Html, html, rpc};

/// The home page. Every `page.rs` under `src/app/` becomes a URL by where it
/// sits, and this one sits at the root.
pub async fn page() -> Html {
    html! {
        <section class="mx-auto w-full max-w-2xl px-6 py-16">
            <h1 class="text-3xl font-bold tracking-tight text-gray-900 dark:text-gray-50">"upurnaka"</h1>
            <p class="mt-3 text-sm text-gray-600 dark:text-gray-400">"Edit src/app/page.rs and reload."</p>

            <button class="mt-6 inline-flex items-center rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-700" onclick={greet()}>"Say hello"</button>
            <p class="mt-3 text-sm text-gray-600 dark:text-gray-400">{message}</p>

            <script>r##"
                const [message, setMessage] = pp.state("");

                async function greet() {
                    setMessage(await pp.rpc("hello", {}));
                }
            "##</script>
        </section>
    }
}

/// Called by `pp.rpc("hello")` above. It runs on the server, in Rust, and the
/// browser never learns there was a function here at all.
#[rpc]
pub async fn hello() -> String {
    "Hello from Rust.".to_string()
}
