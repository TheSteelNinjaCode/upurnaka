//! Meeting chat. Chime carries a message to whoever is connected at that moment;
//! this table is what keeps it, so it is still there on the next join.
//!
//! A message may share one file. Only a signed-in participant can share, and
//! only a file of their own (from My files, or uploaded there from the chat);
//! every participant, guests included, can then download it through a
//! short-lived link minted for their ticket.
use super::{lobby, mint_id, recording};
use crate::{
    chat_files::{self, Attachment, Listing},
    db,
    models::{chat_file, meeting_message as message, meeting_visit as visit},
    rahti::{Error, Result, RpcFile, Validator},
};
use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::Serialize;

/// The most a participant is sent on joining.
const HISTORY: u64 = 200;
/// How long a shared file's download link stays valid.
const ACCESS_SECONDS: u64 = 600;

/// A shared file, as the chat shows it.
#[derive(Serialize)]
pub struct SharedFile {
    pub name: String,
    pub content_type: String,
    pub bytes: i64,
}

#[derive(Serialize)]
pub struct Message {
    pub id: String,
    pub name: String,
    pub text: String,
    pub at: i64,
    pub file: Option<SharedFile>,
}
impl From<message::Model> for Message {
    fn from(row: message::Model) -> Self {
        let file = row.file_object.as_ref().map(|_| SharedFile {
            name: row.file_name.clone().unwrap_or_else(|| "File".into()),
            content_type: row
                .file_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".into()),
            bytes: row.file_bytes.unwrap_or_default(),
        });
        Self {
            id: row.id,
            name: row.sender_name,
            text: row.body,
            at: row.sent_at,
            file,
        }
    }
}

/// Only someone in the call reads or writes its chat: a guest still waiting in
/// the lobby holds a ticket too.
async fn participant(id: &str, ticket: &str) -> Result<visit::Model> {
    let row = lobby::mine(id, ticket).await?;
    if row.state != "joined" {
        return Err(lobby::denied());
    }
    Ok(row)
}

/// A participant who can share files: one signed in, in the call as themself.
async fn sharer(id: &str, ticket: &str) -> Result<visit::Model> {
    let row = participant(id, ticket).await?;
    if row.user_id.starts_with("guest:") {
        return Err(Error::public(
            StatusCode::FORBIDDEN,
            "Sign in to share files in the meeting chat.",
        ));
    }
    Ok(row)
}

pub async fn history(id: String, ticket: String) -> Result<Vec<Message>> {
    participant(&id, &ticket).await?;
    let mut rows = message::Entity::find()
        .filter(message::Column::MeetingId.eq(&id))
        .order_by_desc(message::Column::SentAt)
        .order_by_desc(message::Column::Id)
        .limit(HISTORY)
        .all(db::db())
        .await?;
    rows.reverse();
    Ok(rows.into_iter().map(Message::from).collect())
}

/// The sender's name comes from their visit, never from the request. `file`
/// names one of the sender's own files; the text may then be empty.
pub async fn send(
    id: String,
    ticket: String,
    text: String,
    file: Option<String>,
) -> Result<Message> {
    let text = text.trim().to_string();
    let file = file.filter(|file| !file.trim().is_empty());
    let mut validator = Validator::new();
    validator
        .check(
            "text",
            !text.is_empty() || file.is_some(),
            "Write a message or attach a file.",
        )
        .max_chars("text", &text, 500)
        .max_chars("file", file.as_deref().unwrap_or(""), 64);
    validator.finish()?;
    let (sender, shared) = match &file {
        Some(file) => {
            let sender = sharer(&id, &ticket).await?;
            let mut rows = chat_files::owned_rows(std::slice::from_ref(file)).await?;
            (sender, rows.pop())
        }
        None => (participant(&id, &ticket).await?, None),
    };
    let row = message::ActiveModel {
        id: Set(mint_id()),
        meeting_id: Set(id),
        sender_id: Set(sender.user_id),
        sender_name: Set(sender.name),
        body: Set(text),
        sent_at: Set(rahti::time::Timestamp::now().as_millisecond()),
        file_object: Set(shared.as_ref().map(|row| row.file_id.clone())),
        file_name: Set(shared.as_ref().map(|row| row.name.clone())),
        file_type: Set(shared.as_ref().map(|row| row.content_type.clone())),
        file_bytes: Set(shared.as_ref().map(|row| row.bytes)),
    }
    .insert(db::db())
    .await?;
    Ok(row.into())
}

/// Upload a file from this device into the sender's My files, ready to share.
pub async fn upload(id: String, ticket: String, file: RpcFile) -> Result<Attachment> {
    sharer(&id, &ticket).await?;
    chat_files::store(file, None).await
}

/// The sender's My files, for choosing one to share.
pub async fn my_files(id: String, ticket: String, folder: Option<String>) -> Result<Listing> {
    sharer(&id, &ticket).await?;
    chat_files::browse("files", folder).await
}

/// A short-lived link to a file shared in this meeting's chat.
pub async fn file_link(id: String, ticket: String, message_id: String) -> Result<String> {
    participant(&id, &ticket).await?;
    let row = message::Entity::find_by_id(&message_id)
        .filter(message::Column::MeetingId.eq(&id))
        .one(db::db())
        .await?;
    let Some(object) = row.and_then(|row| row.file_object) else {
        return Err(Error::new(StatusCode::NOT_FOUND, "File not found."));
    };
    recording::signed_url(&object, ACCESS_SECONDS)
        .await
        .map_err(|reason| {
            eprintln!("meeting chat: no access URL for {message_id}: {reason}");
            Error::public(
                StatusCode::BAD_GATEWAY,
                "File storage could not be reached. Try again shortly.",
            )
        })
}

/// Whether any meeting message still shares a storage object.
pub(crate) async fn shares_object(object: &str) -> Result<bool> {
    Ok(message::Entity::find()
        .filter(message::Column::FileObject.eq(object))
        .count(db::db())
        .await?
        > 0)
}

/// A revoked link takes its conversation with it, and any shared file that
/// nothing else still holds.
pub(super) async fn clear(id: &str) -> Result<()> {
    let objects: Vec<String> = message::Entity::find()
        .filter(message::Column::MeetingId.eq(id))
        .filter(message::Column::FileObject.is_not_null())
        .all(db::db())
        .await?
        .into_iter()
        .filter_map(|row| row.file_object)
        .collect();
    message::Entity::delete_many()
        .filter(message::Column::MeetingId.eq(id))
        .exec(db::db())
        .await?;
    for object in objects {
        if let Err(error) = release_if_unused(&object).await {
            eprintln!("meeting chat: could not check {object} before removing it: {error:?}");
        }
    }
    Ok(())
}

/// Remove a storage object once no file or meeting message uses it.
async fn release_if_unused(object: &str) -> Result<()> {
    let in_files = chat_file::Entity::find()
        .filter(chat_file::Column::FileId.eq(object))
        .count(db::db())
        .await?;
    if in_files == 0 && !shares_object(object).await? {
        if let Err(reason) = recording::delete_object(object).await {
            eprintln!("meeting chat: stored object {object} was not removed: {reason}");
        }
    }
    Ok(())
}
