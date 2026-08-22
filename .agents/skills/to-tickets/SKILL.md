---
name: to-tickets
description: Break an accepted PRD or plan into repo-local slice files under `docs/issues/<feature_name>/<slice-slug>.md`, linked back to `docs/prds/<feature_name>/prd.md`.
---

# to-tickets

Split an accepted plan into implementation slices stored in this repository.

## Input sources

- Preferred PRD source: `docs/prds/<feature_name>/prd.md`.
- If the human starts from chat or another doc, derive `<feature_name>` from the topic and confirm the parent path in the output.
- Treat PRDs as historical for behavior claims until verified against live code, tests, and `CONTEXT.md`.

## Output path

- Save slice files to `docs/issues/<feature_name>/<slice-slug>.md`.
- Keep all slices for one feature in the same folder.
- Link each slice's **Parent** section to `docs/prds/<feature_name>/prd.md` when that PRD exists.
- In **Blocked by**, reference sibling slice paths in the same folder when dependencies exist.

## Workflow

1. **Read the source plan**
   - Read the PRD or planning source fully.
   - Read `CONTEXT.md` and relevant ADRs.
   - Verify any behavior-sensitive claims against current code and tests before turning them into required slice outcomes.

2. **Map the implementation seams**
   - This repo is a single Rust crate, so slice by capability or seam, not by package.
   - Prefer slices that follow existing repository boundaries (`src/app/`, `src/command/`, `src/guilds/`, `src/triggers/`, `src/ui/`, docs/wiki/docs updates) when evidence supports them.
   - Keep each slice independently reviewable and buildable.

3. **Write slice files**
   - One file per slice under `docs/issues/<feature_name>/`.
   - Each slice should state the goal, concrete scope, non-goals when needed, dependencies, and acceptance checks.
   - Use repository gate language already evidenced in docs when relevant: `cargo fmt`, build/typecheck, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
   - Keep acceptance criteria specific enough that another agent can pick up the slice without reopening product decisions.

4. **Cross-link and stop**
   - Cross-link sibling slices where ordering matters.
   - Keep naming consistent with the PRD folder layout from `to-spec`.
   - Stop after writing tickets; do not start implementation unless the human explicitly asks.

## Rules

- Save tickets locally in Git, not to an external issue tracker, unless the human explicitly redirects.
- Keep path conventions aligned with `docs/agents/issue-tracker.md`.
- Prefer a small number of meaningful slices over speculative micro-tickets.
- Do not invent monorepo package steps or frontend test guidance for this repo; use Cargo-based examples instead.
