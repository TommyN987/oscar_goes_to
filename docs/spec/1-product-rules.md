# Package 1: product rules

Part of [the specification](README.md). Levels and source codes are defined there.

Done when every MUST rule below has a test carrying its number (rule 6.6.1).
The pure rules are tested in the domain crate before M1 code.
The others gain their test in the slice that builds them, by M2.

## 1.1 Season and catalog

| # | Level | Rule | Done when | Source |
|---|---|---|---|---|
| 1.1.1 | MUST | A season has a ceremony number, a lock instant in UTC, and ordered categories | The 99th seed loads with 24 categories | F, R |
| 1.1.2 | MUST | A category has a name, a position and any number of nominees | Categories with 5 and with 10 nominees both load | R |
| 1.1.3 | MUST | A nominee is one entry in one category; the same person may appear twice in a category | A seed with one person twice in a category loads, and both entries are separately pickable | R |
| 1.1.4 | MUST | The current season is the one with the highest ceremony number | With two seasons seeded, the app shows the higher | C |
| 1.1.5 | MUST | A season's phase is derived, never stored: Upcoming (no nominees), Open, Locked, Complete (every category has a result) | A domain test fixes the phase for each combination of inputs | C |
| 1.1.6 | MUST | The lock instant can be changed by re-importing the seed, and a change never alters stored picks | Re-import with a new lock instant changes the phase and leaves picks untouched | F, C |
| 1.1.7 | MUST | Re-import matches categories and nominees by stable keys | A corrected nominee label keeps its existing picks | C |
| 1.1.8 | MUST | Import refuses to remove a nominee that has picks, unless forced | Such an import fails without the force flag | C |
| 1.1.9 | WONT | Posters or film metadata in v1 | No request leaves the app for film data | C, R |

## 1.2 Ballot

| # | Level | Rule | Done when | Source |
|---|---|---|---|---|
| 1.2.1 | MUST | Per category, a player has at most one will-win and one should-win pick | A second write replaces the first | F |
| 1.2.2 | MUST | A pick names a nominee of its own category | A write naming another category's nominee is refused | C |
| 1.2.3 | MUST | The two picks are independent and may name the same nominee | All four set/unset combinations are accepted | F, C |
| 1.2.4 | MUST | While Open, a player may set, change or clear any pick | All three succeed before lock | F, C |
| 1.2.5 | MUST | The server refuses any pick write at or after the lock instant, by server time | With a fake clock, a write 1 ms before lock succeeds and one at lock returns a distinct "locked" outcome | F, D |
| 1.2.6 | MUST | A player can read only their own picks, in every phase | A request for another player's ballot is refused | F |
| 1.2.7 | MUST | Each pick saves on selection, with no save button | A selected nominee survives a reload with no further action | U |
| 1.2.8 | MUST | Two quick selections in one category resolve to the last one | A component test with reordered responses ends on the last press | D |
| 1.2.9 | MUST | The ballot shows "N of 24 picked" and the lock time in the viewer's time zone | Both render from server data | C |
| 1.2.10 | MUST | After lock the ballot is read-only and marks each will-win pick right, wrong or pending | A fixture with all three states renders them | C |

## 1.3 Results

| # | Level | Rule | Done when | Source |
|---|---|---|---|---|
| 1.3.1 | MUST | Only an admin records results | A non-admin attempt is refused | F |
| 1.3.2 | MUST | A category's result is one or more winning nominees of that category | Two winners can be recorded for one category | R |
| 1.3.3 | MUST | Results can be recorded only after lock | An attempt before lock is refused | C |
| 1.3.4 | MUST | An admin can replace or clear a result | After a correction, the next leaderboard read reflects it | D |
| 1.3.5 | MUST | Result changes are append-only, with time and actor | After a correction, storage holds both entries | D, C |
| 1.3.6 | MUST | Recording the same winner twice leaves one result | A repeated request changes nothing | C |
| 1.3.7 | MUST | Results enter through one application seam | The HTTP handler and a test double both call the same use case | D |
| 1.3.8 | WONT | Automated ingestion before the February 2027 decision | No code polls an external source | D |

## 1.4 Scoring and leaderboard

| # | Level | Rule | Done when | Source |
|---|---|---|---|---|
| 1.4.1 | MUST | A will-win pick scores 1 if its nominee is among the winners, else 0; should-win never scores | A property test equates score with the count of matching categories | F, C |
| 1.4.2 | MUST | Scores are computed on read | No table or column stores a score | D |
| 1.4.3 | MUST | Ranks are shared as 1, 1, 3, and ties list by display name | A unit test with tied scores yields 1, 1, 3 | F, C |
| 1.4.4 | MUST | The leaderboard lists players with at least one will-win pick | A player with no picks is absent | C |
| 1.4.5 | MUST | The leaderboard requires sign-in and is unavailable before lock | A signed-out request is refused, and a request before lock gets a "not yet" outcome | U |
| 1.4.6 | WONT | An "N of 24 announced, last result at HH:MM" line | The leaderboard response carries no such field | U |
| 1.4.7 | WONT | Hiding who picked each winner | See the note below | C |

## 1.5 Should-win statistics

| # | Level | Rule | Done when | Source |
|---|---|---|---|---|
| 1.5.1 | MUST | Per category, the count of should-win picks per nominee | Counts match a fixture | F |
| 1.5.2 | MUST | Visible only after lock | A request before lock is refused | C |
| 1.5.3 | SHOULD | Shown beside the actual winner once known | A category with a result renders both | C |
| 1.5.4 | WONT | Will-win aggregates in v1 | No endpoint returns them | F |

## 1.6 Players and admin

| # | Level | Rule | Done when | Source |
|---|---|---|---|---|
| 1.6.1 | MUST | A player is keyed by Google's subject ID, and sign-in requires a verified email | A token with an unverified email is refused | D, R |
| 1.6.2 | MUST | The display name is prefilled from the Google name, editable, 1–40 characters, unique ignoring case | A duplicate differing only in case is refused | C |
| 1.6.3 | MUST | Admins are set by configuration, not through the UI | Admin rights follow configuration on the deploy that carries it | C, U |
| 1.6.4 | MUST | The admin plays too, and the app shows them no one else's picks | The admin's requests for other ballots are refused | F, C |
| 1.6.5 | MUST | Accounts persist across seasons | A player from season one signs in to season two with the same account | F |
| 1.6.6 | MUST | A player can delete their account and picks | After deletion, no row references the player | U |
| 1.6.7 | MUST | The admin can remove a player, and removal blocks that Google account | A removed player vanishes from the leaderboard, and their next sign-in is refused | U |
| 1.6.8 | WONT | Any email or notification | The app holds no mail credentials | D |

## Notes

- **Hidden picks leak after lock, and that is accepted (1.4.7).**
  Anyone who refreshes after each result sees whose score rose, so they learn who picked every winner.
  Stopping that would mean hiding scores until the end.
  "Hidden" therefore protects against copying before lock, and is only partial afterwards.
- **Removal differs from deletion.**
  Self-deletion (1.6.6) is a clean erase, and that person may return.
  Removal (1.6.7) blocks the account, because a removed player who can sign straight back in has not been removed.
- **Rules 1.1.7 and 1.1.8 exist for label fixes.**
  A typo fix or a rescinded nomination must not silently orphan picks.
- **Open sign-up needed moderation.**
  Unique names (1.6.2) and removal (1.6.7) are the minimum.
