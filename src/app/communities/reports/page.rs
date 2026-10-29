use crate::{
    communities,
    components::app_shell::AppShell,
    rahti::{Html, Metadata, Result, html},
};
pub fn metadata() -> Metadata {
    Metadata::new().title("Community reports")
}
pub async fn page() -> Result<Html> {
    let reports = communities::reports().await?;
    let rows=reports.into_iter().map(|report| html! {
        <article class="rounded-lg border border-edge bg-raised p-5">
            <h2 class="text-lg font-semibold">@{report.community}</h2>
            <p class="mt-2 text-sm text-muted-foreground">"Reported by "@{report.reporter}</p>
            <p class="mt-4 whitespace-pre-wrap">@{report.details}</p>
            <p class="mt-3 text-xs text-muted-foreground">"Reference: "@{report.id}</p>
        </article>
    }).collect::<Vec<_>>();
    let content = if rows.is_empty() {
        html! {
            <p>"No reports have been submitted."</p>
        }
    } else {
        Html::concat(rows)
    };
    Ok(html! {
        <AppShell active="communities">
            <main class="min-h-0 flex-1 overflow-auto p-8">
                <a href="/communities" class="text-link">"Back to communities"</a>
                <h1 class="my-6 text-2xl font-semibold">"Community reports"</h1>
                <div class="grid gap-4">@{content}</div>
            </main>
        </AppShell>
    })
}
