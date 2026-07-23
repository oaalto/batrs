# Agentic Development Usage

This file is the human-facing entry point for daily agentic development work.

## Daily Skill Use

- Unfamiliar area: `/zoom-out`
- Design discussion without doc updates: `/grilling`
- Design discussion with doc updates: `/grill-with-docs`
- Planning artifact: `/to-spec`
- Implementation slices: `/to-tickets`
- Implementation: `/tdd`
- Debugging: `/diagnosing-bugs`
- Architecture review: `/improve-codebase-architecture`
- Wiki query/update/lint: `/wiki`
- Code review: `/review`

## Artifact Rules

- `CONTEXT.md` is for domain language only.
- ADRs are for durable hard-to-reverse trade-off decisions.
- `docs/wiki/` is agent-maintained engineering memory.
- PRDs are historical for behavior claims unless verified against code/tests.
- Agents must read `docs/agent-commands.md` before code-changing work.
- Agents must verify wiki claims against live sources before relying on them for implementation.

## Human Review Checklist

- Did this work change durable domain language? Check `CONTEXT.md`.
- Did this work make a hard-to-reverse trade-off decision? Check ADRs.
- Did this work create reusable subsystem, workflow, trap, or debugging knowledge? Check `docs/wiki/`.
- Did the agent run or explicitly report the relevant validation gates?

Scoped catalog rules live under `.agents/rules/`; see the **Rules index** in your host file for when to load each rule.
