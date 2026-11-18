#!/usr/bin/env bash
# Starts an image against the Postgres of compose.yaml, which must be running, and checks that it serves
# the SPA shell, liveness, readiness and a JSON 404 (spec rules 2.3.6, 2.4.1, 2.4.2, 2.4.5, 2.7.7).
set -euo pipefail

image=$1
port=18080

# The caller names the compose database as the host sees it. The container joins the compose network,
# where the same Postgres answers under its service name instead of localhost.
: "${DATABASE_URL:?is not set; run this script through \`just smoke-image\`}"
database_url_in_compose_network=${DATABASE_URL/@localhost:/@postgres:}

# Nothing listens here, so every probe below also shows that a dead telemetry backend
# neither fails nor slows a request (spec rule 2.7.7).
unreachable_otlp_endpoint=http://localhost:1

container=$(docker run --detach --rm --publish "$port:8080" \
    --network gold-ballot_default \
    --env DATABASE_URL="$database_url_in_compose_network" \
    --env OTEL_EXPORTER_OTLP_ENDPOINT="$unreachable_otlp_endpoint" \
    "$image")
trap 'docker stop "$container" >/dev/null' EXIT

for _ in $(seq 1 50); do
    curl --silent --fail --output /dev/null "http://localhost:$port/api/health" && break
    sleep 0.1
done

expect() {
    local description=$1 actual=$2 expected=$3
    if [[ "$actual" != *"$expected"* ]]; then
        echo "FAILED: $description: expected '$expected' in '$actual'"
        exit 1
    fi
    echo "ok: $description"
}

expect "liveness answers ok" "$(curl --silent "http://localhost:$port/api/health")" '"status":"ok"'
expect "readiness answers once the database is reachable" \
    "$(curl --silent --output /dev/null --write-out '%{http_code}' "http://localhost:$port/api/health/ready")" "204"
expect "a page path serves the shell" "$(curl --silent --include "http://localhost:$port/ballot")" "text/html"
expect "the shell boots the app" "$(curl --silent "http://localhost:$port/")" "_app/immutable"
expect "an unknown API path is a JSON 404" "$(curl --silent "http://localhost:$port/api/nope")" '"code":"not_found"'
expect "the container runs as non-root" "$(docker inspect --format '{{.Config.User}}' "$container")" "nonroot"
