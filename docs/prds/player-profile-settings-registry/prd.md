# Player Profile settings registry

## Status

draft — initial exploration for grilling

## Problem Statement

Player Profile interpretation uses a partial settings registry. `SETTINGS_DEFS` drives normalization for rig, Tzarakk mount, sabre weapon, and four Riftwalker entity labels — but `is_lich` is handled separately via `UserSettings::is_lich_enabled()` reading from `extra`, with its own key constant and automation flag wiring. Adding a new known setting requires edits in at least four places: `SETTINGS_DEFS`, `user_settings_from_player` field extraction, `settings_table_from_normalized_entries` match arms, and `KnownProfileSettings` struct fields plus `from_user_settings`.

The Player Profile module already deepened interpretation (`3ffcccd`, `f98d52f`); the registry remains manual and drifts from the settings dialog and automation var export.

## Initial exploration

| Setting | In `SETTINGS_DEFS` | In `SettingsTable` typed fields | In `KnownProfileSettings` | Automation export |
| --- | --- | --- | --- | --- |
| `rig` | yes | yes | yes | var |
| `tzarakk_mount` | yes | yes | yes | var |
| `sabre_weapon` | yes | yes | yes | var |
| `riftwalker_entity_*` (×4) | yes | yes | yes (array) | vars |
| `is_lich` | **no** | **extra** | yes (bool) | flag |

**Flow:**

1. `interpret_player_toml` → `normalize_player_toml` → `normalize_settings_entries` (known + extras)
2. `settings_table_from_normalized_entries` — hand-written key match → `SettingsTable`
3. `KnownProfileSettings::from_user_settings` — separate lookups; `is_lich` via `is_lich_enabled()`
4. `automation_vars_for_settings` / `automation_flags` — hand-listed keys
5. `GuildDialogProfileDefaults::from_settings` — subset for guild dialog

**Settings editor:** `settings_entries_for_editor` uses full entry list; unknown keys preserved in `extra`.

## Solution (proposed direction)

Unify known settings behind one registry (table or macro-generated definitions) that declares: key, default, typed slot in `KnownProfileSettings`, whether it appears in guild dialog defaults, and automation export kind (var vs. flag). Normalization, TOML round-trip, and runtime interpretation read the same definitions.

Grilling should decide: full derive/macro approach vs. a single `const` table iterated at runtime (ponytail-friendly).

## User Stories

1. As a maintainer adding a player setting, I want one registry entry, so that normalization and runtime profile stay in sync.
2. As a maintainer, I want `is_lich` treated like other known settings, so that special-case `extra` handling disappears.
3. As a player, I want settings dialog to show all known keys with defaults, so that editing is predictable.
4. As a maintainer, I want guild dialog defaults derived from the same registry, so that mount/sabre/riftwalker labels do not duplicate field lists.
5. As a test author, I want registry completeness tested once, so that every `SETTINGS_DEFS` key maps to `KnownProfileSettings`.
6. As a maintainer, I want extra/unknown settings preserved through normalize round-trip, so that forward-compatible player files still work.

## Open questions (for grilling)

1. **Registry shape:** Single `SettingDefinition` enum/struct with `AutomationExport::Var | Flag | None`?
2. **`is_lich` storage:** Promote to typed field on `SettingsTable` vs. keep in `extra` with registry metadata?
3. **Breaking TOML:** Is changing on-disk layout for `is_lich` acceptable, or must migration read old `extra`?
4. **Guild dialog coupling:** Should registry mark which settings appear in `GuildDialogProfileDefaults`?
5. **Scope:** Registry only for `KnownProfileSettings`, or also drive `config.rs` `PlayerToml` struct generation (likely out of scope)?
6. **Bool settings:** Generalize `is_truthy_setting_value` for future flags?

## Implementation Decisions (tentative)

- Extend `SettingDefinition` with type kind (string, bool) and automation mapping.
- Add `is_lich` to `SETTINGS_DEFS` with default `"false"`; migrate read path from `extra` if present.
- Replace hand-written `settings_table_from_normalized_entries` key chain with registry-driven fold.
- `KnownProfileSettings::from_user_settings` becomes registry iteration.
- Keep `extra` bucket for truly unknown keys only.
- No change to Player Profile owning interpretation vs. config owning I/O (per `CONTEXT.md`).

## Testing Decisions

- **Good tests:** Round-trip normalize adds missing defaults; `is_lich` true/false/yes parsing; unknown keys preserved in extra.
- **Prior art:** `player_profile.rs` module tests; `config.rs` `user_settings_is_lich_enabled`.
- **New:** Registry coverage test — every def key appears in normalized output and runtime struct.

## Out of Scope

- Guild selection normalization (already separate via `GuildSelection`).
- Generic commands config registry.
- Settings dialog UI redesign.
- Remote/cloud sync of player files.

## Further Notes

- Recommendation strength: **Worth exploring** — real friction when adding settings, not urgent breakage.
- Aligns with prior deepen commit `3ffcccd Deepen player profile interpretation`.
- `CONTEXT.md` Player Profile section is the authority for ownership boundaries.
