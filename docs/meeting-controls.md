# Meeting controls

Meeting links now have a lobby. The creating account is the organizer; the
seeded application admin moderates legacy links whose owner is absent. Other
signed-in, active accounts request admission and wait on the setup screen.
No Chime attendee credentials are created until the request is admitted.
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
- **Mute participant:** organizer-only, changes Chime's attendee audio capability
  to Receive. The participant can unmute themselves, which restores SendReceive;
  the organizer never turns on another person's microphone.

The AWS deployment must grant `chime:UpdateAttendeeCapabilities` in addition to
the existing meeting/attendee permissions. Provider failures are surfaced;
the interface never reports success before the provider accepts the action.
See [AWS attendee capabilities](https://docs.aws.amazon.com/chime-sdk/latest/APIReference/API_meeting-chime_AttendeeCapabilities.html).

### Enabling participant mute in the current AWS deployment

The current `upurnaka-chime` IAM user can create meetings but AWS rejects
`UpdateAttendeeCapabilities` with `AccessDeniedException`. An AWS administrator
must attach [chime-moderation-policy.json](chime-moderation-policy.json) to that
user, in addition to its existing permissions. This policy permits only the
capability-update operation on meetings in the verified account and control-plane
region. Deployments in another account or region must change that resource ARN.

Using an administrator's AWS profile, from the project root:

```powershell
aws iam put-user-policy --user-name upurnaka-chime --policy-name UpurnakaParticipantModeration --policy-document file://docs/chime-moderation-policy.json --profile YOUR_ADMIN_PROFILE
```

The application credentials cannot inspect or manage IAM policies. Do not grant
them IAM administration or replace their existing meeting permissions. Once the
policy propagates, retry Mute participant; no new meeting is required. The muted
participant can choose to unmute themselves. Permission failures now return a
safe configuration message, and neither mute nor self-unmute changes local mute
state when Chime rejects the request.

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
