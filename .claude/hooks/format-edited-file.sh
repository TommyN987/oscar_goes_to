#!/usr/bin/env bash
# PostToolUse hook for Edit and Write: formats the file that was just changed,
# so `just check` never fails on formatting alone. A formatter failure is ignored.
set -uo pipefail

file=$(jq -r '.tool_input.file_path // empty')
[[ -n "$file" && -f "$file" ]] || exit 0

project=${CLAUDE_PROJECT_DIR:-$(pwd)}

case "$file" in
"$project"/crates/*.rs)
    # rustfmt finds rustfmt.toml by itself; the edition comes from the workspace manifest.
    rustfmt --edition 2024 "$file" >/dev/null 2>&1
    ;;
"$project"/frontend/src/* | "$project"/frontend/*.ts | "$project"/frontend/*.js)
    # prettier skips what .prettierignore lists, such as the generated schema.
    (cd "$project/frontend" && pnpm exec prettier --write --ignore-unknown "$file" >/dev/null 2>&1)
    ;;
esac
exit 0
