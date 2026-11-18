# The Gold Ballot

An Oscar prediction game: Rust backend (axum), SvelteKit static SPA, one container.
The specification in `docs/spec/` is authoritative; `docs/design.md` explains the choices.
Rules are cited by number, such as "spec rule 2.4.2", in code comments and test names.

## Commands

- `just dev` runs backend and frontend with reload.
- `just check` runs every CI check. Run it before calling work done.
- `just generate` regenerates `openapi.json` and `frontend/src/lib/api/schema.d.ts` after a handler changes.

## Backend rules (design sections 11 and 13)

- Three crates, and dependencies point inward only: `adapters` → `application` → `domain`.
- `domain` holds types and pure functions. It has no IO, async runtime or serialization dependency.
  `scripts/check-crate-boundaries.sh` enforces this with an allow-list.
- `application` holds use cases and ports. It depends on no HTTP or SQL crate.
- A port exists only where a test needs a fake, a decision was deferred, or the dependency would otherwise point outward.
  The ports are clock, identity provider, results source and storage.
- Authorization lives in the use cases, through an actor value.
- Wire types belong to `adapters`. The domain carries no serialization or OpenAPI annotations.
- Scores are derived on every read and never stored.
- Use-case tests run against real Postgres. There is no in-memory storage fake.
- Handlers stay thin. Every API error uses the one JSON shape in `crates/adapters/src/http/error.rs`.
- OpenAPI is code-first: handlers carry `#[utoipa::path]`, and the committed `openapi.json` must match them.
- Migrations are forward-only and expand before they contract.

## Frontend rules (design section 14)

- The client holds no domain rules. The server says what is editable, what the scores are, and what a player may see.
- Server state comes through SvelteKit load functions.
- Components live beside their route. `src/lib` holds only `api`, `session` and `ui`.
- Only `src/lib/api` holds the API client instance; everything else imports its operation functions.
  Generated types may be imported anywhere.
- Pages fetch and own state. Leaf components take props and emit events.
- Tailwind and daisyUI only; a scoped `<style>` block needs a stated reason.
  Native elements come first, and every colour lives in the one theme in `src/routes/layout.css`.
- Leaf components are tested in Vitest browser mode with props only. Pages are tested by Playwright against the real container. There are no API mocks.
- SvelteKit 3 is new: imports use `#lib/...` with the file extension, and configuration lives in `vite.config.ts`.
  Check the Svelte MCP server's documentation before writing frontend code.
- TypeScript stays on 6.x until `svelte-check` supports 7.
