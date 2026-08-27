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
- **Accept** — apply accepted answers to documentation (see [After acceptance](#after-acceptance)). The next step is always the installed planning flow (`to-spec` → `to-tickets`) — never implementation.

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

Before composing answers, find how this repo records domain language and decisions. Look for common locations and naming — for example a root glossary file, `docs/` decision records, README architecture sections, or a context map for multi-area repos. Use whatever the project already has; do not impose a layout the repo does not use.

If nothing exists yet, note in **Documentation impact** what you would create on accept and where (following nearby conventions in the repo).

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

Only after the user explicitly accepts the table. Do **not** start implementing under any circumstances.

### Always: create tickets from this grind

Once the table is accepted (and any accepted documentation below is written), the next step is the installed planning flow — turn the accepted decisions into a spec, then into tickets. Never pick up the implementation yourself and never offer to “start building”. If the user seems ready to move on, offer to run the planning flow so the work becomes tickets an agent can grab.

### Update domain language docs

Apply accepted glossary or terminology changes to the files this repo uses for domain language. Match existing format and tone.

If `grill-with-docs` is installed and ships format guides (`CONTEXT-FORMAT.md`, `ADR-FORMAT.md`, or similar next to that skill), follow them. Otherwise mirror the style of existing docs in the repo.

Domain-language docs should describe **what things mean**, not implementation details.

### Write decision records

Create decision records only for items the user accepted from **Documentation impact** that meet the three criteria above. Place them where the repo already keeps architectural or design decisions; create that location only if the repo has no precedent and acceptance included that inventory item.

</supporting-info>
