# 01 — Remove core Nergal resource status trigger

## Status

superseded — absorbed by `docs/features/secondary-status-extraction/` (ticket 01, grilled 2026-07-23). Nergal parsing cleanup and Secondary Status ownership ship in that slice; do not implement separately.

## Parent

`prd.md`

## What to build

Eliminate the duplicate global parser for Nergal Resource Status so only the Nergal guild trigger handles the Vitae/Potentia/Evolution line when Nergal is selected. A maintainer editing the BatMUD line pattern has one place to change; a logged-in player without Nergal selected no longer gets silent gag-and-stats updates from a global trigger.

## Acceptance criteria

- [x] Standalone core Nergal resource status trigger module deleted; removed from global core trigger registration
- [x] Nergal guild trigger remains sole parser for the resource status line when Nergal is in guild selection
- [x] With Nergal selected: resource status line still gagged and `SetNergalResourceStatus` applied (guild trigger path)
- [x] With Nergal selected: no duplicate stats effect from a second parser on the same line
- [x] Unit tests from deleted core trigger migrated into Nergal guild trigger tests where not already covered (gag, effect values, strict field order)
- [x] `cargo test` passes

## Blocked by

None — can start immediately.
