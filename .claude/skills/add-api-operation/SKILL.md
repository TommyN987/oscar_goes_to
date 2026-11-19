---
name: add-api-operation
description: Add one HTTP API operation end to end in The Gold Ballot — use case, handler with OpenAPI annotation, regenerated contract, frontend API function and tests. Use when implementing an operation from spec rule 4.6.1.
argument-hint: "[operation, e.g. \"get my ballot\"]"
---

# Add an API operation

Add the operation: $ARGUMENTS

An operation exists only if spec rule 4.6.1 (`docs/spec/4-game-features.md`) lists it.
If it does not, stop and say so.

The health operations are the worked example; read them before writing:
`crates/adapters/src/http/health.rs`, `crates/adapters/tests/http.rs`, `frontend/src/lib/api/health.ts`.

## 1. Find the rules

Grep the spec for the rules this operation must satisfy, and list their numbers before writing code.
Check package 6 as well: the error shape (6.4.1), input bounds (6.5.2), and what a response must not contain (6.5.5).

## 2. Domain and use case

- A rule goes into `crates/domain` as a pure function, with a test named after its rule number.
- The use case goes into `crates/application`. It takes an actor value and does the authorization itself.
- It reaches storage, the clock and identity only through ports.
- Give it a span: `#[tracing::instrument(skip_all)]`. Record IDs only, never an email, a name or a pick.

## 3. Storage

- A schema change is a new file in `crates/adapters/migrations`; never edit one that is on `main`.
- Implement the port in `crates/adapters/src/postgres.rs`, or a module beside it once that file grows.
- Give each storage function a span with `otel.kind = "client"`, like `is_reachable`.
- Test the use case against real Postgres with `#[sqlx::test]`.

## 4. Handler

- Put it in a module under `crates/adapters/src/http/`, with wire types of its own; domain types never derive `Serialize` or `ToSchema`.
- Annotate it with `#[utoipa::path]`: an `operation_id`, a `tag`, and every response including each failure as `body = ApiError`.
- A new failure needs a new `ErrorCode` variant and its status in `http/error.rs`.
- Register it with one `.routes(routes!(...))` line in `http/mod.rs`.
- Only health, sign-in start and sign-in callback are public (spec rule 4.6.2).
- Test it through the router in `crates/adapters/tests/`, including each failure response.

## 5. Contract

Run `just generate`. It rewrites `openapi.json` and `frontend/src/lib/api/schema.d.ts`; a hook denies editing either by hand.
Read the diff of `openapi.json`: it must show this operation and nothing else.

## 6. Frontend

- Add one function per operation to a module in `frontend/src/lib/api/`, shaped like `readLiveness`: it takes `fetch`, calls the client, and returns typed data or throws.
- Call it from a `+page.ts` or `+layout.ts` load function, never from a component.
- Nothing outside `src/lib/api` imports the client; ESLint fails otherwise.

## 7. Check

Run `just check`. Then confirm one new test fails when you break the code it guards.
Report which rule numbers now have a test, and which of the operation's rules are still open and why.
