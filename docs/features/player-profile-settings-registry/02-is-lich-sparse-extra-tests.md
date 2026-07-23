# 02 — Register `is_lich` with sparse extra and completeness tests

## Parent

`prd.md`

## What to build

Add `is_lich` as a ninth registry row (`Bool` kind, `Extra` persistence, sparse when default/falsy). Remove legacy special-casing for `is_lich` in Player Profile interpretation. Automation flags are built from registry `automation_export` metadata (not hand-listed).

Sparse-extra behavior: `is_lich` omitted from disk `extra` when off; truthy values (`true`, `yes`, `1`, case-insensitive) persisted; explicit falsy values (e.g. `"false"`) dropped on normalize with `changed = true`. Settings editor receives `is_lich` with default via normalization even when absent on disk.

Add registry completeness test (all nine keys, unique slots, normalized output) and explicit `is_lich` edge-case tests. Export `is_truthy_setting_value` from config only if needed for bool parsing; keep `UserSettings::is_lich_enabled()` if existing config tests depend on it.

## Acceptance criteria

- [ ] `is_lich` is a registry row with `sparse_when_default = true` and `automation_export = Flag`
- [ ] `is_lich` absent from player file → runtime `false`, not written to `extra`
- [ ] `is_lich = "yes"` / `"true"` / `"1"` → runtime `true`, persisted in `extra`
- [ ] `is_lich = "false"` (explicit) → dropped from `extra` on normalize; `changed = true`
- [ ] Registry completeness test covers all nine defs with unique `SettingSlot` mapping
- [ ] Unknown keys still preserved in `extra` through round-trip
- [ ] Legacy `is_lich` special-case code paths in Player Profile are removed
- [ ] All `player_profile` and relevant `config` tests pass

## Blocked by

- [01 — Unified registry for string settings](01-unified-registry-string-settings.md)

**Status:** ready-for-agent
