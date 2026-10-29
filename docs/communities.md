# Communities

The community row opens its main feed. The sidebar lists only channels people
explicitly create; there is no synthetic General channel. Main-feed posts retain
an empty `channel_id`, so removing that sidebar row does not move or delete posts.

The community's three-dot menu follows Teams Free's action order. Edits retain
the existing image unless a replacement is uploaded or removal is selected.
Notification preferences are saved per account and community; they do not
configure email or push delivery services.

Owners can invite existing, active Upurnaka accounts by email. Invitations appear
in Communities and grant access only after acceptance. Join links likewise
require sign-in and explicit joining; their preview exposes only the community's
name, description, and emoji. Resetting the join link invalidates its old token.
Invitations are internal records and do not send external email.

Members can read and publish posts and save their own notification preferences.
Owners additionally edit the community, manage membership, create channels,
change the cover, share/reset join links, and delete the community. The only owner
must transfer ownership to an active, joined member before leaving. Posts retain
their authors through ownership transfers. Membership and ownership are read
from the database, not from browser-provided role flags.

Delete requires an application confirmation and hides the community for everyone,
invalidates join links, and denies subsequent access. Existing rows are retained
in storage using `deleted_at`; the application does not expose restoration yet.
Reports are persisted and reviewed by workspace administrators at
`/communities/reports`.

Apply pending schema changes deliberately with `cargo run -- --migrate`.
Focused checks: `cargo test --lib app_tests::communities -- --test-threads=1`.

Reference: [Microsoft Teams community settings](https://support.microsoft.com/en-us/teams/free/communities/update-community-settings-in-microsoft-teams-personal).
