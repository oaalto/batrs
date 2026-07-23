---
title: Combat Awareness
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/combat_awareness.rs
  - src/ui/mod.rs
  - src/app/mod.rs
---

# Combat Awareness

## Summary

Combat Awareness is batrs' interpretation of whether the player is currently in BatMUD combat. A **Combat Scan Snapshot** is the latest observed set of combatants and health from a completed `#scan all` probe; each completed scan replaces the previous snapshot.

The application calls Combat Awareness once per incoming line and fans out `CombatAwarenessEffect` values to stats, automation, outbound commands, and UI snapshot data.

## Module boundary

`src/combat_awareness.rs` owns:

- Canonical round-header matching (`is_round_header`, `ROUND_HEADER_REGEX`)
- Canonical combat-end line (`NOT_IN_COMBAT_LINE`)
- Probe phase machine and `PROBE_COMMAND` (`#scan all`)
- Snapshot parsing and storage (`CombatScanRow`, `CombatAwareness::snapshot()`)
- Combat-active state (`CombatAwareness::is_active()`)
- `CombatAwarenessEffect` emission from `handle_incoming_line` and `observe_user_game_command`

Combat Awareness does **not** own short-score diff accumulation (`combat_round_active` stays in stats), ratatui rendering, automation flag mutation, or guild-specific kata interrupt behavior.

Triggers and guild modules import canonical constants/matchers from Combat Awareness rather than redefining them (for example monk kata interrupt uses `NOT_IN_COMBAT_LINE`; common lich drain uses the same constant).

## CombatAwarenessEffect and app fan-out

```rust
enum CombatAwarenessEffect {
    RoundStarted,
    CombatEnded,
    SendProbe,      // #scan all
    SendShortScore, // @sc
}
```

`src/app/mod.rs` maps effects in `apply_combat_awareness_effects` — one CA call per line, one fan-out:

| Effect | Fan-out |
| --- | --- |
| `RoundStarted` | stats `StartCombatRound`; `in_battle = true` |
| `CombatEnded` | stats `EndCombat`; `in_battle = false` |
| `SendShortScore` | send `@sc` |
| `SendProbe` | send `#scan all` |

Round header emits `RoundStarted`, `SendShortScore`, and `SendProbe` together. Combat ends on `NOT_IN_COMBAT_LINE` with `CombatEnded` only. Lifecycle is reported once per line through this path; there is no parallel `combat_round` trigger or common-trigger `in_battle` lifecycle path.

## UI rendering seam

Combat Awareness exposes snapshot data only. The UI layer renders combat status rows via `ui::render_combat_status_lines` from `CombatAwareness::is_active()` and `CombatAwareness::snapshot()`; the application passes rendered lines into the view model at draw time (`src/app/mod.rs`).

## Verified Facts

- Combat begins on a round header matching `^[\*]+ Round .* [\*]+$`; ends on `You are not in combat right now.` (`NOT_IN_COMBAT_LINE`).
- Probe rows are gagged from scrollback and automation; internal probe responses on combat end are gagged when probe phase is active.
- Scan rows capture name, condition phrase, and health percent; each completed scan replaces the prior snapshot (`CONTEXT.md`, `combat_awareness.rs`).
- Next round header while capturing completes the prior scan into the snapshot (`src/combat_awareness.rs`, `handle_incoming_line`).

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Combat Awareness section
