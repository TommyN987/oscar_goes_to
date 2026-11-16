# Research

Conducted on 2026-11-16, answering the questions handed over by [the frame](../frame.md).
Status: the frame's five questions are answered as far as the sources go; open unknowns are listed below.

## Method

Four web-research threads ran in parallel, one per file:

- [frontend.md](frontend.md): non-React TypeScript frameworks, and Rust-to-TypeScript type sharing.
- [oscar-data.md](oscar-data.md): ceremony facts, and sources for nominees and live winners.
- [hosting.md](hosting.md): zero-cost compute, database, static hosting, domains, email, abuse protection, deployment.
- [telemetry.md](telemetry.md): free OpenTelemetry backends, and the state of OTel in Rust.

Every claim carries a confidence label:

- **C (confirmed):** read in a primary source on the research date.
- **P (probable):** secondary source, or inference from a primary source.
- **U (uncertain):** weak or conflicting evidence.

The labels were assigned by the research agents and were not independently re-verified.
Free tiers and versions change often; re-check before relying on a number.
The research reports facts and trade-offs only; no option was ranked or chosen.

## Headline findings

- **Winners:** no official machine-readable source exists.
  Two unofficial signals (Wikipedia edits, prediction-market prices) each landed within about a minute of the other in 2026, and each has documented errors.
- **Nominees:** no source was faster than about 1.5 hours after the announcement.
- **Email:** every transactional provider whose mechanism could be confirmed needs DNS records on a domain you control.
  Email confirmation therefore triggers the frame's domain exception, unless one of the listed exceptions is used.
- **Hosting:** zero-cost Rust hosting exists, but the nine idle months remove most free databases.
- **Frontend:** most candidates are mid-way through a major transition in Q4 2026.
- **Telemetry:** eight hosted backends have permanent free tiers; Rust OTel tracing is still beta and breaks often.

## Frame questions answered

1. **Frontend framework:** candidates and facts are in [frontend.md](frontend.md); the choice is a design decision.
2. **Automated data source:** partially.
   Nominees: none fast enough to beat manual entry by much.
   Winners: two fast unofficial signals with known error modes; see [oscar-data.md](oscar-data.md).
3. **Zero-cost hosting, email, telemetry backend, free domain:** options exist for each; see [hosting.md](hosting.md) and [telemetry.md](telemetry.md).
4. **Deployment pipeline:** GitHub Actions facts and per-host deploy mechanisms are in [hosting.md](hosting.md), section 8.
5. **Dates:** nominations are announced on 2027-01-21 (stated by Tommy; also in the Academy/ABC release).
   The ceremony is on 2027-03-14 at 7 p.m. ET / 4 p.m. PT, officially stated and subject to change (C).
   That converts to 2027-03-15 00:00 CET (inference: US daylight saving starts that day).

## Findings that cut across threads

- **Idle months versus free databases.**
  The app is near-idle about nine months a year.
  Supabase, Aiven, Oracle Autonomous, Turso and Render Postgres all pause, stop, archive or expire on inactivity.
  Neon and Cloudflare D1 showed no inactivity penalty.
- **Scale-to-zero versus telemetry export.**
  Cloud Run suspends background work between requests and names OpenTelemetry agents as affected (C).
  Batched spans may be delayed or lost without a shutdown flush (inference).
- **Verified email versus zero cost.**
  The frame's constraint is "verified email"; a confirmation email is one mechanism for it.
  Google sign-in returns an `email_verified` claim with no user cap for basic scopes (C).
  Microsoft's `email` claim "isn't guaranteed to be correct" (C), which matters for a Teams audience.
- **Hostname permanence.**
  Platform hostnames give no DNS control (P, inference).
  Passkeys bind to the hostname, so a later hostname change breaks them (C).
- **Domain needs.**
  A domain is required for email DKIM/SPF, a named Cloudflare Tunnel (C), Cloudflare rate limiting (P, inference) and Google brand verification (C).
  The cheapest paid renewal found is `.de` at USD 4.07 a year on Porkbun (C).
- **Peak-night budget on Cloud Run.**
  One instance held open for 3.5 hours uses about 12,600 of 180,000 free vCPU-seconds a month (arithmetic, assuming 1 vCPU and one instance).
- **TypeScript 7.**
  TypeScript 7.0 shipped without the compiler API, breaking template type-checking for Vue, Svelte and Angular until 7.1 (C for the gap, P for the fix date).
- **Domain-model facts from the 99th rules.**
  An actor can be nominated twice in the same category (C), so "one nominee per person per category" is not a safe assumption (inference).
  The 2026 ceremony had a tie (C), so "one winner per category" is not safe either (inference).

## Neighbours not examined

- The Rust backend ecosystem: web framework, database layer, auth crates.
- GDPR obligations for storing EU users' emails, including telemetry that carries them.
- Trademark exposure of "Oscar" and "Academy Awards" in the app's name or domain.
- Accessibility quality of the component libraries; only their existence was checked.

## Unknowns that bear on decisions

- Whether any email provider accepts a free subdomain as sender, or allows single-sender use without a domain.
- oscars.org terms on automated access; Polymarket and Kalshi terms on reusing market data.
- Cold-start times for Rust on Cloud Run and Lambda.
- Northflank Sandbox regions and persistent volumes.
- Inactivity policies of the free telemetry accounts.
- Stable dates for Vue 3.6, Solid 2.0, Qwik 2.0 and TypeScript 7.1.

Each thread file ends with its own full list of unknowns.
