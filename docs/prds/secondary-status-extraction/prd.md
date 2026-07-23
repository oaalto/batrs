# Secondary status extraction from Stats

## Status

draft — initial exploration for grilling

## Problem Statement

Guild-specific HUD rows (Animist soul, Riftwalker entity, Tzarakk mount, Nergal minions + resource status) are stored, updated, and rendered inside `stats.rs`, while `app/mod.rs` `draw` assembles `secondary_status_lines` with guild-selection guards and calls multiple `render_*` methods. Stats is shallow for this concern: its interface includes many unrelated effects (short score, prompt, recovery brackets) plus four guild HUD domains. UI layout logic (which rows appear, ordering, width) leaks into the application shell.

Combat status was recently split above stats (`combat_status_lines`); secondary status below stats repeats the pattern incompletely — combat scan rendering left its state module, guild rows did not.

## Initial exploration

**HUD layout (top to bottom, when logged in):**

1. `combat_status_lines` — from `CombatScanState.render_lines` (not Stats)
2. `stats_line` — `stats.render_inline()` (HP/SP/EP/exp + combat diffs)
3. `secondary_status_lines` — assembled in `app/mod.rs`:
   - Soul if Animist selected OR `has_soul_companion_status()`
   - Riftwalker entity if selected OR `has_riftwalker_entity_status()`
   - Tzarakk mount if selected OR `has_tzarakk_mount_status()`
   - Nergal lines if selected OR minions OR resource status

**Stats module holds:**

| Domain | State fields | StatsEffect variants | Render method |
| --- | --- | --- | --- |
| Tzarakk mount | `tzarakk_mount: Option<...>` | Set/Clear | `render_tzarakk_mount_inline` |
| Riftwalker entity | `riftwalker_entity: Option<...>` | Merge/Clear | `render_riftwalker_entity_inline` |
| Nergal | `nergal_minions`, `nergal_resource_status` | Upsert/Clear/Set | `render_nergal_status_lines` |
| Animist soul | (soul fields) | (soul effects) | `render_soul_inline` |

**Preservation behavior:** Prompt and short-score updates intentionally preserve secondary status (many tests in `stats.rs`).

**Trigger path:** Guild triggers and core triggers emit `StatsEffect` variants; `app/mod.rs` `apply_stats_effects` applies to `Stats`.

## Solution (proposed direction)

Extract a **secondary status** module (or **guild HUD** module) with a small interface: apply status effects, render lines for a given `GuildSelection` + observed state, query whether rows would show. Stats keeps short-score/combat-round/prompt concerns. Triggers might emit into the new module directly or via redirected `StatsEffect` variants — grilling decides.

Alternative shallow fix: only move `draw` assembly into a helper — less leverage, smaller diff.

## User Stories

1. As a player, I want guild-specific HUD rows below my main stats line, so that mount/minion/soul info stays visible during play.
2. As a maintainer adding a new guild HUD row, I want one module to extend, so that `stats.rs` does not grow further.
3. As a maintainer, I want guild-selection gating in one place, so that `draw` does not list per-guild conditions.
4. As a test author, I want to render secondary status given guild selection and effect history, so that UI tests do not need full Stats.
5. As a player, I want short-score and prompt updates not to clear my mount/minion display, so that HUD stays stable within a session.
6. As a maintainer, I want secondary status cleared on Connect Command fresh session, so that reconnect does not show stale guild HUD.

## Open questions (for grilling)

1. **Depth vs. move:** Full extraction from Stats, or only extract render+gate assembly leaving state in Stats?
2. **Effect routing:** Keep `StatsEffect::SetTzarakkMountStatus` etc., or introduce `HudEffect` / per-guild effect enums?
3. **Soul exception:** Animist soul is guild-related but not in Guild Catalog the same way — same module or separate?
4. **Ordering:** Fixed order (soul → riftwalker → tzarakk → nergal) — configurable or hardcoded?
5. **Width wrapping:** Nergal minion multi-line wrapping stays with Nergal renderer — module owns width param?
6. **Relation to Nergal PRD:** If Nergal resource moves to guild module, does secondary status module compose Nergal render output?

## Implementation Decisions (tentative)

- Mirror combat_status pattern: dedicated state + `render_lines(width, guild_selection, fallback_observed_state)`.
- `app/mod.rs` `draw` calls one `secondary_status.render(...)` instead of four blocks.
- Preserve OR guards (show if guild selected OR data already observed) unless grilling tightens.
- Stats shedding: move types `TzarakkMountStatus`, `RiftwalkerEntityStatus`, `NergalMinion`, `NergalResourceStatus` with their render helpers.
- Connect Command reset clears secondary status module in session lifecycle manifest.

## Testing Decisions

- **Good tests:** Render empty when no guild and no data; render mount when Tzarakk selected; render when data exists but guild deselected (if keeping OR semantics); prompt update preserves rows.
- **Prior art:** `stats.rs` preservation tests; `app/mod.rs` draw-related integration tests.
- **Avoid:** Snapshot-testing ratatui spans unless already established pattern.

## Out of Scope

- Main stats line (short score, recovery brackets, combat diffs).
- Combat scan rows (separate PRD: combat-awareness-cohesion).
- Guild dialog profile inputs (mount name, sabre, riftwalker labels).
- New guild HUD types not already in Stats.

## Further Notes

- Recommendation strength: **Worth exploring** — payoff grows with each new guild HUD row.
- Depends conceptually on decisions in nergal-resource-status-ownership PRD but can proceed independently.
- `ui/mod.rs` `ViewModel.secondary_status_lines` name may stay; producer changes.
