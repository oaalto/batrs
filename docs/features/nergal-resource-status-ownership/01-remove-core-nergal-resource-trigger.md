# 01 — Remove core Nergal resource status trigger

## Parent

`prd.md`

## What to build

Eliminate the duplicate global parser for Nergal Resource Status so only the Nergal guild trigger handles the Vitae/Potentia/Evolution line when Nergal is selected. A maintainer editing the BatMUD line pattern has one place to change; a logged-in player without Nergal selected no longer gets silent gag-and-stats updates from a global trigger.

## Acceptance criteria

- [ ] Standalone core Nergal resource status trigger module deleted; removed from global core trigger registration
- [ ] Nergal guild trigger remains sole parser for the resource status line when Nergal is in guild selection
- [ ] With Nergal selected: resource status line still gagged and `SetNergalResourceStatus` applied (guild trigger path)
- [ ] With Nergal selected: no duplicate stats effect from a second parser on the same line
- [ ] Unit tests from deleted core trigger migrated into Nergal guild trigger tests where not already covered (gag, effect values, strict field order)
- [ ] `cargo test` passes

## Blocked by

None — can start immediately.

**Status:** ready-for-agent
