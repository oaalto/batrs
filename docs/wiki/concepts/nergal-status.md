---
title: Nergal Status
type: concept
status: current
updated: 2026-10-29
sources:
  - CONTEXT.md
  - src/guilds/nergal/triggers.rs
  - src/secondary_status.rs
  - src/stats.rs
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

## Dreary Hibernation Readiness

- Dreary hibernation readiness belongs to the **main-line recovery bracket** in `Stats`, not Secondary Status. `Stats::render_inline` receives guild selection and renders normal-colored `d` after `c` and `m` only while Nergal is selected.
- The Nergal guild trigger emits `StatsEffect::SetRecoveryBracketDrearyHibernation(false)` for the exact cocoon start message and `true` for the exact harmony message; both remain visible in scrollback. Parsing respects the guild-trigger switch.
- The harmony message also restores meditation through the existing core recovery trigger when core triggers are enabled; neither trigger tries to infer which skill produced it.
- Login defaults readiness to true. Newly selecting Nergal initializes it to true, unchanged selections preserve it, and deselecting clears it. These are optimistic defaults, not server-confirmed cooldown state. Fresh-session reset clears it; prompt and short-score updates preserve it (`src/app/mod.rs`, `src/stats.rs` tests).

## Related

- [Secondary Status](secondary-status.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Nergal Status section
