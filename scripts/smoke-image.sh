#!/usr/bin/env bash
# Starts an image and checks that it serves the SPA shell, liveness and a JSON 404 (spec rules 2.3.6, 2.4.1, 2.4.2).
set -euo pipefail

image=$1
port=18080

container=$(docker run --detach --rm --publish "$port:8080" "$image")
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
expect "a page path serves the shell" "$(curl --silent --include "http://localhost:$port/ballot")" "text/html"
expect "the shell boots the app" "$(curl --silent "http://localhost:$port/")" "_app/immutable"
expect "an unknown API path is a JSON 404" "$(curl --silent "http://localhost:$port/api/nope")" '"code":"not_found"'
expect "the container runs as non-root" "$(docker inspect --format '{{.Config.User}}' "$container")" "nonroot"
