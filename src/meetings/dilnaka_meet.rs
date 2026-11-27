//! Dilnaka Meet: the provider behind a meeting link.
//!
//! Four calls do the work, all on `/api/v1/meetings`. Creating a meeting
//! brings a conference into existence; adding an attendee mints one person's
//! seat in it, which is the join token their browser needs; deleting the
//! meeting ends it for everybody. Reading a meeting is here because
//! conferences are not permanent and a meeting *link* is — see
//! [`meeting_or_none`]. A fifth, the attendee's capabilities, is the host
//! mute.
//!
//! ## Configuration
//!
//! `DILNAKA_MEETINGS_KEY` (a `dlk_meet_…` key from the Dilnaka console) turns
//! this on. Without it nothing here is called and a meeting link is still a
//! link — it resolves, it lists, it revokes, and the lobby says no provider
//! is connected. `DILNAKA_MEETINGS_URL` points at another Dilnaka deployment
//! and defaults to production.
//!
//! The key is a credential that bills the account: it stays on this server,
//! in the ignored `.env`, and never reaches a browser or a log.
//!
//! ## Shapes
//!
//! Dilnaka Meet answers in Amazon Chime's own field names, which is what
//! `amazon-chime-sdk-js` expects to be handed. The response types below keep
//! those `PascalCase` names on the way in and on the way out to the browser.

use std::sync::OnceLock;
use std::time::Duration;

use axum::http::StatusCode;
use rahti::serde_json::{self, Value, json};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::rahti::{Error, Result};

const DEFAULT_URL: &str = "https://dilnaka.cloud.tsnc.tech";

/// The configured client.
struct Settings {
    key: String,
    base_url: String,
    http: reqwest::Client,
}

/// The settings, or `None` when no provider is wired.
///
/// Read once. A key that appears halfway through a process's life would mean
/// two meetings minted minutes apart behaved differently.
fn settings() -> Option<&'static Settings> {
    static SETTINGS: OnceLock<Option<Settings>> = OnceLock::new();

    // The test suite never reaches Dilnaka. A developer's `.env` has a real
    // key in it, so without this line `cargo test` would create, and bill for,
    // real meetings — and the tests covering the unconfigured path would pass
    // or fail depending on whose machine ran them.
    if cfg!(test) {
        return None;
    }

    SETTINGS
        .get_or_init(|| {
            crate::rahti::load_env();
            let key = setting("DILNAKA_MEETINGS_KEY")?;
            let base_url = setting("DILNAKA_MEETINGS_URL")
                .unwrap_or_else(|| DEFAULT_URL.to_string())
                .trim_end_matches('/')
                .to_string();
            let http = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|error| eprintln!("meetings: Dilnaka Meet is not usable: {error}"))
                .ok()?;
            Some(Settings {
                key,
                base_url,
                http,
            })
        })
        .as_ref()
}

/// Whether a meeting opened now would have a real conference behind it.
pub fn is_configured() -> bool {
    settings().is_some()
}

/// A meeting, in the shape `amazon-chime-sdk-js` is given it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingResponse {
    pub meeting_id: String,
    pub external_meeting_id: String,
    pub media_region: String,
    #[serde(default)]
    pub media_placement: MediaPlacementResponse,
}

/// Where that meeting's media lives.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MediaPlacementResponse {
    pub audio_host_url: String,
    pub audio_fallback_url: String,
    pub signaling_url: String,
    pub turn_control_url: String,
    pub screen_data_url: String,
    pub screen_sharing_url: String,
    pub screen_viewing_url: String,
    pub event_ingestion_url: String,
}

/// One person's seat in a meeting.
///
/// `join_token` is a credential: it is the whole of the authority to speak in
/// this meeting as this attendee. It goes to the browser that asked for it
/// and nowhere else — never to a log, never to a second viewer.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttendeeResponse {
    pub attendee_id: String,
    pub external_user_id: String,
    pub join_token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MeetingEnvelope {
    meeting: MeetingResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AttendeeEnvelope {
    attendee: AttendeeResponse,
}

/// A refusal from Dilnaka Meet: its status and its `{"error": {code, message}}`.
struct Refusal {
    status: StatusCode,
    code: String,
    message: String,
}

/// One authenticated call. `Ok(None)` is an empty body (`204`).
async fn call(
    method: Method,
    path: &str,
    body: Option<Value>,
) -> std::result::Result<Option<Value>, Refusal> {
    let settings = settings().ok_or_else(|| Refusal {
        status: StatusCode::SERVICE_UNAVAILABLE,
        code: "NOT_CONFIGURED".into(),
        message: "DILNAKA_MEETINGS_KEY is not set, so no meeting provider is configured".into(),
    })?;
    let mut request = settings
        .http
        .request(method.clone(), format!("{}{path}", settings.base_url))
        .bearer_auth(&settings.key);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let unreachable = |detail: String| Refusal {
        status: StatusCode::BAD_GATEWAY,
        code: "UNREACHABLE".into(),
        message: format!("{method} {path}: {detail}"),
    };
    let response = request
        .send()
        .await
        .map_err(|error| unreachable(error.to_string()))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| unreachable(error.to_string()))?;
    if status.is_success() {
        if text.trim().is_empty() {
            return Ok(None);
        }
        return serde_json::from_str(&text)
            .map(Some)
            .map_err(|_| unreachable("unreadable JSON".into()));
    }
    let error = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    let field = |name: &str| {
        error["error"][name]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    Err(Refusal {
        status: StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
        code: field("code"),
        message: field("message"),
    })
}

/// A Dilnaka Meet refusal, as this application's error.
///
/// Out of minutes or over the spending cap (`402`) is the organizer's to fix,
/// and its message is written for them, so it is shown. Anything else names
/// the provider's internals and stays in telemetry behind the generic message.
fn failed(refusal: Refusal) -> Error {
    let Refusal {
        status,
        code,
        message,
    } = refusal;
    if status == StatusCode::PAYMENT_REQUIRED && !message.is_empty() {
        return Error::new(
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Dilnaka Meet refused ({code}): {message}"),
        )
        .with_public_message(message);
    }
    Error::internal(format!(
        "the meeting provider refused the call ({status} {code}): {message}"
    ))
}

fn parse<T: serde::de::DeserializeOwned>(value: Option<Value>, what: &str) -> Result<T> {
    value
        .and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| Error::internal(format!("Dilnaka Meet returned no {what}")))
}

/// Create a conference for one meeting link.
///
/// `external_id` is the link's own join code. Dilnaka stores it and hands it
/// back, which is what lets a meeting in the Dilnaka console be traced to the
/// row that owns it.
pub async fn create_meeting(external_id: &str) -> Result<MeetingResponse> {
    let body = json!({ "ExternalMeetingId": external_id });
    let answer = call(Method::POST, "/api/v1/meetings", Some(body))
        .await
        .map_err(failed)?;
    Ok(parse::<MeetingEnvelope>(answer, "meeting")?.meeting)
}

/// The conference behind a stored id, or `None` if it has ended.
///
/// Not an error. A conference ends some minutes after the last attendee
/// leaves, while the link that points at it is meant to be reusable for as
/// long as the organizer wants — so "gone" is the ordinary state of a link
/// nobody is currently in, and the caller's answer to it is to create another.
pub async fn meeting_or_none(meeting_id: &str) -> Result<Option<MeetingResponse>> {
    match call(Method::GET, &format!("/api/v1/meetings/{meeting_id}"), None).await {
        Ok(answer) => Ok(Some(parse::<MeetingEnvelope>(answer, "meeting")?.meeting)),
        Err(refusal) if refusal.status == StatusCode::NOT_FOUND => Ok(None),
        Err(refusal) => Err(failed(refusal)),
    }
}

/// Mint one person's seat.
pub async fn create_attendee(meeting_id: &str, external_user_id: &str) -> Result<AttendeeResponse> {
    let body = json!({ "ExternalUserId": external_user_id });
    let answer = call(
        Method::POST,
        &format!("/api/v1/meetings/{meeting_id}/attendees"),
        Some(body),
    )
    .await
    .map_err(failed)?;
    Ok(parse::<AttendeeEnvelope>(answer, "attendee")?.attendee)
}

/// End a conference for everybody in it.
///
/// A meeting that has already ended is not an error: the caller wanted it
/// gone, and it is gone.
pub async fn delete_meeting(meeting_id: &str) -> Result<()> {
    match call(
        Method::DELETE,
        &format!("/api/v1/meetings/{meeting_id}"),
        None,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(refusal) if refusal.status == StatusCode::NOT_FOUND => Ok(()),
        Err(refusal) => Err(failed(refusal)),
    }
}

/// Take one person's seat away, ending their connection to the conference.
///
/// An attendee or meeting that is already gone is not an error.
pub async fn delete_attendee(meeting_id: &str, attendee_id: &str) -> Result<()> {
    if cfg!(test) {
        return Ok(());
    }
    match call(
        Method::DELETE,
        &format!("/api/v1/meetings/{meeting_id}/attendees/{attendee_id}"),
        None,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(refusal) if refusal.status == StatusCode::NOT_FOUND => Ok(()),
        Err(refusal) => Err(failed(refusal)),
    }
}

/// A host mute disables sending at the media service. Only that attendee can
/// request unmute.
pub async fn set_attendee_audio(meeting_id: &str, attendee_id: &str, send: bool) -> Result<()> {
    if cfg!(test) {
        // Exercise provider rejection through the real RPC without contacting Dilnaka.
        if meeting_id == "denied-moderation-provider" {
            return Err(audio_update_failed(Refusal {
                status: StatusCode::UNAUTHORIZED,
                code: "UNAUTHORIZED".into(),
                message: "fixture key refusal".into(),
            }));
        }
        return Ok(());
    }
    let body = json!({
        "Capabilities": {
            "Audio": if send { "SendReceive" } else { "Receive" },
            "Video": "SendReceive",
            "Content": "SendReceive",
        }
    });
    call(
        Method::PUT,
        &format!("/api/v1/meetings/{meeting_id}/attendees/{attendee_id}/capabilities"),
        Some(body),
    )
    .await
    .map_err(audio_update_failed)?;
    Ok(())
}

/// Keep provider details private while giving the room an actionable failure.
fn audio_update_failed(refusal: Refusal) -> Error {
    let (status, message) = match refusal.status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Meeting moderation is unavailable. Ask the administrator to check the Dilnaka Meet key.",
        ),
        StatusCode::NOT_FOUND => (
            StatusCode::CONFLICT,
            "This participant's meeting connection has ended. Refresh the participant list and try again.",
        ),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            "The meeting service could not change the microphone permission. Try again shortly.",
        ),
    };
    Error::new(
        status,
        format!(
            "{message} Dilnaka Meet capabilities update failed ({} {}): {}",
            refusal.status, refusal.code, refusal.message
        ),
    )
    .with_public_message(message)
}

/// An environment value, where blank means absent.
fn setting(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dilnaka_answers_parse_into_the_browser_shape() {
        let meeting: MeetingEnvelope = serde_json::from_value(json!({
            "Meeting": {
                "MeetingId": "4e1a0000-0000-0000-0000-000000000000",
                "ExternalMeetingId": "abc-defg-hij",
                "MediaRegion": "us-east-1",
                "MediaPlacement": { "AudioHostUrl": "a:3478", "SignalingUrl": "wss://s" },
                "MeetingArn": "ignored"
            }
        }))
        .unwrap();
        assert_eq!(meeting.meeting.media_placement.audio_host_url, "a:3478");
        assert_eq!(meeting.meeting.media_placement.turn_control_url, "");

        let attendee: AttendeeEnvelope = serde_json::from_value(json!({
            "Attendee": { "AttendeeId": "x", "ExternalUserId": "Ana#1", "JoinToken": "t" }
        }))
        .unwrap();
        assert_eq!(attendee.attendee.join_token, "t");
    }

    #[test]
    fn no_key_means_no_provider_in_the_suite() {
        assert!(!is_configured());
    }
}
