---
name: to-spec
description: Turn a rough idea into a parent GitHub planning issue, then stop for review before ticket-splitting.
disable-model-invocation: true
---

# to-spec

Turn a rough feature idea, planning thread, or accepted grill outcome into a GitHub planning issue.

This repo's planning artifacts live in GitHub Issues; `docs/features/` is historical archive material.

## Output

- Create or update one parent GitHub issue for the feature.
- Derive a short kebab-case feature slug for labels, titles, or body metadata when needed.
- If a relevant parent issue already exists for the same feature, update it instead of creating a parallel spec.
- Use labels that distinguish parent planning issues from implementation slice issues.

## Workflow

1. **Gather context first**
   - Read the user request and any linked docs.
   - Read `CONTEXT.md` for domain language.
   - Read relevant ADRs under `docs/adr/`.
   - Read related feature docs under `docs/features/` when the repo already solved or partly specified adjacent behavior.
   - Read existing GitHub issues for the same feature when present.
   - Treat prior PRDs, issue notes, and chat history as historical unless verified against code, tests, or `CONTEXT.md`.

2. **Inspect live repo evidence**
   - Confirm the current implementation seams in code before writing behavior claims.
   - Use the repo's actual stack and commands in examples: this repo is a single Rust crate using Cargo workflows, not a monorepo.
   - Where validation matters, prefer Cargo gate language already used in the repo (`cargo fmt`, build/typecheck, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`).

3. **Write the parent issue**
   - Capture the user problem, desired outcomes, non-goals, user stories, constraints, acceptance checks, and rollout or follow-up notes.
   - Keep language aligned with `CONTEXT.md` and existing docs.
   - Name unresolved choices explicitly instead of inventing decisions.
   - Use concrete repo paths when grounded in evidence.
   - Include a clear status section in the issue body when useful, but treat GitHub issue state and labels as canonical status.

4. **Status and handoff**
   - When the parent issue is ready, stop and ask whether to refine it or run `to-tickets` next.

## Rules

- Publish to GitHub Issues, not repo-local PRD files.
- Do not start implementation.
- Do not silently create a second competing parent issue for the same feature.
- Keep issue conventions consistent with `docs/agents/issue-tracker.md`.
- If GitHub issue creation or update is unavailable, fail clearly instead of falling back to `docs/prds/`.
- If you reference future slices, keep them descriptive only; `to-tickets` owns the actual child issues.
