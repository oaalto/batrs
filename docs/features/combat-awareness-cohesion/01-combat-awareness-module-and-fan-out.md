# 01 — Combat Awareness module + unified app fan-out

## Parent

`prd.md`

## What to build

Promote the combat scan probe state machine into a top-level **Combat Awareness** module with canonical round-header and combat-end matching, `CombatAwarenessEffect` emission, and unchanged probe/gag/snapshot behavior. The application calls Combat Awareness once per incoming line and fans out effects to stats round semantics, `@sc`, `#scan all`, and the `in_battle` automation flag. Combat-ended is reported once per line — no parallel global combat-end path or stats-effect interceptors for lifecycle. Delete the combat-round trigger and strip combat-lifecycle rules from common triggers. Monk guild imports the canonical combat-end constant. Rendering may remain on Combat Awareness for this slice (UI extraction is ticket 02).

## Blocked by

None — can start immediately.

## Status

ready-for-agent

## Acceptance criteria

- [ ] Top-level Combat Awareness module owns probe phase machine, snapshot parsing, canonical `NOT_IN_COMBAT_LINE`, round-header matcher, and `CombatAwarenessEffect` (`RoundStarted`, `CombatEnded`, `SendProbe`, `SendShortScore`).
- [ ] `handle_incoming_line` returns line disposition (visible / gagged) plus emitted effects; round headers and combat-end lines are detected through CA, not duplicate trigger matchers.
- [ ] Application fan-out on `RoundStarted`: stats `StartCombatRound`, send `@sc`, send `#scan all`, `in_battle = true`.
- [ ] Application fan-out on `CombatEnded`: stats `EndCombat`, clear combat/snapshot state, `in_battle = false` — once per line, idempotent.
- [ ] Probe cadence preserved: initial probe on round start; every second non-empty user game command while idle and combat-active.
- [ ] Internal probe lines (echo, captured rows, internal combat-end) stay gagged from scrollback and automation input.
- [ ] Combat-round trigger module deleted; common trigger combat-lifecycle `SetFlag` rules and round `@sc` rule removed.
- [ ] Monk kata interrupt imports canonical combat-end matcher from Combat Awareness (no duplicate literal/regex).
- [ ] Existing combat scan unit tests and application combat integration tests pass (`cargo test`).
