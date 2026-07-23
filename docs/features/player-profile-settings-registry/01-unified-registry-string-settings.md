# 01 — Unified registry for string settings

## Parent

`prd.md`

## What to build

Extend the Player Profile settings registry so all seven existing **string** settings (`rig`, `tzarakk_mount`, `sabre_weapon`, four `riftwalker_entity_*` keys) are declared once in `SETTINGS_DEFS` and drive the full interpretation path: normalization, TOML round-trip bridging, `KnownProfileSettings` population, automation var export, and guild dialog defaults.

Hand-written key match arms and duplicate field lists for these settings are removed. `is_lich` may remain on the legacy path for this ticket; behavior for string settings and unknown `extra` keys must not regress.

## Acceptance criteria

- [ ] Each string setting has one registry row with `kind`, `slot`, `persist`, `guild_dialog`, and `automation_export` metadata
- [ ] `interpret_player_toml` and `settings_entries_for_editor` produce the same string-setting behavior as before
- [ ] Unknown keys are preserved in `extra` through normalize round-trip
- [ ] Empty riftwalker entity labels still normalize to `"entity"`
- [ ] Automation vars for rig, mount, sabre, and riftwalker labels are built from registry metadata
- [ ] Guild dialog defaults for mount, sabre, and riftwalker labels are built from `guild_dialog` flags (`rig` and `is_lich` excluded; `primary_background` still from guild selection)
- [ ] All existing `player_profile` module tests pass

## Blocked by

None — can start immediately.

**Status:** ready-for-agent
