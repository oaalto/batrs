---
name: to-spec
description: Turn a rough idea into a PRD saved in `docs/prds/<feature_name>/prd.md`, then stop for review before implementation or ticket-splitting.
disable-model-invocation: true
---

# to-spec

Turn a rough feature idea, planning thread, or accepted grill outcome into a repo-local PRD.

This repo's planning artifacts live under `docs/prds/` and `docs/issues/`; `docs/features/` is historical archive material.

## Output path

- Save the spec to `docs/prds/<feature_name>/prd.md`.
- Derive `<feature_name>` from the feature topic using a short kebab-case slug.
- Create `docs/prds/<feature_name>/` when missing.
- If a relevant PRD already exists for the same feature, update it instead of creating a parallel spec.

## Workflow

1. **Gather context first**
   - Read the user request and any linked docs.
   - Read `CONTEXT.md` for domain language.
   - Read relevant ADRs under `docs/adr/`.
   - Read related feature docs under `docs/features/` when the repo already solved or partly specified adjacent behavior.
   - Treat prior PRDs, issue notes, and chat history as historical unless verified against code, tests, or `CONTEXT.md`.

2. **Inspect live repo evidence**
   - Confirm the current implementation seams in code before writing behavior claims.
   - Use the repo's actual stack and commands in examples: this repo is a single Rust crate using Cargo workflows, not a monorepo.
   - Where validation matters, prefer Cargo gate language already used in the repo (`cargo fmt`, build/typecheck, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`).

3. **Write the PRD**
   - Capture the user problem, desired outcomes, non-goals, user stories, constraints, acceptance checks, and rollout or follow-up notes.
   - Keep language aligned with `CONTEXT.md` and existing docs.
   - Name unresolved choices explicitly instead of inventing decisions.
   - Use concrete repo paths when grounded in evidence.

4. **Status and handoff**
   - Include a `## Status` section near the top (`draft`, `in review`, `accepted`, or `superseded`).
   - When the PRD is ready, stop and ask whether to refine it or run `to-tickets` next.

## Rules

- Save to Git, not an external tracker, unless the human explicitly redirects.
- Do not start implementation.
- Do not silently create a second competing spec for the same feature.
- Keep the PRD path convention consistent with `docs/agents/issue-tracker.md`.
- If you reference future slices, keep them descriptive only; `to-tickets` owns the actual slice files.
