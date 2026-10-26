use crate::{
    components::app_shell::AppShell,
    rahti::{Html, Metadata, Result, html},
};
use sea_orm::EntityTrait;

pub fn metadata() -> Metadata {
    Metadata::new().title("My account")
}

pub async fn page() -> Result<Html> {
    crate::auth::ready()?;
    let user = crate::auth::current().unwrap_or_default();
    let remaining = crate::models::account::Entity::find_by_id(&user.id)
        .one(crate::db::db())
        .await?
        .map(|a| a.recovery_hashes.lines().count())
        .unwrap_or(0);
    Ok(html! {
        <AppShell active="account">
            <main class="flex-1 overflow-y-auto p-8 sm:p-12">
                <div class="max-w-2xl">
                    <p class="mb-2 text-xs font-semibold uppercase tracking-widest text-brand-ink">
                        "YOUR WORKSPACE"
                    </p>
                    <h1 class="text-3xl font-semibold">"My account"</h1>
                    <p class="mb-8 mt-2 text-sm text-muted">"Your profile and sign-in security."</p>
                    <section class="rounded-xl border border-edge bg-surface p-6">
                        <h2 class="mb-5 text-lg font-semibold">"Profile"</h2>
                        <dl class="space-y-4 text-sm">
                            <div>
                                <dt class="text-muted">"Name"</dt>
                                <dd class="mt-1">@{user.name}</dd>
                            </div>
                            <div>
                                <dt class="text-muted">"Email address"</dt>
                                <dd class="mt-1">@{user.email}</dd>
                            </div>
                        </dl>
                    </section>
                    <section class="mt-5 rounded-xl border border-edge bg-surface p-6">
                        <h2 class="mb-3 text-lg font-semibold">"Sign-in security"</h2>
                        <p class="text-sm text-online">"Two-step verification is enabled"</p>
                        <p class="mt-2 text-sm text-muted">
                            "Sign in with your password and authenticator app."
                        </p>
                        <p class="mt-4 text-sm">@{remaining}" unused recovery codes"</p>
                        <p class="mt-2 text-xs text-muted">
                            "Keep your recovery codes safe. Each can be used once in place of an authenticator code."
                        </p>
                    </section>
                </div>
            </main>
        </AppShell>
    })
}
