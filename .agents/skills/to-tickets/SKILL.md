---
name: to-tickets
description: Break an accepted plan into linked GitHub slice issues under one parent planning issue.
---

# to-tickets

Split an accepted plan into implementation slices stored in GitHub Issues.

This repo's planning artifacts live in GitHub Issues; `docs/features/` is historical archive material.

## Input sources

- Preferred source: the parent GitHub planning issue created by `to-spec`.
- If the human starts from chat or another doc, derive the feature topic from that source and confirm the parent issue in the output.
- Treat parent issues, prior PRDs, and issue notes as historical for behavior claims until verified against live code, tests, and `CONTEXT.md`.

## Output

- Create one child GitHub issue per slice.
- Keep all slices linked to the same parent issue.
- Put the parent issue reference in each slice issue's **Parent** section.
- In **Blocked by**, reference sibling slice issue numbers when dependencies exist.
- Use labels that distinguish slice issues from parent planning issues.

## Workflow

1. **Read the source plan**
   - Read the parent issue or planning source fully.
   - Read `CONTEXT.md` and relevant ADRs.
   - Verify any behavior-sensitive claims against current code and tests before turning them into required slice outcomes.

2. **Map the implementation seams**
   - This repo is a single Rust crate, so slice by capability or seam, not by package.
   - Prefer slices that follow existing repository boundaries (`src/app/`, `src/command/`, `src/guilds/`, `src/triggers/`, `src/ui/`, docs/wiki/docs updates) when evidence supports them.
   - Keep each slice independently reviewable and buildable.

3. **Write slice issues**
   - Create one GitHub issue per slice.
   - Each slice should state the goal, concrete scope, non-goals when needed, dependencies, and acceptance checks.
   - Use repository gate language already evidenced in docs when relevant: `cargo fmt`, build/typecheck, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.
   - Keep acceptance criteria specific enough that another agent can pick up the slice without reopening product decisions.

4. **Cross-link and stop**
   - Cross-link sibling slices where ordering matters.
   - Link slices from the parent issue body or checklist.
   - Stop after writing tickets; do not start implementation unless the human explicitly asks.

## Rules

- Publish tickets to GitHub Issues, not repo-local Markdown files.
- Keep tracker conventions aligned with `docs/agents/issue-tracker.md`.
- Prefer a small number of meaningful slices over speculative micro-tickets.
- Do not silently fall back to `docs/issues/` if GitHub issue creation fails.
- Do not invent monorepo package steps or frontend test guidance for this repo; use Cargo-based examples instead.
