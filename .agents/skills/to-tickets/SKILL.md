---
description: "Planning-lane ticketing step: break a clear spec or plan into independently grabbable implementation tickets on the project issue tracker. Typical input: a spec from `to-spec` or another already-clear plan. Typical output: execution-ready tickets; unresolved planning decisions should go back upstream instead of being hidden in ticket text."
---

# To Issues

Break a clear spec or plan into independently-grabbable issues using vertical slices (tracer bullets).

Use the repo's installed issue-tracker workflow and triage labels. If that seam has not been tailored yet, leave a clear placeholder for post-install review instead of instructing the user to install upstream setup skills.

## Process

### 1. Gather context

Work from whatever is already in the conversation context. If the user passes an issue reference (issue number, URL, or path) as an argument, fetch it from the issue tracker and read its full body and comments.

If the source still has unresolved planning decisions, stop and send it back to the planning lane (`plan-review` or `wayfinder`) instead of papering over the ambiguity in tickets.

### 2. Explore the codebase (optional)

If you have not already explored the codebase, do so to understand the current state of the code. Issue titles and descriptions should use the project's domain glossary vocabulary, and respect ADRs in the area you're touching.

### 3. Draft vertical slices

Break the plan into **tracer bullet** issues. Each issue is a thin vertical slice that cuts through ALL integration layers end-to-end, NOT a horizontal slice of one layer.

Slices may be 'HITL' or 'AFK'. HITL slices require human interaction, such as an architectural decision or a design review. AFK slices can be implemented and merged without human interaction. Prefer AFK over HITL where possible.

<vertical-slice-rules>
- Each slice delivers a narrow but COMPLETE path through every layer (schema, API, UI, tests)
- A completed slice is demoable or verifiable on its own
- Prefer many thin slices over few thick ones
</vertical-slice-rules>

### 4. Quiz the user

Present the proposed breakdown as a numbered list. For each slice, show:

- **Title**: short descriptive name
- **Type**: HITL / AFK
- **Blocked by**: which other slices (if any) must complete first
- **User stories covered**: which user stories this addresses (if the source material has them)

Ask the user:

- Does the granularity feel right? (too coarse / too fine)
- Are the dependency relationships correct?
- Should any slices be merged or split further?
- Are the correct slices marked as HITL and AFK?

Iterate until the user approves the breakdown.

### 5. Publish the issues to the issue tracker

For each approved slice, publish a new issue through the repo's issue-tracker workflow. Use the issue body template below. These issues are considered ready for AFK agents, so publish them with the repo's ready triage label unless instructed otherwise.

Publish issues in dependency order (blockers first) so you can reference real issue identifiers in the "Blocked by" field.

Each published child ticket must include a **Suggested branch name** field near the top of the issue body. If the source is an existing parent issue and it already has a **Suggested branch name** field, copy that value verbatim into every child ticket. If the source is a clear plan without a parent issue, synthesize one shared `feature/<feature-slug>` suggestion once from the larger initiative or spec title and reuse it for every child ticket.

<issue-template>
## Suggested branch name

`feature/<feature-slug>`

## Parent

A reference to the parent issue on the issue tracker (if the source was an existing issue, otherwise omit this section).

## What to build

A concise description of this vertical slice. Describe the end-to-end behavior, not layer-by-layer implementation.

Avoid specific file paths or code snippets — they go stale fast. Exception: if a prototype produced a snippet that encodes a decision more precisely than prose can (state machine, reducer, schema, type shape), inline it here and note briefly that it came from a prototype. Trim to the decision-rich parts — not a working demo, just the important bits.

## Acceptance criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Blocked by

- A reference to the blocking ticket (if any)

Or "None - can start immediately" if no blockers.

</issue-template>

Do NOT close or modify any parent issue.

After publishing, the work leaves planning and is ready for execution skills such as `implement` or `fix`.
