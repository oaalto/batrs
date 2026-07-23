# 03 — Docs + stale reference sweep

## Parent

`prd.md`

## What to build

Update engineering wiki and verify the codebase reflects the final Combat Awareness cohesion shape: single module boundary, effect fan-out, UI-owned rendering. Confirm no stale duplicate combat-end literals or round-header regex outside Combat Awareness (monk and other guild code use imported constants/matchers).

## Blocked by

- [02 — UI owns combat status rendering](02-ui-combat-status-rendering.md)

## Status

ready-for-agent

## Acceptance criteria

- [ ] `docs/wiki/concepts/combat-awareness.md` updated: module ownership, `CombatAwarenessEffect`, app fan-out, UI rendering seam.
- [ ] Repo search finds no duplicate `NOT_IN_COMBAT` string literals or round-header regex outside Combat Awareness exports (monk interrupt uses imported matcher).
- [ ] Explicit regression test for single combat-end fan-out per line exists (add if not already covered by ticket 01).
- [ ] Wiki log entry recorded per project documentation rules if wiki content changed.
