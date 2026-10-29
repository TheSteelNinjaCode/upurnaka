//! Durable, account-owned communities. The database is authoritative on every load.
use crate::{
    auth, chat_format, db,
    models::{
        account, community as row, community_channel as channel, community_member as member,
        community_post as post, community_post_reaction as reaction, community_post_reply as reply,
        community_report as report,
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
}
/// One reaction kind on a post: how many people chose it and whether the
/// viewer is one of them.
#[derive(Clone, Serialize)]
pub struct Reaction {
    pub emoji: String,
    pub count: usize,
    pub mine: bool,
}
#[derive(Clone, Serialize)]
pub struct Reply {
    pub id: String,
    pub body: String,
    pub created_at: i64,
    pub author_name: String,
    pub author_is_owner: bool,
}
/// The Teams quick reactions, in the order the picker shows them.
pub const REACTIONS: &[&str] = &["👍", "❤️", "😆", "😮", "😢", "😠"];
fn summarize(rows: &[&reaction::Model], viewer: &str) -> Vec<Reaction> {
    REACTIONS
        .iter()
        .filter_map(|emoji| {
            let chosen: Vec<_> = rows.iter().filter(|r| r.emoji == *emoji).collect();
            (!chosen.is_empty()).then(|| Reaction {
                emoji: emoji.to_string(),
                count: chosen.len(),
                mine: chosen.iter().any(|r| r.user_id == viewer),
            })
        })
        .collect()
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
) -> Post {
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
}
fn owner() -> Result<String> {
    auth::ready()?;
    let id = auth::current()
        .map(|u| u.id)
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Sign in to manage communities."))?;
    crate::rahti::authorization::authorize_resource("communities.manage", &id)?;
    Ok(id)
}
fn id() -> String {
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
async fn accessible(id: &str) -> Result<row::Model> {
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
async fn author_name(id: &str) -> Result<String> {
    Ok(account::Entity::find_by_id(id)
        .one(db::db())
        .await?
        .map(|u| u.name)
        .unwrap_or_else(|| "Former member".into()))
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
        let name = author_name(author).await?;
        let is_owner = *author == row.owner_id;
        let mine: Vec<_> = reactions.iter().filter(|r| r.post_id == p.id).collect();
        let mut replies = Vec::new();
        for r in reply_rows.iter().filter(|r| r.post_id == p.id) {
            replies.push(Reply {
                id: r.id.clone(),
                body: r.body.clone(),
                created_at: r.created_at,
                author_name: author_name(&r.author_id).await?,
                author_is_owner: r.author_id == row.owner_id,
            });
        }
        let reactions = summarize(&mine, &user);
        posts.push(shown_post(p, name, is_owner, reactions, replies));
    }
    let is_owner = row.owner_id == user;
    Ok(Community {
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
        created_at: Set(crate::rahti::time::now().timestamp_micros()),
        join_token: Set(String::new()),
        deleted_at: Set(None),
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
        created_at: Set(crate::rahti::time::now().timestamp_micros()),
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
pub async fn publish(
    community: String,
    channel_id: String,
    body: String,
    announcement: bool,
    draft: PostDraft,
) -> Result<Post> {
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
        body: Set(body),
        announcement: Set(announcement),
        created_at: Set(crate::rahti::time::now().timestamp_millis()),
        author_id: Set(owner()?),
        subject: Set(if announcement { String::new() } else { subject }),
        headline: Set(headline),
        subheading: Set(subheading),
        accent: Set(accent),
        content: Set(content),
    }
    .insert(db::db())
    .await?;
    let name = author_name(&row.author_id).await?;
    let is_owner = row.author_id == community_row.owner_id;
    Ok(shown_post(row, name, is_owner, Vec::new(), Vec::new()))
}
/// A post of a community the viewer belongs to, or 404 for both a missing
/// post and a community they cannot see.
async fn post_in(community: &str, post_id: &str) -> Result<(row::Model, post::Model)> {
    let community_row = accessible(community).await?;
    let post_row = post::Entity::find_by_id(post_id)
        .filter(post::Column::CommunityId.eq(community))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Post not found."))?;
    Ok((community_row, post_row))
}
/// Add the viewer's reaction, or take it back if it is already there.
pub async fn react(community: String, post_id: String, emoji: String) -> Result<Vec<Reaction>> {
    let mut v = Validator::new();
    v.one_of("emoji", &emoji.as_str(), REACTIONS);
    v.finish()?;
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
            created_at: Set(crate::rahti::time::now().timestamp_millis()),
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
        created_at: Set(crate::rahti::time::now().timestamp_millis()),
    }
    .insert(db::db())
    .await?;
    Ok(Reply {
        id: saved.id,
        body: saved.body,
        created_at: saved.created_at,
        author_name: author_name(&user).await?,
        author_is_owner: user == community_row.owner_id,
    })
}

#[derive(Serialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub state: String,
    pub is_owner: bool,
}
pub async fn members(community: String) -> Result<Vec<Member>> {
    let row = accessible(&community).await?;
    let mut result = vec![Member {
        id: row.owner_id.clone(),
        name: author_name(&row.owner_id).await?,
        state: "joined".into(),
        is_owner: true,
    }];
    for m in member::Entity::find()
        .filter(member::Column::CommunityId.eq(community))
        .filter(member::Column::UserId.ne(&row.owner_id))
        .all(db::db())
        .await?
    {
        if row.owner_id != owner()? && m.state != "joined" {
            continue;
        }
        result.push(Member {
            name: author_name(&m.user_id).await?,
            id: m.user_id,
            state: m.state,
            is_owner: false,
        });
    }
    Ok(result)
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
    })
    .on_conflict(conflict.to_owned())
    .exec(db::db())
    .await?;
    Ok(())
}
pub async fn join_link(community: String, reset: bool) -> Result<String> {
    owned(&community).await?;
    let mut update = row::Entity::update_many()
        .col_expr(row::Column::JoinToken, Expr::value(id()))
        .filter(row::Column::Id.eq(&community))
        .filter(row::Column::OwnerId.eq(owner()?));
    if !reset {
        update = update.filter(row::Column::JoinToken.eq(""));
    }
    update.exec(db::db()).await?;
    Ok(format!(
        "/communities?join={}",
        owned(&community).await?.join_token
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
            Expr::value(crate::rahti::time::now().timestamp()),
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
        created_at: Set(crate::rahti::time::now().timestamp()),
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
