# Package 3: identity and session

Part of [the specification](README.md). Levels and source codes are defined there.

Done when a new Google user signs in on the deployed app and stays signed in after a redeploy.

## 3.1 Sign-in

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 3.1.1 | MUST | Authorization-code flow with PKCE, `state` and `nonce`; scopes `openid email profile` | A callback with a wrong `state` is refused | D, T |
| 3.1.2 | MUST | The ID token is checked for signature, issuer, audience, expiry and nonce | Five tests, one failing each check, are all refused | D |
| 3.1.3 | MUST | First sign-in ends on a "choose your display name" step, prefilled from Google | A second player with the same Google name must choose another | C |
| 3.1.4 | MUST | Later sign-ins match on subject ID, refresh the stored email, and never touch the display name | A changed Google email updates the row and keeps the name | C |
| 3.1.5 | MUST | A blocked account gets a plain refusal page | A removed player's sign-in ends there | U |
| 3.1.6 | MUST | Canceled consent, a Google error and an unverified email each render a message and a retry | Each path has a test | C |
| 3.1.7 | MUST | The post-sign-in return path accepts only same-origin paths | An absolute URL in the return parameter is ignored | C |
| 3.1.8 | MUST | The Google consent screen is published to production under the name "The Gold Ballot" | An account on no test list signs in | R, C |
| 3.1.9 | WONT | Other providers, passwords, emailed codes | Only one sign-in button exists | D, U |

## 3.2 Session

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 3.2.1 | MUST | The session ID is 256 random bits, and only its hash is stored | The cookie value appears nowhere in the database | D, C |
| 3.2.2 | MUST | The cookie is HttpOnly, Secure, SameSite=Lax and host-only | A header test asserts all four | D |
| 3.2.3 | MUST | Every sign-in issues a new session ID | The previous ID is refused afterwards | C |
| 3.2.4 | MUST | A session expires 90 days after its last use | With a fake clock it is valid on day 89 and refused on day 91 | D, U |
| 3.2.5 | MUST | Sign-out, account deletion and removal each end that player's sessions | The old cookie is refused | C |
| 3.2.6 | MUST | Mutations require a JSON body and an `Origin` equal to the canonical origin | A form post and a foreign-origin post are both refused | D |
| 3.2.7 | SHOULD | Expired sessions are purged | None remain after a purge | C |

## 3.3 Authorization

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 3.3.1 | MUST | Each use case takes an actor: anonymous, player, admin or system | The record-result use case refuses a player actor with no HTTP involved | D |
| 3.3.2 | MUST | Every route outside a short public list requires a session | A test walks `openapi.json`, and each non-public route returns 401 without a cookie | C |
| 3.3.3 | MUST | Admin routes return 403 to players | The same walk with a player session | C |
| 3.3.4 | MUST | Admin status is evaluated on each request against the configured emails | A configuration change takes effect with the deploy that carries it | C |
| 3.3.5 | MUST | CLI commands are unreachable over HTTP | No route invokes the seed import | C |

## 3.4 Fake identity gate

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 3.4.1 | MUST | The fake adapter exists only under a compile-time feature | Covered by 2.3.5 | D |
| 3.4.2 | MUST | A binary built with the feature refuses to start under production configuration | It exits non-zero | C |
| 3.4.3 | MUST | The deploy job deploys only the image digest the release job produced | The workflow passes a digest and uses no mutable tag | C |

## 3.5 Account

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 3.5.1 | MUST | The app stores only subject ID, email, display name and timestamps per player; a blocked record keeps subject ID, email and removal time | The schema has no other personal column | C, U |
| 3.5.2 | MUST | Deletion asks for confirmation, erases the player, picks and sessions, and nulls the actor in the results log | No row references the player, and the log entries remain | U, C |
| 3.5.3 | MUST | An admin page lists players by name and email, each with a remove action | A removed player vanishes from the leaderboard and is blocked | U |
| 3.5.4 | MUST | A privacy note says what is stored, who sees it and how to delete it | Every page footer links to it | C |

## Notes

- **A default display name can violate the uniqueness rule.**
  Two relatives with one name would make the second sign-up fail under 1.6.2.
  The naming step in 3.1.3 prevents that, and lets colleagues avoid showing their full Google name.
- **Deletion conflicted with the results log.**
  Rule 1.6.6 says no row may reference a deleted player, and 1.3.5 keeps the actor on every result change.
  Rule 3.5.2 resolves it by nulling the actor.
- **Admin rights are checked per request, not per sign-in.**
  A 90-day session would otherwise keep a removed admin in power.
- **The fake-identity gate has two layers.**
  The compile-time flag protects against a wrong build.
  Rules 3.4.2 and 3.4.3 protect against deploying the wrong image.
- **A blocked record keeps the email.**
  Without it the blocked list in 4.8.8.4 could not show who is blocked.
- **Tommy's setup action:** create the Google OAuth client and consent screen.
