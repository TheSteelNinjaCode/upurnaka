//! Files: everything this person keeps in the workspace, in one place.
//!
//! "My files" is the same drive the chat composer and the cloud file picker
//! use (`crate::chat_files`): folders, uploads, rename, move and delete.
//! "Recordings" lists the meeting recordings this person started, with
//! download and delete (`crate::meetings::recording`).

use crate::{
    chat_files::{self, Attachment, Folder, Listing, Place, Usage},
    components::{
        app_shell::AppShell,
        aria::{hidden, named, submit},
        icon::Icon,
        rahti_ui::{
            alert_dialog::{
                AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
                AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
                AlertDialogTrigger,
            },
            attrs::Attrs,
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
    meetings::recording::{self, Mine},
    rahti::{Html, Json, Metadata, Result, RpcFile, html, rpc},
};
use serde::Serialize;

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Files")
        .description("Your uploaded files and meeting recordings")
}

/// What the page starts with, and what a refresh of My files returns.
#[derive(Serialize)]
pub struct Drive {
    pub listing: Listing,
    pub usage: Usage,
}

async fn drive(view: String, folder: Option<String>) -> Result<Drive> {
    Ok(Drive {
        listing: chat_files::browse(&view, folder).await?,
        usage: chat_files::usage().await?,
    })
}

pub async fn page() -> Result<Html> {
    let start = drive("files".into(), None).await?;
    let recordings = recording::mine().await?;
    let cancel = button_variants(ButtonVariant::Outline, ButtonSize::Default, "");
    let download = button_variants(ButtonVariant::Ghost, ButtonSize::Sm, "text-link");
    let head = "px-4 text-xs uppercase tracking-wide text-muted-foreground";
    Ok(html! {
        <AppShell active="files">
            <main
                class="min-w-0 flex-1 overflow-y-auto p-6 sm:p-10"
                ondragover={allowDrop(event)}
                ondrop={dropFiles(event)}
            >
                <div class="mx-auto max-w-6xl">
                    <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
                        <div>
                            <h1 class="text-3xl font-semibold">"Files"</h1>
                            <p class="mt-2 text-sm text-muted-foreground">{usageText()}</p>
                        </div>
                        <div class="flex flex-wrap gap-2" hidden={tab !== "files"}>
                            <Button
                                variant="outline"
                                size="lg"
                                attrs=@{named("New folder")}
                                onclick={openNewFolder()}
                                disabled={busy}
                            >
                                <Icon name="folder-plus" class="h-4 w-4" />
                                "New folder"
                            </Button>
                            <Button
                                size="lg"
                                class="font-semibold"
                                attrs=@{named("Upload files")}
                                onclick={openPicker()}
                                disabled={busy}
                            >
                                <Icon name="upload" class="h-4 w-4" />
                                {uploading ? "Uploading…" : "Upload"}
                            </Button>
                            <input
                                type="file"
                                multiple
                                hidden
                                aria-label="Choose files to upload"
                                pp-ref={picker}
                                onchange={uploadPicked(event)}
                            />
                        </div>
                    </div>
                    <div
                        role="tablist"
                        aria-label="File sections"
                        class="mb-5 inline-flex gap-1 rounded-lg border border-edge bg-surface p-1"
                    >
                        <button
                            type="button"
                            role="tab"
                            aria-selected={tab === "files"}
                            class={tab === "files" ? "rounded-md bg-raised px-3 py-1.5 text-sm font-medium text-brand-ink" : "rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:text-ink"}
                            onclick={showTab("files")}
                        >
                            "My files"
                        </button>
                        <button
                            type="button"
                            role="tab"
                            aria-selected={tab === "recordings"}
                            class={tab === "recordings" ? "rounded-md bg-raised px-3 py-1.5 text-sm font-medium text-brand-ink" : "rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:text-ink"}
                            onclick={showTab("recordings")}
                        >
                            "Recordings"
                        </button>
                    </div>
                    <p
                        role="alert"
                        hidden={!error || !!dialog}
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
                    <section
                        hidden={tab !== "files"}
                        class={dragging ? "overflow-hidden rounded-xl border border-brand-ink bg-surface" : "overflow-hidden rounded-xl border border-edge bg-surface"}
                        aria-label="My files"
                    >
                        <div
                            class="flex flex-wrap items-center justify-between gap-3 border-b border-line p-4"
                        >
                            <nav
                                aria-label="Folder path"
                                class="flex min-w-0 flex-wrap items-center gap-1 text-sm"
                            >
                                <button
                                    type="button"
                                    class={listing.folder ? "rounded px-1.5 py-0.5 text-link hover:bg-hover" : "rounded px-1.5 py-0.5 font-semibold"}
                                    onclick={openFolder("")}
                                >
                                    "My files"
                                </button>
                                <template pp-for="crumb in listing.path">
                                    <span key={crumb.id} class="flex items-center gap-1">
                                        <span class="text-subtle">"/"</span>
                                        <button
                                            type="button"
                                            class={crumb.id === listing.folder ? "max-w-48 truncate rounded px-1.5 py-0.5 font-semibold" : "max-w-48 truncate rounded px-1.5 py-0.5 text-link hover:bg-hover"}
                                            value={crumb.id}
                                            onclick={openFolder(event.currentTarget.value)}
                                        >
                                            {crumb.name}
                                        </button>
                                    </span>
                                </template>
                            </nav>
                            <Input
                                type="search"
                                placeholder="Filter this folder"
                                class="h-9! w-full sm:w-64"
                                attrs=@{named("Filter files").set("maxlength", "100")}
                                value={query}
                                oninput={setQuery(target.value)}
                            />
                        </div>
                        <div
                            hidden={selected.length === 0}
                            class="flex flex-wrap items-center gap-2 border-b border-line bg-sunken px-4 py-2 text-sm"
                        >
                            <span class="font-medium">{selected.length}" selected"</span>
                            <Button
                                variant="ghost"
                                size="sm"
                                onclick={askMove(selected)}
                                disabled={busy}
                            >
                                "Move"
                            </Button>
                            <Button
                                variant="ghost"
                                size="sm"
                                class="text-danger-ink hover:bg-destructive/10"
                                onclick={askDeleteSelected()}
                                disabled={busy}
                            >
                                "Delete"
                            </Button>
                            <Button variant="ghost" size="sm" onclick={setSelected([])}>
                                "Clear selection"
                            </Button>
                        </div>
                        <Table>
                            <TableHeader class="bg-sunken">
                                <TableRow class="hover:bg-transparent">
                                    <TableHead class="w-10 px-4">
                                        <input
                                            type="checkbox"
                                            aria-label="Select all files"
                                            class="h-4 w-4 accent-brand-ink"
                                            checked={shownFiles.length > 0 && shownFiles.every(file => selected.includes(file.id))}
                                            disabled={shownFiles.length === 0}
                                            onchange={selectAll(event.currentTarget.checked)}
                                        />
                                    </TableHead>
                                    <TableHead class=@{head}>
                                        <button
                                            type="button"
                                            class="uppercase hover:text-ink"
                                            onclick={sortBy("name")}
                                        >
                                            "Name"
                                            {arrow("name")}
                                        </button>
                                    </TableHead>
                                    <TableHead class=@{head}>
                                        <button
                                            type="button"
                                            class="uppercase hover:text-ink"
                                            onclick={sortBy("modified")}
                                        >
                                            "Modified"
                                            {arrow("modified")}
                                        </button>
                                    </TableHead>
                                    <TableHead class=@{head}>"Type"</TableHead>
                                    <TableHead class=@{head}>
                                        <button
                                            type="button"
                                            class="uppercase hover:text-ink"
                                            onclick={sortBy("size")}
                                        >
                                            "Size"
                                            {arrow("size")}
                                        </button>
                                    </TableHead>
                                    <TableHead
                                        class="px-4 text-right text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Actions"
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <template pp-for="folder in shownFolders">
                                    <TableRow
                                        key={["folder", folder.id, folder.name, folder.items, folder.bytes].join("|")}
                                    >
                                        <TableCell class="px-4 py-3"></TableCell>
                                        <TableCell class="px-4 py-3">
                                            <button
                                                type="button"
                                                class="flex items-center gap-2 text-left font-medium hover:text-link"
                                                value={folder.id}
                                                onclick={openFolder(event.currentTarget.value)}
                                            >
                                                <Icon
                                                    name="folder"
                                                    class="h-4 w-4 shrink-0 text-brand-ink"
                                                />
                                                <span class="truncate">{folder.name}</span>
                                            </button>
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {folder.modified}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {folder.items === 1 ? "Folder · 1 item" : "Folder · " + folder.items + " items"}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {size(folder.bytes)}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-right">
                                            <div class="flex justify-end gap-1">
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-link"
                                                    value={folder.id}
                                                    onclick={askRenameFolder(event.currentTarget.value)}
                                                >
                                                    "Rename"
                                                </Button>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-danger-ink hover:bg-destructive/10"
                                                    value={folder.id}
                                                    onclick={askDeleteFolder(event.currentTarget.value)}
                                                >
                                                    "Delete"
                                                </Button>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                </template>
                                <template pp-for="file in shownFiles">
                                    <TableRow
                                        key={["file", file.id, file.name, selected.includes(file.id)].join("|")}
                                    >
                                        <TableCell class="px-4 py-3">
                                            <input
                                                type="checkbox"
                                                aria-label={"Select " + file.name}
                                                class="h-4 w-4 accent-brand-ink"
                                                value={file.id}
                                                checked={selected.includes(file.id)}
                                                onchange={toggleSelected(event.currentTarget.value)}
                                            />
                                        </TableCell>
                                        <TableCell class="max-w-sm px-4 py-3">
                                            <span class="flex items-center gap-2 font-medium">
                                                <Icon
                                                    name="file"
                                                    class="h-4 w-4 shrink-0 text-muted-foreground"
                                                />
                                                <span class="truncate" title={file.name}>
                                                    {file.name}
                                                </span>
                                            </span>
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {file.uploaded}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {kind(file.content_type, file.name)}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {size(file.bytes)}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-right">
                                            <div class="flex justify-end gap-1">
                                                <a
                                                    class=@{download.as_str()}
                                                    href={"/chat/files/" + encodeURIComponent(file.id)}
                                                    target="_blank"
                                                    rel="noopener"
                                                >
                                                    "Download"
                                                </a>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-link"
                                                    value={file.id}
                                                    onclick={askRenameFile(event.currentTarget.value)}
                                                >
                                                    "Rename"
                                                </Button>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-link"
                                                    value={file.id}
                                                    onclick={askMove([event.currentTarget.value])}
                                                >
                                                    "Move"
                                                </Button>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-danger-ink hover:bg-destructive/10"
                                                    value={file.id}
                                                    onclick={askDeleteFiles([event.currentTarget.value])}
                                                >
                                                    "Delete"
                                                </Button>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                </template>
                            </TableBody>
                        </Table>
                        <div
                            hidden={shownFolders.length + shownFiles.length > 0}
                            class="p-10 text-center text-sm text-muted-foreground"
                        >
                            <p hidden={!!query}>
                                "This folder is empty. Upload files, or drop them anywhere on this page."
                            </p>
                            <p hidden={!query}>"Nothing in this folder matches your filter."</p>
                        </div>
                    </section>
                    <section
                        hidden={tab !== "recordings"}
                        class="overflow-hidden rounded-xl border border-edge bg-surface"
                        aria-label="Recordings"
                    >
                        <div class="border-b border-line p-4 text-sm text-muted-foreground">
                            "Meetings you recorded. Download a recording or delete one you no longer need."
                        </div>
                        <div
                            hidden={pickedRecordings.length === 0}
                            class="recording-selection flex flex-wrap items-center gap-2 border-b border-line bg-sunken px-4 py-2 text-sm"
                        >
                            <span class="font-medium">{pickedRecordings.length}" selected"</span>
                            <Button
                                variant="ghost"
                                size="sm"
                                class="text-danger-ink hover:bg-destructive/10"
                                onclick={askDeleteRecordings()}
                                disabled={busy}
                            >
                                "Delete"
                            </Button>
                            <Button variant="ghost" size="sm" onclick={setPickedRecordings([])}>
                                "Clear selection"
                            </Button>
                        </div>
                        <Table>
                            <TableHeader class="bg-sunken">
                                <TableRow class="hover:bg-transparent">
                                    <TableHead class="w-10 px-4">
                                        <input
                                            type="checkbox"
                                            aria-label="Select all recordings"
                                            class="h-4 w-4 accent-brand-ink"
                                            checked={deletableRecordings.length > 0 && deletableRecordings.every(item => pickedRecordings.includes(item.id))}
                                            disabled={deletableRecordings.length === 0}
                                            onchange={selectAllRecordings(event.currentTarget.checked)}
                                        />
                                    </TableHead>
                                    <TableHead class=@{head}>"Meeting"</TableHead>
                                    <TableHead class=@{head}>"Recorded"</TableHead>
                                    <TableHead class=@{head}>"Length"</TableHead>
                                    <TableHead class=@{head}>"Size"</TableHead>
                                    <TableHead class=@{head}>"Status"</TableHead>
                                    <TableHead
                                        class="px-4 text-right text-xs uppercase tracking-wide text-muted-foreground"
                                    >
                                        "Actions"
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <template pp-for="item in recordings">
                                    <TableRow
                                        key={[item.id, item.state, item.bytes, pickedRecordings.includes(item.id)].join("|")}
                                    >
                                        <TableCell class="px-4 py-3">
                                            <input
                                                type="checkbox"
                                                aria-label={"Select recording of " + item.meeting + " from " + item.started}
                                                class="h-4 w-4 accent-brand-ink"
                                                value={item.id}
                                                checked={pickedRecordings.includes(item.id)}
                                                disabled={!canDeleteRecording(item)}
                                                onchange={toggleRecording(event.currentTarget.value)}
                                            />
                                        </TableCell>
                                        <TableCell class="max-w-sm px-4 py-3">
                                            <span class="block truncate font-medium">
                                                {item.meeting}
                                            </span>
                                            <span
                                                class="mt-1 block truncate text-xs text-muted-foreground"
                                            >
                                                {item.name}
                                            </span>
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {item.started}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {duration(item.seconds)}
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-muted-foreground">
                                            {size(item.bytes)}
                                        </TableCell>
                                        <TableCell class="px-4 py-3">
                                            <span class={stateClass(item.state)}>
                                                {stateLabel(item.state)}
                                            </span>
                                        </TableCell>
                                        <TableCell class="px-4 py-3 text-right">
                                            <div class="flex justify-end gap-1">
                                                <a
                                                    class=@{download.as_str()}
                                                    hidden={item.state !== "stored"}
                                                    href={"/files/recordings/" + encodeURIComponent(item.id)}
                                                    target="_blank"
                                                    rel="noopener"
                                                >
                                                    "Download"
                                                </a>
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    class="text-danger-ink hover:bg-destructive/10"
                                                    value={item.id}
                                                    disabled={item.state === "recording" || item.state === "uploading"}
                                                    onclick={askDeleteRecording(event.currentTarget.value)}
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
                            hidden={recordings.length > 0}
                            class="p-10 text-center text-sm text-muted-foreground"
                        >
                            "You have no meeting recordings yet. Recordings you start in Meet appear here."
                        </p>
                    </section>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={nameOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={nameCloser}></span>
                    </DialogClose>
                    <DialogContent attrs=@{named("Name editor")} onclose={closeDialog()}>
                        <DialogHeader>
                            <DialogTitle>{nameTitle()}</DialogTitle>
                            <DialogDescription>
                                {dialog === "new-folder" ? "The folder is created in the folder you are viewing." : "Only the name changes; the contents stay the same."}
                            </DialogDescription>
                        </DialogHeader>
                        <form onsubmit={saveName(event)} class="grid gap-4">
                            <Field>
                                <FieldLabel for="files-name">"Name"</FieldLabel>
                                <Input
                                    id="files-name"
                                    name="name"
                                    required
                                    class="h-11!"
                                    attrs=@{Attrs::new()
                                        .set("maxlength", "200")
                                        .set("autocomplete", "off")
                                        .set("autofocus", "")}
                                    value={draftName}
                                    oninput={setDraftName(target.value)}
                                />
                            </Field>
                            <p
                                role="alert"
                                hidden={!error}
                                class="rounded-lg border border-busy/40 p-3 text-sm text-danger-ink"
                            >
                                {error}
                            </p>
                            <DialogFooter>
                                <DialogClose class=@{cancel.as_str()} disabled={busy}>
                                    "Cancel"
                                </DialogClose>
                                <Button attrs=@{submit()} disabled={busy}>
                                    {busy ? "Saving…" : "Save"}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={moveOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={moveCloser}></span>
                    </DialogClose>
                    <DialogContent attrs=@{named("Move files")} onclose={closeDialog()}>
                        <DialogHeader>
                            <DialogTitle>
                                {moving.length === 1 ? "Move 1 file" : "Move " + moving.length + " files"}
                            </DialogTitle>
                            <DialogDescription>"Choose the folder to move to."</DialogDescription>
                        </DialogHeader>
                        <form onsubmit={saveMove(event)} class="grid gap-4">
                            <Field>
                                <FieldLabel for="files-destination">"Destination"</FieldLabel>
                                <select
                                    id="files-destination"
                                    name="folder"
                                    class="h-11 rounded-md border border-edge bg-sunken px-3 text-sm"
                                    value={destination}
                                    onchange={setDestination(event.currentTarget.value)}
                                >
                                    <option value="">"My files (top level)"</option>
                                    <template pp-for="place in places">
                                        <option key={place.id} value={place.id}>
                                            {place.path}
                                        </option>
                                    </template>
                                </select>
                            </Field>
                            <p
                                role="alert"
                                hidden={!error}
                                class="rounded-lg border border-busy/40 p-3 text-sm text-danger-ink"
                            >
                                {error}
                            </p>
                            <DialogFooter>
                                <DialogClose class=@{cancel.as_str()} disabled={busy}>
                                    "Cancel"
                                </DialogClose>
                                <Button attrs=@{submit()} disabled={busy}>
                                    {busy ? "Moving…" : "Move"}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
                <AlertDialog>
                    <AlertDialogTrigger attrs=@{hidden()}>
                        <span pp-ref={deleteOpener}></span>
                    </AlertDialogTrigger>
                    <AlertDialogContent attrs=@{named("Delete confirmation")}>
                        <AlertDialogHeader>
                            <AlertDialogTitle>{deleting ? deleting.title : ""}</AlertDialogTitle>
                            <AlertDialogDescription>
                                {deleting ? deleting.detail : ""}
                            </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                            <AlertDialogCancel onclick={setDeleting(null)}>
                                "Cancel"
                            </AlertDialogCancel>
                            <AlertDialogAction variant="destructive" onclick={confirmDelete()}>
                                "Delete permanently"
                            </AlertDialogAction>
                        </AlertDialogFooter>
                    </AlertDialogContent>
                </AlertDialog>
                <script>r##"
                    const start = @{Json(&start)};
                    const [tab, setTab] = pp.state('files');
                    const [listing, setListing] = pp.state(start.listing);
                    const [usage, setUsage] = pp.state(start.usage);
                    const [recordings, setRecordings] = pp.state(@{Json(&recordings)});
                    const [query, setQuery] = pp.state('');
                    const [sort, setSort] = pp.state({key: 'modified', descending: true});
                    const [selected, setSelected] = pp.state([]);
                    // The Recordings tab keeps its own selection. A recording
                    // still running or uploading cannot be picked, as it
                    // cannot be deleted.
                    const [pickedRecordings, setPickedRecordings] = pp.state([]);
                    function canDeleteRecording(item) { return item.state !== 'recording' && item.state !== 'uploading'; }
                    const deletableRecordings = recordings.filter(canDeleteRecording);
                    const [busy, setBusy] = pp.state(false);
                    const [uploading, setUploading] = pp.state(false);
                    const [dragging, setDragging] = pp.state(false);
                    const [error, setError] = pp.state('');
                    const [notice, setNotice] = pp.state('');
                    // Which dialog is open: 'new-folder', 'rename-file',
                    // 'rename-folder' or 'move'; empty when none is.
                    const [dialog, setDialog] = pp.state('');
                    const [target, setTarget] = pp.state('');
                    const [draftName, setDraftName] = pp.state('');
                    const [moving, setMoving] = pp.state([]);
                    const [places, setPlaces] = pp.state([]);
                    const [destination, setDestination] = pp.state('');
                    const [deleting, setDeleting] = pp.state(null);
                    const picker = pp.ref(null);
                    const nameOpener = pp.ref(null);
                    const nameCloser = pp.ref(null);
                    const moveOpener = pp.ref(null);
                    const moveCloser = pp.ref(null);
                    const deleteOpener = pp.ref(null);

                    function matches(name) { return name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()); }
                    function compare(a, b, at) {
                        let order = 0;
                        if (sort.key === 'name') order = a.name.localeCompare(b.name, undefined, {numeric: true, sensitivity: 'base'});
                        else if (sort.key === 'size') order = a.bytes - b.bytes;
                        else order = at(a) - at(b);
                        return sort.descending ? -order : order;
                    }
                    const shownFolders = listing.folders.filter(row => matches(row.name)).sort((a, b) => compare(a, b, row => row.modified_at));
                    const shownFiles = listing.files.filter(row => matches(row.name)).sort((a, b) => compare(a, b, row => row.uploaded_at));

                    function size(bytes) {
                        if (!bytes) return '0 B';
                        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
                        const step = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
                        const value = bytes / Math.pow(1024, step);
                        return (step === 0 ? value : value.toFixed(value < 10 ? 1 : 0)) + ' ' + units[step];
                    }
                    function duration(seconds) {
                        const total = Math.max(0, seconds || 0);
                        const hours = Math.floor(total / 3600), minutes = Math.floor(total % 3600 / 60), rest = total % 60;
                        const pad = value => String(value).padStart(2, '0');
                        return hours ? `${hours}:${pad(minutes)}:${pad(rest)}` : `${minutes}:${pad(rest)}`;
                    }
                    function kind(type, name) {
                        const extension = (name.split('.').pop() || '').toUpperCase();
                        if (type.startsWith('image/')) return 'Image';
                        if (type.startsWith('video/')) return 'Video';
                        if (type.startsWith('audio/')) return 'Audio';
                        if (type === 'application/pdf') return 'PDF';
                        if (type.startsWith('text/')) return 'Text';
                        return name.includes('.') && extension.length <= 5 ? extension + ' file' : 'File';
                    }
                    function stateLabel(state) {
                        return {stored: 'Ready', failed: 'Upload failed', recording: 'Recording', uploading: 'Uploading'}[state] || state;
                    }
                    function stateClass(state) {
                        const base = 'inline-flex rounded-md border px-2 py-0.5 text-xs font-medium ';
                        if (state === 'stored') return base + 'border-online/40 bg-online/10 text-online';
                        if (state === 'failed') return base + 'border-busy/40 bg-busy/10 text-danger-ink';
                        return base + 'border-edge text-muted-foreground';
                    }
                    function usageText() {
                        return usage.files === 1 ? `1 file · ${size(usage.bytes)} used` : `${usage.files} files · ${size(usage.bytes)} used`;
                    }
                    function arrow(key) { return sort.key !== key ? '' : sort.descending ? ' ↓' : ' ↑'; }
                    function sortBy(key) {
                        setSort(previous => previous.key === key ? {key, descending: !previous.descending} : {key, descending: key !== 'name'});
                    }
                    function failure(e, fallback) { return e.errors?.name?.[0] || e.errors?.ids?.[0] || e.message || fallback; }
                    function say(text) { setError(''); setNotice(text); }

                    function showTab(next) { setTab(next); setError(''); setNotice(''); setSelected([]); setPickedRecordings([]); }
                    const refresh = pp.event(async function refresh(folder) {
                        const next = await pp.rpc('files_list', {folder: folder === undefined ? listing.folder : folder || null});
                        setListing(next.listing); setUsage(next.usage);
                        setSelected(previous => previous.filter(id => next.listing.files.some(file => file.id === id)));
                    });
                    const openFolder = pp.event(async function openFolder(id) {
                        if (busy) return;
                        setBusy(true); setError(''); setNotice(''); setQuery(''); setSelected([]);
                        try { await refresh(id); }
                        catch (e) { setError(failure(e, 'The folder could not be opened.')); }
                        finally { setBusy(false); }
                    });

                    // Uploading: from the button's file picker, or dropped on the page.
                    function openPicker() { picker.current?.click(); }
                    const upload = pp.event(async function upload(files) {
                        if (!files.length || busy) return;
                        setBusy(true); setUploading(true); setError(''); setNotice('');
                        const failed = [];
                        let done = 0;
                        for (const file of files) {
                            try { await pp.rpc('files_upload', {file, folder: listing.folder || null}); done += 1; }
                            catch (e) { failed.push(file.name + ': ' + (e.errors?.file?.[0] || e.message || 'not uploaded')); }
                        }
                        try { await refresh(); } catch (e) {}
                        setBusy(false); setUploading(false);
                        if (done) setNotice(done === 1 ? '1 file uploaded.' : done + ' files uploaded.');
                        if (failed.length) setError(failed.join(' '));
                    });
                    function uploadPicked(event) {
                        const files = Array.from(event.currentTarget.files || []);
                        event.currentTarget.value = '';
                        upload(files);
                    }
                    function allowDrop(event) {
                        if (tab !== 'files' || !event.dataTransfer?.types?.includes('Files')) return;
                        event.preventDefault(); setDragging(true);
                    }
                    function dropFiles(event) {
                        setDragging(false);
                        if (tab !== 'files' || !event.dataTransfer?.files?.length) return;
                        event.preventDefault();
                        upload(Array.from(event.dataTransfer.files));
                    }

                    // Selection, for moving or deleting several files at once.
                    const toggleSelected = pp.event(function toggleSelected(id) {
                        setSelected(previous => previous.includes(id) ? previous.filter(other => other !== id) : [...previous, id]);
                    });
                    const selectAll = pp.event(function selectAll(on) { setSelected(on ? shownFiles.map(file => file.id) : []); });

                    // The name dialog: a new folder, or a file or folder renamed.
                    function openName(mode, id, name) {
                        setDialog(mode); setTarget(id); setDraftName(name); setError(''); setNotice('');
                        nameOpener.current?.click();
                    }
                    function openNewFolder() { openName('new-folder', '', ''); }
                    const askRenameFile = pp.event(function askRenameFile(id) {
                        const file = listing.files.find(row => row.id === id);
                        if (file) openName('rename-file', id, file.name);
                    });
                    const askRenameFolder = pp.event(function askRenameFolder(id) {
                        const folder = listing.folders.find(row => row.id === id);
                        if (folder) openName('rename-folder', id, folder.name);
                    });
                    function nameTitle() {
                        return dialog === 'new-folder' ? 'New folder' : dialog === 'rename-folder' ? 'Rename folder' : 'Rename file';
                    }
                    function closeDialog() { setDialog(''); setError(''); }
                    async function saveName(event) {
                        event.preventDefault(); if (busy) return;
                        const values = Object.fromEntries(new FormData(event.currentTarget));
                        setBusy(true); setError('');
                        try {
                            if (dialog === 'new-folder') await pp.rpc('files_new_folder', {parent: listing.folder || null, name: values.name});
                            else if (dialog === 'rename-folder') await pp.rpc('files_rename_folder', {id: target, name: values.name});
                            else await pp.rpc('files_rename', {id: target, name: values.name});
                            const done = dialog === 'new-folder' ? 'Folder created.' : 'Renamed.';
                            await refresh();
                            nameCloser.current?.click(); say(done);
                        } catch (e) { setError(failure(e, 'The name could not be saved.')); }
                        finally { setBusy(false); }
                    }

                    // Moving files between folders.
                    const askMove = pp.event(async function askMove(ids) {
                        if (busy || !ids.length) return;
                        setError(''); setNotice('');
                        try { setPlaces(await pp.rpc('files_places', {})); }
                        catch (e) { setError(failure(e, 'Your folders could not be loaded.')); return; }
                        setMoving(ids); setDestination(listing.folder || ''); setDialog('move');
                        moveOpener.current?.click();
                    });
                    async function saveMove(event) {
                        event.preventDefault(); if (busy) return;
                        const values = Object.fromEntries(new FormData(event.currentTarget));
                        setBusy(true); setError('');
                        try {
                            await pp.rpc('files_move', {ids: moving, folder: values.folder || null});
                            const count = moving.length;
                            setSelected([]); await refresh();
                            moveCloser.current?.click(); say(count === 1 ? '1 file moved.' : count + ' files moved.');
                        } catch (e) { setError(failure(e, 'The files could not be moved.')); }
                        finally { setBusy(false); }
                    }

                    // Deleting, always after a confirmation.
                    function confirmAbout(next) { setError(''); setNotice(''); setDeleting(next); deleteOpener.current?.click(); }
                    const askDeleteFiles = pp.event(function askDeleteFiles(ids) {
                        if (!ids.length) return;
                        const one = ids.length === 1 ? listing.files.find(row => row.id === ids[0]) : null;
                        confirmAbout({
                            kind: 'files', ids,
                            title: one ? 'Delete file?' : 'Delete ' + ids.length + ' files?',
                            detail: (one ? '"' + one.name + '"' : 'These files') + ' will be removed from your files and storage. Copies already shared to a community stay in that community. This cannot be undone.',
                        });
                    });
                    const askDeleteSelected = pp.event(function askDeleteSelected() { askDeleteFiles(selected); });
                    const askDeleteFolder = pp.event(function askDeleteFolder(id) {
                        const folder = listing.folders.find(row => row.id === id);
                        if (!folder) return;
                        confirmAbout({
                            kind: 'folder', id,
                            title: 'Delete folder?',
                            detail: '"' + folder.name + '" and everything in it (' + (folder.items === 1 ? '1 item' : folder.items + ' items') + ', ' + size(folder.bytes) + ') will be deleted. This cannot be undone.',
                        });
                    });
                    const askDeleteRecording = pp.event(function askDeleteRecording(id) {
                        const item = recordings.find(row => row.id === id);
                        if (!item) return;
                        confirmAbout({
                            kind: 'recording', id,
                            title: 'Delete recording?',
                            detail: 'The recording of "' + item.meeting + '" from ' + item.started + ' will be removed from storage for everyone who could download it. This cannot be undone.',
                        });
                    });
                    const toggleRecording = pp.event(function toggleRecording(id) {
                        setPickedRecordings(previous => previous.includes(id) ? previous.filter(other => other !== id) : [...previous, id]);
                    });
                    const selectAllRecordings = pp.event(function selectAllRecordings(on) {
                        setPickedRecordings(on ? deletableRecordings.map(item => item.id) : []);
                    });
                    const askDeleteRecordings = pp.event(function askDeleteRecordings() {
                        const ids = pickedRecordings.filter(id => recordings.some(item => item.id === id && canDeleteRecording(item)));
                        if (!ids.length) return;
                        if (ids.length === 1) { askDeleteRecording(ids[0]); return; }
                        confirmAbout({
                            kind: 'recordings', ids,
                            title: 'Delete ' + ids.length + ' recordings?',
                            detail: 'These ' + ids.length + ' recordings will be removed from storage for everyone who could download them. This cannot be undone.',
                        });
                    });
                    async function confirmDelete() {
                        if (busy || !deleting) return;
                        const job = deleting;
                        setBusy(true); setError(''); setNotice('');
                        try {
                            if (job.kind === 'recording') {
                                await pp.rpc('files_delete_recording', {id: job.id});
                                setRecordings(await pp.rpc('files_recordings', {}));
                                setPickedRecordings(previous => previous.filter(id => id !== job.id));
                                say('Recording deleted.');
                            } else if (job.kind === 'recordings') {
                                const count = await pp.rpc('files_delete_recordings', {ids: job.ids});
                                setRecordings(await pp.rpc('files_recordings', {}));
                                setPickedRecordings([]);
                                say(count === 1 ? '1 recording deleted.' : count + ' recordings deleted.');
                            } else if (job.kind === 'folder') {
                                await pp.rpc('files_delete_folder', {id: job.id});
                                await refresh(); say('Folder deleted.');
                            } else {
                                const count = await pp.rpc('files_delete', {ids: job.ids});
                                await refresh(); say(count === 1 ? '1 file deleted.' : count + ' files deleted.');
                            }
                        } catch (e) { setError(failure(e, 'Nothing was deleted.')); }
                        finally { setDeleting(null); setBusy(false); }
                    }
                "##</script>
            </main>
        </AppShell>
    })
}

#[rpc(auth)]
pub async fn files_list(folder: Option<String>) -> Result<Drive> {
    drive("files".into(), folder).await
}

#[rpc(auth)]
pub async fn files_upload(file: RpcFile, folder: Option<String>) -> Result<Attachment> {
    chat_files::store(file, folder).await
}

#[rpc(auth)]
pub async fn files_new_folder(parent: Option<String>, name: String) -> Result<Folder> {
    chat_files::create_folder(parent, name).await
}

#[rpc(auth)]
pub async fn files_rename(id: String, name: String) -> Result<Attachment> {
    chat_files::rename_file(id, name).await
}

#[rpc(auth)]
pub async fn files_rename_folder(id: String, name: String) -> Result<bool> {
    chat_files::rename_folder(id, name).await
}

#[rpc(auth)]
pub async fn files_places() -> Result<Vec<Place>> {
    chat_files::places().await
}

#[rpc(auth)]
pub async fn files_move(ids: Vec<String>, folder: Option<String>) -> Result<bool> {
    chat_files::move_files(ids, folder).await
}

#[rpc(auth)]
pub async fn files_delete(ids: Vec<String>) -> Result<u64> {
    chat_files::delete_files(ids).await
}

#[rpc(auth)]
pub async fn files_delete_folder(id: String) -> Result<u64> {
    chat_files::delete_folder(id).await
}

#[rpc(auth)]
pub async fn files_recordings() -> Result<Vec<Mine>> {
    recording::mine().await
}

#[rpc(auth)]
pub async fn files_delete_recording(id: String) -> Result<bool> {
    recording::remove_own(&id).await
}

/// Delete the recordings selected on the Recordings tab.
#[rpc(auth)]
pub async fn files_delete_recordings(ids: Vec<String>) -> Result<u64> {
    recording::remove_own_many(ids).await
}
