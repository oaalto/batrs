# 03 — Extract runtime and finalize module split

## Parent

`prd.md`

## What to build

Complete the mechanical split by moving runtime concerns out of `mod.rs` into `runtime.rs`, leaving `mod.rs` as the public orchestration layer plus the full test module.

**`runtime.rs`** owns: `KnownProfileSettings`, `GuildDialogProfileDefaults`, `runtime_profile_from_parts`, `KnownProfileSettings::from_user_settings`, `GuildDialogProfileDefaults::from_settings`, automation export functions, `default_riftwalker_entity_labels`, `setting_value`, `non_empty`, and `PlayerRuntimeProfile` / `InterpretedPlayerProfile` if they belong with runtime construction (public structs re-exported from `mod.rs`).

**`mod.rs`** retains: `interpret_player_toml`, `settings_entries_for_editor`, `settings_table_from_entries`, `user_settings_from_player`, TOML orchestration (`normalize_player_toml`, `normalize_settings_entries`, `settings_table_from_normalized_entries`, `normalize_player_guilds`), public re-exports, and all 11 tests in `#[cfg(test)] mod tests`. Tests that reach internals use explicit `super::registry::` / `super::runtime::` imports per PRD.

Verify the finished layout matches the PRD: five production files, each under 200 lines excluding `#[cfg(test)]` blocks. If `mod.rs` production code exceeds the cap, document the count in the PR — do not add a `bridge.rs` submodule (out of scope for this slice).

Confirm `config` and `app` callers are unchanged.

## Acceptance criteria

- [ ] `runtime.rs` owns runtime structs, profile construction, and automation/guild-dialog export logic
- [ ] `mod.rs` owns only public API entry points, TOML orchestration, re-exports, and all 11 tests
- [ ] Public API identical to pre-split: same symbols, signatures, and visibility for `config` and `app` consumers
- [ ] No new `pub` items introduced during extraction
- [ ] Each production file is under 200 lines (`#[cfg(test)]` excluded); line counts noted if any file is close to the cap
- [ ] `normalization.rs` still has no dependency on persistence or runtime
- [ ] All 11 tests pass with zero logic changes; test imports updated only as needed for submodule paths
- [ ] `cargo test --all-targets --all-features` passes; workflow gates (format, clippy) pass

## Blocked by

- 02 — Extract normalization and persistence dispatch

## Status

done
