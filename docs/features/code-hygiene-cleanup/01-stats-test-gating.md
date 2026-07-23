# 01 — Stats test-only API gating

**Parent:** `prd.md` (Slice A)

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

- Existing stats preservation and riftwalker/nergal render tests (no new tests required)
