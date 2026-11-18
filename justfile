# Every recipe CI runs is here, so `just check` and CI cannot drift apart (spec rule 2.2.4).

# The local backend port. The Vite dev proxy in frontend/vite.config.ts points at it.
backend_port := "8080"
image := "gold-ballot"

# List the recipes.
default:
    @just --list

# Run backend and frontend with reload; open http://localhost:5173.
[parallel]
dev: dev-backend dev-frontend

# Run the API on its own, restarting on change.
dev-backend:
    PORT={{ backend_port }} cargo watch --watch crates --exec 'run --bin gold-ballot'

# Run the Vite dev server, which proxies /api to the backend.
dev-frontend:
    cd frontend && pnpm install && pnpm run dev

# Run every check CI runs.
check: check-rust check-frontend check-contract

# Format, lints as errors, tests, and the crate dependency rules.
check-rust:
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

# Start the release image and probe what it serves.
smoke-image: image
    scripts/smoke-image.sh {{ image }}
