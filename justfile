# Every recipe CI runs is here, so `just check` and CI cannot drift apart (spec rule 2.2.4).

# The local backend port. The Vite dev proxy in frontend/vite.config.ts points at it.
backend_port := "8080"

# The local Postgres that `just up` starts, unless the environment names another database.
export DATABASE_URL := env("DATABASE_URL", "postgres://gold_ballot:gold_ballot@localhost:5432/gold_ballot")

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

# Run the API on its own, restarting on change. It exports telemetry to the local Grafana stack.
dev-backend: up
    PORT={{ backend_port }} OTEL_EXPORTER_OTLP_ENDPOINT={{ local_otlp_endpoint }} cargo watch --watch crates --exec 'run --bin gold-ballot'

# Run the Vite dev server, which proxies /api to the backend.
dev-frontend:
    cd frontend && pnpm install && pnpm run dev

# Run every check CI runs.
check: check-rust check-frontend check-contract

# Format, lints as errors, tests against local Postgres, and the architecture rules.
check-rust: postgres
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo nextest run --workspace --locked
    scripts/check-architecture.sh

# Lint, type-check, component tests and the static build.
check-frontend:
    cd frontend && pnpm install --frozen-lockfile
    cd frontend && pnpm run lint
    cd frontend && pnpm run check
    cd frontend && pnpm run test
    cd frontend && pnpm run build

# Fail if openapi.json or the TypeScript client types differ from what the code generates.
check-contract:
    cargo run --quiet --locked --bin gold-ballot -- openapi | diff - openapi.json
    cd frontend && pnpm install --frozen-lockfile
    cd frontend && pnpm exec openapi-typescript ../openapi.json | diff - src/lib/api/schema.d.ts

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
