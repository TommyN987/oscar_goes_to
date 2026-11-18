#!/usr/bin/env bash
# Sends all traffic to the revision created before the one now serving (spec rule 2.5.5).
# The schema does not roll back: migrations are forward-only, so the older code must tolerate the newer schema.
set -euo pipefail

service=$1
region=$2

serving=$(gcloud run services describe "$service" --region="$region" \
    --format='value(status.traffic[0].revisionName)')
previous=$(gcloud run revisions list --service="$service" --region="$region" \
    --sort-by='~metadata.creationTimestamp' --format='value(metadata.name)' |
    awk -v serving="$serving" 'found { print; exit } $0 == serving { found = 1 }')

if [[ -z "$previous" ]]; then
    echo "No revision is older than $serving, so there is nothing to roll back to."
    exit 1
fi

echo "Rolling back from $serving to $previous."
gcloud run services update-traffic "$service" --region="$region" --to-revisions="$previous=100" --quiet
