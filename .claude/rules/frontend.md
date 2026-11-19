---
paths:
  - "frontend/**"
---

# Frontend rules (design section 14)

## Before writing

- SvelteKit 3 and Svelte 5 runes postdate most training data. Check the Svelte MCP server's documentation before using an API you have not used in this repo, and run a component through its autofixer before finishing it.
- Load the daisyUI skill before writing markup.
- Imports use `#lib/...` with the file extension; configuration lives in `vite.config.ts`.
- TypeScript stays on 6.x until `svelte-check` supports 7.

## Structure

- The client holds no domain rules. The server says what is editable, what the scores are, and what a player may see.
- Server state comes through SvelteKit load functions. The app is a static SPA: `ssr = false`, no server files.
- Components live beside their route. `src/lib` holds only `api`, `session` and `ui`; a component moves to `lib/ui` when a second route needs it.
- Only `src/lib/api` holds the API client instance; everything else imports its operation functions. ESLint enforces this.
  Generated types may be imported anywhere.
- Pages fetch and own state. Leaf components take props and emit events.

## Styling

- Tailwind and daisyUI only; a scoped `<style>` block needs a stated reason.
- Native elements come first.
- Every colour lives in the one theme in `src/routes/layout.css`; no colour literal anywhere else.

## Tests

- Leaf components are tested in Vitest browser mode with props only, in a `*.svelte.spec.ts` file beside the component.
- Pages are tested by Playwright against the real container. There are no API mocks.
- Every page defines its loading, empty and error states, each with a component test (spec rule 6.4.5).
- Right, wrong and pending are never shown by colour alone (spec rule 6.3.3).
