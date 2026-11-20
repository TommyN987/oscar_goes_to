#!/usr/bin/env bash
# Fails if a color literal appears in the frontend outside the one theme (spec rule 4.7.1).
# It finds hex colors and the functional notations rgb(), hsl(), hwb(), lab(), lch(), oklab() and oklch(),
# in every file under frontend/src and frontend/static, Tailwind arbitrary values such as bg-[#fff] included.
# It cannot find named colors such as `red`; Tailwind's own palette is switched off in the theme instead.
set -euo pipefail

readonly THEME="frontend/src/routes/layout.css"
# Generated from openapi.json, where a `#/components/...` reference is not a color.
readonly GENERATED_CLIENT_TYPES="schema.d.ts"

# A hex color has 3, 4, 6 or 8 digits. The look-behind skips character references such as &#169;.
readonly HEX_COLOR='(?<![&\w])#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b'
readonly COLOR_FUNCTION='\b(rgba?|hsla?|hwb|lab|lch|oklab|oklch)\('

cd "$(dirname "$0")/.."

# grep exits 0 when it finds a literal, 1 when it finds none, and higher when it could not search.
found=0
literals=$(grep --recursive --line-number --perl-regexp "$HEX_COLOR|$COLOR_FUNCTION" \
    --exclude="$(basename "$THEME")" --exclude="$GENERATED_CLIENT_TYPES" \
    frontend/src frontend/static) || found=$?

case "$found" in
    0)
        echo "Every color lives in $THEME (spec rule 4.7.1). Use a theme color instead of these literals:"
        echo "$literals"
        exit 1
        ;;
    1) ;;
    *)
        echo "The search for color literals failed."
        exit "$found"
        ;;
esac
