# Research: frontend frameworks

Researched on 2026-11-16. Part of [the research record](README.md).
Labels: **C** confirmed, **P** probable, **U** uncertain. Source keys (S1…) resolve at the end.

Ruled out before research: React, and Rust-to-WASM frameworks (Yew, Leptos, Dioxus).
No candidate was ranked or chosen.

## Summary table

| Candidate | Stable | In flux now | New for a React developer (inference) | Static SPA | Usage / retention |
|---|---|---|---|---|---|
| Vue | 3.5.45 | 3.6 at RC, adds opt-in Vapor Mode | Proxy reactivity, templates, setup runs once | Yes | 52.4% / 84.5% |
| Svelte | 5.57.3 | SvelteKit 3.0 is six weeks old | Compiler, runes, `.svelte` files | Yes; docs warn of a performance cost | 27.3% / 87.0% |
| Solid | 1.9.19 | 2.0 at RC removes core APIs; SolidStart moving to maintenance | Fine-grained signals in JSX, components run once | Yes | 10.4% / 90.0% |
| Angular | 22.2.6 | Settled; majors now yearly | Classes, dependency injection, signals | Yes | 47.9% / 48.1% |
| Qwik | 1.20.4 | 2.0 at RC | Resumability | Not found | 3.5% / 64.8% |
| Lit | 3.3.3 | Settled | Web components | Router is a 0.1 labs package | 10.3% / 67.7% |
| Marko | 6.4.16 | Settled | Its own language | Client-only mode undocumented | not reported |
| Ripple | 0.6.1 | Pre-1.0 | `.tsrx` files | No routing or testing docs | not reported |
| Remix 3 | 3.2.0 | Six weeks old, no React inside | Explicit-update components | Has an SPA package | not reported |
| htmx | 2.0.11 | 4.0 on `next` | HTML over the wire | Rust must return HTML | 9.0% / 69.3% |

## Facts that apply to every candidate

- **State of JS 2026 is not published.**
  stateofjs.com lists editions through 2025 only [C, S5].
  The 2025 edition had 13,002 respondents [C, S4].
- **Survey figures are "usage / retention / interest".**
  Usage is the share who have used it [C, S4].
  Retention and interest are positive ÷ (positive + negative) sentiment, computed by the research agent from the survey API; official charts may differ by a point or two [P, S4].
- **npm figures are weekly downloads, 2025-11-03..09 → 2026-11-08..14** [C, S2].
  React itself went 50.1M → 258.5M, so absolute growth looks registry-wide (inference).
- **TypeScript 7 breaks template type-checking for now.**
  `typescript@latest` is 7.0.2 (2026-07-08) [C, S1].
  It shipped without the compiler API; the 7.1 beta (2026-10-20) restores it, with stable tentatively late November [P, S32].
  - svelte-check 4.7.8 accepts `^5 || ^6`; @angular/compiler-cli 22.2.6 accepts `>=6.0 <6.1` [C, S1].
  - vue-tsc 3.3.14 declares `>=5.0.0`; Vue Language Tools 4.0.0-alpha.3 is the port in progress [C, S1].
  - Vue, Svelte and Angular projects are advised to keep TS 6 for framework tooling [P, S32].
- **Testing base.**
  Vitest is 5.1.0 (5.0 on 2026-09-03); browser mode has been stable since 4.0 (2025-10-22) [C, S1; P, S34].
  Playwright is 1.65.1 [C, S1].
  Survey: Vitest 53.8% / 97.8%, Playwright 50.0% / 94.0%, Testing Library 40.9% / 89.6% [P, S4].
- **Live updates.**
  Only htmx ships SSE/WebSocket handling (as bundled extensions) [C, S31].
  The others' APIs were not checked; the usual route is the browser's `EventSource`/`WebSocket` (inference).

## Vue 3 (Vite SPA) / Nuxt

1. **Version and governance.**
   Vue 3.5.45 (2026-11-05); the 3.5 line dates from 2024-09 [C, S1].
   Vue 3.6 is at rc.15 (2026-11-12), in RC since 2026-07-18, with a changelog dominated by opt-in Vapor Mode (no VDOM, with VDOM interop) [C, S6].
   Nuxt is 4.6.4 (4.0 on 2025-07-15); vue-router 5.5.1 and Pinia 4.1.0 both had majors in 2026 [C, S1].
   Vue is MIT and sponsor-funded [C, S10].
   NuxtLabs joined Vercel on 2025-07-08; Nuxt stays MIT [C, S9].
2. **Model.**
   VDOM plus proxy reactivity (`ref`, `computed`, `watch`).
   Setup runs once, there are no dependency arrays, and templates use directives (inference).
3. **TypeScript.**
   `vue-tsc` and the Vue - Official extension check templates; Vite only transpiles [C, S7].
   Reported friction: typing complex props/emits and slow checks on large codebases [P, S35].
4. **Static output.**
   Nuxt with `ssr: false` and `nuxt generate` emits `index.html` plus JS; prerendering adds `200.html`/`404.html`, and fallback handling varies by host [C, S8].
   A plain Vite build is static (inference).
5. **Testing.**
   @vue/test-utils 2.6.0 and vitest-browser-vue 3.1.0 are current; @testing-library/vue 8.1.0 was last published 2024-05 [C, S1].
6. **Ecosystem.**
   @tanstack/vue-query 5.107, @tanstack/vue-form 1.33, vee-validate 4.15.1 (2025-06), reka-ui 2.11, @ark-ui/vue 5.40, PrimeVue 5.0 [C, S1].
7. **Adoption.**
   Vue 52.4% / 84.5% / 47.4%; Nuxt 23.6% / 80.4% [S4].
   npm: vue 7.42M → 25.05M, nuxt 1.02M → 3.40M [C, S2].
   Stars: 55.0k core, 61.1k Nuxt [C, S3].
8. **Pain points.**
   `ref` vs `reactive`, destructuring losing reactivity, Vue 2 legacy [P, S35].

## Svelte 5 / SvelteKit

1. **Version and governance.**
   Svelte 5.57.3 (2026-10-26); 5.0 was 2024-10 [C, S1].
   SvelteKit 3.0 shipped 2026-10-01 and is now 3.2.0: config moves to `vite.config.ts`, `$lib` becomes `#lib`, and `npx sv migrate sveltekit-3` automates migration [C, S11, S1].
   Remote functions and async Svelte still need experimental flags [C, S11].
   Rich Harris has been employed by Vercel since 2021, with governance stated as unchanged [C, S14].
2. **Model.**
   Compiler plus signal-based runes (`$state`, `$derived`, `$effect`, `$props`), no VDOM, `.svelte` files (inference).
3. **TypeScript.**
   svelte-check plus the editor extension.
   Enums and parameter properties need a preprocessor [C, S13].
4. **Static output.**
   Root-layout `ssr = false` plus adapter-static 4.0.1 with a fallback page; individual pages can still be prerendered.
   The docs warn that "SPA mode has a large negative performance impact" [C, S12].
5. **Testing.**
   @testing-library/svelte 5.4.2, vitest-browser-svelte 3.1.0 [C, S1].
6. **Ecosystem.**
   sveltekit-superforms 3.1, @tanstack/svelte-query 6.4, bits-ui 2.21, @ark-ui/svelte 5.25 [C, S1].
7. **Adoption.**
   Svelte 27.3% / 87.0% / 63.4%; SvelteKit 17.7% / 88.7% [S4].
   npm: svelte 2.04M → 9.69M, kit 0.68M → 5.26M.
   Stars: 88.8k and 20.9k.
8. **Pain points.**
   An open issue is titled "State and runes are too difficult"; runes work only in `.svelte` / `.svelte.ts` files [P, S36].
   SvelteKit 3 is six weeks old, so most tutorials target v2 (inference).

## SolidJS / SolidStart

1. **Version.**
   Stable is 1.9.19 (2026-11-10).
   2.0 is at rc.22 (2026-11-15), in RC since 2026-08-13 with a frozen API and no stable date [C, S1, S15].
   - 2.0 removes `createResource`, `batch`, `on` and `createMutable`, splits `createEffect`, and replaces Suspense/ErrorBoundary with `<Loading>`/`<Errored>` [C, S15].
   - SolidStart 2.0 (2026-08-04, now 2.2.2) targets Solid 1.x and needs Vite 8 and Node 24 [C, S16].
   - The RC post says SolidStart is being retired to maintenance releases in favour of a "start mode" [C, S15].
2. **Model.**
   JSX compiled to direct DOM updates, fine-grained signals, components run once.
   Destructuring props breaks reactivity (inference).
3. **TypeScript.**
   Plain `tsc` checks the TSX (inference).
4. **Static output.**
   `npm init solid` without SolidStart gives a client-only app [C, S17].
   SolidStart has an `ssr` boolean [C, S39].
5. **Testing.**
   A `ts-vitest` template exists [C, S17].
   @solidjs/testing-library 0.8.10 was last published 2024-09 [C, S1].
6. **Ecosystem.**
   @solidjs/router 1.0.3 (2.0 on `next`), @tanstack/solid-query 5.107 (6.0 RC), @kobalte/core 0.13, @ark-ui/solid 5.40, @modular-forms/solid 0.25.1 (2025-03) [C, S1].
7. **Adoption.**
   10.4% / 90.0% / 55.8%; the survey notes highest satisfaction five years running [C, S5].
   npm: 1.04M → 11.64M.
   Stars: 36.3k.
8. **Pain points.**
   The ecosystem straddles 1.x and 2.0 [C, S1]; its small size is a recurring complaint [P, S38, dated].

## Angular

1. **Version and cadence.**
   22.2.6 (2026-11-12); v22 shipped 2026-06-03.
   The six-month major cycle applied "until Angular v22"; it is now one major every 12 months, with 12 months active plus 12 months LTS [C, S18].
   v22 made Signal Forms and `resource`/`httpResource` stable, OnPush the default, and TS 6 required [P, S19].
   Zoneless has been the default since v21 [C, S20].
2. **Model.**
   Signals, templates, dependency injection and classes; the largest concept surface of the set (inference).
3. **TypeScript.**
   The compiler checks templates; `strictTemplates` is on by default in v22 [P, S19].
4. **Static output.**
   `dist/<app>/browser/` deploys to any static host with an `index.html` fallback; `outputMode: "static"` prerenders [C, S22].
5. **Testing.**
   Vitest is the default for new CLI projects, Karma is still supported, and browser mode works through Playwright [C, S21].
6. **Ecosystem.**
   Router, forms and HTTP are built in.
   @angular/aria and CDK are at 22.2.7; TanStack Query's adapter is still named `angular-query-experimental` [C, S1].
7. **Adoption.**
   47.9% / 48.1% / 16.1% [S4].
   npm: 4.32M → 8.20M.
   Stars: 101.5k.
8. **Pain points.**
   "Angular" is itself a named pain-point category in the survey (76 mentions) [C, S5].
   v22 changed several defaults in breaking ways [P, S19].

## Qwik

- **Version.**
  Stable is still 1.20.4 (`@builder.io/qwik`).
  v2 (`@qwik.dev/core`) is at 2.0.0-rc.5, in RC since 2026-10-01 with no stable date; migration is `qwik migrate-v2` [C, S1, S23].
  It took about 2 years 8 months to reach RC [P, S40].
- **Model.**
  Resumability (HTML-first, lazily loaded `$` closures), signals, JSX (inference).
- **Static output.**
  A static adapter prerenders each route [C, S24].
  No documented client-only SPA mode was found [U].
- **Ecosystem.**
  @qwik-ui/headless 0.7.7 (2026-06) [C, S1].
- **Adoption.**
  3.5% / 64.8% / 33.8% [S4].
  npm: v1 20.6k → 59.8k, v2 16.1k.
  Stars: 22.1k.
- **Pain points.**
  Resumability's benefit depends on server-rendered HTML (inference).

## Lit

- **Version and governance.**
  3.3.3 (2026-05-14); 3.0 was 2023-10 [C, S1].
  Moved from Google to the OpenJS Foundation on 2025-10-14 [P, S25].
- **Model.**
  Web components with `html` tagged templates and class-based reactive properties; a library rather than a framework (inference).
- **TypeScript.**
  Template checking relies on community tools [C, S26].
  lit-analyzer and ts-lit-plugin were last published 2024-01; the repo was last pushed 2025-08 [C, S1, S3].
- **Testing.**
  @web/test-runner 1.0.1 (2026-11), @open-wc/testing 5.0 [C, S1].
- **Ecosystem.**
  @lit-labs/router is 0.1.4 (2025-02); @lit/task 1.0.3; @tanstack/lit-query 0.2.38 [C, S1].
- **Adoption.**
  10.3% / 67.7% / 32.4% [S4].
  npm: 3.57M → 9.73M.
  Stars: 21.9k.

## Niche and unconventional options

- **Marko 6.**
  `marko@6` has been `latest` since 2025-04 and is now 6.4.16; eBay sponsors it and the site links OpenJS [C, S1, S27].
  It has its own `.marko` language, @marko/type-check 3.2.3 and a language server [C, S1].
  @marko/run 0.11 has a static adapter that crawls routes at build time [C, S28].
  Client-only mode is undocumented there [U].
  npm: 17.3k → 37.0k.
  Stars: 14.5k.
- **Ripple.**
  0.6.1 (2026-10-29), pre-1.0, by Dominic Gannaway.
  Uses `.tsrx` files, `track()` with `.value`, and `@if`/`@for` control flow; supports `mount()` and SSR.
  The README has no routing or testing section [C, S29, S1].
  34 write-in mentions in the survey [C, S5].
  Stars: 7.4k.
- **Remix 3.**
  `remix@3.0.0` was published 2026-10-01 and is now 3.2.0; the blog index shows only the RC post of 2026-08-31 [C, S1, S30].
  It has no React dependency: it uses its own `@remix-run/component` 1.2.0, and `@remix-run/spa` 1.0.2 is described as "client-rendered application routing" [C, S1].
  Its component model uses explicit updates [P, S41].
- **Astro (island host).**
  7.4.12, with two majors in 2026 (6.0 in March, 7.0 in June) [C, S1].
  The company joined Cloudflare on 2026-01-16 and Astro stays MIT [C, S37].
  Survey: 27.5% / 94.0% [S4].
- **htmx + Alpine (the poor fit on paper).**
  htmx 2.0.11 is `latest`; 4.0.0 shipped 2026-08-28 and sits on `next`.
  v4 uses fetch only, makes inheritance explicit, and bundles SSE/WebSocket extensions [C, S1, S31].
  - Against the brief: the Rust service would have to return HTML fragments rather than only JSON, and very little TypeScript gets written (inference).
  - In its favour: no build step, static assets, and streamed HTML over SSE is a first-party extension [C, S31].
  - Survey: htmx 9.0% / 69.3% / 37.8%; Alpine 9.2% / 77.7% [S4].

## Keeping TypeScript types in sync with the Rust API

| Tool | Current state | Label |
|---|---|---|
| utoipa | 6.0.0 (2026-09-22), utoipa-axum 0.3.0; last commit 2026-10-20; 258 open issues/PRs | [C, S33, S3] |
| aide | Stable 0.15.1 (2025-08); 0.16.0-alpha.4 (2026-04); repo pushed 2026-11-15 | [C, S33, S3] |
| openapi-typescript / openapi-fetch | 7.13.0 / 0.17.0 (both 2026-02-11); repo pushed 2026-09 | [C, S1, S3] |
| orval | 8.48.0 (2026-11-14) | [C, S1] |
| @hey-api/openapi-ts | 0.99.0 (2026-06-22), still 0.x, no release since; repo pushed 2026-11-08 | [C, S1, S3] |
| ts-rs | 12.0.1 (2026-01-31); last commit 2026-11-09 | [C, S33, S3] |
| specta | Stable 1.0.5 (2023-07); 2.0.0-rc.28 (2026-11-05) | [C, S33] |
| typeshare | 1.0.5 (2026-01-02); CLI 1.13.4; last commit 2026-01-02 | [C, S33, S3] |

OpenAPI routes describe endpoints as well as types; ts-rs, specta and typeshare export types only (inference).

## Unknowns

- Official State of JS retention/interest chart values (the figures here are computed), and whether htmx interest fell year on year.
- Stable dates for Vue 3.6, Solid 2.0, Qwik 2.0 and TypeScript 7.1; whether vue-tsc 3.3 actually runs on TS 7.0.
- Whether Qwik or Marko supports a client-only SPA; the testing story for Qwik, Marko, Ripple and Remix 3.
- A Remix 3 stable announcement, and the documentation for `@remix-run/spa`.
- Current governance and funding for Solid and Qwik.
- Quality of the accessible component libraries (only existence and versions were checked).
- 2026-dated pain-point evidence for Solid, Qwik and Lit; what was found is blog-level or dated.
- Why hey-api has had no npm release since June.

## Sources

- S1: registry.npmjs.org/<package> (queried 2026-11-16)
- S2: api.npmjs.org/downloads/point/<range>/<package>
- S3: api.github.com/repos/<owner>/<repo>
- S4: https://api.devographics.com/graphql (State of JS 2025 data)
- S5: https://2025.stateofjs.com/en-US/libraries/front-end-frameworks/ and https://stateofjs.com/en-US
- S6: https://raw.githubusercontent.com/vuejs/core/minor/CHANGELOG.md
- S7: https://vuejs.org/guide/typescript/overview
- S8: https://nuxt.com/docs/4.x/getting-started/deployment
- S9: https://vercel.com/blog/nuxtlabs-joins-vercel
- S10: https://vuejs.org/sponsor/
- S11: https://svelte.dev/blog/sveltekit-3-is-here
- S12: https://svelte.dev/docs/kit/single-page-apps
- S13: https://svelte.dev/docs/svelte/typescript
- S14: https://vercel.com/blog/vercel-welcomes-rich-harris-creator-of-svelte
- S15: https://github.com/solidjs/solid/releases/tag/v2.0.0-rc.0
- S16: https://github.com/solidjs/solid-start/discussions/2281
- S17: https://docs.solidjs.com/quick-start
- S18: https://angular.dev/reference/releases
- S19: https://blog.ninja-squad.com/2026/06/03/what-is-new-angular-22.0
- S20: https://angular.dev/guide/zoneless
- S21: https://angular.dev/guide/testing
- S22: https://angular.dev/tools/cli/deployment
- S23: https://next.qwik.dev/blog/qwik-2-rc/
- S24: https://qwik.dev/docs/guides/static-site-generation/
- S25: https://lit.dev/blog/2025-10-14-openjs/
- S26: https://lit.dev/docs/tools/development/
- S27: https://markojs.com/docs/newsletter/june-2026
- S28: https://markojs.com/docs/marko-run/adapters
- S29: https://github.com/Ripple-TS/ripple
- S30: https://remix.run/blog
- S31: https://four.htmx.org/docs/whats-new-in-htmx-4
- S32: https://www.infoq.com/news/2026/11/typescript-7-1-beta
- S33: crates.io/api/v1/crates/<crate>
- S34: https://voidzero.dev/posts/announcing-vitest-4
- S35: https://www.monterail.com/blog/vue-development-challenges-state-of-vue
- S36: https://github.com/sveltejs/svelte/issues/14978 and https://redlib.hbubli.cc/r/sveltejs/comments/1j2mxzj/a_few_svelte5_notes
- S37: https://astro.build/blog/joining-cloudflare/
- S38: https://dev.to/ryansolid/5-places-solidjs-is-not-the-best-5019
- S39: https://docs.solidjs.com/solid-start/reference/config/define-config
- S40: https://dev.classmethod.jp/en/articles/qwik-2-rc-changes/
- S41: https://betterstack.com/community/guides/scaling-nodejs/remix-3-beyond-react/
