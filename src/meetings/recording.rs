//! Meeting recordings: what the organizer's browser captures, kept here until
//! the recording stops and then handed to Dilnaka Storage.
//!
//! The browser records the meeting with `MediaRecorder` and sends it in
//! timeslices as it goes ([`append`]), so an hour-long meeting is hundreds of
//! small requests rather than one upload the size of the meeting, and a tab
//! that closes mid-call has still delivered all but its last few seconds.
//! Each piece is its own file on this machine until [`finish`] closes the
//! recording; a detached task then joins the pieces and uploads the result
//! through the `dilnaka` crate, which picks a single presigned PUT or a
//! multipart transfer by size. Detached on purpose: "Leave" is the most
//! common way a recording stops, and leaving must not cancel the upload.
//!
//! Only the organizer records — the same `meetings.manage` authority that
//! admits, mutes and spotlights — and everybody in the room is told while a
//! recording runs, through [`active`] in the room state every browser polls.
//!
//! `DILNAKA_API_KEY` turns this on. Without it the Record button says so.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use axum::http::StatusCode;
use dilnaka::{Dilnaka, UploadOptions};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Expr, ExprTrait},
};
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use super::lobby;
use crate::db;
use crate::models::meeting_recording as recording;
use crate::rahti::{Error, Result, RpcUpload, Validator, time};

/// One timeslice. The browser sends one every few seconds, so this is a
/// ceiling for a misbehaving client rather than a size anybody reaches.
const PIECE_LIMIT: usize = 64 * 1024 * 1024;
/// About nine hours at the bitrate the page asks for.
const RECORDING_LIMIT: i64 = 10 * 1024 * 1024 * 1024;
/// A recording whose browser has sent nothing for this long has lost its
/// tab. What arrived is still worth keeping, so it is finished, not dropped.
const STALE_AFTER: i64 = 120;
/// How long a download link the organizer asks for stays valid.
const ACCESS_SECONDS: u64 = 600;

fn now() -> i64 {
    time::Utc::now().timestamp()
}

fn client() -> Option<&'static Dilnaka> {
    static CLIENT: OnceLock<Option<Dilnaka>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let key = std::env::var("DILNAKA_API_KEY")
                .ok()
                .map(|key| key.trim().to_string())
                .filter(|key| !key.is_empty())?;
            match Dilnaka::builder().api_key(key).build() {
                Ok(client) => Some(client),
                Err(error) => {
                    eprintln!("recordings: Dilnaka Storage is not usable: {error}");
                    None
                }
            }
        })
        .as_ref()
}

/// Whether a recording started now would have somewhere to go.
pub fn is_configured() -> bool {
    // The suite never reaches the real bucket; see `upload` below.
    cfg!(test) || client().is_some()
}

/// Where a recording's pieces wait until they are uploaded.
fn folder(id: &str) -> PathBuf {
    std::env::temp_dir().join("upurnaka-recordings").join(id)
}

fn piece(id: &str, part: i32) -> PathBuf {
    folder(id).join(format!("part-{part:06}"))
}

/// The name the file has in storage, and so the name it downloads as.
fn file_name(row: &recording::Model) -> String {
    let started = time::DateTime::from_timestamp(row.started_at, 0)
        .map(|at| time::to_app_time(at).format("%Y-%m-%d-%H%M").to_string())
        .unwrap_or_default();
    let extension = if row.content_type == "video/mp4" {
        "mp4"
    } else {
        "webm"
    };
    format!("meeting-{}-{started}.{extension}", row.meeting_id)
}

/// The container `MediaRecorder` was asked for, without its codecs.
fn media_type(declared: &str) -> Result<String> {
    let essence = declared
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let mut validator = Validator::new();
    validator.one_of(
        "content_type",
        &essence.as_str(),
        &["video/webm", "video/mp4"],
    );
    validator.finish()?;
    Ok(essence)
}

/// A recording this person started in this meeting.
async fn owned(id: &str, recording_id: &str) -> Result<recording::Model> {
    let user = crate::auth::current()
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Authentication required"))?;
    recording::Entity::find_by_id(recording_id)
        .filter(recording::Column::MeetingId.eq(id))
        .filter(recording::Column::StartedBy.eq(user.id))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Recording not found."))
}

/// Whether somebody is recording this meeting right now.
pub async fn active(id: &str) -> Result<bool> {
    Ok(recording::Entity::find()
        .filter(recording::Column::MeetingId.eq(id))
        .filter(recording::Column::State.eq("recording"))
        .filter(recording::Column::UpdatedAt.gt(now() - STALE_AFTER))
        .count(db::db())
        .await?
        > 0)
}

#[derive(Serialize)]
pub struct Started {
    pub id: String,
}

pub async fn start(id: String, ticket: String, content_type: String) -> Result<Started> {
    lobby::host(&id).await?;
    let visit = lobby::mine(&id, &ticket).await?;
    if visit.state != "joined" {
        return Err(lobby::denied());
    }
    let content_type = media_type(&content_type)?;
    if !is_configured() {
        return Err(Error::public(
            StatusCode::SERVICE_UNAVAILABLE,
            "Recording storage is not configured for this workspace.",
        ));
    }
    sweep().await;
    if active(&id).await? {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This meeting is already being recorded.",
        ));
    }

    let recording_id = super::mint_id();
    tokio::fs::create_dir_all(folder(&recording_id)).await?;
    let at = now();
    recording::ActiveModel {
        id: Set(recording_id.clone()),
        meeting_id: Set(id),
        started_by: Set(visit.user_id),
        state: Set("recording".into()),
        content_type: Set(content_type),
        parts: Set(0),
        bytes: Set(0),
        started_at: Set(at),
        updated_at: Set(at),
        ended_at: Set(None),
        file_id: Set(None),
        file_key: Set(None),
        failure: Set(None),
    }
    .insert(db::db())
    .await?;
    Ok(Started { id: recording_id })
}

/// Keep one timeslice.
///
/// The piece is written to its own file before it is counted, so a request
/// that dies halfway leaves nothing counted and the browser's retry of the
/// same `part` starts clean; a retry of a piece that *was* counted is
/// answered without being written twice.
pub async fn append(
    id: String,
    recording_id: String,
    part: i32,
    mut chunk: RpcUpload,
) -> Result<bool> {
    lobby::host(&id).await?;
    let row = owned(&id, &recording_id).await?;
    if row.state != "recording" {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This recording has already stopped.",
        ));
    }
    if part < row.parts {
        return Ok(true);
    }
    if part > row.parts {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "A piece of this recording is missing.",
        ));
    }
    // A hint, not a fact — and one the multipart parser drops when the codecs
    // list is unquoted, as `MediaRecorder` writes it. What is declared must
    // match; what is absent is taken as the recording's own type.
    if chunk
        .content_type()
        .is_some_and(|declared| media_type(declared).ok().as_ref() != Some(&row.content_type))
    {
        return Err(Error::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "A recording piece must be the video type the recording started with.",
        ));
    }

    let path = piece(&row.id, part);
    let written = async {
        let file = tokio::fs::File::create(&path).await?;
        let mut writer = tokio::io::BufWriter::new(file);
        let mut size = 0usize;
        while let Some(bytes) = chunk.chunk().await? {
            size += bytes.len();
            if size > PIECE_LIMIT || row.bytes + size as i64 > RECORDING_LIMIT {
                return Err(Error::new(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "This recording has reached its size limit.",
                ));
            }
            writer.write_all(&bytes).await?;
        }
        writer.flush().await?;
        Ok::<_, Error>(size)
    }
    .await;
    let size = match written {
        Ok(size) => size as i64,
        Err(error) => {
            let _ = tokio::fs::remove_file(&path).await;
            return Err(error);
        }
    };

    // Counted only while it is still the next piece of a running recording.
    // A recording that stopped meanwhile is assembled from what it had
    // counted, and this file is simply not part of it.
    recording::Entity::update_many()
        .col_expr(recording::Column::Parts, Expr::value(part + 1))
        .col_expr(
            recording::Column::Bytes,
            Expr::col(recording::Column::Bytes).add(size),
        )
        .col_expr(recording::Column::UpdatedAt, Expr::value(now()))
        .filter(recording::Column::Id.eq(&row.id))
        .filter(recording::Column::State.eq("recording"))
        .filter(recording::Column::Parts.eq(part))
        .exec(db::db())
        .await?;
    Ok(true)
}

/// Stop taking pieces and start the upload. Safe to call twice.
pub async fn finish(id: String, recording_id: String) -> Result<bool> {
    lobby::host(&id).await?;
    let row = owned(&id, &recording_id).await?;
    if close(&row.id).await? {
        tokio::spawn(deliver(row.id));
    }
    Ok(true)
}

/// Upload a recording whose upload failed again, from the pieces a failure
/// keeps on disk. Any organizer of the meeting may ask.
pub async fn retry(id: String, recording_id: String) -> Result<bool> {
    lobby::host(&id).await?;
    let row = recording::Entity::find_by_id(&recording_id)
        .filter(recording::Column::MeetingId.eq(&id))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Recording not found."))?;
    if row.state != "failed" || row.parts == 0 || !folder(&row.id).exists() {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This recording cannot be uploaded again.",
        ));
    }
    let claimed = recording::Entity::update_many()
        .col_expr(recording::Column::State, Expr::value("uploading"))
        .col_expr(recording::Column::UpdatedAt, Expr::value(now()))
        .filter(recording::Column::Id.eq(&row.id))
        .filter(recording::Column::State.eq("failed"))
        .exec(db::db())
        .await?;
    if claimed.rows_affected == 1 {
        tokio::spawn(deliver(row.id));
    }
    Ok(true)
}

/// Claim a recording for upload. Only one caller wins.
async fn close(id: &str) -> Result<bool> {
    let at = now();
    let closed = recording::Entity::update_many()
        .col_expr(recording::Column::State, Expr::value("uploading"))
        .col_expr(recording::Column::EndedAt, Expr::value(at))
        .col_expr(recording::Column::UpdatedAt, Expr::value(at))
        .filter(recording::Column::Id.eq(id))
        .filter(recording::Column::State.eq("recording"))
        .exec(db::db())
        .await?;
    Ok(closed.rows_affected == 1)
}

/// Finish the recordings whose tab went away without saying so.
async fn sweep() {
    let stale = recording::Entity::find()
        .filter(recording::Column::State.eq("recording"))
        .filter(recording::Column::UpdatedAt.lte(now() - STALE_AFTER))
        .limit(20)
        .all(db::db())
        .await;
    for row in stale.unwrap_or_default() {
        if let Ok(true) = close(&row.id).await {
            tokio::spawn(deliver(row.id));
        }
    }
}

struct StoredFile {
    id: String,
    key: String,
}

/// Join the pieces into one file and hand it to Dilnaka Storage.
///
/// A failure keeps the pieces on disk: a recording is not something to
/// throw away because the network was down for a minute.
async fn deliver(id: String) {
    let Ok(Some(row)) = recording::Entity::find_by_id(&id).one(db::db()).await else {
        return;
    };
    let result = async {
        if row.parts == 0 {
            return Err("nothing was recorded".to_string());
        }
        let path = folder(&row.id).join(file_name(&row));
        assemble(&row, &path)
            .await
            .map_err(|error| error.to_string())?;
        let mut metadata = dilnaka::Metadata::new();
        metadata.insert("meeting".into(), row.meeting_id.clone().into());
        metadata.insert("recording".into(), row.id.clone().into());
        metadata.insert("startedBy".into(), row.started_by.clone().into());
        let options = UploadOptions::default()
            .folder(format!("recordings/{}", row.meeting_id))
            .content_type(row.content_type.clone())
            .metadata(metadata);
        upload(&path, options).await
    }
    .await;

    let update = recording::Entity::update_many()
        .col_expr(recording::Column::UpdatedAt, Expr::value(now()))
        .filter(recording::Column::Id.eq(&row.id));
    let update = match result {
        Ok(file) => {
            let _ = tokio::fs::remove_dir_all(folder(&row.id)).await;
            update
                .col_expr(recording::Column::State, Expr::value("stored"))
                .col_expr(recording::Column::FileId, Expr::value(file.id))
                .col_expr(recording::Column::FileKey, Expr::value(file.key))
        }
        Err(reason) => {
            eprintln!("recordings: {} could not be stored: {reason}", row.id);
            if row.parts == 0 {
                let _ = tokio::fs::remove_dir_all(folder(&row.id)).await;
            }
            let reason: String = reason.chars().take(500).collect();
            update
                .col_expr(recording::Column::State, Expr::value("failed"))
                .col_expr(recording::Column::Failure, Expr::value(reason))
        }
    };
    if let Err(error) = update.exec(db::db()).await {
        eprintln!("recordings: {} could not be updated: {error}", row.id);
    }
}

/// `MediaRecorder` timeslices are consecutive pieces of one stream, so the
/// file is their concatenation, in order.
async fn assemble(row: &recording::Model, path: &Path) -> std::io::Result<()> {
    let mut output = tokio::fs::File::create(path).await?;
    for part in 0..row.parts {
        let mut input = tokio::fs::File::open(piece(&row.id, part)).await?;
        tokio::io::copy(&mut input, &mut output).await?;
    }
    output.flush().await
}

#[cfg(not(test))]
async fn upload(path: &Path, options: UploadOptions) -> std::result::Result<StoredFile, String> {
    let client = client().ok_or("DILNAKA_API_KEY is not configured")?;
    let file = client
        .upload_with_options(path, options)
        .await
        .map_err(|error| error.to_string())?;
    Ok(StoredFile {
        id: file.id,
        key: file.key,
    })
}

/// The suite's stand-in for the bucket: the file is checked, not sent.
#[cfg(test)]
async fn upload(path: &Path, options: UploadOptions) -> std::result::Result<StoredFile, String> {
    let size = tokio::fs::metadata(path)
        .await
        .map_err(|error| error.to_string())?
        .len();
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    Ok(StoredFile {
        id: format!("test-file-{size}"),
        key: format!("{}/{name}", options.folder.unwrap_or_default()),
    })
}

#[derive(Serialize)]
pub struct Listed {
    pub id: String,
    pub state: String,
    pub started: String,
    pub seconds: i64,
    pub bytes: i64,
}

/// This meeting's recordings, newest first, for its organizer.
pub async fn list(id: String) -> Result<Vec<Listed>> {
    lobby::host(&id).await?;
    sweep().await;
    let rows = recording::Entity::find()
        .filter(recording::Column::MeetingId.eq(&id))
        .order_by_desc(recording::Column::StartedAt)
        .limit(50)
        .all(db::db())
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| Listed {
            started: time::DateTime::from_timestamp(row.started_at, 0)
                .map(|at| time::to_app_time(at).format("%b %-d, %Y %H:%M").to_string())
                .unwrap_or_default(),
            seconds: row.ended_at.unwrap_or(row.updated_at) - row.started_at,
            id: row.id,
            state: row.state,
            bytes: row.bytes,
        })
        .collect())
}

/// A short-lived link to download one stored recording.
pub async fn access(id: String, recording_id: String) -> Result<String> {
    lobby::host(&id).await?;
    let row = recording::Entity::find_by_id(&recording_id)
        .filter(recording::Column::MeetingId.eq(&id))
        .one(db::db())
        .await?
        .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Recording not found."))?;
    let Some(file_id) = row.file_id.filter(|_| row.state == "stored") else {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "This recording is not in storage yet.",
        ));
    };
    access_url(&file_id).await
}

#[cfg(not(test))]
async fn access_url(file_id: &str) -> Result<String> {
    let unavailable = || {
        Error::public(
            StatusCode::BAD_GATEWAY,
            "Recording storage could not be reached. Try again shortly.",
        )
    };
    let client = client().ok_or_else(unavailable)?;
    match client
        .get_file_access_url(file_id, Some(ACCESS_SECONDS))
        .await
    {
        Ok(access) => Ok(access.url),
        Err(error) => {
            eprintln!("recordings: no access URL for {file_id}: {error}");
            Err(unavailable())
        }
    }
}

#[cfg(test)]
async fn access_url(file_id: &str) -> Result<String> {
    Ok(format!(
        "https://storage.test/{file_id}?expiresIn={ACCESS_SECONDS}"
    ))
}
