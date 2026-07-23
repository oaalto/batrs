# Nergal Resource Status ownership

## Status

superseded — absorbed by `docs/features/secondary-status-extraction/prd.md` (grilled 2026-07-23). Do not implement separately; Nergal parsing/gagging cleanup ships in the Secondary Status extraction slice.

## Problem Statement

Nergal Resource Status (Vitae, Potentia, Evolution points) is parsed twice: once in a global core trigger registered for every logged-in player, and again inside the Nergal guild trigger module. Both matchers use the same BatMUD line pattern and emit the same stats effect. When Nergal is selected, both fire on the same line and merge duplicate effects. This splits Nergal domain logic across the global trigger pipeline and the Nergal guild module, risks regex drift on BatMUD text changes, and allows resource status to update and appear in the HUD even when Nergal is not in the player's guild selection.

## Solution

Move Nergal Resource Status parsing and gagging exclusively into the Nergal guild module. Remove the standalone core trigger and its registry entry. Tighten HUD display to guild-selected only. When Nergal is removed from guild selection, clear Nergal resource status and minions from stats immediately. Keep `NergalResourceStatus` storage, the set effect, and HUD rendering in the stats module as the established secondary-status pattern. Surgical dedup only — no new status submodule, no render move, no minion cohesion refactor.

## User Stories

1. As a Nergal player, I want Vitae/Potentia/Evolution status in the HUD, so that I can track resources without reading gag-target game spam.
2. As a Nergal player, I want the resource status line gagged from scrollback, so that automation output stays clean.
3. As a maintainer, I want one regex for the Nergal resource status line, so that BatMUD format changes require one edit.
4. As a maintainer, I want Nergal-specific line handling colocated in the Nergal guild module, so that locality matches the Guild Catalog seam.
5. As a test author, I want Nergal resource parsing tested next to other Nergal trigger tests, so that guild behavior is verified together.
6. As a player with Nergal not selected, I want no Nergal resource line parsed and no Nergal HUD rows shown, so that unrelated play stays clean.
7. As a player who deselects Nergal mid-session, I want Nergal HUD rows to disappear immediately, so that stale guild data does not linger on screen.
8. As a player who deselects Nergal mid-session, I want Nergal stats cleared from memory, so that re-selecting Nergal does not flash outdated values before the next game line.
9. As a maintainer removing the core trigger, I want no double stat effects when Nergal is selected, so that trigger merging stays predictable.
10. As a maintainer, I want `StatsEffect::SetNergalResourceStatus` to remain the cross-seam signal into stats, so that the stats/HUD boundary stays consistent with other guild secondary status.
11. As a test author, I want an integration test proving resource lines are ignored without Nergal selected, so that guild-gating regressions are caught.
12. As a test author, I want an integration test proving deselect clears Nergal resource status and minions, so that HUD lifecycle is verified end-to-end.
13. As a maintainer, I want strict field-order rejection preserved, so that malformed or reordered BatMUD text does not corrupt stats.
14. As a player, I want minion rows and resource status rows to still render together when Nergal is selected, so that deepening does not regress the combined Nergal HUD block.
15. As a maintainer, I want wiki and domain docs to reflect guild-owned parsing, so that future readers do not look for a global core trigger.

## Implementation Decisions

### Parsing ownership

- Delete the standalone core Nergal resource status trigger module entirely; remove its registration from the global core trigger list.
- The Nergal guild trigger module remains the sole owner of resource status line matching, gagging, and `SetNergalResourceStatus` emission.
- Parsing runs only when Nergal is in the player's guild selection (already true for guild triggers; removing the core trigger closes the guild-agnostic path).

### Stats and type ownership

- Keep `NergalResourceStatus` defined in the stats module alongside other guild secondary-status types (`NergalMinion`, soul companion, riftwalker entity, tzarakk mount).
- Keep `StatsEffect::SetNergalResourceStatus` as the cross-seam update signal; triggers emit it, stats applies it.
- Add `clear_nergal_resource_status()` on stats (sets stored resource status to empty); no new clear effect variant.

### HUD gate

- Change Nergal HUD support check to guild-selected only: show Nergal status rows when `GuildKey::Nergal` is in guild selection.
- Remove fallback OR conditions on `has_nergal_minions()` and `has_nergal_resource_status()` for HUD visibility.

### Clear on deselect

- In guild selection application, when the new selection no longer includes Nergal, call `clear_nergal_minions()` and `clear_nergal_resource_status()` directly on stats.
- Use direct method calls, not a new stats effect variant — deselect is application lifecycle, not trigger parsing. Existing `ClearNergalMinions` effect remains for unsummon trigger paths only.

### Scope boundaries (explicit non-goals for this change)

- Do not extract a new Nergal status submodule.
- Do not move render span logic out of stats.
- Do not refactor minion + resource + unsummon into a deeper combined interface.
- Do not add a re-export shim where the deleted core trigger lived.

### Documentation

- `CONTEXT.md` Nergal Status section updated to record guild-owned parsing, stats-owned storage/render, guild-gated HUD, and clear-on-deselect.
- Update engineering wiki concept page and path-map sources to remove the deleted core trigger reference.

## Testing Decisions

### Testing seams (proposed — primary verification points)

1. **Nergal guild trigger tests** (highest seam for parsing) — incoming resource status line gagged, correct `SetNergalResourceStatus` effect, strict field order rejected. Prior art already exists in guild trigger tests; absorb any unique cases from the deleted core trigger tests.
2. **Application integration tests** (lifecycle seam) — with Nergal selected: line gagged, stats updated, HUD renders. Without Nergal selected: line not gagged, stats unchanged. On deselect after populated stats: resource status and minions cleared, HUD hidden.

Prefer these two seams over new test infrastructure. No new test-only modules.

### Good tests (external behavior)

- Resource status line with Nergal selected → gagged from output, stats show Vitae/Potentia/EP, HUD includes status row.
- Resource status line without Nergal selected → visible in output (not gagged), stats unchanged.
- Reordered/malformed resource line → not gagged, stats unchanged.
- Deselect Nergal after resource status and minions populated → both cleared, HUD no longer shows Nergal rows.
- Re-select Nergal after deselect → HUD empty until next valid resource/minion line.

### Prior art

- Guild trigger resource status tests (gag, effect, field order).
- Application integration test for gagged resource line with Nergal selected.
- Application unsummon test for minion clear with Nergal selected.
- Deleted core trigger unit tests — migrate non-duplicative cases into guild trigger tests, then delete.

### Regression guard

- Potentia/vitae full notices, unsummon minion clear, minion upsert, and ceremony automation in Nergal guild triggers unchanged.

## Out of Scope

- Changing Nergal ceremony commands or potentia/vitae full echo notices.
- Nergal minion slot logic redesign.
- Persisting Nergal resource status to player config.
- Non-Nergal guild resource HUD patterns.
- Extracting shared pattern modules or deepening minion/resource render cohesion.
- Moving `NergalResourceStatus` type ownership out of stats.

## Further Notes

- Recommendation strength: **Strong** — smallest diff with clear duplication removal.
- Grilled decisions locked 2026-07-23; all open questions from initial exploration resolved.
- Connect Command fresh-session reset already clears broader app state; no separate Nergal clear needed beyond guild deselect path unless reconnect testing reveals a gap.
