# The Gold Ballot

An Oscar prediction game. A Rust backend (axum, sqlx, Postgres) serves a JSON API and a SvelteKit static SPA from one container on Cloud Run.

## Where things are decided

- `docs/spec/` is authoritative. Its rules are cited by number, such as "spec rule 2.4.2", in comments and test names.
- `docs/plan.md` orders the work. Its items are cited as "P3.2", and each spec rule has one owning item.
- `docs/reviews/` holds the findings of each review. A plan item's Needs cites the findings deferred to it; read them before proposing the item.
- Read only the spec file for the package in hand, or grep for a rule number. `docs/design.md` explains the choices; `docs/research/` is history and is not needed to write code.
- Rules for backend, frontend and operations files are in `.claude/rules/` and load when you touch those files.

## Commands

- `just up` starts local Postgres and Grafana; `just dev` and `just check` start what they need themselves.
- `just dev` runs backend and frontend with reload.
- `just check` runs every CI check; `just smoke-image` builds and probes the release image.
- `just generate` regenerates `openapi.json` and the TypeScript client types after a handler changes.
- `just prepare-sqlx` regenerates `.sqlx` after a query or a migration changes.

## Working agreement

- **Git is Tommy's.** He creates branches, stages, commits, pushes and merges. Claude only reads git; a hook denies the rest. When work is ready, give him the exact commands.
- **So is everything outside the repo.** A change in Google Cloud, GitHub settings, Grafana or on his machine needs his go-ahead for that specific action.
- **Never print a secret.** Refer to entries in `.env` by name.
- **Verify, then claim.** Say what was run and what was not. End a slice with `/finish-slice`.
- **Build only what a spec rule asks for.** If a rule's done-condition cannot be met yet, say what blocks it; do not defer it silently.
- **Check current documentation** before using SvelteKit 3, sqlx 0.9 or the OpenTelemetry 0.33 crates in a way this repo has not used them yet. They postdate most training data.

## Keeping context small

- Send long command output to a file in the scratchpad and read only the lines you need.
- Delegate wide reading, such as a whole spec package or a full diff review, to a subagent and keep its conclusion.
- Prefer `grep` for a rule number or symbol over reading a whole document.

## Learning

Tommy is learning Svelte and SvelteKit, and OpenTelemetry with `tracing`, through this project.
When a slice first uses a concept from those areas, name it in one line in the hand-off.
`/explain-slice` gives the full walkthrough when he asks for it.
