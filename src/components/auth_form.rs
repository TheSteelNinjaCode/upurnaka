use crate::{
    auth::{self, Challenge, SignedIn},
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
                    <p class="mb-7 mt-2 text-sm leading-relaxed text-muted">@{intro}</p>
                    <form onsubmit={submitCredentials(event)} class="space-y-4">
                        <label class="block text-sm" hidden=@{!register}>
                            "Your name"
                            <input
                                name="name"
                                autocomplete="name"
                                required=@{register}
                                disabled=@{!register}
                                maxlength="80"
                                class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                            />
                        </label>
                        <label class="block text-sm">
                            "Email address"
                            <input
                                name="email"
                                type="email"
                                autocomplete="username"
                                required
                                maxlength="254"
                                class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                            />
                        </label>
                        <label class="block text-sm">
                            "Password"
                            <input
                                name="password"
                                type="password"
                                autocomplete=@{password_autocomplete}
                                required
                                maxlength="128"
                                class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                            />
                            <span class="mt-2 block text-xs text-subtle" hidden=@{!register}>
                                "Use at least 12 characters."
                            </span>
                        </label>
                        <label class="block text-sm" hidden=@{!register}>
                            "Confirm password"
                            <input
                                name="confirmation"
                                type="password"
                                autocomplete="new-password"
                                required=@{register}
                                disabled=@{!register}
                                maxlength="128"
                                class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                            />
                        </label>
                        <button
                            type="submit"
                            disabled={busy}
                            class="h-11 w-full rounded-lg bg-brand text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                        >
                            {busy ? "Please wait…" : "Continue"}
                        </button>
                    </form>
                    <p class="mt-6 text-center text-sm text-muted" hidden=@{!register}>
                        "Already registered? "
                        <a href={signinLink} class="text-link hover:underline">"Sign in"</a>
                    </p>
                    <p class="mt-6 text-center text-sm text-muted" hidden=@{register}>
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
                    <p class="mt-2 text-sm text-muted" hidden={!setupKey}>
                        "Scan this QR code with your authenticator app, then enter its 6-digit code."
                    </p>
                    <p class="mt-2 text-sm text-muted" hidden={!!setupKey}>
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
                        <p class="mb-2 mt-4 text-xs text-muted">
                            "Can’t scan? Enter this setup key in your app:"
                        </p>
                        <code
                            class="block break-all rounded-lg border border-edge bg-sunken p-3 text-sm tracking-wider"
                        >
                            {setupKey}
                        </code>
                    </div>
                    <form class="mt-6 space-y-4" onsubmit={submitCode(event)}>
                        <label class="block text-sm">
                            {useRecovery ? "Recovery code" : "Authenticator code"}
                            <input
                                name="code"
                                pp-ref={codeInput}
                                type="text"
                                autocomplete="one-time-code"
                                inputmode={useRecovery ? "text" : "numeric"}
                                required
                                maxlength="32"
                                class="mt-2 h-12 w-full rounded-lg border border-edge bg-sunken px-3 text-lg tracking-widest outline-none focus:border-brand-ink"
                            />
                        </label>
                        <button
                            type="submit"
                            disabled={busy}
                            class="h-11 w-full rounded-lg bg-brand text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                        >
                            {busy ? "Verifying…" : "Verify and sign in"}
                        </button>
                    </form>
                    <button
                        type="button"
                        class="mt-4 text-sm text-link hover:underline"
                        hidden={!!setupKey}
                        onclick={setUseRecovery(!useRecovery)}
                    >
                        {useRecovery ? "Use authenticator code" : "Use a recovery code"}
                    </button>
                    <button
                        type="button"
                        disabled={busy}
                        onclick={restart()}
                        class="mt-4 block text-sm text-muted hover:text-ink"
                    >
                        "Start again"
                    </button>
                    <p class="mt-4 text-xs text-subtle">
                        "This step expires after 10 minutes. Keep your device’s clock set automatically."
                    </p>
                </div>
                <div hidden={stage !== "recovery"}>
                    <h1 class="text-2xl font-semibold">"Save your recovery codes"</h1>
                    <p class="mb-5 mt-2 text-sm leading-relaxed text-muted">
                        "Your account is ready. Save these codes somewhere safe before continuing. Each code works once if you lose access to your authenticator. These codes are shown only now."
                    </p>
                    <div
                        class="grid grid-cols-1 gap-2 rounded-lg border border-edge bg-sunken p-4 sm:grid-cols-2"
                    >
                        <template pp-for="item in recoveryCodes">
                            <code key={item} class="select-all font-mono text-xs">{item}</code>
                        </template>
                    </div>
                    <label class="my-5 flex items-start gap-3 text-sm text-muted">
                        <input
                            type="checkbox"
                            checked={saved}
                            onchange={setSaved(target.checked)}
                            class="mt-1"
                        />
                        "I saved my recovery codes somewhere safe."
                    </label>
                    <button
                        type="button"
                        onclick={continueToApp()}
                        disabled={!saved}
                        class="h-11 w-full rounded-lg bg-brand text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                    >
                        "Go to my workspace"
                    </button>
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
