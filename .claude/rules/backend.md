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
  Clock and storage exist; the identity provider arrives with plan item P3.4, the results source with its first caller.
- Authorization lives in the use cases, through an actor value.
  The HTTP adapter only turns the session cookie into an `Actor`; a request without a live session is `Actor::Anonymous`,
  and the use case refuses it.
- A storage port is one trait per aggregate in `application/src/storage.rs`; `PostgresStorage` implements them all.
- A type that holds a secret or a configured list, such as `SessionToken` or `AdminEmails`, writes its own `Debug` that hides the value.
  A type with a player's data, such as `Player` or `Actor`, derives `Debug` for test output and is never passed to a `tracing` macro; a span records `player.id` and nothing else about a player.
- There is no unit-of-work abstraction: no use case writes two aggregates atomically.
- Wire types belong to `adapters`. The domain carries no serialization or OpenAPI annotations.
- Scores are derived on every read and never stored.

## Errors

- An error type derives `thiserror::Error`, with one `#[error("...")]` message per variant; an error gets no hand-written `Display` or `Error` impl.
- Where an error only wraps another, the conversion is a `#[from]` variant; `map_err` is for adding context the source lacks, such as the variable's name.
- `thiserror` is declared once, under `[workspace.dependencies]`. It is the only run-time dependency of `domain`.
- Code that runs in production does not panic on a failure: clippy denies `unwrap` and `expect` outside tests.
  A failure becomes a typed error, or a value that is right at the limit, such as a saturated count.
  A call that cannot fail by construction keeps its `expect` under `#[expect(clippy::expect_used, reason = "...")]`,
  and the reason names what guarantees it. A file under `tests/` allows both lints in its first lines.

## HTTP

- Handlers stay thin: extract, call a use case, map the result.
- Every `/api` answer carries `Cache-Control: no-store`, set once in `http/mod.rs`; a handler sets no cache header.
- A cookie header is read as bytes: another cookie of the host may hold bytes that are not ASCII.
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
- Postgres stores and compares; it decides nothing. The application writes every timestamp from the clock port,
  so no column defaults to `now()`, and it computes every derived value, such as a display name's uniqueness key.
- A constraint has an explicit name, and the test of a refused statement asserts that name.
- Builds are offline: `query!` reads the metadata in `.sqlx`. After changing a query or a migration, run
  `just prepare-sqlx`; `just check-rust` fails on stale metadata.
- Call the sqlx CLI only through `just`, or with `--no-dotenv`: it echoes a line of `.env` it cannot parse.

## Telemetry

- Only `crates/adapters/src/telemetry.rs` names OpenTelemetry types; everything else uses `tracing` macros and the `Telemetry` handle.
- The OpenTelemetry crates are pinned and upgraded together, in one pull request.
- Telemetry carries route templates and player IDs only: no email, name, pick, cookie, token or raw URL.
- A use case gets a span with `#[tracing::instrument(skip_all)]`; a storage function gets one with `otel.kind = "client"`.

## Tests

- A rule is closed only when a test proves its done-condition, read literally (plan decision D22).
  A proof that will come later leaves the rule open, owned by the plan item that brings it.
- A test that pins a spec rule carries its number in its name: `rule_2_4_5_readiness_fails_without_the_database`.
- Package 1 is stricter, because `scripts/check-rule-ledger.sh` counts its rules as tested by name:
  a test is named `rule_1_x_y_` only when it meets that rule's done-condition, and the rule's line leaves
  `scripts/rule-ledger-pending.txt` in the same change.
  A test of a package-1 rule's pure half, closed by a later plan item, gets a plain name and a comment naming the rule.
  Outside package 1, a test may carry the number of a rule it pins only in part;
  its comment then says which part, and names the plan item that closes the rule.
- Rules are tested in `domain` with unit and property tests, without IO.
- Use cases are tested against real Postgres with `#[sqlx::test]`, a fake clock and a fake identity. There is no in-memory storage fake.
- The fake identity adapter sits behind the compile-time feature `fake-identity`, which the release image is built without; CI probes the built image.
  `just check-rust` lints and tests with every feature. A test file that signs in through the fake route starts with `#![cfg(feature = "fake-identity")]`.
- A use-case test passes its own fake clock; it never sleeps and never reads the machine's time.
- The router is tested in-process with `tower::ServiceExt::oneshot`, as in `crates/adapters/tests/http.rs`.
- Check that a new test can fail: break the code it guards once, and see it go red.
