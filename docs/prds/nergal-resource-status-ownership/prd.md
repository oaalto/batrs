# Nergal Resource Status ownership

## Status

draft — initial exploration for grilling

## Problem Statement

Nergal Resource Status (Vitae, Potentia, Evolution points) is parsed twice: once in a core trigger (`triggers/nergal_resource_status.rs`, registered in `CORE_TRIGGERS`) and again inside `guilds/nergal/triggers.rs`. Both use the same regex pattern and emit `StatsEffect::SetNergalResourceStatus`. The core trigger runs for every logged-in player regardless of guild selection; the guild trigger runs when Nergal is selected. This duplicates interface surface, risks pattern drift on BatMUD text changes, and splits Nergal domain logic across the global trigger pipeline and the Nergal guild module.

## Initial exploration

| Location | Role |
| --- | --- |
| `triggers/nergal_resource_status.rs` | Standalone trigger in `CORE_TRIGGERS`; gag + `SetNergalResourceStatus` |
| `guilds/nergal/triggers.rs` | Same `RESOURCE_STATUS` regex; also handles minions, unsummon, potentia/vitae full notices, ceremony automation |
| `stats.rs` | `NergalResourceStatus` type, `set_nergal_resource_status`, `render_nergal_status_lines` |
| `app/mod.rs` test | `nergal_resource_status_line_is_gagged_and_updates_stats` — exercises core path with Nergal selected |
| `CONTEXT.md` | Defines Nergal Resource Status concept |

**Regex (identical in both files):**

`^::\.\.:\. \[Vitae: ([0-9]+)/([0-9]+)  Potentia: ([0-9]+)/([0-9]+), Evolution points: ([0-9]+)\]$`

**Trigger order (`triggers/mod.rs`):** Guild triggers run first, then spell vocals, then `COMMON_TRIGGERS` + `CORE_TRIGGERS`. When Nergal is selected, the guild trigger handles the line first; core trigger still runs afterward on the same line (effects merge).

**Rendering:** Resource status line appears below Nergal minion rows via `stats.render_nergal_status_lines`, shown in HUD when Nergal selected or status/minions present.

## Solution (proposed direction)

Move Nergal Resource Status parsing exclusively into the Nergal guild module (or a Nergal-owned submodule). Remove `nergal_resource_status` from `CORE_TRIGGERS`. Optionally deepen Nergal triggers so minion status, resource status, and unsummon share one interface within the guild module.

Grilling should decide: is it acceptable that resource status only updates when Nergal is in the player's guild selection, or must it work guild-agnostically (e.g. preview while browsing)?

## User Stories

1. As a Nergal player, I want Vitae/Potentia/Evolution status in the HUD, so that I can track resources without reading gag-target game spam.
2. As a maintainer, I want one regex for the Nergal resource status line, so that BatMUD format changes require one edit.
3. As a maintainer, I want Nergal-specific triggers colocated in the Nergal guild module, so that locality matches the Guild Catalog seam.
4. As a test author, I want Nergal resource parsing tested next to other Nergal trigger tests, so that guild behavior is verified together.
5. As a player with Nergal not selected, I want no Nergal resource HUD noise, so that unrelated play stays clean.
6. As a maintainer removing the core trigger, I want no double-stat effects when Nergal is selected, so that trigger merging stays predictable.

## Open questions (for grilling)

1. **Guild gate:** Must resource status update only when `GuildKey::Nergal` is selected, or always when the line appears (current core behavior)?
2. **HUD gate:** `app/mod.rs` shows Nergal status when guild selected OR `has_nergal_resource_status()` — keep OR logic or tighten to guild-only?
3. **Type ownership:** Should `NergalResourceStatus` move from `stats.rs` toward Nergal guild types, with stats holding only render state?
4. **Minion cohesion:** Deepen further — one Nergal status module covering minions + resources + render spans?
5. **Deletion:** Delete `triggers/nergal_resource_status.rs` entirely vs. re-export from Nergal for registration convenience?

## Implementation Decisions (tentative)

- Remove `nergal_resource_status::trigger` from `CORE_TRIGGERS` in `triggers/mod.rs`.
- Keep `StatsEffect::SetNergalResourceStatus` as the cross-seam signal to stats/HUD unless grilling chooses a narrower path.
- Consolidate regex into one `lazy_static` in Nergal guild triggers (or shared `guilds/nergal/patterns.rs` if splitting files).
- Migrate core trigger unit tests into `guilds/nergal/triggers.rs` tests.
- Update `app/mod.rs` integration test if guild selection becomes mandatory for the behavior under test.

## Testing Decisions

- **Good tests:** Incoming resource status line → gagged from output, stats HUD shows Vitae/Potentia/EP; strict field order rejected.
- **Prior art:** `triggers/nergal_resource_status.rs` tests; `guilds/nergal/triggers.rs` tests; `app/mod.rs` `nergal_resource_status_line_is_gagged_and_updates_stats`.
- **Regression:** Full/empty potentia and vitae notices in Nergal triggers still fire; minion upsert unchanged.
- **Edge:** Player deselects Nergal mid-session — should resource status clear from HUD?

## Out of Scope

- Changing Nergal ceremony commands or potentia/vitae full echo notices.
- Nergal minion slot logic redesign.
- Persisting Nergal resource status to player config.
- Non-Nergal guild resource HUD patterns.

## Further Notes

- Recommendation strength: **Strong** — smallest diff candidate; clear duplication.
- Quick win: delete core trigger + registry line; verify guild path covers all call sites.
- `CONTEXT.md` already names Nergal Resource Status — update if ownership seam moves.
