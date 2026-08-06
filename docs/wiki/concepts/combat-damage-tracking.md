---
title: Combat Damage Tracking
type: concept
status: current
updated: 2026-08-06
sources:
  - CONTEXT.md
  - docs/hit_messages.md
  - docs/features/combat-damage-tracking/prd.md
  - src/combat_damage/
---

# Combat Damage Tracking

## Summary

Combat Damage records **incoming HP loss** while the player is logged in: recognized hit lines between short-score `H:` updates are attributed when an `H:` line shows a negative HP bracket. Rows land in `~/.batrs/combat_damage.db`; a local HTTP dashboard (default `127.0.0.1:6464`) shows per-verb **confirmed** and **estimated** aggregates.

## Key terms

| Term | Meaning |
| --- | --- |
| Attribution `weight` | `0.0`–`1.0` confidence that this filtered candidate caused the observed `hp_delta`. `1.0` when exactly one candidate between `H:` lines; `1.0/N` when ambiguous. Not hit severity. |
| `catalog_rank` | Melee only: integer `1`–`26` within a weapon family from [`hit_messages.md`](../../hit_messages.md) ascending damage order. Compile-time from catalog line numbers. |
| `weapon_family` | Melee catalog family id (`slash`, `bash`, `pierce`, …). Persisted on rows so verbs that collide across families stay disambiguated. |
| Attribution batch | All rows written from one negative `H:` trigger; share a `batch_id`. Isolated batches have one row; ambiguous batches have multiple **batch siblings**. |
| Batch sibling | Another candidate row in the same attribution batch (`candidate_count > 1`). Drill-down for one verb shows focal rows for that verb plus inline siblings (any category). |
| Confirmed view | Aggregates from isolated rows (`candidate_count = 1`) only — exact `hp_delta` per observation. |
| Estimated view | Read-time extrapolation on ambiguous batches using isolated-derived `known_min`/`known_max` per verb key. Bounds stay conservative; no even-split fallback. |
| Rank-estimated avg | When estimated bounds stay loose `[0, hp_delta]` and batch rows carry `catalog_rank`, point estimate `hp_delta × (rankᵢ / Σ rankⱼ)` over ranked melee candidates. Does not change confirmed numbers or stored `weight`. |

## Module boundary

`src/combat_damage/` owns matchers (skills → spells → melee catalog), `DamageCollector` buffer + SQLite inserts, aggregate queries, and HTTP viewer wiring.

Combat Damage does **not** own combat round state, short-score stats mutation, or combat-awareness gagging (collector runs **before** gag/continue so gagged scan lines still buffer).

## Catalog rank vs weight

- **`weight`** answers: "How sure is this line the cause?" — crowding after filtering damage candidates only.
- **`catalog_rank`** answers: "How hard does this verb usually hit?" — domain prior from catalog order when isolated data has not yet tightened bounds.

Isolated `known_min`/`known_max` from confirmed observations always override rank. Rank never narrows stored `damage_min`/`damage_max` at insert time.

## Drill-down and batch siblings

Verb drill-down (`/events/{category}/{verb}`) lists focal rows for that verb. When a focal row belongs to an ambiguous attribution batch (`candidate_count > 1`), sibling rows from the same `batch_id` appear inline immediately after it — including cross-category siblings. Focal rows sort by the chosen column; siblings follow their focal row, sub-sorted by category then verb. Sibling rows are styled distinctly and link to their own verb drill-down.

## Further reading

- PRD: `docs/features/combat-damage-tracking/prd.md`
- Hit catalog: `docs/hit_messages.md`
- Closed tickets: `docs/features/combat-damage-tracking/tickets/how-should-attribution-weight-work.md`, `how-should-catalog-rank-inform-estimated-extrapolation.md`
