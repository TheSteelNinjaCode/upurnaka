//! Chat attachments: files uploaded from the composer, kept in Dilnaka
//! Storage like meeting recordings, and readable only by whoever uploaded
//! them — chats are not delivered to anybody else yet.

use axum::http::StatusCode;
use dilnaka::UploadOptions;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::Serialize;

use crate::db;
use crate::meetings::{mint_id, recording};
use crate::models::chat_file;
use crate::rahti::{Error, Result, RpcFile, Validator, time};

/// The largest single attachment, inside the framework's 128 MiB request cap.
pub const FILE_LIMIT: usize = 100 * 1024 * 1024;
/// How many files one message may carry.
pub const PER_MESSAGE: usize = 10;
/// How long a download link stays valid.
const ACCESS_SECONDS: u64 = 600;

/// One attachment, as the browser shows it.
#[derive(Clone, Serialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub bytes: i64,
    pub uploaded: String,
}

impl From<chat_file::Model> for Attachment {
    fn from(row: chat_file::Model) -> Self {
        Attachment {
            uploaded: time::DateTime::from_timestamp(row.created_at, 0)
                .map(|at| time::to_app_time(at).format("%b %-d, %Y").to_string())
                .unwrap_or_default(),
            id: row.id,
            name: row.name,
            content_type: row.content_type,
            bytes: row.bytes,
        }
    }
}

fn owner() -> Result<String> {
    crate::auth::current()
        .map(|user| user.id)
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Authentication required"))
}

/// Upload one file from this device to storage.
pub async fn store(file: RpcFile) -> Result<Attachment> {
    let owner = owner()?;
    let name: String = file.name().trim().chars().take(200).collect();
    let mut validator = Validator::new();
    validator
        .check("file", !file.is_empty(), "Choose a file that is not empty.")
        .check(
            "file",
            file.len() <= FILE_LIMIT,
            "Files can be up to 100 MB.",
        )
        .check("file", !name.is_empty(), "The file needs a name.");
    validator.finish()?;
    if !recording::is_configured() {
        return Err(Error::public(
            StatusCode::SERVICE_UNAVAILABLE,
            "File storage is not configured for this workspace. Contact your administrator.",
        ));
    }

    let id = mint_id();
    let content_type = file
        .content_type()
        .filter(|kind| !kind.is_empty())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = file.len() as i64;
    let folder = std::env::temp_dir().join("upurnaka-chat-files").join(&id);
    let path = folder.join(file.safe_name().unwrap_or_else(|| "file".into()));
    let stored = async {
        tokio::fs::create_dir_all(&folder)
            .await
            .map_err(|error| error.to_string())?;
        file.save(&path).await.map_err(|error| error.to_string())?;
        let mut metadata = dilnaka::Metadata::new();
        metadata.insert("chatFile".into(), id.clone().into());
        metadata.insert("owner".into(), owner.clone().into());
        let options = UploadOptions::default()
            .folder(format!("chat/{owner}"))
            .content_type(content_type.clone())
            .metadata(metadata);
        recording::upload(&path, options).await
    }
    .await;
    let _ = tokio::fs::remove_dir_all(&folder).await;
    let stored = stored.map_err(|reason| {
        eprintln!("chat files: {id} could not be stored: {reason}");
        Error::public(
            StatusCode::BAD_GATEWAY,
            "The file could not be uploaded to storage. Try again shortly.",
        )
    })?;

    let row = chat_file::ActiveModel {
        id: Set(id),
        owner_id: Set(owner),
        name: Set(name),
        content_type: Set(content_type),
        bytes: Set(bytes),
        file_id: Set(stored.id),
        file_key: Set(stored.key),
        created_at: Set(time::Utc::now().timestamp()),
    }
    .insert(db::db())
    .await?;
    Ok(row.into())
}

/// This person's uploaded files, newest first, for the cloud file picker.
pub async fn recent() -> Result<Vec<Attachment>> {
    let rows = chat_file::Entity::find()
        .filter(chat_file::Column::OwnerId.eq(owner()?))
        .order_by_desc(chat_file::Column::CreatedAt)
        .limit(100)
        .all(db::db())
        .await?;
    Ok(rows.into_iter().map(Attachment::from).collect())
}

/// The attachments a message names, refusing any this person does not own.
pub async fn owned(ids: &[String]) -> Result<Vec<Attachment>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = chat_file::Entity::find()
        .filter(chat_file::Column::OwnerId.eq(owner()?))
        .filter(chat_file::Column::Id.is_in(ids.iter().cloned()))
        .all(db::db())
        .await?;
    // Keep the order the composer showed them in.
    ids.iter()
        .map(|id| {
            rows.iter()
                .find(|row| &row.id == id)
                .cloned()
                .map(Attachment::from)
                .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "An attached file was not found."))
        })
        .collect()
}

/// A short-lived link to download one of this person's files.
pub async fn link(id: &str) -> Result<String> {
    let row = chat_file::Entity::find_by_id(id)
        .filter(chat_file::Column::OwnerId.eq(owner()?))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "File not found."))?;
    recording::signed_url(&row.file_id, ACCESS_SECONDS)
        .await
        .map_err(|reason| {
            eprintln!("chat files: no access URL for {}: {reason}", row.id);
            Error::public(
                StatusCode::BAD_GATEWAY,
                "File storage could not be reached. Try again shortly.",
            )
        })
}
