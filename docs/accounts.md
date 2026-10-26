# Accounts and authenticator sign-in

Registration is at `/register`; sign-in is at `/signin`. All workspace pages,
including `/` and meeting join URLs, require a session. `/account` shows the
signed-in profile and number of unused recovery codes. The profile menu opens
the account page and signs out.

## Setup

Configure `DATABASE_URL` and a persistent random `AUTH_SECRET` in the ignored
`.env`, then apply the additive migrations deliberately:

```text
cargo run -- --migrate
cargo run
```

Keep `AUTH_SECRET` stable and backed up securely with the database. It signs
sessions and derives a separate AES-256-GCM key for authenticator secrets.
Replacing it signs everybody out and makes existing authenticator secrets
unreadable. Never commit it. Account operations refuse a missing persistent
secret, even in development. See `conventions/authentication.md` for session
cookie configuration and production HTTPS requirements.

## User flow

1. Enter a name, email address, and password of 12–128 characters.
2. Scan the QR code in an authenticator app, or enter its setup key manually.
3. Enter the app's six-digit code. No session is issued before this succeeds.
4. Save the eight recovery codes. Each can replace an authenticator code once,
   after a correct password; they cannot be retrieved later.
5. Continue to the original workspace destination, or Chat by default.

Returning users enter email and password, then an authenticator or recovery
code. Incomplete enrollment resumes through sign-in with the original
password. Starting a new challenge invalidates the previous one. A challenge
expires after ten minutes. Codes use 30-second periods, allow one period of
clock drift in either direction, and cannot be replayed. Five failed password
or second-factor attempts lock that account for 15 minutes; a successful
password alone never resets that budget.

Passwords use Argon2id with independent salts. Only encrypted authenticator
secrets, hashed challenges, and hashed recovery codes are stored. Successful
verification consumes the challenge and code in one conditional database write.
Account responses use `Cache-Control: no-store`.

Meeting lists, renames, and deletions belong to the creating account. Existing
links from before accounts were introduced have no owner: signed-in people can
still join them by URL, but they do not appear in anyone's personal list and
cannot be claimed by registering first.

## Current scope

Email is a normalized login identifier, not a verified mailbox. This release
does not send email, offer password resets, or replace an enrolled authenticator.
Users should retain recovery codes; losing both the authenticator and all
recovery codes currently requires operator assistance. Sessions use Rahti's
fixed expiry and stateless signed cookies; signing out clears the current
browser's cookie. Chat, calendar, contacts, and community demo actions retain
their existing persistence behavior.

## Verification

`cargo test` covers registration, incomplete enrollment, wrong passwords and
codes, lockout, expired/replaced challenges, replay, concurrent verification,
single-use recovery, session cookies, private routes/RPCs, and meeting ownership.
The browser journey registers, enrolls, saves codes, enters the account page,
signs out, checks the route guard, and signs in again with a recovery code.
