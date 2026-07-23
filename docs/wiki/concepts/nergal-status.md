---
title: Nergal Status
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/guilds/nergal/triggers.rs
  - src/stats.rs
---

# Nergal Status

## Summary

Nergal Resource Status is the player's current Nergal-specific resource state: Vitae, Potentia, and Evolution points.

## Verified Facts

- Parsed from BatMUD line matching `::..:. [Vitae: N/M  Potentia: N/M, Evolution points: N]` in `src/guilds/nergal/triggers.rs` when Nergal is in guild selection.
- Stored as `NergalResourceStatus` and rendered in the stats panel (`src/stats.rs`).
- Matching lines are gagged from scrollback output when Nergal guild is selected.
- Resource status line is not processed when Nergal guild is not selected (see `src/app/mod.rs` tests).

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Nergal Status section
