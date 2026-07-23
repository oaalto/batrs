# 02 — UI owns combat status rendering

## Parent

`prd.md`

## What to build

Move combat status presentation out of Combat Awareness into the UI layer. Combat Awareness exposes snapshot data only (`CombatScanRow` or equivalent); the UI module renders condition-colored, width-wrapped combat status rows for the HUD. Player-visible combat panel output is unchanged from before ticket 02.

## Blocked by

- [01 — Combat Awareness module + unified app fan-out](01-combat-awareness-module-and-fan-out.md)

## Status

ready-for-agent

## Acceptance criteria

- [ ] Combat Awareness public API exposes snapshot rows only; no ratatui types in the domain module.
- [ ] UI module renders combat status lines from snapshot data (condition coloring, width wrapping).
- [ ] Application `draw` uses the UI renderer; combat HUD appearance matches pre-ticket behavior.
- [ ] Rendering tests live at the UI layer or adapt to snapshot-in → lines-out; Combat Awareness tests cover data/effects only.
- [ ] `cargo test` passes.
