# Design

Status: converged on 2026-11-16, in two rounds.
Inputs: [the frame](frame.md) and [the research record](research/README.md).
Next phase: Spec.

**Origin tags:**
[R] research record, [U] Tommy, [P] established pattern, [I] Claude's inference, [T] Claude's training knowledge, unverified against current sources.
Backend internals (section 11) were never researched, so most of that section is [T].

## Decisions

| # | Axis | Choice | Origin |
|---|---|---|---|
| 1 | Runtime | One portable container; Cloud Run first, Render as the no-card fallback | [R] facts, [I] portability principle |
| 2 | Database | Postgres on Neon | [R] |
| 3 | Identity | Google sign-in; emailed one-time code as a named extension | [R], [I] |
| 4 | Domain | Paid domain, overriding the frame's original rule | [I], accepted by [U] |
| 5 | Live transport | None; viewers refresh the page | [U] |
| 6 | Frontend delivery | The Rust binary serves the SPA | [I] |
| 7 | Frontend framework | Svelte 5 with SvelteKit 3 in static SPA mode; Vue as fallback | [R], [I] |
| 8 | Session | Opaque ID in an HttpOnly cookie, row in Postgres | [P], [I] |
| 9 | Telemetry | Grafana Cloud; `tracing` bridged to OTel; batch export with SIGTERM flush | [R], [I] |
| 10 | Results ingestion | Manual through one seam; automation decided in February 2027 | [R], [I] |
| 11 | Backend internals | axum, sqlx, three-crate hexagonal layout, code-first OpenAPI | [T], [I] |
| 12 | Repo and CI | One public repo, deploy on merge, freeze in ceremony week | [R], [I] |

## Guiding principle

Changing host must be a configuration change, not a code change.
The free tier is the least stable part of the system:
Shuttle shut down, and Fly and Railway withdrew their free tiers [R].
The app must keep running year after year [U].
So the smallest unit that moves is one OCI container plus a Postgres connection string [I].
Several later choices follow from this: Postgres over SQLite, the binary serving the SPA, and owning the hostname.

## Factor map

| # | Axis | Range considered | Coupled with |
|---|---|---|---|
| 1 | Runtime class | portable container · platform-shaped WASM · always-on box | 2, 5, 9 |
| 2 | Database | Postgres · SQLite · D1 | 1 |
| 3 | Identity | password + email · emailed code · sign-in provider · passkeys | 4, 8 |
| 4 | Domain | platform hostname · free subdomain · paid | 3, 6, 8 |
| 5 | Live transport | none · polling · SSE · WebSocket | 1 |
| 6 | Frontend delivery | separate static host · served by the Rust binary | 4, 8 |
| 7 | Frontend framework | ten candidates | none found |
| 8 | Session mechanism | cookie session · JWT | 3, 4, 6 |
| 9 | Telemetry | backend × instrumentation style × export | 1, 11 |
| 10 | Results ingestion | manual · assisted · automatic | none |
| 11 | Backend internals | web framework, DB layer, layering, API contract | 2, 9 |
| 12 | Repo and CI | layout, visibility, pipeline | 1 |

Tightest coupling (4-6-8):
a frontend and an API on two different platform hostnames are different sites,
so the API's cookie becomes third-party [I].

Outside the map: the season data model (ties, double nominations) belongs to Spec.
GDPR and trademark exposure remain unexamined.

## 1–2. Runtime, database, first host

**Decision:** the container + Postgres class, with Cloud Run + Neon as first host and Render + Neon as the no-card fallback.

| Candidate | Card | Idle months | First request after idle | Ceremony night | Rust code shape | Evidence |
|---|---|---|---|---|---|---|
| A1 Container on Cloud Run + Neon | Required | No penalty found | Container start (unmeasured) plus Neon resume | Well inside the free CPU allowance | Standard async Rust | Confirmed, except cold start |
| A2 Same container on Render + Neon | Not required | No penalty; may restart any time | About 1 minute | Fine once awake | Standard async Rust | Confirmed |
| A3 Northflank service + its database | Not required | "Always on" | None | Fine | Standard async Rust | Regions, sizes, volumes unknown |
| A4 Cloudflare Workers + D1 | Not checked | No penalty stated | None | 100k requests a day cap | WASM, 10 ms CPU per request | Limits confirmed; Rust ergonomics inferred |

- **Why A1:** Cloud Run and Neon are the compute and database with confirmed no idle penalty and a standard container [R].
- **Why not A4:** the platform shapes the Rust into an atypical subset, which undercuts the `tracing` and OTel learning goal [I].
- **Why not A3:** it rests on the least verified free tier in the research.
- **Why Postgres over SQLite:** SQLite needs a persistent disk, which neither A1 nor A2 offers free [R].
- **A1 and A2 are interchangeable:** the same image runs against the same database.
- **Rejected hosts:**
  - Lambda: the free-plan account closes after six months.
  - Koyeb: card hold, and its sleep cannot be woken by HTTP/2.
  - Oracle VM: its idle-reclaim rule matches this app's profile.
  - Home server: owning hardware was never confirmed.
  - Vercel: beta runtime with a 300 s cap.
  - Fly, Railway, Shuttle, Cloudflare Containers: not free.
- **Breaking case:** a Google billing account can incur charges.
  Capping instances at one and setting a budget alert bounds it [I].

## 3–4. Identity and domain

**Decision:** Google sign-in now; an emailed one-time code is the named extension if a real person cannot sign in.
**Decision:** buy a domain.

| Candidate | Outbound email | Covers the pool | Security code we write | Open sign-up abuse | Ceremony-night failure |
|---|---|---|---|---|---|
| B1 Password + confirmation email | Yes | Everyone | Most: hashing, reset, tokens, lockout | Form can mail strangers and burn quota | Reset email must arrive |
| B2 Emailed one-time code | Yes | Everyone | Medium | Same vector | Login email must arrive at midnight |
| B3 Google sign-in | No | Google account holders | Least: one library flow | A bot needs a Google account | Google must be up |
| B4 Passkeys only | No | Device-dependent | Medium | Low | Lost device |
| B5 B3 with B2 as fallback | For fallback users | Everyone | B3 plus B2 | B2's vector remains | Either path |

- **Why B3:** it is the only candidate that meets "verified email" and €0 without sending mail [R], [I].
  Google returns an `email_verified` claim with no user cap for basic scopes [R].
- **Why not B1:** the largest security-critical surface in the app, built against a deadline,
  and delivery to corporate Microsoft mailboxes is not guaranteed [R].
- **Why not B4:** a passkey carries no email claim, so it fails "verified email" [R].
- **Why no Microsoft sign-in:** its email claim "isn't guaranteed to be correct" [R].
- **Flip conditions:** B5 if anyone in the pool lacks a Google account; B1 if auth itself becomes a learning topic.

**Why a paid domain, against the frame's original rule:**

- Without a domain, a host change alters the URL everyone bookmarked, which breaks the guiding principle [I].
- The sign-in redirect is registered per hostname [I].
- A `.de` renews at USD 4.07 a year [R].
- The frame's constraint was amended accordingly.

## 5. Live transport

**Decision:** none. Viewers refresh the page to see new results [U].

| Candidate | Latency | Server state | Phone sleep or network switch | Cloud Run fit | Testing |
|---|---|---|---|---|---|
| No transport, manual refresh | Until the viewer refreshes | None | Not applicable | A handful of requests | Nothing extra |
| Polling with conditional requests | Up to the interval, 5–10 s | None | Self-healing on the next poll | 63k–126k of 2M free requests | Plain request tests |
| SSE | Under 1 s | In-process broadcast; breaks beyond one instance | Reconnect, then refetch | One request per client, 60-minute cap [R] | Needs stream tests |
| WebSocket | Under 1 s | Same as SSE | Hand-written reconnect | Supported [R] | Hardest |

- **Why none:** Claude recommended polling; Tommy judged any streaming mechanism overkill.
  A mechanism that does not exist cannot fail on the one night that cannot be rehearsed.
  Viewers learn the winner from the TV, and the app shows the score consequence [I].
- **Nothing is foreclosed:** polling or SSE can be added later.
- **Known failure mode:** a page left open looks current when it is stale.
  A proposed remedy (showing "N of 24 categories announced, last result at HH:MM") is undecided; see Open items.

## 6. Frontend delivery

**Decision:** the Rust binary serves the SPA.

| Candidate | Origins the browser sees | First paint after idle | Deploy units | Cookie and CORS | Unverified |
|---|---|---|---|---|---|
| E1 Rust binary serves the SPA | One | Waits for container start | One image | First-party cookie, no CORS | Free custom-domain mapping on Cloud Run [T] |
| E2 SPA on a static host, API on a subdomain | Two, same site | Instant | Two | Cookie works; CORS and preflights required | The same mapping, for the API |
| E3 Cloudflare serves the SPA and proxies `/api` | One | Instant | Two plus a proxy Worker | First-party cookie, no CORS | Worker's 100k requests/day cap covers API calls [R] |

- **Why E1:** one container is the smallest unit that moves, and frontend and API can never be version-skewed [I].
- **Why not E3:** a third component in a second language sits in every request path.
- **Breaking case:** a blank page during cold start, which nobody has measured [R], and the unverified domain mapping.
- **Direction:** the day-three skeleton measures both; E3 is the fallback and needs no backend change.

## 7. Frontend framework

**Decision:** Svelte 5 with SvelteKit 3 in static SPA mode; Vue is the fallback.

Four hard requirements screened ten candidates [R] (the gates themselves are [I], derived from the frame):

| Candidate | Static SPA documented | Core model stable ≥ 12 months, not being replaced | TypeScript is the authoring language | Router and tests documented | Result |
|---|---|---|---|---|---|
| Vue | Yes | Yes | Yes | Yes | Pass |
| Svelte | Yes | Yes (5.0 in October 2024) | Yes | Yes | Pass |
| Angular | Yes | Yes | Yes | Yes | Pass |
| Solid | Yes | No: 2.0 RC removes core APIs | Yes | Yes | Near miss |
| Qwik | Not found | No: 2.0 at RC | Yes | Unknown | Fail |
| Lit | Yes | Yes | Yes | No: router is 0.1 labs | Fail |
| Marko | Undocumented | Yes | Partly: own language | Unknown | Fail |
| Ripple | Yes | No: pre-1.0 | Yes | No | Fail |
| Remix 3 | Undocumented | No: six weeks old | Yes | Unknown | Fail |
| htmx | Yes | Yes | No: Rust must return HTML | Yes | Fail |

| Survivor | New concepts for a React developer | Delivery risk in 9 weeks | Template type-checking | Testing | Libraries for this app | Claude's fluency [I] |
|---|---|---|---|---|---|---|
| Vue 3.5 | Moderate: proxy reactivity, templates | Lowest: line is two years old | `vue-tsc`, pinned to TS 6 | Current | Richest | High |
| Svelte 5 | High: compiler, runes, own file format | Medium: SvelteKit 3 is six weeks old | `svelte-check`, pinned to TS 6 | Current | Adequate | High for Svelte 5, low for SvelteKit 3 |
| Angular 22 | Highest: DI, classes, largest surface | Medium: settled, but most to learn | Built in, strict by default | Current | Built in | Medium |

- **Why Svelte:** this axis exists for learning [U], and Svelte moves furthest from React.
  The app is four screens, so the delivery risk has a small base [I].
- **Why not Vue:** if the Composition API feels too close to hooks, the learning goal is under-delivered [I].
- **Why not Angular:** the largest concept surface for four screens, and 48% retention [R].
- **Solid:** worth revisiting for season two once 2.0 is stable.
- **TypeScript:** pinned to 6 until TypeScript 7.1 restores template checking [R].
- **Breaking case:** SvelteKit 3 is new to both authors, and SPA mode is the path its docs discourage [R].
- **Risk control:** a walking skeleton [P] by day three: one route, one API call, one component test, a static build.
  If it fights us, switch to Vue.
- **Flip condition:** Vue if lowest risk or job-market transfer outweighs novelty.

## 8. Session

**Decision:** an opaque session ID in an HttpOnly cookie, with the session row in Postgres.

| Candidate | Revocable | Per-request cost | Script-readable token | Code we write | What it buys here |
|---|---|---|---|---|---|
| S1 Opaque ID in an HttpOnly cookie, row in Postgres | Yes, delete the row | One indexed lookup | No | Least | Fits one database and 50 users |
| S2 JWT in an HttpOnly cookie | Only with a denylist, which is S1's lookup | Signature check | No | More: expiry, refresh, key rotation | Statelessness, unused with one instance |
| S3 JWT in browser storage, sent as a header | Same as S2 | Signature check | Yes | Most | Cross-site use, unneeded when the binary serves the SPA |

- **Why S1:** every meaningful request reads the database anyway, so statelessness buys nothing [I].
- **On JWT, which Tommy floated:** Google sign-in already requires validating a JWT, the ID token,
  so that learning happens where a JWT is inherent [I].
- **Breaking case:** cookies are sent automatically, which opens cross-site request forgery.
- **Direction:** SameSite cookie, JSON-only mutations and an Origin check [P].
  Sessions last the whole season so nobody signs in at midnight [I].

## 9. Telemetry

**Decision:** Grafana Cloud as backend; `tracing` for spans and logs, bridged to OTel, with the OTel API for metrics; batch export with a flush on SIGTERM.

**Backend** (all cells [R])

| Backend | Signals over OTLP | Retention | EU region | Card | Headroom | Local twin |
|---|---|---|---|---|---|---|
| Grafana Cloud | Traces, metrics, logs | 14 d | Per stack (probable) | No | 50 GB each | Yes: `otel-lgtm` image |
| Honeycomb | Traces, metrics | 60 d | Yes (probable) | Not stated | 20M events | No |
| New Relic | All three | ≥ 8 d | Yes | No | 100 GB | No |
| Axiom | All three | 30 d | No, US only | No | 500 GB | No |
| Sentry | Traces, logs | 30 d | Not checked | Not stated | 5M spans | No |
| Logfire | All three | 30 d | Yes | No | 10M records, pauses at cap | No |
| Better Stack | All three | 3 d | Not stated | Not stated | 3 GB | No |
| HyperDX | Not checked | 3 d | Not checked | No | 3 GB | Not checked |

- **Why Grafana Cloud:** all three signals, no card, an EU region,
  and the same stack runs locally, so development and production share tools.
- **Why not Honeycomb:** no local twin; it remains the strongest alternative, with 60-day retention.
- **Late binding:** OTLP makes the backend an endpoint and a header.
- **Breaking case:** 14-day retention loses the ceremony data by April.
- **Direction:** dual export to Honeycomb is a configuration change.

**Instrumentation style**

| Candidate | Span systems in the process | Library spans visible | Official guidance | Learning goal |
|---|---|---|---|---|
| I1 `tracing` for spans and logs, bridged; OTel API for metrics | One | Yes | Partly: docs prefer the OTel API for spans [R] | Both crates exercised |
| I2 OTel API for spans, `tracing` for logs | Two | Only with the bridge anyway | Yes | Less `tracing` |
| I3 `tracing` to stdout JSON, no OTel | One | As log lines | No | Fails the OTel goal |

- **Why I1:** libraries emit `tracing` spans [T], so the bridge is present in every design that shows them.
  I2 then adds a second span API on top.
- **Breaking case:** six breaking OTel releases in two years, with the bridge lagging by days [R].
- **Direction:** confine every OTel type to one telemetry module, and upgrade the crate set as a unit.

**Export on Cloud Run**

| Candidate | Span loss | Added latency | Cost | Visibility delay |
|---|---|---|---|---|
| X1 Batch, flush on SIGTERM | Only on crash or SIGKILL | None | Free | Until the next request or shutdown, at most 15 min [R] |
| X2 Flush before each response | None | One export round-trip | Free | None |
| X3 Instance-based billing | None | None | Idle tails billed; may leave the free tier [I] | None |
| X4 Collector sidecar | Throttled like the app [R] | None | Free | Same as X1 |

- **Why X1:** Cloud Run suspends background work between requests [R], so spans are delayed, not lost;
  SIGTERM gives 10 s of CPU for the final flush [R].
- **X2 stays switchable by configuration** for debugging sessions.
- **Breaking case:** a trace missing right after a lone request looks like a bug.

**Browser telemetry is out of v1:** it is experimental [R], and telemetry depth is on the frame's cut-first list [U].

## 10. Results ingestion

**Decision:** manual entry now, through one seam; the choice between assisted and automatic is made in February 2027.

| Candidate | Wrong winner shown | Latency | Work before 21 January | Ceremony-night burden | Terms exposure | Testable before the night |
|---|---|---|---|---|---|---|
| D1 Manual | Admin slip only | Seconds | Required anyway | 24 taps | None | Fully |
| D2 Assisted: two signals agree, admin confirms | Both signals and the admin wrong | About a minute plus a tap | None | 24 confirmations | Market terms unread | Only on replayed 2026 data |
| D3 Automatic when two signals agree | Both signals wrong | About a minute | None | Watch for errors | Market terms unread | Only on replayed 2026 data |
| D4 Automatic from one source | Happened: two wrong winners in 2025 | About a minute | None | Watch for errors | Depends on source | Only on replayed 2026 data |

- **Why D1 first:** the options nest; every one needs manual entry as fallback and correction tool [I].
  If results enter through one seam, postponing the choice costs nothing.
- **Why not D4:** its failure already occurred in 2025 [R].
- **Open between D2 and D3:** D3's risk is that matching free-text winners and market strings to nominees can only run for real once a year.
- **Nominees:** a season seed file, reviewed in a pull request [I], because no source beat 1.5 hours [R].

## 11. Backend internals

Not researched; cells are [T] unless tagged otherwise.
Crate versions must be verified when the workspace is created.

**Web framework: axum**

| Candidate | Middleware model | `tracing` and OTel pieces | OpenAPI path | In-process tests | Opinions imposed |
|---|---|---|---|---|---|
| axum 0.8 | tower layers | `tower-http`, `axum-tracing-opentelemetry` [R]; same org as `tracing` | `utoipa-axum` [R] | Call the router directly, no socket | Few |
| actix-web 4 | Its own | `tracing-actix-web` maintained [R] | utoipa supports it | Built-in utilities | Few |
| Loco | tower, built on axum | Inherits axum's | Not known | Own harness | Many: ORM, layout, auth |
| Poem | Its own | Built-in OTel middleware | First-party `poem-openapi` | Test client | Few |

- **Why axum:** `tracing`, the OTel bridge, `tower-http` and axum share the tower model, so instrumentation composes as layers [I].
- **Why not actix-web:** it sits outside the tower ecosystem, so the OTel pieces from the research do not apply.
- **Why not Loco:** its opinions replace the architecture Tommy asked to design [U].
- **Rejected:** Rocket (slow release cadence), warp (maintenance mode), Salvo (small ecosystem).
- **Breaking case:** extractor compile errors are opaque, and 0.x releases break.
- **Direction:** keep handlers thin.

**Database layer: sqlx**

| Candidate | Query safety | Async | Migrations | Test isolation | Leak into domain |
|---|---|---|---|---|---|
| sqlx | SQL checked at compile time | Native | Built in | Per-test database | Low |
| Diesel + diesel-async | Typed DSL, no database at build | Add-on crate | CLI | Manual transactions | Low to medium |
| SeaORM | Checked at runtime | Native | Built in | Mock connection | High: entities invite reuse as domain types |
| Raw tokio-postgres | None | Native | External tool | Manual | Low |

- **Why sqlx:** the domain is a handful of tables, and SQL is the transferable skill [I].
- **Breaking case:** compile-time checks need a database or committed query metadata.
- **Direction:** CI verifies the metadata is fresh.

**Layering: hexagonal, as three crates**

| Candidate | Dependency direction enforced by | Domain testable without IO | Seams for clock, sign-in, results | Boilerplate |
|---|---|---|---|---|
| L1 Hexagonal as crates | The compiler | Yes | Traits | Highest |
| L2 Hexagonal as modules in one crate | Convention and review | Yes | Traits | Medium |
| L3 Vertical slices per feature | Nothing | By discipline | Ad hoc | Low |
| L4 Handler → service → repository | Nothing | No | None | Lowest |

- **The three crates:** domain, application, and adapters plus binary.
- **Why L1:** three rules must be tested exactly yet depend on time or IO: lock time, scoring with ties, verified email.
  A crate boundary is the only mechanism that makes "the domain imports sqlx" a compile error [I].
- **Breaking case:** traits with one implementation are ceremony.
- **Rule:** a port exists only where a test needs a fake or a decision was deferred:
  clock, identity provider, results source, storage.

**API contract: code-first OpenAPI**

| Candidate | Source of truth | Conformance checked by | Covers paths and types | Both sides can start |
|---|---|---|---|---|
| C1 Code-first: utoipa → committed `openapi.json` → generated TS client | Rust handler types | Compiler plus a CI freshness check | Both | Once the handler signature exists |
| C2 Spec-first: hand-written OpenAPI | A YAML file | Nothing found in research | Both | Immediately |
| C3 `ts-rs` type export | Rust structs | Compiler | Types only [R] | Once the structs exist |
| C4 Hand-written on both sides | None | Nothing | Neither | Immediately |

- **Why C1:** only the code-to-spec direction has compiler-checked conformance in Rust [I].
- **Fit with spec-driven development [U]:** the feature spec in `docs/` defines the endpoint,
  and the committed `openapi.json` makes every API change a reviewable diff.
- **Breaking case:** hand-written annotations such as status codes can drift from behaviour.
- **Direction:** add a contract test later.

## 12. Repo and CI

- **Layout:** one repo holds the Cargo workspace, the frontend and `docs/`.
  This is forced: one pull request changes API, generated client and UI, and one image ships [I].
- **Visibility: public.**

  | Visibility | CI minutes | Code exposure | Name exposure |
  |---|---|---|---|
  | Public | Unlimited [R] | Anyone can read it | "oscar" in a public repo name; trademark unexamined |
  | Private | About 2,000 a month [R] | None | None |

  A full CI run is roughly 17 minutes [I, low confidence], so private would allow about four pushes a day.
- **Deploy trigger: every merge to main, plus a deploy freeze in ceremony week [I].**

  | Deploy trigger | Merge to live | Rollback | Ceremony-night risk |
  |---|---|---|---|
  | Y1 Every merge to main | Minutes | Shift traffic to the previous revision [R]; schema stays forward | A late merge can break the night |
  | Y2 On a release tag | One manual step | Same | Lower |
  | Y3 By hand | Manual | Same | Depends on the admin |

- **Migrations:** forward-only, expanding before they contract [P], because rollback moves code but not schema.

## Risk controls

- **Day-three skeleton:** one route, one API call, one component test, a static build, deployed.
  It decides Svelte versus Vue, and measures cold start and the custom-domain mapping.
- **Budget guard:** one Cloud Run instance at most, plus a budget alert.
- **Deploy freeze** in ceremony week.
- **Telemetry isolation:** all OTel types live in one module.

## Deferred decisions

- **February 2027:** assisted versus automatic results ingestion.
- **After January 2027, optional:** polling or SSE, if manual refresh proves insufficient.
- **On demand:** emailed one-time code, if someone cannot use Google sign-in.
- **Season two:** revisit Solid once 2.0 is stable.

## Carried into Spec as unverified

- Free custom-domain mapping on Cloud Run [T].
- Container cold-start time.
- Current crate versions for the backend stack [T].
- Whether everyone in the pool has a Google account.

## Open items

- Whether the leaderboard shows a staleness indicator ("N of 24 categories announced, last result at HH:MM").
  Proposed by Claude; not decided.
- Which domain name and TLD to buy.
