# Player Profile settings registry

## Status

ready-for-agent — grilled 2026-07-23

## Problem Statement

Player Profile interpretation uses a partial settings registry. A const table drives normalization for rig, Tzarakk mount, sabre weapon, and four Riftwalker entity labels — but `is_lich` is handled separately via a dedicated lookup reading from the `extra` bucket, with its own key constant and hand-wired automation flag export. Adding a new known setting requires edits in at least four places: the registry table, player-to-entry extraction, normalized-entry-to-settings-table mapping, and `KnownProfileSettings` field wiring plus automation and guild-dialog export lists.

The Player Profile module already deepened interpretation; the registry remains manual and drifts from the settings editor, guild dialog defaults, and automation var/flag export.

## Solution

Unify all known player settings behind one extended const registry in the Player Profile interpretation module. Each registry entry declares: key, default, value kind (string or bool), runtime slot, persistence target (typed settings field or sparse `extra`), whether it feeds guild dialog defaults, and automation export kind (var, flag, or none). Normalization, TOML round-trip bridging, runtime profile construction, automation export, and guild dialog defaults all iterate the same table.

`is_lich` becomes a first-class registry entry: known at normalization and runtime, still stored in `extra` on disk (no `SettingsTable` serde change), omitted from `extra` when false, and explicit falsy values dropped on write. Config module continues owning player file I/O and the `SettingsTable` struct layout; Player Profile continues owning interpretation.

## User Stories

1. As a maintainer adding a player setting, I want one registry entry, so that normalization, runtime profile, automation, and guild dialog stay in sync.
2. As a maintainer, I want `is_lich` treated like other known settings in the registry, so that special-case `extra` handling in interpretation disappears.
3. As a player, I want the settings editor to show all known keys with defaults (including `is_lich`), so that editing is predictable.
4. As a maintainer, I want guild dialog defaults derived from registry flags, so that mount, sabre, and riftwalker label field lists are not duplicated.
5. As a maintainer, I want automation vars and flags derived from registry export metadata, so that new settings do not require hand-listed export code.
6. As a test author, I want a registry completeness test, so that every registry key maps to a unique runtime slot and appears in normalized output.
7. As a maintainer, I want unknown settings preserved in `extra` through normalize round-trip, so that forward-compatible player files still work.
8. As a player with `is_lich` off, I want the key omitted from my player file, so that disk layout stays minimal.
9. As a player with `is_lich` on (`true`, `yes`, or `1`), I want the key persisted in `extra`, so that lich behavior survives restarts.
10. As a player who manually set `is_lich = "false"`, I want normalization to remove that key on save, so that falsy clutter does not accumulate.
11. As a maintainer, I want bool settings parsed with the same truthy rules as today (`1`, `true`, `yes`, case-insensitive), so that existing player files keep working.
12. As a maintainer, I want typed string settings to keep dense persistence (empty strings on typed fields), so that current on-disk layout for rig, mount, sabre, and riftwalker labels is unchanged.
13. As a maintainer, I want empty riftwalker entity labels normalized to `"entity"`, so that runtime and guild dialog defaults stay consistent.
14. As a maintainer, I want `KnownProfileSettings` to remain a typed struct with named fields, so that callers (`rig_for_triggers`, guild modules, automation) do not need a map API.
15. As a maintainer, I want four separate riftwalker entity keys to map to one `[String; 4]` runtime field, so that the registry matches disk layout without losing the array ergonomics.
16. As a maintainer, I want `primary_background` in guild dialog defaults to continue coming from guild selection, not the settings registry, so that theme bucket ownership stays unchanged.
17. As a maintainer, I want `rig` excluded from guild dialog defaults, so that automation rig name does not appear in the guild dialog.
18. As a maintainer, I want Player Profile to own interpretation and config to own I/O, so that `CONTEXT.md` ownership boundaries are preserved.
19. As a test author, I want `is_lich` sparse-extra behavior covered explicitly, so that regressions in drop-on-false logic are caught.
20. As a maintainer, I want no new source files for ~9 settings, so that the registry stays co-located with interpretation in one module.

## Implementation Decisions

### Scope boundary

Registry lives in the **Player Profile interpretation module** only. The config module keeps `SettingsTable`, `PlayerToml` serde layout, and player file read/write unchanged. Bridge functions between config types and the entry list become registry-driven folds; no code generation for config structs.

### Registry shape

Extend the existing const `SETTINGS_DEFS` table (runtime iteration, no macros or proc-macros). Each `SettingDefinition` row carries:

| Field | Purpose |
| --- | --- |
| `key` | TOML / entry key string |
| `default` | Default string value used in normalization |
| `kind` | `String` or `Bool` |
| `slot` | `SettingSlot` enum targeting `KnownProfileSettings` fields |
| `persist` | `PersistSlot` enum: typed settings field or `Extra` |
| `sparse_when_default` | When true, omit from `extra` on write if value is default/falsy |
| `guild_dialog` | Whether value feeds `GuildDialogProfileDefaults` |
| `automation_export` | `Var`, `Flag`, or `None` |

Nine registry rows today: `rig`, `tzarakk_mount`, `sabre_weapon`, four `riftwalker_entity_*` keys, `is_lich`.

### SettingSlot mapping

Keep typed `KnownProfileSettings` (`rig`, `tzarakk_mount`, `sabre_weapon`, `riftwalker_entity_labels: [String; 4]`, `is_lich: bool`). Registry uses `SettingSlot` variants:

- `Rig`, `TzarakkMount`, `SabreWeapon`, `IsLich`
- `RiftwalkerEntity(0..3)` for fire/air/water/earth keys → array indices

`from_user_settings` becomes a registry fold assigning slots instead of per-field lookups.

### Persistence and `is_lich`

- Typed string settings: dense persistence on `SettingsTable` fields (current behavior).
- `is_lich`: `persist = Extra`, `sparse_when_default = true`, `kind = Bool`.
- Read: lift from `extra` into normalized entry list.
- Write: include in `extra` only when truthy; drop key when false or explicit falsy (`"false"`, etc.).
- No typed `is_lich` field added to `SettingsTable`.

### Bool parsing

`kind = Bool` entries parse via the existing `is_truthy_setting_value` helper in config (`1`, `true`, `yes`, case-insensitive). Export the helper from config only if visibility requires it; otherwise call through existing `UserSettings` APIs. Keep `UserSettings::is_lich_enabled()` if existing config tests depend on it.

### Normalization

`normalize_settings_entries` continues to inject missing known keys with defaults and preserve unknown keys after known keys. Registry replaces hand-maintained key membership checks. Riftwalker empty-string → `"entity"` normalization remains for the four entity keys.

### TOML bridge

`user_settings_from_player` and `settings_table_from_normalized_entries` become registry-driven folds over `PersistSlot`:

- Typed slots read/write the corresponding `SettingsTable` field.
- `Extra` slot reads/writes the `extra` map with sparse rules.

### Automation export

Replace hand-listed `automation_vars_for_settings` and hard-coded `automation_flags` with registry iteration from `KnownProfileSettings`:

- `rig`, mount, sabre, four riftwalker keys → `AutomationExport::Var`
- `is_lich` → `AutomationExport::Flag`
- Future editor-only settings can use `None`.

### Guild dialog defaults

`GuildDialogProfileDefaults::from_settings` iterates registry entries where `guild_dialog = true` (mount, sabre, four riftwalker labels). `primary_background` still supplied from guild selection outside the registry. `rig` and `is_lich` excluded.

### Module layout

All registry types, table, and fold logic stay in the **Player Profile module** (single file acceptable). No new submodule or crate-root settings module.

### Documentation

Update this PRD with grilled decisions. No `CONTEXT.md`, player manual, or wiki changes — manual already documents `is_lich` in `extra`; ownership boundary unchanged. No ADR unless implementation surprises.

## Testing Decisions

### Primary test seam

Test through the **Player Profile public interpretation interface** at the highest seam — `interpret_player_toml`, `settings_table_from_entries`, and `settings_entries_for_editor` (or equivalent exported helpers). Do not test config serde or settings dialog UI directly. Single seam: entry list / `PlayerToml` in → normalized `PlayerToml` + `PlayerRuntimeProfile` out.

### Good tests (external behavior)

- Registry completeness: every `SETTINGS_DEFS` row has a unique `SettingSlot`; each key appears in normalized output; each slot populates `KnownProfileSettings`.
- Missing settings receive defaults (including riftwalker `"entity"`).
- `is_lich` absent → runtime `is_lich` false; not in `extra` after round-trip.
- `is_lich = "yes"` / `"true"` / `"1"` → runtime true; persisted in `extra`.
- `is_lich = "false"` explicitly set → dropped from `extra` on normalize; `changed = true`.
- Unknown keys preserved in `extra` through round-trip.
- Guild dialog defaults match flagged registry entries.
- Automation vars and flags match registry export metadata.
- Existing profile behavior tests continue to pass (guild normalization, generic commands preservation).

### Prior art

- `player_profile` module tests (`profile_extracts_known_settings`, `interpret_player_toml_normalizes_settings_without_runtime_editor_entries`, etc.).
- `config` module `user_settings_is_lich_enabled` for truthy parsing contract.

### Avoid

- Asserting internal match-arm structure or registry loop order.
- Testing `SettingsTable` serde field renames (out of scope).
- Settings dialog UI integration tests (out of scope).

## Out of Scope

- Guild selection normalization (owned by `GuildSelection`).
- Generic commands config registry.
- Settings dialog UI redesign.
- Promoting `is_lich` to a typed `SettingsTable` serde field.
- Registry-driven `PlayerToml` / `SettingsTable` struct generation.
- Remote/cloud sync of player files.
- `CONTEXT.md`, player manual, or wiki updates for this slice.

## Further Notes

- Recommendation strength: **Strong** — grilling resolved all open questions; friction is real when adding settings.
- Aligns with prior deepen commits on Player Profile interpretation.
- `CONTEXT.md` Player Profile section remains authority for interpretation vs. config I/O ownership.
- Grilling decisions (2026-07-23): interpretation-only scope, const table, `is_lich` in sparse `extra`, dense typed strings, registry-driven TOML bridge/automation/guild dialog, `SettingSlot` + `PersistSlot`, completeness + sparse-extra tests, PRD-only docs, single-module layout.
