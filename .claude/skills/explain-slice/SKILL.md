---
name: explain-slice
description: Walk Tommy through what a slice of The Gold Ballot taught, concept by concept, with pointers into the code and small exercises.
disable-model-invocation: true
argument-hint: "[topic or path; default: the current branch's diff against main]"
---

# Explain a slice

Tommy is building this app to learn. His stated goals are a new frontend framework (Svelte and SvelteKit) and OpenTelemetry with `tracing`.
He comes from React and TypeScript; do not assume his level in Rust, ask if it matters.

Explain: $ARGUMENTS
If that is empty, explain the changes on the current branch: `git diff origin/main --stat`, then the files that matter.

## How to explain

- Pick the three to six concepts the slice actually rests on. Leave out what he has seen in an earlier slice unless it is used in a new way.
- For each concept, in this order:
  1. **What it is,** in two sentences, in plain words.
  2. **Where it is,** as `file:line`, quoting at most five lines.
  3. **Why this way here,** naming the alternative that was not chosen.
  4. **The nearest thing in React or TypeScript,** and where the analogy breaks. Skip this when there is none.
- Check a claim about SvelteKit 3, Svelte 5 or the OpenTelemetry crates against current documentation before making it. Say so when something is your inference.
- Prefer the code in this repo over generic examples.

## Exercises

End with two or three exercises he can do in under fifteen minutes each. Each one:

- changes real code in this repo, in a place you name;
- has a way to see the result: a test to run, a page to load, or a trace to find in the local Grafana at `http://localhost:3000`;
- states what he should observe, without giving the solution.

Offer to review his attempt. Do not make the change yourself unless he asks.

## Keep it short

One screen per concept at most. Stop after the exercises and ask which concept he wants deeper.
