# Communities

The community row opens its main feed. The sidebar lists only channels people
explicitly create; there is no synthetic General channel. Main-feed posts retain
an empty `channel_id`, so removing that sidebar row does not move or delete posts.

Add channel opens an owner-only modal with a name, an emoji selector, and
Cancel/Add actions. The icon is stored separately from the channel name.
The picker bundles a curated catalog of Microsoft Fluent 3D emoji locally, with
search across names and keywords and nine categories. Recent choices are saved
per account in this browser (up to 24); they are not synced between devices.
Arrow keys navigate the emoji grid and category tabs; Enter in search selects
the first result without submitting the channel form. The server accepts only
catalog emoji. Artwork licensing and the pinned upstream revision are recorded
in `public/images/fluent-emoji/SOURCE.md`; refresh deliberately with
`python scripts/vendor_fluent_emoji.py`.
Blank names, names longer than 60 characters, and case-insensitive duplicates
are refused. Each community permits five channels. Creation locks the community
row and performs the count and insert in one transaction, so simultaneous
requests cannot exceed that limit. Existing channels are preserved by upgrades.

The banner's pencil opens an owner-only Edit banner dialog: a scrolling grid of
preset artwork (`public/images/banners/`, original SVGs) and Upload image. Done
stays disabled until the choice differs from the saved banner; arrow keys move
through the grid. Uploads are PNG/JPEG up to 2 MB, decoded on the server and
re-encoded as a 1600 × 400 JPEG stored in `community.cover_image`; choosing a
preset clears it. Older `mint`/`lavender`/`sunset` covers remain valid presets.

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
