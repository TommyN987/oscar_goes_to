---
paths:
  - "justfile"
  - "Dockerfile"
  - "compose.yaml"
  - "scripts/**"
  - ".github/**"
  - "docs/deploy.md"
---

# Operations rules (spec package 2)

- Every CI check is a `just` recipe, and a workflow only calls recipes, so `just check` and CI cannot drift apart.
- The Cloud Run service's settings live in the `deploy` recipe. A deploy never changes who may call the service.
- `scripts/bootstrap-gcp.sh` is the record of everything created in Google Cloud. A new resource, role or secret goes into that script, which must stay safe to re-run; Tommy runs it.
- Runtime secrets live in Secret Manager and reach the service through `--set-secrets`. No secret value enters the repo, a workflow file or a log.
- No workflow uses `pull_request_target`. The deploy job is the only one with `id-token: write`.
- A workflow pins every action GitHub does not own to a full commit SHA, with the version in a comment: `uses: owner/action@<sha> # v1.2.3`.
  SonarQube fails the pull request otherwise (`githubactions:S7637`). Dependabot moves the pins.
- The Dockerfile uses no BuildKit-only syntax, because Tommy's Docker has no `buildx`.
- Shell scripts are bash with `set -euo pipefail`, long flags, and a header comment that says what the script guarantees.
- `docs/deploy.md` describes how deploys work now. Update it in the same change as the recipe or script it describes.
