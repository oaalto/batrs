# Player Profile Settings Dispatch Consolidation

## Status

done

## Problem Statement

After the deepen slice split Player Profile interpretation into submodules, **five near-identical dispatch surfaces** remain. Each maps a registry enum variant to a concrete field on a typed struct:

| Dispatch surface | Enum | Target struct | Arms |
| --- | --- | --- | --- |
| `read_persist` | `PersistSlot` | `SettingsTable` | 8 (7 typed fields + `Extra` map lookup) |
| `write_persist` | `PersistSlot` | `mut SettingsTable` | 8 (includes sparse-extra branch for `is_lich`) |
| `read_known_slot` | `SettingSlot` | `KnownProfileSettings` | 5 (includes `RiftwalkerEntity(usize)` index) |
| `write_known_slot` | `SettingSlot` | `mut KnownProfileSettings` | 5 (bool coercion for `IsLich`) |
| `write_guild_dialog_slot` | `SettingSlot` | `mut GuildDialogProfileDefaults` | 4 active + 2 no-op (`Rig`, `IsLich`) |

The settings registry slice unified **iteration** behind `SETTINGS_DEFS` — normalization folds, automation export, guild dialog defaults, and TOML bridge loops all walk the same table. But **field access** is still hand-wired: adding a 10th known setting requires editing the registry row **and** adding arms to up to four match statements (five if guild dialog applies).

The duplication is structural, not accidental: three different target types (`SettingsTable`, `KnownProfileSettings`, `GuildDialogProfileDefaults`) share the same `SettingSlot` / `PersistSlot` vocabulary but need different access patterns (clone vs assign, index for riftwalker array, bool parse for `is_lich`, sparse-extra remove/insert).

**Friction when adding a setting today:**

1. New `SettingSlot` + `PersistSlot` variants (if not reusing `Extra`)
2. New `SETTINGS_DEFS` row in the registry
3. New arm in `read_persist` + `write_persist`
4. New arm in `read_known_slot` + `write_known_slot`
5. Possibly new arm in `write_guild_dialog_slot` (if `guild_dialog = true`)
6. `KnownProfileSettings` field (if typed slot) — already required by registry PRD

Steps 3–5 are the target of this slice. Step 6 stays — `KnownProfileSettings` remains a typed struct per registry PRD user story 14.

## Solution

**Dispatch consolidation via enum inherent methods (Option A).** Move all five free-function match tables into `impl` blocks on `PersistSlot` and `SettingSlot` in the persistence module. Delete the five free functions; call sites invoke enum methods directly (`definition.persist.read(...)`, `definition.slot.write(...)`, etc.).

This is organizational deduplication — one home per enum, compiler exhaustiveness on impl arms — not cardinality reduction. Adding a setting still requires new enum variants and new impl arms, but not scattered free functions across modules.

**Accepted trade-off on user story 1:** "Registry row + struct field only" becomes "registry row + struct field + colocated impl arms on `PersistSlot` / `SettingSlot`." This is documented explicitly; Option B (per-row function pointers) or Option C (lookup tables) were rejected as over-engineered at ~9 settings.

## User Stories

1. As a maintainer adding a player setting, I want dispatch logic colocated on the registry enums, so that I do not hunt across five free functions in two modules.
2. As a maintainer adding a player setting, I accept that new enum variants and impl arms are still required, so that the solution stays idiomatic Rust without function-pointer proliferation.
3. As a maintainer, I want sparse-extra write behavior for bool settings unchanged, so that `is_lich` disk layout stays correct.
4. As a maintainer, I want riftwalker entity settings to keep array-index dispatch, so that four keys still map to one `[String; 4]` field.
5. As a maintainer, I want guild dialog dispatch folded into `SettingSlot`, so that all slot dispatch lives in one impl block.
6. As a maintainer, I want `Rig` and `IsLich` to remain no-op arms in `write_guild_dialog`, so that callers can filter on `guild_dialog` without a separate free function.
7. As a maintainer, I want the dual-enum riftwalker shape preserved (`PersistSlot` flat variants vs `SettingSlot::RiftwalkerEntity(usize)`), so that serde layout and runtime array ergonomics stay unchanged.
8. As a maintainer, I want all dispatch impls in the persistence module, so that the runtime module stays under the 200-line cap.
9. As a maintainer, I want the registry module to stay enum + const table only, so that dispatch does not couple the table to runtime struct types.
10. As a maintainer, I want `PersistSlot::write` to take explicit `key` and `sparse_when_default` parameters, so that the `Extra` arm retains sparse logic without depending on `SettingDefinition`.
11. As a maintainer, I want no thin wrapper functions retained, so that call sites use enum methods directly and the duplication target is fully eliminated.
12. As a test author, I want existing player profile tests to pass without modification, so that the refactor is behavior-preserving.
13. As a test author, I want a behavioral round-trip completeness test over `SETTINGS_DEFS`, so that every registry row's `persist` and `slot` variants have working read/write paths.
14. As a test author, I want guild-dialog and sparse-extra behavior to remain covered by existing targeted tests, so that the completeness test does not duplicate those concerns.
15. As a maintainer, I want the solution to stay readable without macro debugging skills, so that future settings work does not require macro expertise.
16. As a maintainer, I want no new runtime dependencies, so that Player Profile interpretation stays self-contained.
17. As a maintainer, I want no public API changes, so that config and app callers require no import or signature updates.
18. As a maintainer, I want normalization to remain pure and separate from dispatch, so that value normalization logic is not absorbed into persistence.
19. As a maintainer, I want `SETTINGS_DEFS` const-table iteration unchanged, so that no proc-macros are introduced.
20. As a maintainer, I want config to continue owning `SettingsTable` I/O and Player Profile to own interpretation, so that `CONTEXT.md` ownership boundaries are preserved.

## Implementation Decisions

### Prerequisite

Blocked on the deepen-player-profile-settings slice (module split must be merged first). All five dispatch free functions must already live in the persistence module; runtime module must still contain `read_known_slot`, `write_known_slot`, and `write_guild_dialog_slot` to be moved.

### Dispatch strategy — Option A (enum inherent methods)

Move dispatch into inherent methods on `PersistSlot` and `SettingSlot`:

- `PersistSlot::read(&self, table, key) -> String`
- `PersistSlot::write(&self, table, key, value, sparse_when_default)` — non-`Extra` arms ignore `key` and `sparse_when_default`
- `SettingSlot::read(&self, settings) -> String`
- `SettingSlot::write(&self, settings, value)` — includes bool coercion for `IsLich`
- `SettingSlot::write_guild_dialog(&self, defaults, value)` — `Rig` and `IsLich` are no-op arms; caller keeps filtering on `guild_dialog` before calling

Delete the five free functions: `read_persist`, `write_persist`, `read_known_slot`, `write_known_slot`, `write_guild_dialog_slot`.

### Module layout

- **Persistence module:** All `PersistSlot` and `SettingSlot` dispatch impls live here. Imports `KnownProfileSettings` and `GuildDialogProfileDefaults` from the runtime module (no cycle — runtime does not import persistence today).
- **Runtime module:** Loses the three slot dispatch free functions (~35 lines). Stays under the 200-line production cap.
- **Registry module:** Enum definitions and `SETTINGS_DEFS` only — no dispatch impls, no new imports of runtime or persistence types.

### Call-site changes

Three call sites update to direct enum methods:

- TOML orchestration (mod): `definition.persist.read(&table, definition.key)` and `definition.persist.write(&mut table, definition.key, value, definition.sparse_when_default)`
- Runtime profile construction: `definition.slot.write(&mut known, value)`; guild dialog loop uses `definition.slot.read(settings)` and `definition.slot.write_guild_dialog(&mut defaults, value)`
- Automation export: `definition.slot.read(settings)`

No thin `pub(crate)` wrapper functions retained.

### Riftwalker dual-enum shape

Keep as-is — no enum unification this slice:

- `PersistSlot`: four flat variants (`RiftwalkerEntityFire` … `Earth`) matching `SettingsTable` serde field names
- `SettingSlot`: single `RiftwalkerEntity(usize)` variant mapping to `KnownProfileSettings.riftwalker_entity_labels[index]`

Optional shared index helper only if both impls need identical index math; do not change enum shapes.

### `Extra` / sparse persistence

`PersistSlot::write` takes explicit `key: &str` and `sparse_when_default: bool` parameters. The `Extra` arm uses them for sparse remove/insert logic (`is_truthy_setting_value` check, remove on falsy when sparse). Typed arms ignore both parameters.

### Guild dialog dispatch

Fold `write_guild_dialog_slot` into `SettingSlot::write_guild_dialog`. No debug_assert or panic on `Rig` / `IsLich` — silent no-op matches current behavior. Caller in `GuildDialogProfileDefaults::from_settings` continues filtering `SETTINGS_DEFS` on `guild_dialog` before calling.

### Behavior preservation (unchanged)

- Dense typed persistence for string settings
- Sparse `extra` for `is_lich` with falsy-write-removes-key
- Riftwalker four-key → `[String; 4]` mapping
- Bool coercion for `IsLich` slot write
- Guild dialog field subset (mount, sabre, riftwalker labels only)
- `SETTINGS_DEFS` const-table iteration
- No new `pub` exports
- No proc-macros

### Rejected alternatives

- **Option B (per-row function pointers):** ~36 tiny functions at 9 settings; bypasses compiler exhaustiveness; rejected.
- **Option C (const lookup tables):** Index discipline burden; awkward for `RiftwalkerEntity(usize)`; rejected.
- **Relax 200-line cap:** Unnecessary — moving slot dispatch to persistence keeps runtime under cap.
- **SettingSlot impl in registry:** Couples const table to runtime struct types; rejected.

## Testing Decisions

### What makes a good test

Test external behavior through registry rows and typed structs — not internal match-arm structure, function pointer addresses, or table ordering. Assert round-trip invariants: read a sentinel value, write it back, read again and compare.

### Testing seam

**Single seam:** the existing player profile integration test module in `mod.rs`, alongside `registry_rows_are_complete_and_unique`. This is the highest seam that exercises registry ↔ dispatch wiring without reaching into implementation details.

### Must pass

- All 11 existing player profile tests unchanged.

### New test — dispatch round-trip completeness

One behavioral test looping `SETTINGS_DEFS`:

1. For each row: create a default `SettingsTable`, call `definition.persist.write` with a sentinel string, call `definition.persist.read`, assert round-trip equals sentinel.
2. For each row: create a default `KnownProfileSettings`, call `definition.slot.write` with a sentinel string, call `definition.slot.read`, assert round-trip equals sentinel (account for `IsLich` bool coercion — use a truthy sentinel like `"yes"` for bool rows, or assert against expected coerced form).

**Scope:** Persist + Known only. Do not include guild-dialog or sparse-Extra coverage in this test — those remain covered by `guild_dialog_defaults_follow_registry_flags` and `is_lich_*` tests.

### Prior art

- `registry_rows_are_complete_and_unique` — static registry completeness (slot uniqueness, normalized entry presence)
- `is_lich_explicit_false_dropped_from_extra_on_normalize` — sparse-extra write behavior
- `guild_dialog_defaults_follow_registry_flags` — guild dialog subset dispatch

### Avoid

- Testing internal table ordering or fn pointer addresses
- Snapshotting match-arm source structure
- Duplicating sparse-extra or guild-dialog semantics in the completeness test

## Out of Scope

- Changing `SettingsTable` serde layout or promoting `is_lich` to a typed field
- Replacing `KnownProfileSettings` with a map or dynamic settings bag
- Code-generating `PlayerToml` / `SettingsTable` structs
- Adding new settings (this slice refactors access only)
- Unifying `PersistSlot` and `SettingSlot` riftwalker enum shapes
- Per-row function pointers (Option B) or const lookup tables (Option C)
- `CONTEXT.md`, wiki, or manual updates
- Relaxing the 200-line production cap on runtime module
- Public API changes

## Dependencies

- **Blocked on:** deepen-player-profile-settings (module split must land first)
- **Builds on:** player-profile-settings-registry (`SETTINGS_DEFS` unified registry)

## Further Notes

- ROI is moderate at ~9 settings today; rises with each new setting. Option A is the smallest diff and lowest risk.
- Slice renamed from "dispatch deduplication" to "dispatch consolidation" — arms still exist, just colocated on enums.
- If a hypothetical 10th setting is added after this slice, the maintainer edits: registry row, struct field(s), `PersistSlot`/`SettingSlot` enum variants (if needed), and corresponding impl arms in the persistence module. Steps 3–5 from the problem statement (scattered free-function arms) are eliminated.
