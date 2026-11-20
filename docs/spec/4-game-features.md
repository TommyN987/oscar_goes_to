# Package 4: game features

Part of [the specification](README.md). Levels and source codes are defined there.
Section 4.8, the screen specifications, is in [4.8-screens.md](4.8-screens.md).

| # | Child | Depends on | Milestone |
|---|---|---|---|
| 4.1 | Season data and seed | 1.1 | M1 |
| 4.2 | Ballot | 4.1, 1.2, 3 | M1 |
| 4.3 | Results entry | 4.1, 1.3, 3 | M2 |
| 4.4 | Leaderboard and statistics | 4.2, 4.3, 1.4, 1.5 | M2 |
| 4.5 | App shell | 3 | M1 |
| 4.6 | API surface | 4.1–4.5 | grows with each |
| 4.7 | Visual system | nothing | M0 onward |
| 4.8 | Screen specifications | 4.2–4.5, 3 | per screen |

## 4.1 Season data and seed

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.1.1 | MUST | Each season is one TOML file in `seasons/`: ceremony number, lock instant with offset, categories in order, nominees with a primary and optional secondary label | The file for the 99th parses, and a malformed file fails naming the line | D, U |
| 4.1.2 | MUST | Categories and nominees carry author-chosen keys, unique within their parent | Changing a label keeps picks; changing a key counts as remove plus add | C |
| 4.1.3 | MUST | Import is a CLI command and one transaction | An import that fails halfway changes nothing | D, C |
| 4.1.4 | MUST | Import validates before writing: unique keys, non-empty labels, at least one category | Each violation exits non-zero with a named error | C |
| 4.1.5 | MUST | Import has a dry run that prints what would be added, changed and removed | A dry run writes nothing | C |
| 4.1.6 | MUST | Removing a category or nominee that has picks or results needs a force flag and reports the count deleted | Without the flag the import fails | C |
| 4.1.7 | MUST | A file may list categories with no nominees | That season imports and is in phase Upcoming | C |
| 4.1.8 | MUST | Production import runs as a manually triggered workflow, dry run first, then apply | The 99th nominees reach production without the database URL leaving the secret store | U |
| 4.1.9 | MUST | Import needs no deploy | A lock-instant change is merged and imported inside the freeze window | C |
| 4.1.10 | MUST | Postgres itself enforces rules 1.2.1 and 1.2.2 | A direct SQL insert breaking either is rejected | C |
| 4.1.11 | SHOULD | The file may carry the nominations date for display | The Upcoming page shows it | C |

## 4.2 Ballot

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.2.1 | MUST | "Make your picks" shows one category at a time, each with its own URL | Reloading a category URL shows that category | G, C |
| 4.2.2 | MUST | A category list in seed order ticks every category with a will-win pick: a sidebar at desktop width, a collapsible list on a phone | The tick appears after the first will-win pick | G |
| 4.2.3 | MUST | Each nominee has a "Will win" and a "Should win" toggle; a category holds at most one pressed toggle of each kind | Pressing a second "Will win" releases the first | G, U |
| 4.2.4 | MUST | Pressing a pressed toggle clears that pick | A reload shows no pick | G, U |
| 4.2.5 | MUST | One page-level status reads saving, "All changes saved", or failed with a retry; a failed toggle reverts | A component test drives all three | G, D |
| 4.2.6 | MUST | At most one write is in flight per category and kind; later presses collapse to the latest | With delayed responses, page and server end on the last press | D, C |
| 4.2.7 | MUST | Progress reads "N of 24 picked" and counts will-win picks | A should-win pick alone leaves N unchanged | G |
| 4.2.8 | MUST | The lock notice shows weekday, date, time and zone in the viewer's zone, plus a countdown based on server time | With server time 2 d 4 h 18 min before lock it reads "02d 04h 18m" | G, U |
| 4.2.9 | MUST | Previous, Next and "Review my ballot" move between categories and the review page | Next on the last category opens "Your ballot" | G, C |
| 4.2.10 | MUST | "Your ballot" has two tabs split by will-win pick; each row shows category, will-win, should-win or a dash, and Edit | A category with only a should-win pick sits under Unpicked and shows it | G, C |
| 4.2.11 | MUST | A summary shows will-win picks out of the total, and the favorites count | The fixture reads "8 predictions saved · 3 favorites" | G |
| 4.2.12 | MUST | "Finish my ballot" opens the first category without a will-win pick, and hides when none remains | A complete ballot shows no such button | G, C |
| 4.2.13 | MUST | A ballot with no picks shows the invitation and the how-it-works steps | Zero picks renders that state | G |
| 4.2.14 | MUST | In Upcoming the page shows a notice and no toggles | A season without nominees renders the notice | C |
| 4.2.15 | MUST | From lock the page is the sealed, read-only view with no Edit links | A locked fixture has no enabled toggle | G |
| 4.2.16 | MUST | Once results arrive, each row shows the winner and Correct, Missed or Pending | A fixture with all three renders them | C |
| 4.2.17 | MUST | A save refused as locked switches the open page to the sealed view | Covered by 6.4.3 | D |
| 4.2.18 | MUST | A pick write has five outcomes: saved, locked, not open, invalid nominee, unauthenticated | Each has a use-case test and appears in `openapi.json` | C |
| 4.2.19 | WONT | Pick history, notes or comments | No such field exists | C |

## 4.3 Results entry

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.3.1 | MUST | An admin page lists categories with their current winners, usable at 360 px | The admin journey passes at phone size | F, C |
| 4.3.2 | MUST | Recording is: choose a category, choose one or more nominees, confirm | The confirmation names the category and the nominees | C |
| 4.3.3 | MUST | The write replaces the category's winner set | Sending the same set twice changes nothing | C |
| 4.3.4 | MUST | Correcting and clearing use the same flow, with confirmation | A corrected category shows the new winners after refresh | D |
| 4.3.5 | MUST | Before lock the page shows "opens at lock" | A request before lock is refused | C |
| 4.3.6 | MUST | A failed save is shown with a retry | A component test with a failing save shows both | C |
| 4.3.7 | MUST | Each set or clear appends a log entry; the current result is the latest entry | After a correction the log holds two entries | D, C |
| 4.3.8 | MUST | Categories without a result list first, as the two groups in 4.8.7.1 | A category moves to the announced group once recorded | C |
| 4.3.9 | WONT | Showing the admin how many players scored | The admin response equals what a player may see | F |
| 4.3.10 | WONT | Bulk or automated result entry in v1 | No such operation exists | D |

## 4.4 Leaderboard and statistics

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.4.1 | MUST | Before lock the page shows "opens at lock" | Covered by 1.4.5 | U |
| 4.4.2 | MUST | From lock it shows rank, display name and score, with the viewer's row marked by text and style | A fixture with a tie renders 1, 1, 3 and marks the viewer | F, C |
| 4.4.3 | MUST | A viewer with no will-win pick is told they are not ranked | That viewer sees the notice and the table | C |
| 4.4.4 | MUST | Final standings stay visible while the season is current | A Complete season still renders the table | C |
| 4.4.5 | MUST | Statistics list, per category, every nominee with its should-win count, highest first | Counts match a fixture, including zeros | F |
| 4.4.6 | SHOULD | Statistics mark the actual winners once recorded | A category with a result marks them | C |
| 4.4.7 | WONT | Past seasons in the UI | Only the current season is reachable; older data stays in the database | U |
| 4.4.8 | WONT | Per-player pages, charts, share images | No such route exists | C |

## 4.5 App shell

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.5.1 | MUST | The signed-out page is the two-panel layout with one "Continue with Google" button | A signed-out visit to any page ends there | G, U |
| 4.5.2 | MUST | First sign-in shows the "Your name" step in that same layout | Covered by 3.1.3 | G, C |
| 4.5.3 | MUST | Navigation follows the phase: while Open, "Make your picks" and "Your ballot"; from lock, "Your ballot", "Leaderboard" and "Statistics" | A locked season shows no "Make your picks" link | G, U |
| 4.5.4 | MUST | The player's name opens Settings, Sign out, and Admin for admins | A player sees no Admin entry | G, C |
| 4.5.5 | MUST | "How it works" also states that a correct prediction scores one point and that others see scores, not picks | The text matches rules 1.4.1 and 1.2.6 | G, C |
| 4.5.6 | MUST | The footer carries "Not affiliated with the Academy" and a link to the privacy note | Both appear on every page | G |
| 4.5.7 | MUST | Settings changes the display name and deletes the account | Both journeys pass | U |
| 4.5.8 | MUST | Unknown paths show a not-found page inside the shell | `/nope` renders it with navigation | C |
| 4.5.9 | WONT | A terms document; the file's "Terms & Privacy Policy" line becomes the privacy-note link | No terms page exists | C |

## 4.6 API surface

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.6.1 | MUST | The API holds exactly the operations listed below | `openapi.json` lists these and no others | C |
| 4.6.2 | MUST | Only health, sign-in start, sign-in callback and the current season are public | The walk in 3.3.2 uses this list | C |
| 4.6.3 | WONT | API versioning | Paths carry no version segment | D, C |

The operations:

- **Session:** start sign-in, sign-in callback, sign out, who am I.
- **Account:** set display name, delete my account.
- **Season:** get the current season with categories, nominees, phase and lock instant (public, rule 4.6.2).
- **Ballot:** get my ballot, set or clear one pick.
- **Standings:** get the leaderboard, get should-win statistics.
- **Admin:** get results, set or clear a category's winners, list players, remove a player, list blocked accounts (with 4.8.8.4), unblock (only if 4.8.8.5 is built).
- **Health:** liveness, readiness.

Versioning is unnecessary because one image ships client and server together.

## 4.7 Visual system

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 4.7.1 | MUST | One custom daisyUI theme holds the palette and square corners | No color literal appears outside the theme | G, D |
| 4.7.2 | MUST | Two type families, a display serif and a UI sans, both free-licensed and self-hosted | The pages request no third-party font host | G, C |
| 4.7.3 | MUST | Layouts follow the file at 390, 768 and 1440 | Screenshots at those widths match the frames' structure | G |
| 4.7.4 | MUST | Ceremony number and date come from the seed | Seeding the 99th changes the nav label and the large numeral | G, C |
| 4.7.5 | MUST | Text contrast meets WCAG AA | The automated scan reports no contrast failure | C |
| 4.7.6 | WONT | A dark-mode switch | One theme ships | C |

## What the Figma drafts showed (G, read on 2026-11-17)

This describes the original drafts, which now sit on the file's Archive page.

- **Look:** an editorial awards-night style in near-black, ivory and champagne gold,
  with a high-contrast display serif, a sans for UI text, square corners and thin rules.
- **Wordmark:** "THE GOLD BALLOT".
- **Six screens:** create account, log in, make your picks, your ballot, ballot locked, empty ballot.
- **Three widths:** 1440, 768 and 390.
- **Sample data:** the 98th ceremony, March 15, 2026, 7:00 PM ET.
- The drafts defined no variables, so color and font names were not read from them.
- About sixty stray one-character text layers sat on the canvas outside the frames.

## What the Figma file holds now (2026-11-20)

- **Pages:** Screens, Design system, and Archive with the drafts. The stray text layers are removed.
- **Screens:** a frame for every screen in section 4.8 at 1440 and 390, and the ballot screens redrawn.
  Eight screens also have a frame at 768; for the others rule 4.7.3 cannot be judged at that width yet.
- **Variables:** Primitives, Theme (colors), Layout (spacing, sizes, radius, border width) and Typography, plus 16 text styles.
  `frontend/src/routes/layout.css` carries the Theme colors (rule 4.7.1), except the dialog scrim, which has no theme entry yet.
- **Changed from the drafts:** muted gray and gold text are darker, so by calculation they reach 4.5:1 on every light surface.
  The automated scan that rule 4.7.5 asks for does not exist yet.
  Inputs and outlined buttons have a stronger outline, "Will win" is a square toggle at every width, and no text is set below 12px.
- **Fonts:** Bodoni Moda for display and Inter for UI text, both under the SIL Open Font License (rule 4.7.2).

## Notes

- **"Last press wins" fails if the network reorders two requests.**
  The server sees arrival order, not press order.
  Rule 4.2.6 fixes the order on the client by allowing one write in flight.
- **A lock change on ceremony day must not need a deploy.**
  The Academy's start time is "subject to change", and the freeze in 2.5.7 blocks deploys that week.
  Rule 4.1.9 keeps import separate from deploy for that reason.
- **Importing at startup was rejected.**
  A rollback to an older revision would re-import the older seed and silently revert the lock time.
- **The leaderboard score has no denominator.**
  That follows from the ruling on 1.4.6.
- **The 99th standings vanish from the UI when the 100th season is seeded (4.4.7).**
  The data stays in the database.
- **The current season is public.**
  The signed-out page shows the ceremony numeral and date from the seed (4.8.1.1), so it reads the season without a session.
  Nominees, the phase and the lock instant are public knowledge.
- **The drafts' auth forms are replaced.**
  They show email and password forms; the design chose Google sign-in only, and 3.1.9 excludes passwords.
  The frames on the Screens page show the Google button.
- **The lock time needs a weekday.**
  For players on CET the lock is 00:00 on a Monday, which reads ambiguously as either night.
- **Gold text on ivory is too faint for rule 4.7.5, so the theme does not use it.**
  Measured on 2026-11-19: `#D6BA7F` on the ivory surfaces is between 1.4:1 and 1.8:1.
  The theme keeps gold for dark surfaces and uses bronze `#6F5A33` on ivory, 4.9:1 at worst.
