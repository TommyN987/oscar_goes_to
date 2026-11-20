---
paths:
  - "crates/**"
  - "Cargo.toml"
---

# Backend rules (design sections 11 and 13)

## Layers

- Three crates, and dependencies point inward only: `adapters` → `application` → `domain`.
- `domain` holds types and pure functions. It has no IO, async runtime or serialization dependency.
  `scripts/check-architecture.sh` enforces this with an allow-list; adding a dependency to `domain` means editing that list on purpose.
- `application` holds use cases and ports. It depends on no HTTP or SQL crate.
- A port exists only where a test needs a fake, a decision was deferred, or the dependency would otherwise point outward.
  The ports are clock, identity provider, results source and storage.
- Authorization lives in the use cases, through an actor value.
- There is no unit-of-work abstraction: no use case writes two aggregates atomically.
- Wire types belong to `adapters`. The domain carries no serialization or OpenAPI annotations.
- Scores are derived on every read and never stored.

## HTTP

- Handlers stay thin: extract, call a use case, map the result.
- Every API error is an `ApiError` from `crates/adapters/src/http/error.rs`, with a stable `ErrorCode`. A code is never renamed.
- OpenAPI is code-first: each handler carries `#[utoipa::path]` and is registered once in `http/mod.rs`.
  After changing a handler or a wire type, run `just generate`; never edit `openapi.json` by hand.
- Use `/add-api-operation` for a new operation.
- User-facing text, such as an `ApiError` message a page may show, uses American spelling: "canceled", "favorite".
  The rule is about user-facing language only; comments may use British spelling.

## Postgres

- Migrations live in `crates/adapters/migrations`, run at startup, are forward-only, and expand before they contract.
  A migration on `main` is never edited; add a new one.
- The pool holds at most five connections.

## Telemetry

- Only `crates/adapters/src/telemetry.rs` names OpenTelemetry types; everything else uses `tracing` macros and the `Telemetry` handle.
- The OpenTelemetry crates are pinned and upgraded together, in one pull request.
- Telemetry carries route templates and player IDs only: no email, name, pick, cookie, token or raw URL.
- A use case gets a span with `#[tracing::instrument(skip_all)]`; a storage function gets one with `otel.kind = "client"`.

## Tests

- A test that pins a spec rule carries its number in its name: `rule_2_4_5_readiness_fails_without_the_database`.
- Rules are tested in `domain` with unit and property tests, without IO.
- Use cases are tested against real Postgres with `#[sqlx::test]`, a fake clock and a fake identity. There is no in-memory storage fake.
- The fake identity adapter sits behind a compile-time feature that the release image is built without, and CI probes the built image.
- The router is tested in-process with `tower::ServiceExt::oneshot`, as in `crates/adapters/tests/http.rs`.
- Check that a new test can fail: break the code it guards once, and see it go red.
