---
name: plan-review
description: "First planning-lane review step: stress-test a plan against the domain model and documented decisions, resolve questions using recommended answers, and return a review table without writing files. Typical input: a loose plan, decision set, or proposal. Typical output: an accepted planning review table that either stays in planning for more clarification or moves forward through `to-spec` and then `to-tickets`."
---

<what-to-do>

Stress-test every aspect of this plan until the design tree is walked and dependencies between decisions are resolved. For each decision point, formulate a clear question and your recommended answer.

**Do not ask the user questions during composition.** Use your recommended answers. Explore the codebase when a question can be answered from code or existing docs.

**Do not write or edit any files during composition** — no glossary updates, decision records, or other project documentation until the user has explicitly accepted the result table.

When composition is complete, present a single **Plan review result** table (see format below) and stop. Wait for the user to **reject**, **change** some answers, or **accept**.

- **Reject** — discard the table; do not write files. Offer to restart from the plan if useful.
- **Change** — update only the rows the user specifies; show the revised table; still no file writes until they accept.
- **Accept** — do not write files here. If the accepted outcome still has unresolved decisions, keep it in planning and direct the user to continue with another planning pass instead of moving downstream yet. If the accepted outcome has a clear plan or clear documentation work, direct the user to run `to-spec` and then `to-tickets` so it becomes agent-grabbable work. If the accepted outcome is pure confirmation, say there is nothing to forward.

</what-to-do>

## Plan review result table

Present every resolved decision in one table:

| #   | Question | Answer | Explanation                                                                      |
| --- | -------- | ------ | -------------------------------------------------------------------------------- |
| 1   | …        | …      | Why this answer; conflicts with existing docs/code if any; trade-offs considered |

Rules:

- One row per decision — not one row per brainstorm bullet.
- **Question** — the decision as a precise question (terminology, boundary, trade-off, scenario outcome).
- **Answer** — the chosen resolution (canonical term, yes/no, chosen alternative).
- **Explanation** — brief rationale: evidence from existing project documentation or code; edge cases; why alternatives were rejected.

After the table, add a short **Documentation impact** section listing what follow-up work should be forwarded on accept (e.g. glossary terms to add/update, decision-record candidates with one-line why). No file edits in this section — inventory only.

<supporting-info>

## Discover existing documentation

Before composing answers, find how this repo records domain language and decisions. Look for common locations and naming — for example a root glossary file, `docs/` decision records, README architecture sections, or a context map for multi-area repos. Use whatever the project already has; do not impose a layout the repo does not use.

If nothing exists yet, note in **Documentation impact** what you would create on accept and where (following nearby conventions in the repo).

## During composition (read-only)

Apply the same rigor as interactive plan review, but resolve each point yourself and record it in the table.

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

No files are written here. This skill is a review surface only.

### Forward accepted documentation work

If the accepted table still contains unresolved planning decisions, keep the result in planning: tell the user to continue with another planning pass instead of moving downstream yet. If the accepted table implies a clear spec-worthy plan or glossary, decision-record, or other documentation follow-up, direct the user to run **`to-spec`** and then **`to-tickets`** so the accepted changes become agent-grabbable tickets. Never write the docs inline here, never pick up implementation yourself, and never offer to “start building”.

### Pure confirmation outcome

If the accepted table only confirms the current plan or language and **Documentation impact** is empty, say **nothing to forward** instead of invoking the ticket pipeline.

</supporting-info>
