# Problem Frame

Status: closed on 2026-11-16.
Research: completed on 2026-11-16, recorded in [research/](research/README.md).
Design: converged on 2026-11-16, recorded in [design.md](design.md).
Spec: completed on 2026-11-18, recorded in [spec/](spec/README.md).
Amended on 2026-11-16 by the design: see the two constraints marked "Amended".

## Situation

The repo is empty, and the product is defined.

- **Product:** a web application for guessing the winners of the Academy Awards.
- **Players:** one shared pool of about 20 people, 50 at most:
  family, movie-nerd friends, and possibly a Teams channel at work.
  Anyone with the link can sign up.
- **Ballot:** each category takes one "will win" pick and an optional "should win" pick.
  Picks are editable until lock.
  Partial ballots are allowed.
- **Scoring:** a correct "will win" earns 1 point, a wrong one earns 0.
  Tied players share a rank.
  "Should win" feeds summary statistics only.
- **Privacy:** a player sees only their own picks,
  plus the leaderboard and aggregate "should win" statistics.
- **Results:** the admin (Tommy) enters winners live during the ceremony.
- **Seasons:** the app recurs yearly, starting with the 99th ceremony.
  Accounts persist across seasons.
  Each season has its own leaderboard.
- **Authors:** Tommy and Claude alternate writing and reviewing, working spec-first.

## Complication

A fixed date, a near-zero budget and two learning goals compete for the same four months.

- **Calendar:** picking must work when nominations land on 21 January 2027.
  Results and the leaderboard must work by ceremony night.
- **Budget:** near-zero cost has to cover hosting, email delivery and a telemetry backend.
- **Open sign-up:** "anyone with the link" makes the 50-player ceiling an expectation, not a guarantee.
  A leaked link or a bot can burn a free email quota.
- **Learning:** a new frontend framework and OpenTelemetry are both unfamiliar ground on a deadline.
- **Domain drift:** categories and nominees change every year,
  so a season is data and not a fixed structure.

## Question

How do we get a tested, observable, near-zero-cost prediction game
in front of about 50 people by nominations on 21 January 2027,
on a frontend stack Tommy is learning as we go?

## Constraints

- Rust backend.
- TypeScript frontend, not React and not Rust/WASM.
- Responsive on phone and desktop, in one UI language.
- Open registration with verified email.
  The mechanism (JWT or otherwise) is a design decision.
- Strong automated tests, CI from the first commit, clean architecture:
  built as if it were a commercial project.
- Instrumentation with the `tracing` crate and OpenTelemetry, with a free telemetry backend.
- Recurring cost of €0, with a paid domain as the only exception.
  Amended: the original rule allowed a domain "only if no free option exists".
  The design buys one regardless, so that a host change never alters the app's URL.
- Picks lock at the official ceremony start.
  The expected value is 2027-03-15 00:00 CET; the official start is authoritative.
  The Academy/ABC release of 7 April 2026 states 14 March 2027, 7 p.m. ET, subject to change.
- Results are entered and published during the ceremony, and seen on page refresh.
  Amended: the original wording was "Results are visible live on ceremony night".
  The design drops any push or polling mechanism.
- When the date is threatened, scope is cut first
  (should-win statistics, data automation, telemetry depth),
  not tests or the deadline.

## Non-goals

- Monetization and billing.
- Multiple pools or private leagues.
- Seeing other players' picks (v1; may be revisited later).
- Tiebreakers.
- Scale beyond about 50 users.
- Native mobile apps.
- Internationalization.
- Weighted or confidence-based scoring.

## Handed to Research

1. Which frontend framework: Vue, Svelte or something more niche?
2. Is there an automated source for nominees, and for winners live on the night?
   If one exists, it replaces manual entry.
3. Which hosting, email delivery and telemetry backend cost nothing, and is a free domain viable?
4. What deployment pipeline options fit those hosts?
   Deployment is not a learning priority.
5. What is the exact nominations date, and has the Academy confirmed the ceremony start time?
   Answered: 21 January 2027, and yes (see the lock constraint above).
