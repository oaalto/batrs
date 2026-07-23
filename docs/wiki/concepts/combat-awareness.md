---
title: Combat Awareness
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/combat_awareness.rs
  - src/app/mod.rs
---

# Combat Awareness

## Summary

Combat Awareness is batrs' interpretation of whether the player is currently in BatMUD combat. A **Combat Scan Snapshot** is the latest observed set of combatants and health from a completed `#scan all` probe; each completed scan replaces the previous snapshot.

The application calls Combat Awareness once per incoming line and fans out `CombatAwarenessEffect` values to stats, automation, outbound commands, and UI snapshot data.

## Verified Facts

- Module: `src/combat_awareness.rs` — probe state machine, round-header regex (`is_round_header`), `NOT_IN_COMBAT_LINE`, `PROBE_COMMAND` (`#scan all`), snapshot parsing, and `CombatAwarenessEffect` emission.
- Combat begins on a round header matching `^[\*]+ Round .* [\*]+$`; ends on `You are not in combat right now.` (`NOT_IN_COMBAT_LINE`). Lifecycle is reported once per line through Combat Awareness → app fan-out; there is no parallel `combat_round` trigger or common-trigger `in_battle` lifecycle path.
- Round header emits `RoundStarted`, `SendShortScore`, and `SendProbe`. App fan-out (`src/app/mod.rs`): `RoundStarted` → stats `StartCombatRound` + `in_battle = true`; `SendShortScore` → `@sc`; `SendProbe` → `#scan all`; `CombatEnded` → stats `EndCombat` + `in_battle = false`.
- Probe rows are gagged from scrollback and automation; internal probe responses on combat end are gagged when probe phase is active.
- Scan rows capture name, condition phrase, and health percent; each completed scan replaces the prior snapshot (`CONTEXT.md`, `combat_awareness.rs`).
- UI renders combat status above stats via `CombatAwareness::render_lines` (`src/app/mod.rs` → `src/ui/mod.rs`).

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Combat Awareness section
