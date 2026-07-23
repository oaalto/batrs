---
title: Secondary Status
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/secondary_status.rs
  - src/app/mod.rs
  - src/triggers/mod.rs
  - src/guilds/animist/triggers.rs
  - src/guilds/riftwalker/triggers.rs
  - src/guilds/tzarakk/triggers.rs
  - src/guilds/nergal/triggers.rs
---

# Secondary Status

## Summary

Secondary Status is the guild-specific HUD row band rendered below the main stats line. It covers Animist soul companion, Riftwalker entity, Tzarakk mount, and Nergal resource status plus minions.

The application applies `SecondaryStatusEffect` values from guild triggers separately from stats effects and calls a single `render_lines(width, guild_selection)` at draw time.

## Module boundary

`src/secondary_status.rs` owns:

- Guild HUD state types (`SoulCompanionStatus`, `TzarakkMountStatus`, `RiftwalkerEntityStatus`, `NergalMinion`, `NergalResourceStatus`)
- `SecondaryStatusEffect` enum and `SecondaryStatus::apply_effect`
- Guild-selected rendering via `render_lines(width, &GuildSelection)` and per-domain render helpers
- Lifecycle: `sync_guild_selection` clears stored state for deselected guilds; Connect Command resets via `FreshSessionReset::SecondaryStatus`

Secondary Status does **not** own prompt parsing, short-score diffs, recovery brackets, combat-round semantics (stats), combat scan snapshot rows (Combat Awareness), or guild trigger parsing.

Guild trigger modules emit `SecondaryStatusEffect` through `TriggerEffects.secondary_status`; the application applies them in `apply_secondary_status_effects` (`src/app/mod.rs`).

## SecondaryStatusEffect and trigger path

```rust
enum SecondaryStatusEffect {
    SetSoulCompanion { percent, description },
    SetTzarakkMountStatus { name, percent, description },
    ClearTzarakkMountStatus,
    MergeRiftwalkerBattleLabel(String),
    MergeRiftwalkerBattleHpFromListen { hp, paren_inside, brackets },
    ClearRiftwalkerEntityStatus,
    UpsertNergalMinion(NergalMinion),
    SetNergalResourceStatus(NergalResourceStatus),
    ClearNergalMinions,
}
```

| Domain | Guild trigger module | Render when selected |
| --- | --- | --- |
| Animist soul | `src/guilds/animist/triggers.rs` | `render_soul_inline` |
| Riftwalker entity | `src/guilds/riftwalker/triggers.rs` | `render_riftwalker_entity_inline` |
| Tzarakk mount | `src/guilds/tzarakk/triggers.rs` | `render_tzarakk_mount_inline` |
| Nergal minions + resources | `src/guilds/nergal/triggers.rs` | `render_nergal_status_lines` |

## UI rendering seam

A guild HUD row renders only when that guild's `GuildKey` is in the player's guild selection. Deselecting a guild calls `sync_guild_selection`, which clears its stored secondary status immediately. The draw path does not apply per-guild visibility guards beyond `render_lines`.

`ViewModel.secondary_status_lines` is populated from `SecondaryStatus::render_lines` when stats are shown (`src/app/mod.rs`).

## Verified Facts

- Guild HUD state and rendering moved out of stats; stats retains prompt, short score, recovery brackets, and combat-round diff semantics only (`CONTEXT.md`, `src/stats.rs`).
- `TriggerEffects` carries separate `stats` and `secondary_status` vectors; builder helpers `.stat()` and `.secondary_status()` mirror each other (`src/triggers/mod.rs`).
- Connect Command fresh-session manifest includes `FreshSessionReset::SecondaryStatus` (`src/app/session_lifecycle/fresh_session.rs`, `src/app/mod.rs`).
- All four guild HUD domains use guild-selected-only visibility; no fallback on stored data when the guild is not selected (`src/secondary_status.rs` tests).

## Related

- [Nergal Status](nergal-status.md)
- [Session Lifecycle](session-lifecycle.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Secondary Status section
