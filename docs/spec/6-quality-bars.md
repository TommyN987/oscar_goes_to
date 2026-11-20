# Package 6: quality bars

Part of [the specification](README.md). Levels and source codes are defined there.

These bars cut across packages 3 and 4.
Done when each feature slice cites the bars it meets.

## 6.1 Language and time

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.1.1 | MUST | All UI text is English | No other language appears in the source strings | U |
| 6.1.2 | MUST | Instants show in the viewer's time zone with the zone named, the lock line adds the weekday, and instants travel as UTC | A test in two browser time zones shows two local times for one lock | C, U |
| 6.1.3 | WONT | A translation framework | No such dependency exists | F |

## 6.2 Devices

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.2.1 | MUST | Every page works at 360 px, 768 px and desktop width without horizontal scroll | The journeys pass at 360, 768 and 1280 px wide | F, G |
| 6.2.2 | MUST | The journeys pass in Chromium, Firefox and WebKit | All three run in CI | C |

## 6.3 Accessibility

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.3.1 | MUST | A category is a labeled group; each toggle exposes its pressed state and a name such as "Will win: Sean Penn" | An automated scan reports no serious or critical issue on any page | G, U |
| 6.3.2 | MUST | The ballot can be completed by keyboard alone | A keyboard-only journey passes | C |
| 6.3.3 | MUST | Right, wrong and pending are never shown by color alone | Each state carries text or an icon | C |

## 6.4 States

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.4.1 | MUST | API errors share one JSON shape with stable codes, declared in OpenAPI | Every non-2xx response in `openapi.json` references that shape | C |
| 6.4.2 | MUST | A failed pick save is shown and the selection reverts | A component test with a failing save shows both | D |
| 6.4.3 | MUST | A lock refusal turns the ballot read-only with a message | A journey crossing the lock with a fake clock shows it | D |
| 6.4.4 | MUST | An expired session leads to sign-in and back to the same page | A journey with an expired session ends where it began | C |
| 6.4.5 | MUST | Every page defines its loading, empty and error states | Each state has a component test | C |
| 6.4.6 | SHOULD | Cold start is at most 5 s; above that, the liveness ping (5.5) or the proxy fallback is adopted | The M0 measurement is compared with 5 s and the decision recorded | C |

## 6.5 Security

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.5.1 | MUST | Responses carry a content security policy, `frame-ancestors 'none'`, HSTS and `nosniff` | A header test asserts them, and the journeys raise no policy violation | C, T |
| 6.5.2 | MUST | Inputs are bounded: a body-size cap, and display names of 1–40 characters with no control characters | An oversize body returns 413 | C |
| 6.5.3 | SHOULD | Sign-in and write routes are rate-limited in the app | A burst beyond the limit returns 429 | R, C |
| 6.5.4 | MUST | The frontend never renders raw HTML | A lint rule forbids it | C |
| 6.5.5 | MUST | Leaderboard and statistics responses contain no pick and no email | Their schemas in `openapi.json` have no such field | F |
| 6.5.6 | MUST | Telemetry carries no cookie, token or authorization code | The scan in 2.7.6 also searches for them | C |

## 6.6 Tests

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.6.1 | MUST | Every MUST rule in package 1 maps to a test carrying its number | A script lists rules without a test, and the list is empty | C |
| 6.6.2 | MUST | Scoring and ranking have property tests | They run in CI | D |
| 6.6.3 | MUST | Two end-to-end journeys exist: player (sign in, name, pick, reload, lock) and admin (record, correct, leaderboard) | Both pass in CI | C |
| 6.6.4 | WONT | A coverage percentage target | No threshold gates CI | C |

## 6.7 Telemetry content

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.7.1 | SHOULD | Counters exist for sign-ins, pick writes by outcome, and results recorded or corrected | They appear in the local Grafana after the journeys | C |
| 6.7.2 | SHOULD | A ceremony-night dashboard is kept in the repo | It imports into Grafana and shows data | C |

## 6.8 Performance

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 6.8.1 | SHOULD | The leaderboard responds within 300 ms at the 95th percentile, warm, with 50 players | A fixture test measures it | C |

## Notes

- **Toggles replaced radios.**
  Round 3 mandated native radio inputs, which cannot be unselected.
  The Figma file's buttons work as toggles, so pressing again clears a pick and rule 1.2.4 needs no extra control.
- **The content security policy travels in two parts (6.5.1).**
  SvelteKit writes the policy into the SPA shell as a `<meta>` tag, with the hash of its one inline script.
  A `<meta>` policy cannot carry `frame-ancestors`, so the server sends that directive as a header on every response.
  The header test asserts the header, and the image smoke test asserts the shell's policy.
  Style attributes stay allowed, because SvelteKit hides its route announcer with one.
  Checked in Chromium on 2026-11-20: the built app and the dev server raise no violation.
- **The body cap is 16 KiB (6.5.2).**
  The server refuses a larger declared `Content-Length` before it reads the body.
  A body that declares no length is not capped yet, because no operation reads one; plan item P3.3 closes that.
- **The shell cannot show a loading state during cold start.**
  The page is blank until the container answers, which the design accepted.
  Rule 6.4.6 ties the M0 measurement to a decision.
