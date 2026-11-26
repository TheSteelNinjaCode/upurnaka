use crate::{
    components::{
        app_shell::AppShell,
        aria::{bound, hidden, named, submit},
        rahti_ui::{
            alert_dialog::{
                AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
                AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
                AlertDialogTrigger,
            },
            attrs::{Attrs, ui_expr},
            badge::Badge,
            button::{Button, ButtonSize, ButtonVariant, button_variants},
            dialog::{
                Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
                DialogTitle, DialogTrigger,
            },
            field::{Field, FieldLabel},
            input::Input,
            table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
        },
    },
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
    let cancel = button_variants(ButtonVariant::Outline, ButtonSize::Default, "");
    Ok(html! {
        <AppShell active="users">
            <main class="min-w-0 flex-1 overflow-y-auto p-6 sm:p-10">
                <div class="mx-auto max-w-6xl">
                    <div class="mb-8 flex flex-wrap items-center justify-between gap-4">
                        <div>
                            <h1 class="text-3xl font-semibold">"Users"</h1>
                            <p class="mt-2 text-sm text-muted-foreground">
                                "Manage accounts and access to your workspace."
                            </p>
                        </div>
                        <Button
                            size="lg"
                            class="font-semibold"
                            attrs=@{named("Add user")}
                            onclick={openCreate()}
                            disabled={busy}
                        >
                            "Add user"
                        </Button>
                    </div>
                    <p
                        role="alert"
                        hidden={!error || !!mode}
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
                                <Input
                                    name="search"
                                    type="search"
                                    placeholder="Search name or email"
                                    class="h-10! min-w-0 flex-1"
                                    attrs=@{named("Search users").set("maxlength", "254")}
                                    value={search}
                                    oninput={setSearch(target.value)}
                                />
                                <Button
                                    variant="outline"
                                    class="h-10!"
                                    attrs=@{submit()}
                                    disabled={busy}
                                >
                                    "Search"
                                </Button>
                            </form>
                        </div>
                        <Table>
                            <TableHeader class="bg-sunken">
                                <TableRow class="hover:bg-transparent">
                                    <TableHead
                                        class="px-5 text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "User"
                                    </TableHead>
                                    <TableHead
                                        class="px-5 text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Role"
                                    </TableHead>
                                    <TableHead
                                        class="px-5 text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Status"
                                    </TableHead>
                                    <TableHead
                                        class="px-5 text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Authenticator"
                                    </TableHead>
                                    <TableHead
                                        class="px-5 text-right text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Actions"
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <template pp-for="row in data.items">
                                    // Keyed by what the row shows, not by id alone: a reused
                                    // row does not re-render the text inside its cells, so a
                                    // user whose name or state changed gets a fresh row.
                                    <TableRow
                                        key={[row.id, row.name, row.email, row.active, row.enrolled].join("|")}
                                        attrs=@{bound("data-email", ui_expr!(data-email={row.email}))}
                                    >
                                        <TableCell class="px-5 py-4">
                                            <span class="block font-medium">{row.name}</span>
                                            <span class="mt-1 block text-xs text-muted-foreground">
                                                {row.email}
                                            </span>
                                        </TableCell>
                                        <TableCell class="px-5 py-4">
                                            {row.role === "admin" ? "Admin" : "User"}
                                        </TableCell>
                                        <TableCell class="px-5 py-4">
                                            <span
                                                class={row.active ? "inline-flex rounded-md border border-online/40 bg-online/10 px-2 py-0.5 text-xs font-medium text-online" : "inline-flex rounded-md border border-busy/40 bg-busy/10 px-2 py-0.5 text-xs font-medium text-danger-ink"}
                                            >
                                                {row.active ? "Active" : "Inactive"}
                                            </span>
                                        </TableCell>
                                        <TableCell class="px-5 py-4 text-muted-foreground">
                                            {row.enrolled ? "Enrolled" : "Setup pending"}
                                        </TableCell>
                                        <TableCell class="px-5 py-4 text-right">
                                            <span
                                                hidden={row.role !== "admin"}
                                                class="text-xs text-subtle"
                                            >
                                                "Protected administrator"
                                            </span>
                                            <div
                                                hidden={row.role === "admin"}
                                                class="flex justify-end gap-1"
                                            >
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-link"
                                                    value={row.id}
                                                    onclick={openEdit(rowById(event.currentTarget.value))}
                                                >
                                                    "Edit"
                                                </Button>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-link"
                                                    value={row.id}
                                                    onclick={toggleActive(rowById(event.currentTarget.value))}
                                                >
                                                    {row.active ? "Deactivate" : "Activate"}
                                                </Button>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-danger-ink hover:bg-destructive/10"
                                                    value={row.id}
                                                    onclick={askDelete(rowById(event.currentTarget.value))}
                                                >
                                                    "Delete"
                                                </Button>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                </template>
                            </TableBody>
                        </Table>
                        <p
                            hidden={data.items.length > 0}
                            class="p-10 text-center text-sm text-muted-foreground"
                        >
                            "No users match your search."
                        </p>
                        <div
                            class="flex items-center justify-between border-t border-line p-5 text-sm"
                        >
                            <Button
                                variant="outline"
                                size="sm"
                                onclick={loadPage(data.page - 1)}
                                disabled={busy || data.page === 0}
                            >
                                "Previous"
                            </Button>
                            <span class="text-muted-foreground">"Page "{data.page + 1}</span>
                            <Button
                                variant="outline"
                                size="sm"
                                onclick={loadPage(data.page + 1)}
                                disabled={busy || (data.page + 1) * 25 >= data.total}
                            >
                                "Next"
                            </Button>
                        </div>
                    </section>
                    <p class="mt-4 text-xs text-subtle">
                        "Only active accounts can enter the workspace. The administrator account was created when the workspace was first set up and cannot be changed here."
                    </p>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={editorOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={editorCloser}></span>
                    </DialogClose>
                    <DialogContent
                        class="sm:max-w-xl!"
                        attrs=@{named("User editor")}
                        onclose={closeEditor()}
                    >
                        <DialogHeader>
                            <DialogTitle>
                                {mode === "create" ? "Create user" : "Edit user"}
                            </DialogTitle>
                            <DialogDescription hidden={mode !== "create"}>
                                "New accounts are active regular users. Share the initial password securely; the user will set up their own authenticator at first sign-in."
                            </DialogDescription>
                        </DialogHeader>
                        <form onsubmit={save(event)} class="grid gap-5 sm:grid-cols-2">
                            <Field>
                                <FieldLabel for="user-name">"Name"</FieldLabel>
                                <Input
                                    id="user-name"
                                    name="name"
                                    required
                                    class="h-11!"
                                    attrs=@{Attrs::new()
                                        .set("maxlength", "80")
                                        .set("autocomplete", "off")
                                        .set("autofocus", "")}
                                    value={draft.name}
                                    oninput={changeDraft("name", target.value)}
                                />
                            </Field>
                            <Field>
                                <FieldLabel for="user-email">"Email address"</FieldLabel>
                                <Input
                                    id="user-email"
                                    name="email"
                                    type="email"
                                    required
                                    class="h-11!"
                                    attrs=@{Attrs::new().set("maxlength", "254").set("autocomplete", "off")}
                                    value={draft.email}
                                    oninput={changeDraft("email", target.value)}
                                />
                            </Field>
                            <Field hidden={mode !== "create"}>
                                <FieldLabel for="user-password">"Initial password"</FieldLabel>
                                <Input
                                    id="user-password"
                                    name="password"
                                    type="password"
                                    class="h-11!"
                                    attrs=@{Attrs::new()
                                        .set("minlength", "12")
                                        .set("maxlength", "128")
                                        .set("autocomplete", "new-password")}
                                    value={password}
                                    oninput={setPassword(target.value)}
                                    disabled={mode !== "create"}
                                    required={mode === "create"}
                                />
                            </Field>
                            <Field hidden={mode !== "create"}>
                                <FieldLabel for="user-confirmation">"Confirm password"</FieldLabel>
                                <Input
                                    id="user-confirmation"
                                    name="confirmation"
                                    type="password"
                                    class="h-11!"
                                    attrs=@{Attrs::new()
                                        .set("maxlength", "128")
                                        .set("autocomplete", "new-password")}
                                    value={confirmation}
                                    oninput={setConfirmation(target.value)}
                                    disabled={mode !== "create"}
                                    required={mode === "create"}
                                />
                            </Field>
                            <p
                                role="alert"
                                hidden={!error}
                                class="rounded-lg border border-busy/40 p-3 text-sm text-danger-ink sm:col-span-2"
                            >
                                {error}
                            </p>
                            <DialogFooter class="items-center sm:col-span-2 sm:justify-between!">
                                <Badge variant="secondary">"Role: User"</Badge>
                                <div class="flex flex-col-reverse gap-2 sm:flex-row">
                                    <DialogClose class=@{cancel.as_str()} disabled={busy}>
                                        "Cancel"
                                    </DialogClose>
                                    <Button attrs=@{submit()} disabled={busy}>
                                        {busy ? "Saving…" : "Save user"}
                                    </Button>
                                </div>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
                <AlertDialog>
                    <AlertDialogTrigger attrs=@{hidden()}>
                        <span pp-ref={deleteOpener}></span>
                    </AlertDialogTrigger>
                    <AlertDialogContent attrs=@{named("Delete user confirmation")}>
                        <AlertDialogHeader>
                            <AlertDialogTitle>"Delete user?"</AlertDialogTitle>
                            <AlertDialogDescription>
                                {deleting ? deleting.email : ""}
                                " will lose access immediately. Their meeting links will be revoked. This cannot be undone."
                            </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                            <AlertDialogCancel onclick={setDeleting(null)}>
                                "Cancel deletion"
                            </AlertDialogCancel>
                            <AlertDialogAction variant="destructive" onclick={confirmDelete()}>
                                "Delete permanently"
                            </AlertDialogAction>
                        </AlertDialogFooter>
                    </AlertDialogContent>
                </AlertDialog>
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
                    const editorOpener = pp.ref(null);
                    const editorCloser = pp.ref(null);
                    const deleteOpener = pp.ref(null);
                    // The editor and the confirmation are rahti-ui dialogs, which
                    // open from a trigger; a row reaches them through the hidden
                    // trigger each one carries, once it has said which user.
                    function openEditor(next) {
                        setMode(next);
                        editorOpener.current?.click();
                    }
                    function changeDraft(field, value) { setDraft(previous => ({...previous, [field]:value})); }
                    function closeEditor() { setMode(''); setPassword(''); setConfirmation(''); }
                    function openCreate() { setDraft({id:'',name:'',email:''}); setPassword(''); setConfirmation(''); setError(''); setNotice(''); setDeleting(null); openEditor('create'); }
                    const openEdit = pp.event(function openEdit(row) { if (!row) return; setDraft({id:row.id,name:row.name,email:row.email}); setPassword(''); setConfirmation(''); setError(''); setNotice(''); setDeleting(null); openEditor('edit'); });
                    // A row's actions name their user by id: a handler in a list
                    // row reads the row through its button, not the loop variable.
                    const rowById = pp.event(function rowById(id) { return data.items.find(item => item.id === id); });
                    const askDelete = pp.event(function askDelete(row) { if (!row) return; setError(''); setNotice(''); setDeleting(row); deleteOpener.current?.click(); });
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
                            editorCloser.current?.click(); setNotice('User saved.');
                            setData(await pp.rpc('list_users', {search: appliedSearch, page:data.page}));
                        } catch(e) { setError(message(e)); }
                        finally { setBusy(false); }
                    }
                    const toggleActive = pp.event(async function toggleActive(row) {
                        if (busy || !row) return; setBusy(true); setError(''); setNotice('');
                        try {
                            await pp.rpc('set_user_active', {id:row.id, active:!row.active});
                            setNotice(row.active ? 'User deactivated. Existing sessions are blocked.' : 'User activated. They can sign in again.');
                            setData(await pp.rpc('list_users', {search: appliedSearch, page:data.page}));
                        } catch(e) { setError(message(e)); }
                        finally { setBusy(false); }
                    });
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
