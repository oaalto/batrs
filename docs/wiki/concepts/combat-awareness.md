---
title: Combat Awareness
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/triggers/combat_round.rs
  - src/app/combat_scan.rs
---

# Combat Awareness

## Summary

Combat Awareness is batrs' interpretation of whether the player is currently in BatMUD combat. A **Combat Scan Snapshot** is the latest observed set of combatants and health from a completed `#scan all` probe; each completed scan replaces the previous snapshot.

## Verified Facts

- Combat begins when a round header line matches `^[\*]+ Round .* [\*]+$`; ends on the line `You are not in combat right now.` (`src/triggers/combat_round.rs`).
- Stats effects: `StartCombatRound` and `EndCombat` applied via `src/stats.rs`.
- Combat scan probe: `PROBE_COMMAND` is `#scan all`; parsing and snapshot state live in `src/app/combat_scan.rs`.
- Scan rows capture name, condition phrase, and health percent; each completed scan replaces the prior snapshot (`CONTEXT.md`, `combat_scan.rs`).
- UI renders combat status above stats (`src/ui/mod.rs`).

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Combat Awareness section
