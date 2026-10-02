---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

Use `/implement` as the feature and general execution lane. Typical input: a clear spec, ticket, or already-diagnosed task that is ready to build. Typical output: implemented changes, validation evidence, and an execution handoff that is ready for `/review` and `/code-review`. If the request is actually an unclear bug, route it first to `diagnosing-bugs`; if the work is specifically bug execution with a clear diagnosis, prefer `/fix`. `/tdd` is not a separate destination here — it is the red→green loop you attach inside this implementation lane when the task or risk warrants it.

### Before you start

1. Check `git branch --show-current` and `git status`.
2. **Already on the right branch** — if the current branch matches the spec/ticket (or the user named one this session), proceed.
3. **Clean tree on default branch** — first look for a published **Suggested branch name** on the immediate spec/ticket artifact and prefer that when present. If no suggestion exists, ask once: create `feature/<feature-slug>` from the feature, epic, PRD, or spec/task description? Propose a short descriptive slug from the larger initiative name when one exists; recommend yes. The default is feature-based naming, not issue-number naming, and one feature branch may span multiple sibling tickets in the same initiative. If the user already chose a branch name, honor it. Only create after confirmation.
4. **Unrelated dirty WIP** — if changes are not part of this spec/ticket, ask once:
   - **Worktree** (recommended for substantial or active WIP): isolated checkout, original WIP untouched.
   - **Stash and branch** — fine for small WIP.
   - **Commit WIP first** — when the user wants WIP preserved on the current branch.
     Do not branch from a dirty HEAD when the dirt is unrelated.
5. **Worktree setup** (when chosen):
   - `git fetch origin`
   - `git worktree add ../<repo>-<feature-slug> -b feature/<feature-slug> origin/main`
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

Do **not** commit unless the user explicitly asks. When work is ready, offer the wrap-up action that fits the input artifact and wait for confirmation:

- **Ticket-backed work (GitHub issue or repo-local ticket file with unambiguous provenance):** offer to commit the implementation changes and close/mark complete the originating ticket in the same step. Keep both actions optional so the user can approve commit-only, close-only, both, or neither.
- **Free-form spec work:** offer to commit only.
- **Ambiguous provenance:** do not guess; fall back to the current commit-only offer.

Never auto-commit or auto-close. Suggest ticket closure only after implementation, validation, review, and any future-sibling knowledge handoff are complete.

### After implementation: hand off knowledge to future tickets

If this ticket is part of a multi-ticket feature/PRD, pass along knowledge that future sibling tickets would otherwise rediscover the hard way.

1. **Identify future siblings.** Find the epic/PRD issue this ticket belongs to (the `to-spec → to-tickets` flow groups tickets under an epic issue; the ticket body usually names it). Enumerate sibling tickets via the issue-tracker commands for this repo in `docs/agents/issue-tracker.md`. A **future sibling** = same epic, still open, and not the current ticket. If there is no epic/ticket linkage (free-text spec), **skip** — do not fabricate a group.
2. **Cap the knowledge** to what a future ticket actually needs — four buckets only:
   - **Contracts/seams** established that siblings touch (interfaces, boundaries, seams tested at).
   - **Decisions that changed the plan** — divergence from the spec and why; flag ADR candidates, don't write ADRs here.
   - **Traps/gotchas** hit and the fix (matches the repo `known-traps` concept).
   - **File/API map** of touched surfaces relevant to siblings.
     Exclude progress status and anything that only concerns the current ticket. If a sibling depends on an interface/seam this ticket changed, call out the **breaking change** and the dependency explicitly.
3. **Draft, don't write.** Draft one delivery per future sibling, prefixed with `**Implementation knowledge from <current ticket>**`, covering the four buckets above, addressed to the repo's knowledge-handoff destination.
4. **Confirm once before delivering** — same approval gate as committing. Show the drafted delivery, ask, and only deliver after confirmation. Fire this section only after review passes, and before the "offer to commit" step.

> **Tracker seam for this repo:** use `/review` for risk-first review, `/code-review` for the two-axis review, `diagnosing-bugs` for diagnosis-first work, `/tdd` for explicit TDD guidance, and the GitHub issue workflow in `docs/agents/issue-tracker.md` (`gh issue ...`) for ticket reads/comments.

This section is a no-op when the ticket is not part of a multi-ticket feature.
