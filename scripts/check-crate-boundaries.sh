#!/usr/bin/env bash
# Asserts the workspace layout and the dependency rules of the hexagonal crates (spec rules 2.1.1, 2.1.2).
set -euo pipefail

failed=0

# Every crate a workspace member depends on at run time, directly or transitively.
dependencies_of() {
    cargo tree --locked --package "$1" --edges normal --prefix none --format '{p}' |
        awk '{ print $1 }' | sort -u | grep -vx "$1" || true
}

workspace_members=$(cargo tree --locked --workspace --depth 0 --prefix none --format '{p}' | awk 'NF { print $1 }' | sort -u | xargs)
if [[ "$workspace_members" != "adapters application domain" ]]; then
    echo "The workspace must hold exactly the crates domain, application and adapters; found: $workspace_members"
    failed=1
fi

# domain: no IO, async runtime or serialization. It is an allow-list, so a new dependency is a decision.
domain_allowed=""
for dependency in $(dependencies_of domain); do
    if ! grep -qx "$dependency" <<<"$domain_allowed"; then
        echo "domain must not depend on '$dependency'. Allow it in $0 only if it does no IO, async or serialization."
        failed=1
    fi
done

# application: no HTTP and no SQL.
application_forbidden="axum axum-core hyper http tower tower-http reqwest sqlx sqlx-core tokio-postgres postgres diesel"
application_dependencies=$(dependencies_of application)
for dependency in $application_forbidden; do
    if grep -qx "$dependency" <<<"$application_dependencies"; then
        echo "application must not depend on the HTTP or SQL crate '$dependency'."
        failed=1
    fi
done

exit "$failed"
