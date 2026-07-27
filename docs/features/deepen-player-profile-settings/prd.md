# Deepen Player Profile Settings Module

## Problem

`src/player_profile.rs` is a single 895-line file that handles:

1. **Setting registry** — `SettingDefinition` struct + `SETTINGS_DEFS` array (8 entries covering rig, tzarakk_mount, sabre_weapon, 4 riftwalker entities, is_lich)
2. **Enum types** — `SettingKind` (String/Bool), `SettingSlot` (Rig/TzarakkMount/SabreWeapon/RiftwalkerEntity(usize)/IsLich), `PersistSlot` (Rig/TzarakkMount/SabreWeapon/RiftwalkerEntityFire/.../Extra), `AutomationExport` (Var/Flag)
3. **Persistence read/write** — `read_persist`, `write_persist` dispatch on `PersistSlot`, `read_known_slot`, `write_known_slot` dispatch on `SettingSlot`
4. **Normalization** — `normalize_player_toml`, `normalize_settings_entries`, `normalize_player_guilds`, `normalized_setting_value`
5. **Runtime profile construction** — `runtime_profile_from_parts`, `interpret_player_toml`, `settings_entries_for_editor`, `settings_table_from_entries`
6. **Automation export** — `automation_vars_for_settings`, `automation_flags_for_settings`
7. **Tests** — 11 test functions

The file is a god-file for player profile settings. Adding a new setting requires touching the enum, the array, 4+ match statements, and potentially the persistence layer. The `read_persist`/`write_persist`/`read_known_slot`/`write_known_slot` functions are all near-identical match statements over enums.

## Goals

- Split the file into focused modules by concern
- Reduce the surface area of changes needed to add a new setting
- Make each concern independently testable
- Keep the public API unchanged

## Non-goals

- Changing the data model or serialization format
- Adding new settings
- Changing how automation vars/flags are exported
- Modifying guild dialog behavior

## Proposed Architecture

```
src/player_profile/
├── mod.rs              # Public types + interpret_player_toml + runtime_profile_from_parts
├── registry.rs         # SettingDefinition, SETTINGS_DEFS, SettingKind, SettingSlot, PersistSlot, AutomationExport
├── persistence.rs      # read_persist, write_persist, definition_for_key
├── runtime.rs          # KnownProfileSettings, GuildDialogProfileDefaults, runtime_profile_from_parts, automation export functions
├── normalization.rs    # normalize_player_toml, normalize_settings_entries, normalize_player_guilds, normalized_setting_value
└── tests.rs            # Existing tests (or inline #[cfg(test)] in respective modules)
```

### `registry.rs` — setting definitions

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingKind { String, Bool }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingSlot {
    Rig,
    TzarakkMount,
    SabreWeapon,
    RiftwalkerEntity(usize),  // 0=fire, 1=air, 2=water, 3=earth
    IsLich,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PersistSlot {
    Rig,
    TzarakkMount,
    SabreWeapon,
    RiftwalkerEntityFire,
    RiftwalkerEntityAir,
    RiftwalkerEntityWater,
    RiftwalkerEntityEarth,
    Extra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutomationExport { Var, Flag }

pub struct SettingDefinition {
    pub key: &'static str,
    pub default: &'static str,
    pub kind: SettingKind,
    pub slot: SettingSlot,
    pub persist: PersistSlot,
    pub sparse_when_default: bool,
    pub guild_dialog: bool,
    pub automation_export: AutomationExport,
}

pub const SETTINGS_DEFS: &[SettingDefinition] = &[ ... ];

pub fn definition_for_key(key: &str) -> Option<&'static SettingDefinition> { ... }
```

### `persistence.rs` — TOML read/write

```rust
fn read_persist(table: &SettingsTable, definition: &SettingDefinition) -> String { ... }
fn write_persist(table: &mut SettingsTable, definition: &SettingDefinition, value: String) { ... }
fn read_known_slot(settings: &KnownProfileSettings, slot: SettingSlot) -> String { ... }
fn write_known_slot(settings: &mut KnownProfileSettings, definition: &SettingDefinition, value: String) { ... }
fn write_guild_dialog_slot(defaults: &mut GuildDialogProfileDefaults, slot: SettingSlot, value: String) { ... }
```

### `normalization.rs` — value normalization

```rust
fn normalized_setting_value(definition: &SettingDefinition, raw: String) -> String { ... }
fn normalize_player_toml(player: &mut PlayerToml) -> bool { ... }
fn normalize_settings_entries(entries: Vec<SettingEntry>) -> (Vec<SettingEntry>, bool) { ... }
fn settings_table_from_normalized_entries(entries: &[SettingEntry]) -> SettingsTable { ... }
fn normalize_player_guilds(player: &mut PlayerToml) -> bool { ... }
```

### `runtime.rs` — runtime profile + automation

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownProfileSettings { ... }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildDialogProfileDefaults { ... }

pub struct PlayerRuntimeProfile { ... }

fn runtime_profile_from_parts(
    selected_guild_keys: Vec<String>,
    guild_primary_background: &str,
    settings: UserSettings,
    generic_commands_config: GenericCommandsConfig,
) -> PlayerRuntimeProfile { ... }

fn automation_vars_for_settings(settings: &KnownProfileSettings) -> Vec<(String, String)> { ... }
fn automation_flags_for_settings(settings: &KnownProfileSettings) -> Vec<(String, bool)> { ... }
```

### `mod.rs` — public surface

```rust
mod registry;
mod persistence;
mod runtime;
mod normalization;

pub use registry::{SettingDefinition, SETTINGS_DEFS, definition_for_key};
pub use runtime::{PlayerRuntimeProfile, KnownProfileSettings, GuildDialogProfileDefaults, runtime_profile_from_parts};

pub struct InterpretedPlayerProfile { ... }

pub fn interpret_player_toml(player: PlayerToml) -> InterpretedPlayerProfile { ... }
pub fn settings_entries_for_editor(player: &PlayerToml) -> Vec<SettingEntry> { ... }
pub fn settings_table_from_entries(entries: &[SettingEntry]) -> SettingsTable { ... }
pub fn user_settings_from_player(player: &PlayerToml) -> UserSettings { ... }
```

## Migration Plan

1. Create the four new modules with extracted code
2. Update `mod.rs` to re-export public types and delegate to submodules
3. Run `cargo test` — all existing tests should pass without modification
4. Add module-level tests in each submodule for its pure functions

## Success Criteria

- Each module file is < 200 lines
- Adding a new setting touches only `registry.rs` (new enum variants, new array entry) and `persistence.rs` (new match arms)
- `normalization.rs` has no dependency on persistence or runtime modules
- `runtime.rs` has no dependency on persistence (it constructs from in-memory types)
- All existing tests pass

## Risks

- **Module boundary confusion**: `read_persist` needs `SettingDefinition` from registry and `SettingsTable` from config. `write_known_slot` needs `KnownProfileSettings`. The module boundaries are clear if `persistence.rs` depends on `registry.rs` for the definition type but uses `runtime.rs` types only through generic trait bounds or by accepting the concrete types.
- **Test migration**: The existing test module in player_profile.rs tests cross-cutting behavior (e.g., `interpret_player_toml` exercises normalization + persistence + runtime). These tests may need to stay in `mod.rs` or move to a dedicated integration-style test file.
- **Hash requirement**: `SettingSlot` and `PersistSlot` are used as match arms and array indices but not as HashMap keys, so `Hash`/`Eq` derives are sufficient — no need for `Ord`.
