---
description: "Planning-lane map builder for ideas that still have real fog. Typical input: a loose idea or partially clarified proposal. Typical output: a compact decision map that keeps unresolved decisions in planning until the path is clear enough to hand off to `to-spec`, then `to-tickets`."
disable-model-invocation: true
---

This skill is invoked when a loose idea requires more than one agent session to turn into a clear plan. It creates a stateful decision map in a markdown file, and drives the user through a sequence of planning tickets to resolve the open questions - which may require either prototyping, research or discussion.

Use the repo's installed issue-tracker and planning workflow seams where they exist. If those seams have not been tailored yet, leave explicit placeholders for post-install review instead of directing the user to upstream setup commands.

## The Decision Map

The decision map is a single compact Markdown file, one per planning effort, git-tracked alongside the project. It is the canonical artifact — the **whole map is loaded as context into every session**, so it must stay compact.

Assets created during tickets should be linked to from the map, not duplicated within it.

### Structure

Numbered entries ("tickets"), each its own section keyed by its number:

```markdown
## #1: Relational Or Non-Relational Database?

Blocked by: #<ticket-number>, #<ticket-number>
Type: Research | Prototype | Plan review

### Question

<question-here>

### Answer

<answer-here>
```

Each ticket must be sized to one 100K token agent session.

## Ticket Types

There are three types of tickets:

- **Research**: Reading documentation, third-party API's, or local resources like knowledge bases. Creates a markdown summary as an asset. Use this when knowledge outside the current working directory is required.
- **Prototype**: Writing UI or logic code to test a hypothesis, or to explore a design space. Uses the /prototype skill. Creates a prototype as an asset. Use this when "how should it look" or "how should it behave" is the key question.
- **Plan review**: Review the route with the agent. Uses the /plan-review skill to stress-test a chunk before it moves downstream. The default case.

## Fog of war

The map is _deliberately_ incomplete beyond the frontier. Your job is to investigate the frontier, and to resolve tickets in order to push the frontier forward. Push back the fog of war, one node at a time.

At some point, the fog of war should have been pushed back far enough that the path to the finish line is clear. Until then, the work stays in planning. Once the path is clear, no more planning tickets will be required, the decision map can be considered 'done', and the next preferred skills are `to-spec` and then `to-tickets`.

## Invocation

There are two ways this skill can be invoked: **bootstrap** and **resume**.

### Bootstrap

User invokes with a loose idea.

1. Run /plan-review on the current chunk to surface the open decisions and recommended answers.
2. Write a new decision map — mostly fog, frontier identified, trivially-decidable entries resolved inline.
3. Stop. Map-building is one session's work; do not also resolve tickets.

### Resume

User invokes with a path to an existing map and a ticket number.

1. Load the **whole map** as context.
2. Run a session to resolve the ticket, invoking skills as needed. If in doubt, use `/plan-review`.
3. Record what the session resolved in the ticket's body.
4. Add newly-discovered tickets (with correct `blocked_by` edges).
5. Stop.

If the decisions made invalidate other parts of the map, update or delete those nodes.

## Parallelism

The user may choose to run tickets in parallel, so expect other agents to make changes to the map.

## Skipping The Decision Map

Many times, the initial plan review will result in no fog of war. No unresolved tickets. Nothing to do, except implement.

In those situations, you should offer the user the chance to skip the decision map - since the decision map is only needed if multi-session decisions need to be made.

If they skip it because the plan is already clear enough, you should recommend `to-spec` and then `to-tickets` instead of staying in planning. If they skip it because the work is small enough to execute immediately, recommend implementing directly.
