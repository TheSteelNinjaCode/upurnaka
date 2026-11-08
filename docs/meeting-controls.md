# Meeting controls

Meeting links now have a lobby. The creating account is the organizer; the
seeded application admin moderates legacy links whose owner is absent. Other
signed-in, active accounts request admission and wait on the setup screen.
No Dilnaka Meet attendee credentials are created until the request is admitted.
Organizers see a waiting notification, a toolbar badge, and an admit/deny list
in People. Guests can cancel and can retry after a denied request.

Admission tickets are random, bound to the meeting and signed-in account, and
consumed once when provider credentials are issued. Provider failures permit a
retry; expired, denied, cancelled, foreign, and reused tickets are refused.
Waiting and connected browsers refresh a 90-second presence lease every five
seconds. Old abandoned records are removed after a day on the next entry
request. Only the organizer can read the waiting list or make admission,
spotlight, and remote-mute decisions. The People count excludes content shares.

Participant nameplates and the People panel open a participant-options menu:

- **Pin for me / Unpin:** local focus only, without moving the SDK video node.
- **Fill frame / Fit to frame:** local crop preference for that participant.
- **Switch camera:** cycles the current user's available cameras, keeping the
  selected background effect. One-camera devices show an explanatory message.
- **Remove me from gallery:** hides the local preview, without disabling the
  camera sent to other participants. More → Show me in gallery restores it.
- **Spotlight for everyone / Stop spotlight:** organizer-only, backed by shared
  server state. A departed attendee no longer occupies the spotlight.
- **Mute participant:** organizer-only, sets the attendee's audio capability to
  Receive through Dilnaka Meet
  (`PUT /api/v1/meetings/{id}/attendees/{attendeeId}/capabilities`), so the
  media service enforces it. The participant can unmute themselves, which
  restores SendReceive; the organizer never turns on another person's microphone.

Mute needs only the `DILNAKA_MEETINGS_KEY` the rest of Meet uses. Provider
failures are surfaced as a safe message, and the interface never reports
success, or changes local mute state, before Dilnaka Meet accepts the action.

Screen sharing uses a large content area and a participant strip on desktop,
with a stacked layout on small screens. A paused content tile stays visible
with a status placeholder rather than disappearing. A locked phone, protected
content, or an OS restriction can prevent capture; the application cannot bypass
those restrictions or reliably infer a device lock from a black video frame.

Raised hands show a gold frame, hand badge, and a six-second dismissible
notification naming the participant. Lowering the hand clears the persistent
badge/frame. These controls preserve existing camera elements and streams.

While your own hand is raised, two seconds of sustained, unmuted microphone
activity starts a five-second "Your hand will lower" countdown. The app uses
Chime's local volume notifications (at least 10% volume), so this is an audio
activity heuristic, not speech recognition; no audio is recorded or transcribed.
Silence and muted microphones do not expire a raised hand. "Keep it raised" or
the dismiss button cancels automatic lowering for this raise; lowering and
raising again enables it again. Expiry sends the same hand-lowered event as the
manual button, updating other participants. Camera state does not affect it.
Disconnects cancel a pending countdown; leaving clears the audio subscription
and all countdown state.

Apply the new schema with `cargo run -- --migrate` before restarting. No npm or
additional Rust dependencies are required. Router tests exercise authorization,
ticket lifecycle, shared spotlight/mute, cancellation, expiry, and malformed
payloads. Browser tests run real lobby and moderation RPCs between two browser
sessions while replacing only the external AWS/media transport with a fixture.
