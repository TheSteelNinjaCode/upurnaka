//! Communities: the invitation to start one, and the ones already started.

use crate::communities::{self, Community};
use axum::extract::Query;
use serde::Deserialize;

use crate::components::app_shell::AppShell;
use crate::components::aria::{hidden, submit};
use crate::components::channel_emoji_picker::ChannelEmojiPicker;
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger,
};
use crate::components::rahti_ui::field::{Field, FieldLabel};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::textarea::Textarea;
use crate::rahti::{Html, Json, Metadata, Result, RpcFile, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Communities")
        .description("Bring your community together in one place.")
}

/// A template chip under "Create with a template".
const TEMPLATE: &str = "rounded-full text-[13px] font-semibold";

#[derive(Default, Deserialize)]
pub struct CommunityQuery {
    community: Option<String>,
    join: Option<String>,
}

pub async fn page(Query(query): Query<CommunityQuery>) -> Result<Html> {
    let saved = communities::list().await?;
    let invitations = communities::invitations().await?;
    let join_preview = match query.join.as_deref() {
        Some(token) => communities::preview(token).await?,
        None => None,
    };
    let join_token = query.join.unwrap_or_default();
    let selected = query
        .community
        .filter(|id| saved.iter().any(|c| &c.id == id))
        .or_else(|| saved.first().map(|c| c.id.clone()))
        .unwrap_or_default();
    let cancel_link = button_variants(
        ButtonVariant::Link,
        ButtonSize::Default,
        "text-[13px] text-link",
    );
    Ok(html! {
        <AppShell active="communities">
            <main
                class="ambient-strong scroll-thin relative min-h-0 flex-1 overflow-y-auto rounded-tl-xl border-l border-t border-line"
            >
                <section
                    class="community-invites"
                    hidden={!joinPreview && pendingInvites.length === 0}
                >
                    <div hidden={!joinPreview}>
                        <h2>{joinPreview?.avatar}" "{joinPreview?.name}</h2>
                        <p>{joinPreview?.description}</p>
                        <Button
                            onclick={acceptInvitation(joinPreview.id, joinToken)}
                            disabled={working}
                        >
                            "Join community"
                        </Button>
                    </div>
                    <template pp-for="invitation in pendingInvites">
                        <div key={invitation.id}>
                            <h2>"You're invited to "{invitation.name}</h2>
                            <Button
                                onclick={acceptInvitation(invitation.id, null)}
                                disabled={working}
                            >
                                "Accept invitation"
                            </Button>
                        </div>
                    </template>
                </section>
                <p class="community-status" role="status" hidden={!notice}>{notice}</p>
                <div
                    class="mx-auto grid w-full max-w-[1280px] grid-cols-2 items-center gap-12 px-16 py-16"
                    hidden={communities.length > 0}
                >
                    <div>
                        <h1 class="text-[30px] font-semibold leading-tight tracking-tight text-ink">
                            "Build your community"
                        </h1>
                        <p
                            class="mt-5 max-w-[320px] text-[14px] leading-relaxed text-muted-foreground"
                        >
                            "Bring your community together in one place to plan events, stay organized, and get more done."
                        </p>
                        <Button
                            size="lg"
                            class="mt-8 text-[13px] font-semibold"
                            onclick={openCreate("")}
                        >
                            "Create your own"
                        </Button>
                        <h2 class="mt-12 text-[17px] font-semibold text-ink">
                            "Create with a template"
                        </h2>
                        <div class="mt-5 flex items-center gap-3">
                            <Button
                                variant="outline"
                                size="sm"
                                class=@{TEMPLATE}
                                onclick={openCreate("School")}
                            >
                                <span class="leading-none">"🎓"</span>
                                "School"
                            </Button>
                            <Button
                                variant="outline"
                                size="sm"
                                class=@{TEMPLATE}
                                onclick={openCreate("Business")}
                            >
                                <span class="leading-none">"🏪"</span>
                                "Business"
                            </Button>
                            <Button
                                variant="outline"
                                size="sm"
                                class=@{TEMPLATE}
                                onclick={openCreate("Life")}
                            >
                                <span class="leading-none">"🏅"</span>
                                "Life"
                            </Button>
                        </div>
                    </div>
                    <div class="relative h-[460px]">
                        <div
                            class="absolute inset-x-6 bottom-6 top-6 rounded-2xl bg-linear-to-br from-[#4b3f9e] to-[#2a2560]"
                        >
                        </div>
                        <div
                            class="absolute inset-x-12 bottom-14 top-14 rounded-xl bg-white/95 p-5 shadow-2xl shadow-black/50"
                        >
                            <div
                                class="grid h-full grid-cols-6 grid-rows-6 gap-1 rounded-lg bg-[#f1f1f6] p-2"
                            >
                                <div
                                    class="col-span-3 col-start-2 row-span-2 row-start-4 rounded bg-[#a7e3b0]"
                                >
                                </div>
                                <div
                                    class="col-span-2 col-start-5 row-span-3 row-start-3 rounded bg-[#a9c9f5]"
                                >
                                </div>
                                <div
                                    class="col-span-2 col-start-1 row-span-1 row-start-6 rounded bg-[#d9d2f7]"
                                >
                                </div>
                            </div>
                        </div>
                        <div
                            class="absolute left-0 top-14 w-[330px] rounded-xl bg-linear-to-br from-[#c9bcff] to-[#9f8ff2] p-5 shadow-2xl shadow-black/40"
                        >
                            <p class="text-[11px] font-semibold text-[#2d2258]">
                                "Non-profits, mentoring, fundraising"
                            </p>
                            <p class="mt-2 text-[26px] font-semibold leading-tight text-[#1d1540]">
                                "Time to rally the volunteers"
                            </p>
                        </div>
                        <div
                            class="absolute right-2 top-28 flex w-[290px] items-center gap-3 rounded-xl bg-[#2a2550]/95 px-4 py-3 shadow-2xl shadow-black/50"
                        >
                            <span
                                class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-white/15 text-ink"
                            >
                                <Icon name="calendar-day" class="h-[18px] w-[18px]" />
                            </span>
                            <span class="min-w-0 flex-1">
                                <span class="block truncate text-[13px] font-semibold text-ink">
                                    "Onboard volunteers"
                                </span>
                                <span class="block truncate text-[11px] text-muted-foreground">
                                    "Monday 10:00 AM"
                                </span>
                            </span>
                            <span
                                class="shrink-0 rounded bg-brand px-3 py-1 text-[11px] font-semibold text-white"
                            >
                                "Join"
                            </span>
                        </div>
                    </div>
                </div>
                <div class="community-workspace" hidden={communities.length === 0}>
                    <aside class="community-sidebar" aria-label="Your communities">
                        <header class="community-sidebar-heading">
                            <h1>"Communities"</h1>
                            <Button
                                variant="ghost"
                                size="icon"
                                attrs=@{Attrs::new().set("aria-label", "Create community").set("title", "Create your own")}
                                onclick={openCreate("")}
                            >
                                <Icon name="plus" class="size-5" />
                            </Button>
                        </header>
                        <nav aria-label="Communities">
                            <template pp-for="community in communities">
                                <div key={community.id} class="community-nav-group">
                                    <div
                                        class="community-nav-row"
                                        data-selected={selectedCommunity === community.id}
                                    >
                                        <button
                                            class="community-nav-item"
                                            aria-current={selectedCommunity === community.id && channelId === "" ? "page" : null}
                                            onclick={selectCommunity(community.id)}
                                            pp-ref={element => communityElements.current[community.id] = element}
                                        >
                                            <span class="community-small-avatar">
                                                <img
                                                    src={community.picture || "/images/upurnaka-logo.png"}
                                                    alt=""
                                                    hidden={!community.picture}
                                                />
                                                <span hidden={!!community.picture}>
                                                    {community.avatar}
                                                </span>
                                            </span>
                                            <strong>{community.name}</strong>
                                        </button>
                                        <button
                                            class="community-more"
                                            aria-label={"More options for " + community.name}
                                            aria-haspopup="menu"
                                            aria-controls="community-actions-menu"
                                            popovertarget="community-actions-menu"
                                            onclick={prepareMenu(community.id, event)}
                                            onkeydown={openMenuKey(community.id, event)}
                                        >
                                            <Icon name="more" class="size-5" />
                                        </button>
                                    </div>
                                    <div
                                        class="community-channel-list"
                                        hidden={selectedCommunity !== community.id}
                                    >
                                        <template pp-for="channel in community.channels">
                                            <button
                                                key={channel.id}
                                                class="community-channel"
                                                aria-current={channelId === channel.id ? "page" : null}
                                                onclick={selectChannel(channel.id)}
                                            >
                                                <img
                                                    class="channel-nav-emoji"
                                                    src={channelEmojiImage(channel.icon) || "/favicon.ico"}
                                                    alt=""
                                                    hidden={!channelEmojiImage(channel.icon)}
                                                />
                                                <span
                                                    aria-hidden="true"
                                                    class={channelEmojiImage(channel.icon) ? "sr-only" : ""}
                                                >
                                                    {channel.icon}
                                                </span>
                                                " "
                                                {channel.name}
                                            </button>
                                        </template>
                                    </div>
                                </div>
                            </template>
                        </nav>
                        <button
                            class="community-add-channel"
                            onclick={openChannelDialog()}
                            hidden={!activeCommunity?.is_owner}
                        >
                            <Icon name="plus" class="size-5" />
                            "Add channel"
                        </button>
                    </aside>
                    <section class="community-detail" aria-label="Selected community">
                        <div class="community-cover" data-cover={activeCommunity?.cover || "mint"}>
                            <div class="community-cover-art" aria-hidden="true"></div>
                            <button
                                class="community-cover-edit"
                                aria-label="Change community cover"
                                title="Change community cover"
                                onclick={setCoverPicker(!coverPicker)}
                                hidden={!activeCommunity?.is_owner}
                            >
                                <Icon name="edit" class="size-5" />
                            </button>
                            <div class="community-cover-picker" hidden={!coverPicker}>
                                <button onclick={changeCover("mint")} disabled={working}>
                                    "Mint"
                                </button>
                                <button onclick={changeCover("lavender")} disabled={working}>
                                    "Lavender"
                                </button>
                                <button onclick={changeCover("sunset")} disabled={working}>
                                    "Sunset"
                                </button>
                            </div>
                        </div>
                        <header class="community-detail-header">
                            <div class="community-identity-title">
                                <span class="community-small-avatar">
                                    <img
                                        class="community-saved-picture"
                                        src={activeCommunity?.picture || "/images/upurnaka-logo.png"}
                                        alt=""
                                        hidden={!activeCommunity?.picture || !!activeChannel}
                                    />
                                    <img
                                        class="channel-header-emoji"
                                        src={channelEmojiImage(activeChannel?.icon) || "/favicon.ico"}
                                        alt=""
                                        hidden={!activeChannel || !channelEmojiImage(activeChannel.icon)}
                                    />
                                    <span
                                        hidden={activeChannel ? !!channelEmojiImage(activeChannel.icon) : !!activeCommunity?.picture}
                                    >
                                        {activeChannel?.icon || activeCommunity?.avatar || "👥"}
                                    </span>
                                </span>
                                <h2>
                                    {activeChannel?.name || activeCommunity?.name || "Community"}
                                </h2>
                            </div>
                            <div
                                class="community-tabs"
                                role="tablist"
                                aria-label="Community content"
                                onkeydown={handleCommunityTabKeys(event)}
                            >
                                <button
                                    role="tab"
                                    aria-selected={activeTab === "posts"}
                                    aria-controls="community-posts"
                                    id="community-tab-posts"
                                    pp-ref={postsTab}
                                    tabindex={activeTab === "posts" ? 0 : -1}
                                    onclick={setActiveTab("posts")}
                                >
                                    "Posts"
                                </button>
                                <button
                                    role="tab"
                                    aria-selected={activeTab === "files"}
                                    aria-controls="community-files"
                                    id="community-tab-files"
                                    pp-ref={filesTab}
                                    tabindex={activeTab === "files" ? 0 : -1}
                                    onclick={setActiveTab("files")}
                                >
                                    "Files"
                                </button>
                                <button
                                    role="tab"
                                    aria-selected={activeTab === "photos"}
                                    aria-controls="community-photos"
                                    id="community-tab-photos"
                                    pp-ref={photosTab}
                                    tabindex={activeTab === "photos" ? 0 : -1}
                                    onclick={setActiveTab("photos")}
                                >
                                    "Photos"
                                </button>
                            </div>
                            <div class="community-header-actions">
                                <a href="/calendar?new=1" class="community-event-link">
                                    <Icon name="calendar" class="size-4" />
                                    "Events"
                                </a>
                                <Button
                                    variant="outline"
                                    size="icon"
                                    attrs=@{Attrs::new().set("aria-label", "Community details and members")}
                                    onclick={setDetailsOpen(!detailsOpen)}
                                >
                                    <Icon name="settings" class="size-4" />
                                </Button>
                            </div>
                        </header>
                        <div class="community-details-panel" hidden={!detailsOpen}>
                            <h3>"About this community"</h3>
                            <p>
                                {activeCommunity?.description || "Bring your community together in one place."}
                            </p>
                            <h3>"Community guidelines"</h3>
                            <p>{activeCommunity?.guidelines || "Be kind and respectful."}</p>
                            <h3>{"Members (" + (activeCommunity?.member_count || 1) + ")"}</h3>
                            <Button
                                variant="outline"
                                onclick={openAction("manage", selectedCommunity)}
                            >
                                "View members"
                            </Button>
                        </div>
                        <p class="community-workspace-error" role="alert" hidden={!workspaceError}>
                            {workspaceError}
                        </p>
                        <div
                            id="community-posts"
                            aria-labelledby="community-tab-posts"
                            role="tabpanel"
                            class="community-content"
                            hidden={activeTab !== "posts"}
                        >
                            <form class="community-composer" onsubmit={publishPost(event)}>
                                <label for="community-post-body" class="community-composer-heading">
                                    <span class="community-author-avatar">
                                        @{crate::auth::current().map(|u| u.initials()).unwrap_or_default()}
                                    </span>
                                    "Post in channel"
                                </label>
                                <textarea
                                    id="community-post-body"
                                    placeholder="Share something with your community…"
                                    maxlength="5000"
                                    value={postDraft}
                                    oninput={setPostDraft(target.value)}
                                >
                                </textarea>
                                <div class="community-composer-actions">
                                    <Button
                                        attrs=@{submit()}
                                        disabled={working || !postDraft.trim()}
                                    >
                                        {working ? "Posting…" : announcement ? "Post announcement" : "Post"}
                                    </Button>
                                    <button
                                        type="button"
                                        aria-pressed={announcement}
                                        onclick={setAnnouncement(!announcement)}
                                    >
                                        "◁ Announcement"
                                    </button>
                                </div>
                            </form>
                            <section class="community-setup" hidden={activePosts.length > 0}>
                                <h2>"Set up your community"</h2>
                                <p>"Get started with these quick actions"</p>
                                <div class="community-quick-actions">
                                    <button onclick={setDetailsOpen(true)}>
                                        <span>"👥"</span>
                                        "Community details"
                                    </button>
                                    <a href="/calendar?new=1">
                                        <span>"🗓️"</span>
                                        "Create new event"
                                    </a>
                                </div>
                            </section>
                            <template pp-for="post in activePosts">
                                <article key={post.id} class="community-post">
                                    <header>
                                        <span class="community-author-avatar">
                                            {authorInitials(post.author_name)}
                                        </span>
                                        <strong>{post.author_name}</strong>
                                        <time>{new Date(post.created_at).toLocaleString()}</time>
                                    </header>
                                    <h3 hidden={!post.announcement}>"Announcement"</h3>
                                    <p>{post.body}</p>
                                </article>
                            </template>
                        </div>
                        <div
                            id="community-files"
                            aria-labelledby="community-tab-files"
                            role="tabpanel"
                            class="community-empty-tab"
                            hidden={activeTab !== "files"}
                        >
                            <Icon name="folder" class="size-12" />
                            <h3>"No files in this community yet"</h3>
                            <p>"Files shared with this community will appear here."</p>
                        </div>
                        <div
                            id="community-photos"
                            aria-labelledby="community-tab-photos"
                            role="tabpanel"
                            class="community-empty-tab"
                            hidden={activeTab !== "photos"}
                        >
                            <Icon name="image" class="size-12" />
                            <h3>
                                {activeCommunity?.picture ? "Community photo" : "No photos in this community yet"}
                            </h3>
                            <img
                                src={activeCommunity?.picture || "/images/upurnaka-logo.png"}
                                alt="Community photo"
                                hidden={!activeCommunity?.picture}
                                class="community-photo"
                            />
                        </div>
                    </section>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={channelOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={channelCloser}></span>
                    </DialogClose>
                    <DialogContent class="community-channel-dialog" hide_close=@{true}>
                        <DialogHeader>
                            <DialogTitle>{channelDialogTitle}</DialogTitle>
                        </DialogHeader>
                        <form class="community-channel-form" onsubmit={addChannel(event)}>
                            <label for="channel-name">"Channel name"</label>
                            <div class="community-channel-identity">
                                <ChannelEmojiPicker
                                    value={channelIcon}
                                    select={onChannelIconSelected}
                                />
                                <Input
                                    id="channel-name"
                                    name="name"
                                    placeholder="Give your channel a name"
                                    value={channelName}
                                    oninput={setChannelName(target.value)}
                                    disabled={working}
                                    attrs=@{Attrs::new().set("maxlength", "60").set("autocomplete", "off").set("autofocus", "").set("aria-describedby", "community-channel-error")}
                                />
                            </div>
                            <p
                                id="community-channel-error"
                                class="community-channel-error"
                                role="alert"
                                hidden={!channelError && !channelAtLimit}
                            >
                                {channelError || "This community already has 5 channels."}
                            </p>
                            <footer class="community-channel-footer">
                                <p>"Owners may add up to 5 channels per community"</p>
                                <div>
                                    <Button
                                        variant="outline"
                                        onclick={channelCloser.current?.click()}
                                        attrs=@{Attrs::new().set("type", "button")}
                                        disabled={working}
                                    >
                                        "Cancel"
                                    </Button>
                                    <Button
                                        attrs=@{submit()}
                                        disabled={working || !channelName.trim() || channelAtLimit}
                                    >
                                        {working ? "Adding…" : "Add"}
                                    </Button>
                                </div>
                            </footer>
                        </form>
                    </DialogContent>
                </Dialog>
                <div
                    id="community-actions-menu"
                    popover="auto"
                    role="menu"
                    aria-label="Community options"
                    class="community-actions-menu"
                    pp-ref={actionsMenu}
                    pp-style={menuPosition}
                    ontoggle={menuToggled(event)}
                    onkeydown={menuKey(event)}
                >
                    <button
                        role="menuitem"
                        pp-ref={shareMenuItem}
                        hidden={!menuCommunity?.is_owner}
                        onclick={openAction("share", menuId)}
                    >
                        <Icon name="link" class="size-5" />
                        "Share join link"
                    </button>
                    <button
                        role="menuitem"
                        pp-ref={inviteMenuItem}
                        hidden={!menuCommunity?.is_owner}
                        onclick={openAction("invite", menuId)}
                    >
                        <Icon name="person-add" class="size-5" />
                        "Invite members"
                    </button>
                    <button
                        role="menuitem"
                        pp-ref={notificationMenuItem}
                        onclick={openAction("notifications", menuId)}
                    >
                        <Icon name="bell" class="size-5" />
                        "Notifications"
                    </button>
                    <hr />
                    <button
                        role="menuitem"
                        pp-ref={manageMenuItem}
                        onclick={openAction("manage", menuId)}
                    >
                        <Icon name="settings" class="size-5" />
                        "Manage community"
                    </button>
                    <button
                        role="menuitem"
                        pp-ref={editMenuItem}
                        hidden={!menuCommunity?.is_owner}
                        onclick={openEdit(menuId)}
                    >
                        <Icon name="edit" class="size-5" />
                        "Edit community"
                    </button>
                    <hr />
                    <button
                        role="menuitem"
                        pp-ref={reportMenuItem}
                        onclick={openAction("report", menuId)}
                    >
                        <Icon name="warning" class="size-5" />
                        "Report a concern"
                    </button>
                    <hr />
                    <button
                        role="menuitem"
                        pp-ref={leaveMenuItem}
                        onclick={openAction("leave", menuId)}
                    >
                        <Icon name="sign-out" class="size-5" />
                        "Leave community"
                    </button>
                    <button
                        role="menuitem"
                        pp-ref={deleteMenuItem}
                        hidden={!menuCommunity?.is_owner}
                        onclick={openAction("delete", menuId)}
                    >
                        <Icon name="trash" class="size-5" />
                        "Delete community"
                    </button>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={actionOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={actionCloser}></span>
                    </DialogClose>
                    <DialogContent class="community-action-dialog">
                        <DialogHeader>
                            <DialogTitle>{actionTitle}</DialogTitle>
                        </DialogHeader>
                        <p class="text-sm text-muted-foreground">{actionCommunity?.name}</p>
                        <section hidden={action !== "share"} class="community-action-fields">
                            <p>
                                "People with this link can sign in and join this community. Share it only with people you want to invite."
                            </p>
                            <label for="community-join-link">"Join link"</label>
                            <input
                                id="community-join-link"
                                pp-ref={joinLinkInput}
                                value={shareUrl}
                                readonly
                            />
                            <Button onclick={copyJoinLink()} disabled={working || !shareUrl}>
                                "Copy link"
                            </Button>
                        </section>
                        <form
                            hidden={action !== "invite"}
                            class="community-action-fields"
                            onsubmit={sendInvitation(event)}
                        >
                            <p>
                                "Invite someone with an Upurnaka account. They will see an invitation when they open Communities."
                            </p>
                            <label for="community-invite-email">"Email address"</label>
                            <Input
                                id="community-invite-email"
                                value={inviteEmail}
                                oninput={setInviteEmail(target.value)}
                                attrs=@{Attrs::new().set("type","email").set("maxlength","254")}
                            />
                            <Button attrs=@{submit()} disabled={working || !inviteEmail.trim()}>
                                "Send invitation"
                            </Button>
                        </form>
                        <form
                            hidden={action !== "notifications"}
                            class="community-action-fields"
                            onsubmit={saveNotifications(event)}
                        >
                            <label for="community-notification-preference">
                                "Community notifications"
                            </label>
                            <select
                                id="community-notification-preference"
                                value={notificationPreference}
                                onchange={setNotificationPreference(target.value)}
                            >
                                <option value="all">"All activity"</option>
                                <option value="mentions">"Mentions only"</option>
                                <option value="off">"Off"</option>
                            </select>
                            <Button attrs=@{submit()} disabled={working}>"Save"</Button>
                        </form>
                        <section hidden={action !== "manage"} class="community-action-fields">
                            <h3>"Members and invitations"</h3>
                            <template pp-for="member in members">
                                <div key={member.id} class="community-member-row">
                                    <div>
                                        <strong>{member.name}</strong>
                                        <p>
                                            {member.is_owner ? "Owner" : member.state === "invited" ? "Invitation pending" : "Member"}
                                        </p>
                                    </div>
                                    <div hidden={!actionCommunity?.is_owner || member.is_owner}>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            hidden={member.state !== "joined"}
                                            onclick={setTransferTarget(member.id)}
                                            disabled={working}
                                        >
                                            "Make owner"
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            onclick={removeMember(member.id)}
                                            disabled={working}
                                        >
                                            {member.state === "invited" ? "Cancel invitation" : "Remove"}
                                        </Button>
                                    </div>
                                </div>
                            </template>
                            <div hidden={!transferTarget} class="community-action-fields">
                                <p>
                                    "Transfer ownership to this member? You will become a member and lose owner controls."
                                </p>
                                <Button onclick={transferOwnership()} disabled={working}>
                                    "Confirm transfer"
                                </Button>
                                <Button variant="outline" onclick={setTransferTarget("")}>
                                    "Cancel"
                                </Button>
                            </div>
                            <Button
                                variant="outline"
                                hidden={!actionCommunity?.is_owner}
                                onclick={resetJoinLink()}
                                disabled={working}
                            >
                                "Reset join link"
                            </Button>
                            <p hidden={!actionCommunity?.is_owner}>
                                "Resetting the link invalidates older join links."
                            </p>
                            @{if crate::auth::current().is_some_and(|u| u.role == crate::auth::Role::Admin) { html!{<a href="/communities/reports" class="text-link">"Review community reports"</a>} } else { Html::empty() }}
                        </section>
                        <form
                            hidden={action !== "report"}
                            class="community-action-fields"
                            onsubmit={submitReport(event)}
                        >
                            <p>
                                "Describe your concern. Your report will be available to workspace administrators."
                            </p>
                            <label for="community-report-details">"Your concern"</label>
                            <textarea
                                id="community-report-details"
                                maxlength="2000"
                                value={reportDetails}
                                oninput={setReportDetails(target.value)}
                            >
                            </textarea>
                            <Button
                                attrs=@{submit()}
                                disabled={working || reportDetails.trim().length < 10 || !!reportReceipt}
                            >
                                "Submit report"
                            </Button>
                        </form>
                        <section hidden={action !== "leave"} class="community-action-fields">
                            <p hidden={!actionCommunity?.is_owner}>
                                "You are the community owner. Make another joined member the owner in Manage community before leaving."
                            </p>
                            <p hidden={actionCommunity?.is_owner}>
                                "Leave this community? It will disappear from your list and you will lose access to its posts and channels."
                            </p>
                            <Button
                                onclick={leaveCommunity()}
                                disabled={working || actionCommunity?.is_owner}
                            >
                                "Leave community"
                            </Button>
                        </section>
                        <section hidden={action !== "delete"} class="community-action-fields">
                            <p>
                                "Delete this community? It will be removed from every member's list. Posts, channels, and join links will no longer be accessible."
                            </p>
                            <Button
                                variant="destructive"
                                onclick={deleteCommunity()}
                                disabled={working}
                            >
                                "Delete community"
                            </Button>
                            <Button variant="outline" onclick={actionCloser.current?.click()}>
                                "Cancel"
                            </Button>
                        </section>
                        <p role="alert" class="text-sm text-danger-ink" hidden={!actionError}>
                            {actionError}
                        </p>
                        <p role="status" class="community-action-notice" hidden={!actionNotice}>
                            {actionNotice}
                        </p>
                    </DialogContent>
                </Dialog>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={createOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={createCloser}></span>
                    </DialogClose>
                    <DialogContent class="community-create-dialog" hide_close=@{true}>
                        <div class="flex items-start justify-between gap-6">
                            <DialogHeader class="text-left!">
                                <DialogTitle class="text-[24px] font-bold leading-tight">
                                    {editingId ? "Edit community" : step === 2 ? "Choose a category" : template ? "Create your " + template.toLowerCase() + " community" : "Create your community"}
                                </DialogTitle>
                            </DialogHeader>
                            <button
                                type="button"
                                class="community-create-close"
                                aria-label="Close community creation"
                                onclick={createCloser.current?.click()}
                            >
                                <Icon name="close" class="size-6" />
                            </button>
                        </div>
                        <form
                            class="community-create-form"
                            onsubmit={goToCategory(event)}
                            novalidate
                            hidden={step !== 1}
                        >
                            <div class="community-identity">
                                <button
                                    type="button"
                                    class="community-avatar"
                                    aria-label="Edit community image"
                                    aria-describedby="community-image-help"
                                    disabled={saving}
                                    onclick={openImagePicker()}
                                >
                                    <img
                                        src={picturePreview || "/images/upurnaka-logo.png"}
                                        alt=""
                                        class="community-avatar-photo"
                                        hidden={!picturePreview}
                                        onerror={rejectPicture()}
                                    />
                                    <span
                                        class="community-avatar-emoji"
                                        aria-hidden="true"
                                        hidden={!!picturePreview}
                                    >
                                        {avatar}
                                    </span>
                                    <span class="community-avatar-edit">"Edit"</span>
                                </button>
                                <input
                                    id="community-picture"
                                    type="file"
                                    accept="image/png,image/jpeg"
                                    pp-ref={pictureInput}
                                    onchange={choosePicture(event)}
                                    hidden
                                />
                                <Field class="min-w-0 flex-1 gap-1">
                                    <FieldLabel
                                        r#for="community-name"
                                        class="text-[15px] font-normal"
                                    >
                                        "Community name"
                                    </FieldLabel>
                                    <Input
                                        id="community-name"
                                        name="name"
                                        placeholder="Enter a community name"
                                        class="community-name-input"
                                        attrs=@{Attrs::new().set("autocomplete", "off").set("autofocus", "").set("maxlength", "80")}
                                        value={name}
                                        oninput={setName(target.value)}
                                        invalid={!!fieldErrors.name}
                                    />
                                    <p
                                        class="text-xs text-busy"
                                        role="alert"
                                        hidden={!fieldErrors.name}
                                    >
                                        {fieldErrors.name?.join(" ")}
                                    </p>
                                </Field>
                            </div>
                            <p id="community-image-help" class="mt-2 text-xs text-muted-foreground">
                                "PNG or JPEG, up to 2 MB."
                            </p>
                            <button
                                type="button"
                                class="text-sm text-link"
                                hidden={!editingId || (!picturePreview && !editingPicture)}
                                onclick={clearCommunityPicture()}
                            >
                                "Remove community image"
                            </button>
                            <p
                                class="mt-2 text-sm text-busy"
                                role="alert"
                                hidden={!pictureError && !fieldErrors.picture}
                            >
                                {pictureError || fieldErrors.picture?.join(" ")}
                            </p>
                            <Field class="mt-8 gap-2">
                                <FieldLabel
                                    r#for="community-description"
                                    class="text-[15px] font-normal"
                                >
                                    "Description (optional)"
                                </FieldLabel>
                                <Textarea
                                    id="community-description"
                                    name="description"
                                    class="community-description"
                                    placeholder="Write a short description about your community so people know what it's about."
                                    attrs=@{Attrs::new().set("maxlength", "1000")}
                                    value={description}
                                    oninput={setDescription(target.value)}
                                />
                                <p
                                    class="text-xs text-busy"
                                    role="alert"
                                    hidden={!fieldErrors.description}
                                >
                                    {fieldErrors.description?.join(" ")}
                                </p>
                            </Field>
                            <div class="mt-8">
                                <div class="mb-2 flex items-center justify-between gap-4">
                                    <label for="community-guidelines" class="text-[15px]">
                                        "Community guidelines"
                                    </label>
                                    <button
                                        type="button"
                                        class="community-guidelines-edit"
                                        aria-label={editingGuidelines ? "Done editing community guidelines" : "Edit community guidelines"}
                                        aria-expanded={editingGuidelines}
                                        onclick={toggleGuidelines()}
                                    >
                                        <span hidden={editingGuidelines}>
                                            <Icon name="edit" class="size-5" />
                                        </span>
                                        <span hidden={!editingGuidelines}>"Done"</span>
                                    </button>
                                </div>
                                <p class="community-guidelines-preview" hidden={editingGuidelines}>
                                    {guidelines}
                                </p>
                                <textarea
                                    id="community-guidelines"
                                    name="guidelines"
                                    class="community-description w-full focus-visible:outline-2 focus-visible:outline-brand-ink"
                                    maxlength="2000"
                                    pp-ref={guidelinesInput}
                                    value={guidelines}
                                    oninput={setGuidelines(target.value)}
                                    hidden={!editingGuidelines}
                                >
                                </textarea>
                                <p
                                    class="text-xs text-busy"
                                    role="alert"
                                    hidden={!fieldErrors.guidelines}
                                >
                                    {fieldErrors.guidelines?.join(" ")}
                                </p>
                            </div>
                            <p class="mt-4 text-sm text-busy" role="alert" hidden={!error}>
                                {error}
                            </p>
                            <DialogFooter class="community-create-footer">
                                <Button
                                    class="min-w-32 text-[15px] font-semibold"
                                    attrs=@{submit()}
                                    disabled={saving || name.trim().length < 2 || (picture && !picturePreview)}
                                >
                                    {saving ? "Saving…" : editingId ? "Save" : template ? "Create" : "Next"}
                                </Button>
                            </DialogFooter>
                        </form>
                        <div hidden={step !== 2}>
                            <div class="flex flex-col gap-3">
                                <template pp-for="category in CATEGORIES">
                                    <Button
                                        key={category}
                                        variant="outline"
                                        size="lg"
                                        class="h-12! w-full text-[13px] font-semibold hover:border-brand"
                                        disabled={saving}
                                        onclick={create(category)}
                                    >
                                        {category}
                                    </Button>
                                </template>
                            </div>
                            <p class="mt-4 text-center text-xs text-busy" hidden={!error}>
                                {error}
                            </p>
                            <DialogFooter class="mt-7 sm:justify-between!">
                                <Button
                                    variant="outline"
                                    class="w-24 text-[13px] font-semibold"
                                    onclick={setStep(1)}
                                >
                                    "Back"
                                </Button>
                                <DialogClose class=@{cancel_link.as_str()}>"Cancel"</DialogClose>
                            </DialogFooter>
                        </div>
                    </DialogContent>
                </Dialog>
                <style>r##"
                    .community-nav-row { display:flex;align-items:center;border-radius:8px; }
                    .community-nav-row[data-selected=true],.community-nav-row:hover { background:#292929; }
                    .community-nav-row .community-nav-item { flex:1;min-width:0;background:transparent; }
                    .community-more { display:grid;place-items:center;flex-shrink:0;width:36px;height:36px;margin-right:8px;border-radius:4px;color:#ccc; }
                    .community-more:hover,.community-more[aria-expanded=true] { background:#444; }
                    .community-actions-menu { position:fixed;margin:0;width:260px;max-width:calc(100vw - 24px);max-height:calc(100dvh - 24px);overflow:auto;border:1px solid #454545;border-radius:6px;padding:5px 0;background:#292929;color:#ddd;box-shadow:0 8px 28px #0006; }
                    .community-actions-menu:popover-open { display:block; }
                    .community-actions-menu button { display:flex;align-items:center;gap:12px;width:100%;padding:12px 16px;text-align:left;font-size:14px; }
                    .community-actions-menu button:hover { background:#3c3c3c; }
                    .community-actions-menu button:focus { outline:2px solid white;outline-offset:-4px;border-radius:5px; }
                    .community-actions-menu hr { border:0;border-top:1px solid #505050;margin:6px 0; }
                    .community-action-dialog { width:min(580px,calc(100vw - 32px));max-height:calc(100dvh - 40px);overflow:auto; }
                    .community-action-fields { display:grid;gap:14px;font-size:14px; }
                    .community-action-fields p { color:#bbb;line-height:1.6; }
                    .community-action-fields input,.community-action-fields textarea,.community-action-fields select { width:100%;border:1px solid #555;border-radius:4px;background:#202020;color:#eee;padding:10px; }
                    .community-action-fields textarea { min-height:110px; }
                    .community-action-fields h3 { font-weight:600; }
                    .community-member-row { display:flex;align-items:center;justify-content:space-between;gap:12px;border-bottom:1px solid #444;padding:12px 0; }
                    .community-member-row > div:last-child { display:flex;gap:6px;flex-wrap:wrap; }
                    .community-action-notice,.community-status { color:#c4d8bc;font-size:14px;line-height:1.5; }
                    .community-status { padding:12px 24px;background:#253323; }
                    .community-invites { padding:20px;background:#242033;display:grid;gap:16px; }
                    .community-invites h2 { font-weight:600;margin-bottom:8px; }
                    .community-invites p { margin-bottom:12px; }
                    .community-workspace { display: grid; grid-template-columns: 300px minmax(0,1fr); min-height: 100%; color: #f2f2f2; background: linear-gradient(135deg,#18121f,#241415 60%,#16141c); }
                    .community-sidebar { padding: 24px 16px; }
                    .community-sidebar-heading { display:flex;align-items:center;justify-content:space-between;margin-bottom:22px;gap:12px; }
                    .community-sidebar-heading h1 { font-size:23px;font-weight:700; }
                    .community-nav-group { margin-bottom:8px; }
                    .community-nav-item { display:flex;align-items:center;gap:12px;text-align:left;padding:14px 12px;width:100%;border-radius:8px; }
                    .community-nav-item strong { overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-nav-item:hover,.community-nav-item[aria-current=page] { background:#292929; }
                    .community-small-avatar { display:grid;place-items:center;flex-shrink:0;width:38px;height:38px;border-radius:8px;background:linear-gradient(135deg,#fff0ca,#e7c16b);font-size:24px;overflow:hidden; }
                    .community-nav-item .community-small-avatar { width:48px;height:48px;font-size:30px; }
                    .community-small-avatar img { width:100%;height:100%;object-fit:cover; }
                    .community-channel-list { padding:6px 0 6px 50px; }
                    .community-channel { display:block;padding:7px 12px;width:100%;text-align:left;font-size:14px;color:#bbb;border-radius:4px;overflow-wrap:anywhere; }
                    .community-channel[aria-current=page] { color:#b6b0ff;background:#ffffff09; }
                    .community-channel .channel-nav-emoji { display:inline-block;width:22px;height:22px;object-fit:contain;vertical-align:middle;margin-right:6px; }
                    .community-small-avatar .channel-header-emoji { padding:3px;object-fit:contain; }
                    .community-add-channel { display:flex;align-items:center;gap:12px;margin:20px 12px;color:#a6a0ed;font-size:14px; }
                    .community-channel-dialog { width:min(680px,calc(100vw - 32px));max-width:680px;max-height:calc(100dvh - 32px);overflow:auto;border:0;border-radius:10px;padding:28px;background:#292929;gap:28px; }
                    .community-channel-dialog [data-slot=dialog-title] { font-size:23px;line-height:1.4;font-weight:700; }
                    .community-channel-form { display:grid;gap:6px;font-size:14px; }
                    .community-channel-identity { display:flex;gap:12px;align-items:stretch; }
                    .community-channel-identity > [data-slot=popover] { flex-shrink:0; }
                    .community-channel-icon-trigger { display:flex;align-items:center;justify-content:center;gap:8px;height:38px;min-width:66px;padding:6px 10px;border:1px solid #aaa;border-radius:4px;background:#222;color:#eee;font-size:19px; }
                    .community-channel-identity > input { min-width:0;height:38px;flex:1;border:1px solid transparent;border-radius:4px;background:#202020;padding:8px 12px;font-size:15px;box-shadow:none; }
                    .community-channel-identity > input:focus-visible { outline:2px solid #a6a0ff;outline-offset:1px; }
                    .community-channel-identity > input::placeholder { color:#aaa; }
                    .community-channel-footer { display:flex;align-items:center;justify-content:space-between;gap:20px;margin-top:20px; }
                    .community-channel-footer > p { max-width:285px;color:#ddd;font-size:15px;line-height:1.45; }
                    .community-channel-footer > div { display:flex;gap:10px; }
                    .community-channel-footer button { min-width:108px;height:36px;font-size:14px; }
                    .community-channel-footer button[type=button] { background:transparent;border:1px solid #555; }
                    .community-channel-footer button[type=submit]:disabled { background:#171717;color:#777;opacity:1; }
                    .community-channel-error { color:#ffbaba;font-size:13px;line-height:1.5;margin-top:8px; }
                    @media(max-width:540px) { .community-channel-dialog { padding:22px; } .community-channel-dialog [data-slot=dialog-title] { font-size:20px; } .community-channel-footer { align-items:stretch;flex-direction:column; } .community-channel-footer > div { justify-content:flex-end; } }
                    .community-detail { min-width:0;background:#262626;border-radius:12px 0 0 0;margin:10px 0 0;overflow:hidden; }
                    .community-cover { position:relative;height:clamp(160px,22vw,280px);margin:12px 12px 0;overflow:hidden;border-radius:8px;background:#bfe9d0; }
                    .community-cover[data-cover=lavender] { background:#d2c9f4; }
                    .community-cover[data-cover=sunset] { background:#f4d2b2; }
                    .community-cover-art { position:absolute;inset:-35% -8%;transform:rotate(-24deg);background:radial-gradient(ellipse at 30% 30%,#ebff9c55,transparent 60%),repeating-radial-gradient(ellipse at 55% 60%,transparent 0 30px,#ffffff55 32px 34px,transparent 36px 62px); }
                    .community-cover-art::after { content:"";position:absolute;inset:5% 18%;border:45px solid #658c5655;border-radius:50%;transform:rotate(28deg);background:radial-gradient(#426e5266 1px,transparent 2px) 0 0 / 9px 9px;box-shadow:0 0 24px #fff8,inset 0 0 25px #fffc; }
                    .community-cover-edit { position:absolute;right:10px;bottom:10px;display:grid;place-items:center;border-radius:4px;padding:8px;background:#262626;color:white; }
                    .community-cover-picker { position:absolute;right:10px;bottom:52px;display:flex;gap:6px;padding:8px;border-radius:6px;background:#292929;box-shadow:0 4px 20px #0006; }
                    .community-cover-picker button { font-size:13px;padding:8px;border:1px solid #555;border-radius:4px; }
                    .community-detail-header { display:flex;align-items:center;gap:22px;min-height:72px;padding:0 22px;border-bottom:1px solid #ffffff12;flex-wrap:wrap; }
                    .community-identity-title { display:flex;align-items:center;gap:12px;min-width:0; }
                    .community-identity-title h2 { font-size:20px;font-weight:700;max-width:230px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-tabs { display:flex;gap:20px;align-self:stretch; }
                    .community-tabs button { position:relative;font-size:14px;color:#c4c4c4;padding:18px 0; }
                    .community-tabs button[aria-selected=true] { color:white;font-weight:600; }
                    .community-tabs button[aria-selected=true]::after { content:"";position:absolute;bottom:0;left:0;right:0;height:3px;border-radius:3px;background:#9186ef; }
                    .community-header-actions { display:flex;align-items:center;gap:8px;margin-left:auto; }
                    .community-event-link { display:flex;align-items:center;gap:8px;border:1px solid #555;border-radius:4px;padding:8px 12px;font-size:14px; }
                    .community-content { max-width:1140px;margin:auto;padding:24px clamp(18px,5vw,90px) 50px; }
                    .community-composer,.community-post { border:1px solid #222;border-radius:8px;background:#282828;padding:18px;box-shadow:0 2px 4px #0003; }
                    .community-composer-heading { display:flex;align-items:center;gap:12px;font-size:14px;font-weight:600;color:#ddd; }
                    .community-author-avatar { width:36px;height:36px;display:grid;place-items:center;flex-shrink:0;border-radius:50%;background:#d2dce9;color:#35445c;font-size:13px; }
                    .community-composer textarea { width:100%;min-height:66px;resize:vertical;padding:14px 0 8px;outline:none;background:transparent;font-size:14px; }
                    .community-composer:focus-within textarea,.community-composer textarea:not(:placeholder-shown) { min-height:110px; }
                    .community-composer-actions { display:flex;align-items:center;gap:22px; }
                    .community-composer-actions > button[type=button] { color:#ddd;font-size:14px;padding:8px; }
                    .community-composer-actions > button[aria-pressed=true] { color:#b6b0ff; }
                    .community-setup { padding:48px 0 20px;text-align:center; }
                    .community-setup h2 { font-size:23px;font-weight:700;margin-bottom:10px; }
                    .community-setup > p { color:#ccc;font-size:15px; }
                    .community-quick-actions { display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:32px; }
                    .community-quick-actions > * { display:flex;align-items:center;gap:16px;text-align:left;padding:18px;border:1px solid #555;border-radius:5px;font-size:16px;font-weight:600; }
                    .community-quick-actions span { font-size:32px; }
                    .community-post { margin-top:20px; }
                    .community-post header { display:flex;align-items:center;gap:10px;font-size:13px;flex-wrap:wrap; }
                    .community-post time { color:#aaa;font-size:11px;margin-left:auto; }
                    .community-post > p,.community-details-panel p { white-space:pre-wrap;overflow-wrap:anywhere;line-height:1.6;font-size:14px;margin-top:14px; }
                    .community-post h3 { color:#b6b0ff;font-weight:700;margin-top:16px; }
                    .community-details-panel { padding:22px;border-bottom:1px solid #444;background:#202020; }
                    .community-details-panel h3 { font-weight:600;margin-top:14px; }
                    .community-details-panel p { color:#bbb;margin-top:6px; }
                    .community-empty-tab { display:flex;align-items:center;flex-direction:column;gap:16px;padding:70px 24px;text-align:center;color:#bbb; }
                    .community-photo { width:200px;height:200px;object-fit:cover;border-radius:10px; }
                    .community-workspace-error { margin:16px 24px;padding:12px;border:1px solid #b76a6a;border-radius:6px;color:#ffbaba; }
                    @media(max-width:1000px) { .community-workspace { grid-template-columns:240px minmax(0,1fr); } .community-detail-header { gap:12px;padding:10px 16px 0; } .community-content { padding:22px; } .community-header-actions { margin-bottom:10px; } }
                    @media(max-width:700px) { .community-workspace { display:block; } .community-sidebar { padding:16px; } .community-sidebar-heading { margin-bottom:12px; } .community-sidebar nav { display:flex;gap:8px;overflow:auto; } .community-nav-item { min-width:150px; } .community-channel-list { padding-left:8px; } .community-detail { margin:0;border-radius:12px 12px 0 0; } .community-quick-actions { grid-template-columns:1fr; } .community-cover { height:150px; } .community-detail-header { gap:12px; } .community-identity-title { width:100%; } }
                    .community-create-dialog {
                        width: min(800px, calc(100vw - 40px)); max-width: 800px;
                        min-height: min(740px, calc(100dvh - 40px));
                        max-height: calc(100dvh - 40px); overflow-y: auto;
                        padding: 32px; border: 1px solid #d6d6d6; border-radius: 5px;
                        background: #292929; color: #f5f5f5; gap: 34px;
                        grid-template-rows: auto 1fr;
                    }
                    .community-create-close { color: #ccc; padding: 4px; border-radius: 4px; }
                    .community-create-close:hover { background: #3d3d3d; }
                    .community-create-form { display: flex; flex-direction: column; min-height: 0; }
                    .community-identity { display: flex; align-items: center; gap: 26px; }
                    .community-avatar { position: relative; flex-shrink: 0; width: 118px; height: 118px; overflow: hidden; border: 1px solid #777; border-radius: 5px; background: linear-gradient(140deg, #faedcf, #eccc83); }
                    .community-avatar-emoji { display: grid; place-items: center; height: 100%; padding-bottom: 12px; font-size: 65px; }
                    .community-avatar-photo { width: 100%; height: 100%; object-fit: cover; }
                    .community-avatar-edit { position: absolute; bottom: 0; inset-inline: 0; background: #625332cc; padding: 3px; color: white; font-size: 16px; }
                    .community-name-input { height: 58px; border: 0; border-bottom: 1px solid #939393; border-radius: 0; background: transparent; padding: 4px 12px; font-size: 30px; font-weight: 700; box-shadow: none; }
                    .community-name-input:focus-visible { border-color: #a6a0ff; outline: 2px solid #a6a0ff; outline-offset: 2px; }
                    .community-description { min-height: 110px; resize: vertical; border: 0; border-radius: 5px; background: #202020; padding: 12px 16px; font-size: 16px; line-height: 1.5; }
                    .community-description::placeholder { color: #999; }
                    .community-guidelines-preview { white-space: pre-wrap; overflow-wrap: anywhere; font-size: 16px; line-height: 1.5; padding-right: 24px; }
                    .community-guidelines-edit { color: #ccc; padding: 4px; border-radius: 4px; font-size: 13px; }
                    .community-guidelines-edit:hover { background: #3d3d3d; }
                    .community-create-footer { margin-top: auto; padding-top: 40px; }
                    @media (max-width: 540px) {
                        .community-create-dialog { padding: 22px; gap: 26px; }
                        .community-identity { gap: 16px; }
                        .community-avatar { width: 84px; height: 84px; }
                        .community-avatar-emoji { font-size: 44px; }
                        .community-name-input { font-size: 23px; }
                    }
                "##</style>
                <script>r##"
                    const [communities, setCommunities] = pp.state(@{Json(&saved)});
                    const searchStore = pp.context(window.UpurnakaSearchContext);
                    const communityElements = pp.ref({});
                    const [selectedCommunity, setSelectedCommunity] = pp.state(@{selected.as_str()});
                    const [channelId, setChannelId] = pp.state('');
                    const [activeTab, setActiveTab] = pp.state('posts');
                    const postsTab = pp.ref(null), filesTab = pp.ref(null), photosTab = pp.ref(null);
                    function handleCommunityTabKeys(event) {
                        const tabs = ['posts', 'files', 'photos'];
                        let index = tabs.indexOf(activeTab);
                        if (event.key === 'ArrowRight') index = (index + 1) % 3;
                        else if (event.key === 'ArrowLeft') index = (index + 2) % 3;
                        else if (event.key === 'Home') index = 0;
                        else if (event.key === 'End') index = 2;
                        else return;
                        event.preventDefault(); setActiveTab(tabs[index]);
                        [postsTab, filesTab, photosTab][index].current?.focus();
                    }
                    const [channelName, setChannelName] = pp.state('');
                    const [channelIcon, setChannelIcon] = pp.state('💬');
                    const [channelError, setChannelError] = pp.state('');
                    const [channelCommunityId, setChannelCommunityId] = pp.state('');
                    const channelOpener = pp.ref(null), channelCloser = pp.ref(null);
                    const onChannelIconSelected = pp.event(icon => setChannelIcon(icon));
                    const channelEmojiImages = pp.memo(() => @{Json(&crate::emoji::image_map())}, []);
                    function channelEmojiImage(icon) { return channelEmojiImages[String(icon || '').replace(/\uFE0F/g, '')] || ''; }
                    const channelCommunity = communities.find(c=>c.id===channelCommunityId) || null;
                    const channelDialogTitle = 'Create a channel in "' + (channelCommunity?.name || '') + '"';
                    const channelAtLimit = (channelCommunity?.channels.length || 0) >= 5;
                    function openChannelDialog() {
                        if (!activeCommunity?.is_owner || working) return;
                        setChannelCommunityId(activeCommunity.id); setChannelName(''); setChannelIcon('💬'); setChannelError('');
                        channelOpener.current?.click();
                    }
                    const [postDraft, setPostDraft] = pp.state('');
                    const [announcement, setAnnouncement] = pp.state(false);
                    const [coverPicker, setCoverPicker] = pp.state(false);
                    const [detailsOpen, setDetailsOpen] = pp.state(false);
                    const [working, setWorking] = pp.state(false);
                    const [workspaceError, setWorkspaceError] = pp.state('');
                    const activeCommunity = communities.find(c => c.id === selectedCommunity) || null;
                    const activeChannel = activeCommunity?.channels.find(c => c.id === channelId) || null;
                    const activePosts = activeCommunity?.posts.filter(p => p.channel_id === channelId) || [];
                    function authorInitials(name) { return String(name || '').trim().split(/\s+/).slice(0,2).map(part=>part[0] || '').join('').toUpperCase(); }
                    const [pendingInvites, setPendingInvites] = pp.state(@{Json(&invitations)});
                    const [joinPreview, setJoinPreview] = pp.state(@{Json(&join_preview)});
                    const joinToken = @{join_token.as_str()};
                    const [notice, setNotice] = pp.state('');
                    const actionsMenu = pp.ref(null);
                    const [menuId, setMenuId] = pp.state('');
                    const [menuPosition, setMenuPosition] = pp.state('');
                    const shareMenuItem = pp.ref(null), inviteMenuItem = pp.ref(null), notificationMenuItem = pp.ref(null), manageMenuItem = pp.ref(null), editMenuItem = pp.ref(null), reportMenuItem = pp.ref(null), leaveMenuItem = pp.ref(null), deleteMenuItem = pp.ref(null);
                    const menuCommunity = communities.find(c => c.id === menuId) || null;
                    function prepareMenu(id, event) {
                        setMenuId(id); const rect = event.currentTarget.getBoundingClientRect();
                        setMenuPosition('left:' + Math.max(12, Math.min(rect.right - 260, innerWidth - 272)) + 'px;top:' + Math.max(12, Math.min(rect.bottom + 6, innerHeight - 450)) + 'px');
                    }
                    function menuItems() { return [shareMenuItem,inviteMenuItem,notificationMenuItem,manageMenuItem,editMenuItem,reportMenuItem,leaveMenuItem,deleteMenuItem].map(ref => ref.current).filter(element => element && !element.hidden && !element.disabled); }
                    const focusMenu = pp.event(() => { if (!actionsMenu.current?.contains(document.activeElement)) menuItems()[0]?.focus(); });
                    function menuToggled(event) { if (event.newState === 'open') requestAnimationFrame(focusMenu); }
                    function openMenuKey(id, event) {
                        if (event.key !== 'ArrowDown') return;
                        event.preventDefault(); prepareMenu(id, event); actionsMenu.current?.showPopover();
                    }
                    function menuKey(event) {
                        const items = menuItems(), index = items.indexOf(document.activeElement);
                        if (event.key === 'Tab') { actionsMenu.current?.hidePopover(); return; }
                        let next;
                        if (event.key === 'ArrowDown') next = (index + 1) % items.length;
                        else if (event.key === 'ArrowUp') next = (index + items.length - 1) % items.length;
                        else if (event.key === 'Home') next = 0;
                        else if (event.key === 'End') next = items.length - 1;
                        else return;
                        event.preventDefault(); items[next]?.focus();
                    }
                    const actionOpener = pp.ref(null), actionCloser = pp.ref(null), joinLinkInput = pp.ref(null);
                    const [action, setAction] = pp.state('');
                    const [actionId, setActionId] = pp.state('');
                    const [actionError, setActionError] = pp.state('');
                    const [actionNotice, setActionNotice] = pp.state('');
                    const [shareUrl, setShareUrl] = pp.state('');
                    const [inviteEmail, setInviteEmail] = pp.state('');
                    const [notificationPreference, setNotificationPreference] = pp.state('all');
                    const [members, setMembers] = pp.state([]);
                    const [transferTarget, setTransferTarget] = pp.state('');
                    const [reportDetails, setReportDetails] = pp.state('');
                    const [reportReceipt, setReportReceipt] = pp.state('');
                    const actionCommunity = communities.find(c => c.id === actionId) || null;
                    const actionTitle = {share:'Share join link',invite:'Invite members',notifications:'Notifications',manage:'Manage community',report:'Report a concern',leave:'Leave community',delete:'Delete community'}[action] || 'Community';
                    async function openAction(kind, id) {
                        actionsMenu.current?.hidePopover(); setAction(kind); setActionId(id); setActionError(''); setActionNotice(''); setTransferTarget(''); setReportDetails('');setReportReceipt('');setInviteEmail('');setShareUrl('');
                        const target = communities.find(c => c.id === id); setNotificationPreference(target?.notifications || 'all');
                        actionOpener.current?.click();
                        if (!['share','manage'].includes(kind)) return;
                        setWorking(true);
                        try {
                            if (kind === 'share') setShareUrl(location.origin + await pp.rpc('community_join_link', {id,reset:false}));
                            else setMembers(await pp.rpc('community_members', {id}));
                        } catch (failure) {setActionError(failure.message || 'Could not load community settings.');}
                        finally {setWorking(false);}
                    }
                    async function copyJoinLink() {
                        try { await navigator.clipboard.writeText(shareUrl);setActionNotice('Join link copied.'); }
                        catch (_) {joinLinkInput.current?.focus();joinLinkInput.current?.select();setActionNotice('Select and copy the join link above.');}
                    }
                    async function sendInvitation(event) {
                        event.preventDefault();if(working)return;setWorking(true);setActionError('');
                        try {await pp.rpc('invite_community_member',{id:actionId,email:inviteEmail});setInviteEmail('');setActionNotice('Invitation saved. This person can accept it in Communities.');}
                        catch(failure){setActionError(failure.message);}
                        finally{setWorking(false);}
                    }
                    async function saveNotifications(event) {
                        event.preventDefault();if(working)return;setWorking(true);setActionError('');
                        try {const preference=await pp.rpc('set_community_notifications',{id:actionId,preference:notificationPreference});updateCommunity(actionId,()=>({notifications:preference}));setActionNotice('Notification preference saved.');}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    async function resetJoinLink() {
                        if(working)return;setWorking(true);setActionError('');
                        try {await pp.rpc('community_join_link',{id:actionId,reset:true});setActionNotice('New join link created. Previous join links are no longer valid.');}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    async function removeMember(target) {
                        if(working)return;setWorking(true);setActionError('');
                        try {await pp.rpc('remove_community_member',{id:actionId,target});const next=await pp.rpc('community_members',{id:actionId});setMembers(next);updateCommunity(actionId,()=>({member_count:next.filter(m=>m.state==='joined').length}));}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    async function transferOwnership() {
                        if(working)return;setWorking(true);setActionError('');
                        try {await pp.rpc('transfer_community_ownership',{id:actionId,target:transferTarget});updateCommunity(actionId,()=>({is_owner:false}));setMembers(await pp.rpc('community_members',{id:actionId}));setTransferTarget('');setActionNotice('Ownership transferred. You are now a member.');}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    async function submitReport(event) {
                        event.preventDefault();if(working||reportReceipt)return;setWorking(true);setActionError('');
                        try {const receipt=await pp.rpc('report_community',{id:actionId,details:reportDetails});setReportReceipt(receipt);setActionNotice('Report submitted to workspace administrators. Reference: '+receipt);}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    function removeFromList(id) {
                        const next=communities.filter(c=>c.id!==id);setCommunities(next);
                        if(selectedCommunity===id)selectCommunity(next[0]?.id || '');actionCloser.current?.click();
                    }
                    async function leaveCommunity() {
                        if(working)return;setWorking(true);setActionError('');
                        try {await pp.rpc('leave_community',{id:actionId});removeFromList(actionId);setNotice('You left the community.');}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    async function deleteCommunity() {
                        if(working)return;setWorking(true);setActionError('');
                        try {await pp.rpc('delete_community',{id:actionId});removeFromList(actionId);setNotice('Community deleted.');}
                        catch(failure){setActionError(failure.message);}finally{setWorking(false);}
                    }
                    async function acceptInvitation(id, token) {
                        if(working)return;setWorking(true);
                        try {const joined=await pp.rpc('join_community',{id,token});setCommunities(current=>[...current.filter(c=>c.id!==id),joined]);selectCommunity(id);setPendingInvites(current=>current.filter(i=>i.id!==id));setJoinPreview(null);setNotice('You joined the community.');}
                        catch(failure){setNotice(failure.message);}finally{setWorking(false);}
                    }
                    function selectCommunity(id) {
                        setSelectedCommunity(id); selectChannel(''); setDetailsOpen(false); setCoverPicker(false);
                        const url = new URL(location.href); url.searchParams.set('community', id); history.replaceState(null, '', url);
                    }
                    function selectChannel(id) { setChannelId(id); setActiveTab('posts'); setPostDraft(''); setWorkspaceError(''); }
                    function updateCommunity(id, changes) { setCommunities(current => current.map(c => c.id === id ? {...c, ...changes(c)} : c)); }
                    async function addChannel(event) {
                        event.preventDefault(); if (working || channelAtLimit) return;
                        const fields = Object.fromEntries(new FormData(event.currentTarget));
                        const name = String(fields.name || '').trim(); if (!name) return;
                        setWorking(true); setChannelError(''); const id = channelCommunityId;
                        try {
                            const channel = await pp.rpc('create_community_channel', {id, name, icon:channelIcon});
                            updateCommunity(id, c => ({channels:[...c.channels, channel]}));
                            channelCloser.current?.click(); selectCommunity(id); selectChannel(channel.id);
                        } catch (failure) { setChannelError(failure.body?.error || failure.message || 'The channel could not be saved.'); }
                        finally { setWorking(false); }
                    }
                    async function publishPost(event) {
                        event.preventDefault(); if (working || !postDraft.trim()) return;
                        setWorking(true); setWorkspaceError(''); const id = selectedCommunity;
                        try {
                            const post = await pp.rpc('publish_community_post', {id, channel:channelId, body:postDraft, announcement});
                            updateCommunity(id, c => ({posts:[post, ...c.posts]})); setPostDraft('');
                        } catch (failure) { setWorkspaceError(failure.message || 'Your post could not be saved.'); }
                        finally { setWorking(false); }
                    }
                    async function changeCover(cover) {
                        if (working) return; setWorking(true); setWorkspaceError(''); const id = selectedCommunity;
                        try { const saved = await pp.rpc('change_community_cover', {id, cover}); updateCommunity(id, () => ({cover:saved})); setCoverPicker(false); }
                        catch (failure) { setWorkspaceError(failure.message || 'The cover could not be saved.'); }
                        finally { setWorking(false); }
                    }
                    const openSearchCommunity = pp.event(row => {
                        selectCommunity(row.id);
                        requestAnimationFrame(() => {
                            const card = communityElements.current[row.id];
                            card?.focus({preventScroll:true}); card?.scrollIntoView({block:'center'});
                        });
                    });
                    pp.effect(() => {
                        searchStore?.publish('communities',communities.map(community => ({key:'community:'+community.id,id:community.id,kind:'communities',title:community.name,subtitle:community.category})),openSearchCommunity);
                    },[communities,searchStore,openSearchCommunity]);
                    pp.effect(() => () => searchStore?.remove('communities'),[]);
                    const createOpener = pp.ref(null);
                    const createCloser = pp.ref(null);
                    const CATEGORIES = [
                        "Family and friends", "Group projects", "Individual projects",
                        "Hobbies", "Events", "Gaming or recreation",
                    ];
                    const DEFAULT_GUIDELINES = "Be kind and respectful to your fellow community members. Don't be rude or cruel. Participate as yourself and don't post anything that violates community guidelines.";
                    const pictureInput = pp.ref(null);
                    const [picture, setPicture] = pp.state(null);
                    const [picturePreview, setPicturePreview] = pp.state("");
                    const [pictureError, setPictureError] = pp.state("");
                    const pictureLoaded = pp.event(event => setPicturePreview(String(event.target.result || "")));
                    const pictureFailed = pp.event(() => rejectPicture());
                    pp.effect(() => {
                        if (!picture) return;
                        const reader = new FileReader();
                        reader.onload = pictureLoaded;
                        reader.onerror = pictureFailed;
                        reader.readAsDataURL(picture);
                        return () => { reader.onload = null; reader.onerror = null; if (reader.readyState === 1) reader.abort(); };
                    }, [picture]);
                    const guidelinesInput = pp.ref(null);
                    const [description, setDescription] = pp.state("");
                    const [guidelines, setGuidelines] = pp.state(DEFAULT_GUIDELINES);
                    const [avatar, setAvatar] = pp.state("👥");
                    const [editingGuidelines, setEditingGuidelines] = pp.state(false);
                    const focusGuidelines = pp.event(() => {
                        if (editingGuidelines) guidelinesInput.current?.focus();
                    });
                    pp.effect(() => {
                        if (!editingGuidelines) return;
                        const frame = requestAnimationFrame(focusGuidelines);
                        return () => cancelAnimationFrame(frame);
                    }, [editingGuidelines, focusGuidelines]);
                    const [fieldErrors, setFieldErrors] = pp.state({});
                    const [step, setStep] = pp.state(1);
                    const [name, setName] = pp.state("");
                    const [template, setTemplate] = pp.state("");
                    const [error, setError] = pp.state("");
                    const [saving, setSaving] = pp.state(false);
                    const [editingId, setEditingId] = pp.state('');
                    const [editingPicture, setEditingPicture] = pp.state('');
                    const [removePicture, setRemovePicture] = pp.state(false);
                    function openEdit(id) {
                        actionsMenu.current?.hidePopover();const target=communities.find(c=>c.id===id);if(!target?.is_owner)return;
                        setEditingId(id);setEditingPicture(target.picture || '');setRemovePicture(false);setName(target.name);setTemplate(target.category);setDescription(target.description);setGuidelines(target.guidelines);setAvatar(target.avatar);setPicture(null);setPicturePreview(target.picture || '');setPictureError('');setError('');setFieldErrors({});setEditingGuidelines(false);setStep(1);createOpener.current?.click();
                    }
                    function clearCommunityPicture(){setPicture(null);setPicturePreview('');setEditingPicture('');setRemovePicture(true);}

                    function openCreate(fromTemplate) {
                        setEditingId('');setEditingPicture('');setRemovePicture(false);
                        setTemplate(fromTemplate);
                        setName(fromTemplate);
                        setDescription("");
                        setGuidelines(DEFAULT_GUIDELINES);
                        setAvatar({School:"🎓",Business:"🏪",Life:"🏅"}[fromTemplate] || "👥");
                        setPicture(null);
                        setPicturePreview("");
                        setPictureError("");
                        if (pictureInput.current) pictureInput.current.value = "";
                        setEditingGuidelines(false);
                        setFieldErrors({});
                        setError("");
                        setStep(1);
                        createOpener.current?.click();
                    }

                    function openImagePicker() {
                        if (saving || !pictureInput.current) return;
                        pictureInput.current.value = "";
                        pictureInput.current.click();
                    }

                    function choosePicture(event) {
                        const file = event.currentTarget.files[0];
                        if (!file) return;
                        if (!['image/png', 'image/jpeg'].includes(file.type) || file.size === 0 || file.size > 2 * 1024 * 1024) {
                            setPictureError("Choose a PNG or JPEG picture up to 2 MB.");
                            return;
                        }
                        setPictureError("");
                        setFieldErrors({...fieldErrors, picture: undefined});
                        setRemovePicture(false);
                        setPicturePreview("");
                        setPicture(file);
                    }

                    function rejectPicture() {
                        setPicture(null);
                        setPicturePreview("");
                        setPictureError("This picture could not be read. Choose a valid PNG or JPEG.");
                    }

                    function toggleGuidelines() {
                        const editing = !editingGuidelines;
                        setEditingGuidelines(editing);
                    }

                    function goToCategory(event) {
                        event.preventDefault();
                        if (saving || name.trim().length < 2 || (picture && !picturePreview)) return;
                        if (editingId || template) create(template);
                        else setStep(2);
                    }

                    async function create(category) {
                        if (saving) return;
                        setSaving(true);
                        setError("");
                        try {
                            const community = editingId ? await pp.rpc('edit_community', {id:editingId,name:name.trim(),description:description.trim(),guidelines:guidelines.trim(),picture,remove_picture:removePicture}) : await pp.rpc("create_community", {
                                name: name.trim(),
                                category: category,
                                description: description.trim(),
                                guidelines: guidelines.trim(),
                                avatar: avatar,
                                picture: picture,
                            });
                            setCommunities((current) => editingId ? current.map(c=>c.id===editingId?community:c) : current.concat([community]));
                            selectCommunity(community.id);
                            createCloser.current?.click();
                        } catch (failure) {
                            setFieldErrors(failure.errors || {});
                            setError(failure.message || "The community could not be created.");
                            if (failure.errors && !failure.errors.category) setStep(1);
                        } finally {
                            setSaving(false);
                        }
                    }
                "##</script>
            </main>
        </AppShell>
    })
}

/// The categories the create flow offers. A value outside this list is a 422,
/// so a hand-made request cannot invent one.
const CATEGORIES: [&str; 9] = [
    "Family and friends",
    "Group projects",
    "Individual projects",
    "Hobbies",
    "Events",
    "Gaming or recreation",
    "School",
    "Business",
    "Life",
];

#[rpc(auth)]
pub async fn create_community(
    name: String,
    category: String,
    description: Option<String>,
    guidelines: Option<String>,
    avatar: Option<String>,
    picture: Option<RpcFile>,
) -> Result<Community> {
    let name = name.trim().to_string();
    let category = category.trim().to_string();
    let description = description.unwrap_or_default().trim().to_string();
    let guidelines = guidelines.unwrap_or_default().trim().to_string();
    let avatar = avatar.unwrap_or_else(|| "👥".to_string());
    let allowed_images: Vec<String> = ["🎓", "🏪", "🏅", "👥", "🌱", "⚽", "🎨", "💡"]
        .iter()
        .map(|value| value.to_string())
        .collect();
    let allowed: Vec<String> = CATEGORIES.iter().map(|value| value.to_string()).collect();

    let mut validator = Validator::new();
    validator
        .required("name", &name)
        .min_chars("name", &name, 2)
        .max_chars("name", &name, 80)
        .one_of("category", &category, &allowed)
        .max_chars("description", &description, 1000)
        .max_chars("guidelines", &guidelines, 2000)
        .one_of("avatar", &avatar, &allowed_images);
    validator.finish()?;

    let picture = match picture {
        Some(file) => Some(community_picture(file).await?),
        None => None,
    };

    let id = communities::new_id();

    communities::save(Community {
        id,
        name,
        category,
        description,
        guidelines,
        avatar,
        picture,
        cover: "mint".into(),
        channels: Vec::new(),
        posts: Vec::new(),
        is_owner: true,
        notifications: "all".into(),
        member_count: 1,
    })
    .await
}

#[rpc(auth)]
pub async fn create_community_channel(
    id: String,
    name: String,
    icon: Option<String>,
) -> Result<communities::Channel> {
    communities::add_channel(id, name, icon.unwrap_or_else(|| "💬".into())).await
}
#[rpc(auth)]
pub async fn publish_community_post(
    id: String,
    channel: String,
    body: String,
    announcement: bool,
) -> Result<communities::Post> {
    communities::publish(id, channel, body, announcement).await
}
#[rpc(auth)]
pub async fn change_community_cover(id: String, cover: String) -> Result<String> {
    communities::cover(id, cover).await
}

#[rpc(auth)]
pub async fn edit_community(
    id: String,
    name: String,
    description: String,
    guidelines: String,
    picture: Option<RpcFile>,
    remove_picture: bool,
) -> Result<Community> {
    let picture = match picture {
        Some(file) => Some(community_picture(file).await?),
        None => None,
    };
    communities::edit(id, name, description, guidelines, picture, remove_picture).await
}
#[rpc(auth)]
pub async fn community_members(id: String) -> Result<Vec<communities::Member>> {
    communities::members(id).await
}
#[rpc(auth)]
pub async fn community_join_link(id: String, reset: bool) -> Result<String> {
    communities::join_link(id, reset).await
}
#[rpc(auth)]
pub async fn invite_community_member(id: String, email: String) -> Result<bool> {
    communities::invite(id, email).await
}
#[rpc(auth)]
pub async fn join_community(id: String, token: Option<String>) -> Result<Community> {
    communities::join(id, token).await
}
#[rpc(auth)]
pub async fn set_community_notifications(id: String, preference: String) -> Result<String> {
    communities::notifications(id, preference).await
}
#[rpc(auth)]
pub async fn transfer_community_ownership(id: String, target: String) -> Result<bool> {
    communities::transfer(id, target).await
}
#[rpc(auth)]
pub async fn remove_community_member(id: String, target: String) -> Result<bool> {
    communities::remove_member(id, target).await
}
#[rpc(auth)]
pub async fn leave_community(id: String) -> Result<bool> {
    communities::leave(id).await
}
#[rpc(auth)]
pub async fn delete_community(id: String) -> Result<bool> {
    communities::delete(id).await
}
#[rpc(auth)]
pub async fn report_community(id: String, details: String) -> Result<String> {
    communities::report(id, details).await
}

/// Decode the content rather than trusting the upload's name or MIME type.
/// Return only a bounded, re-encoded thumbnail to the community card.
async fn community_picture(picture: RpcFile) -> Result<String> {
    use base64::Engine;

    let mut validator = Validator::new();
    validator.check(
        "picture",
        !picture.is_empty() && picture.len() <= 2 * 1024 * 1024,
        "Choose a PNG or JPEG picture up to 2 MB.",
    );
    validator.finish()?;
    let bytes = picture.into_bytes().await?;
    let normalized =
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<u8>, String> {
            let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
                .with_guessed_format()
                .map_err(|_| "Choose a valid PNG or JPEG picture.".to_string())?;
            if !matches!(
                reader.format(),
                Some(image::ImageFormat::Png | image::ImageFormat::Jpeg)
            ) {
                return Err("Choose a PNG or JPEG picture.".into());
            }
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(4096);
            limits.max_image_height = Some(4096);
            limits.max_alloc = Some(64 * 1024 * 1024);
            reader.limits(limits);
            let decoded = reader.decode().map_err(|_| {
                "Choose a valid picture no larger than 4096 × 4096 pixels.".to_string()
            })?;
            let thumbnail = decoded.resize_to_fill(256, 256, image::imageops::FilterType::Lanczos3);
            let mut output = std::io::Cursor::new(Vec::new());
            thumbnail
                .write_to(&mut output, image::ImageFormat::Png)
                .map_err(|_| "This picture could not be processed.".to_string())?;
            Ok(output.into_inner())
        })
        .await?;
    let mut validator = Validator::new();
    validator.check(
        "picture",
        normalized.is_ok(),
        normalized
            .as_ref()
            .err()
            .map(String::as_str)
            .unwrap_or("Choose a valid picture."),
    );
    validator.finish()?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(normalized.unwrap())
    ))
}
