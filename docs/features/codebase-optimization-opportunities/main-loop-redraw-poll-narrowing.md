# Narrow redundant batrs client main-loop redraw and poll work

## Parent

`docs/features/codebase-optimization-opportunities/prd.md`

## What to build

Reduce measurable redundant work in the batrs client main loop when idle or under light activity, without changing visible UI behavior, Command Dispatch behavior, or Session Lifecycle behavior. From the user's perspective, the batrs client should behave the same while doing less unnecessary redraw or polling work where evidence shows that work is redundant.

## Acceptance criteria

- [ ] Visible batrs client behavior remains unchanged for ordinary input, output, redraw, and reconnect flows.
- [ ] The main loop avoids at least one measured source of redundant redraw or polling work without introducing a broader event-model redesign.
- [ ] Command Dispatch and Session Lifecycle ownership boundaries remain unchanged.
- [ ] Any tests or checks added for this work verify behavior rather than internal loop structure.

## Blocked by

- None (can start immediately).

## Status

ready-for-agent
