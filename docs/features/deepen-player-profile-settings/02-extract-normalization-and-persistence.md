# 02 — Extract normalization and persistence dispatch

## Parent

`prd.md`

## What to build

Peel the next two dependency-free layers off `mod.rs` into dedicated submodules.

**`normalization.rs`** owns `normalized_setting_value` only — a pure function taking a `SettingDefinition` and raw string, returning the normalized string. It may depend on `registry` and config's truthy helper; it must not import persistence, runtime, or TOML types.

**`persistence.rs`** owns the five dispatch functions unchanged in behavior: `read_persist`, `write_persist`, `read_known_slot`, `write_known_slot`, and `write_guild_dialog_slot`. Match tables move as-is; no dispatch deduplication. Expose what `mod.rs` and `runtime` need via `pub(crate)`.

Wire `mod.rs` to call through the new submodules. Runtime, TOML orchestration, and all 11 tests remain in `mod.rs` for now.

## Acceptance criteria

- [ ] `normalization.rs` contains only `normalized_setting_value` with no imports from persistence or runtime
- [ ] `persistence.rs` contains all five dispatch functions with behavior identical to pre-extraction
- [ ] Dispatch match tables are moved, not deduplicated or rewritten
- [ ] `mod.rs` orchestration calls normalization and persistence through submodule paths
- [ ] No new `pub` exports; visibility is private or `pub(crate)` as needed
- [ ] All 11 `player_profile` tests pass unchanged
- [ ] `cargo test --all-targets --all-features` passes

## Blocked by

- 01 — Extract registry submodule

## Status

done
