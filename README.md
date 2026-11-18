# The Gold Ballot

Guess the Oscars with Tommy: a prediction game for the Academy Awards.

A Rust backend (axum) serves a JSON API and a SvelteKit single-page app from one container.
The [frame](docs/frame.md), [design](docs/design.md) and [specification](docs/spec/README.md) explain why it is built this way.

## Run it locally

You need:

- [rustup](https://rustup.rs); the Rust version in `rust-toolchain.toml` installs itself
- Node.js at the version in `.node-version`, and pnpm at the version in `frontend/package.json`
- [just](https://just.systems) and [cargo-watch](https://crates.io/crates/cargo-watch)
- Docker, only for `just image`

Then:

```sh
just dev
```

Open <http://localhost:5173>.
The Vite dev server reloads the frontend and proxies `/api` to the backend on port 8080, which restarts when Rust code changes.

## Check your work

```sh
just check
```

This runs every check CI runs.
The component tests need a browser once: `cd frontend && pnpm exec playwright install chromium`.

After changing an API handler, regenerate the contract and commit the result:

```sh
just generate
```

`just` alone lists every recipe.

## Layout

| Path | Holds |
|---|---|
| `crates/domain` | The game's rules as types and pure functions |
| `crates/application` | Use cases and their ports |
| `crates/adapters` | HTTP, configuration, and the `gold-ballot` binary |
| `frontend` | The SvelteKit single-page app |
| `openapi.json` | The API contract, generated from the handlers |
| `docs` | Frame, research, design and specification |

## Licence

[MIT](LICENSE)
