# 01 — Dispatch consolidation

## Parent

`prd.md`

## What to build

Consolidate all five player-profile settings dispatch surfaces into inherent methods on `PersistSlot` and `SettingSlot`, colocated in the persistence module. Delete the free functions (`read_persist`, `write_persist`, `read_known_slot`, `write_known_slot`, `write_guild_dialog_slot`). Call sites invoke enum methods directly.

**PersistSlot** gains `read(table, key)` and `write(table, key, value, sparse_when_default)`. Non-`Extra` arms ignore `key` and `sparse_when_default`; the `Extra` arm retains sparse remove/insert behavior for `is_lich`.

**SettingSlot** gains `read(settings)`, `write(settings, value)` (with bool coercion for `IsLich`), and `write_guild_dialog(defaults, value)` (`Rig` and `IsLich` are no-op arms). Guild-dialog callers continue filtering registry rows on `guild_dialog` before calling.

Update all call sites: TOML orchestration uses `definition.persist.*`; runtime profile construction and guild-dialog defaults use `definition.slot.*`; automation export uses `definition.slot.read`. No thin wrapper functions retained.

Preserve dual-enum riftwalker shape (flat `PersistSlot` variants vs `SettingSlot::RiftwalkerEntity(usize)`). Registry module stays enum + const table only. Runtime module loses slot dispatch free functions and stays under the 200-line production cap.

Prerequisite: deepen-player-profile-settings slice merged (module split landed).

## Acceptance criteria

- [ ] `PersistSlot::read` and `PersistSlot::write` live in the persistence module with behavior identical to the deleted free functions
- [ ] `SettingSlot::read`, `SettingSlot::write`, and `SettingSlot::write_guild_dialog` live in the persistence module with behavior identical to the deleted free functions
- [ ] All five free dispatch functions are deleted; no thin delegating wrappers remain
- [ ] TOML orchestration, runtime profile construction, guild-dialog defaults, and automation export call enum methods directly
- [ ] Sparse-extra write behavior for `is_lich` unchanged
- [ ] Riftwalker four-key → `[String; 4]` mapping unchanged
- [ ] Guild dialog field subset unchanged (`rig` and `is_lich` excluded from dialog defaults)
- [ ] Registry module has no dispatch impls and no new imports of runtime or persistence types
- [ ] Runtime module production code under 200 lines
- [ ] No new `pub` exports; no proc-macros; no public API changes
- [ ] All 11 existing `player_profile` tests pass unchanged
- [ ] `cargo test --all-targets --all-features` passes; workflow gates (format, clippy) pass

## Blocked by

None — can start immediately (prerequisite: deepen-player-profile-settings slice merged).

## Status

done
