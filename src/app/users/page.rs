use crate::{
    components::app_shell::AppShell,
    rahti::{Html, Json, Metadata, Result, html, rpc},
    users::{self, UserList, UserSummary},
};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Users")
        .description("Manage workspace users")
}
pub async fn page() -> Result<Html> {
    users::require_admin().await?;
    let listing = users::list(String::new(), 0).await?;
    Ok(html! {
        <AppShell active="users">
            <main class="min-w-0 flex-1 overflow-y-auto p-6 sm:p-10">
                <div class="mx-auto max-w-6xl">
                    <div class="mb-8 flex flex-wrap items-center justify-between gap-4">
                        <div>
                            <h1 class="text-3xl font-semibold">"Users"</h1>
                            <p class="mt-2 text-sm text-muted">
                                "Manage accounts and access to your workspace."
                            </p>
                        </div>
                        <button
                            type="button"
                            onclick={openCreate()}
                            disabled={busy}
                            class="rounded-lg bg-brand px-5 py-3 text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                        >
                            "Add user"
                        </button>
                    </div>
                    <section
                        hidden={mode === ""}
                        aria-label="User editor"
                        class="mb-6 rounded-xl border border-edge bg-surface p-6"
                    >
                        <div class="mb-5 flex items-center justify-between gap-3">
                            <h2 class="text-lg font-semibold">
                                {mode === "create" ? "Create user" : "Edit user"}
                            </h2>
                            <button
                                type="button"
                                onclick={closeEditor()}
                                disabled={busy}
                                class="text-sm text-muted hover:text-ink"
                            >
                                "Cancel"
                            </button>
                        </div>
                        <p class="mb-5 text-sm text-muted" hidden={mode !== "create"}>
                            "New accounts are active regular users. Share the initial password securely; the user will set up their own authenticator at first sign-in."
                        </p>
                        <form onsubmit={save(event)} class="grid gap-5 sm:grid-cols-2">
                            <label class="text-sm">
                                "Name"
                                <input
                                    name="name"
                                    value={draft.name}
                                    oninput={changeDraft("name", target.value)}
                                    pp-ref={nameInput}
                                    required
                                    maxlength="80"
                                    autocomplete="off"
                                    class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                                />
                            </label>
                            <label class="text-sm">
                                "Email address"
                                <input
                                    name="email"
                                    type="email"
                                    value={draft.email}
                                    oninput={changeDraft("email", target.value)}
                                    required
                                    maxlength="254"
                                    autocomplete="off"
                                    class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                                />
                            </label>
                            <label class="text-sm" hidden={mode !== "create"}>
                                "Initial password"
                                <input
                                    name="password"
                                    type="password"
                                    value={password}
                                    oninput={setPassword(target.value)}
                                    disabled={mode !== "create"}
                                    required={mode === "create"}
                                    minlength="12"
                                    maxlength="128"
                                    autocomplete="new-password"
                                    class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                                />
                            </label>
                            <label class="text-sm" hidden={mode !== "create"}>
                                "Confirm password"
                                <input
                                    name="confirmation"
                                    type="password"
                                    value={confirmation}
                                    oninput={setConfirmation(target.value)}
                                    disabled={mode !== "create"}
                                    required={mode === "create"}
                                    maxlength="128"
                                    autocomplete="new-password"
                                    class="mt-2 h-11 w-full rounded-lg border border-edge bg-sunken px-3 outline-none focus:border-brand-ink"
                                />
                            </label>
                            <div class="flex items-center gap-4 sm:col-span-2">
                                <button
                                    type="submit"
                                    disabled={busy}
                                    class="rounded-lg bg-brand px-5 py-2.5 text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                                >
                                    {busy ? "Saving…" : "Save user"}
                                </button>
                                <span class="text-xs text-subtle">"Role: User"</span>
                            </div>
                        </form>
                    </section>
                    <section
                        hidden={!deleting}
                        role="alertdialog"
                        aria-label="Delete user confirmation"
                        class="mb-6 rounded-xl border border-busy/50 bg-surface p-6"
                    >
                        <h2 class="text-lg font-semibold">"Delete user?"</h2>
                        <p class="my-3 text-sm text-muted">
                            {deleting ? deleting.email : ""}
                            " will lose access immediately. Their meeting links will be revoked. This cannot be undone."
                        </p>
                        <div class="flex gap-3">
                            <button
                                type="button"
                                onclick={confirmDelete()}
                                disabled={busy}
                                class="rounded-lg bg-busy px-4 py-2 text-sm text-white disabled:opacity-50"
                            >
                                "Delete permanently"
                            </button>
                            <button
                                type="button"
                                onclick={setDeleting(null)}
                                disabled={busy}
                                class="rounded-lg border border-edge px-4 py-2 text-sm hover:bg-hover"
                            >
                                "Cancel deletion"
                            </button>
                        </div>
                    </section>
                    <p
                        role="alert"
                        hidden={!error}
                        class="mb-5 rounded-lg border border-busy/40 p-3 text-sm text-danger-ink"
                    >
                        {error}
                    </p>
                    <p
                        role="status"
                        aria-live="polite"
                        hidden={!notice}
                        class="mb-5 rounded-lg border border-edge p-3 text-sm text-online"
                    >
                        {notice}
                    </p>
                    <section class="overflow-hidden rounded-xl border border-edge bg-surface">
                        <div
                            class="flex flex-wrap items-center justify-between gap-4 border-b border-line p-5"
                        >
                            <p class="text-sm font-semibold">{data.total}" accounts"</p>
                            <form
                                onsubmit={searchUsers(event)}
                                class="flex flex-1 gap-2 sm:max-w-md"
                            >
                                <input
                                    name="search"
                                    aria-label="Search users"
                                    value={search}
                                    oninput={setSearch(target.value)}
                                    maxlength="254"
                                    placeholder="Search name or email"
                                    class="h-10 min-w-0 flex-1 rounded-lg border border-edge bg-sunken px-3 text-sm outline-none focus:border-brand-ink"
                                />
                                <button
                                    type="submit"
                                    disabled={busy}
                                    class="rounded-lg border border-edge px-4 text-sm hover:bg-hover disabled:opacity-50"
                                >
                                    "Search"
                                </button>
                            </form>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left text-sm">
                                <thead class="bg-sunken text-xs uppercase tracking-wide text-muted">
                                    <tr>
                                        <th class="px-5 py-4">"User"</th>
                                        <th class="px-5 py-4">"Role"</th>
                                        <th class="px-5 py-4">"Status"</th>
                                        <th class="px-5 py-4">"Authenticator"</th>
                                        <th class="px-5 py-4 text-right">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <template pp-for="row in data.items">
                                        <tr
                                            key={row.id}
                                            data-email={row.email}
                                            class="border-t border-line"
                                        >
                                            <td class="px-5 py-4">
                                                <span class="block font-medium">{row.name}</span>
                                                <span class="mt-1 block text-xs text-muted">
                                                    {row.email}
                                                </span>
                                            </td>
                                            <td class="px-5 py-4">
                                                {row.role === "admin" ? "Admin" : "User"}
                                            </td>
                                            <td class="px-5 py-4">
                                                <span
                                                    class={row.active ? "text-online" : "text-danger-ink"}
                                                >
                                                    {row.active ? "Active" : "Inactive"}
                                                </span>
                                            </td>
                                            <td class="px-5 py-4 text-muted">
                                                {row.enrolled ? "Enrolled" : "Setup pending"}
                                            </td>
                                            <td class="px-5 py-4 text-right">
                                                <span
                                                    hidden={row.role !== "admin"}
                                                    class="text-xs text-subtle"
                                                >
                                                    "Protected administrator"
                                                </span>
                                                <div
                                                    hidden={row.role === "admin"}
                                                    class="flex justify-end gap-3"
                                                >
                                                    <button
                                                        type="button"
                                                        onclick={openEdit(row)}
                                                        disabled={busy}
                                                        class="text-link hover:underline disabled:opacity-50"
                                                    >
                                                        "Edit"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onclick={toggleActive(row)}
                                                        disabled={busy}
                                                        class="text-link hover:underline disabled:opacity-50"
                                                    >
                                                        {row.active ? "Deactivate" : "Activate"}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onclick={askDelete(row)}
                                                        disabled={busy}
                                                        class="text-danger-ink hover:underline disabled:opacity-50"
                                                    >
                                                        "Delete"
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    </template>
                                </tbody>
                            </table>
                            <p
                                hidden={data.items.length > 0}
                                class="p-10 text-center text-sm text-muted"
                            >
                                "No users match your search."
                            </p>
                        </div>
                        <div
                            class="flex items-center justify-between border-t border-line p-5 text-sm"
                        >
                            <button
                                type="button"
                                onclick={loadPage(data.page - 1)}
                                disabled={busy || data.page === 0}
                                class="text-link disabled:opacity-40"
                            >
                                "Previous"
                            </button>
                            <span class="text-muted">"Page "{data.page + 1}</span>
                            <button
                                type="button"
                                onclick={loadPage(data.page + 1)}
                                disabled={busy || (data.page + 1) * 25 >= data.total}
                                class="text-link disabled:opacity-40"
                            >
                                "Next"
                            </button>
                        </div>
                    </section>
                    <p class="mt-4 text-xs text-subtle">
                        "Only active accounts can enter the workspace. The administrator account is managed through database seeding."
                    </p>
                </div>
                <script>r##"
                    const [data, setData] = pp.state(@{Json(&listing)});
                    const [search, setSearch] = pp.state('');
                    const [appliedSearch, setAppliedSearch] = pp.state('');
                    const [mode, setMode] = pp.state('');
                    const [draft, setDraft] = pp.state({id:'', name:'', email:''});
                    const [password, setPassword] = pp.state('');
                    const [confirmation, setConfirmation] = pp.state('');
                    const [busy, setBusy] = pp.state(false);
                    const [error, setError] = pp.state('');
                    const [notice, setNotice] = pp.state('');
                    const [deleting, setDeleting] = pp.state(null);
                    const nameInput = pp.ref(null);
                    pp.effect(() => { if (mode) nameInput.current?.focus(); }, [mode]);
                    function changeDraft(field, value) { setDraft(previous => ({...previous, [field]:value})); }
                    function closeEditor() { setMode(''); setPassword(''); setConfirmation(''); }
                    function openCreate() { setDraft({id:'',name:'',email:''}); setPassword(''); setConfirmation(''); setError(''); setNotice(''); setDeleting(null); setMode('create'); }
                    function openEdit(row) { setDraft({id:row.id,name:row.name,email:row.email}); setPassword(''); setConfirmation(''); setError(''); setNotice(''); setDeleting(null); setMode('edit'); }
                    function askDelete(row) { closeEditor(); setError(''); setNotice(''); setDeleting(row); }
                    function message(e) { return e.body?.error || e.message || 'Could not complete the request.'; }
                    async function loadPage(page) {
                        if (busy) return; setBusy(true); setError('');
                        try { setData(await pp.rpc('list_users', {search: appliedSearch, page})); }
                        catch (e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    async function searchUsers(event) {
                        event.preventDefault(); if (busy) return;
                        const values = Object.fromEntries(new FormData(event.currentTarget));
                        setBusy(true); setError('');
                        try { const result = await pp.rpc('list_users', {search:values.search, page:0}); setData(result); setAppliedSearch(values.search); }
                        catch(e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    async function save(event) {
                        event.preventDefault(); if (busy) return;
                        const values = Object.fromEntries(new FormData(event.currentTarget));
                        setBusy(true); setError(''); setNotice('');
                        try {
                            if (mode === 'create') await pp.rpc('create_user', values);
                            else await pp.rpc('update_user', {id:draft.id, name:values.name, email:values.email});
                            closeEditor(); setNotice('User saved.');
                            setData(await pp.rpc('list_users', {search: appliedSearch, page:data.page}));
                        } catch(e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    async function toggleActive(row) {
                        if (busy) return; setBusy(true); setError(''); setNotice('');
                        try {
                            await pp.rpc('set_user_active', {id:row.id, active:!row.active});
                            setNotice(row.active ? 'User deactivated. Existing sessions are blocked.' : 'User activated. They can sign in again.');
                            setData(await pp.rpc('list_users', {search: appliedSearch, page:data.page}));
                        } catch(e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    async function confirmDelete() {
                        if (busy || !deleting) return; setBusy(true); setError(''); setNotice('');
                        try {
                            await pp.rpc('delete_user', {id:deleting.id}); setDeleting(null); setNotice('User deleted.');
                            setData(await pp.rpc('list_users', {search: appliedSearch, page:data.page}));
                        } catch(e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                "##</script>
            </main>
        </AppShell>
    })
}
#[rpc(auth)]
pub async fn list_users(search: String, page: u64) -> Result<UserList> {
    users::list(search, page).await
}
#[rpc(auth)]
pub async fn create_user(
    name: String,
    email: String,
    password: String,
    confirmation: String,
) -> Result<UserSummary> {
    users::create(name, email, password, confirmation).await
}
#[rpc(auth)]
pub async fn update_user(id: String, name: String, email: String) -> Result<UserSummary> {
    users::update(id, name, email).await
}
#[rpc(auth)]
pub async fn set_user_active(id: String, active: bool) -> Result<UserSummary> {
    users::set_active(id, active).await
}
#[rpc(auth)]
pub async fn delete_user(id: String) -> Result<bool> {
    users::delete(id).await
}
