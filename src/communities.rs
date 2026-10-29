//! Durable, account-owned communities. The database is authoritative on every load.
use crate::{
    auth, db,
    models::{community as row, community_channel as channel, community_post as post},
    rahti::{Error, Result, Validator},
};
use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    sea_query::Expr,
};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Serialize)]
pub struct Post {
    pub id: String,
    pub channel_id: String,
    pub body: String,
    pub announcement: bool,
    pub created_at: i64,
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
    pub channels: Vec<Channel>,
    pub posts: Vec<Post>,
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
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Community not found."))
}
async fn shown(row: row::Model) -> Result<Community> {
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
        })
        .collect();
    let posts = post::Entity::find()
        .filter(post::Column::CommunityId.eq(&row.id))
        .order_by_desc(post::Column::CreatedAt)
        .order_by_desc(post::Column::Id)
        .all(db::db())
        .await?
        .into_iter()
        .map(|p| Post {
            id: p.id,
            channel_id: p.channel_id,
            body: p.body,
            announcement: p.announcement,
            created_at: p.created_at,
        })
        .collect();
    Ok(Community {
        id: row.id,
        name: row.name,
        category: row.category,
        description: row.description,
        guidelines: row.guidelines,
        avatar: row.avatar,
        picture: row.picture,
        cover: row.cover,
        channels,
        posts,
    })
}
pub async fn list() -> Result<Vec<Community>> {
    let rows = row::Entity::find()
        .filter(row::Column::OwnerId.eq(owner()?))
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
        created_at: Set(crate::rahti::time::now().timestamp_micros()),
    }
    .insert(db::db())
    .await?;
    Ok(community)
}
pub fn new_id() -> String {
    id()
}
pub async fn add_channel(community: String, name: String) -> Result<Channel> {
    let name = name.trim().to_string();
    let mut v = Validator::new();
    v.required("name", &name).max_chars("name", &name, 60);
    v.finish()?;
    owned(&community).await?;
    let row = channel::ActiveModel {
        id: Set(id()),
        community_id: Set(community),
        name: Set(name),
        created_at: Set(crate::rahti::time::now().timestamp_micros()),
    }
    .insert(db::db())
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
    Ok(Channel {
        id: row.id,
        name: row.name,
    })
}
pub async fn publish(
    community: String,
    channel_id: String,
    body: String,
    announcement: bool,
) -> Result<Post> {
    let body = body.trim().to_string();
    let mut v = Validator::new();
    v.required("body", &body).max_chars("body", &body, 5000);
    v.finish()?;
    owned(&community).await?;
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
    }
    .insert(db::db())
    .await?;
    Ok(Post {
        id: row.id,
        channel_id: row.channel_id,
        body: row.body,
        announcement: row.announcement,
        created_at: row.created_at,
    })
}
pub async fn cover(community: String, cover: String) -> Result<String> {
    let mut v = Validator::new();
    v.one_of("cover", &cover.as_str(), &["mint", "lavender", "sunset"]);
    v.finish()?;
    owned(&community).await?;
    row::Entity::update_many()
        .col_expr(row::Column::Cover, Expr::value(&cover))
        .filter(row::Column::Id.eq(community))
        .filter(row::Column::OwnerId.eq(owner()?))
        .exec(db::db())
        .await?;
    Ok(cover)
}
