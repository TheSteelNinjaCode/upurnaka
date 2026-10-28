//! Chat attachments: files uploaded from the composer, kept in Dilnaka
//! Storage like meeting recordings, and readable only by whoever uploaded
//! them — chats are not delivered to anybody else yet.
//!
//! The cloud file picker browses them like a drive: folders are rows here,
//! and each file names the folder it sits in. Dilnaka Storage keeps every
//! object under `chat/{owner}`, so a folder never touches the bucket.

use std::collections::HashMap;

use axum::http::StatusCode;
use dilnaka::UploadOptions;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::Serialize;

use crate::db;
use crate::meetings::{mint_id, recording};
use crate::models::{chat_file, chat_folder};
use crate::rahti::{Error, Result, RpcFile, Validator, time};

/// The largest single attachment, inside the framework's 128 MiB request cap.
pub const FILE_LIMIT: usize = 100 * 1024 * 1024;
/// How many files one message may carry.
pub const PER_MESSAGE: usize = 10;
/// How long a download link stays valid.
const ACCESS_SECONDS: u64 = 600;
/// How many files Recent and Photos show.
const RECENT: u64 = 100;
/// How deep folders may nest, so a path always fits on one line.
const DEPTH_LIMIT: usize = 8;

fn shown_date(at: i64) -> String {
    time::DateTime::from_timestamp(at, 0)
        .map(|at| time::to_app_time(at).format("%b %-d, %Y").to_string())
        .unwrap_or_default()
}

/// One attachment, as the browser shows it.
#[derive(Clone, Serialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub bytes: i64,
    pub uploaded: String,
    pub uploaded_at: i64,
}

impl From<chat_file::Model> for Attachment {
    fn from(row: chat_file::Model) -> Self {
        Attachment {
            uploaded: shown_date(row.created_at),
            uploaded_at: row.created_at,
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

/// A folder in the picker, with what it holds counted all the way down.
#[derive(Clone, Serialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub bytes: i64,
    pub items: i64,
    pub modified: String,
    pub modified_at: i64,
}

/// One step of the path down to the folder being shown.
#[derive(Clone, Serialize)]
pub struct Crumb {
    pub id: String,
    pub name: String,
}

/// What the picker shows for one view: a folder's contents, or a flat list.
#[derive(Clone, Serialize)]
pub struct Listing {
    pub view: String,
    pub folder: Option<String>,
    pub path: Vec<Crumb>,
    pub folders: Vec<Folder>,
    pub files: Vec<Attachment>,
}

/// A folder of this person's, refusing anybody else's.
async fn own_folder(owner: &str, id: &str) -> Result<chat_folder::Model> {
    chat_folder::Entity::find_by_id(id)
        .filter(chat_folder::Column::OwnerId.eq(owner))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Folder not found."))
}

async fn all_folders(owner: &str) -> Result<Vec<chat_folder::Model>> {
    Ok(chat_folder::Entity::find()
        .filter(chat_folder::Column::OwnerId.eq(owner))
        .all(db::db())
        .await?)
}

/// The folders from the top down to `id`, which is last.
fn path_to(folders: &[chat_folder::Model], id: &str) -> Vec<Crumb> {
    let mut path: Vec<Crumb> = Vec::new();
    let mut at = Some(id.to_string());
    while let Some(id) = at.take() {
        let Some(folder) = folders.iter().find(|folder| folder.id == id) else {
            break;
        };
        if path.len() > DEPTH_LIMIT || path.iter().any(|crumb| crumb.id == id) {
            break;
        }
        path.push(Crumb {
            id: folder.id.clone(),
            name: folder.name.clone(),
        });
        at = folder.parent_id.clone();
    }
    path.reverse();
    path
}

/// One view of the picker: `files` browses folders from `folder` (the top
/// when absent); `recent` and `photos` list files newest first; `shared` is
/// what other people sent, and nobody can send files yet.
pub async fn browse(view: &str, folder: Option<String>) -> Result<Listing> {
    let owner = owner()?;
    let mut validator = Validator::new();
    validator.one_of("view", &view, &["files", "recent", "photos", "shared"]);
    validator.finish()?;
    let folder = folder.filter(|id| !id.trim().is_empty() && view == "files");
    let mut listing = Listing {
        view: view.to_string(),
        folder: folder.clone(),
        path: Vec::new(),
        folders: Vec::new(),
        files: Vec::new(),
    };
    match view {
        "recent" | "photos" => {
            let mut query = chat_file::Entity::find().filter(chat_file::Column::OwnerId.eq(&owner));
            if view == "photos" {
                query = query.filter(chat_file::Column::ContentType.starts_with("image/"));
            }
            listing.files = query
                .order_by_desc(chat_file::Column::CreatedAt)
                .limit(RECENT)
                .all(db::db())
                .await?
                .into_iter()
                .map(Attachment::from)
                .collect();
        }
        "files" => {
            if let Some(id) = &folder {
                own_folder(&owner, id).await?;
            }
            let folders = all_folders(&owner).await?;
            let files = chat_file::Entity::find()
                .filter(chat_file::Column::OwnerId.eq(&owner))
                .order_by_desc(chat_file::Column::CreatedAt)
                .all(db::db())
                .await?;
            // Bytes, items and the latest change, counted in every folder
            // above each file and folder.
            let mut totals: HashMap<&str, (i64, i64, i64)> = folders
                .iter()
                .map(|row| (row.id.as_str(), (0, 0, row.created_at)))
                .collect();
            let contents = folders
                .iter()
                .map(|row| (row.parent_id.as_deref(), 0, row.created_at))
                .chain(
                    files
                        .iter()
                        .map(|row| (row.folder_id.as_deref(), row.bytes, row.created_at)),
                );
            for (parent, bytes, at) in contents {
                let Some(parent) = parent else { continue };
                for crumb in path_to(&folders, parent) {
                    if let Some(total) = totals.get_mut(crumb.id.as_str()) {
                        total.0 += bytes;
                        total.1 += 1;
                        total.2 = total.2.max(at);
                    }
                }
            }
            listing.folders = folders
                .iter()
                .filter(|row| row.parent_id == folder)
                .map(|row| {
                    let (bytes, items, modified_at) = totals[row.id.as_str()];
                    Folder {
                        id: row.id.clone(),
                        name: row.name.clone(),
                        bytes,
                        items,
                        modified: shown_date(modified_at),
                        modified_at,
                    }
                })
                .collect();
            listing.files = files
                .into_iter()
                .filter(|row| row.folder_id == folder)
                .map(Attachment::from)
                .collect();
            if let Some(id) = &folder {
                listing.path = path_to(&folders, id);
            }
        }
        _ => {}
    }
    Ok(listing)
}

/// Make a folder inside `parent`, or at the top of My files.
pub async fn create_folder(parent: Option<String>, name: String) -> Result<Folder> {
    let owner = owner()?;
    let parent = parent.filter(|id| !id.trim().is_empty());
    let name = name.trim().to_string();
    let mut validator = Validator::new();
    validator
        .check("name", !name.is_empty(), "Give the folder a name.")
        .check(
            "name",
            name.chars().count() <= 100,
            "Folder names can be up to 100 characters.",
        )
        .check(
            "name",
            !name.contains(['/', '\\']) && name != "." && name != "..",
            "Folder names can't contain / or \\.",
        );
    validator.finish()?;
    let folders = all_folders(&owner).await?;
    if let Some(id) = &parent {
        own_folder(&owner, id).await?;
    }
    let taken = folders
        .iter()
        .any(|row| row.parent_id == parent && row.name.to_lowercase() == name.to_lowercase());
    let depth = parent
        .as_deref()
        .map(|id| path_to(&folders, id).len())
        .unwrap_or_default();
    let mut validator = Validator::new();
    validator
        .check("name", !taken, "A folder with this name is already here.")
        .check(
            "name",
            depth < DEPTH_LIMIT,
            "Folders can't be nested any deeper here.",
        );
    validator.finish()?;

    let now = time::Utc::now().timestamp();
    let row = chat_folder::ActiveModel {
        id: Set(mint_id()),
        owner_id: Set(owner),
        parent_id: Set(parent),
        name: Set(name),
        created_at: Set(now),
    }
    .insert(db::db())
    .await?;
    Ok(Folder {
        id: row.id,
        name: row.name,
        bytes: 0,
        items: 0,
        modified: shown_date(now),
        modified_at: now,
    })
}

/// Upload one file from this device to storage, into a picker folder.
pub async fn store(file: RpcFile, folder: Option<String>) -> Result<Attachment> {
    let owner = owner()?;
    let folder = folder.filter(|id| !id.trim().is_empty());
    if let Some(id) = &folder {
        own_folder(&owner, id).await?;
    }
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
    let scratch = std::env::temp_dir().join("upurnaka-chat-files").join(&id);
    let path = scratch.join(file.safe_name().unwrap_or_else(|| "file".into()));
    let stored = async {
        tokio::fs::create_dir_all(&scratch)
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
    let _ = tokio::fs::remove_dir_all(&scratch).await;
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
        folder_id: Set(folder),
    }
    .insert(db::db())
    .await?;
    Ok(row.into())
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
