---
name: finish-slice
description: Close a slice of work in The Gold Ballot — run the checks, review against the spec, and hand Tommy the tally and the exact git commands. Use when a slice's code is written and before saying it is done.
---

# Finish a slice

Tommy commits, pushes and merges himself. This skill prepares everything up to that point and changes nothing in git.

## 1. Verify

Send each command's output to a file in the scratchpad and read only failures and totals.

1. `just check`
2. `just smoke-image`, if anything under `crates/`, `frontend/`, `Dockerfile` or `scripts/smoke-image.sh` changed.
3. For each new test that pins a spec rule, confirm it can fail: break the guarded code once and see it go red, then restore it.

A failure is fixed before going on, or reported as the slice's state. Never report a check as passed that did not run.

## 2. Review against the spec

Ask the `spec-conformance-reviewer` subagent to review `git diff origin/main` against the spec rules this slice cites.
Fix what it finds, or carry each finding into the tally below.

## 3. Report

Write the hand-off in this order.

1. **State in one sentence:** what works now, and the most important thing that does not.
2. **Not verified:** each thing that could not be checked, and why.
3. **Choices made** that the spec left open, one line each.
4. **Tally:** a table of every spec rule the slice touched, each with one disposition:
   - done, with how it was verified;
   - blocked, with the reason and who can unblock it;
   - deferred, only if Tommy confirmed the deferral.
5. **New concepts:** one line for each Svelte, SvelteKit, `tracing` or OpenTelemetry concept the slice used for the first time, with a `file:line`. Mention `/explain-slice`.
6. **Tommy's steps, in order:**
   - anything outside the repo first, such as running `scripts/bootstrap-gcp.sh` or adding a line to `.env`, with why it must come before the merge;
   - then the git commands, in a code block: `git add -A`, a `git commit -m` with a message in the style of `git log --oneline -5`, `git push -u origin <branch>`, `gh pr create --fill`;
   - then what to check after the deploy, with the exact command or URL.

## 4. Leave things tidy

- Stop any server or container this session started, except the compose services.
- If documentation describes what changed (`README.md`, `docs/deploy.md`, a file in `.claude/rules/`), it is updated in the same slice.
