# Deploy

How The Gold Ballot reaches Cloud Run, how to roll it back, and what was measured at M0.
The rules are in [package 2 of the specification](spec/2-foundation.md), sections 2.5 and 2.8.

## How a deploy happens

A merge to `main` runs `.github/workflows/deploy.yml`.

1. It builds the release image and smoke-tests that image (`just smoke-image`).
2. It authenticates to Google Cloud without a stored key, through workload identity federation.
3. It pushes the image to Artifact Registry (`just push-image`).
4. It deploys the image and sends it all traffic (`just deploy`).

The service's settings live in the `deploy` recipe of the `justfile`:
region `europe-west1`, at most one instance, scale to zero, request-based billing, 256 MiB, first-generation environment.
The recipe also mounts three secrets as environment variables, so the service definition holds no secret value:
`database-url` as `DATABASE_URL`, and `otlp-endpoint` and `otlp-headers` as `OTEL_EXPORTER_OTLP_ENDPOINT` and `OTEL_EXPORTER_OTLP_HEADERS`.

Each new revision connects to Neon and applies pending migrations before it serves.
If that fails, the revision never becomes ready and the previous one keeps serving.

A deploy never changes who may call the service.

## Telemetry

The service exports traces, logs and metrics to Grafana Cloud over OTLP, in batches from background threads.
Cloud Run pauses background work between requests, so a lone request's trace appears with the next request,
or when the instance stops, which is at most 15 minutes later.
On SIGTERM the server finishes the requests in flight and exports what is still in memory.

To see telemetry at once while debugging, make the service export before each response, at the price of one export per request:

```sh
gcloud run services update gold-ballot --region=europe-west1 --update-env-vars=TELEMETRY_FLUSH_EACH_REQUEST=true
```

The setting survives deploys; remove it with `--remove-env-vars=TELEMETRY_FLUSH_EACH_REQUEST`.

A failed export never fails a request. It prints an error with the target `opentelemetry_sdk` in the service's logs in Google Cloud.

## Deploy freeze

From 8 to 15 March 2027 a merge does not deploy (`scripts/deploy-freeze.sh`, in Central European Time).
During the freeze, start the Deploy workflow by hand from the Actions tab; a manual run always deploys.

## Roll back

```sh
just rollback
```

This sends all traffic to the revision created before the one now serving.
It moves code, not schema: migrations are forward-only, so the older code must tolerate the newer schema.
The next deploy sends traffic to the newest revision again.

It needs `gcloud` logged in as a project owner, with the project as gcloud's configured project.

## One-time setup

`scripts/bootstrap-gcp.sh` creates everything on the Google Cloud side and can be re-run safely:

- the APIs: Cloud Run, Artifact Registry, IAM, IAM Credentials, Security Token Service, Secret Manager
- an image repository that keeps the newest five images, to stay inside the free 0.5 GB
- `gold-ballot-run`, the account the app runs as; its only permission is to read the secrets below
- the secret `database-url`, stored in `europe-west1`: Neon's direct connection string, read from `NEON_DATABASE_URL` in the environment or in `.env`.
  The app uses the direct string, not the pooled one: its pool holds at most five connections, and migrations need a direct connection.
- the secrets `otlp-endpoint` and `otlp-headers`, read from `GRAFANA_OTLP_ENDPOINT` and `GRAFANA_OTLP_HEADERS`.
  Grafana Cloud shows both values for a stack under Connections, OpenTelemetry (OTLP):
  the endpoint looks like `https://otlp-gateway-<region>.grafana.net/otlp`, and the header like `Authorization=Basic <token>`.

A secret that already exists is left alone. To change one, add a version: `gcloud secrets versions add <name> --data-file=-`.
- `github-deploy`, the account GitHub deploys as: it may push images, deploy revisions, and run them as `gold-ballot-run`
- an identity pool that lets only workflows on `main` of this repository act as `github-deploy`

It prints three values, stored as GitHub Actions variables:
`GCP_PROJECT_ID`, `GCP_WORKLOAD_IDENTITY_PROVIDER` and `GCP_DEPLOY_SERVICE_ACCOUNT`.

The first revision was deployed by hand on 2026-11-18, with `just smoke-image`, `just push-image` and `just deploy`.

### Public access

A new Cloud Run service is private: it answers only callers that Google has authenticated and that hold the invoker role.
The project's organization forbids granting that role to everyone, so the owner opens the service by turning the invoker check off, once:

```sh
gcloud run services update gold-ballot --region=europe-west1 --no-invoker-iam-check
```

Done on 2026-11-18; the service has answered the public since.

## Repository and account settings (rules 2.1.4, 2.1.5, 2.5.4 and 2.5.6)

Read from the GitHub API on 2026-11-20:

- `main` takes changes only through pull requests, with no approval required.
  The checks `rust`, `frontend`, `contract` and `image` must pass on an up-to-date branch.
  The rule binds administrators too, and force pushes and deletion are off.
- Secret scanning and push protection are on.

Reported by Tommy on 2026-11-20:

- A push straight to `main` was rejected.
- The Neon project is in AWS Europe Central 1 (Frankfurt).
- The €1 billing alert exists.

The `supply-chain` check was added to CI after that reading and is not among the required checks yet.

## M0 measurements (rule 2.8.3)

| What | Value | Measured |
|---|---|---|
| CI duration, cold caches | 53 s wall clock; the slowest job, `frontend`, took 48 s | 2026-11-18, the first run on pull request 1 |
| Rollback | 15 s for `just rollback` | 2026-11-18, rehearsed from revision 2 to revision 1 |
| Cold start, no database | 0.77 s for the first request after 17 idle minutes, of which Cloud Run logged 0.17 s in the service; warm requests took 0.17–0.27 s | 2026-11-18, from Tommy's machine, with an authenticated request |
| Cold start, waking Neon | 1.26 s for the first readiness request after 17 idle minutes: the container starts, connects to a suspended Neon, checks migrations and runs a query; warm requests took 0.15–0.22 s | 2026-11-18 23:06 UTC, from Tommy's machine, revision 5, before telemetry was added |
| Cold start, with telemetry | 1.88 s for the same request on revision 6, which also starts the telemetry exporters; warm requests took 0.09–0.20 s | 2026-11-19 00:25 UTC, from Tommy's machine |
| CI duration, full M0 | 2 min 36 s on the telemetry pull request; 6 min 31 s on the first `main` run after it, with a cold dependency cache | 2026-11-18 |

Each cold start is one sample, so the differences between them include the variation in Neon's wake-up time.
Rule 6.4.6 allows 5 s, so no keep-warm ping is adopted.
