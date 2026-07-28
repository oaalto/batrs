# Player Profile Settings Dispatch Deduplication

## Status

draft — grill-ready

## Problem Statement

After [deepen-player-profile-settings](deepen-player-profile-settings/prd.md) splits `src/player_profile.rs` into submodules, **five near-identical dispatch surfaces** remain. Each maps a registry enum variant to a concrete field on a typed struct:

| Function | Enum | Target struct | Arms |
| --- | --- | --- | --- |
| `read_persist` | `PersistSlot` | `SettingsTable` | 8 (7 typed fields + `Extra` map lookup) |
| `write_persist` | `PersistSlot` | `mut SettingsTable` | 8 (includes sparse-extra branch for `is_lich`) |
| `read_known_slot` | `SettingSlot` | `KnownProfileSettings` | 5 (includes `RiftwalkerEntity(usize)` index) |
| `write_known_slot` | `SettingSlot` | `mut KnownProfileSettings` | 5 (bool coercion for `IsLich`) |
| `write_guild_dialog_slot` | `SettingSlot` | `mut GuildDialogProfileDefaults` | 4 active + 2 no-op (`Rig`, `IsLich`) |

The [player-profile-settings-registry](player-profile-settings-registry/prd.md) slice unified **iteration** behind `SETTINGS_DEFS` — normalization folds, automation export, guild dialog defaults, and TOML bridge loops all walk the same table. But **field access** is still hand-wired: adding a 10th known setting requires editing the registry row **and** adding arms to up to four match statements (five if guild dialog applies).

Today (~895-line monolith, post-registry):

```rust
// persistence.rs (representative — four functions follow the same shape)
fn read_persist(table: &SettingsTable, definition: &SettingDefinition) -> String {
    match definition.persist {
        PersistSlot::Rig => table.rig.clone(),
        PersistSlot::TzarakkMount => table.tzarakk_mount.clone(),
        // ... 6 more arms
        PersistSlot::Extra => table.extra.get(definition.key).cloned().unwrap_or_default(),
    }
}
```

The duplication is structural, not accidental: three different target types (`SettingsTable`, `KnownProfileSettings`, `GuildDialogProfileDefaults`) share the same `SettingSlot` / `PersistSlot` vocabulary but need different access patterns (clone vs assign, index for riftwalker array, bool parse for `is_lich`, sparse-extra remove/insert).

**Friction when adding a setting today:**

1. New `SettingSlot` + `PersistSlot` variants (if not reusing `Extra`)
2. New `SETTINGS_DEFS` row in `registry.rs`
3. New arm in `read_persist` + `write_persist`
4. New arm in `read_known_slot` + `write_known_slot`
5. Possibly new arm in `write_guild_dialog_slot` (if `guild_dialog = true`)
6. `KnownProfileSettings` field (if typed slot) — already required by registry PRD

Steps 3–5 are the target of this slice. Step 6 stays — `KnownProfileSettings` remains a typed struct per registry PRD user story 14.

## Goals

- Adding a new known setting touches **registry row + typed struct field(s)** only — not N separate match arms
- Preserve all existing behavior: dense typed persistence, sparse `extra` for `is_lich`, riftwalker array indexing, guild dialog field subset, bool coercion
- Keep `SETTINGS_DEFS` const-table iteration (no proc-macros per registry PRD)
- No public API changes

## Non-goals

- Changing `SettingsTable` serde layout or promoting `is_lich` to a typed field
- Replacing `KnownProfileSettings` with a map or dynamic settings bag
- Code-generating `PlayerToml` / `SettingsTable` structs
- New settings (this slice refactors access only)
- `CONTEXT.md`, wiki, or manual updates

## Inherited Constraints

From [deepen-player-profile-settings](deepen-player-profile-settings/prd.md) (must hold before and after this slice):

| Constraint | Implication for dispatch dedup |
| --- | --- |
| Module layout: `registry`, `persistence`, `runtime`, `normalization`, `mod` | Dispatch refactor lives primarily in `persistence.rs`; registry shape may extend |
| `normalization.rs` pure only | Bool/string normalization stays separate; dispatch does not absorb `normalized_setting_value` |
| `KnownProfileSettings` typed struct | Slot dispatch must read/write named fields, not a hash map |
| No new `pub` exports | Any new registry metadata is `pub(crate)` or private |
| `SETTINGS_DEFS` const table, no proc-macros | Solutions must be const-constructible or enum-impl based |
| Config owns `SettingsTable` I/O; Player Profile owns interpretation | `read_persist` / `write_persist` stay in Player Profile `persistence` module |
| All 11 existing tests pass | Behavior-preserving refactor; tests may add completeness cases but must not break |

From [player-profile-settings-registry](player-profile-settings-registry/prd.md):

| Constraint | Implication |
| --- | --- |
| `SettingSlot::RiftwalkerEntity(0..3)` → `[String; 4]` | Dispatch must handle indexed array access |
| `is_lich`: `PersistSlot::Extra`, `sparse_when_default` | `write_persist` for `Extra` must retain sparse remove/insert logic |
| `AutomationExport` drives vars/flags | No change — already registry-iterated in `runtime.rs` |
| `guild_dialog` flag on rows | `write_guild_dialog_slot` only runs for flagged entries; `Rig` / `IsLich` stay no-op |

## User Stories

1. As a maintainer adding a player setting, I want one registry row and struct field wiring only, so that I do not edit four match statements.
2. As a maintainer, I want sparse-extra write behavior for bool settings unchanged, so that `is_lich` disk layout stays correct.
3. As a maintainer, I want riftwalker entity settings to keep array-index dispatch, so that four keys still map to one `[String; 4]` field.
4. As a maintainer, I want guild dialog dispatch to remain a subset of `SettingSlot`, so that `rig` and `is_lich` do not leak into dialog defaults.
5. As a test author, I want existing player profile tests to pass without modification, so that the refactor is behavior-preserving.
6. As a test author, I want a dispatch completeness invariant test, so that every `PersistSlot` / `SettingSlot` variant used in `SETTINGS_DEFS` has a working read/write path.
7. As a maintainer, I want the solution to stay readable without macro debugging skills, so that future settings work does not require macro expertise.
8. As a maintainer, I want no new runtime dependencies, so that Player Profile interpretation stays self-contained.

## Approach Options

**No option pre-selected.** The follow-up grill should pick one.

### Option A — Enum inherent methods

Move dispatch into `impl` blocks on `PersistSlot` and `SettingSlot`:

```rust
impl PersistSlot {
    pub(crate) fn read(&self, table: &SettingsTable, key: &str) -> String { ... }
    pub(crate) fn write(&self, table: &mut SettingsTable, key: &str, value: String, sparse: bool) { ... }
}

impl SettingSlot {
    pub(crate) fn read(&self, settings: &KnownProfileSettings) -> String { ... }
    pub(crate) fn write(&self, settings: &mut KnownProfileSettings, value: String) { ... }
    pub(crate) fn write_guild_dialog(&self, defaults: &mut GuildDialogProfileDefaults, value: String) { ... }
}
```

`read_persist` becomes `definition.persist.read(table, definition.key)` — thin wrappers or inlined at call sites.

| Pros | Cons |
| --- | --- |
| Idiomatic Rust; no function pointers or macros | Still one match arm per variant **per method** — dedup is organizational (one enum, three methods) not cardinality reduction |
| Easy to grep; each enum owns its field mapping | Adding a setting still touches enum definition + 2–3 impl methods |
| `Extra` / sparse / bool coercion localized in the right method | `SettingSlot` and `PersistSlot` variants must stay in sync manually |
| Fits const-table registry; no proc-macros | Five functions become thin delegators — net line reduction modest |

**Best when:** The goal is **one place per enum** to maintain arms, accepting that new variants still mean new impl arms but not scattered across five free functions.

### Option B — Per-row function pointers on `SettingDefinition`

Extend each registry row with dispatch hooks:

```rust
struct SettingDefinition {
    // ... existing fields ...
    read_persist: fn(&SettingsTable) -> String,
    write_persist: fn(&mut SettingsTable, String),
    read_slot: fn(&KnownProfileSettings) -> String,
    write_slot: fn(&mut KnownProfileSettings, String),
    write_guild_dialog: Option<fn(&mut GuildDialogProfileDefaults, String)>,
}
```

Each `SETTINGS_DEFS` row references small `fn` items like `read_rig_persist`, `write_rig_persist`, etc. Top-level dispatch functions disappear or become one-liner `definition.read_persist(table)`.

| Pros | Cons |
| --- | --- |
| Adding a setting = new row + new fn items only; **no central match growth** | ~4–5 tiny fns per setting → fn proliferation (9 settings × 4 ≈ 36 fns) |
| `Extra` row uses dedicated fns with `key` captured in closure... **cannot** — const table needs named fns taking `definition` or static key | `Extra` / sparse needs `definition` parameter on fn signature anyway |
| Call sites already have `&SettingDefinition` | Function pointers bypass enum exhaustiveness — completeness test becomes essential |
| True "registry row is the only edit site" if fn items are co-located with row | Harder to read than enum match; grep for field access less obvious |
| | `SETTINGS_DEFS` rows get wide; table harder to scan |

**Best when:** Cardinality reduction matters more than fn count; team accepts completeness tests over compiler exhaustiveness.

### Option C — Const lookup tables indexed by enum discriminant

Build `const` arrays parallel to enums:

```rust
type ReadPersistFn = fn(&SettingsTable, &str) -> String;

const READ_PERSIST: [ReadPersistFn; PersistSlot::COUNT] = [
    read_persist_rig,
    read_persist_tzarakk_mount,
    // ...
];
```

`read_persist` becomes `READ_PERSIST[definition.persist.index()](table, definition.key)`. Same pattern for write and `SettingSlot`.

| Pros | Cons |
| --- | --- |
| Central dispatch loops are O(1) table lookup — no `match` | Requires stable enum indexing (`repr(u8)` or manual `index()` const) |
| Adding a variant forces table length update (compile error if `COUNT` wrong) | Three parallel tables (read/write persist, read/write slot, guild dialog) |
| No proc-macros; const-constructible | `RiftwalkerEntity(usize)` breaks simple discriminant — needs either four fixed variants (already true for `PersistSlot`) or sub-dispatch inside one table entry |
| Moderate line reduction | Index maintenance burden; easy to mis-order table vs enum |
| | Less idiomatic than enum methods for Rust readers |

**Best when:** Team wants **one** generic read/write loop per target type and accepts manual index discipline.

## Open Questions (for grill)

1. **Option selection:** A (enum methods), B (per-row fn pointers), or C (lookup tables) — or a hybrid (e.g. A for `SettingSlot`, B for `PersistSlot` because `Extra` is special)?
2. **`write_guild_dialog_slot`:** Fold into `SettingSlot::write_guild_dialog` (Option A), `Option<fn>` on row (Option B), or separate small match retained as intentional subset?
3. **Completeness test shape:** Assert every `SETTINGS_DEFS` row's `persist` / `slot` variants round-trip a non-panicking read+write? Or static assertion that table length matches enum variant count?
4. **`PersistSlot` vs `SettingSlot` duplication:** Four `PersistSlot` riftwalker variants map to one `SettingSlot::RiftwalkerEntity(n)` — should this slice unify the enums (wider scope) or accept dual enums with shared indexing helpers?
5. **Line-count:** Does `persistence.rs` < 200 lines cap from deepen slice still apply after dedup, or relax for this slice?
6. **New tests:** Is a dispatch completeness test required in this slice, or is passing existing 11 tests sufficient?

## Testing Decisions (provisional — resolve in grill)

### Must pass

- All 11 existing `player_profile` tests unchanged (from deepen slice).

### Likely additions (pending grill)

- **Registry ↔ dispatch completeness:** Every `PersistSlot` value appearing in `SETTINGS_DEFS` round-trips through `read_persist` / `write_persist`; every `SettingSlot` through `read_known_slot` / `write_known_slot`.
- **`is_lich` sparse-extra:** Explicit write-falsy-removes-key case (may already be covered by existing tests — confirm before duplicating).
- **Guild dialog subset:** Only `guild_dialog = true` rows populate `GuildDialogProfileDefaults` fields (likely already covered).

### Avoid

- Testing internal table ordering or fn pointer addresses.
- Snapshotting match-arm source structure.

## Success Criteria (post-grill — fill during grilling)

- [ ] Adding a hypothetical 10th setting requires registry row + struct field only (no match-arm edits) — _or_ grill documents accepted trade-off if Option A chosen
- [ ] All 11 existing tests pass
- [ ] `is_lich` sparse-extra behavior unchanged
- [ ] Riftwalker four-key → `[String; 4]` mapping unchanged
- [ ] No new `pub` API surface
- [ ] No proc-macros introduced

## Dependencies

- **Blocked on:** [deepen-player-profile-settings](deepen-player-profile-settings/prd.md) (module split must land first)
- **Builds on:** [player-profile-settings-registry](player-profile-settings-registry/prd.md) (`SETTINGS_DEFS` unified registry)

## Further Notes

- Recommendation strength: **Moderate** — friction is real but only bites when adding settings (~9 today). ROI rises with each new setting.
- Option A is the smallest diff and lowest risk; Options B/C achieve stronger "one edit site" at readability cost. The grill should pick based on expected setting growth rate and team preference for exhaustiveness vs tables.
- If Option A is chosen, consider renaming this slice "dispatch consolidation" rather than "deduplication" — arms still exist, just colocated.
