# 02 — Dispatch round-trip completeness test

## Parent

`prd.md`

## What to build

Add one behavioral completeness test in the player profile integration test module that loops `SETTINGS_DEFS` and proves every registry row's persist and slot variants have working read/write paths.

For each row: write a sentinel value through `definition.persist.write`, read back via `definition.persist.read` on a default `SettingsTable`, and assert round-trip equality. Repeat for `definition.slot.write` / `definition.slot.read` on a default `KnownProfileSettings` — account for `IsLich` bool coercion (use a truthy sentinel or assert against the expected coerced form).

Scope is persist + known slot only. Do not cover guild-dialog or sparse-extra semantics in this test; those remain covered by existing targeted tests.

## Acceptance criteria

- [ ] One new test loops all `SETTINGS_DEFS` rows and round-trips persist read/write without panic
- [ ] Same test round-trips slot read/write for every row, with correct handling of bool coercion on `IsLich`
- [ ] Test lives alongside `registry_rows_are_complete_and_unique` in the player profile test module
- [ ] No duplication of guild-dialog or sparse-extra behavior already covered by `guild_dialog_defaults_follow_registry_flags` and `is_lich_*` tests
- [ ] All 11 existing `player_profile` tests still pass
- [ ] `cargo test --all-targets --all-features` passes

## Blocked by

- 01 — Dispatch consolidation

## Status

done
