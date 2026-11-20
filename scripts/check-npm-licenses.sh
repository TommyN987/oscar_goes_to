#!/usr/bin/env bash
# Fails if an npm dependency of the frontend carries a license that is not on the list below
# (spec rule 2.3.7). The dependencies must be installed: pnpm reads each license from the installed package.
set -euo pipefail

# Permissive licenses, each spelled as pnpm reports it.
# MPL-2.0 is allowed for lightningcss alone in practice: it compiles the CSS and is not part of the built SPA.
readonly ALLOWED_LICENSES=(
    "MIT"
    "Apache-2.0"
    "BSD-2-Clause"
    "BSD-3-Clause"
    "ISC"
    "BlueOak-1.0.0"
    "Python-2.0"
    "(MIT OR CC0-1.0)"
    "MPL-2.0"
)

cd "$(dirname "$0")/../frontend"

# Prints one line per license that is not allowed, with the packages that carry it.
unlisted=$(pnpm licenses list --json | node --eval '
    const allowed = new Set(process.argv.slice(1));
    const packagesByLicense = JSON.parse(require("node:fs").readFileSync(0, "utf8"));
    for (const [license, packages] of Object.entries(packagesByLicense)) {
        if (!allowed.has(license)) {
            console.log(`${license}: ${[...new Set(packages.map((dependency) => dependency.name))].join(", ")}`);
        }
    }
' "${ALLOWED_LICENSES[@]}")

if [[ -n "$unlisted" ]]; then
    echo "These npm dependencies carry a license that $0 does not allow:"
    echo "$unlisted"
    exit 1
fi
