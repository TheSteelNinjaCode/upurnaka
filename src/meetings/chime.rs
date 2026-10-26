//! AWS Chime SDK Meetings: the provider behind a meeting link.
//!
//! Three calls do the work. `CreateMeeting` brings a conference into
//! existence in a media region; `CreateAttendee` mints one person's seat in
//! it, which is the join token their browser needs; `DeleteMeeting` ends it
//! for everybody. `GetMeeting` is here because Chime meetings are not
//! permanent and a meeting *link* is — see [`meeting_or_none`].
//!
//! ## Configuration
//!
//! One value turns this on: `CHIME_MEDIA_REGION`, the region the audio and
//! video run in. Without it nothing here is called and a meeting link is
//! still a link — it resolves, it lists, it revokes, and the lobby says no
//! provider is connected.
//!
//! Credentials and the control-plane region come from the ordinary AWS
//! chain, so this file reads neither: `AWS_ACCESS_KEY_ID` /
//! `AWS_SECRET_ACCESS_KEY` (or `AWS_PROFILE`, or an instance role) and
//! `AWS_REGION`. A deployment on EC2, ECS or Lambda supplies all of them
//! without a secret in a file, which is why they are not named in
//! `rahti.config.json` and not read here by hand. The media region is the
//! one value that is a product decision rather than a credential, so it is
//! the one this module asks for.
//!
//! ## Shapes
//!
//! The three response types below are not Rust's idea of a meeting — they
//! are the JSON `amazon-chime-sdk-js` expects to be handed, field for field,
//! which is why they are `PascalCase` and why nothing renames them on the
//! way to the browser.

use std::sync::OnceLock;

use aws_config::BehaviorVersion;
use aws_sdk_chimesdkmeetings::Client;
use aws_sdk_chimesdkmeetings::config::{Credentials, Region};
use aws_sdk_chimesdkmeetings::types::{Attendee, MediaPlacement, Meeting};
use aws_smithy_http_client::tls::{self, rustls_provider::CryptoMode};
use serde::Serialize;
use tokio::sync::OnceCell;

use crate::rahti::{Error, Result};

/// What this application decides about Chime, as opposed to what the AWS
/// environment decides.
pub struct Settings {
    /// Where the media runs: `us-east-1`, `eu-west-1`, and so on.
    pub media_region: String,
}

/// The configured settings, or `None` when no provider is wired.
///
/// Read once. A region that appears halfway through a process's life would
/// mean two meetings minted minutes apart behaved differently, which is not
/// a thing worth supporting.
pub fn settings() -> Option<&'static Settings> {
    static SETTINGS: OnceLock<Option<Settings>> = OnceLock::new();

    // The test suite never reaches AWS. A developer's `.env` has a real media
    // region in it, so without this line `cargo test` would create, and bill
    // for, real Chime meetings — and the tests covering the unconfigured path
    // would pass or fail depending on whose machine ran them. A compile-time
    // answer rather than an environment one, because an environment this
    // process does not control is exactly what is being guarded against.
    if cfg!(test) {
        return None;
    }

    SETTINGS
        .get_or_init(|| {
            // The same `.env` the database and auth read; idempotent, so it
            // does not matter which of them got there first.
            crate::rahti::load_env();

            setting("CHIME_MEDIA_REGION").map(|media_region| Settings { media_region })
        })
        .as_ref()
}

/// Whether a meeting opened now would have a real conference behind it.
pub fn is_configured() -> bool {
    settings().is_some()
}

/// A Chime meeting, in the shape `amazon-chime-sdk-js` is given it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingResponse {
    pub meeting_id: String,
    pub external_meeting_id: String,
    pub media_region: String,
    pub media_placement: MediaPlacementResponse,
}

/// Where that meeting's media lives.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
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
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttendeeResponse {
    pub attendee_id: String,
    pub external_user_id: String,
    pub join_token: String,
}

/// Create a conference for one meeting link.
///
/// `external_id` is the link's own join code. Chime stores it verbatim and
/// hands it back, which is what lets a meeting found in the AWS console be
/// traced to the row that owns it.
pub async fn create_meeting(external_id: &str) -> Result<MeetingResponse> {
    let settings = require()?;

    let output = client()
        .await?
        .create_meeting()
        // Idempotency within Chime's dedupe window. The join code alone
        // would collide with a deliberate re-create after Chime reaped the
        // last conference, so the moment is part of the token.
        .client_request_token(format!(
            "{external_id}-{}",
            crate::rahti::time::now().timestamp()
        ))
        .external_meeting_id(external_id)
        .media_region(&settings.media_region)
        .send()
        .await
        .map_err(provider_failed)?;

    let meeting = output
        .meeting()
        .ok_or_else(|| Error::internal("Chime created a meeting and returned no meeting"))?;

    shape(meeting)
}

/// The conference behind a stored id, or `None` if Chime no longer has one.
///
/// Not an error. A Chime meeting ends when the last attendee leaves and is
/// cleaned up after that, while the link that points at it is meant to be
/// reusable for as long as the organizer wants — so "gone" is the ordinary
/// state of a link nobody is currently in, and the caller's answer to it is
/// to create another.
pub async fn meeting_or_none(meeting_id: &str) -> Result<Option<MeetingResponse>> {
    require()?;

    let output = client()
        .await?
        .get_meeting()
        .meeting_id(meeting_id)
        .send()
        .await;

    let output = match output {
        Ok(output) => output,
        Err(error)
            if error
                .as_service_error()
                .is_some_and(|e| e.is_not_found_exception()) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(provider_failed(error)),
    };

    let Some(meeting) = output.meeting() else {
        return Ok(None);
    };

    shape(meeting).map(Some)
}

/// Mint one person's seat.
pub async fn create_attendee(meeting_id: &str, external_user_id: &str) -> Result<AttendeeResponse> {
    require()?;

    let output = client()
        .await?
        .create_attendee()
        .meeting_id(meeting_id)
        .external_user_id(external_user_id)
        .send()
        .await
        .map_err(provider_failed)?;

    let attendee = output
        .attendee()
        .ok_or_else(|| Error::internal("Chime created an attendee and returned none"))?;

    Ok(AttendeeResponse {
        attendee_id: required(attendee.attendee_id(), "AttendeeId")?,
        external_user_id: Attendee::external_user_id(attendee)
            .unwrap_or(external_user_id)
            .to_string(),
        join_token: required(attendee.join_token(), "JoinToken")?,
    })
}

/// End a conference for everybody in it.
///
/// A meeting Chime has already cleaned up is not an error: the caller wanted
/// it gone, and it is gone.
pub async fn delete_meeting(meeting_id: &str) -> Result<()> {
    require()?;

    let result = client()
        .await?
        .delete_meeting()
        .meeting_id(meeting_id)
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(error)
            if error
                .as_service_error()
                .is_some_and(|e| e.is_not_found_exception()) =>
        {
            Ok(())
        }
        Err(error) => Err(provider_failed(error)),
    }
}

/// A host mute disables sending at the media service. Only that attendee can request unmute.
pub async fn set_attendee_audio(meeting_id: &str, attendee_id: &str, send: bool) -> Result<()> {
    if cfg!(test) {
        return Ok(());
    }
    use aws_sdk_chimesdkmeetings::types::{AttendeeCapabilities, MediaCapabilities};
    let capabilities = AttendeeCapabilities::builder()
        .audio(if send {
            MediaCapabilities::SendReceive
        } else {
            MediaCapabilities::Receive
        })
        .video(MediaCapabilities::SendReceive)
        .content(MediaCapabilities::SendReceive)
        .build()?;
    client()
        .await?
        .update_attendee_capabilities()
        .meeting_id(meeting_id)
        .attendee_id(attendee_id)
        .capabilities(capabilities)
        .send()
        .await
        .map_err(provider_failed)?;
    Ok(())
}

/// The client, built once.
///
/// Credentials resolve per request inside the SDK, so this is cheap to hold
/// and correct to share: a rotated instance role is picked up without a
/// restart.
async fn client() -> Result<&'static Client> {
    static CLIENT: OnceCell<Client> = OnceCell::const_new();

    let settings = require()?;

    Ok(CLIENT
        .get_or_init(|| async {
            let mut loader = aws_config::defaults(BehaviorVersion::latest());

            // The SDK ships no default HTTPS client here on purpose: its
            // `default-https-client` feature is off in Cargo.toml, because it
            // would build rustls on aws-lc-rs and drag ~68 MB of C through
            // CMake into every release build. Ring is what the rest of this
            // binary already links -- sea-orm/sqlx resolves to it -- so saying
            // so here costs one TLS stack instead of three. This also means
            // the client is REQUIRED: without it the SDK has no connector and
            // every call fails at dispatch.
            loader = loader.http_client(
                aws_smithy_http_client::Builder::new()
                    .tls_provider(tls::Provider::Rustls(CryptoMode::Ring))
                    .build_https(),
            );

            // The control plane's region, which is not necessarily the media
            // region. Only chosen here when the environment names neither,
            // so an operator who sets `AWS_REGION` is not overridden.
            let region = setting("AWS_REGION")
                .or_else(|| setting("AWS_DEFAULT_REGION"))
                .unwrap_or_else(|| settings.media_region.clone());
            loader = loader.region(Region::new(region));

            // A key pair written in `.env` is used as written. Anything else
            // — a profile, an EC2/ECS/Lambda role, SSO — is left to the
            // default chain, which is also what a *blank* pair gets: see
            // `setting`, and the note beside these names in `.env`.
            if let (Some(key_id), Some(secret)) = (
                setting("AWS_ACCESS_KEY_ID"),
                setting("AWS_SECRET_ACCESS_KEY"),
            ) {
                loader = loader.credentials_provider(Credentials::new(
                    key_id,
                    secret,
                    setting("AWS_SESSION_TOKEN"),
                    None,
                    "upurnaka-env",
                ));
            }

            Client::new(&loader.load().await)
        })
        .await)
}

/// An environment value this application reads itself, where blank means
/// absent.
///
/// The distinction matters for the AWS names. The SDK's own environment
/// provider stops the credential chain when it finds `AWS_ACCESS_KEY_ID`
/// set, so a key left as `""` in a hand-edited `.env` does not fall through
/// to a profile or an instance role — it fails the call with an invalid
/// key. Reading them here instead makes an empty line and a missing line the
/// same thing, which is what somebody editing that file expects.
fn setting(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// The settings, or an error that says exactly what is missing.
fn require() -> Result<&'static Settings> {
    settings().ok_or_else(|| {
        Error::internal("CHIME_MEDIA_REGION is not set, so no meeting provider is configured")
    })
}

/// An AWS failure, as a server error.
///
/// Kept private deliberately. The SDK's text names regions, account-shaped
/// identifiers and the operation that failed, which belongs in telemetry and
/// not in a browser — `Error::internal` is the constructor that draws that
/// line, and the reader gets the generic message with a request id.
fn provider_failed<E: std::error::Error>(error: E) -> Error {
    Error::internal(format!("the meeting provider refused the call: {error}"))
}

/// A Chime meeting, in the browser's shape.
fn shape(meeting: &Meeting) -> Result<MeetingResponse> {
    Ok(MeetingResponse {
        meeting_id: required(meeting.meeting_id(), "MeetingId")?,
        external_meeting_id: required(meeting.external_meeting_id(), "ExternalMeetingId")?,
        media_region: required(meeting.media_region(), "MediaRegion")?,
        media_placement: meeting.media_placement().map(placement).unwrap_or_default(),
    })
}

fn placement(placement: &MediaPlacement) -> MediaPlacementResponse {
    MediaPlacementResponse {
        audio_host_url: placement.audio_host_url().unwrap_or_default().to_string(),
        audio_fallback_url: placement
            .audio_fallback_url()
            .unwrap_or_default()
            .to_string(),
        signaling_url: placement.signaling_url().unwrap_or_default().to_string(),
        turn_control_url: placement.turn_control_url().unwrap_or_default().to_string(),
        screen_data_url: placement.screen_data_url().unwrap_or_default().to_string(),
        screen_sharing_url: placement
            .screen_sharing_url()
            .unwrap_or_default()
            .to_string(),
        screen_viewing_url: placement
            .screen_viewing_url()
            .unwrap_or_default()
            .to_string(),
        event_ingestion_url: placement
            .event_ingestion_url()
            .unwrap_or_default()
            .to_string(),
    }
}

/// A field the API documents as present, which this application cannot do
/// without. Missing is a provider contract change, not a user's problem.
fn required(value: Option<&str>, field: &str) -> Result<String> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .ok_or_else(|| Error::internal(format!("Chime returned no {field}")))
}
