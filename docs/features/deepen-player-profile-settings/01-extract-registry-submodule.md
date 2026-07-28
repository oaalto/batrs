# 01 — Extract registry submodule

## Parent

`prd.md`

## What to build

Replace the `player_profile` monolith with a directory module and move all setting registry concerns into `registry.rs`. The registry submodule owns every key constant, registry enum (`SettingKind`, `SettingSlot`, `PersistSlot`, `AutomationExport`), `SettingDefinition`, `SETTINGS_DEFS`, and `definition_for_key`.

`mod.rs` becomes the module root: it declares submodules, re-exports the three public key constants (`TZARAKK_MOUNT_KEY`, `SABRE_WEAPON_KEY`, `RIFTWALKER_ENTITY_LABEL_KEYS`), and temporarily retains all other logic (persistence dispatch, normalization, runtime, TOML orchestration, tests) until later tickets peel layers off. Delete the old `player_profile.rs` file once the directory module resolves.

Behavior must not change: all 11 existing tests pass with no assertion or fixture edits.

## Acceptance criteria

- [ ] `src/player_profile/` directory exists with `mod.rs` and `registry.rs`
- [ ] `registry.rs` owns all `*_KEY` constants, registry enums, `SettingDefinition`, `SETTINGS_DEFS`, and `definition_for_key`
- [ ] `mod.rs` re-exports only the three public key constants; no new `pub` items beyond what existed pre-split
- [ ] `src/player_profile.rs` removed; `mod player_profile` in `main.rs` resolves to the directory module
- [ ] `config` and `app` callers require no import or signature changes
- [ ] All 11 `player_profile` tests pass unchanged
- [ ] `cargo test --all-targets --all-features` passes

## Blocked by

None — can start immediately.

## Status

ready-for-agent
