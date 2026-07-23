# 01 — Stats test-only API gating

**Parent:** `prd.md` (Slice A)

> **Note (2026-07-23):** `merge_riftwalker_battle_hp`, `has_nergal_minions`, and `has_nergal_resource_status` no longer live on the stats model — they moved to `src/secondary_status.rs` with Secondary Status extraction (`docs/features/secondary-status-extraction/`). If this slice runs after that extraction, gate or remove the Secondary Status test helpers instead; stats-specific dead-code targets may already be gone.

**What to build:** The stats model exposes no production APIs that exist only for tests. Dead-code suppressions are removed; test-only accessors use the same `#[cfg(test)]` pattern already used for combat-round invocation counters.

**Blocked by:** None — can start immediately

**Status:** ready-for-agent

## Acceptance criteria

- [ ] `#[allow(dead_code)]` removed from `merge_riftwalker_battle_hp`, `has_nergal_minions`, and `has_nergal_resource_status`
- [ ] Those three items gated with `#[cfg(test)]` (or deleted and tests updated to assert state directly)
- [ ] Production code unchanged: `merge_riftwalker_battle_hp_from_listen` remains the live HP-merge path
- [ ] All existing stats module unit tests pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Test seam

- Existing stats preservation tests (no new tests required)
- Riftwalker/Nergal render and test-helper gating targets moved to Secondary Status — see `docs/features/secondary-status-extraction/` if those helpers are no longer on stats
