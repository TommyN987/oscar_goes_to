# Implementation plan

Status: approved by Tommy on 2026-11-20.
Input: [the specification](spec/README.md), as amended the same day.

The plan orders the work and gives every spec rule one owning item.
It adds no requirement: a rule's text and done-condition live in `docs/spec/`.

## How to read an item

- **Number:** items are cited as "P3.2". The prefix keeps them apart from spec rule numbers.
- **Level:** MUST, SHOULD, MAY or WONT, as in the spec. An item's level equals the level of the rules it owns.
- **Rules:** the spec rules the item owns. A rule has exactly one owner, even when other items touch it.
- **Needs:** items, decisions or actions by Tommy that must come first.
- **Done when:** the check that closes the item. Each owned rule's own done-condition still applies.
- **Operations:** the API operations from spec rule 4.6.1 that the item adds.

## What decides the order

1. **Dates outside the repo set the critical path, not code dependencies.**
   The OAuth client, the domain (7 January), nominations (21 January) and the deploy freeze (8 March) are fixed points.
   The plan front-loads everything that needs none of them.
2. **Every merge deploys and migrates production (rule 2.4.6).**
   A half-built slice is live when it merges, and a migration on `main` is permanent.
   The two schemas (P3.1, P4.1) are therefore separate items, and the game schema lands whole.
3. **Production stays Upcoming until 21 January (rule 1.1.5).**
   M1's ballot can only be proven on a fixture season in the test image.
   That puts the end-to-end harness (P3.6) on the M1 critical path.
4. **The test image is the attack surface.**
   Fake sign-in and a controllable clock are bypasses if they ship.
   The release probe (rule 2.3.5) covers every test-only route, not only sign-in.
5. **Package 6 cannot be a slice, but its instruments can.**
   The accessibility scan, viewport matrix, telemetry scan and OpenAPI walk are built once, and each page registers with them.

## Decisions taken with the plan

All were proposed by Claude and accepted by Tommy on 2026-11-20, except D5, D10 and D12, which are Tommy's rulings; D14 was accepted the same day, after P2's first commit.

| # | Decision | Recorded in |
|---|---|---|
| D1 | "Get the current season" is public, because the signed-out page shows the seed's numeral and date (4.8.1.1) | Spec rule 4.6.2, amended |
| D2 | A package-1 rule needs a test carrying its number at any layer, not a domain test. The pure rules are tested before M1 code; the rest close with their slices, by M2 | Spec package 1, amended |
| D3 | Rules 1.3.5, 4.3.7, 3.5.4 and 4.8.11.1 are MUST, because MUST rules 3.5.2, 4.5.6 and 4.8.1.3 depend on them | Spec, amended |
| D4 | Rule 5.1 is due on 20 January 2027, and rule 5.6 first runs in December 2026; the rest of package 5 stays at M2 | Spec package 5, amended |
| D5 | Render (rule 2.5.8) is MAY, pursued only if proven necessary | Spec rule 2.5.8, amended |
| D6 | The dress rehearsal (5.4) runs on the test image with a scratch database, because a fake season in production would hide the 99th or be invisible (1.1.4, 2.5.9) | This plan, P10.3 |
| D7 | A signed-in player without a display name may only read who-am-I, set a name, sign out or delete the account; any other operation returns its own error code (4.8.2.4) | This plan, P3.5.2 |
| D8 | The end-to-end clock (6.4.3) is a control under the same compile-time feature as fake identity, covered by the release probe | This plan, P6.5 |
| D9 | P2 holds every pure package-1 rule now, including those first used at M2 | This plan, P2 |
| D10 | A season has nominees in every category or in none; import refuses anything between (Tommy's ruling) | Spec rule 4.1.4, amended |
| D11 | The domain has its own instant type, `UtcInstant`, and no time crate: it only compares instants | This plan, P2 |
| D12 | Leaderboard ties list by display name ignoring case; statistics ties list in seed order | Spec rules 1.4.3 and 4.4.5, amended |
| D13 | `DisplayName` yields the lowercase key that decides uniqueness; Postgres stores and indexes that key and never computes it | This plan, P2.5 and P3.1 |
| D14 | A nominee is a film with an optional credit and an optional title of its own; films are shared by key, and a person is not modeled. The two lines of a row are derived from those parts in the HTTP adapter, because composing them is presentation; the domain only knows what a category leads with | Spec rules 4.1.1, 4.1.2, 4.1.4, 4.1.12 and 1.1.7 |
| D15 | A key is made of lowercase letters, digits and hyphens; the three kinds of key are separate types that refuse anything else | Spec rules 4.1.2 and 4.1.4 |

## Open decisions

| # | Question | Decided in |
|---|---|---|
| O1 | How the production import runs without a deploy and without the database URL leaving the secret store (4.1.8, 4.1.9). A Cloud Run job is the first candidate, unverified | P4.5 |
| O2 | Which date the sign-in panel shows (4.7.4, 4.8.1.1): the lock instant is 15 March for a viewer on CET, and the ceremony is known as 14 March | P3.5.1 |

## Work breakdown

### P1 Foundation gaps

No dependency; parallel with P2.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P1.1 | MUST | Settings and records sweep | 2.1.4, 2.1.5, 2.5.4, 2.5.6 | Tommy: GitHub and billing settings | A test fails when `config.rs` reads a variable `.env.example` omits; `docs/deploy.md` records a rejected direct push, a rejected token commit, Neon's region and the alert's test notification, each dated |
| P1.2 | MUST | Response hardening: content security policy, `frame-ancestors`, HSTS, `nosniff`, body cap | 6.5.1, 6.5.2 | The policy for a static SPA checked against current documentation | A header test passes, and an oversize body returns 413 |
| P1.3 | MUST | Trace ID on exported logs | 2.7.3 | none | A log record at the fake collector carries its trace ID |
| P1.4 | MUST | Lint guards and the dialog scrim token | 4.7.1, 6.5.4 | none | A color literal outside `layout.css` fails `just check`, and so does an `{@html}` |
| P1.5 | SHOULD | Supply chain and `CLAUDE.md` wording | 2.3.7, 2.1.8 | none | A dependency with a known advisory fails CI, and the update bot opens a pull request |

The journey half of 6.5.1 closes in P3.6, and the display-name half of 6.5.2 in P2.5.
P1.2 caps a body by its declared length only; P3.3 caps a body that declares none, with the first operation that reads one.

### P2 Domain core

Pure Rust in `crates/domain`; no dependency.

| # | Level | Item | Rules | Done when |
|---|---|---|---|---|
| P2.1 | MUST | Rule ledger script | 6.6.1 | An untested package-1 MUST rule fails `just check` unless it is on a pending list; a listed rule that has a test also fails; the list is empty at M2 |
| P2.2 | MUST | Season, film, category, nominee, phase | 1.1.2, 1.1.3, 1.1.5 | One test fixes the phase for every combination of inputs |
| P2.3 | MUST | Pick validity and editability at an instant | 1.2.2, 1.2.3, 1.2.4, 1.2.5 | 1 ms before lock is editable; at lock the outcome is "locked" |
| P2.4 | MUST | Winner sets, scoring, ranks, statistics | 1.3.2, 1.3.3, 1.4.1, 1.4.3, 1.4.4, 1.5.1, 6.6.2 | Property tests run in `just check`, and a tie yields 1, 1, 3 |
| P2.5 | MUST | Verified email and display-name shape | 1.6.1 | An unverified claim is refused; names of 0 or 41 characters, or with a control character, are refused |
| P2.6 | MUST | Seed diff as a pure function | none; supports P4.3 | Two fixture seasons yield the expected add, change and remove lists, with forced removals flagged |

- Instants are `UtcInstant`, the domain's own type (D11).
- Rule 1.1.1 moved from P2.2 to P4.2 while P2 was built, with Tommy's approval: its done-condition names the 99th seed, which P4.2 writes.
  The domain test of its pure half carries no rule number, so the ledger keeps the rule pending.

### P3 Identity

Needs P2.5 and P1.2.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P3.1 | MUST | Identity schema, the first migration | 3.5.1, 1.6.5, 2.4.6, 2.3.1 | none | The personal columns equal the rule's list; the migration applies exactly once; stale sqlx metadata fails CI |
| P3.2 | MUST | Sessions on the fake identity adapter. Operations: who am I, sign out | 3.2.1–3.2.5, 3.3.1, 3.3.4, 1.6.3, 3.4.1, 3.4.2, 2.2.3, 2.3.5, 2.7.2 | P3.1 | A session is valid on day 89 and refused on day 91; the release image returns 404 on the fake route; one trace shows HTTP, use-case and SQL spans |
| P3.3 | MUST | Request guards and the OpenAPI walk | 3.2.6, 3.3.2, 3.3.3, 4.6.1, 4.6.2, 6.4.1 | P3.2 | One test walks `openapi.json`: 401 without a cookie, 403 for a player on admin routes, every error references the shared shape, and no operation falls outside the 4.6.1 list |
| P3.4 | MUST | Google adapter. Operations: start sign-in, sign-in callback | 3.1.1, 3.1.2, 3.1.4–3.1.7, 2.3.6 | P3.2; current documentation of the OIDC crate | Five token tests, a wrong `state` and three failure paths are refused against an in-process issuer; the release image redirects to Google |
| P3.5.1 | MUST | Sign-in page, failure messages, self-hosted fonts | 4.5.1, 4.8.1.1–4.8.1.4, 4.8.10.2–4.8.10.4, 6.4.4, 4.7.2 | P3.4, O2 | A signed-out visit to any page ends there, and returns to that page after sign-in |
| P3.5.2 | MUST | Name step. Operation: set display name | 3.1.3, 1.6.2, 4.5.2, 4.8.2.1–4.8.2.5 | P3.5.1, D7 | A duplicate differing only in case shows "That name is taken. Try another.", and the field keeps the typed value |
| P3.6 | MUST | End-to-end harness and page registry | 2.3.4, 6.2.1, 6.2.2, 6.3.1, 4.7.5, 4.7.3, 2.7.6, 6.5.6 | P3.5.1 | A sign-in journey passes in three browsers; every registered page passes at 360, 768 and 1280 with no serious scan issue; exported spans hold no email, cookie or token |
| P3.7 | MUST | Settings and deletion. Operation: delete my account | 1.6.6, 3.5.2, 4.5.7, 4.8.9.1–4.8.9.4 | P3.5.2 | After deletion no row references the player, and the old cookie is refused |
| P3.8 | MUST | Admin players. Operations: list players, remove a player | 1.6.7, 3.5.3, 4.8.8.1–4.8.8.3 | P3.3 | A removed player's next sign-in ends on the refusal page, and the admin's own row has no Remove action |
| P3.9 | SHOULD | Session purge, rate limits, blocked list, Google button branding. Operation: list blocked accounts | 3.2.7, 6.5.3, 4.8.8.4, 4.8.1.5 | P3.8 | Each rule's own done-condition |
| P3.10 | MUST | Google sign-in on the deployed app | 3.1.8, 3.4.3 | Tommy: OAuth client, consent screen, a bootstrap run | An account on no test list signs in and stays signed in after a redeploy; the workflow passes a digest |
| P3.11 | MUST | Privacy note | 3.5.4, 4.8.11.1 | none; P3.5.1 links to it | Its three claims match rules 3.5.1, 1.2.6 and 4.8.9.4 |

- P3.4 does not need Tommy's OAuth client: its tests run against an in-process issuer. Only P3.10 waits for the client.
- P3.1 stores `DisplayName::uniqueness_key` in its own column under a unique index; Postgres never lowercases a name itself (D13).
- On 2026-11-20 the deploy workflow passes the tag `:${{ github.sha }}`, not a digest (`.github/workflows/deploy.yml`), so rule 3.4.3 is open.

### P4 Season data

Needs P3.1, P2.2 and P2.6.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P4.1 | MUST | Game schema: seasons, films, categories, nominees, picks, result entries | 4.1.10, 1.2.1, 1.4.2 | P3.1 | Direct SQL inserts breaking 1.2.1 or 1.2.2 are rejected, and no column stores a score |
| P4.2 | MUST | Seed format, `seasons/99.toml` without nominees, a fixture season | 4.1.1, 4.1.4, 4.1.7, 2.1.1, 1.1.1 | P2.2 | The 99th parses with 24 categories, and a malformed file fails naming the line |
| P4.3 | MUST | Import command: dry run, apply, force | 4.1.2, 4.1.3, 4.1.5, 4.1.6, 1.1.6, 1.1.7, 1.1.8, 3.3.5 | P4.1, P4.2, P2.6 | A failed import changes nothing; a corrected title or credit keeps its picks; a removal with picks fails without the force flag |
| P4.4 | MUST | Operation: get the current season | 1.1.4, 4.1.12 | P4.1 | With two seasons seeded, the higher is returned with its derived phase, and a test fixes the two lines of each kind of row |
| P4.5 | MUST | Production import workflow | 4.1.8, 4.1.9 | P4.3, O1; Tommy runs the bootstrap script | The 99th reaches production by dry run then apply, with no new service revision |

### P5 App shell

Needs P3.5.2 and P4.4.

| # | Level | Item | Rules | Done when |
|---|---|---|---|---|
| P5.1 | MUST | Shell: phase navigation, name menu, footer, not-found, load error | 4.5.3, 4.5.4, 4.5.6, 4.5.8, 4.7.4, 4.8.10.1, 4.8.10.5, 4.8.3.4 | `/nope` renders inside the shell; a player sees no Admin entry; seeding changes the nav label |
| P5.2 | MUST | Upcoming notice and instant formatting | 4.2.14, 4.8.3.1, 4.8.3.2, 6.1.2 | Two browser time zones show two local times for one lock |
| P5.3 | SHOULD | Category names and nominations date on the Upcoming notice | 4.8.3.3, 4.1.11 | 24 names render for the 99th |

### P6 Ballot

Needs P4 and P5. P6.2 is independent of P6.1.
The domain refuses a pick in an unknown category as its own case; P6.1 folds that into the five outcomes of rule 4.2.18.
P6.3 decides how a one-line place tells apart two nominees of one category that share a headline.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P6.1 | MUST | Operations: get my ballot, set or clear one pick | 4.2.18, 1.2.6, 1.6.4 | P4.1, P3.3 | The five outcomes each have a use-case test, and no operation takes another player's ID |
| P6.2 | MUST | Ballot editing rune module | 4.2.5, 4.2.6, 1.2.8, 6.4.2 | none | With reordered responses, page and server end on the last press; a failed save reverts |
| P6.3 | MUST | "Make your picks" page | 4.2.1–4.2.4, 4.2.7–4.2.9, 1.2.7, 1.2.9, 6.3.2 | P6.1, P6.2, P5.1 | A keyboard-only journey completes a ballot, and the picks survive a reload |
| P6.4 | MUST | "Your ballot" review, the empty state, "How it works" | 4.2.10–4.2.13, 4.5.5 | P6.3 | The fixture reads "8 predictions saved · 3 favorites" |
| P6.5 | MUST | Sealed view and lock crossing | 4.2.15, 4.2.17, 6.4.3 | P6.4, D8 | A journey crossing the lock ends read-only with a message |

### P7 M1 exit

Due 14 January 2027.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P7.1 | MUST | Domain, canonical host, OAuth redirect registered again | 2.6.1–2.6.4 | Tommy buys the domain by 7 January | `https://<domain>/api/health` returns 200, and the other host redirects |
| P7.2 | SHOULD | CI duration and the cold-start decision | 2.3.9, 6.4.6 | P3.6 | The median of ten runs and the comparison with 5 s are recorded |
| P7.3 | MUST | M1 gate | done-conditions of packages 3, 4.1, 4.2 and 4.5 | P3–P6, P7.1 | The player journey passes at phone size in the test image, and a new Google user signs in at the domain |

### P8 Results

For M2. Needs P4 and P3; independent of P6.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P8.1 | MUST | Operations: get results, set or clear a category's winners; the results log | 1.3.1, 1.3.4–1.3.7, 4.3.3, 4.3.5, 4.3.7 | P4.1, P3.3 | A player actor is refused with no HTTP involved, and a correction leaves two log entries |
| P8.2 | MUST | Admin results page, and the admin area's two tabs | 4.3.1, 4.3.2, 4.3.4, 4.3.6, 4.3.8, 4.8.7.1–4.8.7.6, 4.8.8.6 | P8.1, P3.8 | The admin records and corrects a result at 360 px |
| P8.3 | MUST | Results view of "Your ballot" | 4.2.16, 1.2.10, 4.8.4.1–4.8.4.6 | P8.1, P6.5 | A two-winner fixture marks both pickers Correct |

### P9 Standings

Needs P6.1 and P8.1.

| # | Level | Item | Rules | Done when |
|---|---|---|---|---|
| P9.1 | MUST | Leaderboard operation and page | 1.4.5, 4.4.1–4.4.4, 4.8.5.1–4.8.5.3, 4.8.5.6, 6.5.5 | The page equals a hand-computed fixture with a tie, and its schema has no pick and no email |
| P9.2 | MUST | Statistics operation and page | 1.5.2, 4.4.5, 4.8.6.1–4.8.6.3, 4.8.6.6 | Counts match the fixture, zeros included |
| P9.3 | MUST | Both journeys in CI | 6.6.3 | The player and admin journeys pass in three browsers |
| P9.4 | SHOULD | Winner marks, Refresh button, rank-one headline, the 300 ms bar | 1.5.3, 4.4.6, 4.8.5.4, 4.8.5.5, 4.8.6.4, 6.8.1 | Each rule's own done-condition |

### P10 Operations and M2 exit

Due 7 March 2027, except where an item says otherwise.

| # | Level | Item | Rules | Needs | Done when |
|---|---|---|---|---|---|
| P10.1 | MUST | Encrypted daily dump, due 20 January (D4) | 5.1, 5.1.1 | Tommy: the key in his password manager, a bootstrap run | Yesterday's dump is in the bucket |
| P10.2 | MUST | Restore rehearsal, ceremony-night runbook, pre-season checklist | 5.2, 5.3, 5.6 | P10.1 | A restored dump has matching row counts, and the checklist has a first run dated December 2026 (D4) |
| P10.3 | SHOULD | Dress rehearsal, uptime monitor, counters, dashboard | 5.4, 5.5, 6.7.1, 6.7.2 | P9, D6 | A fake season reaches Complete with a correct leaderboard |
| P10.4 | MUST | M2 gate, and the February decision on automated ingestion (1.3.8) | done-conditions of packages 4.3, 4.4 and 5 | P8, P9, P10.2 | The ledger's pending list is empty, and `openapi.json` equals the 4.6.1 list |

### P11 Optional

| # | Level | Item | Rules | Done when |
|---|---|---|---|---|
| P11.1 | MAY | Reload hint, Render, "Your favorite" tag, category filter, unblock, error alert | 2.4.8, 2.5.8, 4.8.6.5, 4.8.7.7, 4.8.8.5, 5.7 | Decided one by one at P10.4; none is planned |

### P12 Standing constraints

| # | Level | Item | Rules | Done when |
|---|---|---|---|---|
| P12.1 | WONT | The 20 WONT rules stay absent | 1.1.9, 1.3.8, 1.4.6, 1.4.7, 1.5.4, 1.6.8, 2.1.9, 2.5.9, 2.7.9, 3.1.9, 4.2.19, 4.3.9, 4.3.10, 4.4.7, 4.4.8, 4.5.9, 4.6.3, 4.7.6, 6.1.3, 6.6.4 | The `/finish-slice` review reports none introduced |
| P12.2 | MUST | Bars each page slice cites | 6.1.1, 6.3.3, 6.4.5, 4.8.0.2 | Each page has loading, empty and error component tests, and its tally cites the bars |

## Couplings

- **Players table, picks and import rules.**
  Picks reference players, and import must count picks.
  That forces P3.1, then P4.1, then P4.3.
- **The harness and every page slice.**
  From P3.7 on, a page slice is not done until its page is in the P3.6 registry.
- **Canonical origin, Origin check, OAuth redirect and the host-only cookie.**
  Moving to the domain (P7.1) probably signs everyone out.
  That is harmless before 21 January.
- **Three browsers at three widths against the 10-minute CI bar (2.3.9).**
  On 2026-11-18 a run took between 2 min 36 s and 6 min 31 s (`docs/deploy.md`).
  P7.2 measures again once P3.6 has landed.

## Tommy's actions

| # | Level | Action | By | Unblocks |
|---|---|---|---|---|
| T1 | MUST | Confirm branch protection, push protection and the €1 billing alert | P1.1 | P1.1 |
| T2 | MUST | Create the Google OAuth client and consent screen | about 13 December | P3.10 |
| T3 | SHOULD | Draw 768 frames for the remaining screens | before each page slice | Rule 4.7.3 at 768 |
| T4 | MUST | Buy the domain | 7 January | P7.1 |
| T5 | MUST | Put the backup key in the password manager | 20 January | P10.1 |
| T6 | MUST | Type the 99th nominees and run the import | 21 January | The Open phase |

## Checkpoints

| Date | Reached when |
|---|---|
| 29 November 2026 | P1 and P2 are merged |
| 20 December 2026 | P3 is merged, and P3.10 passes on the deployed app |
| 3 January 2027 | P4 and P5 are merged |
| 10 January 2027 | P6 is merged |
| 14 January 2027 | P7.3: M1 |
| 20–21 January 2027 | P10.1 is running, and the nominees are imported |
| 21 February 2027 | P8 and P9 are merged |
| 7 March 2027 | P10.4: M2 |

If P3.6 is not green by 20 December, P1.5, P3.9 and P5.3 leave M1 first.

## Coverage on 2026-11-20

A script compared the items above with the rule tables in `docs/spec/`: 288 rules, each owned once, each at its item's level.
Rule 2.1.9, a WONT, and rule 4.1.12, a MUST, were added later that day and are counted below, which makes 290.
The script was not kept.

| Disposition | Rules |
|---|---|
| Already met | 32 |
| MUST, owned by an open item | 212 |
| SHOULD, owned by an open item | 20 |
| MAY, unplanned (P11.1) | 6 |
| WONT, standing (P12.1) | 20 |

**Already met:** 2.1.2, 2.1.3, 2.1.6, 2.1.7, 2.2.1, 2.2.2, 2.2.4, 2.3.2, 2.3.3, 2.3.8, 2.4.1–2.4.5, 2.4.7, 2.5.1–2.5.3, 2.5.5, 2.5.7, 2.7.1, 2.7.4, 2.7.5, 2.7.7, 2.7.8, 2.8.1–2.8.4, 4.8.0.1, 4.8.0.3.

- The package-2 statuses come from a read-only audit of the repo on 2026-11-20; no check was re-run for it.
- Rule 2.1.6 was judged from the README's text, not from a clean checkout.
- Rules 2.5.1 and 2.5.2 were judged from the workflow and the `deploy` recipe, not from the live service.
