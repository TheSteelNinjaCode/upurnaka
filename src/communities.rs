//! Durable, account-owned communities. The database is authoritative on every load.
use crate::{
    auth, chat_format, db,
    models::{
        account, community as row, community_channel as channel, community_file as shared,
        community_member as member, community_post as post, community_post_reaction as reaction,
        community_post_reply as reply, community_report as report,
    },
    rahti::{Error, Result, Validator},
};
use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
    sea_query::{Expr, OnConflict},
};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub icon: String,
}
#[derive(Clone, Serialize)]
pub struct Post {
    pub id: String,
    pub channel_id: String,
    pub body: String,
    pub announcement: bool,
    pub created_at: i64,
    pub author_name: String,
    pub author_is_owner: bool,
    pub subject: String,
    pub headline: String,
    pub subheading: String,
    pub accent: String,
    pub formatting: Option<Vec<chat_format::Line>>,
    pub reactions: Vec<Reaction>,
    pub replies: Vec<Reply>,
    /// Files uploaded with this post, from the Files tab.
    pub files: Vec<SharedFile>,
    /// The viewer wrote it, so they may edit it.
    pub is_mine: bool,
    pub edited: bool,
    pub pinned: bool,
    /// Deleted posts stay in the feed as a placeholder with no content.
    pub deleted: bool,
}
/// One reaction kind on a post: how many people chose it and whether the
/// viewer is one of them.
#[derive(Clone, Serialize)]
pub struct Reaction {
    pub emoji: String,
    pub count: usize,
    pub mine: bool,
}
/// A file shared to a community, as the Files tab and its post show it.
#[derive(Clone, Serialize)]
pub struct SharedFile {
    pub id: String,
    pub post_id: String,
    pub name: String,
    pub content_type: String,
    pub bytes: i64,
    pub created_at: i64,
    pub uploader_name: String,
}
/// The largest file a community accepts, inside the framework's request cap.
pub const FILE_LIMIT: usize = 100 * 1024 * 1024;
#[derive(Clone, Serialize)]
pub struct Reply {
    pub id: String,
    pub body: String,
    pub created_at: i64,
    pub author_name: String,
    pub author_is_owner: bool,
}
/// The Teams quick reactions on the hover toolbar. Any bundled Fluent emoji
/// is accepted from the full picker.
pub const REACTIONS: &[&str] = &["👍", "❤️", "😆", "😮"];
/// Group reactions by emoji in the order each was first chosen.
fn summarize(rows: &[&reaction::Model], viewer: &str) -> Vec<Reaction> {
    let mut rows = rows.to_vec();
    rows.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
    let mut result: Vec<Reaction> = Vec::new();
    for row in rows {
        match result.iter_mut().find(|r| r.emoji == row.emoji) {
            Some(reaction) => {
                reaction.count += 1;
                reaction.mine |= row.user_id == viewer;
            }
            None => result.push(Reaction {
                emoji: row.emoji.clone(),
                count: 1,
                mine: row.user_id == viewer,
            }),
        }
    }
    result
}
/// What the composer sends besides the plain text. Everything is optional so
/// a plain post stays `{body, announcement}`.
#[derive(Default, serde::Deserialize)]
#[serde(default)]
pub struct PostDraft {
    pub subject: String,
    pub headline: String,
    pub subheading: String,
    pub accent: String,
    pub formatting: Option<Vec<chat_format::Line>>,
}
/// Announcement header colors, named so the browser owns the exact shades.
pub const ACCENTS: &[&str] = &[
    "blue", "teal", "green", "purple", "magenta", "red", "orange", "slate",
];
fn shown_post(
    p: post::Model,
    author_name: String,
    author_is_owner: bool,
    reactions: Vec<Reaction>,
    replies: Vec<Reply>,
    files: Vec<SharedFile>,
    viewer: &str,
) -> Post {
    let is_mine = p.author_id == viewer;
    let (edited, pinned) = (p.edited_at.is_some(), p.pinned_at.is_some());
    if p.deleted_at.is_some() {
        return Post {
            id: p.id,
            channel_id: p.channel_id,
            body: String::new(),
            announcement: false,
            created_at: p.created_at,
            author_name,
            author_is_owner,
            subject: String::new(),
            headline: String::new(),
            subheading: String::new(),
            accent: String::new(),
            formatting: None,
            reactions: Vec::new(),
            replies,
            files: Vec::new(),
            is_mine,
            edited: false,
            pinned: false,
            deleted: true,
        };
    }
    Post {
        id: p.id,
        channel_id: p.channel_id,
        body: p.body,
        announcement: p.announcement,
        created_at: p.created_at,
        author_name,
        author_is_owner,
        subject: p.subject,
        headline: p.headline,
        subheading: p.subheading,
        accent: p.accent,
        formatting: rahti::serde_json::from_str(&p.content).ok(),
        reactions,
        replies,
        files,
        is_mine,
        edited,
        pinned,
        deleted: false,
    }
}
#[derive(Clone, Serialize)]
pub struct Community {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub guidelines: String,
    pub avatar: String,
    pub picture: Option<String>,
    pub cover: String,
    pub cover_image: Option<String>,
    pub channels: Vec<Channel>,
    pub posts: Vec<Post>,
    pub is_owner: bool,
    pub notifications: String,
    pub member_count: usize,
    /// The viewer may copy the join link: owners always, members when allowed.
    pub can_share_link: bool,
    pub members_can_share: bool,
    pub require_approval: bool,
}
pub(crate) fn owner() -> Result<String> {
    auth::ready()?;
    let id = auth::current()
        .map(|u| u.id)
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Sign in to manage communities."))?;
    crate::rahti::authorization::authorize_resource("communities.manage", &id)?;
    Ok(id)
}
pub(crate) fn id() -> String {
    use rand::TryRng;
    let mut bytes = [0; 16];
    rand::rngs::SysRng
        .try_fill_bytes(&mut bytes)
        .expect("system randomness");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
async fn owned(id: &str) -> Result<row::Model> {
    row::Entity::find_by_id(id)
        .filter(row::Column::OwnerId.eq(owner()?))
        .filter(row::Column::DeletedAt.is_null())
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Community not found."))
}
pub(crate) async fn accessible(id: &str) -> Result<row::Model> {
    let user = owner()?;
    let row = row::Entity::find_by_id(id)
        .filter(row::Column::DeletedAt.is_null())
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Community not found."))?;
    if row.owner_id != user
        && member::Entity::find()
            .filter(member::Column::CommunityId.eq(id))
            .filter(member::Column::UserId.eq(user))
            .filter(member::Column::State.eq("joined"))
            .one(db::db())
            .await?
            .is_none()
    {
        return Err(Error::new(StatusCode::NOT_FOUND, "Community not found."));
    }
    Ok(row)
}
pub(crate) async fn author_name(id: &str) -> Result<String> {
    Ok(account::Entity::find_by_id(id)
        .one(db::db())
        .await?
        .map(|u| u.name)
        .unwrap_or_else(|| "Former member".into()))
}
/// What `user` is called in `community`: their nickname there, else their name.
pub(crate) async fn display_name(community: &str, user: &str) -> Result<String> {
    let nickname = member::Entity::find()
        .filter(member::Column::CommunityId.eq(community))
        .filter(member::Column::UserId.eq(user))
        .one(db::db())
        .await?
        .map(|m| m.nickname)
        .unwrap_or_default();
    if nickname.is_empty() {
        author_name(user).await
    } else {
        Ok(nickname)
    }
}
async fn shown(row: row::Model) -> Result<Community> {
    let user = owner()?;
    let memberships = member::Entity::find()
        .filter(member::Column::CommunityId.eq(&row.id))
        .all(db::db())
        .await?;
    let notifications = memberships
        .iter()
        .find(|m| m.user_id == user)
        .map(|m| m.notifications.clone())
        .unwrap_or_else(|| "all".into());
    let member_count = 1 + memberships
        .iter()
        .filter(|m| m.user_id != row.owner_id && m.state == "joined")
        .count();
    let channels = channel::Entity::find()
        .filter(channel::Column::CommunityId.eq(&row.id))
        .order_by_asc(channel::Column::CreatedAt)
        .order_by_asc(channel::Column::Id)
        .all(db::db())
        .await?
        .into_iter()
        .map(|c| Channel {
            id: c.id,
            name: c.name,
            icon: c.icon,
        })
        .collect();
    let rows = post::Entity::find()
        .filter(post::Column::CommunityId.eq(&row.id))
        .order_by_desc(post::Column::CreatedAt)
        .order_by_desc(post::Column::Id)
        .all(db::db())
        .await?;
    let reactions = reaction::Entity::find()
        .filter(reaction::Column::CommunityId.eq(&row.id))
        .all(db::db())
        .await?;
    let file_rows = shared::Entity::find()
        .filter(shared::Column::CommunityId.eq(&row.id))
        .order_by_asc(shared::Column::CreatedAt)
        .order_by_asc(shared::Column::Id)
        .all(db::db())
        .await?;
    let reply_rows = reply::Entity::find()
        .filter(reply::Column::CommunityId.eq(&row.id))
        .order_by_asc(reply::Column::CreatedAt)
        .order_by_asc(reply::Column::Id)
        .all(db::db())
        .await?;
    let mut posts = Vec::new();
    for p in rows {
        let author = if p.author_id.is_empty() {
            &row.owner_id
        } else {
            &p.author_id
        };
        let name = display_name(&row.id, author).await?;
        let is_owner = *author == row.owner_id;
        let mine: Vec<_> = reactions.iter().filter(|r| r.post_id == p.id).collect();
        let mut replies = Vec::new();
        for r in reply_rows.iter().filter(|r| r.post_id == p.id) {
            replies.push(Reply {
                id: r.id.clone(),
                body: r.body.clone(),
                created_at: r.created_at,
                author_name: display_name(&row.id, &r.author_id).await?,
                author_is_owner: r.author_id == row.owner_id,
            });
        }
        let reactions = summarize(&mine, &user);
        let mut files = Vec::new();
        for f in file_rows.iter().filter(|f| f.post_id == p.id) {
            files.push(shown_file(f.clone()).await?);
        }
        posts.push(shown_post(
            p, name, is_owner, reactions, replies, files, &user,
        ));
    }
    let is_owner = row.owner_id == user;
    Ok(Community {
        can_share_link: is_owner || row.members_can_share,
        members_can_share: row.members_can_share,
        require_approval: row.require_approval,
        id: row.id,
        name: row.name,
        category: row.category,
        description: row.description,
        guidelines: row.guidelines,
        avatar: row.avatar,
        picture: row.picture,
        cover: row.cover,
        cover_image: row.cover_image,
        channels,
        posts,
        is_owner,
        notifications,
        member_count,
    })
}
pub async fn list() -> Result<Vec<Community>> {
    let user = owner()?;
    let memberships = member::Entity::find()
        .filter(member::Column::UserId.eq(&user))
        .filter(member::Column::State.eq("joined"))
        .all(db::db())
        .await?;
    let rows = row::Entity::find()
        .filter(
            Condition::any()
                .add(row::Column::OwnerId.eq(user))
                .add(row::Column::Id.is_in(memberships.into_iter().map(|m| m.community_id))),
        )
        .filter(row::Column::DeletedAt.is_null())
        .order_by_desc(row::Column::CreatedAt)
        .order_by_desc(row::Column::Id)
        .all(db::db())
        .await?;
    let mut result = Vec::new();
    for row in rows {
        result.push(shown(row).await?);
    }
    Ok(result)
}
pub async fn save(community: Community) -> Result<Community> {
    row::ActiveModel {
        id: Set(community.id.clone()),
        owner_id: Set(owner()?),
        name: Set(community.name.clone()),
        category: Set(community.category.clone()),
        description: Set(community.description.clone()),
        guidelines: Set(community.guidelines.clone()),
        avatar: Set(community.avatar.clone()),
        picture: Set(community.picture.clone()),
        cover: Set(community.cover.clone()),
        cover_image: Set(community.cover_image.clone()),
        created_at: Set(rahti::time::Timestamp::now().as_microsecond()),
        join_token: Set(String::new()),
        deleted_at: Set(None),
        members_can_share: Set(true),
        require_approval: Set(false),
    }
    .insert(db::db())
    .await?;
    Ok(community)
}
pub fn new_id() -> String {
    id()
}
pub async fn add_channel(community: String, name: String, icon: String) -> Result<Channel> {
    let name = name.trim().to_string();
    let mut v = Validator::new();
    v.required("name", &name)
        .max_chars("name", &name, 60)
        .max_chars("icon", &icon, 32)
        .check(
            "icon",
            crate::emoji::supported(&icon),
            "Choose an icon from the emoji picker.",
        );
    v.finish()?;
    owned(&community).await?;
    let transaction = db::db().begin().await?;
    // Serialize count and insert across processes on both supported databases.
    let locked = row::Entity::update_many()
        .col_expr(row::Column::Name, Expr::col(row::Column::Name).into())
        .filter(row::Column::Id.eq(&community))
        .filter(row::Column::OwnerId.eq(owner()?))
        .filter(row::Column::DeletedAt.is_null())
        .exec(&transaction)
        .await?;
    if locked.rows_affected != 1 {
        return Err(Error::new(StatusCode::NOT_FOUND, "Community not found."));
    }
    if channel::Entity::find()
        .filter(channel::Column::CommunityId.eq(&community))
        .count(&transaction)
        .await?
        >= 5
    {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "Owners may add up to 5 channels per community.",
        ));
    }
    if channel::Entity::find()
        .filter(channel::Column::CommunityId.eq(&community))
        .all(&transaction)
        .await?
        .iter()
        .any(|c| c.name.to_lowercase() == name.to_lowercase())
    {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "A channel with this name already exists.",
        ));
    }
    let row = channel::ActiveModel {
        id: Set(id()),
        community_id: Set(community),
        name: Set(name),
        created_at: Set(rahti::time::Timestamp::now().as_microsecond()),
        icon: Set(icon),
    }
    .insert(&transaction)
    .await
    .map_err(|e| {
        if e.sql_err()
            .is_some_and(|s| matches!(s, sea_orm::SqlErr::UniqueConstraintViolation(_)))
        {
            Error::new(
                StatusCode::CONFLICT,
                "A channel with this name already exists.",
            )
        } else {
            Error::from(e)
        }
    })?;
    transaction.commit().await?;
    Ok(Channel {
        id: row.id,
        name: row.name,
        icon: row.icon,
    })
}
/// A post's content after validation, ready to store.
struct Checked {
    body: String,
    subject: String,
    headline: String,
    subheading: String,
    accent: String,
    content: String,
}
fn checked(body: String, announcement: bool, draft: PostDraft) -> Result<Checked> {
    let body = body.trim().to_string();
    let subject = draft.subject.trim().to_string();
    let headline = draft.headline.trim().to_string();
    let subheading = draft.subheading.trim().to_string();
    let accent = if announcement {
        if draft.accent.is_empty() {
            "blue".to_string()
        } else {
            draft.accent
        }
    } else {
        String::new()
    };
    let mut v = Validator::new();
    v.max_chars("body", &body, 5000)
        .max_chars("subject", &subject, 200)
        .max_chars("headline", &headline, 100)
        .max_chars("subheading", &subheading, 200);
    if announcement {
        v.required("headline", &headline)
            .one_of("accent", &accent.as_str(), ACCENTS);
    } else {
        v.check(
            "body",
            !body.is_empty() || !subject.is_empty(),
            "Write a message or add a subject.",
        )
        .check(
            "headline",
            headline.is_empty() && subheading.is_empty(),
            "Only announcements have a headline.",
        );
    }
    v.finish()?;
    if let Some(lines) = &draft.formatting {
        chat_format::validate_within(lines, &body, 5000)?;
    }
    let content = match &draft.formatting {
        Some(lines) => rahti::serde_json::to_string(lines)?,
        None => String::new(),
    };
    Ok(Checked {
        body,
        subject: if announcement { String::new() } else { subject },
        headline,
        subheading,
        accent,
        content,
    })
}
pub async fn publish(
    community: String,
    channel_id: String,
    body: String,
    announcement: bool,
    draft: PostDraft,
) -> Result<Post> {
    let checked = checked(body, announcement, draft)?;
    let community_row = accessible(&community).await?;
    if !channel_id.is_empty()
        && channel::Entity::find_by_id(&channel_id)
            .filter(channel::Column::CommunityId.eq(&community))
            .one(db::db())
            .await?
            .is_none()
    {
        return Err(Error::new(StatusCode::NOT_FOUND, "Channel not found."));
    }
    let row = post::ActiveModel {
        id: Set(id()),
        community_id: Set(community),
        channel_id: Set(channel_id),
        body: Set(checked.body),
        announcement: Set(announcement),
        created_at: Set(rahti::time::Timestamp::now().as_millisecond()),
        author_id: Set(owner()?),
        subject: Set(checked.subject),
        headline: Set(checked.headline),
        subheading: Set(checked.subheading),
        accent: Set(checked.accent),
        content: Set(checked.content),
        edited_at: Set(None),
        deleted_at: Set(None),
        pinned_at: Set(None),
    }
    .insert(db::db())
    .await?;
    let name = display_name(&community_row.id, &row.author_id).await?;
    let is_owner = row.author_id == community_row.owner_id;
    let viewer = row.author_id.clone();
    Ok(shown_post(
        row,
        name,
        is_owner,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &viewer,
    ))
}
async fn shown_file(f: shared::Model) -> Result<SharedFile> {
    Ok(SharedFile {
        uploader_name: display_name(&f.community_id, &f.uploader_id).await?,
        id: f.id,
        post_id: f.post_id,
        name: f.name,
        content_type: f.content_type,
        bytes: f.bytes,
        created_at: f.created_at,
    })
}
/// Upload one file from the Files tab. As in Teams, it shows up in posts: the
/// upload becomes a post by the uploader that carries the file.
pub async fn share_file(
    community: String,
    channel_id: String,
    file: crate::rahti::RpcFile,
) -> Result<Post> {
    use crate::meetings::recording;
    let name: String = file.name().trim().chars().take(200).collect();
    let mut v = Validator::new();
    v.check("file", !file.is_empty(), "Choose a file that is not empty.")
        .check(
            "file",
            file.len() <= FILE_LIMIT,
            "Files can be up to 100 MB.",
        )
        .check("file", !name.is_empty(), "The file needs a name.");
    v.finish()?;
    let community_row = sharing_into(&community, &channel_id).await?;
    if !recording::is_configured() {
        return Err(Error::public(
            StatusCode::SERVICE_UNAVAILABLE,
            "File storage is not configured for this workspace. Contact your administrator.",
        ));
    }
    let user = owner()?;
    let file_row_id = id();
    let content_type = file
        .content_type()
        .filter(|kind| !kind.is_empty())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = file.len() as i64;
    let scratch = std::env::temp_dir()
        .join("upurnaka-community-files")
        .join(&file_row_id);
    let path = scratch.join(file.safe_name().unwrap_or_else(|| "file".into()));
    let stored = async {
        tokio::fs::create_dir_all(&scratch)
            .await
            .map_err(|error| error.to_string())?;
        file.save(&path).await.map_err(|error| error.to_string())?;
        let mut metadata = dilnaka::Metadata::new();
        metadata.insert("communityFile".into(), file_row_id.clone().into());
        metadata.insert("community".into(), community.clone().into());
        let options = dilnaka::UploadOptions::default()
            .folder(format!("community/{community}"))
            .content_type(content_type.clone())
            .metadata(metadata);
        recording::upload(&path, options).await
    }
    .await;
    let _ = tokio::fs::remove_dir_all(&scratch).await;
    let stored = stored.map_err(|reason| {
        eprintln!("community files: {file_row_id} could not be stored: {reason}");
        Error::public(
            StatusCode::BAD_GATEWAY,
            "The file could not be uploaded to storage. Try again shortly.",
        )
    })?;
    let stored = Stored {
        name,
        content_type,
        bytes,
        file_id: stored.id,
        file_key: stored.key,
    };
    post_file(&community_row, channel_id, user, file_row_id, stored).await
}
/// Share files from the uploader's Dilnaka Storage. Each becomes a post that
/// carries it, as a device upload does, and reads the same storage object.
pub async fn share_cloud_files(
    community: String,
    channel_id: String,
    files: Vec<String>,
) -> Result<Vec<Post>> {
    let mut v = Validator::new();
    v.check("files", !files.is_empty(), "Choose at least one file.")
        .check(
            "files",
            files.len() <= CLOUD_SHARE_LIMIT,
            "Share up to 10 files at a time.",
        );
    v.finish()?;
    let community_row = sharing_into(&community, &channel_id).await?;
    let user = owner()?;
    let mut posts = Vec::new();
    for row in crate::chat_files::owned_rows(&files).await? {
        let stored = Stored {
            name: row.name,
            content_type: row.content_type,
            bytes: row.bytes,
            file_id: row.file_id,
            file_key: row.file_key,
        };
        posts.push(
            post_file(
                &community_row,
                channel_id.clone(),
                user.clone(),
                id(),
                stored,
            )
            .await?,
        );
    }
    Ok(posts)
}
/// How many cloud files one share may carry.
pub const CLOUD_SHARE_LIMIT: usize = 10;
/// A file already in storage, ready to be shared.
struct Stored {
    name: String,
    content_type: String,
    bytes: i64,
    file_id: String,
    file_key: String,
}
/// The community a member shares into, refusing a channel it does not hold.
async fn sharing_into(community: &str, channel_id: &str) -> Result<row::Model> {
    let community_row = accessible(community).await?;
    if !channel_id.is_empty()
        && channel::Entity::find_by_id(channel_id)
            .filter(channel::Column::CommunityId.eq(community))
            .one(db::db())
            .await?
            .is_none()
    {
        return Err(Error::new(StatusCode::NOT_FOUND, "Channel not found."));
    }
    Ok(community_row)
}
/// The post that carries a shared file, written with the file in one transaction.
async fn post_file(
    community_row: &row::Model,
    channel_id: String,
    user: String,
    file_row_id: String,
    stored: Stored,
) -> Result<Post> {
    let community = community_row.id.clone();
    let now = rahti::time::Timestamp::now().as_millisecond();
    let post_id = id();
    let transaction = db::db().begin().await?;
    post::ActiveModel {
        id: Set(post_id.clone()),
        community_id: Set(community.clone()),
        channel_id: Set(channel_id),
        body: Set(String::new()),
        announcement: Set(false),
        created_at: Set(now),
        author_id: Set(user.clone()),
        subject: Set(String::new()),
        headline: Set(String::new()),
        subheading: Set(String::new()),
        accent: Set(String::new()),
        content: Set(String::new()),
        edited_at: Set(None),
        deleted_at: Set(None),
        pinned_at: Set(None),
    }
    .insert(&transaction)
    .await?;
    shared::ActiveModel {
        id: Set(file_row_id),
        community_id: Set(community),
        post_id: Set(post_id.clone()),
        uploader_id: Set(user),
        name: Set(stored.name),
        content_type: Set(stored.content_type),
        bytes: Set(stored.bytes),
        file_id: Set(stored.file_id),
        file_key: Set(stored.file_key),
        created_at: Set(now),
    }
    .insert(&transaction)
    .await?;
    transaction.commit().await?;
    reloaded(community_row, &post_id).await
}
/// A short-lived link to a shared file, for members of its community while
/// its post is not deleted.
pub async fn file_link(file_id: &str) -> Result<String> {
    let row = shared::Entity::find_by_id(file_id)
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "File not found."))?;
    post_in(&row.community_id, &row.post_id)
        .await
        .map_err(|_| Error::new(StatusCode::NOT_FOUND, "File not found."))?;
    crate::meetings::recording::signed_url(&row.file_id, 600)
        .await
        .map_err(|reason| {
            eprintln!("community files: no access URL for {}: {reason}", row.id);
            Error::public(
                StatusCode::BAD_GATEWAY,
                "File storage could not be reached. Try again shortly.",
            )
        })
}
/// The post as the viewer now sees it, with its reactions and replies.
async fn reloaded(community_row: &row::Model, post_id: &str) -> Result<Post> {
    let viewer = owner()?;
    let p = post::Entity::find_by_id(post_id)
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Post not found."))?;
    let author = if p.author_id.is_empty() {
        community_row.owner_id.clone()
    } else {
        p.author_id.clone()
    };
    let reactions = reaction::Entity::find()
        .filter(reaction::Column::PostId.eq(post_id))
        .all(db::db())
        .await?;
    let mut files = Vec::new();
    for f in shared::Entity::find()
        .filter(shared::Column::PostId.eq(post_id))
        .order_by_asc(shared::Column::CreatedAt)
        .all(db::db())
        .await?
    {
        files.push(shown_file(f).await?);
    }
    let mut replies = Vec::new();
    for r in reply::Entity::find()
        .filter(reply::Column::PostId.eq(post_id))
        .order_by_asc(reply::Column::CreatedAt)
        .order_by_asc(reply::Column::Id)
        .all(db::db())
        .await?
    {
        replies.push(Reply {
            id: r.id,
            body: r.body,
            created_at: r.created_at,
            author_name: display_name(&community_row.id, &r.author_id).await?,
            author_is_owner: r.author_id == community_row.owner_id,
        });
    }
    Ok(shown_post(
        p,
        display_name(&community_row.id, &author).await?,
        author == community_row.owner_id,
        summarize(&reactions.iter().collect::<Vec<_>>(), &viewer),
        replies,
        files,
        &viewer,
    ))
}
/// Only the author edits a post; the kind (post or announcement) stays.
pub async fn edit_post(
    community: String,
    post_id: String,
    body: String,
    draft: PostDraft,
) -> Result<Post> {
    let (community_row, existing) = post_in(&community, &post_id).await?;
    if existing.author_id != owner()? {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "Only the author can edit this post.",
        ));
    }
    let checked = checked(body, existing.announcement, draft)?;
    post::Entity::update_many()
        .col_expr(post::Column::Body, Expr::value(checked.body))
        .col_expr(post::Column::Subject, Expr::value(checked.subject))
        .col_expr(post::Column::Headline, Expr::value(checked.headline))
        .col_expr(post::Column::Subheading, Expr::value(checked.subheading))
        .col_expr(post::Column::Accent, Expr::value(checked.accent))
        .col_expr(post::Column::Content, Expr::value(checked.content))
        .col_expr(
            post::Column::EditedAt,
            Expr::value(rahti::time::Timestamp::now().as_millisecond()),
        )
        .filter(post::Column::Id.eq(&post_id))
        .filter(post::Column::DeletedAt.is_null())
        .exec(db::db())
        .await?;
    reloaded(&community_row, &post_id).await
}
/// The author or the community owner may delete a post, and undo it.
async fn moderated(community: &str, post_id: &str) -> Result<(row::Model, post::Model)> {
    let community_row = accessible(community).await?;
    let p = post::Entity::find_by_id(post_id)
        .filter(post::Column::CommunityId.eq(community))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Post not found."))?;
    let user = owner()?;
    if p.author_id != user && community_row.owner_id != user {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "Only the author or the community owner can delete this post.",
        ));
    }
    Ok((community_row, p))
}
pub async fn delete_post(community: String, post_id: String, restore: bool) -> Result<Post> {
    let (community_row, _) = moderated(&community, &post_id).await?;
    let deleted_at = (!restore).then(|| rahti::time::Timestamp::now().as_millisecond());
    let mut update = post::Entity::update_many()
        .col_expr(post::Column::DeletedAt, Expr::value(deleted_at))
        .filter(post::Column::Id.eq(&post_id));
    if !restore {
        // A deleted post is no longer pinned for anyone.
        update = update.col_expr(post::Column::PinnedAt, Expr::value(Option::<i64>::None));
    }
    update.exec(db::db()).await?;
    reloaded(&community_row, &post_id).await
}
/// Pinning is for everyone, so only the community owner may do it.
pub async fn pin_post(community: String, post_id: String, pinned: bool) -> Result<Post> {
    let community_row = owned(&community).await?;
    post_in(&community, &post_id).await?;
    let pinned_at = pinned.then(|| rahti::time::Timestamp::now().as_millisecond());
    post::Entity::update_many()
        .col_expr(post::Column::PinnedAt, Expr::value(pinned_at))
        .filter(post::Column::Id.eq(&post_id))
        .exec(db::db())
        .await?;
    reloaded(&community_row, &post_id).await
}
/// A post of a community the viewer belongs to, or 404 for both a missing
/// post and a community they cannot see.
async fn post_in(community: &str, post_id: &str) -> Result<(row::Model, post::Model)> {
    let community_row = accessible(community).await?;
    let post_row = post::Entity::find_by_id(post_id)
        .filter(post::Column::CommunityId.eq(community))
        .filter(post::Column::DeletedAt.is_null())
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Post not found."))?;
    Ok((community_row, post_row))
}
/// Add the viewer's reaction, or take it back if it is already there.
pub async fn react(community: String, post_id: String, emoji: String) -> Result<Vec<Reaction>> {
    let canonical = (emoji.chars().count() <= 16)
        .then(|| crate::emoji::canonical(&emoji))
        .flatten();
    let mut v = Validator::new();
    v.check("emoji", canonical.is_some(), "Choose a supported emoji.");
    v.finish()?;
    let emoji = canonical.unwrap_or_default().to_string();
    post_in(&community, &post_id).await?;
    let user = owner()?;
    let removed = reaction::Entity::delete_many()
        .filter(reaction::Column::PostId.eq(&post_id))
        .filter(reaction::Column::UserId.eq(&user))
        .filter(reaction::Column::Emoji.eq(&emoji))
        .exec(db::db())
        .await?;
    if removed.rows_affected == 0 {
        reaction::Entity::insert(reaction::ActiveModel {
            id: Set(id()),
            community_id: Set(community.clone()),
            post_id: Set(post_id.clone()),
            user_id: Set(user.clone()),
            emoji: Set(emoji),
            created_at: Set(rahti::time::Timestamp::now().as_millisecond()),
        })
        .on_conflict(
            OnConflict::columns([
                reaction::Column::PostId,
                reaction::Column::UserId,
                reaction::Column::Emoji,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec_without_returning(db::db())
        .await?;
    }
    let rows = reaction::Entity::find()
        .filter(reaction::Column::PostId.eq(&post_id))
        .all(db::db())
        .await?;
    Ok(summarize(&rows.iter().collect::<Vec<_>>(), &user))
}
pub async fn reply(community: String, post_id: String, body: String) -> Result<Reply> {
    let body = body.trim().to_string();
    let mut v = Validator::new();
    v.required("body", &body).max_chars("body", &body, 2000);
    v.finish()?;
    let (community_row, _) = post_in(&community, &post_id).await?;
    let user = owner()?;
    let saved = reply::ActiveModel {
        id: Set(id()),
        community_id: Set(community),
        post_id: Set(post_id),
        author_id: Set(user.clone()),
        body: Set(body),
        created_at: Set(rahti::time::Timestamp::now().as_millisecond()),
    }
    .insert(db::db())
    .await?;
    Ok(Reply {
        id: saved.id,
        body: saved.body,
        created_at: saved.created_at,
        author_name: display_name(&community_row.id, &user).await?,
        author_is_owner: user == community_row.owner_id,
    })
}

/// One person on the Members, Pending members or Blocked tab. `state` is
/// `joined`, `invited` (an owner invited them), `requested` (they asked to
/// join by link) or `blocked`; owners see every state, members only `joined`.
#[derive(Serialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    /// Their nickname in this community, or empty.
    pub nickname: String,
    pub email: String,
    pub state: String,
    pub is_owner: bool,
    pub is_me: bool,
}
pub async fn members(community: String) -> Result<Vec<Member>> {
    let row = accessible(&community).await?;
    let viewer = owner()?;
    let rows = member::Entity::find()
        .filter(member::Column::CommunityId.eq(&community))
        .all(db::db())
        .await?;
    let account = |id: &str| {
        let id = id.to_string();
        async move { account::Entity::find_by_id(id).one(db::db()).await }
    };
    let owner_account = account(&row.owner_id).await?;
    let mut result = vec![Member {
        id: row.owner_id.clone(),
        name: owner_account
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Former member".into()),
        nickname: rows
            .iter()
            .find(|m| m.user_id == row.owner_id)
            .map(|m| m.nickname.clone())
            .unwrap_or_default(),
        email: if row.owner_id == viewer {
            owner_account.map(|a| a.email).unwrap_or_default()
        } else {
            String::new()
        },
        state: "joined".into(),
        is_owner: true,
        is_me: row.owner_id == viewer,
    }];
    for m in rows.into_iter().filter(|m| m.user_id != row.owner_id) {
        if row.owner_id != viewer && m.state != "joined" {
            continue;
        }
        if !matches!(
            m.state.as_str(),
            "joined" | "invited" | "requested" | "blocked"
        ) {
            continue;
        }
        let person = account(&m.user_id).await?;
        result.push(Member {
            name: person
                .as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Former member".into()),
            // Only owners manage people by address; members see names.
            email: if row.owner_id == viewer {
                person.map(|a| a.email).unwrap_or_default()
            } else {
                String::new()
            },
            is_me: m.user_id == viewer,
            id: m.user_id,
            nickname: m.nickname,
            state: m.state,
            is_owner: false,
        });
    }
    Ok(result)
}
/// Set or clear the viewer's own nickname in a community.
pub async fn set_nickname(community: String, nickname: String) -> Result<String> {
    let nickname = nickname.trim().to_string();
    let mut v = Validator::new();
    v.max_chars("nickname", &nickname, 60).check(
        "nickname",
        !nickname.chars().any(char::is_control),
        "Nicknames can't contain control characters.",
    );
    v.finish()?;
    let row = accessible(&community).await?;
    let user = owner()?;
    if row.owner_id == user {
        // The owner's preferences row exists only once they set something.
        membership(&community, &user, "preferences", None).await?;
    }
    member::Entity::update_many()
        .col_expr(member::Column::Nickname, Expr::value(&nickname))
        .filter(member::Column::CommunityId.eq(&community))
        .filter(member::Column::UserId.eq(&user))
        .exec(db::db())
        .await?;
    Ok(nickname)
}
/// The owner's Settings tab: who may share the join link, and whether
/// joining by link needs approval.
pub async fn settings(
    community: String,
    members_can_share: bool,
    require_approval: bool,
) -> Result<Community> {
    owned(&community).await?;
    row::Entity::update_many()
        .col_expr(row::Column::MembersCanShare, Expr::value(members_can_share))
        .col_expr(row::Column::RequireApproval, Expr::value(require_approval))
        .filter(row::Column::Id.eq(&community))
        .filter(row::Column::OwnerId.eq(owner()?))
        .exec(db::db())
        .await?;
    shown(owned(&community).await?).await
}
/// The person `target` in `community` in one of `states`, for an owner action.
async fn pending(community: &str, target: &str, states: &[&str]) -> Result<member::Model> {
    member::Entity::find()
        .filter(member::Column::CommunityId.eq(community))
        .filter(member::Column::UserId.eq(target))
        .filter(member::Column::State.is_in(states.iter().copied()))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "This person is no longer pending."))
}
/// Approve a request to join: the person becomes a member.
pub async fn approve(community: String, target: String) -> Result<bool> {
    owned(&community).await?;
    pending(&community, &target, &["requested"]).await?;
    member::Entity::update_many()
        .col_expr(member::Column::State, Expr::value("joined"))
        .filter(member::Column::CommunityId.eq(&community))
        .filter(member::Column::UserId.eq(&target))
        .filter(member::Column::State.eq("requested"))
        .exec(db::db())
        .await?;
    Ok(true)
}
/// Decline a request to join, or withdraw an invitation.
pub async fn decline(community: String, target: String) -> Result<bool> {
    owned(&community).await?;
    pending(&community, &target, &["requested", "invited"]).await?;
    member::Entity::delete_many()
        .filter(member::Column::CommunityId.eq(&community))
        .filter(member::Column::UserId.eq(&target))
        .filter(member::Column::State.is_in(["requested", "invited"]))
        .exec(db::db())
        .await?;
    Ok(true)
}
/// Block someone: they leave the community and can't join or be invited.
pub async fn block(community: String, target: String) -> Result<bool> {
    let row = owned(&community).await?;
    if target == row.owner_id {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "The owner cannot be blocked.",
        ));
    }
    if account::Entity::find_by_id(&target)
        .one(db::db())
        .await?
        .is_none()
    {
        return Err(Error::new(StatusCode::NOT_FOUND, "Person not found."));
    }
    membership(&community, &target, "blocked", None).await?;
    Ok(true)
}
/// Unblock someone, who may then be invited or join again.
pub async fn unblock(community: String, target: String) -> Result<bool> {
    owned(&community).await?;
    pending(&community, &target, &["blocked"]).await?;
    member::Entity::delete_many()
        .filter(member::Column::CommunityId.eq(&community))
        .filter(member::Column::UserId.eq(&target))
        .filter(member::Column::State.eq("blocked"))
        .exec(db::db())
        .await?;
    Ok(true)
}
async fn is_blocked(community: &str, user: &str) -> Result<bool> {
    Ok(member::Entity::find()
        .filter(member::Column::CommunityId.eq(community))
        .filter(member::Column::UserId.eq(user))
        .filter(member::Column::State.eq("blocked"))
        .one(db::db())
        .await?
        .is_some())
}
pub async fn edit(
    community: String,
    name: String,
    description: String,
    guidelines: String,
    picture: Option<String>,
    remove_picture: bool,
) -> Result<Community> {
    let name = name.trim().to_string();
    let mut v = Validator::new();
    v.required("name", &name)
        .min_chars("name", &name, 2)
        .max_chars("name", &name, 80)
        .max_chars("description", &description, 1000)
        .max_chars("guidelines", &guidelines, 2000);
    v.finish()?;
    let old = owned(&community).await?;
    row::Entity::update_many()
        .col_expr(row::Column::Name, Expr::value(name))
        .col_expr(row::Column::Description, Expr::value(description.trim()))
        .col_expr(row::Column::Guidelines, Expr::value(guidelines.trim()))
        .col_expr(
            row::Column::Picture,
            Expr::value(if remove_picture {
                None
            } else {
                picture.or(old.picture)
            }),
        )
        .filter(row::Column::Id.eq(&community))
        .filter(row::Column::OwnerId.eq(owner()?))
        .exec(db::db())
        .await?;
    shown(owned(&community).await?).await
}
pub async fn notifications(community: String, preference: String) -> Result<String> {
    let mut v = Validator::new();
    v.one_of(
        "preference",
        &preference.as_str(),
        &["all", "mentions", "off"],
    );
    v.finish()?;
    let row = accessible(&community).await?;
    let user = owner()?;
    if row.owner_id == user {
        membership(&community, &user, "preferences", Some(&preference)).await?;
    } else {
        let changed = member::Entity::update_many()
            .col_expr(member::Column::Notifications, Expr::value(&preference))
            .filter(member::Column::CommunityId.eq(community))
            .filter(member::Column::UserId.eq(user))
            .filter(member::Column::State.eq("joined"))
            .exec(db::db())
            .await?;
        if changed.rows_affected != 1 {
            return Err(Error::new(StatusCode::NOT_FOUND, "Community not found."));
        }
    }
    Ok(preference)
}
async fn membership(
    community: &str,
    user: &str,
    state: &str,
    notifications: Option<&str>,
) -> Result<()> {
    let mut conflict = OnConflict::columns([member::Column::CommunityId, member::Column::UserId]);
    if notifications.is_some() {
        conflict.update_column(member::Column::Notifications);
    } else {
        conflict.update_column(member::Column::State);
    }
    member::Entity::insert(member::ActiveModel {
        id: Set(id()),
        community_id: Set(community.into()),
        user_id: Set(user.into()),
        state: Set(state.into()),
        notifications: Set(notifications.unwrap_or("all").into()),
        nickname: Set(String::new()),
    })
    .on_conflict(conflict.to_owned())
    .exec(db::db())
    .await?;
    Ok(())
}
/// The join link. Owners may always copy it and replace it (`reset`);
/// members may copy it while the owner allows them to share it.
pub async fn join_link(community: String, reset: bool) -> Result<String> {
    let row = accessible(&community).await?;
    let is_owner = row.owner_id == owner()?;
    if !is_owner && (reset || !row.members_can_share) {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "Only community owners can share this link.",
        ));
    }
    let mut update = row::Entity::update_many()
        .col_expr(row::Column::JoinToken, Expr::value(id()))
        .filter(row::Column::Id.eq(&community))
        .filter(row::Column::DeletedAt.is_null());
    if !reset {
        update = update.filter(row::Column::JoinToken.eq(""));
    }
    update.exec(db::db()).await?;
    Ok(format!(
        "/communities?join={}",
        accessible(&community).await?.join_token
    ))
}
#[derive(Serialize)]
pub struct Invitation {
    pub id: String,
    pub name: String,
    pub description: String,
    pub avatar: String,
}
pub async fn preview(token: &str) -> Result<Option<Invitation>> {
    if token.is_empty() {
        return Ok(None);
    }
    Ok(row::Entity::find()
        .filter(row::Column::JoinToken.eq(token))
        .filter(row::Column::DeletedAt.is_null())
        .one(db::db())
        .await?
        .map(|r| Invitation {
            id: r.id,
            name: r.name,
            description: r.description,
            avatar: r.avatar,
        }))
}
pub async fn invitations() -> Result<Vec<Invitation>> {
    let mut result = Vec::new();
    for m in member::Entity::find()
        .filter(member::Column::UserId.eq(owner()?))
        .filter(member::Column::State.eq("invited"))
        .all(db::db())
        .await?
    {
        if let Some(r) = row::Entity::find_by_id(m.community_id)
            .filter(row::Column::DeletedAt.is_null())
            .one(db::db())
            .await?
        {
            result.push(Invitation {
                id: r.id,
                name: r.name,
                description: r.description,
                avatar: r.avatar,
            });
        }
    }
    Ok(result)
}
pub async fn invite(community: String, email: String) -> Result<bool> {
    let mut v = Validator::new();
    v.required("email", &email)
        .email("email", &email)
        .max_chars("email", &email, 254);
    v.finish()?;
    let row = owned(&community).await?;
    let target = account::Entity::find()
        .filter(account::Column::Email.eq(email.trim().to_lowercase()))
        .filter(account::Column::Active.eq(true))
        .filter(account::Column::Verified.eq(true))
        .one(db::db())
        .await?
        .ok_or_else(|| {
            Error::new(
                StatusCode::NOT_FOUND,
                "No active account matches this email.",
            )
        })?;
    if is_blocked(&community, &target.id).await? {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This person is blocked. Unblock them before inviting them.",
        ));
    }
    if target.id == row.owner_id
        || member::Entity::find()
            .filter(member::Column::CommunityId.eq(&community))
            .filter(member::Column::UserId.eq(&target.id))
            .one(db::db())
            .await?
            .is_some()
    {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This person is already a member or has an invitation.",
        ));
    }
    let changed = member::Entity::insert(member::ActiveModel {
        id: Set(id()),
        community_id: Set(community),
        user_id: Set(target.id),
        state: Set("invited".into()),
        notifications: Set("all".into()),
        nickname: Set(String::new()),
    })
    .on_conflict(
        OnConflict::columns([member::Column::CommunityId, member::Column::UserId])
            .do_nothing()
            .to_owned(),
    )
    .exec_without_returning(db::db())
    .await?;
    if changed == 0 {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This person is already a member or has an invitation.",
        ));
    }
    Ok(true)
}
pub async fn join(community: String, token: Option<String>) -> Result<Community> {
    let user = owner()?;
    let row = row::Entity::find_by_id(&community)
        .filter(row::Column::DeletedAt.is_null())
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Community not found."))?;
    let invited = member::Entity::find()
        .filter(member::Column::CommunityId.eq(&community))
        .filter(member::Column::UserId.eq(&user))
        .filter(member::Column::State.is_in(["invited", "joined"]))
        .one(db::db())
        .await?
        .is_some();
    if row.owner_id != user
        && !invited
        && (row.join_token.is_empty() || token.as_deref() != Some(row.join_token.as_str()))
    {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "A valid invitation is required.",
        ));
    }
    if row.owner_id != user && is_blocked(&community, &user).await? {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "You can't join this community.",
        ));
    }
    // Joining by link waits for an owner when the community asks for approval;
    // an invitation was the owner's approval already.
    if row.owner_id != user && !invited && row.require_approval {
        membership(&community, &user, "requested", None).await?;
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "Your request to join was sent. A community owner must approve it.",
        ));
    }
    membership(&community, &user, "joined", None).await?;
    shown(row).await
}
pub async fn leave(community: String) -> Result<bool> {
    let row = accessible(&community).await?;
    if row.owner_id == owner()? {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "Transfer ownership to another member before leaving this community.",
        ));
    }
    member::Entity::delete_many()
        .filter(member::Column::CommunityId.eq(community))
        .filter(member::Column::UserId.eq(owner()?))
        .exec(db::db())
        .await?;
    Ok(true)
}
pub async fn transfer(community: String, target: String) -> Result<bool> {
    owned(&community).await?;
    if account::Entity::find_by_id(&target)
        .filter(account::Column::Active.eq(true))
        .filter(account::Column::Verified.eq(true))
        .one(db::db())
        .await?
        .is_none()
    {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "Choose an active community member.",
        ));
    }
    if member::Entity::find()
        .filter(member::Column::CommunityId.eq(&community))
        .filter(member::Column::UserId.eq(&target))
        .filter(member::Column::State.eq("joined"))
        .one(db::db())
        .await?
        .is_none()
    {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "Choose a current community member.",
        ));
    }
    let user = owner()?;
    membership(&community, &user, "joined", None).await?;
    let txn = db::db().begin().await?;
    // Preserve attribution of posts saved before per-post authors existed.
    post::Entity::update_many()
        .col_expr(post::Column::AuthorId, Expr::value(&user))
        .filter(post::Column::CommunityId.eq(&community))
        .filter(post::Column::AuthorId.eq(""))
        .exec(&txn)
        .await?;
    let changed = row::Entity::update_many()
        .col_expr(row::Column::OwnerId, Expr::value(target))
        .filter(row::Column::Id.eq(community))
        .filter(row::Column::OwnerId.eq(user))
        .filter(row::Column::DeletedAt.is_null())
        .exec(&txn)
        .await?;
    if changed.rows_affected != 1 {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "Ownership changed. Refresh and try again.",
        ));
    }
    txn.commit().await?;
    Ok(true)
}
pub async fn remove_member(community: String, target: String) -> Result<bool> {
    let row = owned(&community).await?;
    if target == row.owner_id {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "The owner cannot be removed.",
        ));
    }
    member::Entity::delete_many()
        .filter(member::Column::CommunityId.eq(community))
        .filter(member::Column::UserId.eq(target))
        .exec(db::db())
        .await?;
    Ok(true)
}
pub async fn delete(community: String) -> Result<bool> {
    owned(&community).await?;
    row::Entity::update_many()
        .col_expr(
            row::Column::DeletedAt,
            Expr::value(rahti::time::Timestamp::now().as_second()),
        )
        .col_expr(row::Column::JoinToken, Expr::value(""))
        .filter(row::Column::Id.eq(community))
        .filter(row::Column::OwnerId.eq(owner()?))
        .exec(db::db())
        .await?;
    Ok(true)
}
pub async fn report(community: String, details: String) -> Result<String> {
    let mut v = Validator::new();
    v.required("details", &details)
        .min_chars("details", &details, 10)
        .max_chars("details", &details, 2000);
    v.finish()?;
    accessible(&community).await?;
    let ticket = id();
    report::ActiveModel {
        id: Set(ticket.clone()),
        community_id: Set(community),
        reporter_id: Set(owner()?),
        details: Set(details),
        created_at: Set(rahti::time::Timestamp::now().as_second()),
    }
    .insert(db::db())
    .await?;
    Ok(ticket)
}
#[derive(Serialize)]
pub struct Report {
    pub id: String,
    pub community: String,
    pub reporter: String,
    pub details: String,
}
pub async fn reports() -> Result<Vec<Report>> {
    crate::users::require_admin().await?;
    let mut result = Vec::new();
    for r in report::Entity::find()
        .order_by_desc(report::Column::CreatedAt)
        .all(db::db())
        .await?
    {
        let community = row::Entity::find_by_id(r.community_id)
            .one(db::db())
            .await?
            .map(|r| r.name)
            .unwrap_or_else(|| "Deleted community".into());
        result.push(Report {
            id: r.id,
            community,
            reporter: author_name(&r.reporter_id).await?,
            details: r.details,
        });
    }
    Ok(result)
}
/// The preset banners under `public/images/banners/`. `custom` names the
/// uploaded image kept in `cover_image`.
pub const BANNERS: &[&str] = &[
    "stationery",
    "paintbrush",
    "keepsakes",
    "bakery",
    "giftbox",
    "stars",
    "clock",
    "compass",
    "mint",
    "lavender",
    "sunset",
];
#[derive(Clone, Serialize)]
pub struct Banner {
    pub cover: String,
    pub cover_image: Option<String>,
}
/// Choose a preset banner, or `custom` with a newly uploaded image. Choosing
/// `custom` without one keeps the image already uploaded, if there is one.
pub async fn cover(community: String, cover: String, image: Option<String>) -> Result<Banner> {
    let mut v = Validator::new();
    let mut allowed = BANNERS.to_vec();
    allowed.push("custom");
    v.one_of("cover", &cover.as_str(), &allowed);
    v.finish()?;
    let old = owned(&community).await?;
    let cover_image = if cover == "custom" {
        image.or(old.cover_image)
    } else {
        None
    };
    let mut v = Validator::new();
    v.check(
        "image",
        cover != "custom" || cover_image.is_some(),
        "Upload an image for the banner.",
    );
    v.finish()?;
    row::Entity::update_many()
        .col_expr(row::Column::Cover, Expr::value(&cover))
        .col_expr(row::Column::CoverImage, Expr::value(cover_image.clone()))
        .filter(row::Column::Id.eq(community))
        .filter(row::Column::OwnerId.eq(owner()?))
        .exec(db::db())
        .await?;
    Ok(Banner { cover, cover_image })
}
