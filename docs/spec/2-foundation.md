# Package 2: foundation

Part of [the specification](README.md). Levels and source codes are defined there.

Done when a merge to main reaches the public URL through CI, and its trace shows in Grafana.

## 2.1 Repository

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.1.1 | MUST | One Cargo workspace at the root with `crates/domain`, `crates/application`, `crates/adapters` (a library plus the binary target); `frontend/`; `seasons/`; `docs/` | `cargo metadata` lists exactly those three crates | D, U |
| 2.1.2 | MUST | CI asserts the manifests: `domain` has no IO, async-runtime or serialization dependency; `application` depends on no HTTP or SQL crate | Adding sqlx to `domain` fails CI | D, C |
| 2.1.3 | MUST | Toolchains are pinned: Rust, Node, pnpm, and TypeScript 6.x | CI and a fresh clone resolve identical versions | D |
| 2.1.4 | MUST | No secret enters the repo; `.env.example` names every variable; push protection is on | A test fails when `.env.example` omits a variable, and GitHub reports push protection as on | C, U |
| 2.1.5 | MUST | `main` accepts changes only through pull requests with green checks | A direct push to `main` is rejected | F, C |
| 2.1.6 | MUST | A README takes a fresh clone to a running local app | Following only the README works on a clean checkout | C |
| 2.1.7 | MUST | The repo is MIT-licensed | A `LICENSE` file holds the MIT text | U |
| 2.1.8 | SHOULD | `CLAUDE.md` points to rule files in `.claude/rules/` that state the architecture rules from design sections 11, 13 and 14 | The rule files list them, and `CLAUDE.md` names their folder | C |
| 2.1.9 | WONT | A demonstrated rejection of a committed token | No test commit carries a real token | U |

Rule 2.1.5 requires checks, not approvals: one GitHub account cannot approve its own pull request.

Rule 2.1.9 replaces the old done-condition of 2.1.4.
Push protection stops only tokens in a known provider's format, and GitGuardian checks every pull request.
A made-up value passed both on 2026-11-20; showing a rejection would take a real token, which is more than this project needs.

## 2.2 Local development

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.2.1 | MUST | One command starts Postgres and the local Grafana stack | After it, Postgres accepts connections and Grafana answers on localhost | D |
| 2.2.2 | MUST | One command runs backend and frontend with reload, same-origin through a dev proxy | Editing a component updates the browser, and `/api` reaches the local backend | D, C |
| 2.2.3 | MUST | Local sign-in uses the fake identity adapter | A dev build signs in as a chosen fake player without Google | D |
| 2.2.4 | MUST | One command runs every CI check | CI calls the same recipes as that command | C |

## 2.3 Continuous integration

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.3.1 | MUST | Rust: format, lints as errors, domain tests, use-case tests on a Postgres service, sqlx metadata freshness | Each failing check blocks merge | D |
| 2.3.2 | MUST | Frontend: lint including the "client only in `lib/api`" rule, `svelte-check`, component tests, build | Importing the client from a route file fails CI | D |
| 2.3.3 | MUST | Contract: regenerated `openapi.json` and TypeScript client equal the committed ones | A handler change without regeneration fails CI | D |
| 2.3.4 | MUST | End-to-end: Playwright runs against a test image built with fake identity | A sign-in-and-pick journey passes | D |
| 2.3.5 | MUST | Release gate: the release image is built without fake identity, and CI probes it | The fake sign-in route returns 404 on the release image | D |
| 2.3.6 | MUST | Release smoke: the release image itself serves the shell, health, and the redirect to Google | All three pass on the image that will deploy | C |
| 2.3.7 | SHOULD | Supply chain: advisory and license checks for Rust and npm, plus an update bot | A dependency with a known advisory fails CI | C |
| 2.3.8 | MUST | No workflow exposes secrets to fork pull requests | No workflow uses `pull_request_target` | C |
| 2.3.9 | SHOULD | A full run takes under 10 minutes | The median of the last ten runs is under 10 minutes | R, C |

## 2.4 Container

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.4.1 | MUST | One image holds the binary and the built SPA, runs as non-root, listens on `$PORT` | `docker run` serves `/` and `/api/health` | D |
| 2.4.2 | MUST | Unknown non-API paths return the SPA shell; unknown `/api/*` paths return a JSON 404 | `/ballot` returns HTML and `/api/nope` returns JSON | C |
| 2.4.3 | MUST | Configuration comes only from environment variables, and startup fails fast | Starting without `DATABASE_URL` exits non-zero and names it | C |
| 2.4.4 | MUST | On SIGTERM the server finishes in-flight requests and flushes telemetry within 10 s | A test sends SIGTERM mid-request; the request completes and its span is exported | D, R |
| 2.4.5 | MUST | Liveness answers without the database; readiness requires it | With Postgres stopped, liveness is 200 and readiness is not | C |
| 2.4.6 | MUST | Forward-only migrations run at startup before serving | A revision with a pending migration applies it exactly once | D |
| 2.4.7 | MUST | Hashed assets are cached immutably; the shell is never cached | Response headers match in a test | C |
| 2.4.8 | MAY | The server tells an outdated open tab to reload | A request carrying an old build ID gets a "reload" outcome | C |

## 2.5 Deploy

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.5.1 | MUST | A merge to `main` deploys to Cloud Run with keyless authentication | GitHub holds no long-lived Google key, and a merge produces a serving revision | D, R |
| 2.5.2 | MUST | The service runs in an EU region with at most one instance, scale to zero, request-based billing | The deployed service description shows those values | D |
| 2.5.3 | MUST | Runtime secrets come from the platform's secret store | The service definition contains no secret value | C |
| 2.5.4 | MUST | Neon hosts one EU project, and the app's pool holds at most five connections | The pool size is visible in configuration | R, C |
| 2.5.5 | MUST | Rollback to the previous revision is one documented command | It has been rehearsed once at M0 | R |
| 2.5.6 | MUST | A billing alert fires at €1 | The budget exists at €1 and emails an address Tommy reads | D, U |
| 2.5.7 | MUST | No automatic deploy happens from 8 to 15 March 2027 | A merge inside the window does not deploy | D |
| 2.5.8 | MAY | The same image runs on Render; pursued only if proven necessary | It was deployed there once and answered health | D, U |
| 2.5.9 | WONT | A separate staging environment in v1 | Only one environment exists | U |

## 2.6 Domain

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.6.1 | MUST | By M1, a registered domain serves the app over HTTPS, with auto-renew on | `https://<domain>/api/health` returns 200 with a valid certificate | D |
| 2.6.2 | MUST | The free Cloud Run domain mapping is verified on the day the domain exists, with the Cloudflare proxy as fallback | The outcome and evidence are recorded in `docs/` | D, T |
| 2.6.3 | MUST | One canonical host; every other host redirects to it | A request to the non-canonical host returns a redirect | C |
| 2.6.4 | MUST | The domain is bought by 7 January 2027 | A registration exists on that date | U |

The Google sign-in redirect is registered per hostname, so it is re-registered when the domain arrives.

## 2.7 Telemetry baseline

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.7.1 | MUST | OTel types appear in one module of `crates/adapters` only | A CI check finds no `opentelemetry` import elsewhere | D, U |
| 2.7.2 | MUST | Each request yields a trace: HTTP span, use-case span, SQL spans | One request's trace in Grafana shows all three levels | D |
| 2.7.3 | MUST | Log events export as OTel logs carrying the trace ID, and also print as JSON | A log line in Grafana opens its trace | D |
| 2.7.4 | MUST | Metrics cover request count and latency by route and status | Both series are queryable in Grafana | C |
| 2.7.5 | MUST | Export is batched with a SIGTERM flush; per-request flush is a configuration switch | Covered by the test in 2.4.4 | D |
| 2.7.6 | MUST | Telemetry carries no email, name or pick content; players appear by ID only | A scan of spans exported by the end-to-end run finds no email pattern | R |
| 2.7.7 | MUST | A dead telemetry backend never fails or slows a request | Tests pass with an unreachable OTLP endpoint | C |
| 2.7.8 | MUST | The OTel crates are pinned and upgraded together | One pull request bumps the whole set | D |
| 2.7.9 | WONT | Browser telemetry in v1 | The frontend ships no telemetry SDK | D |

## 2.8 M0 exit: the day-three skeleton

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 2.8.1 | MUST | The skeleton is one page, one API call through the generated client, one component test, one Rust test, deployed at the platform hostname | CI is green and the page loads at that hostname | D, U |
| 2.8.2 | MUST | Svelte stays only if four checks pass by the end of day three: static SPA build with a load function, Tailwind with daisyUI, `svelte-check` on TypeScript 6, a component test in CI | Any failed check without a fix switches the frontend to Vue | D |
| 2.8.3 | MUST | Two measurements are recorded: cold-start time and CI duration | The numbers are in `docs/` | D |
| 2.8.4 | MUST | Every version marked T in the design is checked against current documentation | The design's unverified list is empty or re-dated | D |

The domain-mapping measurement moved from M0 to rule 2.6.2, because the domain is bought later.

## Notes

- **The image we test is not the image we ship.**
  End-to-end tests need fake identity, and the release must not contain it.
  Rule 2.3.6 therefore smoke-tests the release image itself.
- **Frontend and API can be version-skewed in a tab left open across a deploy.**
  The design's claim that they "can never be" skewed is false for that case.
  Rule 2.4.7 limits it, 2.5.7 removes it on the night, and 2.4.8 would close it.
- **Tommy's setup actions:** create the Google Cloud project with billing, the Neon project and the Grafana Cloud account, and install `gcloud` (O: missing on 2026-11-16).
