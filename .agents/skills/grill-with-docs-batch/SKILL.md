---
name: grill-with-docs-batch
description: Autonomous grilling session that stress-tests a plan against the domain model and documented decisions, resolves questions using recommended answers, and presents a review table before any documentation is written. Use when the user wants grill-with-docs without interactive Q&A — they review, edit, or accept the composed answers first.
---

<what-to-do>

Stress-test every aspect of this plan until the design tree is walked and dependencies between decisions are resolved. For each decision point, formulate a clear question and your recommended answer.

**Do not ask the user questions during composition.** Use your recommended answers. Explore the codebase when a question can be answered from code or existing docs.

**Do not write or edit any files during composition** — no glossary updates, decision records, or other project documentation until the user has explicitly accepted the result table.

When composition is complete, present a single **Grill result** table (see format below) and stop. Wait for the user to **reject**, **change** some answers, or **accept**.

- **Reject** — discard the table; do not write files. Offer to restart from the plan if useful.
- **Change** — update only the rows the user specifies; show the revised table; still no file writes until they accept.
- **Accept** — apply accepted answers to documentation (see [After acceptance](#after-acceptance)).

</what-to-do>

## Grill result table

Present every resolved decision in one table:

| #   | Question | Answer | Explanation                                                                      |
| --- | -------- | ------ | -------------------------------------------------------------------------------- |
| 1   | …        | …      | Why this answer; conflicts with existing docs/code if any; trade-offs considered |

Rules:

- One row per decision — not one row per brainstorm bullet.
- **Question** — the decision as a precise question (terminology, boundary, trade-off, scenario outcome).
- **Answer** — the chosen resolution (canonical term, yes/no, chosen alternative).
- **Explanation** — brief rationale: evidence from existing project documentation or code; edge cases; why alternatives were rejected.

After the table, add a short **Documentation impact** section listing what would be written on accept (e.g. glossary terms to add/update, decision-record candidates with one-line why). No file edits in this section — inventory only.

<supporting-info>

## Discover existing documentation

Before composing answers, find how this repo records domain language and decisions:

- **Domain glossary:** `CONTEXT.md` at repo root (single-context batrs client).
- **Engineering wiki:** `docs/wiki/` — load `.agents/skills/wiki/SKILL.md` and consult `docs/wiki/index.md` + `docs/wiki/path-map.json` for subsystem/concept pages.
- **Planning artifacts:** `docs/features/<feature_name>/prd.md` and slice files (see `docs/agents/issue-tracker.md`).
- **ADRs:** `docs/adr/` when present.

Graphify is not installed — do not assume `graphify-out/`. Use targeted code reads (start from `src/app/mod.rs`, `src/command/mod.rs`, `src/guilds/`) to verify claims.

If nothing exists yet for a topic, note in **Documentation impact** what you would create on accept and where (following nearby conventions in the repo).

## During composition (read-only)

Apply the same rigor as interactive grilling, but resolve each point yourself and record it in the table.

### Challenge against existing language

When the plan uses a term that conflicts with the project's documented vocabulary, note the conflict in that row's explanation and pick the recommended resolution (align with existing docs, extend the glossary, or flag for user override).

### Sharpen fuzzy language

When the plan uses vague or overloaded terms, recommend a precise canonical term in the **Answer** column and explain the distinction in **Explanation**.

### Discuss concrete scenarios

Invent scenarios that probe edge cases and force precision about boundaries between concepts. Encode the outcome as table rows (scenario → resolution).

### Cross-reference with code

When the plan states how something works, check whether the code agrees. If there is a contradiction, surface it in **Explanation** and recommend which side should win (usually code reality unless the plan is intentional change).

### Decision-record candidates (inventory only)

Flag decision-record candidates in **Documentation impact** when all three are true:

1. **Hard to reverse** — meaningful cost to change later
2. **Surprising without context** — a future reader will wonder why
3. **Result of a real trade-off** — genuine alternatives existed

If any is missing, do not list a decision record for that point.

## After acceptance

Only after the user explicitly accepts the table:

### Update domain language docs

Apply accepted glossary or terminology changes to the files this repo uses for domain language. Match existing format and tone.

Mirror the style of existing `CONTEXT.md` sections and wiki concept pages (`docs/wiki/concepts/`). For ADR candidates, follow any format guides next to `.agents/skills/grill-with-docs/` or `.pi/skills/grilling/` if present; otherwise match tone of `docs/adr/` when that directory exists.

Domain-language docs should describe **what things mean**, not implementation details.

### Write decision records

Create decision records only for items the user accepted from **Documentation impact** that meet the three criteria above. Place them where the repo already keeps architectural or design decisions; create that location only if the repo has no precedent and acceptance included that inventory item.

</supporting-info>
