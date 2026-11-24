//! Communities: the invitation to start one, and the ones already started.

use crate::chat_files;
use crate::communities::{self, Community};
use crate::community_events;
use axum::extract::Query;
use serde::Deserialize;

use crate::components::app_shell::AppShell;
use crate::components::aria::{hidden, named, submit};
use crate::components::channel_emoji_picker::ChannelEmojiPicker;
use crate::components::emoji_panel::EmojiPanel;
use crate::components::fluent_emoji_grid::FluentEmojiGrid;
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::Attrs;
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
use crate::components::rahti_ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
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

// The Dilnaka Storage picker behind "Attach cloud files", laid out as Chat's:
// places on the left, the folder being shown on the right.
const CLOUD_DIALOG: &str = "community-cloud-dialog h-[min(640px,calc(100dvh-32px))] grid-rows-[auto_minmax(0,1fr)] gap-0! overflow-hidden bg-surface! p-0! sm:max-w-[960px]! sm:grid-cols-[216px_minmax(0,1fr)] sm:grid-rows-1";
const CLOUD_NAV: &str = "flex gap-1 overflow-x-auto border-b border-line bg-pane px-3 py-2 sm:flex-col sm:overflow-visible sm:border-b-0 sm:border-r sm:px-3 sm:py-6";
const CLOUD_NAV_ITEM: &str = "relative flex shrink-0 items-center gap-3 rounded-md px-3 py-2 text-sm text-ink/85 hover:bg-hover before:absolute before:inset-y-2 before:-left-3 before:w-[3px] before:rounded-full before:content-[''] max-sm:before:hidden data-[active=true]:bg-hover data-[active=true]:font-semibold data-[active=true]:text-ink data-[active=true]:before:bg-brand-ink [&[data-active=true]_svg]:text-brand-ink";
const CLOUD_HEAD: &str = "grid grid-cols-[28px_40px_minmax(0,1fr)_110px] items-center border-b border-line px-2 py-1.5 text-xs font-medium text-ink/85 sm:grid-cols-[28px_44px_minmax(0,1fr)_150px_120px] sm:text-[13px]";
const CLOUD_ROW: &str = "group grid w-full grid-cols-[28px_40px_minmax(0,1fr)_110px] items-center border-b border-line px-2 py-2.5 text-left hover:bg-hover focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring data-[selected=true]:bg-brand-soft/70 sm:grid-cols-[28px_44px_minmax(0,1fr)_150px_120px]";
const CLOUD_CHECK: &str = "flex size-[18px] items-center justify-center justify-self-center rounded-full border border-edge text-white opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100 group-data-[selected=true]:border-brand group-data-[selected=true]:bg-brand group-data-[selected=true]:opacity-100 max-sm:opacity-100";
const CLOUD_NOTE: &str = "px-3 py-3 text-sm text-muted-foreground";
/// A folder drawn filled, as drives draw them.
const FOLDER_ICON: &str = "size-7 fill-tint-gold text-tint-amber";

/// A preset tile in the Edit banner dialog.
#[derive(serde::Serialize)]
struct BannerChoice {
    key: &'static str,
    label: &'static str,
}

fn banner_label(key: &str) -> &'static str {
    match key {
        "stationery" => "Stationery on lavender",
        "paintbrush" => "Paintbrush and butterfly",
        "keepsakes" => "Heart, flower and photos",
        "bakery" => "Box of pastries",
        "giftbox" => "Wrapped gift",
        "stars" => "Puffy stars",
        "clock" => "Wall clock",
        "compass" => "Compass and rope",
        "mint" => "Mint rings",
        "lavender" => "Lavender waves",
        "sunset" => "Sunset waves",
        _ => "Banner",
    }
}

#[derive(Default, Deserialize)]
pub struct CommunityQuery {
    community: Option<String>,
    join: Option<String>,
    /// A post to open from a copied link.
    post: Option<String>,
}

pub async fn page(Query(query): Query<CommunityQuery>) -> Result<Html> {
    let saved = communities::list().await?;
    let invitations = communities::invitations().await?;
    let join_preview = match query.join.as_deref() {
        Some(token) => communities::preview(token).await?,
        None => None,
    };
    let join_token = query.join.unwrap_or_default();
    let focus_post = query.post.unwrap_or_default();
    let selected = query
        .community
        .filter(|id| saved.iter().any(|c| &c.id == id))
        .or_else(|| saved.first().map(|c| c.id.clone()))
        .unwrap_or_default();
    let banners: Vec<_> = communities::BANNERS
        .iter()
        .map(|key| BannerChoice {
            key,
            label: banner_label(key),
        })
        .collect();
    let user = crate::auth::current().unwrap_or_default();
    let initials = user.initials();
    let author = user.name.clone();
    let compose_emojis: Vec<_> = crate::emoji::catalog()
        .iter()
        .take(48)
        .map(|emoji| {
            rahti::serde_json::json!({"symbol": emoji.glyph, "name": emoji.name, "image": emoji.image})
        })
        .collect();
    let cancel_link = button_variants(
        ButtonVariant::Link,
        ButtonSize::Default,
        "text-[13px] text-link",
    );
    let cancel_button = button_variants(ButtonVariant::Outline, ButtonSize::Default, "");
    let app_zone = crate::rahti::time::timezone().name();
    let upload_button = button_variants(
        ButtonVariant::Default,
        ButtonSize::Default,
        "community-files-upload",
    );
    Ok(html! {
        <AppShell active="communities">
            <main
                class="ambient-strong scroll-thin relative flex min-h-0 flex-1 flex-col overflow-y-auto rounded-tl-xl border-l border-t border-line"
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
                <div
                    class="community-workspace"
                    data-events={eventsView}
                    hidden={communities.length === 0}
                >
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
                    <section
                        class="community-detail scroll-thin"
                        aria-label="Selected community"
                        hidden={!!eventsView}
                        pp-style={"--community-header:" + headerHeight + "px"}
                    >
                        <div
                            class="community-cover"
                            data-cover={activeCommunity?.cover || "mint"}
                            hidden={activeTab !== "posts" || !!manageTab}
                        >
                            <img
                                class="community-cover-image"
                                src={bannerSource(activeCommunity?.cover, activeCommunity?.cover_image)}
                                alt=""
                            />
                            <button
                                class="community-cover-edit"
                                aria-label="Edit banner"
                                title="Edit banner"
                                onclick={openBannerDialog()}
                                hidden={!activeCommunity?.is_owner}
                            >
                                <Icon name="edit" class="size-5" />
                            </button>
                        </div>
                        <header class="community-detail-header" pp-ref={detailHeader}>
                            <div class="community-identity-title">
                                <span class="community-small-avatar">
                                    <img
                                        class="community-saved-picture"
                                        src={activeCommunity?.picture || "/images/upurnaka-logo.png"}
                                        alt=""
                                        hidden={!activeCommunity?.picture || (!!activeChannel && !manageTab)}
                                    />
                                    <img
                                        class="channel-header-emoji"
                                        src={channelEmojiImage(activeChannel?.icon) || "/favicon.ico"}
                                        alt=""
                                        hidden={!activeChannel || !!manageTab || !channelEmojiImage(activeChannel.icon)}
                                    />
                                    <span
                                        hidden={activeChannel && !manageTab ? !!channelEmojiImage(activeChannel.icon) : !!activeCommunity?.picture}
                                    >
                                        {(manageTab ? "" : activeChannel?.icon) || activeCommunity?.avatar || "👥"}
                                    </span>
                                </span>
                                <h2>
                                    {(manageTab ? "" : activeChannel?.name) || activeCommunity?.name || "Community"}
                                </h2>
                            </div>
                            <div
                                class="community-tabs"
                                role="tablist"
                                aria-label="Community content"
                                hidden={!!manageTab}
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
                            <div
                                class="community-tabs"
                                role="tablist"
                                aria-label="Manage community"
                                hidden={!manageTab}
                                onkeydown={handleManageTabKeys(event)}
                            >
                                <template pp-for="tab in manageTabs">
                                    <button
                                        key={tab.key}
                                        role="tab"
                                        type="button"
                                        value={tab.key}
                                        aria-selected={manageTab === tab.key}
                                        aria-controls="community-manage"
                                        tabindex={manageTab === tab.key ? 0 : -1}
                                        onclick={showManageTab(event.currentTarget.value)}
                                    >
                                        {tab.label}
                                    </button>
                                </template>
                            </div>
                            <div class="community-header-actions">
                                <button
                                    type="button"
                                    class="community-event-link"
                                    onclick={openEvents()}
                                >
                                    <Icon name="calendar" class="size-4" />
                                    "Events"
                                </button>
                                <div
                                    class="community-header-group"
                                    role="group"
                                    aria-label="Community actions"
                                >
                                    <button
                                        type="button"
                                        aria-label="Add people"
                                        title="Add people"
                                        hidden={!activeCommunity?.is_owner}
                                        onclick={openAction("invite", selectedCommunity)}
                                    >
                                        <Icon name="person-add" class="size-5" />
                                    </button>
                                    <button
                                        type="button"
                                        aria-label="Copy link"
                                        title="Copy link"
                                        hidden={!activeCommunity?.can_share_link}
                                        onclick={openAction("share", selectedCommunity)}
                                    >
                                        <Icon name="link" class="size-5" />
                                    </button>
                                    <button
                                        type="button"
                                        aria-label="Community details and members"
                                        title="Manage community"
                                        aria-pressed={!!manageTab}
                                        onclick={toggleManage()}
                                    >
                                        <Icon name="settings" class="size-5" />
                                    </button>
                                </div>
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
                            hidden={activeTab !== "posts" || !!manageTab}
                        >
                            <form
                                class="community-composer"
                                data-mode={composeMode || "collapsed"}
                                onsubmit={publishPost(event)}
                                onkeydown={handleComposerKey(event)}
                            >
                                <div class="community-composer-collapsed" hidden={!!composeMode}>
                                    <button
                                        type="button"
                                        class="community-composer-heading"
                                        onclick={openComposer("post")}
                                    >
                                        <span class="community-author-avatar">
                                            @{initials.as_str()}
                                        </span>
                                        "Post in channel"
                                    </button>
                                    <div class="community-compose-modes">
                                        <button
                                            type="button"
                                            class="community-compose-mode"
                                            data-primary="true"
                                            onclick={openComposer("post")}
                                        >
                                            <Icon name="compose" class="size-4" />
                                            "Post"
                                        </button>
                                        <button
                                            type="button"
                                            class="community-compose-mode"
                                            onclick={openComposer("announcement")}
                                        >
                                            <Icon name="announcement" class="size-4" />
                                            "Announcement"
                                        </button>
                                    </div>
                                </div>
                                <div class="community-composer-expanded" hidden={!composeMode}>
                                    <header class="community-composer-author">
                                        <span class="community-author-avatar">
                                            @{initials.as_str()}
                                        </span>
                                        <strong>@{author.as_str()}</strong>
                                        <span
                                            class="community-composer-editing"
                                            hidden={!editingPostId}
                                        >
                                            "Editing post"
                                        </span>
                                        <button
                                            type="button"
                                            class="community-composer-close"
                                            aria-label="Discard post"
                                            title="Discard post"
                                            onclick={closeComposer()}
                                            disabled={working}
                                        >
                                            <Icon name="close" class="size-5" />
                                        </button>
                                    </header>
                                    <input
                                        class="community-compose-line"
                                        aria-label="Subject"
                                        placeholder="Add a subject"
                                        maxlength="200"
                                        autocomplete="off"
                                        value={postSubject}
                                        oninput={setPostSubject(target.value)}
                                        hidden={composeMode !== "post"}
                                        disabled={working}
                                        pp-ref={subjectInput}
                                    />
                                    <div
                                        class="community-announcement-banner relative mx-[-8px] mt-3 grid h-[clamp(160px,22vw,244px)] place-items-center rounded bg-banner px-16"
                                        data-accent={postAccent}
                                        hidden={composeMode !== "announcement"}
                                    >
                                        <input
                                            aria-label="Headline"
                                            placeholder="Type a headline"
                                            maxlength="100"
                                            autocomplete="off"
                                            value={postHeadline}
                                            class="w-full bg-transparent text-center text-[clamp(24px,3.4vw,36px)] font-bold text-white outline-none placeholder:text-white/70"
                                            oninput={setPostHeadline(target.value)}
                                            disabled={working}
                                            pp-ref={headlineInput}
                                        />
                                        <button
                                            type="button"
                                            class="community-accent-button absolute bottom-3 right-3 grid size-[34px] place-items-center rounded bg-surface"
                                            aria-label="Change announcement color"
                                            title="Change announcement color"
                                            aria-expanded={accentOpen}
                                            onclick={setAccentOpen(!accentOpen)}
                                            disabled={working}
                                        >
                                            <span
                                                class="size-4 rounded-[2px] bg-banner outline outline-white/40"
                                                data-accent={postAccent}
                                            >
                                            </span>
                                        </button>
                                        <div
                                            class="community-accent-palette absolute bottom-[54px] right-3 z-5 grid grid-cols-[repeat(4,28px)] gap-2 rounded-md bg-popover p-2.5 shadow-xl shadow-black/50"
                                            role="radiogroup"
                                            aria-label="Announcement color"
                                            hidden={!accentOpen}
                                        >
                                            <template pp-for="color in accents">
                                                <button
                                                    key={color}
                                                    type="button"
                                                    role="radio"
                                                    class="size-7 rounded bg-banner aria-checked:outline-2 aria-checked:outline-offset-2 aria-checked:outline-ink"
                                                    data-accent={color}
                                                    value={color}
                                                    aria-label={color}
                                                    title={color}
                                                    aria-checked={postAccent === color}
                                                    onclick={chooseAccent(event)}
                                                >
                                                </button>
                                            </template>
                                        </div>
                                    </div>
                                    <input
                                        class="community-compose-line"
                                        aria-label="Subheading"
                                        placeholder="Add a subheading"
                                        maxlength="200"
                                        autocomplete="off"
                                        value={postSubheading}
                                        oninput={setPostSubheading(target.value)}
                                        hidden={composeMode !== "announcement"}
                                        disabled={working}
                                    />
                                    <div
                                        class="community-format-toolbar"
                                        role="toolbar"
                                        aria-label="Formatting options"
                                    >
                                        <button
                                            type="button"
                                            aria-label="Bold"
                                            title="Bold (Ctrl+B)"
                                            aria-pressed={!!activeFormats.strong}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("strong")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="bold" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Italic"
                                            title="Italic (Ctrl+I)"
                                            aria-pressed={!!activeFormats.em}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("em")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="italic" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Underline"
                                            title="Underline (Ctrl+U)"
                                            aria-pressed={!!activeFormats.underline}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("underline")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="underline" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Strikethrough"
                                            title="Strikethrough"
                                            aria-pressed={!!activeFormats.strike}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("strike")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="strikethrough" class="size-4" />
                                        </button>
                                        <span class="community-toolbar-divider" aria-hidden="true">
                                        </span>
                                        <button
                                            type="button"
                                            aria-label="Bulleted list"
                                            title="Bulleted list"
                                            aria-pressed={!!activeFormats.bullet_list}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("bullet_list")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="list" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Numbered list"
                                            title="Numbered list"
                                            aria-pressed={!!activeFormats.ordered_list}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("ordered_list")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="list-ordered" class="size-4" />
                                        </button>
                                        <span class="community-toolbar-divider" aria-hidden="true">
                                        </span>
                                        <button
                                            type="button"
                                            aria-label="Highlight"
                                            title="Highlight"
                                            aria-expanded={stylePanel === "highlight"}
                                            onmousedown={event.preventDefault()}
                                            onclick={toggleStylePanel("highlight")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="highlight" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Font color"
                                            title="Font color"
                                            aria-expanded={stylePanel === "color"}
                                            onmousedown={event.preventDefault()}
                                            onclick={toggleStylePanel("color")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="font-color" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Font size"
                                            title="Font size"
                                            aria-expanded={stylePanel === "size"}
                                            onmousedown={event.preventDefault()}
                                            onclick={toggleStylePanel("size")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="font-size" class="size-4" />
                                        </button>
                                        <span class="community-toolbar-divider" aria-hidden="true">
                                        </span>
                                        <button
                                            type="button"
                                            aria-label="Quote"
                                            title="Quote"
                                            aria-pressed={!!activeFormats.blockquote}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("blockquote")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="quote" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Insert link"
                                            title="Insert link"
                                            aria-expanded={stylePanel === "link"}
                                            onmousedown={event.preventDefault()}
                                            onclick={openLink()}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="link" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Code snippet"
                                            title="Code snippet"
                                            aria-pressed={!!activeFormats.code_block}
                                            onmousedown={event.preventDefault()}
                                            onclick={formatText("code_block")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="code" class="size-4" />
                                        </button>
                                        <span class="community-toolbar-divider" aria-hidden="true">
                                        </span>
                                        <button
                                            type="button"
                                            aria-label="Announcement"
                                            title="Announcement"
                                            aria-pressed={composeMode === "announcement"}
                                            onmousedown={event.preventDefault()}
                                            onclick={toggleAnnouncement()}
                                            disabled={working || !!editingPostId}
                                        >
                                            <Icon name="announcement" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="More options"
                                            title="More options"
                                            aria-expanded={stylePanel === "more"}
                                            onmousedown={event.preventDefault()}
                                            onclick={toggleStylePanel("more")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="more" class="size-4" />
                                        </button>
                                    </div>
                                    <div class="community-style-panel" hidden={!stylePanel}>
                                        <div
                                            hidden={stylePanel !== "highlight" && stylePanel !== "color"}
                                        >
                                            <template pp-for="swatch in shownSwatches">
                                                <button
                                                    key={swatch.value}
                                                    type="button"
                                                    class="community-swatch"
                                                    value={swatch.value}
                                                    aria-label={swatch.label}
                                                    title={swatch.label}
                                                    pp-style={"background:" + swatch.value}
                                                    onmousedown={event.preventDefault()}
                                                    onclick={applySwatch(event)}
                                                >
                                                </button>
                                            </template>
                                        </div>
                                        <div hidden={stylePanel !== "size"}>
                                            <button
                                                type="button"
                                                class="community-style-option"
                                                onmousedown={event.preventDefault()}
                                                onclick={applyStyle("size", "small")}
                                            >
                                                "Small"
                                            </button>
                                            <button
                                                type="button"
                                                class="community-style-option"
                                                onmousedown={event.preventDefault()}
                                                onclick={applyStyle("size", "normal")}
                                            >
                                                "Medium"
                                            </button>
                                            <button
                                                type="button"
                                                class="community-style-option"
                                                onmousedown={event.preventDefault()}
                                                onclick={applyStyle("size", "large")}
                                            >
                                                "Large"
                                            </button>
                                        </div>
                                        <div hidden={stylePanel !== "link"}>
                                            <input
                                                type="url"
                                                aria-label="Link address"
                                                placeholder="https://example.com"
                                                value={linkUrl}
                                                oninput={setLinkUrl(target.value)}
                                                onkeydown={handleLinkKey(event)}
                                                pp-ref={linkInput}
                                            />
                                            <button
                                                type="button"
                                                class="community-style-option"
                                                onclick={saveLink()}
                                            >
                                                "Insert"
                                            </button>
                                        </div>
                                        <div hidden={stylePanel !== "more"}>
                                            <button
                                                type="button"
                                                class="community-style-option"
                                                onmousedown={event.preventDefault()}
                                                onclick={formatText("clear")}
                                            >
                                                "Clear formatting"
                                            </button>
                                        </div>
                                        <div
                                            class="community-compose-emoji"
                                            hidden={stylePanel !== "emoji"}
                                        >
                                            <FluentEmojiGrid
                                                items={composeEmojis}
                                                select={insertEmoji}
                                            />
                                        </div>
                                    </div>
                                    <div class="community-rich-host">
                                        <div pp-ref={editorHost}></div>
                                        <p hidden={editorReady}>"Loading editor…"</p>
                                    </div>
                                    <p
                                        class="community-compose-error"
                                        role="alert"
                                        hidden={!composeError}
                                    >
                                        {composeError}
                                    </p>
                                    <footer class="community-composer-footer">
                                        <button
                                            type="button"
                                            aria-label="Emoji"
                                            title="Emoji"
                                            aria-expanded={stylePanel === "emoji"}
                                            onmousedown={event.preventDefault()}
                                            onclick={toggleStylePanel("emoji")}
                                            disabled={working || !editorReady}
                                        >
                                            <Icon name="emoji" class="size-5" />
                                        </button>
                                        <Button attrs=@{submit()} disabled={working || !canPost}>
                                            {working ? (editingPostId ? "Saving…" : "Posting…") : editingPostId ? "Save" : "Post"}
                                        </Button>
                                    </footer>
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
                                <article
                                    key={post.id}
                                    class="community-post"
                                    data-post={post.id}
                                    data-focused={focusedPost === post.id ? "true" : "false"}
                                    data-deleted={post.deleted ? "true" : "false"}
                                >
                                    <div
                                        class="community-post-actions"
                                        role="toolbar"
                                        aria-label="Post actions"
                                        hidden={post.deleted}
                                    >
                                        <template pp-for="emoji in reactionChoices">
                                            <button
                                                key={emoji}
                                                type="button"
                                                class="community-quick-reaction"
                                                aria-label={"React with " + (reactionNames[emoji] || emoji)}
                                                title={reactionNames[emoji] || emoji}
                                                onclick={toggleReaction(post.id, emoji)}
                                                disabled={reacting}
                                            >
                                                <img
                                                    src={emojiImage(emoji)}
                                                    alt=""
                                                    width="22"
                                                    height="22"
                                                />
                                            </button>
                                        </template>
                                        <button
                                            type="button"
                                            aria-label="More reactions"
                                            title="More reactions"
                                            aria-haspopup="dialog"
                                            aria-expanded={reactionPanelOpen && reactionFromToolbar && reactionPostId === post.id}
                                            popovertarget="community-reaction-picker"
                                            onclick={prepareReactionPicker(post.id, event, true)}
                                        >
                                            <Icon name="emoji-add" class="size-5" />
                                        </button>
                                        <span class="community-toolbar-divider" aria-hidden="true">
                                        </span>
                                        <button
                                            type="button"
                                            aria-label="Edit"
                                            title="Edit"
                                            hidden={!post.is_mine}
                                            onclick={startEdit(post.id)}
                                        >
                                            <Icon name="edit" class="size-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="More options"
                                            title="More options"
                                            aria-haspopup="menu"
                                            aria-controls="community-post-menu"
                                            aria-expanded={postMenuOpen && menuPostId === post.id}
                                            popovertarget="community-post-menu"
                                            onclick={preparePostMenu(post.id, event)}
                                        >
                                            <Icon name="more" class="size-4" />
                                        </button>
                                    </div>
                                    <header class="community-post-author">
                                        <span class="community-author-avatar">
                                            {authorInitials(post.author_name)}
                                        </span>
                                        <strong>{post.author_name}</strong>
                                        <span
                                            class="community-owner-badge"
                                            hidden={!post.author_is_owner}
                                        >
                                            <Icon name="shield" class="size-3" />
                                            "Owner"
                                        </span>
                                        <time title={new Date(post.created_at).toLocaleString()}>
                                            {postTime(post.created_at)}
                                        </time>
                                        <span class="community-post-edited" hidden={!post.edited}>
                                            "Edited"
                                        </span>
                                        <span class="community-post-pinned" hidden={!post.pinned}>
                                            <Icon name="pin" class="size-3" />
                                            "Pinned"
                                        </span>
                                    </header>
                                    <p class="community-post-deleted" hidden={!post.deleted}>
                                        "This post has been deleted."
                                        <button
                                            type="button"
                                            hidden={!canDelete(post)}
                                            onclick={restorePost(post.id)}
                                            disabled={working}
                                        >
                                            "Undo"
                                        </button>
                                    </p>
                                    <div
                                        class="community-post-banner mx-[18px] mt-4 grid min-h-[clamp(150px,22vw,244px)] place-items-center rounded bg-banner p-6 text-center"
                                        data-accent={post.accent || "blue"}
                                        hidden={!post.announcement}
                                    >
                                        <h3>{post.headline || "Announcement"}</h3>
                                    </div>
                                    <div
                                        class="community-post-subheading"
                                        hidden={!post.subheading}
                                    >
                                        {post.subheading}
                                    </div>
                                    <h3 class="community-post-subject" hidden={!post.subject}>
                                        {post.subject}
                                    </h3>
                                    <p hidden={!!post.formatting || !post.body}>{post.body}</p>
                                    <div class="community-rich-message" hidden={!post.formatting}>
                                        <template
                                            pp-for="(line, lineIndex) in post.formatting || []"
                                        >
                                            <div key={lineIndex} class={postLineClass(line.kind)}>
                                                <span
                                                    hidden={line.kind !== "bullet"}
                                                    aria-hidden="true"
                                                >
                                                    "• "
                                                </span>
                                                <span
                                                    hidden={line.kind !== "ordered"}
                                                    aria-hidden="true"
                                                >
                                                    {postListNumber(post.formatting, lineIndex)}
                                                    ". "
                                                </span>
                                                <template pp-for="(run, runIndex) in line.runs">
                                                    <span
                                                        key={runIndex}
                                                        class={postRunClass(run)}
                                                        pp-style={postRunStyle(run)}
                                                    >
                                                        <a
                                                            href={run.link || "#"}
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                            hidden={!run.link}
                                                        >
                                                            {run.text}
                                                        </a>
                                                        <span hidden={!!run.link}>{run.text}</span>
                                                    </span>
                                                </template>
                                                <br hidden={line.runs.length > 0} />
                                            </div>
                                        </template>
                                    </div>
                                    <div class="community-post-files" hidden={!post.files.length}>
                                        <template pp-for="file in post.files">
                                            <a
                                                key={file.id}
                                                class="community-file-card"
                                                href={"/communities/files/" + file.id}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                            >
                                                <span
                                                    class="community-file-icon file-kind grid h-9 w-8 shrink-0 place-items-center rounded text-[9px] font-bold tracking-[.02em]"
                                                    data-kind={fileKind(file)}
                                                >
                                                    {fileBadge(file)}
                                                </span>
                                                <span>
                                                    <strong>{file.name}</strong>
                                                    <small>{fileSize(file.bytes)}</small>
                                                </span>
                                            </a>
                                        </template>
                                    </div>
                                    <div class="community-reactions" hidden={post.deleted}>
                                        <template pp-for="reaction in post.reactions">
                                            <button
                                                key={reaction.emoji}
                                                type="button"
                                                class="community-reaction"
                                                aria-pressed={reaction.mine}
                                                aria-label={reactionLabel(reaction)}
                                                title={reactionLabel(reaction)}
                                                onclick={toggleReaction(post.id, reaction.emoji)}
                                                disabled={reacting}
                                            >
                                                <img
                                                    src={emojiImage(reaction.emoji)}
                                                    alt=""
                                                    width="18"
                                                    height="18"
                                                />
                                                {reaction.count}
                                            </button>
                                        </template>
                                        <button
                                            type="button"
                                            class="community-reaction-add"
                                            aria-label="Add reaction"
                                            title="Add reaction"
                                            aria-haspopup="dialog"
                                            popovertarget="community-reaction-picker"
                                            onclick={prepareReactionPicker(post.id, event)}
                                        >
                                            <Icon name="emoji-add" class="size-4" />
                                        </button>
                                    </div>
                                    <button
                                        type="button"
                                        class="community-conversation-toggle"
                                        aria-expanded={!!openConversations[post.id]}
                                        onclick={toggleConversation(post.id)}
                                    >
                                        <Icon name="chat" class="size-4" />
                                        {conversationLabel(post)}
                                    </button>
                                    <ol
                                        class="community-replies"
                                        aria-label="Replies"
                                        hidden={!openConversations[post.id]}
                                    >
                                        <li
                                            class="community-replies-empty"
                                            hidden={post.replies.length > 0}
                                        >
                                            "No replies yet. Start the conversation."
                                        </li>
                                        <template pp-for="reply in post.replies">
                                            <li key={reply.id} class="community-reply">
                                                <span class="community-author-avatar">
                                                    {authorInitials(reply.author_name)}
                                                </span>
                                                <div>
                                                    <p class="community-reply-meta">
                                                        <strong>{reply.author_name}</strong>
                                                        <span
                                                            class="community-owner-badge"
                                                            hidden={!reply.author_is_owner}
                                                        >
                                                            <Icon name="shield" class="size-3" />
                                                            "Owner"
                                                        </span>
                                                        <time
                                                            title={new Date(reply.created_at).toLocaleString()}
                                                        >
                                                            {postTime(reply.created_at)}
                                                        </time>
                                                    </p>
                                                    <p class="community-reply-body">{reply.body}</p>
                                                </div>
                                            </li>
                                        </template>
                                    </ol>
                                    <button
                                        type="button"
                                        class="community-reply-open"
                                        hidden={replyingTo === post.id}
                                        onclick={startReply(post.id, event)}
                                    >
                                        <span class="community-author-avatar">
                                            @{initials.as_str()}
                                        </span>
                                        "Reply"
                                    </button>
                                    <form
                                        class="community-reply-form"
                                        hidden={replyingTo !== post.id}
                                        onsubmit={sendReply(post.id, event)}
                                    >
                                        <span class="community-author-avatar">
                                            @{initials.as_str()}
                                        </span>
                                        <textarea
                                            aria-label="Reply"
                                            placeholder="Reply"
                                            maxlength="2000"
                                            rows="1"
                                            value={replyingTo === post.id ? replyDraft : ""}
                                            oninput={setReplyDraft(target.value)}
                                            onkeydown={replyKey(post.id, event)}
                                            disabled={replying}
                                        >
                                        </textarea>
                                        <button
                                            type="submit"
                                            aria-label="Send reply"
                                            title="Send reply"
                                            disabled={replying || !replyDraft.trim()}
                                        >
                                            <Icon name="send" class="size-4" />
                                        </button>
                                    </form>
                                    <p
                                        class="community-reply-error"
                                        role="alert"
                                        hidden={replyError.post !== post.id}
                                    >
                                        {replyError.message}
                                    </p>
                                </article>
                            </template>
                        </div>
                        <div
                            id="community-files"
                            aria-labelledby="community-tab-files"
                            role="tabpanel"
                            class="community-files"
                            hidden={activeTab !== "files" || !!manageTab}
                        >
                            <input
                                type="file"
                                multiple
                                class="sr-only"
                                tabindex="-1"
                                aria-hidden="true"
                                pp-ref={fileInput}
                                onchange={uploadFiles(event)}
                            />
                            <section class="community-files-empty" hidden={activeFiles.length > 0}>
                                <img
                                    src="/images/community-files-empty.svg"
                                    alt=""
                                    width="184"
                                    height="184"
                                />
                                <h3>"Share files in this community"</h3>
                                <p>
                                    "When you upload files to this tab, they’ll show up in posts."
                                </p>
                                <DropdownMenu>
                                    <DropdownMenuTrigger
                                        menu="community-upload"
                                        class=@{upload_button.as_str()}
                                        disabled={uploading}
                                    >
                                        <Icon name="upload" class="size-4" />
                                        {uploading ? "Uploading…" : "Upload"}
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent
                                        menu="community-upload"
                                        side="bottom"
                                        align="start"
                                        class="community-upload-menu w-60"
                                        attrs=@{named("Upload")}
                                    >
                                        <DropdownMenuItem onclick={openCloudFiles()}>
                                            <Icon name="cloud" class="size-4" />
                                            "Attach cloud files"
                                        </DropdownMenuItem>
                                        <DropdownMenuItem onclick={pickFiles()}>
                                            <Icon name="upload" class="size-4" />
                                            "Upload from this device"
                                        </DropdownMenuItem>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </section>
                            <section class="community-files-list" hidden={activeFiles.length === 0}>
                                <div class="community-files-toolbar">
                                    <DropdownMenu>
                                        <DropdownMenuTrigger
                                            menu="community-upload-more"
                                            class=@{upload_button.as_str()}
                                            disabled={uploading}
                                        >
                                            <Icon name="upload" class="size-4" />
                                            {uploading ? "Uploading…" : "Upload"}
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent
                                            menu="community-upload-more"
                                            side="bottom"
                                            align="start"
                                            class="community-upload-menu w-60"
                                            attrs=@{named("Upload")}
                                        >
                                            <DropdownMenuItem onclick={openCloudFiles()}>
                                                <Icon name="cloud" class="size-4" />
                                                "Attach cloud files"
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onclick={pickFiles()}>
                                                <Icon name="upload" class="size-4" />
                                                "Upload from this device"
                                            </DropdownMenuItem>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>
                                <table>
                                    <thead>
                                        <tr>
                                            <th scope="col">"Name"</th>
                                            <th scope="col">"Modified"</th>
                                            <th scope="col">"Modified by"</th>
                                            <th scope="col">"Size"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <template pp-for="file in activeFiles">
                                            <tr key={file.id}>
                                                <td>
                                                    <a
                                                        class="community-file-name"
                                                        href={"/communities/files/" + file.id}
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                    >
                                                        <span
                                                            class="community-file-icon file-kind grid h-9 w-8 shrink-0 place-items-center rounded text-[9px] font-bold tracking-[.02em]"
                                                            data-kind={fileKind(file)}
                                                        >
                                                            {fileBadge(file)}
                                                        </span>
                                                        {file.name}
                                                    </a>
                                                </td>
                                                <td>
                                                    <time
                                                        title={new Date(file.created_at).toLocaleString()}
                                                    >
                                                        {fileDate(file.created_at)}
                                                    </time>
                                                </td>
                                                <td>{file.uploader_name}</td>
                                                <td>{fileSize(file.bytes)}</td>
                                            </tr>
                                        </template>
                                    </tbody>
                                </table>
                            </section>
                            <p class="community-files-status" role="status" hidden={!uploadStatus}>
                                {uploadStatus}
                            </p>
                            <p class="community-files-error" role="alert" hidden={!fileError}>
                                {fileError}
                            </p>
                        </div>
                        <div
                            id="community-photos"
                            aria-labelledby="community-tab-photos"
                            role="tabpanel"
                            class="community-photos"
                            hidden={activeTab !== "photos" || !!manageTab}
                        >
                            <section
                                class="community-photos-empty"
                                hidden={activePhotos.length > 0}
                            >
                                <img
                                    src="/images/community-photos-empty.svg"
                                    alt=""
                                    width="184"
                                    height="184"
                                />
                                <h3>"No photos shared in the community"</h3>
                                <p>"Photos added to community automatically show up here."</p>
                            </section>
                            <section
                                class="community-photos-grid"
                                aria-label="Photos"
                                hidden={activePhotos.length === 0}
                            >
                                <template pp-for="photo in activePhotos">
                                    <a
                                        key={photo.id}
                                        class="community-photo"
                                        href={"/communities/files/" + photo.id}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        title={photo.name}
                                    >
                                        <img
                                            src={"/communities/files/" + photo.id}
                                            alt={photo.name}
                                            loading="lazy"
                                        />
                                        <span class="community-photo-caption">
                                            <strong>{photo.name}</strong>
                                            <small>
                                                {photo.uploader_name + " · " + fileDate(photo.created_at)}
                                            </small>
                                        </span>
                                    </a>
                                </template>
                            </section>
                        </div>
                        <div
                            id="community-manage"
                            role="tabpanel"
                            class="community-manage"
                            aria-label={manageTabLabel}
                            hidden={!manageTab}
                        >
                            <p class="community-manage-error" role="alert" hidden={!manageError}>
                                {manageError}
                            </p>
                            <p class="community-manage-notice" role="status" hidden={!manageNotice}>
                                {manageNotice}
                            </p>
                            <p
                                class="community-manage-notice"
                                role="status"
                                hidden={!manageLoading}
                            >
                                "Loading members…"
                            </p>
                            <section
                                class="community-manage-members"
                                hidden={manageTab !== "members"}
                            >
                                <div class="community-manage-confirm" hidden={!manageTransfer}>
                                    <p>
                                        {"Make " + manageTransferName + " the owner? You'll become a member and lose owner controls."}
                                    </p>
                                    <button
                                        type="button"
                                        class="community-manage-primary"
                                        disabled={manageWorking}
                                        onclick={confirmManageTransfer()}
                                    >
                                        "Make owner"
                                    </button>
                                    <button
                                        type="button"
                                        class="community-manage-secondary"
                                        onclick={setManageTransfer("")}
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                                <button
                                    type="button"
                                    class="community-manage-group"
                                    aria-expanded={ownersOpen}
                                    onclick={setOwnersOpen(!ownersOpen)}
                                >
                                    <Icon name="chevron-down" class="size-4" />
                                    {"Owners (" + managedOwners.length + ")"}
                                </button>
                                <div class="community-manage-table" hidden={!ownersOpen}>
                                    <div class="community-manage-head" aria-hidden="true">
                                        <span>"Name"</span>
                                        <span>"Role"</span>
                                    </div>
                                    <template pp-for="person in managedOwners">
                                        <div key={person.id} class="community-manage-row">
                                            <div class="community-manage-person">
                                                <span
                                                    class="community-manage-avatar"
                                                    aria-hidden="true"
                                                >
                                                    {personInitials(person)}
                                                </span>
                                                <span class="community-manage-name">
                                                    <strong>
                                                        {person.nickname || person.name}
                                                    </strong>
                                                    <small hidden={!person.nickname}>
                                                        {person.name}
                                                    </small>
                                                </span>
                                                <button
                                                    type="button"
                                                    class="community-manage-nickname"
                                                    hidden={!person.is_me || nicknameEditing}
                                                    onclick={startNickname()}
                                                >
                                                    <Icon name="edit" class="size-4" />
                                                    {person.nickname ? "Change nickname" : "Set nickname"}
                                                </button>
                                                <form
                                                    class="community-manage-nickname-form"
                                                    hidden={!person.is_me || !nicknameEditing}
                                                    onsubmit={saveNickname(event)}
                                                >
                                                    <input
                                                        name="nickname"
                                                        maxlength="60"
                                                        autocomplete="off"
                                                        aria-label="Nickname"
                                                        placeholder="Nickname"
                                                        value={nicknameDraft}
                                                        oninput={setNicknameDraft(target.value)}
                                                        onkeydown={nicknameKey(event)}
                                                    />
                                                    <button
                                                        type="submit"
                                                        class="community-manage-primary"
                                                        disabled={manageWorking}
                                                    >
                                                        "Save"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="community-manage-secondary"
                                                        onclick={setNicknameEditing(false)}
                                                    >
                                                        "Cancel"
                                                    </button>
                                                </form>
                                            </div>
                                            <span class="community-manage-role">"Owner"</span>
                                        </div>
                                    </template>
                                </div>
                                <button
                                    type="button"
                                    class="community-manage-group"
                                    aria-expanded={membersOpen}
                                    onclick={setMembersOpen(!membersOpen)}
                                >
                                    <Icon name="chevron-down" class="size-4" />
                                    {"Members (" + managedMembers.length + ")"}
                                </button>
                                <div
                                    class="community-manage-table"
                                    hidden={!membersOpen || managedMembers.length === 0}
                                >
                                    <div class="community-manage-head" aria-hidden="true">
                                        <span>"Name"</span>
                                        <span>"Role"</span>
                                    </div>
                                    <template pp-for="person in managedMembers">
                                        <div key={person.id} class="community-manage-row">
                                            <div class="community-manage-person">
                                                <span
                                                    class="community-manage-avatar"
                                                    aria-hidden="true"
                                                >
                                                    {personInitials(person)}
                                                </span>
                                                <span class="community-manage-name">
                                                    <strong>
                                                        {person.nickname || person.name}
                                                    </strong>
                                                    <small
                                                        hidden={!person.nickname && !person.email}
                                                    >
                                                        {person.nickname ? person.name : person.email}
                                                    </small>
                                                </span>
                                                <button
                                                    type="button"
                                                    class="community-manage-nickname"
                                                    hidden={!person.is_me || nicknameEditing}
                                                    onclick={startNickname()}
                                                >
                                                    <Icon name="edit" class="size-4" />
                                                    {person.nickname ? "Change nickname" : "Set nickname"}
                                                </button>
                                                <form
                                                    class="community-manage-nickname-form"
                                                    hidden={!person.is_me || !nicknameEditing}
                                                    onsubmit={saveNickname(event)}
                                                >
                                                    <input
                                                        name="nickname"
                                                        maxlength="60"
                                                        autocomplete="off"
                                                        aria-label="Nickname"
                                                        placeholder="Nickname"
                                                        value={nicknameDraft}
                                                        oninput={setNicknameDraft(target.value)}
                                                        onkeydown={nicknameKey(event)}
                                                    />
                                                    <button
                                                        type="submit"
                                                        class="community-manage-primary"
                                                        disabled={manageWorking}
                                                    >
                                                        "Save"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="community-manage-secondary"
                                                        onclick={setNicknameEditing(false)}
                                                    >
                                                        "Cancel"
                                                    </button>
                                                </form>
                                            </div>
                                            <div class="community-manage-role">
                                                <select
                                                    aria-label={"Role for " + person.name}
                                                    hidden={!activeCommunity?.is_owner}
                                                    data-person={person.id}
                                                    disabled={manageWorking}
                                                    onchange={chooseRole(event)}
                                                >
                                                    <option value="member" selected>
                                                        "Member"
                                                    </option>
                                                    <option value="owner">"Owner"</option>
                                                </select>
                                                <span hidden={!!activeCommunity?.is_owner}>
                                                    "Member"
                                                </span>
                                                <button
                                                    type="button"
                                                    class="community-manage-secondary"
                                                    hidden={!activeCommunity?.is_owner}
                                                    value={person.id}
                                                    disabled={manageWorking}
                                                    onclick={removeManaged(event.currentTarget.value)}
                                                >
                                                    "Remove"
                                                </button>
                                                <button
                                                    type="button"
                                                    class="community-manage-secondary"
                                                    hidden={!activeCommunity?.is_owner}
                                                    value={person.id}
                                                    disabled={manageWorking}
                                                    onclick={blockManaged(event.currentTarget.value)}
                                                >
                                                    "Block"
                                                </button>
                                            </div>
                                        </div>
                                    </template>
                                </div>
                            </section>
                            <section
                                class="community-manage-pending"
                                hidden={manageTab !== "pending"}
                            >
                                <div
                                    class="community-manage-empty"
                                    hidden={manageLoading || managedPending.length > 0}
                                >
                                    <img
                                        src="/images/community-pending-empty.svg"
                                        alt=""
                                        width="300"
                                        height="300"
                                    />
                                    <h3>"No pending members"</h3>
                                    <p>
                                        "Invite members to your community. Pending invite and join requests will show up here."
                                    </p>
                                    <button
                                        type="button"
                                        class="community-manage-primary community-manage-wide"
                                        onclick={openAction("invite", selectedCommunity)}
                                    >
                                        "Invite members"
                                    </button>
                                </div>
                                <div hidden={managedPending.length === 0}>
                                    <div class="community-manage-toolbar">
                                        <button
                                            type="button"
                                            class="community-manage-primary"
                                            onclick={openAction("invite", selectedCommunity)}
                                        >
                                            <Icon name="person-add" class="size-4" />
                                            "Invite members"
                                        </button>
                                    </div>
                                    <div class="community-manage-table">
                                        <div class="community-manage-head" aria-hidden="true">
                                            <span>"Name"</span>
                                            <span>"Status"</span>
                                        </div>
                                        <template pp-for="person in managedPending">
                                            <div key={person.id} class="community-manage-row">
                                                <div class="community-manage-person">
                                                    <span
                                                        class="community-manage-avatar"
                                                        aria-hidden="true"
                                                    >
                                                        {personInitials(person)}
                                                    </span>
                                                    <span class="community-manage-name">
                                                        <strong>{person.name}</strong>
                                                        <small>{person.email}</small>
                                                    </span>
                                                </div>
                                                <div class="community-manage-role">
                                                    <span>
                                                        {person.state === "requested" ? "Requested to join" : "Invited"}
                                                    </span>
                                                    <button
                                                        type="button"
                                                        class="community-manage-primary"
                                                        hidden={person.state !== "requested"}
                                                        value={person.id}
                                                        disabled={manageWorking}
                                                        onclick={approveManaged(event.currentTarget.value)}
                                                    >
                                                        "Approve"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="community-manage-secondary"
                                                        value={person.id}
                                                        disabled={manageWorking}
                                                        onclick={declineManaged(event.currentTarget.value)}
                                                    >
                                                        {person.state === "requested" ? "Decline" : "Cancel invite"}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="community-manage-secondary"
                                                        hidden={person.state !== "requested"}
                                                        value={person.id}
                                                        disabled={manageWorking}
                                                        onclick={blockManaged(event.currentTarget.value)}
                                                    >
                                                        "Block"
                                                    </button>
                                                </div>
                                            </div>
                                        </template>
                                    </div>
                                </div>
                            </section>
                            <section
                                class="community-manage-blocked"
                                hidden={manageTab !== "blocked"}
                            >
                                <div
                                    class="community-manage-empty"
                                    hidden={manageLoading || managedBlocked.length > 0}
                                >
                                    <img
                                        src="/images/community-blocked-empty.svg"
                                        alt=""
                                        width="300"
                                        height="300"
                                    />
                                    <h3>"No blocked contacts"</h3>
                                    <p>
                                        "Blocked contacts will not be able to search for or join this community."
                                    </p>
                                </div>
                                <div
                                    class="community-manage-table"
                                    hidden={managedBlocked.length === 0}
                                >
                                    <div class="community-manage-head" aria-hidden="true">
                                        <span>"Name"</span>
                                        <span></span>
                                    </div>
                                    <template pp-for="person in managedBlocked">
                                        <div key={person.id} class="community-manage-row">
                                            <div class="community-manage-person">
                                                <span
                                                    class="community-manage-avatar"
                                                    aria-hidden="true"
                                                >
                                                    {personInitials(person)}
                                                </span>
                                                <span class="community-manage-name">
                                                    <strong>{person.name}</strong>
                                                    <small>{person.email}</small>
                                                </span>
                                            </div>
                                            <div class="community-manage-role">
                                                <button
                                                    type="button"
                                                    class="community-manage-secondary"
                                                    value={person.id}
                                                    disabled={manageWorking}
                                                    onclick={unblockManaged(event.currentTarget.value)}
                                                >
                                                    "Unblock"
                                                </button>
                                            </div>
                                        </div>
                                    </template>
                                </div>
                            </section>
                            <section
                                class="community-manage-settings"
                                hidden={manageTab !== "settings"}
                            >
                                <button
                                    type="button"
                                    class="community-manage-group"
                                    aria-expanded={linkSettingsOpen}
                                    onclick={setLinkSettingsOpen(!linkSettingsOpen)}
                                >
                                    <Icon name="chevron-down" class="size-4" />
                                    "Link settings"
                                </button>
                                <div class="community-manage-options" hidden={!linkSettingsOpen}>
                                    <div class="community-manage-option">
                                        <div>
                                            <h4>"Create new link"</h4>
                                            <p>"Existing community links will be disabled."</p>
                                        </div>
                                        <button
                                            type="button"
                                            class="community-manage-secondary community-manage-outline"
                                            disabled={manageWorking}
                                            onclick={createNewLink()}
                                        >
                                            "Create new link"
                                        </button>
                                    </div>
                                    <label class="community-manage-option">
                                        <div>
                                            <h4>
                                                "Allow members to share a link to this community"
                                            </h4>
                                            <p>
                                                "Members can invite others to the community via a share link."
                                            </p>
                                        </div>
                                        <span class="community-event-switch">
                                            <input
                                                type="checkbox"
                                                role="switch"
                                                aria-label="Allow members to share a link to this community"
                                                checked={!!activeCommunity?.members_can_share}
                                                disabled={manageWorking}
                                                onchange={saveCommunitySetting("members_can_share", event.currentTarget.checked)}
                                            />
                                            <span aria-hidden="true"></span>
                                        </span>
                                    </label>
                                </div>
                                <button
                                    type="button"
                                    class="community-manage-group"
                                    aria-expanded={joinSettingsOpen}
                                    onclick={setJoinSettingsOpen(!joinSettingsOpen)}
                                >
                                    <Icon name="chevron-down" class="size-4" />
                                    "Join settings"
                                </button>
                                <div class="community-manage-options" hidden={!joinSettingsOpen}>
                                    <label class="community-manage-option">
                                        <div>
                                            <h4>"Require approval to join"</h4>
                                            <p>
                                                "Community owners must approve new members before they can join."
                                            </p>
                                        </div>
                                        <span class="community-event-switch">
                                            <input
                                                type="checkbox"
                                                role="switch"
                                                aria-label="Require approval to join"
                                                checked={!!activeCommunity?.require_approval}
                                                disabled={manageWorking}
                                                onchange={saveCommunitySetting("require_approval", event.currentTarget.checked)}
                                            />
                                            <span aria-hidden="true"></span>
                                        </span>
                                    </label>
                                </div>
                            </section>
                        </div>
                    </section>
                    <section
                        class="community-events scroll-thin"
                        aria-label="Community events"
                        hidden={!eventsView}
                    >
                        <header class="community-events-header" hidden={eventsView !== "list"}>
                            <button
                                type="button"
                                class="community-events-back"
                                aria-label="Back to community"
                                title="Back"
                                onclick={closeEvents()}
                            >
                                <Icon name="arrow-right" class="size-5 rotate-180" />
                            </button>
                            <span class="community-small-avatar">
                                <img
                                    src={activeCommunity?.picture || "/images/upurnaka-logo.png"}
                                    alt=""
                                    hidden={!activeCommunity?.picture}
                                />
                                <span hidden={!!activeCommunity?.picture}>
                                    {activeCommunity?.avatar || "👥"}
                                </span>
                            </span>
                            <div class="community-events-title">
                                <h2>"Community events"</h2>
                                <p>{activeCommunity?.name}</p>
                            </div>
                            <button
                                type="button"
                                class="community-events-create"
                                data-outlined="true"
                                onclick={newEvent()}
                            >
                                <Icon name="plus" class="size-4" />
                                "Create event"
                            </button>
                        </header>
                        <div class="community-events-body" hidden={eventsView !== "list"}>
                            <p class="community-events-note" role="status" hidden={!eventsLoading}>
                                "Loading events…"
                            </p>
                            <p class="community-events-error" role="alert" hidden={!eventsError}>
                                {eventsError}
                            </p>
                            <p class="community-events-note" role="status" hidden={!eventNotice}>
                                {eventNotice}
                            </p>
                            <div
                                class="community-events-empty"
                                hidden={eventsLoading || !!eventsError || events.length > 0}
                            >
                                <img
                                    src="/images/community-events-empty.svg"
                                    alt=""
                                    width="200"
                                    height="200"
                                />
                                <h3>
                                    "Schedule new events and see all your upcoming events in one view."
                                </h3>
                                <button
                                    type="button"
                                    class="community-events-create"
                                    onclick={newEvent()}
                                >
                                    <Icon name="plus" class="size-4" />
                                    "Create event"
                                </button>
                            </div>
                            <ol class="community-events-list" hidden={events.length === 0}>
                                <template pp-for="item in events">
                                    <li key={item.id} class="community-event-card">
                                        <img
                                            class="community-event-banner"
                                            src={"/images/banners/" + item.banner + ".svg"}
                                            alt=""
                                        />
                                        <div class="community-event-date" aria-hidden="true">
                                            <small>{eventMonth(item)}</small>
                                            <strong>{eventDay(item)}</strong>
                                        </div>
                                        <div class="community-event-info">
                                            <h3>{item.title}</h3>
                                            <p class="community-event-when">
                                                <Icon name="clock" class="size-4" />
                                                {eventWhen(item)}
                                            </p>
                                            <p
                                                class="community-event-where"
                                                hidden={!item.location}
                                            >
                                                <Icon name="location" class="size-4" />
                                                {item.location}
                                            </p>
                                            <p
                                                class="community-event-where"
                                                hidden={!item.join_url}
                                            >
                                                <Icon name="video" class="size-4" />
                                                "Online event"
                                            </p>
                                            <p
                                                class="community-event-details"
                                                hidden={!item.details || !!item.formatting}
                                            >
                                                {item.details}
                                            </p>
                                            <div
                                                class="community-event-details community-rich-message"
                                                hidden={!item.formatting}
                                            >
                                                <template
                                                    pp-for="(line, lineIndex) in item.formatting || []"
                                                >
                                                    <div
                                                        key={lineIndex}
                                                        class={postLineClass(line.kind)}
                                                    >
                                                        <span
                                                            hidden={line.kind !== "bullet"}
                                                            aria-hidden="true"
                                                        >
                                                            "• "
                                                        </span>
                                                        <span
                                                            hidden={line.kind !== "ordered"}
                                                            aria-hidden="true"
                                                        >
                                                            {postListNumber(item.formatting, lineIndex)}
                                                            ". "
                                                        </span>
                                                        <template
                                                            pp-for="(run, runIndex) in line.runs"
                                                        >
                                                            <span
                                                                key={runIndex}
                                                                class={postRunClass(run)}
                                                                pp-style={postRunStyle(run)}
                                                            >
                                                                <a
                                                                    href={run.link || "#"}
                                                                    target="_blank"
                                                                    rel="noopener noreferrer"
                                                                    hidden={!run.link}
                                                                >
                                                                    {run.text}
                                                                </a>
                                                                <span hidden={!!run.link}>
                                                                    {run.text}
                                                                </span>
                                                            </span>
                                                        </template>
                                                        <br hidden={line.runs.length > 0} />
                                                    </div>
                                                </template>
                                            </div>
                                            <p class="community-event-organizer">
                                                {"Organized by " + item.organizer_name}
                                            </p>
                                        </div>
                                        <a
                                            class="community-event-join"
                                            href={item.join_url || "#"}
                                            hidden={!item.join_url}
                                        >
                                            "Join"
                                        </a>
                                    </li>
                                </template>
                            </ol>
                        </div>
                        <form
                            class="community-event-form"
                            hidden={eventsView !== "new"}
                            onsubmit={sendEvent(event)}
                            onkeydown={eventFormKey(event)}
                        >
                            <header class="community-event-form-header">
                                <span class="community-event-form-icon">
                                    <Icon name="calendar-day" class="size-5" />
                                </span>
                                <h2>"New event"</h2>
                                <span class="community-event-form-tab" aria-current="page">
                                    "Details"
                                </span>
                                <button
                                    type="submit"
                                    class="community-event-send"
                                    disabled={eventSaving}
                                >
                                    {eventSaving ? "Sending…" : "Send"}
                                </button>
                                <button
                                    type="button"
                                    class="community-event-close"
                                    onclick={closeEventForm()}
                                >
                                    "Close"
                                </button>
                            </header>
                            <div class="community-event-fields">
                                <label class="community-event-zone">
                                    "Time zone: "
                                    <select
                                        name="timezone"
                                        value={eventZone}
                                        onchange={setEventZone(event.currentTarget.value)}
                                    >
                                        <template pp-for="zone in eventZones">
                                            <option key={zone.value} value={zone.value}>
                                                {zone.label}
                                            </option>
                                        </template>
                                    </select>
                                </label>
                                <p class="community-event-error" role="alert" hidden={!eventError}>
                                    {eventError}
                                </p>
                                <div class="community-event-row" data-align="top">
                                    <button
                                        type="button"
                                        class="community-event-row-icon"
                                        aria-label="Change banner"
                                        title="Change banner"
                                        onclick={nextEventBanner()}
                                    >
                                        <Icon name="image" class="size-5" />
                                    </button>
                                    <button
                                        type="button"
                                        class="community-event-banner-pick"
                                        aria-label="Change banner"
                                        onclick={nextEventBanner()}
                                    >
                                        <img
                                            src={"/images/banners/" + eventBanner + ".svg"}
                                            alt=""
                                        />
                                    </button>
                                </div>
                                <div class="community-event-row">
                                    <span class="community-event-row-icon">
                                        <Icon name="edit" class="size-5" />
                                    </span>
                                    <input
                                        class="community-event-title"
                                        name="title"
                                        maxlength="120"
                                        autocomplete="off"
                                        placeholder="Add title"
                                        aria-label="Title"
                                        pp-ref={eventTitleInput}
                                    />
                                </div>
                                <div class="community-event-row">
                                    <span class="community-event-row-icon">
                                        <Icon name="clock" class="size-5" />
                                    </span>
                                    <div class="community-event-times">
                                        <input
                                            type="date"
                                            aria-label="Start date"
                                            value={eventStartDate}
                                            onchange={moveEventStart(event.currentTarget.value, eventStartTime)}
                                        />
                                        <select
                                            aria-label="Start time"
                                            value={eventStartTime}
                                            hidden={eventAllDay}
                                            onchange={moveEventStart(eventStartDate, event.currentTarget.value)}
                                        >
                                            <template pp-for="slot in timeSlots">
                                                <option key={slot.value} value={slot.value}>
                                                    {slot.label}
                                                </option>
                                            </template>
                                        </select>
                                        <Icon name="arrow-right" class="size-4 text-subtle" />
                                        <input
                                            type="date"
                                            aria-label="End date"
                                            value={eventEndDate}
                                            onchange={setEventEndDate(event.currentTarget.value)}
                                        />
                                        <select
                                            aria-label="End time"
                                            value={eventEndTime}
                                            hidden={eventAllDay}
                                            onchange={setEventEndTime(event.currentTarget.value)}
                                        >
                                            <template pp-for="slot in timeSlots">
                                                <option key={slot.value} value={slot.value}>
                                                    {slot.label}
                                                </option>
                                            </template>
                                        </select>
                                        <span class="community-event-duration" hidden={eventAllDay}>
                                            {eventDuration}
                                        </span>
                                        <label class="community-event-switch">
                                            <input
                                                type="checkbox"
                                                role="switch"
                                                checked={eventAllDay}
                                                onchange={setEventAllDay(event.currentTarget.checked)}
                                            />
                                            <span aria-hidden="true"></span>
                                            "All day"
                                        </label>
                                    </div>
                                </div>
                                <div class="community-event-row">
                                    <span class="community-event-row-icon">
                                        <Icon name="repeat" class="size-5" />
                                    </span>
                                    <div class="community-event-times">
                                        <select
                                            aria-label="Repeat"
                                            value={eventRepeat}
                                            onchange={setEventRepeat(event.currentTarget.value)}
                                        >
                                            <option value="none">"Does not repeat"</option>
                                            <option value="daily">"Every day"</option>
                                            <option value="weekly">"Every week"</option>
                                            <option value="monthly">"Every month"</option>
                                        </select>
                                        <label
                                            class="community-event-count"
                                            hidden={eventRepeat === "none"}
                                        >
                                            "For"
                                            <input
                                                type="number"
                                                min="2"
                                                max="52"
                                                aria-label="Number of occurrences"
                                                value={eventOccurrences}
                                                onchange={setEventOccurrences(Number(event.currentTarget.value))}
                                            />
                                            "occurrences"
                                        </label>
                                    </div>
                                </div>
                                <div class="community-event-row">
                                    <span class="community-event-row-icon">
                                        <Icon name="location" class="size-5" />
                                    </span>
                                    <input
                                        class="community-event-input"
                                        name="location"
                                        maxlength="300"
                                        autocomplete="off"
                                        placeholder="Add location"
                                        aria-label="Location"
                                    />
                                </div>
                                <div class="community-event-row">
                                    <span class="community-event-row-icon"></span>
                                    <label class="community-event-switch">
                                        <input
                                            type="checkbox"
                                            role="switch"
                                            checked={eventOnline}
                                            onchange={setEventOnline(event.currentTarget.checked)}
                                        />
                                        <span aria-hidden="true"></span>
                                        "Online event"
                                    </label>
                                </div>
                                <div class="community-event-row" data-align="top">
                                    <span class="community-event-row-icon">
                                        <Icon name="list" class="size-5" />
                                    </span>
                                    <div class="community-event-details-field">
                                        <div
                                            class="community-event-toolbar"
                                            role="toolbar"
                                            aria-label="Formatting options"
                                        >
                                            <button
                                                type="button"
                                                aria-label="Bold"
                                                title="Bold (Ctrl+B)"
                                                aria-pressed={!!eventFormats.strong}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("strong")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="bold" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Italic"
                                                title="Italic (Ctrl+I)"
                                                aria-pressed={!!eventFormats.em}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("em")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="italic" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Underline"
                                                title="Underline (Ctrl+U)"
                                                aria-pressed={!!eventFormats.underline}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("underline")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="underline" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Strikethrough"
                                                title="Strikethrough"
                                                aria-pressed={!!eventFormats.strike}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("strike")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="strikethrough" class="size-4" />
                                            </button>
                                            <span
                                                class="community-toolbar-divider"
                                                aria-hidden="true"
                                            >
                                            </span>
                                            <button
                                                type="button"
                                                aria-label="Highlight"
                                                title="Highlight"
                                                aria-expanded={eventStylePanel === "highlight"}
                                                onmousedown={event.preventDefault()}
                                                onclick={toggleEventPanel("highlight")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="highlight" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Font color"
                                                title="Font color"
                                                aria-expanded={eventStylePanel === "color"}
                                                onmousedown={event.preventDefault()}
                                                onclick={toggleEventPanel("color")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="font-color" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Font size"
                                                title="Font size"
                                                aria-expanded={eventStylePanel === "size"}
                                                onmousedown={event.preventDefault()}
                                                onclick={toggleEventPanel("size")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="font-size" class="size-4" />
                                            </button>
                                            <select
                                                class="community-event-block"
                                                aria-label="Paragraph style"
                                                value={eventBlockStyle}
                                                onchange={setEventBlock(event.currentTarget.value)}
                                                disabled={!eventEditorReady}
                                            >
                                                <option value="paragraph">"Paragraph"</option>
                                                <option value="quote">"Quote"</option>
                                                <option value="code">"Code"</option>
                                            </select>
                                            <span
                                                class="community-toolbar-divider"
                                                aria-hidden="true"
                                            >
                                            </span>
                                            <button
                                                type="button"
                                                aria-label="Decrease indent"
                                                title="Decrease indent"
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("outdent")}
                                                disabled={!eventEditorReady || !eventFormats.list_item}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.6"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M21 5H11M21 12H11M21 19H3M3 5h4M7 9l-4 3 4 3"
                                                    />
                                                </svg>
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Increase indent"
                                                title="Increase indent"
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("indent")}
                                                disabled={!eventEditorReady || !eventFormats.list_item}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.6"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M21 5H11M21 12H11M21 19H3M3 9l4 3-4 3"
                                                    />
                                                </svg>
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Bulleted list"
                                                title="Bulleted list"
                                                aria-pressed={!!eventFormats.bullet_list}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("bullet_list")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="list" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Numbered list"
                                                title="Numbered list"
                                                aria-pressed={!!eventFormats.ordered_list}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("ordered_list")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="list-ordered" class="size-4" />
                                            </button>
                                            <span
                                                class="community-toolbar-divider"
                                                aria-hidden="true"
                                            >
                                            </span>
                                            <button
                                                type="button"
                                                aria-label="Quote"
                                                title="Quote"
                                                aria-pressed={!!eventFormats.blockquote}
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("blockquote")}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="quote" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Insert link"
                                                title="Insert link"
                                                aria-expanded={eventStylePanel === "link"}
                                                onmousedown={event.preventDefault()}
                                                onclick={openEventLink()}
                                                disabled={!eventEditorReady}
                                            >
                                                <Icon name="link" class="size-4" />
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Clear formatting"
                                                title="Clear formatting"
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("clear")}
                                                disabled={!eventEditorReady}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.6"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M4 4h12M10 4l-4 16M4 20h8M15 14l6 6M21 14l-6 6"
                                                    />
                                                </svg>
                                            </button>
                                            <span
                                                class="community-toolbar-divider"
                                                aria-hidden="true"
                                            >
                                            </span>
                                            <button
                                                type="button"
                                                aria-label="Undo"
                                                title="Undo (Ctrl+Z)"
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("undo")}
                                                disabled={!eventEditorReady}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.6"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="M9 14 4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 0 11H11"
                                                    />
                                                </svg>
                                            </button>
                                            <button
                                                type="button"
                                                aria-label="Redo"
                                                title="Redo (Ctrl+Y)"
                                                onmousedown={event.preventDefault()}
                                                onclick={formatEvent("redo")}
                                                disabled={!eventEditorReady}
                                            >
                                                <svg
                                                    class="size-4"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="1.6"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    aria-hidden="true"
                                                >
                                                    <path
                                                        d="m15 14 5-5-5-5M20 9H9.5a5.5 5.5 0 0 0 0 11H13"
                                                    />
                                                </svg>
                                            </button>
                                        </div>
                                        <div
                                            class="community-event-style-panel"
                                            hidden={!eventStylePanel}
                                        >
                                            <div
                                                hidden={eventStylePanel !== "highlight" && eventStylePanel !== "color"}
                                            >
                                                <template pp-for="swatch in eventSwatches">
                                                    <button
                                                        key={swatch.value}
                                                        type="button"
                                                        class="community-swatch"
                                                        value={swatch.value}
                                                        aria-label={swatch.label}
                                                        title={swatch.label}
                                                        pp-style={"background:" + swatch.value}
                                                        onmousedown={event.preventDefault()}
                                                        onclick={styleEvent(eventStylePanel, event.currentTarget.value)}
                                                    >
                                                    </button>
                                                </template>
                                            </div>
                                            <div hidden={eventStylePanel !== "size"}>
                                                <button
                                                    type="button"
                                                    class="community-style-option"
                                                    onmousedown={event.preventDefault()}
                                                    onclick={styleEvent("size", "small")}
                                                >
                                                    "Small"
                                                </button>
                                                <button
                                                    type="button"
                                                    class="community-style-option"
                                                    onmousedown={event.preventDefault()}
                                                    onclick={styleEvent("size", "normal")}
                                                >
                                                    "Medium"
                                                </button>
                                                <button
                                                    type="button"
                                                    class="community-style-option"
                                                    onmousedown={event.preventDefault()}
                                                    onclick={styleEvent("size", "large")}
                                                >
                                                    "Large"
                                                </button>
                                            </div>
                                            <div hidden={eventStylePanel !== "link"}>
                                                <input
                                                    type="url"
                                                    aria-label="Link address"
                                                    placeholder="https://example.com"
                                                    value={eventLinkUrl}
                                                    oninput={setEventLinkUrl(target.value)}
                                                    onkeydown={eventLinkKey(event)}
                                                    pp-ref={eventLinkInput}
                                                />
                                                <button
                                                    type="button"
                                                    class="community-style-option"
                                                    onclick={saveEventLink()}
                                                >
                                                    "Insert"
                                                </button>
                                            </div>
                                        </div>
                                        <div class="community-event-editor-wrap">
                                            <span
                                                class="community-event-placeholder"
                                                aria-hidden="true"
                                                hidden={!!eventDetailsText || eventHasList}
                                            >
                                                "Type details for this new meeting"
                                            </span>
                                            <div
                                                class="community-event-editor"
                                                pp-ref={eventEditorHost}
                                            >
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </form>
                    </section>
                </div>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={cloudOpener}></span>
                    </DialogTrigger>
                    <DialogContent
                        class=@{CLOUD_DIALOG}
                        hide_close=@{true}
                        attrs=@{named("Attach cloud files")}
                    >
                        <nav class=@{CLOUD_NAV} aria-label="Storage">
                            <DialogTitle class="mb-4 hidden px-2 text-xl! font-semibold sm:block">
                                "Dilnaka Storage"
                            </DialogTitle>
                            <DialogDescription class="sr-only">
                                "Choose files from your Dilnaka Storage to share in this channel."
                            </DialogDescription>
                            <button
                                type="button"
                                class=@{CLOUD_NAV_ITEM}
                                data-active={cloudView === "files"}
                                onclick={showCloudView("files")}
                            >
                                <Icon name="folder" class="size-[18px] shrink-0" />
                                "My files"
                            </button>
                            <button
                                type="button"
                                class=@{CLOUD_NAV_ITEM}
                                data-active={cloudView === "recent"}
                                onclick={showCloudView("recent")}
                            >
                                <Icon name="clock" class="size-[18px] shrink-0" />
                                "Recent"
                            </button>
                            <button
                                type="button"
                                class=@{CLOUD_NAV_ITEM}
                                data-active={cloudView === "photos"}
                                onclick={showCloudView("photos")}
                            >
                                <Icon name="image" class="size-[18px] shrink-0" />
                                "Photos"
                            </button>
                        </nav>
                        <section class="flex min-h-0 min-w-0 flex-col">
                            <header class="flex items-center gap-4 px-4 pb-3 pt-4 sm:px-6 sm:pt-6">
                                <h3
                                    class="community-cloud-path flex min-w-0 flex-1 items-center gap-1 text-lg font-semibold sm:text-xl"
                                >
                                    <button
                                        type="button"
                                        class="truncate rounded px-1 text-subtle hover:bg-hover hover:text-ink"
                                        hidden={cloudView !== "files" || !cloudFolder}
                                        onclick={openCloudFolder("")}
                                    >
                                        "My files"
                                    </button>
                                    <template pp-for="crumb in cloudPath.slice(0, -1)">
                                        <span
                                            key={crumb.id}
                                            class="flex min-w-0 items-center gap-1"
                                        >
                                            <Icon
                                                name="chevron-right"
                                                class="size-4 shrink-0 text-subtle"
                                            />
                                            <button
                                                type="button"
                                                class="truncate rounded px-1 text-subtle hover:bg-hover hover:text-ink"
                                                value={crumb.id}
                                                onclick={openCloudFolder(event.currentTarget.value)}
                                            >
                                                {crumb.name}
                                            </button>
                                        </span>
                                    </template>
                                    <span
                                        class="flex min-w-0 items-center gap-1"
                                        hidden={cloudView !== "files" || !cloudFolder}
                                    >
                                        <Icon
                                            name="chevron-right"
                                            class="size-4 shrink-0 text-subtle"
                                        />
                                    </span>
                                    <span class="truncate px-1">{cloudHeading}</span>
                                </h3>
                            </header>
                            <div
                                class="min-h-0 flex-1 overflow-y-auto px-2 scroll-thin sm:px-4"
                                role="group"
                                aria-label="Cloud files"
                            >
                                <div class=@{CLOUD_HEAD} role="presentation">
                                    <Icon name="file" class="col-start-2 size-4 text-subtle" />
                                    <span class="px-1">"Name"</span>
                                    <span class="px-1">"Modified"</span>
                                    <span class="hidden px-1 sm:block">"Size"</span>
                                </div>
                                <p
                                    class="px-3 py-1 text-xs text-danger-ink"
                                    role="alert"
                                    hidden={!cloudError}
                                >
                                    {cloudError}
                                </p>
                                <p class=@{CLOUD_NOTE} role="status" hidden={!cloudLoading}>
                                    "Loading your files…"
                                </p>
                                <div
                                    class="community-cloud-empty flex flex-col items-center gap-2 px-6 py-16 text-center"
                                    hidden={cloudLoading || cloudRows.length > 0 || !!cloudError}
                                >
                                    <Icon name="cloud" class="size-10 text-subtle" />
                                    <p class="text-sm text-ink">{cloudEmpty}</p>
                                </div>
                                <div
                                    class="community-cloud-files"
                                    role="list"
                                    hidden={cloudLoading}
                                >
                                    <template pp-for="row in cloudRows">
                                        <button
                                            key={row.key}
                                            type="button"
                                            role="listitem"
                                            class=@{CLOUD_ROW}
                                            value={row.key}
                                            data-selected={cloudPicked.includes(row.id)}
                                            aria-label={row.kind === "folder" ? "Open folder " + row.name : (cloudPicked.includes(row.id) ? "Deselect " : "Select ") + row.name}
                                            onclick={pickCloudRow(event.currentTarget.value)}
                                        >
                                            <span
                                                class=@{CLOUD_CHECK}
                                                hidden={row.kind === "folder"}
                                            >
                                                <span hidden={!cloudPicked.includes(row.id)}>
                                                    <Icon name="check" class="size-3" />
                                                </span>
                                            </span>
                                            <span hidden={row.kind !== "folder"}></span>
                                            <span
                                                class="flex justify-center"
                                                hidden={row.icon !== "folder"}
                                            >
                                                <Icon name="folder" class=@{FOLDER_ICON} />
                                            </span>
                                            <span
                                                class="flex justify-center"
                                                hidden={row.icon !== "image"}
                                            >
                                                <Icon name="image" class="size-6 text-tint-blue" />
                                            </span>
                                            <span
                                                class="flex justify-center"
                                                hidden={row.icon !== "doc"}
                                            >
                                                <Icon
                                                    name="file-text"
                                                    class="size-6 text-tint-red"
                                                />
                                            </span>
                                            <span
                                                class="flex justify-center"
                                                hidden={row.icon !== "file"}
                                            >
                                                <Icon name="file" class="size-6 text-brand-ink" />
                                            </span>
                                            <span class="min-w-0 truncate px-1 text-sm text-ink">
                                                {row.name}
                                            </span>
                                            <span class="truncate px-1 text-sm text-ink/85">
                                                {row.modified}
                                            </span>
                                            <span
                                                class="hidden truncate px-1 text-sm text-ink/85 sm:block"
                                            >
                                                {row.detail}
                                            </span>
                                        </button>
                                    </template>
                                </div>
                            </div>
                            <footer
                                class="flex items-center gap-3 border-t border-line px-4 py-3 sm:px-6 sm:py-4"
                            >
                                <p class="min-w-0 flex-1 truncate text-xs text-subtle">
                                    {cloudPicked.length === 0 ? "" : cloudPicked.length + " selected"}
                                </p>
                                <Button
                                    class="community-cloud-share"
                                    disabled={cloudPicked.length === 0 || cloudSharing}
                                    onclick={shareCloudFiles()}
                                >
                                    {cloudSharing ? "Sharing…" : cloudPicked.length > 1 ? "Share " + cloudPicked.length + " files" : "Share"}
                                </Button>
                                <DialogClose class=@{cancel_button.as_str()}>
                                    <span pp-ref={cloudCloser}></span>
                                    "Cancel"
                                </DialogClose>
                            </footer>
                        </section>
                    </DialogContent>
                </Dialog>
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
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={bannerOpener}></span>
                    </DialogTrigger>
                    <DialogClose attrs=@{hidden()}>
                        <span pp-ref={bannerCloser}></span>
                    </DialogClose>
                    <DialogContent class="community-banner-dialog">
                        <DialogHeader>
                            <DialogTitle>"Edit banner"</DialogTitle>
                        </DialogHeader>
                        <div
                            class="community-banner-grid"
                            role="radiogroup"
                            aria-label="Banner images"
                            onkeydown={moveBannerFocus(event)}
                        >
                            <button
                                type="button"
                                role="radio"
                                class="community-banner-choice"
                                value="custom"
                                aria-label="Uploaded image"
                                aria-checked={bannerChoice === "custom"}
                                tabindex={bannerChoice === "custom" ? 0 : -1}
                                hidden={!bannerCustomImage}
                                onclick={chooseBanner(event)}
                            >
                                <img src={bannerCustomImage || "/images/banners/mint.svg"} alt="" />
                            </button>
                            <template pp-for="banner in banners">
                                <button
                                    key={banner.key}
                                    type="button"
                                    role="radio"
                                    class="community-banner-choice"
                                    value={banner.key}
                                    aria-label={banner.label}
                                    aria-checked={bannerChoice === banner.key}
                                    tabindex={bannerTabStop === banner.key ? 0 : -1}
                                    onclick={chooseBanner(event)}
                                >
                                    <img src={"/images/banners/" + banner.key + ".svg"} alt="" />
                                </button>
                            </template>
                        </div>
                        <p class="community-banner-error" role="alert" hidden={!bannerError}>
                            {bannerError}
                        </p>
                        <input
                            class="sr-only"
                            type="file"
                            accept="image/png,image/jpeg"
                            tabindex="-1"
                            aria-hidden="true"
                            pp-ref={bannerInput}
                            onchange={bannerFileChosen(event)}
                        />
                        <footer class="community-banner-footer">
                            <Button
                                variant="outline"
                                onclick={pickBannerImage()}
                                disabled={working}
                            >
                                <Icon name="upload" class="size-5" />
                                "Upload image"
                            </Button>
                            <Button onclick={saveBanner()} disabled={working || !bannerChanged}>
                                {working ? "Saving…" : "Done"}
                            </Button>
                        </footer>
                    </DialogContent>
                </Dialog>
                <div
                    id="community-reaction-picker"
                    popover="auto"
                    role="dialog"
                    aria-label="Reactions"
                    class="community-reaction-popover"
                    pp-ref={reactionPopover}
                    pp-style={reactionPosition}
                    ontoggle={reactionPopoverToggled(event)}
                >
                    <EmojiPanel
                        recent_key="reaction-emoji"
                        label="React with {}"
                        closes="community-reaction-picker"
                        placeholder="Find something fun"
                        open={reactionPanelOpen}
                        session={reactionSession}
                        select={onReactionEmoji}
                    />
                </div>
                <div
                    id="community-post-menu"
                    popover="auto"
                    role="menu"
                    aria-label="Post options"
                    class="community-actions-menu community-post-menu"
                    pp-ref={postMenu}
                    pp-style={postMenuPosition}
                    ontoggle={postMenuToggled(event)}
                    onkeydown={postMenuKey(event)}
                >
                    <button role="menuitem" onclick={copyPostLink(menuPostId)}>
                        <Icon name="link" class="size-5" />
                        "Copy link"
                    </button>
                    <button
                        role="menuitem"
                        hidden={!menuPost || !canDelete(menuPost)}
                        onclick={deletePost(menuPostId)}
                    >
                        <Icon name="trash" class="size-5" />
                        "Delete"
                        <kbd>"Del"</kbd>
                    </button>
                    <button
                        role="menuitem"
                        hidden={!activeCommunity?.is_owner}
                        onclick={togglePin(menuPostId)}
                    >
                        <Icon name="pin" class="size-5" />
                        {menuPost?.pinned ? "Unpin for everyone" : "Pin for everyone"}
                        <kbd>"P"</kbd>
                    </button>
                    <button role="menuitem" onclick={openConversationWindow(menuPostId)}>
                        <Icon name="open-window" class="size-5" />
                        "Open conversation in new window"
                    </button>
                </div>
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
                                            {member.is_owner ? "Owner" : member.state === "invited" ? "Invitation pending" : member.state === "requested" ? "Requested to join" : member.state === "blocked" ? "Blocked" : "Member"}
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
                                        for="community-name"
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
                                    for="community-description"
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
                    .community-nav-row[data-selected=true],.community-nav-row:hover { background:var(--color-popover); }
                    .community-nav-row .community-nav-item { flex:1;min-width:0;background:transparent; }
                    .community-more { display:grid;place-items:center;flex-shrink:0;width:36px;height:36px;margin-right:8px;border-radius:4px;color:var(--color-ink); }
                    .community-more:hover,.community-more[aria-expanded=true] { background:var(--color-hover); }
                    .community-actions-menu { position:fixed;margin:0;width:260px;max-width:calc(100vw - 24px);max-height:calc(100dvh - 24px);overflow:auto;border:1px solid var(--color-edge);border-radius:6px;padding:5px 0;background:var(--color-popover);color:var(--color-ink);box-shadow:0 8px 28px #0006; }
                    .community-actions-menu:popover-open { display:block; }
                    .community-actions-menu button { display:flex;align-items:center;gap:12px;width:100%;padding:12px 16px;text-align:left;font-size:14px; }
                    .community-actions-menu button:hover { background:var(--color-hover); }
                    .community-actions-menu button:focus { outline:2px solid var(--color-ink);outline-offset:-4px;border-radius:5px; }
                    .community-actions-menu hr { border:0;border-top:1px solid var(--color-edge);margin:6px 0; }
                    .community-action-dialog { width:min(580px,calc(100vw - 32px));max-height:calc(100dvh - 40px);overflow:auto; }
                    .community-action-fields { display:grid;gap:14px;font-size:14px; }
                    .community-action-fields p { color:var(--color-muted-foreground);line-height:1.6; }
                    .community-action-fields input,.community-action-fields textarea,.community-action-fields select { width:100%;border:1px solid var(--color-edge);border-radius:4px;background:var(--color-sunken);color:var(--color-ink);padding:10px; }
                    .community-action-fields textarea { min-height:110px; }
                    .community-action-fields h3 { font-weight:600; }
                    .community-member-row { display:flex;align-items:center;justify-content:space-between;gap:12px;border-bottom:1px solid var(--color-edge);padding:12px 0; }
                    .community-member-row > div:last-child { display:flex;gap:6px;flex-wrap:wrap; }
                    .community-action-notice,.community-status { color:var(--color-success-ink);font-size:14px;line-height:1.5; }
                    .community-status { padding:12px 24px;background:var(--color-success-soft); }
                    .community-invites { padding:20px;background:var(--color-brand-soft);display:grid;gap:16px; }
                    .community-invites h2 { font-weight:600;margin-bottom:8px; }
                    .community-invites p { margin-bottom:12px; }
                    .community-workspace { display: grid; grid-template-columns: 300px minmax(0,1fr); flex: 1 1 auto; min-height: 0; color: var(--color-ink); background: var(--color-pane); }
                    .community-sidebar { padding: 24px 16px; min-height:0; overflow-y:auto; }
                    .community-invites { flex:none; }
                    .community-sidebar-heading { display:flex;align-items:center;justify-content:space-between;margin-bottom:22px;gap:12px; }
                    .community-sidebar-heading h1 { font-size:23px;font-weight:700; }
                    .community-nav-group { margin-bottom:8px; }
                    .community-nav-item { display:flex;align-items:center;gap:12px;text-align:left;padding:14px 12px;width:100%;border-radius:8px; }
                    .community-nav-item strong { overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-nav-item:hover,.community-nav-item[aria-current=page] { background:var(--color-popover); }
                    .community-small-avatar { display:grid;place-items:center;flex-shrink:0;width:38px;height:38px;border-radius:8px;background:linear-gradient(135deg,#fff0ca,#e7c16b);font-size:24px;overflow:hidden; }
                    .community-nav-item .community-small-avatar { width:48px;height:48px;font-size:30px; }
                    .community-small-avatar img { width:100%;height:100%;object-fit:cover; }
                    .community-channel-list { padding:6px 0 6px 50px; }
                    .community-channel { display:block;padding:7px 12px;width:100%;text-align:left;font-size:14px;color:var(--color-muted-foreground);border-radius:4px;overflow-wrap:anywhere; }
                    .community-channel[aria-current=page] { color:var(--color-brand-ink);background:var(--color-hover); }
                    .community-channel .channel-nav-emoji { display:inline-block;width:22px;height:22px;object-fit:contain;vertical-align:middle;margin-right:6px; }
                    .community-small-avatar .channel-header-emoji { padding:3px;object-fit:contain; }
                    .community-add-channel { display:flex;align-items:center;gap:12px;margin:20px 12px;color:var(--color-brand-ink);font-size:14px; }
                    .community-channel-dialog { width:min(680px,calc(100vw - 32px));max-width:680px;max-height:calc(100dvh - 32px);overflow:auto;border:0;border-radius:10px;padding:28px;background:var(--color-popover);gap:28px; }
                    .community-channel-dialog [data-slot=dialog-title] { font-size:23px;line-height:1.4;font-weight:700; }
                    .community-channel-form { display:grid;gap:6px;font-size:14px; }
                    .community-channel-identity { display:flex;gap:12px;align-items:stretch; }
                    .community-channel-identity > [data-slot=popover] { flex-shrink:0; }
                    .community-channel-icon-trigger { display:flex;align-items:center;justify-content:center;gap:8px;height:38px;min-width:66px;padding:6px 10px;border:1px solid var(--color-edge);border-radius:4px;background:var(--color-sunken);color:var(--color-ink);font-size:19px; }
                    .community-channel-identity > input { min-width:0;height:38px;flex:1;border:1px solid transparent;border-radius:4px;background:var(--color-sunken);padding:8px 12px;font-size:15px;box-shadow:none; }
                    .community-channel-identity > input:focus-visible { outline:2px solid var(--color-brand-ink);outline-offset:1px; }
                    .community-channel-identity > input::placeholder { color:var(--color-muted-foreground); }
                    .community-channel-footer { display:flex;align-items:center;justify-content:space-between;gap:20px;margin-top:20px; }
                    .community-channel-footer > p { max-width:285px;color:var(--color-ink);font-size:15px;line-height:1.45; }
                    .community-channel-footer > div { display:flex;gap:10px; }
                    .community-channel-footer button { min-width:108px;height:36px;font-size:14px; }
                    .community-channel-footer button[type=button] { background:transparent;border:1px solid var(--color-edge); }
                    .community-channel-footer button[type=submit]:disabled { background:var(--color-sunken);color:var(--color-muted-foreground);opacity:1; }
                    .community-channel-error { color:var(--color-danger-ink);font-size:13px;line-height:1.5;margin-top:8px; }
                    @media(max-width:540px) { .community-channel-dialog { padding:22px; } .community-channel-dialog [data-slot=dialog-title] { font-size:20px; } .community-channel-footer { align-items:stretch;flex-direction:column; } .community-channel-footer > div { justify-content:flex-end; } }
                    .community-detail { position:relative;min-width:0;min-height:0;background:var(--color-surface);border-radius:12px 0 0 0;margin:10px 0 0;overflow-x:hidden;overflow-y:auto;overscroll-behavior:contain; }
                    .community-cover { position:relative;height:clamp(160px,22vw,280px);margin:12px 12px 0;overflow:hidden;border-radius:8px;background:#bfe9d0; }
                    .community-cover-image { position:absolute;inset:0;width:100%;height:100%;object-fit:cover; }
                    .community-cover-edit { position:absolute;right:10px;bottom:10px;display:grid;place-items:center;border-radius:4px;padding:8px;background:var(--color-surface);color:var(--color-ink);box-shadow:0 2px 8px #0005; }
                    .community-cover-edit:hover { background:var(--color-hover); }
                    .community-cover-edit:focus-visible { outline:2px solid var(--color-ink);outline-offset:2px; }
                    .community-banner-dialog { max-width:min(880px,calc(100vw - 32px)) !important;padding:32px 40px 36px !important;background:var(--color-popover) !important;border-color:var(--color-edge) !important;color:var(--color-ink); }
                    .community-banner-dialog h2 { font-size:24px;font-weight:700; }
                    .community-banner-grid { display:grid;grid-template-columns:repeat(2,minmax(0,1fr));grid-auto-rows:max-content;gap:24px 26px;max-height:min(58vh,560px);min-height:0;overflow-y:auto;padding:6px;margin:0 -6px; }
                    .community-banner-choice { display:block;border-radius:12px;overflow:hidden;background:var(--color-raised);outline:none; }
                    .community-banner-choice[hidden] { display:none; }
                    .community-banner-choice img { display:block;width:100%;aspect-ratio:2/1;object-fit:cover;transition:transform .2s; }
                    .community-banner-choice:hover img { transform:scale(1.03); }
                    .community-banner-choice:focus-visible { box-shadow:0 0 0 2px var(--color-popover),0 0 0 4px var(--color-ink); }
                    .community-banner-choice[aria-checked=true] { box-shadow:0 0 0 2px var(--color-popover),0 0 0 4px var(--color-brand-ink); }
                    .community-banner-error { color:var(--color-danger-ink);font-size:14px; }
                    .community-banner-footer { display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:20px; }
                    .community-banner-footer > button:last-child { min-width:144px;background:var(--color-brand);color:var(--color-brand-foreground); }
                    .community-banner-footer > button:last-child:hover:not(:disabled) { background:var(--color-brand-press); }
                    .community-banner-footer > button:last-child:disabled { background:var(--color-popover);color:var(--color-muted-foreground);opacity:1; }
                    .community-detail-header { position:sticky;top:0;z-index:20;background:var(--color-surface);display:flex;align-items:center;gap:22px;min-height:72px;padding:0 22px;border-bottom:1px solid var(--color-line);flex-wrap:wrap; }
                    .community-identity-title { display:flex;align-items:center;gap:12px;min-width:0; }
                    .community-identity-title h2 { font-size:20px;font-weight:700;max-width:230px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-tabs { display:flex;gap:20px;align-self:stretch; }
                    .community-tabs[hidden],.community-manage[hidden],.community-manage > section[hidden],.community-manage-table[hidden],.community-manage-options[hidden],.community-manage-empty[hidden],.community-manage-confirm[hidden],.community-manage-error[hidden],.community-manage-notice[hidden],.community-manage-nickname[hidden],.community-manage-nickname-form[hidden],.community-manage-role [hidden],.community-manage-name small[hidden],.community-manage-pending > div[hidden] { display:none; }
                    .community-header-group button[aria-pressed=true] { background:var(--color-hover);color:var(--color-brand-ink); }
                    .community-manage { min-height:calc(100% - var(--community-header,72px));display:flex;flex-direction:column;padding:24px clamp(18px,3vw,32px) 40px; }
                    .community-manage > section { display:flex;flex-direction:column;flex:1; }
                    .community-manage-error { margin-bottom:12px;padding:8px 12px;border-radius:4px;border:1px solid var(--color-danger-line);background:var(--color-danger-soft);color:var(--color-danger-ink);font-size:14px; }
                    .community-manage-notice { margin-bottom:12px;font-size:13px;color:var(--color-success-ink); }
                    .community-manage-group { display:flex;align-items:center;gap:10px;align-self:flex-start;padding:10px 4px;font-size:14px;font-weight:700;color:var(--color-ink); }
                    .community-manage-group svg { transition:transform .15s; }
                    .community-manage-group[aria-expanded=false] svg { transform:rotate(-90deg); }
                    .community-manage-group + .community-manage-table,.community-manage-group + .community-manage-options { margin-bottom:28px; }
                    .community-manage-table { display:flex;flex-direction:column; }
                    .community-manage-head,.community-manage-row { display:grid;grid-template-columns:minmax(0,1fr) minmax(220px,38%);align-items:center;gap:16px;padding:0 18px; }
                    .community-manage-head { min-height:40px;font-size:12px;font-weight:600;color:var(--color-ink); }
                    .community-manage-row { min-height:62px;border-bottom:1px solid var(--color-edge); }
                    .community-manage-person { display:flex;flex-wrap:wrap;align-items:center;gap:12px;min-width:0;padding:10px 0; }
                    .community-manage-avatar { display:grid;place-items:center;flex-shrink:0;width:32px;height:32px;border-radius:50%;background:var(--color-avatar);color:var(--color-avatar-ink);font-size:12px;font-weight:700; }
                    .community-manage-name { display:flex;flex-direction:column;min-width:0; }
                    .community-manage-name strong { font-size:14px;font-weight:600;color:var(--color-ink);overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-manage-name small { font-size:12px;color:var(--color-muted-foreground);overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-manage-nickname { display:inline-flex;align-items:center;gap:6px;height:28px;padding:0 8px;border-radius:4px;font-size:13px;color:var(--color-ink); }
                    .community-manage-nickname:hover { background:var(--color-hover);color:var(--color-ink); }
                    .community-manage-nickname-form { display:flex;flex-wrap:wrap;align-items:center;gap:8px; }
                    .community-manage-nickname-form input { height:30px;width:200px;padding:0 10px;border:0;border-bottom:2px solid var(--color-brand-ink);border-radius:4px 4px 0 0;background:var(--color-raised);color:var(--color-ink);font-size:14px;outline:none; }
                    .community-manage-role { display:flex;flex-wrap:wrap;align-items:center;gap:8px;font-size:14px;color:var(--color-ink); }
                    .community-manage-role select { height:32px;min-width:140px;padding:0 8px;border:0;border-radius:4px;background:transparent;color:var(--color-ink);font-size:14px;cursor:pointer; }
                    .community-manage-role select:hover:not(:disabled) { background:var(--color-hover); }
                    .community-manage-role select option { background:var(--color-popover);color:var(--color-ink); }
                    .community-manage-primary,.community-manage-secondary { display:inline-flex;align-items:center;justify-content:center;gap:8px;height:32px;padding:0 14px;border-radius:4px;font-size:14px;font-weight:600; }
                    .community-manage-primary { background:var(--color-brand);color:var(--color-brand-foreground); }
                    .community-manage-primary:hover:not(:disabled) { background:var(--color-brand-press); }
                    .community-manage-secondary { color:var(--color-ink); }
                    .community-manage-secondary:hover:not(:disabled) { background:var(--color-hover); }
                    .community-manage-outline { border:1px solid var(--color-edge); }
                    .community-manage-primary:disabled,.community-manage-secondary:disabled { opacity:.5; }
                    .community-manage-wide { min-width:300px;margin-top:20px; }
                    .community-manage-confirm { display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin-bottom:16px;padding:12px 16px;border-radius:6px;background:var(--color-raised);border:1px solid var(--color-edge);font-size:14px;color:var(--color-ink); }
                    .community-manage-confirm p { flex:1;min-width:240px; }
                    .community-manage-toolbar { display:flex;justify-content:flex-end;margin-bottom:12px; }
                    .community-manage-empty { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;text-align:center;padding:40px 16px; }
                    .community-manage-empty img { width:300px;height:300px;max-width:70vw;max-height:70vw; }
                    .community-manage-empty h3 { margin-top:24px;font-size:18px;font-weight:700;color:var(--color-ink); }
                    .community-manage-empty p { max-width:640px;font-size:14px;color:var(--color-ink); }
                    .community-manage-options { display:flex;flex-direction:column;gap:20px;padding:4px 4px 0; }
                    .community-manage-option { display:flex;align-items:center;justify-content:space-between;gap:24px; }
                    label.community-manage-option { cursor:pointer; }
                    .community-manage-option h4 { font-size:15px;font-weight:600;color:var(--color-ink); }
                    .community-manage-option p { margin-top:4px;font-size:14px;color:var(--color-ink); }
                    @media(max-width:700px) { .community-manage-head,.community-manage-row { grid-template-columns:minmax(0,1fr);gap:4px;padding:0 4px; } .community-manage-head span:last-child { display:none; } .community-manage-row { padding-bottom:10px; } .community-manage-wide { min-width:0;width:100%; } }
                    .community-tabs button { position:relative;font-size:14px;color:var(--color-muted-foreground);padding:18px 0; }
                    .community-tabs button[aria-selected=true] { color:var(--color-ink);font-weight:600; }
                    .community-tabs button[aria-selected=true]::after { content:"";position:absolute;bottom:0;left:0;right:0;height:3px;border-radius:3px;background:var(--color-brand-ink); }
                    .community-header-actions { display:flex;align-items:center;gap:8px;margin-left:auto; }
                    .community-event-link { display:flex;align-items:center;gap:8px;height:32px;border:1px solid var(--color-edge);border-radius:4px;padding:0 12px;font-size:14px;font-weight:600;color:var(--color-ink); }
                    .community-event-link:hover { background:var(--color-hover); }
                    .community-header-group { display:flex;border:1px solid var(--color-edge);border-radius:4px;overflow:hidden; }
                    .community-header-group button { display:grid;place-items:center;width:32px;height:30px;color:var(--color-ink); }
                    .community-header-group button[hidden] { display:none; }
                    .community-header-group button + button { border-left:1px solid var(--color-edge); }
                    .community-header-group button:hover,.community-header-group button[aria-expanded=true] { background:var(--color-hover); }
                    .community-detail[hidden],.community-events[hidden] { display:none; }
                    .community-events { position:relative;display:flex;flex-direction:column;min-width:0;min-height:0;background:var(--color-surface);border-radius:12px 0 0 0;margin:10px 0 0;overflow-y:auto;overscroll-behavior:contain; }
                    .community-events-header { position:sticky;top:0;z-index:5;display:flex;align-items:center;gap:12px;min-height:56px;padding:8px 16px;background:var(--color-surface);border-bottom:1px solid var(--color-line); }
                    .community-events-header[hidden],.community-events-body[hidden],.community-event-form[hidden] { display:none; }
                    .community-events-back { display:grid;place-items:center;width:32px;height:32px;border-radius:4px;color:var(--color-ink); }
                    .community-events-back:hover { background:var(--color-hover); }
                    .community-events-header .community-small-avatar { width:32px;height:32px;font-size:20px; }
                    .community-events-title { min-width:0;flex:1; }
                    .community-events-title h2 { font-size:14px;font-weight:600;color:var(--color-ink); }
                    .community-events-title p { font-size:12px;color:var(--color-muted-foreground);overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-events-create { display:inline-flex;align-items:center;gap:8px;height:32px;padding:0 14px;border-radius:4px;background:var(--color-brand);color:var(--color-brand-foreground);font-size:14px;font-weight:600; }
                    .community-events-create:hover { background:var(--color-brand-press); }
                    .community-events-create[data-outlined=true] { box-shadow:0 0 0 2px var(--color-popover),0 0 0 3px #fff0; }
                    .community-events-body { flex:1;display:flex;flex-direction:column;padding:24px clamp(18px,4vw,48px) 40px; }
                    .community-events-note { margin-bottom:12px;font-size:13px;color:var(--color-success-ink); }
                    .community-events-note[hidden],.community-events-error[hidden],.community-events-empty[hidden],.community-events-list[hidden] { display:none; }
                    .community-events-error { margin-bottom:12px;font-size:13px;color:var(--color-danger-ink); }
                    .community-events-empty { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:28px;text-align:center;padding:40px 16px; }
                    .community-events-empty img { width:200px;height:200px; }
                    .community-events-empty h3 { font-size:18px;font-weight:700;color:var(--color-ink); }
                    .community-events-list { display:flex;flex-direction:column;gap:12px;max-width:960px; }
                    .community-event-card { display:grid;grid-template-columns:120px 56px minmax(0,1fr) auto;align-items:start;gap:16px;padding:12px;border-radius:8px;background:var(--color-raised);border:1px solid var(--color-line); }
                    .community-event-banner { width:120px;height:72px;border-radius:6px;object-fit:cover; }
                    .community-event-date { display:flex;flex-direction:column;align-items:center;padding:6px 0;border-radius:6px;background:var(--color-raised); }
                    .community-event-date small { font-size:11px;font-weight:700;color:var(--color-brand-ink); }
                    .community-event-date strong { font-size:22px;line-height:1.1;color:var(--color-ink); }
                    .community-event-info { min-width:0;display:flex;flex-direction:column;gap:4px; }
                    .community-event-info h3 { font-size:15px;font-weight:600;color:var(--color-ink);overflow-wrap:anywhere; }
                    .community-event-when,.community-event-where { display:flex;align-items:center;gap:6px;font-size:13px;color:var(--color-ink); }
                    .community-event-where[hidden],.community-event-details[hidden],.community-event-join[hidden] { display:none; }
                    .community-event-details { font-size:13px;color:var(--color-muted-foreground);white-space:pre-line;display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden; }
                    .community-event-organizer { font-size:12px;color:var(--color-muted-foreground); }
                    .community-event-join { display:inline-flex;align-items:center;height:32px;padding:0 16px;border-radius:4px;background:var(--color-brand);color:var(--color-brand-foreground);font-size:14px;font-weight:600; }
                    .community-event-join:hover { background:var(--color-brand-press); }
                    .community-workspace[data-events=new] { grid-template-columns:minmax(0,1fr); }
                    .community-workspace[data-events=new] .community-sidebar { display:none; }
                    .community-event-form { display:flex;flex-direction:column;min-height:100%; }
                    .community-event-form-header { position:sticky;top:0;z-index:5;display:flex;align-items:center;gap:14px;min-height:60px;padding:0 28px 0 36px;background:var(--color-surface);border-bottom:1px solid var(--color-line); }
                    .community-event-form-icon { display:grid;place-items:center;width:28px;height:28px;border-radius:4px;background:var(--color-brand);color:var(--color-brand-foreground); }
                    .community-event-form-header h2 { font-size:18px;font-weight:700;color:var(--color-ink); }
                    .community-event-form-tab { position:relative;align-self:stretch;display:flex;align-items:center;margin-left:4px;font-size:14px;font-weight:600;color:var(--color-ink); }
                    .community-event-form-tab::after { content:"";position:absolute;left:0;right:0;bottom:0;height:3px;border-radius:3px 3px 0 0;background:var(--color-brand-ink); }
                    .community-event-send { margin-left:auto;height:32px;padding:0 14px;border-radius:4px;background:var(--color-brand);color:var(--color-brand-foreground);font-size:14px;font-weight:600; }
                    .community-event-send:hover:not(:disabled) { background:var(--color-brand-press); }
                    .community-event-send:disabled { opacity:.6; }
                    .community-event-close { height:32px;padding:0 14px;border-radius:4px;border:1px solid var(--color-edge);color:var(--color-ink);font-size:14px;font-weight:600; }
                    .community-event-close:hover { background:var(--color-hover); }
                    .community-event-fields { display:flex;flex-direction:column;gap:16px;width:100%;max-width:1200px;padding:16px 28px 40px 36px; }
                    .community-event-zone { display:flex;align-items:center;gap:2px;font-size:14px;color:var(--color-ink); }
                    .community-event-zone select { field-sizing:content;max-width:100%;padding:4px 4px 4px 2px;border:0;border-radius:4px;background:transparent;color:var(--color-ink);font-size:14px;cursor:pointer; }
                    .community-event-zone select:hover { background:var(--color-hover); }
                    .community-event-zone select option,.community-event-times select option,.community-event-block option { background:var(--color-popover);color:var(--color-ink); }
                    .community-event-error { margin-left:48px;padding:8px 12px;border-radius:4px;border:1px solid var(--color-danger-line);background:var(--color-danger-soft);color:var(--color-danger-ink);font-size:14px; }
                    .community-event-error[hidden],.community-event-count[hidden],.community-event-duration[hidden],.community-event-times select[hidden],.community-event-style-panel[hidden],.community-event-placeholder[hidden] { display:none; }
                    .community-event-row { display:grid;grid-template-columns:32px minmax(0,1fr);align-items:center;gap:16px; }
                    .community-event-row[data-align=top] { align-items:start; }
                    .community-event-row-icon { display:grid;place-items:center;width:32px;height:32px;border-radius:4px;color:var(--color-ink); }
                    button.community-event-row-icon:hover { background:var(--color-hover); }
                    .community-event-banner-pick { display:block;width:100%;border-radius:12px;overflow:hidden; }
                    .community-event-banner-pick img { display:block;width:100%;aspect-ratio:4.3/1;object-fit:cover; }
                    .community-event-title,.community-event-input { height:32px;padding:0 12px;border:0;border-bottom:2px solid transparent;border-radius:4px;background:var(--color-raised);color:var(--color-ink);font-size:14px;outline:none; }
                    .community-event-input { height:44px; }
                    .community-event-title:focus,.community-event-input:focus { border-bottom-color:var(--color-brand-ink);border-radius:4px 4px 0 0; }
                    .community-event-title::placeholder,.community-event-input::placeholder { color:var(--color-muted-foreground); }
                    .community-event-times { display:flex;flex-wrap:wrap;align-items:center;gap:10px; }
                    .community-event-times input[type=date],.community-event-times select,.community-event-count input { height:32px;padding:0 10px;border:0;border-radius:4px;background:var(--color-raised);color:var(--color-ink);font-size:14px;color-scheme:dark; }
                    .community-event-times input[type=date] { position:relative;width:176px; }
                    .community-event-times input[type=date]::-webkit-calendar-picker-indicator { position:absolute;inset:0;width:auto;height:auto;opacity:0;cursor:pointer; }
                    .community-event-times select { min-width:96px;cursor:pointer; }
                    .community-event-count { display:flex;align-items:center;gap:8px;font-size:14px;color:var(--color-ink); }
                    .community-event-count input { width:72px; }
                    .community-event-duration { font-size:14px;color:var(--color-ink); }
                    .community-event-switch { display:inline-flex;align-items:center;gap:10px;font-size:14px;color:var(--color-ink);cursor:pointer; }
                    .community-event-switch input { position:absolute;opacity:0;width:1px;height:1px; }
                    .community-event-switch span { position:relative;width:40px;height:20px;border-radius:10px;border:1px solid var(--color-edge);transition:background .15s; }
                    .community-event-switch span::after { content:"";position:absolute;top:3px;left:3px;width:12px;height:12px;border-radius:50%;background:var(--color-ink);transition:transform .15s; }
                    .community-event-switch input:checked + span { background:var(--color-brand-ink);border-color:var(--color-brand-ink); }
                    .community-event-switch input:checked + span::after { transform:translateX(20px);background:var(--color-ink); }
                    .community-event-switch input:focus-visible + span { outline:2px solid var(--color-ink);outline-offset:2px; }
                    .community-event-details-field { display:flex;flex-direction:column;gap:6px;min-width:0; }
                    .community-event-toolbar { display:flex;flex-wrap:wrap;align-items:center;gap:2px;min-height:32px;color:var(--color-ink); }
                    .community-event-toolbar button { display:grid;place-items:center;width:32px;height:32px;border-radius:4px; }
                    .community-event-toolbar button:hover:not(:disabled) { background:var(--color-hover); }
                    .community-event-toolbar button[aria-pressed=true],.community-event-toolbar button[aria-expanded=true] { background:var(--color-hover);color:var(--color-brand-ink); }
                    .community-event-toolbar button:disabled,.community-event-block:disabled { opacity:.45; }
                    .community-event-block { height:32px;margin:0 2px;padding:0 6px;border:0;border-radius:4px;background:transparent;color:var(--color-ink);font-size:13px;cursor:pointer; }
                    .community-event-block:hover:not(:disabled) { background:var(--color-hover); }
                    .community-event-style-panel { display:flex;flex-wrap:wrap;gap:6px;padding:6px;border-radius:6px;background:var(--color-raised); }
                    .community-event-style-panel > div { display:flex;flex-wrap:wrap;align-items:center;gap:6px; }
                    .community-event-style-panel > div[hidden] { display:none; }
                    .community-event-style-panel input { height:30px;min-width:min(320px,60vw);padding:0 10px;border:1px solid var(--color-edge);border-radius:4px;background:var(--color-popover);color:var(--color-ink);font-size:14px; }
                    .community-event-editor-wrap { position:relative;border-radius:4px;background:var(--color-raised); }
                    .community-event-editor-wrap:focus-within { box-shadow:inset 0 -2px var(--color-brand-ink); }
                    .community-event-placeholder { position:absolute;top:12px;left:14px;font-size:14px;color:var(--color-muted-foreground);pointer-events:none; }
                    .community-event-editor .ProseMirror { min-height:96px;max-height:420px;overflow-y:auto;padding:12px 14px;color:var(--color-ink);font-size:14px;line-height:1.6;outline:none;white-space:pre-wrap;overflow-wrap:anywhere; }
                    .community-event-editor .ProseMirror ul { list-style:disc;padding-left:22px; }
                    .community-event-editor .ProseMirror ol { list-style:decimal;padding-left:22px; }
                    .community-event-editor .ProseMirror blockquote { padding-left:12px;border-left:3px solid var(--color-brand-ink);color:var(--color-ink); }
                    .community-event-editor .ProseMirror pre { padding:8px 10px;border-radius:4px;background:var(--color-popover);font-family:ui-monospace,Consolas,monospace;font-size:13px; }
                    .community-event-editor .ProseMirror a { color:var(--color-brand-ink);text-decoration:underline; }
                    @media(max-width:700px) { .community-events { margin:0;border-radius:12px 12px 0 0; } .community-event-card { grid-template-columns:56px minmax(0,1fr); } .community-event-banner { display:none; } .community-event-join { grid-column:2; justify-self:start; } .community-event-row { gap:10px; } .community-event-form-header { flex-wrap:wrap;padding:8px 16px; } .community-event-fields { padding:16px; } .community-event-error { margin-left:42px; } .community-event-times input[type=date] { width:150px; } }
                    .community-content { max-width:1140px;margin:auto;padding:24px clamp(18px,5vw,90px) 50px; }
                    .community-composer,.community-post { border:1px solid var(--color-line);border-radius:8px;background:var(--color-surface);padding:18px;box-shadow:0 2px 4px #0003; }
                    .community-composer-heading { display:flex;align-items:center;gap:12px;font-size:14px;font-weight:600;color:var(--color-ink); }
                    .community-author-avatar { width:36px;height:36px;display:grid;place-items:center;flex-shrink:0;border-radius:50%;background:var(--color-avatar);color:var(--color-avatar-ink);font-size:13px; }
                    .community-composer { padding:0; }
                    .community-composer-collapsed { padding:14px 16px 12px; }
                    .community-composer-heading { width:100%;text-align:left;border-radius:6px;padding:2px 0; }
                    .community-compose-modes { display:flex;gap:16px;margin-top:16px; }
                    .community-compose-mode { display:inline-flex;align-items:center;gap:8px;height:32px;padding:0 24px;border-radius:4px;font-size:14px;font-weight:600;color:var(--color-ink); }
                    .community-compose-mode svg { color:var(--color-ink); }
                    .community-compose-mode[data-primary] { background:var(--color-hover);color:var(--color-brand-ink); }
                    .community-compose-mode[data-primary] svg { color:var(--color-brand-ink); }
                    .community-compose-mode:hover { background:var(--color-hover); }
                    .community-compose-mode:focus-visible,.community-composer-heading:focus-visible { outline:2px solid var(--color-ink);outline-offset:1px; }
                    .community-composer-expanded { padding:0 16px 14px; }
                    .community-composer-author { display:flex;align-items:center;gap:10px;height:56px;border-bottom:1px solid var(--color-edge);font-size:13px; }
                    .community-composer-author strong { font-weight:700; }
                    .community-composer-close { margin-left:auto;display:grid;place-items:center;width:32px;height:32px;border-radius:4px;color:var(--color-ink); }
                    .community-composer-close:hover,.community-format-toolbar button:hover:not(:disabled),.community-composer-footer > button[type=button]:hover:not(:disabled) { background:var(--color-hover); }
                    .community-compose-line { display:block;width:100%;height:56px;padding:0 6px;border-bottom:1px solid var(--color-edge);background:transparent;outline:none;font-size:18px;color:var(--color-ink); }
                    .community-compose-line::placeholder { color:var(--color-muted-foreground); }
                    .community-compose-line:focus { border-bottom-color:var(--color-brand-ink); }
                    .community-format-toolbar { display:flex;flex-wrap:wrap;align-items:center;gap:2px;padding:10px 0 6px;color:var(--color-ink); }
                    .community-format-toolbar button,.community-composer-footer > button[type=button] { display:grid;place-items:center;width:32px;height:32px;border-radius:4px; }
                    .community-format-toolbar button[aria-pressed=true],.community-format-toolbar button[aria-expanded=true] { background:var(--color-hover);color:var(--color-brand-ink); }
                    .community-format-toolbar button:disabled { opacity:.45; }
                    .community-toolbar-divider { width:1px;height:16px;margin:0 6px;background:var(--color-edge); }
                    .community-style-panel > div { display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:6px 0; }
                    .community-style-panel > div[hidden] { display:none; }
                    .community-style-panel input { flex:1;min-width:0;height:32px;padding:0 10px;border:1px solid var(--color-edge);border-radius:4px;background:var(--color-popover);outline:none;font-size:14px; }
                    .community-swatch { width:24px;height:24px;border-radius:50%;outline:1px solid #ffffff33; }
                    .community-swatch:hover,.community-swatch:focus-visible { outline:2px solid var(--color-ink); }
                    .community-style-option { height:32px;padding:0 12px;border:1px solid var(--color-edge);border-radius:4px;font-size:13px; }
                    .community-style-option:hover { background:var(--color-hover); }
                    .community-compose-emoji { max-height:220px;overflow-y:auto; }
                    .community-rich-host { position:relative; }
                    .community-rich-host .ProseMirror { min-height:96px;max-height:360px;overflow-y:auto;padding:4px 6px 12px;outline:none;font-size:14px;line-height:1.6;white-space:pre-wrap;overflow-wrap:anywhere; }
                    .community-rich-host .ProseMirror p { margin:0;min-height:1.6em; }
                    .community-rich-host .ProseMirror:has(> p:only-child > br:only-child)::before,.community-rich-host .ProseMirror:has(> p:only-child:empty)::before { content:"Type a message";position:absolute;color:var(--color-muted-foreground);pointer-events:none; }
                    .community-rich-host .ProseMirror ul { list-style:disc;padding-left:24px; }
                    .community-rich-host .ProseMirror ol { list-style:decimal;padding-left:24px; }
                    .community-rich-host .ProseMirror blockquote { border-left:3px solid var(--color-edge);padding-left:12px;color:var(--color-ink); }
                    .community-rich-host .ProseMirror pre { background:var(--color-popover);padding:8px 10px;border-radius:4px;font-family:monospace; }
                    .community-rich-host .ProseMirror a { color:var(--color-brand-ink);text-decoration:underline; }
                    .community-rich-host > p { padding:4px 6px;font-size:13px;color:var(--color-muted-foreground); }
                    .community-compose-error { color:var(--color-danger-ink);font-size:13px;padding:4px 6px; }
                    .community-composer-footer { display:flex;align-items:center;gap:4px;color:var(--color-ink);padding-top:6px; }
                    .community-composer-footer > button[data-slot=button] { margin-left:auto;min-width:96px;background:var(--color-brand);color:var(--color-brand-foreground); }
                    .community-composer-footer > button[data-slot=button]:hover:not(:disabled) { background:var(--color-brand-press); }
                    .community-composer[data-mode=collapsed] { position:sticky;top:calc(var(--community-header,72px) + 12px);z-index:10; }
                    .community-composer[data-mode=collapsed]::before { content:"";position:absolute;left:-12px;right:-12px;bottom:100%;height:13px;background:var(--color-surface); }
                    .community-composer[data-mode=collapsed]::after { content:"";position:absolute;left:-12px;right:-12px;top:100%;height:12px;background:linear-gradient(var(--color-surface),transparent);pointer-events:none; }
                    .community-post { padding:0;border-color:var(--color-line);border-radius:6px; }
                    .community-post > :not(.community-post-author):not(.community-post-banner):not(.community-conversation-toggle):not(.community-replies):not(.community-reply-open):not(.community-reply-form) { margin-left:18px;margin-right:18px; }
                    .community-reactions { display:flex;flex-wrap:wrap;align-items:center;gap:6px;margin-top:14px;margin-bottom:12px; }
                    .community-reaction { display:inline-flex;align-items:center;gap:5px;height:24px;padding:0 8px;border:1px solid var(--color-edge);border-radius:12px;font-size:12px;color:var(--color-ink); }
                    .community-reaction:hover { background:var(--color-hover); }
                    .community-reaction img { width:18px;height:18px;object-fit:contain; }
                    .community-reaction-add { display:grid;place-items:center;width:34px;height:24px;border:1px solid var(--color-edge);border-radius:12px;color:var(--color-muted-foreground); }
                    .community-reaction-add:hover { background:var(--color-hover);color:var(--color-ink); }
                    .community-post { position:relative;scroll-margin-top:calc(var(--community-header,72px) + 140px); }
                    .community-post[data-focused=true] { box-shadow:0 0 0 2px var(--color-brand-ink);transition:box-shadow .3s; }
                    .community-post-actions { position:absolute;top:-16px;right:16px;margin:0 !important;z-index:6;display:flex;align-items:center;gap:2px;padding:4px;border:1px solid var(--color-edge);border-radius:6px;background:var(--color-popover);box-shadow:0 4px 14px #0008;opacity:0;pointer-events:none;transform:translateY(4px);transition:opacity .12s,transform .12s; }
                    .community-post:hover > .community-post-actions,.community-post:focus-within > .community-post-actions,.community-post-actions:has([aria-expanded=true]) { opacity:1;pointer-events:auto;transform:none; }
                    .community-post-actions[hidden] { display:none; }
                    .community-post-actions button { display:grid;place-items:center;width:32px;height:32px;border-radius:4px;color:var(--color-ink); }
                    .community-post-actions button:hover { background:var(--color-hover); }
                    .community-post-actions button[aria-expanded=true] { background:var(--color-hover);color:var(--color-brand-ink); }
                    .community-post-actions button:focus-visible { outline:2px solid var(--color-ink);outline-offset:-2px; }
                    .community-quick-reaction img { width:22px;height:22px;object-fit:contain;transition:transform .12s; }
                    .community-quick-reaction:hover img { transform:scale(1.2); }
                    .community-post-actions .community-toolbar-divider { margin:0 4px; }
                    .community-post-edited,.community-post-pinned { color:var(--color-muted-foreground);font-size:12px; }
                    .community-post-pinned { display:inline-flex;align-items:center;gap:3px;color:var(--color-brand-ink); }
                    .community-post-deleted { margin-top:12px;margin-bottom:16px;font-size:14px;font-style:italic;color:var(--color-muted-foreground); }
                    .community-post-deleted button { margin-left:10px;font-style:normal;font-weight:600;color:var(--color-brand-ink); }
                    .community-post-deleted button:hover { text-decoration:underline; }
                    .community-post[data-deleted=true] .community-post-author::before { background:var(--color-edge); }
                    .community-composer-editing { color:var(--color-brand-ink);font-size:12px; }
                    .community-reaction-popover { position:fixed;margin:0;padding:18px;border:1px solid var(--color-edge);border-radius:6px;background:var(--color-popover);color:var(--color-ink);box-shadow:0 12px 32px #0008; }
                    .community-reaction-popover:popover-open { display:block; }
                    .community-post-menu button { position:relative; }
                    .community-post-menu kbd { margin-left:auto;font:inherit;font-size:12px;color:var(--color-muted-foreground); }
                    .community-post-menu button[hidden] { display:none; }
                    .community-reaction[aria-pressed=true] { border-color:var(--color-brand-ink);background:var(--color-brand-soft);color:var(--color-brand-ink); }
                    .community-conversation-toggle,.community-reply-open { display:flex;align-items:center;gap:8px;width:100%;padding:12px 18px;border-top:1px solid var(--color-line);font-size:14px;font-weight:600;color:var(--color-ink);text-align:left; }
                    .community-conversation-toggle:hover,.community-reply-open:hover { background:var(--color-raised); }
                    .community-reply-open { font-weight:400;color:var(--color-muted-foreground);border-radius:0 0 6px 6px; }
                    .community-reply-open .community-author-avatar,.community-reply-form .community-author-avatar { width:20px;height:20px;font-size:8px; }
                    .community-replies { border-top:1px solid var(--color-line);padding:6px 18px; }
                    .community-replies[hidden] { display:none; }
                    .community-replies-empty { padding:8px 0;font-size:13px;color:var(--color-muted-foreground); }
                    .community-reply { display:flex;gap:10px;padding:10px 0; }
                    .community-reply .community-author-avatar { width:28px;height:28px;font-size:11px; }
                    .community-reply > div { min-width:0; }
                    .community-reply-meta { display:flex;flex-wrap:wrap;align-items:center;gap:6px;font-size:13px; }
                    .community-reply-meta strong { font-weight:600; }
                    .community-reply-meta time { color:var(--color-muted-foreground);font-size:12px; }
                    .community-reply-body { margin-top:2px;font-size:14px;line-height:1.5;white-space:pre-wrap;overflow-wrap:anywhere; }
                    .community-reply-form { display:flex;align-items:flex-start;gap:8px;padding:10px 18px 12px;border-top:1px solid var(--color-line); }
                    .community-reply-form[hidden] { display:none; }
                    .community-reply-form .community-author-avatar { margin-top:8px; }
                    .community-reply-form textarea { flex:1;min-width:0;min-height:36px;max-height:160px;resize:vertical;padding:7px 10px;border:1px solid var(--color-edge);border-bottom:2px solid var(--color-brand-ink);border-radius:4px;background:var(--color-popover);outline:none;font-size:14px;field-sizing:content; }
                    .community-reply-form button { display:grid;place-items:center;width:36px;height:36px;border-radius:4px;color:var(--color-brand-ink); }
                    .community-reply-form button:hover:not(:disabled) { background:var(--color-hover); }
                    .community-reply-form button:disabled { color:var(--color-muted-foreground); }
                    .community-reply-error { color:var(--color-danger-ink);font-size:13px;padding-bottom:10px; }
                    .community-post .community-post-author { position:relative;display:flex;align-items:center;gap:8px;padding:16px 18px 0;font-size:14px;flex-wrap:wrap; }
                    .community-post-author::before { content:"";position:absolute;left:0;top:16px;width:4px;height:32px;border-radius:0 3px 3px 0;background:var(--color-brand-ink); }
                    .community-post-author .community-author-avatar { width:32px;height:32px;font-size:12px; }
                    .community-post-author strong { font-weight:400;color:var(--color-ink); }
                    .community-owner-badge { display:inline-flex;align-items:center;gap:3px;height:18px;padding:0 5px;border:1px solid var(--color-brand);border-radius:4px;background:var(--color-brand-soft);color:var(--color-brand-ink);font-size:10px;font-weight:600; }
                    .community-post .community-post-author time { margin-left:4px;color:var(--color-muted-foreground);font-size:12px; }
                    .community-post .community-post-banner h3 { margin:0;color:#a6d8ff;font-size:clamp(24px,3vw,36px);font-weight:700;line-height:1.2;overflow-wrap:anywhere; }
                    .community-post-subheading { margin-top:16px;font-size:18px;font-weight:700;color:var(--color-ink);overflow-wrap:anywhere; }
                    .community-post .community-post-subject { color:var(--color-ink);font-size:18px;font-weight:700;margin-top:14px; }
                    .community-rich-message { margin-top:12px;font-size:14px;line-height:1.6; }
                    .community-post-line { min-height:1.6em;white-space:pre-wrap;overflow-wrap:anywhere; }
                    .community-post-line.is-bullet,.community-post-line.is-ordered { padding-left:12px; }
                    .community-post-line.is-quote { border-left:3px solid var(--color-edge);padding-left:12px;color:var(--color-ink); }
                    .community-post-line.is-code { background:var(--color-popover);padding:0 10px;font-family:monospace; }
                    .community-inline-code { background:var(--color-popover);border-radius:3px;padding:0 4px;font-family:monospace; }
                    .community-post-link { color:var(--color-brand-ink);text-decoration:underline; }
                    .community-setup { padding:48px 0 20px;text-align:center; }
                    .community-setup h2 { font-size:23px;font-weight:700;margin-bottom:10px; }
                    .community-setup > p { color:var(--color-ink);font-size:15px; }
                    .community-quick-actions { display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:32px; }
                    .community-quick-actions > * { display:flex;align-items:center;gap:16px;text-align:left;padding:18px;border:1px solid var(--color-edge);border-radius:5px;font-size:16px;font-weight:600; }
                    .community-quick-actions span { font-size:32px; }
                    .community-post { margin-top:20px; }
                    .community-post > p,.community-details-panel p { white-space:pre-wrap;overflow-wrap:anywhere;line-height:1.6;font-size:14px;margin-top:14px; }
                    .community-post h3 { color:var(--color-brand-ink);font-weight:700;margin-top:16px; }
                    .community-details-panel { padding:22px;border-bottom:1px solid var(--color-edge);background:var(--color-sunken); }
                    .community-details-panel h3 { font-weight:600;margin-top:14px; }
                    .community-details-panel p { color:var(--color-muted-foreground);margin-top:6px; }
                    .community-files { min-height:calc(100% - var(--community-header,72px));display:flex;flex-direction:column;padding:24px clamp(18px,4vw,48px) 40px; }
                    .community-files-empty { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:0;text-align:center;padding:40px 16px; }
                    .community-files-empty[hidden],.community-files-list[hidden] { display:none; }
                    .community-files-empty img { width:184px;height:184px; }
                    .community-files-empty h3 { margin-top:56px;font-size:18px;font-weight:700;color:var(--color-ink); }
                    .community-files-empty p { margin-top:12px;font-size:14px;color:var(--color-ink); }
                    .community-files-empty .community-files-upload { margin-top:120px; }
                    .community-files-upload { gap:8px;height:32px;padding:0 16px;background:var(--color-brand) !important;color:var(--color-brand-foreground) !important;font-weight:600; }
                    .community-files-upload:hover:not(:disabled) { background:var(--color-brand-press) !important; }
                    .community-files-toolbar { display:flex;align-items:center;gap:12px;padding-bottom:12px;border-bottom:1px solid var(--color-edge); }
                    .community-files-list table { width:100%;border-collapse:collapse;font-size:14px; }
                    .community-files-list th { padding:12px 10px;text-align:left;font-size:12px;font-weight:600;color:var(--color-muted-foreground);border-bottom:1px solid var(--color-edge); }
                    .community-files-list td { padding:10px;border-bottom:1px solid var(--color-line);color:var(--color-ink);white-space:nowrap; }
                    .community-files-list td:first-child { width:55%;white-space:normal; }
                    .community-files-list tr:hover td { background:var(--color-raised); }
                    .community-file-name { display:inline-flex;align-items:center;gap:12px;color:var(--color-ink);overflow-wrap:anywhere; }
                    .community-file-name:hover { text-decoration:underline; }
                    .community-files-status { margin-top:12px;font-size:13px;color:var(--color-success-ink); }
                    .community-files-error { margin-top:12px;font-size:13px;color:var(--color-danger-ink); }
                    .community-post-files { display:flex;flex-wrap:wrap;gap:8px;margin-top:12px; }
                    .community-post-files[hidden] { display:none; }
                    .community-file-card { display:flex;align-items:center;gap:12px;min-width:min(280px,100%);max-width:100%;padding:10px 14px;border:1px solid var(--color-edge);border-radius:6px;background:var(--color-popover); }
                    .community-file-card:hover { background:var(--color-raised); }
                    .community-file-card > span:last-child { display:grid;min-width:0; }
                    .community-file-card strong { font-size:14px;font-weight:600;color:var(--color-ink);overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
                    .community-file-card small { font-size:12px;color:var(--color-muted-foreground); }
                    @media(max-width:700px) { .community-files-list th:nth-child(3),.community-files-list td:nth-child(3),.community-files-list th:nth-child(4),.community-files-list td:nth-child(4) { display:none; } }
                    .community-photos { min-height:calc(100% - var(--community-header,72px));display:flex;flex-direction:column;padding:24px clamp(18px,4vw,48px) 40px; }
                    .community-photos[hidden],.community-photos-empty[hidden],.community-photos-grid[hidden] { display:none; }
                    .community-photos-empty { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;padding:40px 16px; }
                    .community-photos-empty img { width:184px;height:184px; }
                    .community-photos-empty h3 { margin-top:56px;font-size:18px;font-weight:700;color:var(--color-ink); }
                    .community-photos-empty p { margin-top:12px;font-size:14px;color:var(--color-ink); }
                    .community-photos-grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:8px; }
                    .community-photo { position:relative;display:block;aspect-ratio:1;overflow:hidden;border-radius:6px;background:var(--color-hover); }
                    .community-photo img { width:100%;height:100%;object-fit:cover;transition:transform .2s; }
                    .community-photo:hover img { transform:scale(1.03); }
                    .community-photo:focus-visible { outline:2px solid var(--color-brand-ink);outline-offset:2px; }
                    .community-photo-caption { position:absolute;inset:auto 0 0;display:flex;flex-direction:column;gap:2px;padding:24px 10px 8px;background:linear-gradient(transparent,#000c);color:var(--color-ink);opacity:0;transition:opacity .15s; }
                    .community-photo:hover .community-photo-caption,.community-photo:focus-visible .community-photo-caption { opacity:1; }
                    .community-photo-caption strong { overflow:hidden;font-size:13px;font-weight:600;text-overflow:ellipsis;white-space:nowrap; }
                    .community-photo-caption small { font-size:12px;color:var(--color-ink); }
                    .community-workspace-error { margin:16px 24px;padding:12px;border:1px solid var(--color-danger-line);border-radius:6px;color:var(--color-danger-ink); }
                    @media(max-width:1000px) { .community-workspace { grid-template-columns:240px minmax(0,1fr); } .community-detail-header { gap:12px;padding:10px 16px 0; } .community-content { padding:22px; } .community-header-actions { margin-bottom:10px; } }
                    @media(max-width:700px) { .community-workspace { display:block; } .community-sidebar { padding:16px; } .community-sidebar-heading { margin-bottom:12px; } .community-sidebar nav { display:flex;gap:8px;overflow:auto; } .community-nav-item { min-width:150px; } .community-channel-list { padding-left:8px; } .community-detail { margin:0;border-radius:12px 12px 0 0; } .community-quick-actions { grid-template-columns:1fr; } .community-cover { height:150px; } .community-banner-dialog { padding:24px 16px !important; } .community-banner-grid { gap:14px; } .community-detail-header { gap:12px; } .community-identity-title { width:100%; } }
                    .community-create-dialog {
                        width: min(800px, calc(100vw - 40px)); max-width: 800px;
                        min-height: min(740px, calc(100dvh - 40px));
                        max-height: calc(100dvh - 40px); overflow-y: auto;
                        padding: 32px; border: 1px solid var(--color-edge); border-radius: 5px;
                        background: var(--color-popover); color: var(--color-ink); gap: 34px;
                        grid-template-rows: auto 1fr;
                    }
                    .community-create-close { color: var(--color-ink); padding: 4px; border-radius: 4px; }
                    .community-create-close:hover { background: var(--color-hover); }
                    .community-create-form { display: flex; flex-direction: column; min-height: 0; }
                    .community-identity { display: flex; align-items: center; gap: 26px; }
                    .community-avatar { position: relative; flex-shrink: 0; width: 118px; height: 118px; overflow: hidden; border: 1px solid var(--color-edge); border-radius: 5px; background: linear-gradient(140deg, #faedcf, #eccc83); }
                    .community-avatar-emoji { display: grid; place-items: center; height: 100%; padding-bottom: 12px; font-size: 65px; }
                    .community-avatar-photo { width: 100%; height: 100%; object-fit: cover; }
                    .community-avatar-edit { position: absolute; bottom: 0; inset-inline: 0; background: #625332cc; padding: 3px; color: var(--color-ink); font-size: 16px; }
                    .community-name-input { height: 58px; border: 0; border-bottom: 1px solid var(--color-edge); border-radius: 0; background: transparent; padding: 4px 12px; font-size: 30px; font-weight: 700; box-shadow: none; }
                    .community-name-input:focus-visible { border-color: var(--color-brand-ink); outline: 2px solid var(--color-brand-ink); outline-offset: 2px; }
                    .community-description { min-height: 110px; resize: vertical; border: 0; border-radius: 5px; background: var(--color-sunken); padding: 12px 16px; font-size: 16px; line-height: 1.5; }
                    .community-description::placeholder { color: var(--color-muted-foreground); }
                    .community-guidelines-preview { white-space: pre-wrap; overflow-wrap: anywhere; font-size: 16px; line-height: 1.5; padding-right: 24px; }
                    .community-guidelines-edit { color: var(--color-ink); padding: 4px; border-radius: 4px; font-size: 13px; }
                    .community-guidelines-edit:hover { background: var(--color-hover); }
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
                    const [detailsOpen, setDetailsOpen] = pp.state(false);
                    const [working, setWorking] = pp.state(false);
                    const [workspaceError, setWorkspaceError] = pp.state('');
                    const activeCommunity = communities.find(c => c.id === selectedCommunity) || null;
                    const activeChannel = activeCommunity?.channels.find(c => c.id === channelId) || null;
                    const activePosts = (activeCommunity?.posts.filter(p => p.channel_id === channelId) || []).slice().sort((a, b) => Number(!!b.pinned) - Number(!!a.pinned));
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
                        setSelectedCommunity(id); selectChannel(''); setDetailsOpen(false); setEventsView('');
                        const url = new URL(location.href); url.searchParams.set('community', id); history.replaceState(null, '', url);
                    }
                    function selectChannel(id) { setChannelId(id); setActiveTab('posts'); setWorkspaceError(''); setEventsView(''); setManageTab(''); }
                    // Manage community: the gear swaps the channel tabs for Teams'
                    // Members, Pending members, Blocked and Settings. Only owners see
                    // past Members.
                    const [manageTab, setManageTab] = pp.state('');
                    const [managed, setManaged] = pp.state([]);
                    const [manageLoading, setManageLoading] = pp.state(false);
                    const [manageWorking, setManageWorking] = pp.state(false);
                    const [manageError, setManageError] = pp.state('');
                    const [manageNotice, setManageNotice] = pp.state('');
                    const [manageTransfer, setManageTransfer] = pp.state('');
                    const [ownersOpen, setOwnersOpen] = pp.state(true);
                    const [membersOpen, setMembersOpen] = pp.state(true);
                    const [linkSettingsOpen, setLinkSettingsOpen] = pp.state(true);
                    const [joinSettingsOpen, setJoinSettingsOpen] = pp.state(true);
                    const [nicknameEditing, setNicknameEditing] = pp.state(false);
                    const [nicknameDraft, setNicknameDraft] = pp.state('');
                    const manageTicket = pp.ref(0);
                    const allManageTabs = [{key:'members', label:'Members'}, {key:'pending', label:'Pending members'}, {key:'blocked', label:'Blocked'}, {key:'settings', label:'Settings'}];
                    const manageTabs = activeCommunity?.is_owner ? allManageTabs : allManageTabs.slice(0, 1);
                    const manageTabLabel = (allManageTabs.find(tab => tab.key === manageTab) || allManageTabs[0]).label;
                    const managedOwners = managed.filter(person => person.is_owner);
                    const managedMembers = managed.filter(person => !person.is_owner && person.state === 'joined');
                    const managedPending = managed.filter(person => person.state === 'invited' || person.state === 'requested');
                    const managedBlocked = managed.filter(person => person.state === 'blocked');
                    const manageTransferName = (() => { const person = managed.find(p => p.id === manageTransfer); return person ? person.nickname || person.name : ''; })();
                    function personInitials(person) {
                        return String(person.nickname || person.name || '?').split(/\s+/).filter(Boolean).slice(0, 2).map(part => part[0].toUpperCase()).join('') || '?';
                    }
                    const loadManaged = pp.event(async function loadManaged() {
                        const id = selectedCommunity, ticket = ++manageTicket.current;
                        if (!id) return;
                        setManageLoading(true); setManageError('');
                        try {
                            const people = await pp.rpc('community_members', {id});
                            if (ticket === manageTicket.current) setManaged(people);
                            return people;
                        } catch (failure) {
                            if (ticket === manageTicket.current) setManageError(failure.message || 'Members could not be loaded.');
                        } finally {
                            if (ticket === manageTicket.current) setManageLoading(false);
                        }
                    });
                    function toggleManage() {
                        if (manageTab) { setManageTab(''); return; }
                        setDetailsOpen(false); setEventsView(''); setManageNotice(''); setManageTransfer(''); setNicknameEditing(false); setManaged([]);
                        setManageTab('members'); loadManaged();
                    }
                    function showManageTab(key) {
                        if (!manageTabs.some(tab => tab.key === key)) return;
                        setManageTab(key); setManageNotice(''); setManageError(''); setManageTransfer('');
                    }
                    function handleManageTabKeys(event) {
                        const keys = manageTabs.map(tab => tab.key);
                        let index = keys.indexOf(manageTab);
                        if (event.key === 'ArrowRight') index = (index + 1) % keys.length;
                        else if (event.key === 'ArrowLeft') index = (index + keys.length - 1) % keys.length;
                        else if (event.key === 'Home') index = 0;
                        else if (event.key === 'End') index = keys.length - 1;
                        else return;
                        event.preventDefault(); showManageTab(keys[index]);
                        const tabs = event.currentTarget.querySelectorAll('[role=tab]');
                        requestAnimationFrame(() => tabs[index]?.focus());
                    }
                    // Each owner action reloads the people, so every tab stays current.
                    const manageAction = pp.event(async function manageAction(call, notice) {
                        if (manageWorking) return;
                        const id = selectedCommunity;
                        setManageWorking(true); setManageError(''); setManageNotice('');
                        try {
                            await call(id);
                            setManageNotice(notice);
                            const people = await loadManaged();
                            if (people) updateCommunity(id, () => ({member_count: Math.max(people.filter(person => person.state === 'joined').length, 1)}));
                        } catch (failure) { setManageError(failure.message || 'That change could not be saved.'); }
                        finally { setManageWorking(false); }
                    });
                    function nameOf(target) { const person = managed.find(p => p.id === target); return person ? person.nickname || person.name : 'This person'; }
                    function removeManaged(target) { manageAction(id => pp.rpc('remove_community_member', {id, target}), nameOf(target) + ' was removed from the community.'); }
                    function blockManaged(target) { manageAction(id => pp.rpc('block_community_member', {id, target}), nameOf(target) + ' was blocked.'); }
                    function unblockManaged(target) { manageAction(id => pp.rpc('unblock_community_member', {id, target}), nameOf(target) + ' was unblocked.'); }
                    function approveManaged(target) { manageAction(id => pp.rpc('approve_community_member', {id, target}), nameOf(target) + ' joined the community.'); }
                    function declineManaged(target) {
                        const person = managed.find(p => p.id === target);
                        manageAction(id => pp.rpc('decline_community_member', {id, target}), person?.state === 'requested' ? 'Request declined.' : 'Invitation canceled.');
                    }
                    function chooseRole(event) {
                        const select = event.currentTarget, target = select.dataset.person;
                        if (select.value === 'owner') setManageTransfer(target);
                        select.value = 'member';
                    }
                    const confirmManageTransfer = pp.event(async function confirmManageTransfer() {
                        if (manageWorking || !manageTransfer) return;
                        const id = selectedCommunity, target = manageTransfer, name = manageTransferName;
                        setManageWorking(true); setManageError('');
                        try {
                            await pp.rpc('transfer_community_ownership', {id, target});
                            updateCommunity(id, () => ({is_owner:false}));
                            setManageTransfer(''); setManageTab('members');
                            setManageNotice(name + ' is now the owner. You are a member.');
                            await loadManaged();
                        } catch (failure) { setManageError(failure.message || 'Ownership could not be transferred.'); }
                        finally { setManageWorking(false); }
                    });
                    function startNickname() {
                        const me = managed.find(person => person.is_me);
                        setNicknameDraft(me?.nickname || ''); setNicknameEditing(true);
                    }
                    function nicknameKey(event) {
                        if (event.key !== 'Escape') return;
                        event.preventDefault(); setNicknameEditing(false);
                    }
                    const saveNickname = pp.event(async function saveNickname(event) {
                        event.preventDefault();
                        if (manageWorking) return;
                        const id = selectedCommunity;
                        setManageWorking(true); setManageError('');
                        try {
                            const nickname = await pp.rpc('set_community_nickname', {id, nickname:nicknameDraft});
                            setNicknameEditing(false);
                            setManageNotice(nickname ? 'Your nickname is now ' + nickname + '.' : 'Nickname removed.');
                            await loadManaged();
                        } catch (failure) {
                            setManageError(failure.errors?.nickname?.[0] || failure.message || 'Your nickname could not be saved.');
                        } finally { setManageWorking(false); }
                    });
                    const createNewLink = pp.event(async function createNewLink() {
                        if (manageWorking) return;
                        const id = selectedCommunity;
                        setManageWorking(true); setManageError(''); setManageNotice('');
                        try {
                            const link = await pp.rpc('community_join_link', {id, reset:true});
                            const url = location.origin + link;
                            try { await navigator.clipboard.writeText(url); setManageNotice('New link created and copied. Previous links no longer work.'); }
                            catch (_) { setManageNotice('New link created: ' + url); }
                        } catch (failure) { setManageError(failure.message || 'A new link could not be created.'); }
                        finally { setManageWorking(false); }
                    });
                    const saveCommunitySetting = pp.event(async function saveCommunitySetting(name, value) {
                        const community = activeCommunity;
                        if (!community || manageWorking) return;
                        const next = {members_can_share: community.members_can_share, require_approval: community.require_approval, [name]: value};
                        setManageWorking(true); setManageError(''); setManageNotice('');
                        updateCommunity(community.id, () => next);
                        try {
                            const saved = await pp.rpc('save_community_settings', {id:community.id, ...next});
                            updateCommunity(community.id, () => ({members_can_share:saved.members_can_share, require_approval:saved.require_approval, can_share_link:saved.can_share_link}));
                            setManageNotice('Settings saved.');
                        } catch (failure) {
                            updateCommunity(community.id, () => ({members_can_share:community.members_can_share, require_approval:community.require_approval}));
                            setManageError(failure.message || 'Settings could not be saved.');
                        } finally { setManageWorking(false); }
                    });
                    // Events: Teams' "Community events" view in place of the channel,
                    // and its New event form. Each occurrence is listed until it ends.
                    const eventBanners = @{Json(&communities::BANNERS)};
                    const eventTitleInput = pp.ref(null);
                    const eventsTicket = pp.ref(0);
                    const [eventsView, setEventsView] = pp.state('');
                    const [events, setEvents] = pp.state([]);
                    const [eventsLoading, setEventsLoading] = pp.state(false);
                    const [eventsError, setEventsError] = pp.state('');
                    const [eventNotice, setEventNotice] = pp.state('');
                    const [eventSaving, setEventSaving] = pp.state(false);
                    const [eventError, setEventError] = pp.state('');
                    const localZone = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
                    const [eventZone, setEventZone] = pp.state(localZone);
                    const [eventBanner, setEventBanner] = pp.state(eventBanners[0]);
                    const [eventStartDate, setEventStartDate] = pp.state('');
                    const [eventStartTime, setEventStartTime] = pp.state('09:00');
                    const [eventEndDate, setEventEndDate] = pp.state('');
                    const [eventEndTime, setEventEndTime] = pp.state('09:30');
                    const [eventAllDay, setEventAllDay] = pp.state(false);
                    const [eventRepeat, setEventRepeat] = pp.state('none');
                    const [eventOccurrences, setEventOccurrences] = pp.state(4);
                    const [eventOnline, setEventOnline] = pp.state(true);
                    function zoneLabel(zone) {
                        try {
                            const offset = new Intl.DateTimeFormat('en-US', {timeZone:zone, timeZoneName:'longOffset'}).formatToParts(new Date()).find(part => part.type === 'timeZoneName')?.value || 'GMT';
                            return '(' + (offset === 'GMT' ? 'UTC' : offset.replace('GMT', 'UTC')) + ') ' + zone.replace(/_/g, ' ');
                        } catch (_) { return zone; }
                    }
                    const eventZones = Array.from(new Set([localZone, @{app_zone},'UTC', 'America/Managua', 'America/Mexico_City', 'America/New_York', 'America/Los_Angeles', 'Europe/London', 'Europe/Madrid', 'Asia/Tokyo'])).filter(zone => zone).map(value => ({value, label: zoneLabel(value)}));
                    function twoDigits(value) { return String(value).padStart(2, '0'); }
                    function slotLabel(value) {
                        const [hour, minute] = value.split(':').map(Number);
                        return twoDigits(hour % 12 || 12) + ':' + twoDigits(minute) + (hour < 12 ? ' AM' : ' PM');
                    }
                    const timeSlots = Array.from({length:48}, (_, index) => { const value = twoDigits(Math.floor(index / 2)) + ':' + (index % 2 ? '30' : '00'); return {value, label:slotLabel(value)}; });
                    function localDay(date) { return date.getFullYear() + '-' + twoDigits(date.getMonth() + 1) + '-' + twoDigits(date.getDate()); }
                    function wallClock(day, time) { const [y, m, d] = day.split('-').map(Number); const [h, min] = time.split(':').map(Number); return new Date(y, m - 1, d, h, min); }
                    function eventMinutes() {
                        if (!eventStartDate || !eventEndDate) return 0;
                        return Math.round((wallClock(eventEndDate, eventEndTime) - wallClock(eventStartDate, eventStartTime)) / 60000);
                    }
                    const eventDuration = (() => {
                        const minutes = eventMinutes();
                        if (minutes <= 0) return '';
                        const hours = Math.floor(minutes / 60), rest = minutes % 60;
                        return (hours ? hours + 'h' : '') + (hours && rest ? ' ' : '') + (rest ? rest + 'm' : '');
                    })();
                    // Moving the start keeps the length, as Teams does.
                    function moveEventStart(day, time) {
                        if (!day) return;
                        const length = Math.max(eventMinutes(), 30);
                        const end = new Date(wallClock(day, time).getTime() + length * 60000);
                        setEventStartDate(day); setEventStartTime(time);
                        setEventEndDate(localDay(end)); setEventEndTime(twoDigits(end.getHours()) + ':' + twoDigits(end.getMinutes()));
                    }
                    const loadEvents = pp.event(async function loadEvents() {
                        const id = selectedCommunity, ticket = ++eventsTicket.current;
                        if (!id) return;
                        setEventsLoading(true); setEventsError('');
                        try {
                            const listed = await pp.rpc('community_events', {id});
                            if (ticket === eventsTicket.current) setEvents(listed);
                        } catch (failure) {
                            if (ticket === eventsTicket.current) setEventsError(failure.message || 'Events could not be loaded.');
                        } finally {
                            if (ticket === eventsTicket.current) setEventsLoading(false);
                        }
                    });
                    function openEvents() { setDetailsOpen(false); setEventNotice(''); setEvents([]); setEventsView('list'); loadEvents(); }
                    function closeEvents() { setEventsView(''); }
                    function newEvent() {
                        const start = new Date(); start.setSeconds(0, 0);
                        start.setMinutes(start.getMinutes() < 30 ? 30 : 60);
                        const end = new Date(start.getTime() + 30 * 60000);
                        setEventStartDate(localDay(start)); setEventStartTime(twoDigits(start.getHours()) + ':' + twoDigits(start.getMinutes()));
                        setEventEndDate(localDay(end)); setEventEndTime(twoDigits(end.getHours()) + ':' + twoDigits(end.getMinutes()));
                        setEventZone(localZone); setEventAllDay(false); setEventRepeat('none'); setEventOccurrences(4); setEventOnline(true);
                        setEventBanner(eventBanners[Math.floor(Math.random() * eventBanners.length)]);
                        setEventError(''); setEventNotice(''); setEventsView('new'); clearEventEditor();
                        setTimeout(() => {
                            const input = eventTitleInput.current, form = input?.form;
                            if (!input) return;
                            ['title', 'location'].forEach(name => { if (form.elements[name]) form.elements[name].value = ''; });
                            input.focus();
                        }, 0);
                    }
                    function nextEventBanner() { setEventBanner(eventBanners[(eventBanners.indexOf(eventBanner) + 1) % eventBanners.length]); }
                    function closeEventForm() { setEventsView('list'); setEventError(''); }
                    function eventFormKey(event) {
                        if (event.key !== 'Escape' || eventSaving) return;
                        event.preventDefault();
                        if (eventStylePanel) { setEventStylePanel(''); eventEditor.current?.focus(); return; }
                        closeEventForm();
                    }
                    async function sendEvent(event) {
                        event.preventDefault();
                        if (eventSaving) return;
                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        if (!String(data.title || '').trim()) { setEventError('Add a title for the event.'); eventTitleInput.current?.focus(); return; }
                        const id = selectedCommunity;
                        setEventSaving(true); setEventError('');
                        try {
                            const created = await pp.rpc('create_community_event', {id, input:{
                                title:data.title, location:data.location || '', timezone:eventZone,
                                details:eventDetailsText, formatting:eventDetailsText ? eventLines : null,
                                start_date:eventStartDate, start_time:eventStartTime, end_date:eventEndDate, end_time:eventEndTime,
                                all_day:eventAllDay, recurrence:eventRepeat, occurrences:eventRepeat === 'none' ? 1 : Number(eventOccurrences) || 1,
                                online:eventOnline, banner:eventBanner,
                            }});
                            if (id === selectedCommunity) {
                                setEvents(current => [...current, ...created].sort((a, b) => new Date(a.starts_at) - new Date(b.starts_at)));
                                setEventNotice(created.length === 1 ? 'Event scheduled.' : created.length + ' events scheduled.');
                                setEventsView('list');
                            }
                        } catch (failure) {
                            const fields = failure.errors || {};
                            setEventError(Object.values(fields).flat()[0] || failure.message || 'The event could not be scheduled.');
                        } finally { setEventSaving(false); }
                    }
                    function eventMonth(item) { return new Date(item.starts_at).toLocaleDateString([], {month:'short'}).toUpperCase(); }
                    function eventDay(item) { return new Date(item.starts_at).getDate(); }
                    function eventWhen(item) {
                        const start = new Date(item.starts_at), end = new Date(item.ends_at);
                        const day = start.toLocaleDateString([], {weekday:'short', month:'short', day:'numeric'});
                        if (item.all_day) return day + ' · All day';
                        const clock = {hour:'numeric', minute:'2-digit'};
                        const sameDay = start.toDateString() === end.toDateString();
                        return day + ' · ' + start.toLocaleTimeString([], clock) + ' – ' + (sameDay ? '' : end.toLocaleDateString([], {month:'short', day:'numeric'}) + ' ') + end.toLocaleTimeString([], clock);
                    }
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
                    // The composer follows Teams: a collapsed card that opens as a
                    // post (subject) or an announcement (colored headline). ProseMirror
                    // owns only the empty editor host, as in chat; the message travels
                    // as chat_format lines and is rendered without HTML.
                    const accents = @{Json(&communities::ACCENTS)};
                    const composeEmojis = @{Json(&compose_emojis)};
                    const colorSwatches = [{value:'#e8ebfa',label:'Default'},{value:'#f1707b',label:'Red'},{value:'#ffaa44',label:'Orange'},{value:'#f8d22a',label:'Yellow'},{value:'#92c353',label:'Green'},{value:'#4fb2f5',label:'Blue'},{value:'#b4a0ff',label:'Purple'},{value:'#9ea2ae',label:'Gray'}];
                    const highlightSwatches = [{value:'#fff100',label:'Yellow highlight'},{value:'#7cfc00',label:'Green highlight'},{value:'#00ffff',label:'Cyan highlight'},{value:'#ff69b4',label:'Pink highlight'},{value:'#ffa500',label:'Orange highlight'},{value:'#c0c0c0',label:'Gray highlight'}];
                    const [composeMode, setComposeMode] = pp.state('');
                    const [editingPostId, setEditingPostId] = pp.state('');
                    const [postSubject, setPostSubject] = pp.state('');
                    const [postHeadline, setPostHeadline] = pp.state('');
                    const [postSubheading, setPostSubheading] = pp.state('');
                    const [postAccent, setPostAccent] = pp.state('blue');
                    const [accentOpen, setAccentOpen] = pp.state(false);
                    const [postRich, setPostRich] = pp.state(null);
                    const [stylePanel, setStylePanel] = pp.state('');
                    const [composeError, setComposeError] = pp.state('');
                    const [linkUrl, setLinkUrl] = pp.state('');
                    const [activeFormats, setActiveFormats] = pp.state({});
                    const [editorReady, setEditorReady] = pp.state(false);
                    const editorHost = pp.ref(null), editor = pp.ref(null), editorLibrary = pp.ref(null);
                    const subjectInput = pp.ref(null), headlineInput = pp.ref(null), linkInput = pp.ref(null), linkSelection = pp.ref(null);
                    const shownSwatches = stylePanel === 'highlight' ? highlightSwatches : colorSwatches;
                    const postLines = postRich ? formattedLines(postRich) : [];
                    const postText = postLines.map(line => line.runs.map(run => run.text).join('')).join('\n').trim();
                    const canPost = composeMode === 'announcement' ? !!postHeadline.trim() : !!(postText || postSubject.trim());
                    function formattedLines(doc) {
                        const lines = [];
                        function visit(node, kind) {
                            const type = node.type;
                            if (type === 'paragraph' || type === 'code_block') {
                                let runs = [];
                                const emit = () => { lines.push({kind:type === 'code_block' ? 'code' : kind || 'paragraph',runs}); runs = []; };
                                for (const child of node.content || []) {
                                    if (child.type === 'hard_break') {emit(); continue;}
                                    const marks = Object.fromEntries((child.marks || []).map(mark => [mark.type,mark.attrs || true]));
                                    (child.text || '').split('\n').forEach((text,index) => {
                                        if (index) emit();
                                        if (text) runs.push({text,bold:!!marks.strong,italic:!!marks.em,underline:!!marks.underline,strike:!!marks.strike,code:!!marks.code,link:marks.link?.href || null,color:marks.color?.value || null,highlight:marks.highlight?.value || null,size:marks.size?.value || null});
                                    });
                                }
                                emit(); return;
                            }
                            const next = {bullet_list:'bullet',ordered_list:'ordered',blockquote:'quote'}[type] || kind;
                            (node.content || []).forEach(child => visit(child,next));
                        }
                        visit(doc,'paragraph');
                        while (lines.length && !lines[lines.length - 1].runs.length) lines.pop();
                        return lines;
                    }
                    function updateActiveFormats() {
                        const view = editor.current;
                        if (!view) return;
                        const {state} = view, {$from,empty,from,to} = state.selection;
                        const marks = state.storedMarks || $from.marks();
                        const active = {};
                        for (const name of ['strong','em','underline','strike']) active[name] = empty ? marks.some(mark => mark.type.name === name) : state.doc.rangeHasMark(from,to,state.schema.marks[name]);
                        for (let depth = $from.depth; depth > 0; depth--) active[$from.node(depth).type.name] = true;
                        setActiveFormats(active);
                    }
                    const editorDispatch = pp.event(transaction => {
                        const view = editor.current;
                        if (!view) return;
                        view.updateState(view.state.apply(transaction));
                        if (transaction.docChanged) { setPostRich(view.state.doc.toJSON()); setComposeError(''); }
                        updateActiveFormats();
                    });
                    const pasteEditorText = pp.event((view,event) => {
                        event.preventDefault();
                        const text = event.clipboardData?.getData('text/plain') || '';
                        if (!working && text) view.dispatch(view.state.tr.insertText(text));
                        return true;
                    });
                    pp.effect(() => {
                        let disposed = false;
                        import('/js/vendor/chat-editor.js').then(lib => {
                            if (disposed || !editorHost.current) return;
                            editorLibrary.current = lib;
                            const simple = name => ({parseDOM:[{tag:name === 'underline' ? 'u' : 's'}],toDOM:() => [name === 'underline' ? 'u' : 's',0]});
                            let marks = lib.schema.spec.marks.addToEnd('underline',simple('underline')).addToEnd('strike',simple('strike'));
                            for (const name of ['color','highlight','size']) marks = marks.addToEnd(name,{attrs:{value:{}},toDOM:mark => ['span',{style:name === 'color' ? 'color:'+mark.attrs.value : name === 'highlight' ? 'background-color:'+mark.attrs.value+';color:#111' : 'font-size:'+({small:'12px',normal:'14px',large:'20px'}[mark.attrs.value] || '14px')},0]});
                            const nodes = lib.addListNodes(lib.schema.spec.nodes,'paragraph block*','block').remove('image').remove('horizontal_rule').remove('heading');
                            const schema = new lib.Schema({nodes,marks});
                            editor.current = new lib.EditorView(editorHost.current,{
                                state:lib.EditorState.create({schema,plugins:[lib.history(),lib.keymap({'Mod-z':lib.undo,'Mod-y':lib.redo,'Mod-Shift-z':lib.redo,'Mod-b':lib.toggleMark(schema.marks.strong),'Mod-i':lib.toggleMark(schema.marks.em),'Mod-u':lib.toggleMark(schema.marks.underline),'Enter':lib.splitListItem(schema.nodes.list_item),'Tab':lib.sinkListItem(schema.nodes.list_item),'Shift-Tab':lib.liftListItem(schema.nodes.list_item)}),lib.keymap(lib.baseKeymap)]}),
                                attributes:{'aria-label':'Message','role':'textbox','aria-multiline':'true'},
                                dispatchTransaction:editorDispatch,
                                handleDOMEvents:{paste:pasteEditorText,drop:(_view,event) => {event.preventDefault(); return true;}}
                            });
                            setEditorReady(true);
                        }).catch(() => { if (!disposed) setComposeError('The formatting editor could not be loaded. Try again.'); });
                        return () => { disposed = true; editor.current?.destroy(); editor.current = null; };
                    }, []);
                    pp.effect(() => { editor.current?.setProps({editable:() => !working}); }, [working, editorReady]);
                    function resetEditor(doc) {
                        const view = editor.current, lib = editorLibrary.current;
                        if (view && lib) {
                            let state = lib.EditorState.create({schema:view.state.schema,plugins:view.state.plugins});
                            if (doc) {
                                try { state = lib.EditorState.create({schema:view.state.schema,plugins:view.state.plugins,doc:view.state.schema.nodeFromJSON(doc)}); }
                                catch (_) { setComposeError('The post could not be loaded for editing.'); }
                            }
                            view.updateState(state);
                        }
                        setPostRich(doc && view ? view.state.doc.toJSON() : null); setActiveFormats({});
                    }
                    // The inverse of formattedLines: saved lines back into an editor document.
                    function documentFromLines(lines) {
                        const content = [];
                        const textNodes = runs => runs.filter(run => run.text).map(run => {
                            const marks = [];
                            if (run.bold) marks.push({type:'strong'}); if (run.italic) marks.push({type:'em'});
                            if (run.underline) marks.push({type:'underline'}); if (run.strike) marks.push({type:'strike'});
                            if (run.code) marks.push({type:'code'}); if (run.link) marks.push({type:'link',attrs:{href:run.link,title:null}});
                            if (run.color) marks.push({type:'color',attrs:{value:run.color}}); if (run.highlight) marks.push({type:'highlight',attrs:{value:run.highlight}});
                            if (run.size) marks.push({type:'size',attrs:{value:run.size}});
                            return {type:'text',text:run.text,...(marks.length ? {marks} : {})};
                        });
                        const paragraph = runs => { const nodes = textNodes(runs); return {type:'paragraph',...(nodes.length ? {content:nodes} : {})}; };
                        for (const line of lines) {
                            const last = content[content.length - 1];
                            if (line.kind === 'bullet' || line.kind === 'ordered') {
                                const type = line.kind === 'bullet' ? 'bullet_list' : 'ordered_list';
                                const item = {type:'list_item',content:[paragraph(line.runs)]};
                                if (last?.type === type) last.content.push(item); else content.push({type,content:[item]});
                            } else if (line.kind === 'quote') {
                                if (last?.type === 'blockquote') last.content.push(paragraph(line.runs)); else content.push({type:'blockquote',content:[paragraph(line.runs)]});
                            } else if (line.kind === 'code') {
                                const text = line.runs.map(run => run.text).join('');
                                if (last?.type === 'code_block') last.content = [{type:'text',text:(last.content?.[0]?.text || '') + '\n' + text}];
                                else content.push({type:'code_block',...(text ? {content:[{type:'text',text}]} : {})});
                            } else content.push(paragraph(line.runs));
                        }
                        return {type:'doc',content:content.length ? content : [{type:'paragraph'}]};
                    }
                    function startEdit(postId) {
                        const post = activeCommunity?.posts.find(p => p.id === postId);
                        if (!post || !post.is_mine || working) return;
                        const lines = post.formatting || (post.body ? post.body.split('\n').map(text => ({kind:'paragraph',runs:text ? [{text}] : []})) : []);
                        setEditingPostId(post.id); setComposeMode(post.announcement ? 'announcement' : 'post');
                        setPostSubject(post.subject || ''); setPostHeadline(post.headline || ''); setPostSubheading(post.subheading || '');
                        setPostAccent(post.accent || 'blue'); setAccentOpen(false); setStylePanel(''); setComposeError('');
                        resetEditor(documentFromLines(lines));
                        requestAnimationFrame(() => { detailHeader.current?.closest('.community-detail')?.scrollTo({top: 0, behavior: 'smooth'}); editor.current?.focus(); });
                    }
                    function openComposer(mode) {
                        setComposeMode(mode); setComposeError(''); setStylePanel(''); setAccentOpen(false);
                        requestAnimationFrame(() => (mode === 'announcement' ? headlineInput : subjectInput).current?.focus());
                    }
                    function closeComposer() {
                        if (working) return;
                        setComposeMode(''); setEditingPostId(''); setPostSubject(''); setPostHeadline(''); setPostSubheading(''); setPostAccent('blue');
                        setAccentOpen(false); setStylePanel(''); setComposeError(''); setLinkUrl(''); resetEditor();
                    }
                    function toggleAnnouncement() {
                        if (editingPostId) return;
                        const mode = composeMode === 'announcement' ? 'post' : 'announcement';
                        setComposeMode(mode); setAccentOpen(false); setComposeError('');
                        requestAnimationFrame(() => (mode === 'announcement' ? headlineInput : subjectInput).current?.focus());
                    }
                    function chooseAccent(event) { setPostAccent(event.currentTarget.value); setAccentOpen(false); headlineInput.current?.focus(); }
                    function toggleStylePanel(name) { setStylePanel(stylePanel === name ? '' : name); }
                    function formatText(name) {
                        const view = editor.current, lib = editorLibrary.current;
                        if (!view || working) return;
                        const {state} = view;
                        let command;
                        if (state.schema.marks[name]) command = lib.toggleMark(state.schema.marks[name]);
                        else if (name === 'bullet_list' || name === 'ordered_list') command = activeFormats[name] ? lib.liftListItem(state.schema.nodes.list_item) : lib.wrapInList(state.schema.nodes[name]);
                        else if (name === 'blockquote') command = activeFormats.blockquote ? lib.lift : lib.wrapIn(state.schema.nodes.blockquote);
                        else if (name === 'code_block') command = lib.setBlockType(activeFormats.code_block ? state.schema.nodes.paragraph : state.schema.nodes.code_block);
                        else if (name === 'clear') {
                            const {from,to,empty} = state.selection;
                            view.dispatch(empty ? state.tr.setStoredMarks([]) : state.tr.removeMark(from,to));
                            setStylePanel('');
                        }
                        if (command) command(state,view.dispatch);
                        view.focus(); updateActiveFormats();
                    }
                    function applyStyle(name,value) {
                        const view = editor.current;
                        if (!view || working) return;
                        const {from,to,empty} = view.state.selection;
                        const tr = name === 'color' && value === '#e8ebfa' ? view.state.tr.removeMark(from,to,view.state.schema.marks.color).removeStoredMark(view.state.schema.marks.color)
                            : empty ? view.state.tr.addStoredMark(view.state.schema.marks[name].create({value})) : view.state.tr.addMark(from,to,view.state.schema.marks[name].create({value}));
                        view.dispatch(tr); setStylePanel(''); view.focus();
                    }
                    function applySwatch(event) { applyStyle(stylePanel, event.currentTarget.value); }
                    const insertEmoji = pp.event(symbol => {
                        const view = editor.current;
                        if (!view || working) return;
                        view.dispatch(view.state.tr.insertText(symbol)); setStylePanel(''); view.focus();
                    });
                    function openLink() {
                        const view = editor.current;
                        if (!view || working) return;
                        if (stylePanel === 'link') { setStylePanel(''); return; }
                        linkSelection.current = view.state.selection;
                        const mark = view.state.selection.$from.marks().find(mark => mark.type.name === 'link');
                        setLinkUrl(mark?.attrs.href || ''); setStylePanel('link');
                        requestAnimationFrame(() => linkInput.current?.focus());
                    }
                    function handleLinkKey(event) {
                        if (event.key === 'Enter') { event.preventDefault(); saveLink(); }
                        if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); setStylePanel(''); editor.current?.focus(); }
                    }
                    function saveLink() {
                        const view = editor.current, href = linkUrl.trim();
                        if (!view || !linkSelection.current) return;
                        if (href && !/^https?:\/\/[^\s]+$/i.test(href)) { setComposeError('Enter a full http:// or https:// link.'); return; }
                        const {from,to,empty} = linkSelection.current;
                        let tr = view.state.tr;
                        if (!href) tr = tr.removeMark(from,to,view.state.schema.marks.link);
                        else if (empty) tr = tr.insertText(href,from).addMark(from,from+href.length,view.state.schema.marks.link.create({href}));
                        else tr = tr.addMark(from,to,view.state.schema.marks.link.create({href}));
                        view.dispatch(tr); setStylePanel(''); setComposeError(''); view.focus();
                    }
                    function handleComposerKey(event) {
                        if (!composeMode) return;
                        if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) { event.preventDefault(); publishPost(event); }
                        else if (event.key === 'Enter' && event.target.tagName === 'INPUT' && event.target.type !== 'url') { event.preventDefault(); editor.current?.focus(); }
                        else if (event.key === 'Escape' && (stylePanel || accentOpen)) { event.preventDefault(); setStylePanel(''); setAccentOpen(false); editor.current?.focus(); }
                    }
                    async function publishPost(event) {
                        event.preventDefault(); if (working || !canPost) return;
                        setWorking(true); setComposeError(''); const id = selectedCommunity;
                        const announcement = composeMode === 'announcement';
                        const formatted = postLines.some(line => line.kind !== 'paragraph' || line.runs.some(run => run.bold || run.italic || run.underline || run.strike || run.code || run.link || run.color || run.highlight || run.size));
                        try {
                            const fields = {
                                body:postText,
                                subject: announcement ? '' : postSubject.trim(),
                                headline: announcement ? postHeadline.trim() : '',
                                subheading: announcement ? postSubheading.trim() : '',
                                accent: announcement ? postAccent : '',
                                formatting: formatted ? postLines : null
                            };
                            if (editingPostId) {
                                const post = await pp.rpc('edit_community_post', {id, post: editingPostId, ...fields});
                                updatePost(id, editingPostId, () => post);
                            } else {
                                const post = await pp.rpc('publish_community_post', {id, channel:channelId, announcement, ...fields});
                                updateCommunity(id, c => ({posts:[post, ...c.posts]}));
                            }
                            setWorking(false); closeComposer();
                        } catch (failure) {
                            const fields = failure.errors || {};
                            setComposeError(Object.values(fields).flat()[0] || failure.message || 'Your post could not be saved.');
                        }
                        finally { setWorking(false); }
                    }
                    // The pane scrolls under a sticky header; the collapsed composer
                    // sticks just below it, so the header's height becomes a CSS variable.
                    const detailHeader = pp.ref(null);
                    const [headerHeight, setHeaderHeight] = pp.state(72);
                    const measureHeader = pp.event(() => setHeaderHeight(Math.round(detailHeader.current?.getBoundingClientRect().height || 72)));
                    pp.effect(() => {
                        if (!detailHeader.current) return;
                        const observer = new ResizeObserver(measureHeader);
                        observer.observe(detailHeader.current);
                        return () => observer.disconnect();
                    }, []);
                    // Files tab: every file shared in this channel, newest first. Each
                    // upload becomes a post carrying the file, as in Teams.
                    const fileLimit = @{communities::FILE_LIMIT};
                    const fileInput = pp.ref(null);
                    const [uploading, setUploading] = pp.state(false);
                    const [uploadStatus, setUploadStatus] = pp.state('');
                    const [fileError, setFileError] = pp.state('');
                    const activeFiles = activePosts.filter(p => !p.deleted).flatMap(p => p.files || []).sort((a, b) => b.created_at - a.created_at);
                    const fileKinds = {pdf:'pdf',doc:'word',docx:'word',rtf:'word',odt:'word',xls:'excel',xlsx:'excel',csv:'excel',ods:'excel',ppt:'powerpoint',pptx:'powerpoint',odp:'powerpoint',zip:'archive',rar:'archive','7z':'archive',gz:'archive',txt:'text',md:'text',json:'code',js:'code',rs:'code',html:'code',css:'code',mp4:'video',mov:'video',webm:'video',mp3:'audio',wav:'audio'};
                    function fileExtension(file) { const name = String(file.name || ''); const dot = name.lastIndexOf('.'); return dot > 0 ? name.slice(dot + 1).toLowerCase() : ''; }
                    function fileKind(file) { return String(file.content_type || '').startsWith('image/') ? 'image' : fileKinds[fileExtension(file)] || 'file'; }
                    // Photos tab: every image shared in this channel, newest first.
                    const activePhotos = activeFiles.filter(file => fileKind(file) === 'image');
                    function fileBadge(file) { return (fileExtension(file) || 'file').slice(0, 4).toUpperCase(); }
                    function fileSize(bytes) {
                        if (bytes < 1024) return bytes + ' B';
                        const units = ['KB', 'MB', 'GB']; let value = bytes / 1024, unit = 0;
                        while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit++; }
                        return (value < 10 ? value.toFixed(1) : Math.round(value)) + ' ' + units[unit];
                    }
                    function fileDate(value) {
                        const date = new Date(value), now = new Date();
                        return date.toDateString() === now.toDateString() ? date.toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'}) : date.toLocaleDateString([], {month:'short', day:'numeric', ...(date.getFullYear() === now.getFullYear() ? {} : {year:'numeric'})});
                    }
                    function pickFiles() {
                        if (uploading || !fileInput.current) return;
                        fileInput.current.value = ''; fileInput.current.click();
                    }
                    async function uploadFiles(event) {
                        const files = Array.from(event.currentTarget.files || []);
                        if (!files.length || uploading) return;
                        const tooLarge = files.find(file => file.size > fileLimit || file.size === 0);
                        if (tooLarge) { setFileError(tooLarge.size ? tooLarge.name + ' is larger than 100 MB.' : tooLarge.name + ' is empty.'); return; }
                        setUploading(true); setFileError(''); const id = selectedCommunity, channel = channelId;
                        let done = 0;
                        try {
                            for (const file of files) {
                                setUploadStatus('Uploading ' + (done + 1) + ' of ' + files.length + '…');
                                const post = await pp.rpc('upload_community_file', {id, channel, file});
                                updateCommunity(id, c => ({posts: [post, ...c.posts]}));
                                done++;
                            }
                            setUploadStatus(done === 1 ? 'File uploaded and posted.' : done + ' files uploaded and posted.');
                        } catch (failure) {
                            setUploadStatus('');
                            setFileError((done ? done + ' uploaded. ' : '') + (failure.errors?.file?.join(' ') || failure.message || 'The file could not be uploaded.'));
                        } finally { setUploading(false); }
                    }
                    // "Attach cloud files": pick from the uploader's Dilnaka Storage.
                    // Each pick becomes a post carrying the file, like a device upload.
                    const cloudShareLimit = @{communities::CLOUD_SHARE_LIMIT};
                    const cloudOpener = pp.ref(null);
                    const cloudCloser = pp.ref(null);
                    const cloudTicket = pp.ref(0);
                    const [cloudView, setCloudView] = pp.state('files');
                    const [cloudFolder, setCloudFolder] = pp.state('');
                    const [cloudPath, setCloudPath] = pp.state([]);
                    const [cloudListing, setCloudListing] = pp.state({folders:[], files:[]});
                    const [cloudPicked, setCloudPicked] = pp.state([]);
                    const [cloudLoading, setCloudLoading] = pp.state(false);
                    const [cloudSharing, setCloudSharing] = pp.state(false);
                    const [cloudError, setCloudError] = pp.state('');
                    const openCloudFiles = pp.event(function openCloudFiles() {
                        if (uploading) return;
                        setCloudPicked([]); setCloudError('');
                        setTimeout(() => cloudOpener.current?.click(), 0);
                        loadCloud('files', '');
                    });
                    // Only the newest request may fill the list, so a slow folder
                    // never replaces the one the person moved on to.
                    const loadCloud = pp.event(async function loadCloud(view, folder) {
                        const ticket = ++cloudTicket.current;
                        if (view !== cloudView || folder !== cloudFolder) { setCloudListing({folders:[], files:[]}); setCloudPath([]); }
                        setCloudView(view); setCloudFolder(folder); setCloudError(''); setCloudLoading(true);
                        try {
                            const listing = await pp.rpc('community_cloud_files', {view, folder: folder || null});
                            if (ticket !== cloudTicket.current) return;
                            setCloudListing(listing); setCloudPath(listing.path);
                        } catch (failure) {
                            if (ticket === cloudTicket.current) setCloudError(failure.message || 'Your cloud files could not be loaded.');
                        } finally {
                            if (ticket === cloudTicket.current) setCloudLoading(false);
                        }
                    });
                    function showCloudView(view) { loadCloud(view, ''); }
                    function openCloudFolder(id) { loadCloud('files', id); }
                    function cloudIcon(file) {
                        const type = file.content_type || '';
                        if (type.startsWith('image/')) return 'image';
                        if (type === 'application/pdf' || type.startsWith('text/') || /word|document|sheet|presentation/.test(type)) return 'doc';
                        return 'file';
                    }
                    // Folders first, as drives list them; then by name.
                    const cloudRows = pp.memo(() => {
                        const byName = (a, b) => a.name.localeCompare(b.name, undefined, {numeric:true, sensitivity:'base'});
                        const folders = cloudListing.folders.map(folder => ({kind:'folder', key:'d:' + folder.id, id:folder.id, name:folder.name, icon:'folder', modified:folder.modified, detail:folder.items === 0 ? 'Empty' : fileSize(folder.bytes)})).sort(byName);
                        const files = cloudListing.files.map(file => ({kind:'file', key:'f:' + file.id, id:file.id, name:file.name, icon:cloudIcon(file), modified:file.uploaded, detail:fileSize(file.bytes)}));
                        return [...folders, ...(cloudView === 'files' ? files.sort(byName) : files)];
                    }, [cloudListing, cloudView]);
                    const cloudHeading = cloudView === 'files' ? (cloudPath.length ? cloudPath[cloudPath.length - 1].name : 'My files') : {recent:'Recent', photos:'Photos'}[cloudView];
                    const cloudEmpty = {
                        files: cloudFolder ? 'This folder is empty.' : 'Nothing here yet. Files you add to Dilnaka Storage will show up here.',
                        recent: 'Files you upload to Dilnaka Storage will show up here.',
                        photos: 'Images you upload to Dilnaka Storage will show up here.',
                    }[cloudView];
                    const pickCloudRow = pp.event(function pickCloudRow(key) {
                        const row = cloudRows.find(item => item.key === key);
                        if (!row) return;
                        if (row.kind === 'folder') { openCloudFolder(row.id); return; }
                        if (cloudPicked.includes(row.id)) { setCloudPicked(cloudPicked.filter(id => id !== row.id)); setCloudError(''); return; }
                        if (cloudPicked.length >= cloudShareLimit) { setCloudError('Share up to ' + cloudShareLimit + ' files at a time.'); return; }
                        setCloudPicked([...cloudPicked, row.id]);
                    });
                    const shareCloudFiles = pp.event(async function shareCloudFiles() {
                        if (!cloudPicked.length || cloudSharing) return;
                        const id = selectedCommunity, channel = channelId, files = cloudPicked;
                        setCloudSharing(true); setCloudError('');
                        try {
                            const posts = await pp.rpc('share_community_cloud_files', {id, channel, files});
                            updateCommunity(id, c => ({posts: [...posts.slice().reverse(), ...c.posts]}));
                            setFileError('');
                            setUploadStatus(posts.length === 1 ? 'File shared and posted.' : posts.length + ' files shared and posted.');
                            cloudCloser.current?.click();
                        } catch (failure) {
                            setCloudError(failure.errors?.files?.join(' ') || failure.message || 'The files could not be shared.');
                        } finally { setCloudSharing(false); }
                    });
                    // Reactions and replies under each post, as in a Teams channel.
                    const reactionChoices = @{Json(&communities::REACTIONS)};
                    const reactionNames = {'👍':'Like','❤️':'Heart','😆':'Laugh','😮':'Surprised','😢':'Sad','😠':'Angry'};
                    const [reacting, setReacting] = pp.state(false);
                    function emojiImage(glyph) { return channelEmojiImage(glyph) || '/favicon.ico'; }
                    // The hover toolbar's "More reactions" and the round button under a
                    // post share one Fluent emoji picker, placed beside whichever opened it.
                    const reactionPopover = pp.ref(null);
                    const [reactionPostId, setReactionPostId] = pp.state('');
                    const [reactionPosition, setReactionPosition] = pp.state('');
                    const [reactionPanelOpen, setReactionPanelOpen] = pp.state(false);
                    const [reactionSession, setReactionSession] = pp.state(0);
                    // While a picker or menu opened from a post's toolbar is open, the
                    // toolbar stays shown beside it, as in Teams.
                    const [reactionFromToolbar, setReactionFromToolbar] = pp.state(false);
                    function prepareReactionPicker(postId, event, fromToolbar) {
                        setReactionPostId(postId); setReactionFromToolbar(!!fromToolbar); setReactionSession(reactionSession + 1);
                        const rect = event.currentTarget.getBoundingClientRect(), width = Math.min(420, innerWidth - 24), height = 470;
                        const left = Math.max(12, Math.min(rect.right - width, innerWidth - width - 12));
                        const top = rect.bottom + 6 + height < innerHeight ? rect.bottom + 6 : Math.max(12, rect.top - height - 6);
                        setReactionPosition('left:' + left + 'px;top:' + top + 'px;width:' + width + 'px');
                    }
                    function reactionPopoverToggled(event) { setReactionPanelOpen(event.newState === 'open'); }
                    const onReactionEmoji = pp.event(glyph => { if (reactionPostId) toggleReaction(reactionPostId, glyph); });
                    // The "…" menu: copy link, delete, pin and open in a new window.
                    const postMenu = pp.ref(null);
                    const [menuPostId, setMenuPostId] = pp.state('');
                    const [postMenuPosition, setPostMenuPosition] = pp.state('');
                    const [postMenuOpen, setPostMenuOpen] = pp.state(false);
                    const menuPost = activeCommunity?.posts.find(p => p.id === menuPostId) || null;
                    function canDelete(post) { return !!post && (post.is_mine || !!activeCommunity?.is_owner); }
                    function preparePostMenu(postId, event) {
                        setMenuPostId(postId); const rect = event.currentTarget.getBoundingClientRect();
                        setPostMenuPosition('left:' + Math.max(12, Math.min(rect.right - 270, innerWidth - 282)) + 'px;top:' + Math.max(12, Math.min(rect.bottom + 6, innerHeight - 240)) + 'px;width:270px');
                    }
                    function postMenuItems() { return Array.from(postMenu.current?.querySelectorAll('[role=menuitem]') || []).filter(item => !item.hidden); }
                    const focusPostMenu = pp.event(() => postMenuItems()[0]?.focus());
                    function postMenuToggled(event) { setPostMenuOpen(event.newState === 'open'); if (event.newState === 'open') requestAnimationFrame(focusPostMenu); }
                    function postMenuKey(event) {
                        const items = postMenuItems(), index = items.indexOf(document.activeElement);
                        if (event.key === 'Delete' && canDelete(menuPost)) { event.preventDefault(); deletePost(menuPostId); return; }
                        if ((event.key === 'p' || event.key === 'P') && activeCommunity?.is_owner) { event.preventDefault(); togglePin(menuPostId); return; }
                        if (event.key === 'Tab') { postMenu.current?.hidePopover(); return; }
                        let next;
                        if (event.key === 'ArrowDown') next = (index + 1) % items.length;
                        else if (event.key === 'ArrowUp') next = (index + items.length - 1) % items.length;
                        else if (event.key === 'Home') next = 0;
                        else if (event.key === 'End') next = items.length - 1;
                        else return;
                        event.preventDefault(); items[next]?.focus();
                    }
                    function postLink(postId) { return location.origin + '/communities?community=' + encodeURIComponent(selectedCommunity) + '&post=' + encodeURIComponent(postId); }
                    async function copyPostLink(postId) {
                        postMenu.current?.hidePopover();
                        try { await navigator.clipboard.writeText(postLink(postId)); setNotice('Link copied.'); }
                        catch (_) { setWorkspaceError('The link could not be copied. Copy it from the address bar instead.'); }
                    }
                    function openConversationWindow(postId) {
                        postMenu.current?.hidePopover();
                        window.open(postLink(postId), '_blank', 'popup,width=760,height=820,noopener');
                    }
                    async function deletePost(postId) {
                        postMenu.current?.hidePopover();
                        if (working) return; setWorking(true); setWorkspaceError(''); const id = selectedCommunity;
                        try { const post = await pp.rpc('delete_community_post', {id, post: postId}); updatePost(id, postId, () => post); if (editingPostId === postId) closeComposer(); }
                        catch (failure) { setWorkspaceError(failure.message || 'The post could not be deleted.'); }
                        finally { setWorking(false); }
                    }
                    async function restorePost(postId) {
                        if (working) return; setWorking(true); setWorkspaceError(''); const id = selectedCommunity;
                        try { const post = await pp.rpc('delete_community_post', {id, post: postId, restore: true}); updatePost(id, postId, () => post); }
                        catch (failure) { setWorkspaceError(failure.message || 'The post could not be restored.'); }
                        finally { setWorking(false); }
                    }
                    async function togglePin(postId) {
                        postMenu.current?.hidePopover();
                        const target = activeCommunity?.posts.find(p => p.id === postId);
                        if (working || !target) return; setWorking(true); setWorkspaceError(''); const id = selectedCommunity;
                        try { const post = await pp.rpc('pin_community_post', {id, post: postId, pinned: !target.pinned}); updatePost(id, postId, () => post); setNotice(post.pinned ? 'Post pinned for everyone.' : 'Post unpinned.'); }
                        catch (failure) { setWorkspaceError(failure.message || 'The post could not be pinned.'); }
                        finally { setWorking(false); }
                    }
                    // A link to a post opens its channel, expands the conversation and
                    // scrolls the post into view with a brief highlight.
                    const focusPostId = @{Json(&focus_post)};
                    const [focusedPost, setFocusedPost] = pp.state('');
                    const revealPost = pp.event(postId => {
                        const card = document.querySelector('.community-post[data-post="' + CSS.escape(postId) + '"]');
                        card?.scrollIntoView({block: 'center'}); setFocusedPost(postId);
                        setTimeout(() => setFocusedPost(''), 2500);
                    });
                    pp.effect(() => {
                        const post = focusPostId && activeCommunity?.posts.find(p => p.id === focusPostId);
                        if (!post) return;
                        selectChannel(post.channel_id);
                        setOpenConversations(current => ({...current, [post.id]: true}));
                        requestAnimationFrame(() => requestAnimationFrame(() => revealPost(post.id)));
                    }, []);
                    const [openConversations, setOpenConversations] = pp.state({});
                    const [replyingTo, setReplyingTo] = pp.state('');
                    const [replyDraft, setReplyDraft] = pp.state('');
                    const [replying, setReplying] = pp.state(false);
                    const [replyError, setReplyError] = pp.state({post:'', message:''});
                    function updatePost(communityId, postId, change) {
                        updateCommunity(communityId, c => ({posts: c.posts.map(p => p.id === postId ? {...p, ...change(p)} : p)}));
                    }
                    function reactionLabel(reaction) {
                        return (reactionNames[reaction.emoji] || reaction.emoji) + ': ' + reaction.count + (reaction.mine ? ', including you' : '');
                    }
                    function conversationLabel(post) {
                        const count = post.replies.length;
                        if (openConversations[post.id]) return 'Hide conversation';
                        return count ? count + (count === 1 ? ' reply' : ' replies') : 'View conversation';
                    }
                    async function toggleReaction(postId, emoji) {
                        if (reacting) return;
                        setReacting(true); const id = selectedCommunity;
                        try {
                            const reactions = await pp.rpc('react_to_community_post', {id, post: postId, emoji});
                            updatePost(id, postId, () => ({reactions}));
                        } catch (failure) { setReplyError({post: postId, message: failure.message || 'Your reaction could not be saved.'}); }
                        finally { setReacting(false); }
                    }
                    function toggleConversation(postId) { setOpenConversations({...openConversations, [postId]: !openConversations[postId]}); }
                    function startReply(postId, event) {
                        const card = event.currentTarget.closest('article');
                        if (replyingTo !== postId) setReplyDraft('');
                        setReplyingTo(postId); setReplyError({post:'', message:''});
                        setOpenConversations({...openConversations, [postId]: true});
                        requestAnimationFrame(() => card?.querySelector('.community-reply-form textarea')?.focus());
                    }
                    function replyKey(postId, event) {
                        if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); sendReply(postId, event); }
                        else if (event.key === 'Escape') { event.preventDefault(); setReplyingTo(''); setReplyDraft(''); }
                    }
                    async function sendReply(postId, event) {
                        event.preventDefault();
                        const body = replyDraft.trim();
                        if (replying || !body) return;
                        setReplying(true); setReplyError({post:'', message:''}); const id = selectedCommunity;
                        try {
                            const reply = await pp.rpc('reply_to_community_post', {id, post: postId, body});
                            updatePost(id, postId, p => ({replies: [...p.replies, reply]}));
                            setReplyDraft(''); setReplyingTo('');
                            setOpenConversations({...openConversations, [postId]: true});
                        } catch (failure) { setReplyError({post: postId, message: failure.errors?.body?.join(' ') || failure.message || 'Your reply could not be sent.'}); }
                        finally { setReplying(false); }
                    }
                    function postTime(value) {
                        const date = new Date(value), now = new Date();
                        const time = date.toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'});
                        if (date.toDateString() === now.toDateString()) return time;
                        return date.toLocaleDateString([], {month:'numeric', day:'numeric', ...(date.getFullYear() === now.getFullYear() ? {} : {year:'numeric'})}) + ' ' + time;
                    }
                    function postLineClass(kind) { return 'community-post-line' + (kind && kind !== 'paragraph' ? ' is-' + kind : ''); }
                    function postListNumber(lines,index) { let start = index; while (start > 0 && lines[start-1].kind === 'ordered') start--; return index - start + 1; }
                    function postRunClass(run) { return [run.bold?'font-bold':'',run.italic?'italic':'',run.underline?'underline':'',run.strike?'line-through':'',run.code?'community-inline-code':'',run.link?'community-post-link':''].join(' '); }
                    function postRunStyle(run) { return (run.color?'color:'+run.color+';':'')+(run.highlight?'background-color:'+run.highlight+';color:#111;':'')+(run.size?'font-size:'+({small:'12px',normal:'14px',large:'20px'}[run.size] || '14px')+';':''); }
                    // The New event details: a second ProseMirror editor with Teams'
                    // toolbar. Like a post, the details travel as chat_format lines.
                    const eventEditorHost = pp.ref(null), eventEditor = pp.ref(null), eventLinkInput = pp.ref(null), eventLinkSelection = pp.ref(null);
                    const [eventEditorReady, setEventEditorReady] = pp.state(false);
                    const [eventRich, setEventRich] = pp.state(null);
                    const [eventFormats, setEventFormats] = pp.state({});
                    const [eventStylePanel, setEventStylePanel] = pp.state('');
                    const [eventLinkUrl, setEventLinkUrl] = pp.state('');
                    const eventSwatches = eventStylePanel === 'highlight' ? highlightSwatches : colorSwatches;
                    const eventLines = eventRich ? formattedLines(eventRich) : [];
                    const eventDetailsText = eventLines.map(line => line.runs.map(run => run.text).join('')).join('\n').trim();
                    const eventHasList = eventLines.some(line => line.kind !== 'paragraph');
                    const eventBlockStyle = eventFormats.code_block ? 'code' : eventFormats.blockquote ? 'quote' : 'paragraph';
                    function updateEventFormats() {
                        const view = eventEditor.current;
                        if (!view) return;
                        const {state} = view, {$from,empty,from,to} = state.selection;
                        const marks = state.storedMarks || $from.marks();
                        const active = {};
                        for (const name of ['strong','em','underline','strike']) active[name] = empty ? marks.some(mark => mark.type.name === name) : state.doc.rangeHasMark(from,to,state.schema.marks[name]);
                        for (let depth = $from.depth; depth > 0; depth--) active[$from.node(depth).type.name] = true;
                        setEventFormats(active);
                    }
                    const eventEditorDispatch = pp.event(transaction => {
                        const view = eventEditor.current;
                        if (!view) return;
                        view.updateState(view.state.apply(transaction));
                        if (transaction.docChanged) { setEventRich(view.state.doc.toJSON()); setEventError(''); }
                        updateEventFormats();
                    });
                    const pasteEventText = pp.event((view,event) => {
                        event.preventDefault();
                        const text = event.clipboardData?.getData('text/plain') || '';
                        if (!eventSaving && text) view.dispatch(view.state.tr.insertText(text));
                        return true;
                    });
                    pp.effect(() => {
                        let disposed = false;
                        import('/js/vendor/chat-editor.js').then(lib => {
                            if (disposed || !eventEditorHost.current) return;
                            editorLibrary.current = lib;
                            const simple = name => ({parseDOM:[{tag:name === 'underline' ? 'u' : 's'}],toDOM:() => [name === 'underline' ? 'u' : 's',0]});
                            let marks = lib.schema.spec.marks.addToEnd('underline',simple('underline')).addToEnd('strike',simple('strike'));
                            for (const name of ['color','highlight','size']) marks = marks.addToEnd(name,{attrs:{value:{}},toDOM:mark => ['span',{style:name === 'color' ? 'color:'+mark.attrs.value : name === 'highlight' ? 'background-color:'+mark.attrs.value+';color:#111' : 'font-size:'+({small:'12px',normal:'14px',large:'20px'}[mark.attrs.value] || '14px')},0]});
                            const nodes = lib.addListNodes(lib.schema.spec.nodes,'paragraph block*','block').remove('image').remove('horizontal_rule').remove('heading');
                            const schema = new lib.Schema({nodes,marks});
                            eventEditor.current = new lib.EditorView(eventEditorHost.current,{
                                state:lib.EditorState.create({schema,plugins:[lib.history(),lib.keymap({'Mod-z':lib.undo,'Mod-y':lib.redo,'Mod-Shift-z':lib.redo,'Mod-b':lib.toggleMark(schema.marks.strong),'Mod-i':lib.toggleMark(schema.marks.em),'Mod-u':lib.toggleMark(schema.marks.underline),'Enter':lib.splitListItem(schema.nodes.list_item),'Tab':lib.sinkListItem(schema.nodes.list_item),'Shift-Tab':lib.liftListItem(schema.nodes.list_item)}),lib.keymap(lib.baseKeymap)]}),
                                attributes:{'aria-label':'Details','role':'textbox','aria-multiline':'true'},
                                dispatchTransaction:eventEditorDispatch,
                                handleDOMEvents:{paste:pasteEventText,drop:(_view,event) => {event.preventDefault(); return true;}}
                            });
                            setEventEditorReady(true);
                        }).catch(() => { if (!disposed) setEventError('The formatting editor could not be loaded. Try again.'); });
                        return () => { disposed = true; eventEditor.current?.destroy(); eventEditor.current = null; };
                    }, []);
                    pp.effect(() => { eventEditor.current?.setProps({editable:() => !eventSaving}); }, [eventSaving, eventEditorReady]);
                    function clearEventEditor() {
                        const view = eventEditor.current, lib = editorLibrary.current;
                        if (view && lib) view.updateState(lib.EditorState.create({schema:view.state.schema,plugins:view.state.plugins}));
                        setEventRich(null); setEventFormats({}); setEventStylePanel('');
                    }
                    function toggleEventPanel(name) { setEventStylePanel(eventStylePanel === name ? '' : name); }
                    function formatEvent(name) {
                        const view = eventEditor.current, lib = editorLibrary.current;
                        if (!view || !lib || eventSaving) return;
                        const {state} = view, nodes = state.schema.nodes;
                        let command;
                        if (state.schema.marks[name]) command = lib.toggleMark(state.schema.marks[name]);
                        else if (name === 'bullet_list' || name === 'ordered_list') command = eventFormats[name] ? lib.liftListItem(nodes.list_item) : lib.wrapInList(nodes[name]);
                        else if (name === 'blockquote') command = eventFormats.blockquote ? lib.lift : lib.wrapIn(nodes.blockquote);
                        else if (name === 'indent') command = lib.sinkListItem(nodes.list_item);
                        else if (name === 'outdent') command = lib.liftListItem(nodes.list_item);
                        else if (name === 'undo') command = lib.undo;
                        else if (name === 'redo') command = lib.redo;
                        else if (name === 'clear') {
                            const {from,to,empty} = state.selection;
                            view.dispatch(empty ? state.tr.setStoredMarks([]) : state.tr.removeMark(from,to));
                        }
                        if (command) command(state,view.dispatch);
                        setEventStylePanel(''); view.focus(); updateEventFormats();
                    }
                    function setEventBlock(style) {
                        const view = eventEditor.current, lib = editorLibrary.current;
                        if (!view || !lib) return;
                        const nodes = view.state.schema.nodes;
                        if (eventFormats.blockquote) lib.lift(view.state,view.dispatch);
                        if (style === 'code') lib.setBlockType(nodes.code_block)(view.state,view.dispatch);
                        else lib.setBlockType(nodes.paragraph)(view.state,view.dispatch);
                        if (style === 'quote') lib.wrapIn(nodes.blockquote)(view.state,view.dispatch);
                        view.focus(); updateEventFormats();
                    }
                    function styleEvent(name, value) {
                        const view = eventEditor.current;
                        if (!view || eventSaving) return;
                        const {from,to,empty} = view.state.selection, mark = view.state.schema.marks[name];
                        const tr = name === 'color' && value === '#e8ebfa' ? view.state.tr.removeMark(from,to,mark).removeStoredMark(mark)
                            : empty ? view.state.tr.addStoredMark(mark.create({value})) : view.state.tr.addMark(from,to,mark.create({value}));
                        view.dispatch(tr); setEventStylePanel(''); view.focus();
                    }
                    function openEventLink() {
                        const view = eventEditor.current;
                        if (!view || eventSaving) return;
                        if (eventStylePanel === 'link') { setEventStylePanel(''); return; }
                        eventLinkSelection.current = view.state.selection;
                        const mark = view.state.selection.$from.marks().find(mark => mark.type.name === 'link');
                        setEventLinkUrl(mark?.attrs.href || ''); setEventStylePanel('link');
                        requestAnimationFrame(() => eventLinkInput.current?.focus());
                    }
                    function eventLinkKey(event) {
                        if (event.key === 'Enter') { event.preventDefault(); saveEventLink(); }
                        if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); setEventStylePanel(''); eventEditor.current?.focus(); }
                    }
                    function saveEventLink() {
                        const view = eventEditor.current, href = eventLinkUrl.trim();
                        if (!view || !eventLinkSelection.current) return;
                        if (href && !/^https?:\/\/[^\s]+$/i.test(href)) { setEventError('Enter a full http:// or https:// link.'); return; }
                        const {from,to,empty} = eventLinkSelection.current;
                        let tr = view.state.tr;
                        if (!href) tr = tr.removeMark(from,to,view.state.schema.marks.link);
                        else if (empty) tr = tr.insertText(href,from).addMark(from,from+href.length,view.state.schema.marks.link.create({href}));
                        else tr = tr.addMark(from,to,view.state.schema.marks.link.create({href}));
                        view.dispatch(tr); setEventStylePanel(''); setEventError(''); view.focus();
                    }
                    const banners = @{Json(&banners)};
                    const bannerOpener = pp.ref(null), bannerCloser = pp.ref(null), bannerInput = pp.ref(null);
                    const [bannerCommunityId, setBannerCommunityId] = pp.state('');
                    const [bannerChoice, setBannerChoice] = pp.state('');
                    const [bannerUpload, setBannerUpload] = pp.state(null);
                    const [bannerUploadPreview, setBannerUploadPreview] = pp.state('');
                    const [bannerError, setBannerError] = pp.state('');
                    const bannerCommunity = communities.find(c => c.id === bannerCommunityId) || null;
                    const bannerCustomImage = bannerUploadPreview || bannerCommunity?.cover_image || '';
                    const bannerChanged = !!bannerChoice && (bannerChoice !== bannerCommunity?.cover || !!bannerUpload);
                    const bannerTabStop = banners.some(b => b.key === bannerChoice) ? bannerChoice : banners[0].key;
                    function bannerSource(cover, image) {
                        if (cover === 'custom' && image) return image;
                        return '/images/banners/' + (banners.some(b => b.key === cover) ? cover : 'mint') + '.svg';
                    }
                    function openBannerDialog() {
                        if (!activeCommunity?.is_owner || working) return;
                        setBannerCommunityId(activeCommunity.id); setBannerChoice(activeCommunity.cover); setBannerUpload(null); setBannerUploadPreview(''); setBannerError('');
                        bannerOpener.current?.click();
                    }
                    function chooseBanner(event) { setBannerChoice(event.currentTarget.value); setBannerError(''); }
                    // Arrow keys move through the two-column grid and select, like a radio group.
                    function moveBannerFocus(event) {
                        const steps = {ArrowRight: 1, ArrowDown: 2, ArrowLeft: -1, ArrowUp: -2};
                        if (!(event.key in steps) && event.key !== 'Home' && event.key !== 'End') return;
                        const choices = Array.from(event.currentTarget.querySelectorAll('[role=radio]')).filter(choice => !choice.hidden);
                        let index = choices.indexOf(document.activeElement);
                        if (event.key === 'Home') index = 0;
                        else if (event.key === 'End') index = choices.length - 1;
                        else index = Math.max(0, Math.min(choices.length - 1, index + steps[event.key]));
                        event.preventDefault();
                        choices[index].focus(); choices[index].scrollIntoView({block: 'nearest'});
                        setBannerChoice(choices[index].value); setBannerError('');
                    }
                    function pickBannerImage() {
                        if (working || !bannerInput.current) return;
                        bannerInput.current.value = ''; bannerInput.current.click();
                    }
                    const bannerLoaded = pp.event(event => { setBannerUploadPreview(String(event.target.result || '')); setBannerChoice('custom'); });
                    const bannerUnreadable = pp.event(() => { setBannerUpload(null); setBannerError('This image could not be read. Choose a valid PNG or JPEG.'); });
                    function bannerFileChosen(event) {
                        const file = event.currentTarget.files?.[0];
                        if (!file) return;
                        if (!['image/png', 'image/jpeg'].includes(file.type) || file.size > 2 * 1024 * 1024) {
                            setBannerError('Choose a PNG or JPEG picture up to 2 MB.'); return;
                        }
                        setBannerError(''); setBannerUpload(file);
                        const reader = new FileReader();
                        reader.onload = bannerLoaded; reader.onerror = bannerUnreadable;
                        reader.readAsDataURL(file);
                    }
                    async function saveBanner() {
                        if (working || !bannerChanged) return;
                        setWorking(true); setBannerError(''); const id = bannerCommunityId;
                        try {
                            const image = bannerChoice === 'custom' ? bannerUpload : null;
                            const saved = await pp.rpc('change_community_cover', {id, cover: bannerChoice, image});
                            updateCommunity(id, () => ({cover: saved.cover, cover_image: saved.cover_image}));
                            setBannerUpload(null); setBannerUploadPreview('');
                            bannerCloser.current?.click();
                        } catch (failure) { setBannerError(failure.errors?.image?.join(' ') || failure.message || 'The banner could not be saved.'); }
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
        cover_image: None,
        channels: Vec::new(),
        posts: Vec::new(),
        is_owner: true,
        notifications: "all".into(),
        member_count: 1,
        can_share_link: true,
        members_can_share: true,
        require_approval: false,
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
    subject: Option<String>,
    headline: Option<String>,
    subheading: Option<String>,
    accent: Option<String>,
    formatting: Option<Vec<crate::chat_format::Line>>,
) -> Result<communities::Post> {
    let draft = communities::PostDraft {
        subject: subject.unwrap_or_default(),
        headline: headline.unwrap_or_default(),
        subheading: subheading.unwrap_or_default(),
        accent: accent.unwrap_or_default(),
        formatting,
    };
    communities::publish(id, channel, body, announcement, draft).await
}
#[rpc(auth)]
pub async fn react_to_community_post(
    id: String,
    post: String,
    emoji: String,
) -> Result<Vec<communities::Reaction>> {
    communities::react(id, post, emoji).await
}
#[rpc(auth)]
pub async fn edit_community_post(
    id: String,
    post: String,
    body: String,
    subject: Option<String>,
    headline: Option<String>,
    subheading: Option<String>,
    accent: Option<String>,
    formatting: Option<Vec<crate::chat_format::Line>>,
) -> Result<communities::Post> {
    let draft = communities::PostDraft {
        subject: subject.unwrap_or_default(),
        headline: headline.unwrap_or_default(),
        subheading: subheading.unwrap_or_default(),
        accent: accent.unwrap_or_default(),
        formatting,
    };
    communities::edit_post(id, post, body, draft).await
}
#[rpc(auth)]
pub async fn delete_community_post(
    id: String,
    post: String,
    restore: Option<bool>,
) -> Result<communities::Post> {
    communities::delete_post(id, post, restore.unwrap_or(false)).await
}
#[rpc(auth)]
pub async fn pin_community_post(
    id: String,
    post: String,
    pinned: bool,
) -> Result<communities::Post> {
    communities::pin_post(id, post, pinned).await
}
#[rpc(auth)]
pub async fn upload_community_file(
    id: String,
    channel: String,
    file: RpcFile,
) -> Result<communities::Post> {
    communities::share_file(id, channel, file).await
}
/// The Events view: a community's events that have not ended yet.
#[rpc(auth)]
pub async fn community_events(id: String) -> Result<Vec<community_events::CommunityEvent>> {
    community_events::upcoming(id).await
}
/// "Send" on the New event form.
#[rpc(auth)]
pub async fn create_community_event(
    id: String,
    input: community_events::EventInput,
) -> Result<Vec<community_events::CommunityEvent>> {
    community_events::create(id, input).await
}
/// Share files picked from the uploader's Dilnaka Storage to the Files tab.
#[rpc(auth)]
pub async fn share_community_cloud_files(
    id: String,
    channel: String,
    files: Vec<String>,
) -> Result<Vec<communities::Post>> {
    communities::share_cloud_files(id, channel, files).await
}
/// One view of the Dilnaka Storage picker: My files (a folder), Recent or Photos.
#[rpc(auth)]
pub async fn community_cloud_files(
    view: String,
    folder: Option<String>,
) -> Result<chat_files::Listing> {
    chat_files::browse(&view, folder).await
}
#[rpc(auth)]
pub async fn reply_to_community_post(
    id: String,
    post: String,
    body: String,
) -> Result<communities::Reply> {
    communities::reply(id, post, body).await
}
#[rpc(auth)]
pub async fn change_community_cover(
    id: String,
    cover: String,
    image: Option<RpcFile>,
) -> Result<communities::Banner> {
    let image = match image {
        Some(file) => Some(uploaded_image(file, "image", 1600, 400, Encoding::Jpeg).await?),
        None => None,
    };
    communities::cover(id, cover, image).await
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
/// "Set nickname" on the viewer's own row of the Members tab.
#[rpc(auth)]
pub async fn set_community_nickname(id: String, nickname: String) -> Result<String> {
    communities::set_nickname(id, nickname).await
}
/// The owner's Settings tab toggles.
#[rpc(auth)]
pub async fn save_community_settings(
    id: String,
    members_can_share: bool,
    require_approval: bool,
) -> Result<Community> {
    communities::settings(id, members_can_share, require_approval).await
}
/// Pending members: approve a request to join.
#[rpc(auth)]
pub async fn approve_community_member(id: String, target: String) -> Result<bool> {
    communities::approve(id, target).await
}
/// Pending members: decline a request, or withdraw an invitation.
#[rpc(auth)]
pub async fn decline_community_member(id: String, target: String) -> Result<bool> {
    communities::decline(id, target).await
}
#[rpc(auth)]
pub async fn block_community_member(id: String, target: String) -> Result<bool> {
    communities::block(id, target).await
}
#[rpc(auth)]
pub async fn unblock_community_member(id: String, target: String) -> Result<bool> {
    communities::unblock(id, target).await
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

async fn community_picture(picture: RpcFile) -> Result<String> {
    uploaded_image(picture, "picture", 256, 256, Encoding::Png).await
}

/// How an upload is re-encoded: lossless for the small card thumbnail, JPEG
/// for the banner so a photograph stays small in every community payload.
#[derive(Clone, Copy)]
enum Encoding {
    Png,
    Jpeg,
}

/// Decode the content rather than trusting the upload's name or MIME type.
/// Return only a bounded, re-encoded image cropped to `width` × `height`:
/// a square thumbnail for the community card, a wide one for its banner.
async fn uploaded_image(
    upload: RpcFile,
    field: &'static str,
    width: u32,
    height: u32,
    encoding: Encoding,
) -> Result<String> {
    use base64::Engine;

    let mut validator = Validator::new();
    validator.check(
        field,
        !upload.is_empty() && upload.len() <= 2 * 1024 * 1024,
        "Choose a PNG or JPEG picture up to 2 MB.",
    );
    validator.finish()?;
    let bytes = upload.into_bytes().await?;
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
            let resized =
                decoded.resize_to_fill(width, height, image::imageops::FilterType::Lanczos3);
            let mut output = std::io::Cursor::new(Vec::new());
            match encoding {
                Encoding::Png => resized.write_to(&mut output, image::ImageFormat::Png),
                Encoding::Jpeg => resized.to_rgb8().write_with_encoder(
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 85),
                ),
            }
            .map_err(|_| "This picture could not be processed.".to_string())?;
            Ok(output.into_inner())
        })
        .await?;
    let mut validator = Validator::new();
    validator.check(
        field,
        normalized.is_ok(),
        normalized
            .as_ref()
            .err()
            .map(String::as_str)
            .unwrap_or("Choose a valid picture."),
    );
    validator.finish()?;
    let mime = match encoding {
        Encoding::Png => "image/png",
        Encoding::Jpeg => "image/jpeg",
    };
    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(normalized.unwrap())
    ))
}
