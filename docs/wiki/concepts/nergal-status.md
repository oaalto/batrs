---
title: Nergal Status
type: concept
status: current
updated: 2026-10-29
sources:
  - CONTEXT.md
  - src/guilds/nergal/triggers.rs
  - src/secondary_status.rs
  - src/app/mod.rs
---

# Nergal Status

## Summary

Nergal Resource Status is the player's current Nergal-specific resource state: Vitae, Potentia, and Evolution points. The Nergal guild module owns parsing and gagging; Secondary Status owns storage, effects, and HUD rendering.

## Verified Facts

- Parsed from BatMUD line matching `::..:. [Vitae: N/M  Potentia: N/M, Evolution points: N]` in `src/guilds/nergal/triggers.rs` when Nergal is in guild selection.
- Stored as `NergalResourceStatus` in Secondary Status; minions stored in a three-slot array (`src/secondary_status.rs`).
- Guild trigger emits `SecondaryStatusEffect::SetNergalResourceStatus`, `UpsertNergalMinion`, or `ClearNergalMinions` via `TriggerEffects.secondary_status`.
- Matching lines are gagged from scrollback output when Nergal guild is selected.
- Resource status line is not processed when Nergal guild is not selected (see `src/app/mod.rs` tests).
- HUD rows render only when `GuildKey::Nergal` is in guild selection — no fallback on stored minion or resource status presence (`src/secondary_status.rs`, `src/app/mod.rs`).
- Deselecting Nergal calls `SecondaryStatus::sync_guild_selection`, which clears Nergal minions and resource status (`src/app/mod.rs`).
- The client sends literal `nergal sc` once on successful login with Nergal selected, including login after `/connect`, and once when Nergal is newly selected while logged in. Repeated prompts and unchanged selection saves do not resend it. Profile loading refreshes newly selected Nergal; the post-login branch refreshes already-selected Nergal without duplicating that send (`src/app/mod.rs`).

## Related

- [Secondary Status](secondary-status.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Nergal Status section
