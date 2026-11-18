# Every recipe CI runs is here, so `just check` and CI cannot drift apart (spec rule 2.2.4).

# The local backend port. The Vite dev proxy in frontend/vite.config.ts points at it.
backend_port := "8080"

# The local Postgres that `just up` starts, unless the environment names another database.
export DATABASE_URL := env("DATABASE_URL", "postgres://gold_ballot:gold_ballot@localhost:5432/gold_ballot")
image := "gold-ballot"

# Cloud Run. The Google Cloud project is whichever one gcloud is configured for.
service := "gold-ballot"
region := "europe-west1"
runtime_account := "gold-ballot-run"
database_url_secret := "database-url"

# List the recipes.
default:
    @just --list

# Start local Postgres and wait until it accepts connections.
up:
    docker compose up --detach --wait

# Stop local Postgres and discard its data.
down:
    docker compose down

# Run backend and frontend with reload; open http://localhost:5173.
[parallel]
dev: dev-backend dev-frontend

# Run the API on its own, restarting on change.
dev-backend: up
    PORT={{ backend_port }} cargo watch --watch crates --exec 'run --bin gold-ballot'

# Run the Vite dev server, which proxies /api to the backend.
dev-frontend:
    cd frontend && pnpm install && pnpm run dev

# Run every check CI runs.
check: check-rust check-frontend check-contract

# Format, lints as errors, tests against local Postgres, and the crate dependency rules.
check-rust: up
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    scripts/check-crate-boundaries.sh

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
smoke-image: image up
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
        --set-secrets=DATABASE_URL={{ database_url_secret }}:latest \
        --quiet
    gcloud run services update-traffic {{ service }} --region={{ region }} --to-latest --quiet

# Send all traffic to the revision created before the one now serving.
rollback:
    scripts/rollback.sh {{ service }} {{ region }}
