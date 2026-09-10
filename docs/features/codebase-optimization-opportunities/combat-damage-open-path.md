# Separate Combat Damage routine opens from migration and backfill

## Parent

`docs/features/codebase-optimization-opportunities/prd.md`

## What to build

Make ordinary Combat Damage database usage stop rerunning heavyweight schema-maintenance and historical-backfill work when that work is not needed. From the user's perspective, normal batrs startup and normal Combat Damage viewer actions should preserve the same behavior and stored-data semantics while avoiding unnecessary database-open cost.

## Acceptance criteria

- [x] Normal batrs startup preserves current Combat Damage behavior without rerunning unnecessary migration/backfill work on ordinary opens.
- [x] Combat Damage viewer actions that write local review state preserve current behavior and stored-data semantics without paying full routine-maintenance cost on every open.
- [x] Schema upgrades and any required historical maintenance still run when genuinely needed.
- [x] Existing Combat Damage semantics remain unchanged, including ADR-backed melee aggregation and separate unattributed HP-loss storage.

## Blocked by

- None (can start immediately).

## Status

done
