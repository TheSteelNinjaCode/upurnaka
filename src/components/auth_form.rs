use crate::{
    auth::{self, Challenge, SignedIn},
    components::{
        aria::submit,
        rahti_ui::{
            attrs::Attrs,
            button::Button,
            checkbox::Checkbox,
            field::{Field, FieldDescription, FieldGroup, FieldLabel},
            input::{Input, input_variants},
        },
    },
    rahti::{Html, Result, component, html, rpc},
};

#[rpc]
pub async fn register_account(
    name: String,
    email: String,
    password: String,
    confirmation: String,
) -> Result<Challenge> {
    auth::register(name, email, password, confirmation).await
}
#[rpc]
pub async fn start_signin(email: String, password: String) -> Result<Challenge> {
    auth::begin_signin(email, password).await
}
#[rpc]
pub async fn verify_signin(token: String, code: String) -> Result<SignedIn> {
    auth::finish_signin(token, code).await
}

#[component]
pub fn AuthForm(register: bool, next: &str) -> Html {
    let title = if register {
        "Create your account"
    } else {
        "Welcome back"
    };
    let intro = if register {
        "Your people, conversations, and meetings. All in one place."
    } else {
        "Sign in to pick up where you left off."
    };
    let password_autocomplete = if register {
        "new-password"
    } else {
        "current-password"
    };
    // Registration asks two more questions. They are left out of a sign-in
    // form altogether rather than hidden, so nothing disabled travels with it.
    let name_field = if register {
        html! {
            <Field>
                <FieldLabel for="auth-name">"Your name"</FieldLabel>
                <Input
                    id="auth-name"
                    name="name"
                    required
                    class="h-11!"
                    attrs=@{Attrs::new().set("autocomplete", "name").set("maxlength", "80")}
                />
            </Field>
        }
    } else {
        Html::empty()
    };
    let password_hint = if register {
        html! {
            <FieldDescription>"Use at least 12 characters."</FieldDescription>
        }
    } else {
        Html::empty()
    };
    let confirmation_field = if register {
        html! {
            <Field>
                <FieldLabel for="auth-confirmation">"Confirm password"</FieldLabel>
                <Input
                    id="auth-confirmation"
                    name="confirmation"
                    type="password"
                    required
                    class="h-11!"
                    attrs=@{Attrs::new().set("autocomplete", "new-password").set("maxlength", "128")}
                />
            </Field>
        }
    } else {
        Html::empty()
    };
    // An element rather than an `<Input>`: the code step focuses it when it
    // appears, and a ref is a binding a component tag cannot carry.
    let code_input = input_variants("h-12! text-lg! tracking-widest");
    html! {
        <main class="ambient flex h-dvh items-center justify-center overflow-y-auto px-5 py-10">
            <section
                class="my-auto w-full max-w-[460px] rounded-2xl border border-edge bg-surface p-7 shadow-2xl sm:p-10"
                aria-label="Account access"
            >
                <a
                    href="/signin"
                    class="mb-8 flex items-center gap-3 text-lg font-semibold tracking-tight"
                >
                    <img src="/images/upurnaka-logo.png" alt="" class="h-8 w-8" />
                    "upurnaka"
                </a>
                <div hidden={stage !== "credentials"}>
                    <p class="mb-2 text-xs font-semibold uppercase tracking-widest text-brand-ink">
                        "YOUR WORKSPACE"
                    </p>
                    <h1 class="text-2xl font-semibold tracking-tight">@{title}</h1>
                    <p class="mb-7 mt-2 text-sm leading-relaxed text-muted-foreground">@{intro}</p>
                    <form onsubmit={submitCredentials(event)}>
                        <FieldGroup class="gap-4">
                            @{name_field}
                            <Field>
                                <FieldLabel for="auth-email">"Email address"</FieldLabel>
                                <Input
                                    id="auth-email"
                                    name="email"
                                    type="email"
                                    required
                                    class="h-11!"
                                    attrs=@{Attrs::new().set("autocomplete", "username").set("maxlength", "254")}
                                />
                            </Field>
                            <Field>
                                <FieldLabel for="auth-password">"Password"</FieldLabel>
                                <Input
                                    id="auth-password"
                                    name="password"
                                    type="password"
                                    required
                                    class="h-11!"
                                    attrs=@{Attrs::new().set("autocomplete", password_autocomplete).set("maxlength", "128")}
                                />
                                @{password_hint}
                            </Field>
                            @{confirmation_field}
                            <Button
                                size="lg"
                                class="h-11! w-full font-semibold"
                                attrs=@{submit()}
                                disabled={busy}
                            >
                                {busy ? "Please wait…" : "Continue"}
                            </Button>
                        </FieldGroup>
                    </form>
                    <p class="mt-6 text-center text-sm text-muted-foreground" hidden=@{!register}>
                        "Already registered? "
                        <a href={signinLink} class="text-link hover:underline">"Sign in"</a>
                    </p>
                    <p class="mt-6 text-center text-sm text-muted-foreground" hidden=@{register}>
                        "New to upurnaka? "
                        <a href={registerLink} class="text-link hover:underline">
                            "Create an account"
                        </a>
                    </p>
                </div>
                <div hidden={stage !== "otp"}>
                    <p class="mb-2 text-xs font-semibold uppercase tracking-widest text-brand-ink">
                        "SECOND STEP"
                    </p>
                    <h1 class="text-2xl font-semibold">
                        {setupKey ? "Set up your authenticator" : "Verify it’s you"}
                    </h1>
                    <p class="mt-2 text-sm text-muted-foreground" hidden={!setupKey}>
                        "Scan this QR code with your authenticator app, then enter its 6-digit code."
                    </p>
                    <p class="mt-2 text-sm text-muted-foreground" hidden={!!setupKey}>
                        "Enter the code from your authenticator app, or use a saved recovery code."
                    </p>
                    <div class="mt-5" hidden={!setupKey}>
                        <template pp-for="image in qr ? [qr] : []">
                            <img
                                key={image}
                                src={image}
                                alt="Authenticator setup QR code"
                                class="mx-auto h-48 w-48 rounded-lg bg-white p-2"
                            />
                        </template>
                        <p class="mb-2 mt-4 text-xs text-muted-foreground">
                            "Can’t scan? Enter this setup key in your app:"
                        </p>
                        <code
                            class="block break-all rounded-lg border border-edge bg-sunken p-3 text-sm tracking-wider"
                        >
                            {setupKey}
                        </code>
                    </div>
                    <form class="mt-6" onsubmit={submitCode(event)}>
                        <FieldGroup class="gap-4">
                            <Field>
                                <FieldLabel for="auth-code">
                                    {useRecovery ? "Recovery code" : "Authenticator code"}
                                </FieldLabel>
                                <input
                                    id="auth-code"
                                    name="code"
                                    type="text"
                                    data-slot="input"
                                    autocomplete="one-time-code"
                                    required
                                    maxlength="32"
                                    class=@{code_input.as_str()}
                                    pp-ref={codeInput}
                                    inputmode={useRecovery ? "text" : "numeric"}
                                />
                            </Field>
                            <Button
                                size="lg"
                                class="h-11! w-full font-semibold"
                                attrs=@{submit()}
                                disabled={busy}
                            >
                                {busy ? "Verifying…" : "Verify and sign in"}
                            </Button>
                        </FieldGroup>
                    </form>
                    <div class="mt-4 flex flex-col items-start gap-1">
                        <Button
                            variant="link"
                            size="sm"
                            class="px-0! text-link"
                            hidden={!!setupKey}
                            onclick={setUseRecovery(!useRecovery)}
                        >
                            {useRecovery ? "Use authenticator code" : "Use a recovery code"}
                        </Button>
                        <Button
                            variant="ghost"
                            size="sm"
                            class="px-0! text-muted-foreground hover:bg-transparent!"
                            disabled={busy}
                            onclick={restart()}
                        >
                            "Start again"
                        </Button>
                    </div>
                    <p class="mt-4 text-xs text-subtle">
                        "This step expires after 10 minutes. Keep your device’s clock set automatically."
                    </p>
                </div>
                <div hidden={stage !== "recovery"}>
                    <h1 class="text-2xl font-semibold">"Save your recovery codes"</h1>
                    <p class="mb-5 mt-2 text-sm leading-relaxed text-muted-foreground">
                        "Your account is ready. Save these codes somewhere safe before continuing. Each code works once if you lose access to your authenticator. These codes are shown only now."
                    </p>
                    <div
                        class="grid grid-cols-1 gap-2 rounded-lg border border-edge bg-sunken p-4 sm:grid-cols-2"
                    >
                        <template pp-for="item in recoveryCodes">
                            <code key={item} class="select-all font-mono text-xs">{item}</code>
                        </template>
                    </div>
                    <Field orientation="horizontal" class="my-5">
                        <Checkbox
                            id="auth-saved"
                            checked={saved}
                            onchange={setSaved(target.checked)}
                        />
                        <FieldLabel for="auth-saved" class="font-normal text-muted-foreground">
                            "I saved my recovery codes somewhere safe."
                        </FieldLabel>
                    </Field>
                    <Button
                        size="lg"
                        class="h-11! w-full font-semibold"
                        disabled={!saved}
                        onclick={continueToApp()}
                    >
                        "Go to my workspace"
                    </Button>
                </div>
                <p
                    role="alert"
                    aria-live="polite"
                    hidden={!error}
                    class="mt-5 rounded-lg border border-busy/40 bg-busy/10 p-3 text-sm text-danger-ink"
                >
                    {error}
                </p>
                <p class="mt-7 text-center text-xs text-subtle">
                    "Protected with a password and two-step verification"
                </p>
                <script>r##"
                    const registering = @{register};
                    const destination = @{next};
                    const signinLink = '/signin?next=' + encodeURIComponent(destination);
                    const registerLink = '/register?next=' + encodeURIComponent(destination);
                    const [stage, setStage] = pp.state('credentials');
                    const [busy, setBusy] = pp.state(false);
                    const [error, setError] = pp.state('');
                    const [token, setToken] = pp.state('');
                    const [setupKey, setSetupKey] = pp.state('');
                    const [qr, setQr] = pp.state('');
                    const [recoveryCodes, setRecoveryCodes] = pp.state([]);
                    const [saved, setSaved] = pp.state(false);
                    const [useRecovery, setUseRecovery] = pp.state(false);
                    const codeInput = pp.ref(null);
                    pp.effect(() => {
                        if (stage === 'otp') codeInput.current?.focus();
                    }, [stage, useRecovery]);
                    function message(e) { return e.body?.error || e.message || 'Something went wrong. Please try again.'; }
                    async function submitCredentials(event) {
                        event.preventDefault(); if (busy) return;
                        const form = event.currentTarget;
                        const values = Object.fromEntries(new FormData(form));
                        setBusy(true); setError('');
                        try {
                            const result = registering
                                ? await pp.rpc('register_account', values)
                                : await pp.rpc('start_signin', values);
                            form.reset();
                            setToken(result.token); setSetupKey(result.setup_key || ''); setQr(result.qr || ''); setStage('otp');
                        } catch (e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    async function submitCode(event) {
                        event.preventDefault(); if (busy) return;
                        const form = event.currentTarget;
                        const values = Object.fromEntries(new FormData(form));
                        setBusy(true); setError('');
                        try {
                            const result = await pp.rpc('verify_signin', {token, code: values.code});
                            form.reset(); setToken(''); setSetupKey(''); setQr('');
                            if (result.recovery_codes.length) { setRecoveryCodes(result.recovery_codes); setStage('recovery'); }
                            else { pp.redirect(destination); }
                        } catch (e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    function restart() { setStage('credentials'); setError(''); setToken(''); setSetupKey(''); setQr(''); setUseRecovery(false); }
                    function continueToApp() { if (saved) { setRecoveryCodes([]); pp.redirect(destination); } }
                "##</script>
            </section>
        </main>
    }
}
