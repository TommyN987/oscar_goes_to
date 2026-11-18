#!/usr/bin/env bash
# Prints `frozen=true` if DATE (YYYY-MM-DD) falls in ceremony week, when nothing deploys automatically
# (spec rule 2.5.7), and `frozen=false` otherwise.
set -euo pipefail

date=$1
readonly FREEZE_FIRST_DAY="2027-03-08"
readonly FREEZE_LAST_DAY="2027-03-15"

if [[ ! "$date" < "$FREEZE_FIRST_DAY" && ! "$date" > "$FREEZE_LAST_DAY" ]]; then
    echo "frozen=true"
else
    echo "frozen=false"
fi
