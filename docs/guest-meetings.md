# Shared meeting links

Opening `/meet/j/{id}` without signing in offers browser entry or installation
of the Upurnaka web app. The installation option uses the browser's native
install prompt when available and otherwise explains the browser menu steps.
This is an installable web app, not a native executable download. Installation
availability depends on the browser and a secure origin (HTTPS or localhost).

Guests enter a display name and configure their devices before requesting entry.
They wait in the lobby without receiving provider credentials or transmitting
media to the meeting. The signed-in link owner must join the meeting to admit
or decline them. Legacy unowned links retain their administrator policy.

Guests do not receive an account session. Each request receives a random
256-bit bearer ticket, scoped to one meeting, kept in component memory and
renewed by lobby/room polling. It expires after 90 seconds without renewal.
Reloading requires a fresh request and fresh approval. Never copy tickets into
URLs, logs, local storage, or shared participant data. Account-owned entries
remain bound to their signed-in owner. Cancellation, denial, expiration, and
link revocation prevent provider credential issuance.

Other workspace routes and moderator/recording RPCs still require sign-in.
No database migration or new environment setting is required.

Run `cargo test --lib app_tests::meeting_controls -- --test-threads=1` for the
admission rules and two-browser flow. The browser test substitutes the AWS
provider boundary with a deterministic SDK fixture; real AWS connectivity and
native installation dialogs are not exercised by the suite.

Reference: [Microsoft Teams guest joining](https://support.microsoft.com/en-us/teams/meetings/join-a-meeting-without-an-account-in-microsoft-teams).
