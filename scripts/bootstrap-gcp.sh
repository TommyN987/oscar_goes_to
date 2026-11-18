#!/usr/bin/env bash
# One-time Google Cloud setup for keyless deploys from GitHub Actions (spec rules 2.5.1 and 2.5.2).
# Run it as an owner of the project gcloud is configured for. Re-running it is safe.
set -euo pipefail

readonly GITHUB_REPOSITORY="TommyN987/oscar_goes_to"
readonly REGION="europe-west1"
readonly IMAGE_REPOSITORY="gold-ballot"
readonly IMAGES_KEPT=5
readonly RUNTIME_ACCOUNT="gold-ballot-run"
readonly DEPLOY_ACCOUNT="github-deploy"
readonly IDENTITY_POOL="github"
readonly IDENTITY_PROVIDER="oscar-goes-to"

project=$(gcloud config get-value project)
project_number=$(gcloud projects describe "$project" --format='value(projectNumber)')
runtime_email="$RUNTIME_ACCOUNT@$project.iam.gserviceaccount.com"
deploy_email="$DEPLOY_ACCOUNT@$project.iam.gserviceaccount.com"
pool_name="projects/$project_number/locations/global/workloadIdentityPools/$IDENTITY_POOL"

exists() { "$@" >/dev/null 2>&1; }

# A new service account takes a few seconds to become visible to IAM, so a binding is tried more than once.
bind() {
    local attempt
    for attempt in 1 2 3 4 5 6; do
        "$@" >/dev/null 2>&1 && return
        sleep 5
    done
    "$@" >/dev/null
}

echo "Enabling the APIs."
gcloud services enable \
    run.googleapis.com \
    artifactregistry.googleapis.com \
    iam.googleapis.com \
    iamcredentials.googleapis.com \
    sts.googleapis.com

echo "Creating the image repository, which keeps only the newest $IMAGES_KEPT images to stay in the free tier."
exists gcloud artifacts repositories describe "$IMAGE_REPOSITORY" --location="$REGION" ||
    gcloud artifacts repositories create "$IMAGE_REPOSITORY" \
        --repository-format=docker \
        --location="$REGION" \
        --description="Release images of The Gold Ballot"
cleanup_policy=$(mktemp)
trap 'rm -f "$cleanup_policy"' EXIT
cat >"$cleanup_policy" <<JSON
[
  { "name": "delete-every-image", "action": { "type": "Delete" }, "condition": { "tagState": "any" } },
  { "name": "keep-the-newest", "action": { "type": "Keep" }, "mostRecentVersions": { "keepCount": $IMAGES_KEPT } }
]
JSON
gcloud artifacts repositories set-cleanup-policies "$IMAGE_REPOSITORY" \
    --location="$REGION" --policy="$cleanup_policy" --no-dry-run

echo "Creating the service accounts: one the app runs as, one GitHub deploys as."
exists gcloud iam service-accounts describe "$runtime_email" ||
    gcloud iam service-accounts create "$RUNTIME_ACCOUNT" --display-name="The Gold Ballot at run time"
exists gcloud iam service-accounts describe "$deploy_email" ||
    gcloud iam service-accounts create "$DEPLOY_ACCOUNT" --display-name="GitHub Actions deploy"

echo "Letting the deploy account push images, deploy revisions and run them as the runtime account."
bind gcloud artifacts repositories add-iam-policy-binding "$IMAGE_REPOSITORY" --location="$REGION" \
    --member="serviceAccount:$deploy_email" --role=roles/artifactregistry.writer
bind gcloud projects add-iam-policy-binding "$project" \
    --member="serviceAccount:$deploy_email" --role=roles/run.developer --condition=None
bind gcloud iam service-accounts add-iam-policy-binding "$runtime_email" \
    --member="serviceAccount:$deploy_email" --role=roles/iam.serviceAccountUser

echo "Letting workflows on main of $GITHUB_REPOSITORY, and nothing else, act as the deploy account."
exists gcloud iam workload-identity-pools describe "$IDENTITY_POOL" --location=global ||
    gcloud iam workload-identity-pools create "$IDENTITY_POOL" --location=global \
        --display-name="GitHub Actions"
exists gcloud iam workload-identity-pools providers describe "$IDENTITY_PROVIDER" \
    --location=global --workload-identity-pool="$IDENTITY_POOL" ||
    gcloud iam workload-identity-pools providers create-oidc "$IDENTITY_PROVIDER" \
        --location=global --workload-identity-pool="$IDENTITY_POOL" \
        --display-name="$IDENTITY_PROVIDER" \
        --issuer-uri="https://token.actions.githubusercontent.com" \
        --attribute-mapping="google.subject=assertion.sub,attribute.repository=assertion.repository,attribute.ref=assertion.ref" \
        --attribute-condition="assertion.repository == '$GITHUB_REPOSITORY' && assertion.ref == 'refs/heads/main'"
bind gcloud iam service-accounts add-iam-policy-binding "$deploy_email" \
    --member="principalSet://iam.googleapis.com/$pool_name/attribute.repository/$GITHUB_REPOSITORY" \
    --role=roles/iam.workloadIdentityUser

cat <<VARIABLES

Done. A deployed service stays private until its owner makes it public; deploys never change that.
The deploy workflow reads these as GitHub Actions variables:

  GCP_PROJECT_ID=$project
  GCP_WORKLOAD_IDENTITY_PROVIDER=$pool_name/providers/$IDENTITY_PROVIDER
  GCP_DEPLOY_SERVICE_ACCOUNT=$deploy_email
VARIABLES
