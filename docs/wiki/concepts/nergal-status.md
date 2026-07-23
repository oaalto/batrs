---
title: Nergal Status
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/triggers/nergal_resource_status.rs
  - src/stats.rs
---

# Nergal Status

## Summary

Nergal Resource Status is the player's current Nergal-specific resource state: Vitae, Potentia, and Evolution points.

## Verified Facts

- Parsed from BatMUD line matching `::..:. [Vitae: N/M  Potentia: N/M, Evolution points: N]` (`src/triggers/nergal_resource_status.rs`).
- Stored as `NergalResourceStatus` and rendered in the stats panel (`src/stats.rs`).
- Matching lines are gagged from scrollback output.
- Guild-specific triggers also exist under `src/guilds/nergal/triggers.rs`; shared parser in `src/triggers/nergal_resource_status.rs`.
- Resource status line is only processed when Nergal guild is selected (see `src/app/mod.rs` tests).

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Nergal Status section
