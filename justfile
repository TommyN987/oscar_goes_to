# Every recipe CI runs is here, so `just check` and CI cannot drift apart (spec rule 2.2.4).

# The local backend port. The Vite dev proxy in frontend/vite.config.ts points at it.
backend_port := "8080"

# The local Postgres that `just up` starts, unless the environment names another database.
export DATABASE_URL := env("DATABASE_URL", "postgres://gold_ballot:gold_ballot@localhost:5432/gold_ballot")

# Every build checks its SQL against the metadata in .sqlx and never against a database,
# as the image build must once the binary holds a query.
export SQLX_OFFLINE := "true"

# Where the local Grafana stack that `just up` starts receives OTLP over HTTP.
local_otlp_endpoint := "http://localhost:4318"
image := "gold-ballot"

# Cloud Run. The Google Cloud project is whichever one gcloud is configured for.
service := "gold-ballot"
region := "europe-west1"
runtime_account := "gold-ballot-run"
database_url_secret := "database-url"
otlp_endpoint_secret := "otlp-endpoint"
otlp_headers_secret := "otlp-headers"

# List the recipes.
default:
    @just --list

# Start local Postgres and the local Grafana stack; Grafana is at http://localhost:3000.
up:
    docker compose up --detach --wait

# Start only local Postgres, which is all the tests need.
postgres:
    docker compose up --detach --wait postgres

# Stop the local services and discard their data.
down:
    docker compose down

# Run backend and frontend with reload; open http://localhost:5173.
[parallel]
dev: dev-backend dev-frontend

# Run the API on its own, restarting on change. It exports telemetry to the local Grafana stack,
# and it is built with fake identity, so sign-in needs no Google (spec rule 2.2.3).
dev-backend: up
    PORT={{ backend_port }} OTEL_EXPORTER_OTLP_ENDPOINT={{ local_otlp_endpoint }} cargo watch --watch crates --exec 'run --package adapters --features fake-identity --bin gold-ballot'

# Run the Vite dev server, which proxies /api to the backend.
dev-frontend:
    cd frontend && pnpm install && pnpm run dev

# Run every check CI runs.
check: check-rust check-frontend check-contract check-supply-chain

# Format, lints as errors, tests against local Postgres, the architecture rules, the rule ledger and the sqlx metadata.
# Lints and tests run with every feature, so the fake identity adapter is covered; `smoke-image` probes the build without it.
check-rust: postgres && check-sqlx
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    cargo nextest run --workspace --all-features --locked
    scripts/check-architecture.sh
    scripts/check-rule-ledger.sh

# Fail if .sqlx differs from what the queries and the migrations produce (spec rule 2.3.1).
# The sqlx CLI only warns about a file that no query produces any more; the second line fails on one.
check-sqlx: migrate-local
    cargo sqlx prepare --no-dotenv --workspace --check -- --all-targets --all-features
    @[ "$(ls .sqlx)" = "$(ls "${CARGO_TARGET_DIR:-target}/sqlx-prepare-check")" ] || { echo ".sqlx holds metadata of a query that is gone; run \`just prepare-sqlx\`"; exit 1; }

# Regenerate .sqlx after changing a query or a migration.
prepare-sqlx: migrate-local
    cargo sqlx prepare --no-dotenv --workspace -- --all-targets --all-features

# Bring the local database to this checkout's schema, which the two recipes above read.
# `--no-dotenv` matters: without it the sqlx CLI reads .env and echoes a line it cannot parse.
[private]
migrate-local: postgres
    cargo sqlx migrate run --no-dotenv --source crates/adapters/migrations

# Lint, the one-theme rule, type-check, component tests and the static build.
check-frontend:
    cd frontend && pnpm install --frozen-lockfile
    cd frontend && pnpm run lint
    scripts/check-color-literals.sh
    cd frontend && pnpm run check
    cd frontend && pnpm run test
    cd frontend && pnpm run build

# Fail if openapi.json or the TypeScript client types differ from what the code generates.
check-contract:
    cargo run --quiet --locked --bin gold-ballot -- openapi | diff - openapi.json
    cd frontend && pnpm install --frozen-lockfile
    cd frontend && pnpm exec openapi-typescript ../openapi.json | diff - src/lib/api/schema.d.ts

# Fail on a Rust or npm dependency with a known advisory or a license off the allow-lists. Needs the network.
check-supply-chain:
    cargo deny --locked check advisories licenses
    cd frontend && pnpm install --frozen-lockfile
    cd frontend && pnpm audit
    scripts/check-npm-licenses.sh

# Regenerate openapi.json and the TypeScript client types after changing a handler.
generate:
    cargo run --quiet --bin gold-ballot -- openapi > openapi.json
    cd frontend && pnpm run generate:api

# Build the release image.
image:
    docker build --tag {{ image }} .

# Start the release image against local Postgres and probe what it serves.
smoke-image: image postgres
    scripts/smoke-image.sh {{ image }}

# Push the image built by `just image` to a registry reference. Docker must already be logged in to that registry.
push-image image_ref:
    docker tag {{ image }} {{ image_ref }}
    docker push {{ image_ref }}

# Deploy an image that is already in the registry, and send it all traffic. It never changes who may call the service.
deploy image_ref:
    gcloud run deploy {{ service }} \
        --image={{ image_ref }} \
        --region={{ region }} \
        --service-account={{ runtime_account }}@$(gcloud config get-value project).iam.gserviceaccount.com \
        --execution-environment=gen1 \
        --memory=256Mi \
        --max-instances=1 \
        --min-instances=0 \
        --cpu-throttling \
        --set-secrets=DATABASE_URL={{ database_url_secret }}:latest,OTEL_EXPORTER_OTLP_ENDPOINT={{ otlp_endpoint_secret }}:latest,OTEL_EXPORTER_OTLP_HEADERS={{ otlp_headers_secret }}:latest \
        --quiet
    gcloud run services update-traffic {{ service }} --region={{ region }} --to-latest --quiet

# Send all traffic to the revision created before the one now serving.
rollback:
    scripts/rollback.sh {{ service }} {{ region }}
