# Specification

Status: complete on 2026-11-18, after six rounds of dialogue.
Inputs: [the frame](../frame.md), [the research record](../research/README.md) and [the design](../design.md).
Next phase: Code. The Figma frames for the screens in section 4.8 were added on 2026-11-19.

Product name: **The Gold Ballot**. Tagline: "Guess the Oscars with Tommy".

## How to read a rule

Every rule has a number, a level, a requirement, a done-condition and a source.
The files hold the final state of each rule, with all amendments applied.

**Levels**

- **MUST:** non-negotiable.
- **SHOULD:** expected, negotiable.
- **MAY:** optional.
- **WONT:** explicitly out of scope for v1.

**Sources**

- **F** frame. **D** design. **R** research. **U** Tommy's ruling.
- **G** seen in the original drafts of Tommy's Figma file ("Guess the Oscars with Tommy"), as read on 2026-11-17.
  Those drafts now sit on the file's Archive page.
- **O** observed on Tommy's machine.
- **C** proposed by Claude and accepted by Tommy.
- **T** Claude's training knowledge, unverified against current sources.

## Work breakdown

| # | Package | Level | Depends on | Milestone | Done when | File |
|---|---|---|---|---|---|---|
| 1 | Product rules | MUST | nothing | before M1 code | Every MUST rule has a domain test carrying its number | [1-product-rules.md](1-product-rules.md) |
| 2 | Foundation | MUST | nothing | M0 | A merge to main reaches the public URL through CI, and its trace shows in Grafana | [2-foundation.md](2-foundation.md) |
| 3 | Identity and session | MUST | 2 | M1 | A new Google user signs in on the deployed app and stays signed in after a redeploy | [3-identity.md](3-identity.md) |
| 4 | Game features | MUST | 1, 3 | see children | All children are done | [4-game-features.md](4-game-features.md) |
| 4.1 | Season data and seed | MUST | 1.1 | M1 | The 99th seed imports, then re-imports with a corrected label, leaving picks intact | same |
| 4.2 | Ballot | MUST | 4.1, 1.2, 3 | M1 | A player completes a ballot on a phone, and a write at the lock instant is refused | same |
| 4.3 | Results entry | MUST | 4.1, 1.3, 3 | M2 | The admin records and corrects results on a phone | same |
| 4.4 | Leaderboard and statistics | MUST | 4.2, 4.3, 1.4, 1.5 | M2 | The leaderboard equals a hand-computed fixture, including a tie | same |
| 4.5 | App shell | MUST | 3 | M1 | A signed-out visit ends on the sign-in page, and navigation follows the phase | same |
| 4.6 | API surface | MUST | 4.1–4.5 | grows with each | `openapi.json` lists exactly the specified operations | same |
| 4.7 | Visual system | MUST | nothing | M0 onward | One theme holds the palette, and no color literal sits outside it | same |
| 4.8 | Screen specifications | MUST | 4.2–4.5, 3 | per screen | Each screen has Figma frames before its slice is coded | [4.8-screens.md](4.8-screens.md) |
| 5 | Operations | SHOULD | 2 | M2 | One restore from backup has been rehearsed | [5-operations.md](5-operations.md) |
| 6 | Quality bars | MUST | cuts across 3 and 4 | each milestone | Each feature slice cites the bars it meets | [6-quality-bars.md](6-quality-bars.md) |

## Milestones

| Milestone | Date | Content |
|---|---|---|
| M0 | Day three of implementation | The walking skeleton (rules 2.8) |
| M1 | 14 January 2027 | Packages 2, 3, 4.1, 4.2, 4.5; a week before nominations on 21 January |
| M2 | 7 March 2027 | Packages 4.3, 4.4, 5; before the deploy freeze of 8–15 March |

The ceremony is on 14 March 2027 at 7 p.m. ET; picks lock then, expected 2027-03-15 00:00 CET.

## Dependencies

- **Independent pairs:** packages 1 and 2 can be built in parallel, and so can 4.2 and 4.3 once 4.1 exists.
- **Named coupling:** package 6 constrains 3 and 4 and cannot be scheduled as its own slice.
- **Build order:** 2, 3, 4.1, 4.5, 4.2 for M1; then 4.3, 4.4, 5 for M2.

## Coverage of the frame

| Frame constraint | Owned by |
|---|---|
| Rust backend, TypeScript frontend | 2.1.1, 2.8.2 |
| Responsive, one language | 6.2.1, 6.1.1 |
| Open registration, verified email | 3.1, 1.6.1 |
| Tests, CI, clean architecture | 2.3, 6.6, 2.1.2 |
| `tracing` and OTel, free backend | 2.7 |
| Near-zero cost | 2.5.2, 2.5.6, 5.5 |
| Lock at ceremony start | 1.2.5, 4.1.9 |
| Results entered during the ceremony | 4.3 |

## Reconciliations made while recording

Four rules were aligned so the recorded spec does not contradict itself.

- **4.3.8** was SHOULD in round 4; round 6 accepted the same behavior as MUST in 4.8.7.1. It is recorded as MUST.
- **4.6.1** listed the API operations before the blocked list existed. "List blocked accounts" and "unblock" were added, tied to 4.8.8.4 and 4.8.8.5.
- **3.1.8** named the consent screen "Guess the Oscars with Tommy" in round 3. It is recorded as "The Gold Ballot", the later product name.
- **1.2.4 and 6.3.1** assumed radio inputs and a clear control. They are recorded in their round-5 form: toggles, pressed again to clear.

## Open, outside the spec

- **Domain:** Tommy buys it by 7 January 2027 (rule 2.6.4). Name and registrar are undecided.
- **Accounts:** Google Cloud with billing, Neon, Grafana Cloud, the Google OAuth client, and `gcloud` on Tommy's machine (O: missing on 2026-11-16).
- **Figma:** the frames for the screens in 4.8 exist since 2026-11-19 (rule 4.8.0.1).
  They are plain layers bound to variables and text styles; turning them into components is still open.
- **Slices:** who writes which.
- **Unexamined since research:** GDPR obligations and trademark exposure.

## Still unverified (T)

- Free custom-domain mapping on Cloud Run (2.6.2).
- A content security policy that fits a SvelteKit static SPA (6.5.1).
- Whether Google's sign-in branding rules apply to the button (4.8.1.5).
- Grafana Cloud alerting on the free tier (5.7).
- Every item on the design's unverified list (2.8.4).
