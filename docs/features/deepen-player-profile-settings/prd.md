# Deepen Player Profile Settings Module

## Status

ready

## Problem Statement

`src/player_profile.rs` is a single ~895-line file that owns player settings interpretation end to end:

1. **Setting registry** — `SettingDefinition`, `SETTINGS_DEFS` (9 entries), `SettingKind`, `SettingSlot`, `PersistSlot`, `AutomationExport`
2. **Persistence dispatch** — `read_persist`, `write_persist` match on `PersistSlot`; `read_known_slot`, `write_known_slot` match on `SettingSlot`; `write_guild_dialog_slot` match on `SettingSlot` against `GuildDialogProfileDefaults`
3. **Normalization** — `normalized_setting_value` (pure), plus TOML orchestration (`normalize_player_toml`, `normalize_settings_entries`, `settings_table_from_normalized_entries`, `normalize_player_guilds`)
4. **Runtime profile** — `KnownProfileSettings`, `GuildDialogProfileDefaults`, `runtime_profile_from_parts`, automation export, guild dialog defaults
5. **Public API** — `interpret_player_toml`, `settings_entries_for_editor`, `settings_table_from_entries`, `user_settings_from_player`, `PlayerRuntimeProfile`, `InterpretedPlayerProfile`
6. **Tests** — 11 integration-style tests in `#[cfg(test)] mod tests`

The [player-profile-settings-registry](player-profile-settings-registry/prd.md) slice unified known settings behind `SETTINGS_DEFS` and registry-driven folds for normalization, TOML bridge, automation, and guild dialog. The monolith remains hard to navigate: registry types, dispatch tables, normalization, runtime construction, and tests all share one file. Adding a setting still requires touching multiple match arms in addition to a registry row — but **eliminating those match arms is explicitly out of scope for this slice** (see [player-profile-dispatch-dedup](player-profile-dispatch-dedup/prd.md)).

## Solution

**Mechanical split only.** Extract the monolith into a `src/player_profile/` directory with focused submodules. Move code without changing behavior, data model, serialization, or public API. Preserve all 11 existing tests verbatim in `mod.rs`.

## User Stories

1. As a maintainer, I want player profile settings split by concern into submodules, so that I can find registry, persistence, normalization, and runtime logic without scrolling an 895-line file.
2. As a maintainer, I want each production submodule under 200 lines, so that individual concerns stay reviewable.
3. As a maintainer, I want the public `player_profile` API unchanged, so that `config` and `app` callers require no import or signature updates.
4. As a maintainer, I want all 11 existing tests to pass without modification, so that the split is provably behavior-preserving.
5. As a maintainer, I want normalization pure functions isolated from persistence and runtime, so that value-normalization logic is testable without TOML or profile construction.
6. As a maintainer, I want TOML orchestration to stay in `mod.rs`, so that the interpretation entry points remain the obvious coordination layer.
7. As a maintainer, I want key constants colocated with registry rows, so that adding a setting does not scatter string literals.
8. As a maintainer, I want only the three externally used key constants re-exported from `mod.rs`, so that the public surface stays minimal.
9. As a test author, I want tests to reach `pub(crate)` internals via explicit submodule paths, so that test imports document which layer is under test.
10. As a maintainer, I want dispatch deduplication deferred to a follow-up slice, so that this PR stays a safe mechanical refactor with a clear boundary.

## Implementation Decisions

### Scope boundary

- **In scope:** File/module split, `pub(crate)` visibility wiring, submodule `mod` declarations, moving code to match the layout below.
- **Out of scope:** Dispatch deduplication (`read_persist` / `write_persist` / `read_known_slot` / `write_known_slot` / `write_guild_dialog_slot` match tables stay as-is, moved to `persistence.rs`). See follow-up PRD at `docs/features/player-profile-dispatch-dedup/prd.md`.
- **Out of scope:** New settings, serde/TOML layout changes, automation export behavior changes, guild dialog behavior changes, `CONTEXT.md` / wiki / manual updates.

### Module layout

```
src/player_profile/
├── mod.rs              # Public API, TOML orchestration, #[cfg(test)] mod tests (all 11 tests)
├── registry.rs         # Key constants, enums, SettingDefinition, SETTINGS_DEFS, definition_for_key
├── persistence.rs      # read_persist, write_persist, read_known_slot, write_known_slot, write_guild_dialog_slot
├── runtime.rs          # KnownProfileSettings, GuildDialogProfileDefaults, runtime_profile_from_parts,
│                       # automation_vars_for_settings, automation_flags_for_settings, impl blocks
└── normalization.rs    # normalized_setting_value only (pure)
```

`src/player_profile.rs` becomes `mod player_profile;` re-export shell or is replaced by `src/player_profile/mod.rs` with `main.rs` unchanged (`mod player_profile;` resolves to the directory).

### Layer boundaries (strict)

| Module | Owns | Must not own |
| --- | --- | --- |
| `registry.rs` | All `*_KEY` constants, `SettingKind`, `SettingSlot`, `PersistSlot`, `AutomationExport`, `SettingDefinition`, `SETTINGS_DEFS`, `definition_for_key` | Read/write dispatch, normalization, runtime structs |
| `normalization.rs` | `normalized_setting_value` (pure: `SettingDefinition` + raw string → normalized string) | TOML types, `SettingsTable`, `PlayerToml`, `KnownProfileSettings`, persistence |
| `persistence.rs` | Five dispatch functions (match tables unchanged) | Normalization logic beyond calling `normalized_setting_value` at call sites that already do |
| `runtime.rs` | `KnownProfileSettings`, `GuildDialogProfileDefaults`, `runtime_profile_from_parts`, `KnownProfileSettings::from_user_settings`, `GuildDialogProfileDefaults::from_settings`, automation export fns, `default_riftwalker_entity_labels`, `setting_value`, `non_empty` | TOML read/write orchestration |
| `mod.rs` | `interpret_player_toml`, `settings_entries_for_editor`, `settings_table_from_entries`, `user_settings_from_player`, `normalize_player_toml`, `normalize_settings_entries`, `settings_table_from_normalized_entries`, `normalize_player_guilds`, public struct re-exports, `#[cfg(test)] mod tests` | Registry row definitions, dispatch match arms |

`normalization.rs` has **no** dependency on `persistence` or `runtime`. `runtime.rs` depends on `registry` and `persistence` (for `read_known_slot` / `write_known_slot` in impl blocks) but not on TOML orchestration in `mod.rs`.

### Public API preservation (exact)

**Re-exported from `mod.rs` (unchanged signatures and types):**

| Symbol | Visibility |
| --- | --- |
| `TZARAKK_MOUNT_KEY` | `pub const` |
| `SABRE_WEAPON_KEY` | `pub const` |
| `RIFTWALKER_ENTITY_LABEL_KEYS` | `pub const` |
| `PlayerRuntimeProfile` | `pub struct` |
| `InterpretedPlayerProfile` | `pub struct` |
| `KnownProfileSettings` | `pub struct` |
| `GuildDialogProfileDefaults` | `pub struct` |
| `interpret_player_toml` | `pub fn` |
| `settings_entries_for_editor` | `pub fn` |
| `settings_table_from_entries` | `pub fn` |
| `user_settings_from_player` | `pub fn` |

**Not newly exported** (remain private or `pub(crate)`):

- `SettingDefinition`, `SETTINGS_DEFS`, `definition_for_key`
- `runtime_profile_from_parts`
- All registry enums (`SettingKind`, `SettingSlot`, `PersistSlot`, `AutomationExport`)
- All persistence dispatch functions
- Internal key constants (`RIG_KEY`, `IS_LICH_KEY`, individual riftwalker keys, etc.) — live in `registry.rs`, not re-exported

No new `pub` items. No signature changes on existing public items.

### Key constant placement

All key string constants (`RIG_KEY`, `TZARAKK_MOUNT_KEY`, `SABRE_WEAPON_KEY`, four riftwalker keys, `IS_LICH_KEY`, `DEFAULT_RIFTWALKER_ENTITY_LABEL`, `RIFTWALKER_ENTITY_LABEL_KEYS`) live in `registry.rs`. `mod.rs` re-exports only the three public constants consumed by `app/mod.rs`.

### Line-count budget

**Hard cap: < 200 lines per production file.** `#[cfg(test)]` blocks are **excluded** from the count. If `mod.rs` exceeds 200 lines production code after the split, move TOML helpers to a `pub(crate)` submodule (e.g. `bridge.rs`) in a follow-up — do not expand scope in this slice.

### Visibility

- Submodule items default to private; expose across submodules via `pub(crate)`.
- `runtime_profile_from_parts`: `pub(crate)` (tests in `mod.rs` call it today).
- Persistence dispatch fns: private to `persistence.rs` or `pub(crate)` if `runtime.rs` / `mod.rs` need them.

### Tests

- **All 11 tests stay in `mod.rs`** inside `#[cfg(test)] mod tests`. No `tests.rs`, no per-submodule test modules in this slice.
- Tests that need registry or runtime internals import explicitly: `use super::registry::{...}` / `use super::runtime::{...}` (or call through `pub(crate)` fns).
- No test logic changes. `cargo test` must pass with zero assertion or fixture edits.

### Migration steps

1. Create `src/player_profile/` directory and submodule files.
2. Move code to submodules per layout above; add `mod` declarations and `pub(crate)` / `pub use` wiring in `mod.rs`.
3. Delete or replace `src/player_profile.rs` with directory module.
4. Run `cargo test --all-targets --all-features` — all tests green.
5. Verify production line counts per file (< 200, tests excluded).

## Testing Decisions

### Primary test seam

Existing 11 tests in `mod.rs` — integration-style through `interpret_player_toml`, `runtime_profile_from_parts`, `settings_table_from_entries`, `user_settings_from_player`. **No new tests required** for a pure mechanical split.

### Avoid

- Asserting internal module paths or match-arm structure.
- Splitting or rewriting existing tests across submodules.

## Success Criteria

- [ ] `src/player_profile/` directory with `mod.rs`, `registry.rs`, `persistence.rs`, `runtime.rs`, `normalization.rs`
- [ ] Each production file < 200 lines (`#[cfg(test)]` excluded)
- [ ] Public API identical to pre-split (grep `player_profile::` in `config.rs` and `app/mod.rs` — no caller changes)
- [ ] All 11 tests pass unchanged
- [ ] `normalization.rs` has no imports from `persistence` or `runtime`
- [ ] Dispatch match tables unchanged in behavior (moved, not deduplicated)

## Risks

- **Module boundary creep:** TOML orchestration must stay in `mod.rs`; resist moving `normalize_player_toml` into `normalization.rs` (violates strict layer decision).
- **`mod.rs` line count:** Tests + orchestration may push production code near the cap; monitor during implementation.
- **Circular deps:** `runtime.rs` calling `persistence` and `mod.rs` calling both — use `pub(crate)` carefully; `registry` must not depend on `runtime` or `persistence`.

## Follow-up

Dispatch deduplication: `docs/features/player-profile-dispatch-dedup/prd.md` (grill-ready stub; approach not pre-selected).

## Further Notes

- Depends on completed [player-profile-settings-registry](player-profile-settings-registry/prd.md) (`SETTINGS_DEFS` unified registry).
- Grilling decisions (2026-07-28): mechanical split only; strict normalization layer; tests in `mod.rs`; exact public API; key constants in `registry.rs` with 3 re-exports; < 200 lines production code; `pub(crate)` + explicit test imports; dispatch dedup deferred.
