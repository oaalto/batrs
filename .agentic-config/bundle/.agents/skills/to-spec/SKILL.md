---
description: Planning-lane specification step: turn a clear plan into a spec on the project issue tracker. Typical input: a route that is already clear, either directly from `plan-review` when no unresolved decisions remain or from a resolved `wayfinder` map once the planning route is clear. Typical output: a spec/PRD that is ready to move forward to `to-tickets`.
---

This skill turns a planning lane decision into a spec/PRD. Use it only once the route is clear enough to specify: either `plan-review` already resolved the plan, or `wayfinder` pushed back the fog of war until the route became clear. Unresolved decisions stay in the planning lane instead of being forced into the spec. Do NOT interview the user — just synthesize what you already know.

Use the repo's installed issue-tracker workflow and triage labels. If the repo has not wired that seam yet, leave a clear placeholder for the post-install review to tailor instead of telling the user to install upstream setup skills.

## Process

1. Explore the repo to understand the current state of the codebase, if you haven't already. Use the project's domain glossary vocabulary throughout the PRD, and respect any ADRs in the area you're touching.

2. Sketch out the major modules you will need to build or modify to complete the implementation. Actively look for opportunities to extract deep modules that can be tested in isolation.

A deep module (as opposed to a shallow module) is one which encapsulates a lot of functionality in a simple, testable interface which rarely changes.

Document the major modules that will be built or modified, and assume tests should cover all affected modules at the correct seam.

3. Write the PRD using the template below, then publish it through the repo's issue-tracker workflow. Apply the repo's ready-for-agent triage label if that label exists. Include a **Suggested branch name** field near the top of the published issue body using the existing `feature/<feature-slug>` convention, derived from the larger initiative or spec title so sibling tickets can share one feature branch. The preferred next skill after publishing is `to-tickets`.

<prd-template>

## Suggested branch name

`feature/<feature-slug>`

## Problem Statement

The problem that the user is facing, from the user's perspective.

## Solution

The solution to the problem, from the user's perspective.

## User Stories

A LONG, numbered list of user stories. Each user story should be in the format of:

1. As an <actor>, I want a <feature>, so that <benefit>

<user-story-example>
1. As a mobile bank customer, I want to see balance on my accounts, so that I can make better informed decisions about my spending
</user-story-example>

This list of user stories should be extremely extensive and cover all aspects of the feature.

## Implementation Decisions

A list of implementation decisions that were made. This can include:

- The modules that will be built/modified
- The interfaces of those modules that will be modified
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets. They may end up being outdated very quickly.

Exception: if a prototype produced a snippet that encodes a decision more precisely than prose can (state machine, reducer, schema, type shape), inline it within the relevant decision and note briefly that it came from a prototype. Trim to the decision-rich parts — not a working demo, just the important bits.

## Testing Decisions

A list of testing decisions that were made. Include:

- A description of what makes a good test (only test external behavior, not implementation details)
- That tests should cover all affected modules at the correct seam
- Prior art for the tests (i.e. similar types of tests in the codebase)

## Out of Scope

A description of the things that are out of scope for this PRD.

## Further Notes

Any further notes about the feature.

</prd-template>
