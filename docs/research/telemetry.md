# Research: zero-cost OpenTelemetry

Researched on 2026-11-16. Part of [the research record](README.md).
Labels: **C** confirmed in a primary source, **P** probable (secondary source or inference), **U** uncertain or conflicting.
Bracketed numbers refer to the source list at the end.

## 1. Hosted backends accepting OTLP

No inactivity or account-deletion policy was found for any provider (see Unknowns).

| Backend | Permanent free tier | Retention | Seats | Card | OTLP endpoint / auth | EU |
|---|---|---|---|---|---|---|
| Grafana Cloud | 10k metric series; 50 GB each logs, traces, profiles; 50k frontend sessions per month (C)[1] | 14 d (C)[1] | 3 (C)[1] | No (C)[1] | `https://otlp-gateway-<REGION>.grafana.net/otlp`, HTTP, `Authorization: Basic` (C)[2] | Per-stack region (P, inferred from `<REGION>`)[2] |
| Honeycomb | 20M events/mo, 100M metric data points, 2 triggers, no SLOs (C)[3][4] | 60 d (C)[4] | Unlimited (C)[4] | Not stated | `api.honeycomb.io` / `api.eu1.honeycomb.io`, header `x-honeycomb-team` (P)[5] | Yes (P)[5] |
| New Relic | 100 GB/mo, all signals (C)[6] | ≥8 d (C)[6] | 1 full + unlimited basic (C)[6] | No (C)[6] | `otlp.nr-data.net` / `otlp.eu01.nr-data.net`, gRPC + HTTP, header `api-key`, 1 MB payload cap (C)[7] | Yes (C)[7] |
| Axiom Personal | 500 GB/mo ingest, 25 GB storage, 3 datasets, 3 monitors; logs, traces, metrics (C)[8][9][10] | 30 d (C)[8] | 1 (C)[9] | No (C)[8] | `https://AXIOM_DOMAIN/v1/{traces,logs,metrics}`, HTTP only documented, `Authorization: Bearer` + `X-Axiom-Dataset` (C)[10] | Personal is US-only (C)[9] |
| Sentry Developer | 5k errors, 5M spans, 5 GB logs, 50 replays (C)[11] | 30 d (C)[11] | 1 (C)[11] | Not stated | `https://o<org>.ingest.sentry.io/api/<project>/integration/otlp/v1/traces` (and `/logs`), header `x-sentry-auth: sentry sentry_key=<public key>` (C)[12] | Not checked |
| Pydantic Logfire Personal | 10M records/mo (logs + spans + metrics); ingestion pauses at the cap (C)[13] | 30 d (C)[13] | 1 + 2 read-only guests, 3 projects (C)[13] | No (C)[13] | `logfire-us.pydantic.dev` / `logfire-eu.pydantic.dev`, HTTP + gRPC, `Authorization: <write-token>` with no `Bearer` (C)[14] | Yes (C)[13] |
| Better Stack | 3 GB logs, 3 GB traces, 30 GB metrics (C)[15] | 3 d logs and traces (C)[15] | Not stated | Not stated | `https://$INGESTING_HOST/v1/...` or gRPC `:443`, `Authorization: Bearer $SOURCE_TOKEN` (C)[16] | Not stated |
| HyperDX | 3 GB/mo (C)[17] | 3 d (C)[17] | 1 (C)[17] | No (C)[17] | Not checked | Not checked |
| Uptrace Cloud | "Free up to 50 GB and 5,000 timeseries per month", but the same page also says "Free for 14 days" (U)[18] | 28 d (C)[18] | Not stated | No (C)[18] | Not checked | Not checked |

Other Sentry facts: OTLP covers traces and logs, not metrics; span events are dropped; span links and array attributes are not searchable (C)[12].
It was announced as open beta in October 2025 (P)[19]; the current docs carry no beta label (C)[12].

Logfire has a Rust SDK: crate `logfire` 0.14.0, released 2026-11-15 (C)[20].

ClickHouse announced Managed ClickStack (the HyperDX successor) in beta in February 2026 (P)[21].

**No permanent free tier, or not usable:**

- SigNoz Cloud: Teams plan from $49/mo (C)[22].
- OpenObserve Cloud: 14-day trial, then $0.50/GB (C)[23]. Third-party pages claim a free tier (U).
- Dash0: 14-day trial only (C)[24].
- Elastic Serverless: no free tier listed (C)[25].
- Datadog: free tier is infrastructure metrics only, 5 hosts, 1-day retention (C)[26].
- Dynatrace: 15-day trial (P)[27].
- Lightstep / ServiceNow Cloud Observability: end of life 2026-03-01 or contract end, with no successor (C)[28].
- Baselime: the site still shows "Get started for free" under a "joined Cloudflare" banner (C)[29].
  Cloudflare had planned to sunset it in late 2024 (P)[30]; its real status is unclear (U).
- Cloudflare: Workers Logs free tier is 200k events/day with 3-day retention.
  Cloudflare exports OTLP outward; no ingestion endpoint for non-Cloudflare apps is documented (C)[31][32].

**Cloud-provider allowances:**

- Google Cloud: Trace, first 2.5M spans/mo per billing account; Logging, 50 GiB per project per month with 30-day storage; Monitoring, 150 MiB per billing account (C)[33].
  OTLP goes to `telemetry.googleapis.com` over HTTP for traces, metrics and logs, authenticated with Application Default Credentials and IAM role `roles/telemetry.writer` (C)[34].
- AWS: X-Ray, 100k traces recorded and 1M retrieved per month; CloudWatch, 5 GB logs and 10 custom metrics.
  The page does not say whether this is always-free or 12-month (C)[35].
  OTel metric ingestion is listed at $0.50/GB with no free allowance (C)[35].

## 2. Self-hosted / local

| Option | Facts |
|---|---|
| Jaeger v2 (2.23) | All-in-one image; "transient in-memory storage"; OTLP on 4317/4318 (C)[36]. v1 reached end of life 2025-12-31 (P)[37]. Traces only (P). |
| `grafana/otel-lgtm` | Collector, Prometheus, Tempo, Loki, Pyroscope and Grafana in one container; "intended for development, demo, and testing"; UI on 3000; 1 GB noted as insufficient, 2 GB is the script default; data persists if `/data` is mounted (C)[38]. |
| Aspire dashboard standalone | `mcr.microsoft.com/aspire/dashboard`; UI 18888, OTLP 4317/4318; in-memory by default with optional persistence; "designed for development and short-term diagnostics" (C)[39]. |
| otel-desktop-viewer | Local CLI; traces, metrics and logs; DuckDB; 512 MB in-memory cap, or 2 GB with `--db`; ports 8000/4317/4318 (C)[40]. |
| SigNoz self-hosted | ≥4 GB Docker memory; runs ClickHouse, Keeper, Postgres, collector and server (C)[41]. |
| OpenObserve self-hosted | Single binary or Docker on port 5080; AGPL-3.0 (C)[23][42]. Minimum RAM is undocumented; a "~512 MB" figure comes from a competitor's blog (U). |
| Uptrace self-hosted | Needs ClickHouse and PostgreSQL, plus Redis (P)[43]. |
| Collector file exporter | Alpha; newline-delimited OTLP JSON; size-based rotation; the `otlpjsonfile` receiver reads it back (C)[44]. |

Free VM capacity: Oracle Always Free gives two AMD micro instances (1/8 OCPU, 1 GB each) plus Ampere A1 totalling 2 OCPU / 12 GB.
Instances idle for 7 days (CPU, network and A1 memory all under 20%) "may be reclaimed" (C)[45].
Inference: of these shapes only the A1 meets SigNoz's 4 GB minimum, and a near-idle app matches the reclamation criteria.

**Unconventional approaches:**

- **Spans or logs to stdout, read from platform logs.**
  `opentelemetry-stdout` is "for debugging and learning… not recommended for production", and its format is "subject to change" (C)[46].
  On Cloud Run, JSON log lines carrying `logging.googleapis.com/trace` and `spanId` are linked to traces (C)[47], within the free 50 GiB Logging allotment (C)[33].
  Limitation (inference): this yields searchable log lines, not a trace waterfall, and retention and search depend on the host's log product.
- **Spans in the app's own database.**
  No off-the-shelf Rust exporter was found (U; the search was not exhaustive).
  It would need a custom `SpanExporter` and a custom query UI (inference).
  otel-desktop-viewer uses the same pattern locally with DuckDB (C)[40].

## 3. OpenTelemetry in Rust

| Crate | Version (date) | Status |
|---|---|---|
| `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `opentelemetry-appender-tracing`, `opentelemetry-semantic-conventions` | 0.33.1 (2026-10-09) (C)[20] | Logs and Metrics API, SDK and OTLP exporter: Stable. Appender: Stable. Traces API, SDK and OTLP: Beta. Context and Propagators: Beta (C)[48] |
| `tracing-opentelemetry` | 0.34.0 (2026-09-23), targets OTel 0.33 (C)[20][49] | Maintained by tokio-rs, outside the OTel project (C)[50] |

- The OTLP logs and metrics exporters were declared stable in 0.33.1, while "Distributed Tracing remains pre-stable"; all crates are still 0.x (C)[51].
  MSRV is 1.75 (C)[48].
- Breaking minor releases of `opentelemetry`: 0.28 (Feb 2025), 0.29 (Mar), 0.30 (May), 0.31 (Sep), 0.32 (May 2026), 0.33 (Sep 2026): four in 2025, two in 2026 (C)[20].
  `tracing-opentelemetry` followed each one 5–10 days later, from 0.29 to 0.34 (C)[20].
- Compatibility pain:
  - Version numbers are "not synchronized", with the bridge running one minor ahead (C)[52].
  - 0.32.0 renamed the `code.*` attributes and changed `otel.status_message` to `otel.status_description` (C)[49].
  - 0.33.0 carried a breaking deadlock fix (C)[49].
  - A circular dependency between the core repo and the bridge blocked releases (issue #3190) (C)[53].
  - All OTel crates must share one minor version (P; the changelog mentions a "version mismatch" error message)[49].
- `tracing` versus the OTel API:
  - The proposal to adopt tokio `tracing` as the OTel tracing API (#1689) was closed on 2025-03-26.
    The project "will continue to build and maintain its own Distributed Tracing API" (C)[54].
  - #1571 settled on "Option 3 – Maintain Both APIs" and was closed on 2026-03-18: "Interop works now; unified context… seems unrealistic" (C)[55].
    The interop is the context activation added in `tracing-opentelemetry` 0.32 (C)[49].
  - `docs/traces.md` (marked Work-In-Progress) says to prefer the OTel Tracing API for custom spans.
    `tracing` "should be used for logs and events, not for OpenTelemetry spans".
    The bridge is third-party; it is "nearly identical" for internal spans and "a workaround" for edge spans (C)[50].
  - `docs/logs.md` (Stable) says to use `tracing` with `opentelemetry-appender-tracing` for logs (C)[56].
- Helper crates (C)[20] unless noted:
  - `init-tracing-opentelemetry` 0.44.0 (2026-10-28): ten releases in 2026, pins OTel 0.33, has a flush-on-drop guard, and reads `OTEL_*` variables including the sampler [57].
  - `axum-tracing-opentelemetry` 0.42.1: extracts `traceparent`, targets axum 0.8 [58].
  - `tower-http` 0.7.1: `TraceLayer` emits `tracing` spans and events only, with no OTel propagation [59].
  - `tracing-actix-web` 0.7.26 (2026-11-08), now in the actix-extras repo.
  - `reqwest-tracing` 0.7.1 (2026-05-19).
  - `opentelemetry-instrumentation-tower` 0.19.0 and `-actix-web` 0.25.0 (2026-09-21).
    The official docs say "No stable instrumentation libraries exist yet" [50].
  - `actix-web-opentelemetry`: last release 2025-04-02.
  - `sqlx-tracing` 0.2.1 and `diesel-tracing` 0.4.0.
    There is no `opentelemetry-instrumentation-sqlx` crate.
- gRPC versus HTTP in `opentelemetry-otlp`:
  - Default features are `http-proto` + `reqwest-blocking-client` (C)[60].
  - `grpc-tonic` adds tonic, prost and tokio, and needs the provider created inside a tokio runtime (C)[60][61].
  - TLS is opt-in: `tls-ring` or `tls-aws-lc` plus a roots feature for gRPC, `reqwest-rustls` for HTTP (C)[60].
  - Async HTTP clients are not supported by the default processors (C)[60].
  - Axiom documents HTTP only (C)[10]. Browsers cannot use gRPC (C)[62].

## 4. Operational facts

- The Rust `BatchSpanProcessor` exports from a dedicated background thread, on batch size or timer.
  `shutdown()` flushes it, and can deadlock if called on a current-thread tokio runtime (C)[61].
- `SimpleSpanProcessor` exports synchronously and is "useful for debugging and testing".
  With `reqwest-blocking-client`, spans "must be emitted from a non-tokio runtime thread" (C)[63].
- Cloud Run, request-based billing:
  - CPU is allocated only during requests.
    Background activity is suspended and resumes on the next request (C)[64][65].
  - The docs name "monitoring agents like OpenTelemetry" as needing instance-based billing (C)[64].
  - Shutdown is SIGTERM, then 10 s with CPU, then SIGKILL.
    An idle instance is kept at most 15 minutes (C)[66].
  - The free tier covers both modes: 180k vCPU-seconds, 360k GiB-seconds and 2M requests for request-based; 240k and 450k for instance-based (C)[67].
  - A collector sidecar is documented (C)[68].
    Sidecars get CPU only while a request is being processed (C)[66].
  - Inference: spans batched after the response may be delayed until the next request, or lost without a SIGTERM flush.
- Render free tier: spins down after 15 minutes, takes about 1 minute to spin up, and "might restart… at any time" (C)[69].
- Lambda: the collector layer's `decouple` processor (alpha) delays the freeze until export finishes (C)[70].
- Sampling: the SDK offers `AlwaysOn`, `AlwaysOff`, `TraceIdRatioBased`, `ParentBased` and `JaegerRemote` (C)[71].
  Per-span filtering can break traces, and tail sampling is done in the Collector (C)[50].
- Browser:
  - "Client instrumentation for the browser is experimental and mostly unspecified" (C)[72].
  - `@opentelemetry/sdk-trace-web` is at 2.12.0 and `instrumentation-fetch` at 0.301.2 (C)[73].
  - Fetch instrumentation skips header injection on cross-origin URLs unless they match `propagateTraceHeaderCorsUrls` (C, from package source)[73].
  - `traceparent` is not CORS-safelisted, so it triggers a preflight and needs `Access-Control-Allow-Headers` (C for the list, inference for the header)[74].
  - The OTel docs advise a reverse proxy in front of the collector, plus CORS and CSP configuration (C)[62].
- Browser ingest keys:
  - Honeycomb recommends a Collector or proxy so the key stays server-side (C)[75].
  - Logfire issues restricted frontend application tokens, with "a backend proxy is not required" (C)[76].
  - Grafana Frontend Observability sets a CORS allowed origin per app (C)[77].
  - Sentry's OTLP header carries the public key (C)[12].
- PII: the OTel guidance is data minimisation, with responsibility on the implementer.
  Collector `attribute`, `filter`, `redaction` and `transform` processors are named.
  Its examples hash `user.email` or keep only the email domain, and it warns that hashing predictable values is reversible (C)[78].

## 5. Adjacent tooling

| Item | Facts |
|---|---|
| `tracing-test` | 0.2.6 (2026-02-07; previous release 2024-06). `#[traced_test]`, `logs_contain`. Captures own-crate logs only unless `no-env-filter` is enabled (C)[20][79]. |
| `tokio-console` | 0.1.14, with `console-subscriber` 0.5.0 (2025-10-30). Needs `--cfg tokio_unstable`. Port 6669 (C)[20][80]. |
| `tracing-error` | 0.2.1 (2024-11-29). "Currently experimental" (C)[20][81]. |
| `metrics` | 0.24.6 (2026-05-13). A facade with official Prometheus and TCP exporters; no OTel exporter is listed (C)[20][82]. OTel metrics are Stable, and `opentelemetry-prometheus` is Beta (C)[48]. |
| Health probes | Cloud Run has startup, liveness and readiness probes. They are billed for CPU and memory, not as requests (C)[83]. |
| UptimeRobot free | 50 monitors, 5-minute interval, HTTP/port/ping/keyword (C)[84]. The terms say "available for any use, including commercial" (C)[85]; third-party posts claim a non-commercial restriction (U). |
| Better Stack free | 10 monitors, 10 heartbeats, a status page, 3-minute checks (C)[86]. |

Keep-awake pinging: Render's idle timer resets on inbound HTTP, and each workspace gets 750 free instance hours per month (C)[69].
A service awake all month uses about 744 hours (inference).
Render's free-tier docs do not address pinging (C)[69].
A community reply attributed to Render support calls it "abusing the service" (P)[87].

## Unknowns

- Inactivity or deletion policies for free accounts at Grafana, Honeycomb, New Relic, Axiom, Sentry, Logfire and Better Stack.
- Card requirement for Honeycomb, Sentry and Better Stack.
- EU region for Sentry, Better Stack and HyperDX; seat count for Better Stack.
- Whether Uptrace's 50 GB is permanent, and whether HyperDX's free plan will survive the ClickStack transition.
- Whether AWS's X-Ray and CloudWatch allowances are permanent.
- Whether Sentry OTLP is GA.
- Measured binary-size difference between gRPC and HTTP exporters.
- CORS behaviour of each vendor's OTLP endpoint for direct browser export.
- Render's terms-of-service wording on keep-alive, and its SIGTERM grace period.
- Koyeb's free compute status (the pricing page seen by this thread showed only a free Postgres tier; the hosting thread found a free web service).
- Whether a 1.0 date exists for Rust traces.
  Meeting notes that mention 2026 targets were seen only second-hand.

## Sources

1. https://grafana.com/pricing/
2. https://grafana.com/docs/grafana-cloud/send-data/otlp/send-data-otlp/
3. https://www.honeycomb.io/pricing
4. https://info.honeycomb.io/free-plan-signup
5. https://docs.honeycomb.io/send-data/opentelemetry/collector
6. https://newrelic.com/pricing
7. https://docs.newrelic.com/docs/opentelemetry/best-practices/opentelemetry-otlp/
8. https://axiom.co/pricing
9. https://axiom.co/docs/reference/limits
10. https://axiom.co/docs/send-data/opentelemetry
11. https://sentry.io/pricing/
12. https://docs.sentry.io/concepts/otlp/ and https://docs.sentry.io/concepts/otlp/direct/traces/
13. https://pydantic.dev/pricing
14. https://pydantic.dev/docs/logfire/guides/alternative-clients/
15. https://betterstack.com/pricing
16. https://betterstack.com/docs/logs/open-telemetry/
17. https://www.hyperdx.io/pricing
18. https://uptrace.dev/pricing
19. https://sentry.io/changelog/otlp-ingestion-endpoints-for-traces-and-logs-are-in-open-beta
20. https://crates.io (API `/api/v1/crates/<name>`, queried 2026-11-16)
21. https://clickhouse.com/blog/introducing-managed-clickstack-beta
22. https://signoz.io/pricing/
23. https://openobserve.ai/pricing/
24. https://www.dash0.com/pricing
25. https://www.elastic.co/pricing/serverless-observability
26. https://www.datadoghq.com/pricing/
27. https://last9.io/blog/dynatrace-pricing/
28. https://docs.lightstep.com/changelog/eol-notice
29. https://baselime.io
30. https://blog.cloudflare.com/cloudflare-acquires-baselime-expands-observability-capabilities/
31. https://developers.cloudflare.com/workers/observability/logs/workers-logs/
32. https://developers.cloudflare.com/workers/observability/exporting-opentelemetry-data/
33. https://cloud.google.com/products/observability/pricing
34. https://docs.cloud.google.com/stackdriver/docs/reference/telemetry/overview
35. https://aws.amazon.com/cloudwatch/pricing/
36. https://www.jaegertracing.io/docs/latest/getting-started/
37. https://jaegertracing.io/download/
38. https://github.com/grafana/docker-otel-lgtm
39. https://aspire.dev/dashboard/standalone/
40. https://github.com/CtrlSpice/otel-desktop-viewer
41. https://signoz.io/docs/install/docker/
42. https://openobserve.ai/docs/getting-started/
43. https://uptrace.dev/get/hosted/install
44. https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/fileexporter/README.md
45. https://docs.oracle.com/en-us/iaas/Content/FreeTier/resourceref.htm
46. https://docs.rs/opentelemetry-stdout/latest/opentelemetry_stdout/
47. https://docs.cloud.google.com/logging/docs/structured-logging
48. https://github.com/open-telemetry/opentelemetry-rust/blob/main/README.md
49. https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/CHANGELOG.md
50. https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/traces.md
51. https://github.com/open-telemetry/opentelemetry-rust/releases
52. https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/README.md
53. https://github.com/open-telemetry/opentelemetry-rust/issues/3190
54. https://github.com/open-telemetry/opentelemetry-rust/issues/1689
55. https://github.com/open-telemetry/opentelemetry-rust/issues/1571
56. https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/logs.md
57. https://docs.rs/init-tracing-opentelemetry/latest/init_tracing_opentelemetry/
58. https://docs.rs/axum-tracing-opentelemetry/latest/axum_tracing_opentelemetry/
59. https://docs.rs/tower-http/latest/tower_http/trace/index.html
60. https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/Cargo.toml
61. https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html
62. https://opentelemetry.io/docs/languages/js/exporters/
63. https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.SimpleSpanProcessor.html
64. https://docs.cloud.google.com/run/docs/configuring/billing-settings
65. https://docs.cloud.google.com/run/docs/tips/general
66. https://docs.cloud.google.com/run/docs/container-contract
67. https://cloud.google.com/run/pricing
68. https://docs.cloud.google.com/run/docs/tutorials/custom-metrics-opentelemetry-sidecar
69. https://render.com/docs/free
70. https://github.com/open-telemetry/opentelemetry-lambda/blob/main/collector/processor/decoupleprocessor/README.md
71. https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/enum.Sampler.html
72. https://opentelemetry.io/docs/languages/js/getting-started/browser/ and https://github.com/open-telemetry/opentelemetry-js/blob/main/README.md
73. https://www.npmjs.com/package/@opentelemetry/instrumentation-fetch (0.301.2 tarball, `dist/fetch.mjs`)
74. https://developer.mozilla.org/en-US/docs/Glossary/CORS-safelisted_request_header
75. https://docs.honeycomb.io/send-data/javascript-browser/
76. https://pydantic.dev/docs/logfire/integrations/javascript/browser/
77. https://grafana.com/docs/grafana-cloud/monitor-applications/frontend-observability/quickstart/javascript/
78. https://opentelemetry.io/docs/security/handling-sensitive-data/
79. https://docs.rs/tracing-test/latest/tracing_test/
80. https://github.com/tokio-rs/console
81. https://docs.rs/tracing-error/latest/tracing_error/
82. https://github.com/metrics-rs/metrics/blob/main/README.md
83. https://docs.cloud.google.com/run/docs/configuring/healthchecks
84. https://uptimerobot.com/pricing.md
85. https://uptimerobot.com/terms/
86. https://betterstack.com/uptime
87. https://community.render.com (support reply surfaced via web search; the thread itself was not opened)
