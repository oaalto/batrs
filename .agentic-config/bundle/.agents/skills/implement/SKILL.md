---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

### Before you start

1. Check `git branch --show-current` and `git status`.
2. **Already on the right branch** — if the current branch matches the spec/ticket (or the user named one this session), proceed.
3. **Clean tree on default branch** — ask once: create `feature/<short-name>` from the spec/issue? Propose the name (base it on the epic issue for the ticket when one exists); recommend yes. Only create after confirmation.
4. **Unrelated dirty WIP** — if changes are not part of this spec/ticket, ask once:
   - **Worktree** (recommended for substantial or active WIP): isolated checkout, original WIP untouched.
   - **Stash and branch** — fine for small WIP.
   - **Commit WIP first** — when the user wants WIP preserved on the current branch.
     Do not branch from a dirty HEAD when the dirt is unrelated.
5. **Worktree setup** (when chosen):
   - `git fetch origin`
   - `git worktree add ../<repo>-<branch-slug> -b feature/<slug> origin/main`
   - Run all edits, tests, and commits in that path; install deps there if needed.
6. After merge, remind the user: `git worktree remove <path>`.

### Test-driven development (TDD)

TDD is the red → green loop. When the user wants to build features or fix bugs test-first, or mentions "red-green-refactor", or wants integration tests:

- Read `CONTEXT.md` (if it exists) so test names and interface vocabulary match the project's domain language.
- **Seams:** A seam is the public boundary you test at. Test only at **pre-agreed seams** — before writing any test, write down the seams under test and confirm them with the user.
- **Anti-patterns to avoid:** implementation-coupled tests (mock internals), tautological tests (assertion recomputes the expected value the way the code does), horizontal slicing (write all tests first, then all implementation). Work in **vertical slices**: one test → one implementation → repeat.
- **Rules of the loop:** Red before green (write the failing test first, then only enough code to pass it). One slice at a time. Refactoring belongs to the review stage, not the loop.

### Review

Once done, **delegate review to sub-agents** — do not review inline in the implement thread.

1. **Risk-first review** — spawn one sub-agent. Prompt it to load and follow `/review` with the task intent, changed files, and diff. Return that skill's output (`## Findings`, `## Open Questions`, `## Residual Risk`).
2. **Two-axis review** — load `/code-review`, pin the fixed point, then spawn **two parallel sub-agents** (Standards and Spec) per that skill's step 4. Aggregate their reports under `## Standards` and `## Spec`.

Launch the risk-first sub-agent and both two-axis sub-agents in one parallel batch when the harness allows. Aggregate every sub-agent report before finishing.

### After implementation

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Do **not** commit unless the user explicitly asks. When work is ready, offer to commit and wait for confirmation.

### After implementation: hand off knowledge to future tickets

If this ticket is part of a multi-ticket feature/PRD, pass along knowledge that future sibling tickets would otherwise rediscover the hard way.

1. **Identify future siblings.** Find the epic/PRD issue this ticket belongs to (the `to-spec → to-tickets` flow groups tickets under an epic issue; the ticket body usually names it). Enumerate sibling tickets via the issue-tracker commands for this repo (set at install — see the post-install review). A **future sibling** = same epic, still open, and not the current ticket. If there is no epic/ticket linkage (free-text spec), **skip** — do not fabricate a group.
2. **Cap the knowledge** to what a future ticket actually needs — four buckets only:
   - **Contracts/seams** established that siblings touch (interfaces, boundaries, seams tested at).
   - **Decisions that changed the plan** — divergence from the spec and why; flag ADR candidates, don't write ADRs here.
   - **Traps/gotchas** hit and the fix (matches the repo `known-traps` concept).
   - **File/API map** of touched surfaces relevant to siblings.
     Exclude progress status and anything that only concerns the current ticket. If a sibling depends on an interface/seam this ticket changed, call out the **breaking change** and the dependency explicitly.
3. **Draft, don't write.** Draft one delivery per future sibling, prefixed with `**Implementation knowledge from <current ticket>**`, covering the four buckets above, addressed to the repo's knowledge-handoff destination (issue comment vs local `docs/issues/` file — set at install).
4. **Confirm once before delivering** — same approval gate as committing. Show the drafted delivery, ask, and only deliver after confirmation. Fire this section only after review passes, and before the "offer to commit" step.

> **Tracker seam (initialize at install):** the issue-tracker commands and knowledge-handoff destination above are placeholders. The project's post-install review wires them to the installed `implement` skill body (`/review`, `/code-review`, and the issue tracker/command set confirmed against this repo's actual setup).

This section is a no-op when the ticket is not part of a multi-ticket feature.
