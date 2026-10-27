use crate::{
    components::{
        app_shell::AppShell,
        rahti_ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    },
    rahti::{Html, Metadata, Result, html},
};
use sea_orm::EntityTrait;

pub fn metadata() -> Metadata {
    Metadata::new().title("My account")
}

pub async fn page() -> Result<Html> {
    crate::auth::ready()?;
    let user = crate::auth::current().unwrap_or_default();
    let role_label = if user.role == crate::auth::Role::Admin {
        "Admin"
    } else {
        "User"
    };
    let remaining = crate::models::account::Entity::find_by_id(&user.id)
        .one(crate::db::db())
        .await?
        .map(|a| a.recovery_hashes.lines().count())
        .unwrap_or(0);
    Ok(html! {
        <AppShell active="account">
            <main class="flex-1 overflow-y-auto p-8 sm:p-12">
                <div class="max-w-2xl">
                    <h1 class="text-3xl font-semibold">"My account"</h1>
                    <p class="mb-8 mt-2 text-sm text-muted-foreground">
                        "Your profile and sign-in security."
                    </p>
                    <Card class="bg-surface">
                        <CardHeader>
                            <CardTitle class="text-lg">"Profile"</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <dl class="space-y-4 text-sm">
                                <div>
                                    <dt class="text-muted-foreground">"Role"</dt>
                                    <dd class="mt-1">@{role_label}</dd>
                                </div>
                                <div>
                                    <dt class="text-muted-foreground">"Name"</dt>
                                    <dd class="mt-1">@{user.name}</dd>
                                </div>
                                <div>
                                    <dt class="text-muted-foreground">"Email address"</dt>
                                    <dd class="mt-1">@{user.email}</dd>
                                </div>
                            </dl>
                        </CardContent>
                    </Card>
                    <Card class="mt-5 bg-surface">
                        <CardHeader>
                            <CardTitle class="text-lg">"Sign-in security"</CardTitle>
                            <CardDescription class="text-online">
                                "Two-step verification is enabled"
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <p class="text-sm text-muted-foreground">
                                "Sign in with your password and authenticator app."
                            </p>
                            <p class="mt-4 text-sm">@{remaining}" unused recovery codes"</p>
                            <p class="mt-2 text-xs text-muted-foreground">
                                "Keep your recovery codes safe. Each can be used once in place of an authenticator code."
                            </p>
                        </CardContent>
                    </Card>
                </div>
            </main>
        </AppShell>
    })
}
