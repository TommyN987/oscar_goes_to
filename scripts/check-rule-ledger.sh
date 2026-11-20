#!/usr/bin/env bash
# Keeps the ledger of package-1 rules and their tests (spec rule 6.6.1).
# Every MUST rule in docs/spec/1-product-rules.md must have a test that carries its number, or a line in
# scripts/rule-ledger-pending.txt; it may not have both, so the pending list cannot go stale.
# A Rust test carries a number in its name, as `rule_1_2_5_...`; a frontend test in its title, as `rule 1.2.5`.
set -euo pipefail

cd "$(dirname "$0")/.."

readonly RULES="docs/spec/1-product-rules.md"
readonly PENDING="scripts/rule-ledger-pending.txt"

must_rules=$(grep --only-matching --perl-regexp '^\| \K[0-9]+(\.[0-9]+)+(?= \| MUST )' "$RULES")
pending_rules=$(grep --invert-match --extended-regexp '^(#|$)' "$PENDING" | cut --delimiter=' ' --fields=1 || true)

# Whether any test carries the rule's number. The character after the number keeps 1.2.1 apart from 1.2.10.
has_test() {
    local rule=$1
    grep --recursive --quiet --include='*.rs' "fn rule_${rule//./_}_" crates ||
        grep --recursive --quiet --extended-regexp --include='*.spec.ts' --include='*.test.ts' \
            "rule ${rule//./\\.}([^0-9.]|$)" frontend/src
}

failed=0
for rule in $must_rules; do
    pending=no
    grep --quiet --line-regexp --fixed-strings "$rule" <<<"$pending_rules" && pending=yes
    if has_test "$rule"; then
        if [[ $pending == yes ]]; then
            echo "Rule $rule has a test now; delete its line from $PENDING."
            failed=1
        fi
    elif [[ $pending == no ]]; then
        echo "Rule $rule has no test carrying its number and no line in $PENDING."
        failed=1
    fi
done

for rule in $pending_rules; do
    if ! grep --quiet --line-regexp --fixed-strings "$rule" <<<"$must_rules"; then
        echo "$PENDING names $rule, which is not a MUST rule in $RULES."
        failed=1
    fi
done

exit "$failed"
