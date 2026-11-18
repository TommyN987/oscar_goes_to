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

A deploy never changes who may call the service.

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

- the APIs: Cloud Run, Artifact Registry, IAM, IAM Credentials, Security Token Service
- an image repository that keeps the newest five images, to stay inside the free 0.5 GB
- `gold-ballot-run`, the account the app runs as, which holds no roles
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

Status on 2026-11-18: not yet run, so the service answers 403 to the public.

## M0 measurements (rule 2.8.3)

| What | Value | Measured |
|---|---|---|
| CI duration, cold caches | 53 s wall clock; the slowest job, `frontend`, took 48 s | 2026-11-18, the first run on pull request 1 |
| Rollback | 15 s for `just rollback` | 2026-11-18, rehearsed from revision 2 to revision 1 |
| Cold start | 0.77 s for the first request after 17 idle minutes, of which Cloud Run logged 0.17 s in the service; warm requests took 0.17–0.27 s and logged 0.002 s | 2026-11-18, from Tommy's machine, with an authenticated request |

Rule 6.4.6 allows 5 s, so no keep-warm ping is adopted.
The skeleton has no database yet; the number is measured again once a request also wakes Neon.
