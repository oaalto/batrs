# 02 — Nergal guild-gated HUD lifecycle

## Status

superseded — absorbed by `docs/features/secondary-status-extraction/` (ticket 01, grilled 2026-07-23). Guild-gated HUD and clear-on-deselect now live in Secondary Status (`sync_guild_selection`); do not implement separately.

## Parent

`prd.md`

## What to build

Complete the grilled Nergal Resource Status ownership: HUD shows only when Nergal is selected, and deselecting Nergal clears resource status and minions from stats. A player without Nergal sees the resource line in scrollback (not gagged) with no stats update; a player who removes Nergal mid-session loses Nergal HUD rows immediately without stale data reappearing on re-select until new game lines arrive.

## Acceptance criteria

- [ ] Nergal HUD rows shown only when `GuildKey::Nergal` is in guild selection — no fallback on stored minion or resource status presence
- [ ] `clear_nergal_resource_status()` added on stats; called with `clear_nergal_minions()` when guild selection no longer includes Nergal
- [ ] Clear-on-deselect uses direct stats method calls in guild selection application — no new clear stats effect variant
- [ ] Integration test: Nergal selected → resource line gagged, stats updated, HUD renders status
- [ ] Integration test: Nergal not selected → resource line visible in output, stats unchanged
- [ ] Integration test: deselect Nergal after populated resource status and minions → both cleared, HUD hidden
- [ ] Engineering wiki concept page and path-map updated to reflect guild-owned parsing (no core trigger source)
- [ ] `cargo test` passes

## Blocked by

- 01 — Remove core Nergal resource status trigger

**Status:** ready-for-agent
