---
title: Combat Damage Tracking
type: concept
status: current
updated: 2026-08-07
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
| `weapon_family` | Melee catalog family id (`slash`, `bash`, `pierce`, …). Persisted on rows so verbs that collide across families stay disambiguated. Melee aggregation and landing sub-sections key on this field. |
| Attribution batch | All rows written from one negative `H:` trigger; share a `batch_id`. Isolated batches have one row; ambiguous batches have multiple **batch siblings**. |
| Batch sibling | Another candidate row in the same attribution batch (`candidate_count > 1`). Drill-down for one verb shows focal rows for that verb plus inline siblings (any category). |
| Confirmed view | Aggregates from isolated rows (`candidate_count = 1`) only — exact `hp_delta` per observation. |
| Estimated view | Read-time extrapolation on ambiguous batches using isolated-derived `known_min`/`known_max` per verb key. Bounds stay conservative; no even-split fallback. |
| Rank-estimated avg | When estimated bounds stay loose `[0, hp_delta]` and batch rows carry `catalog_rank`, point estimate `hp_delta × (rankᵢ / Σ rankⱼ)` over ranked melee candidates. Does not change confirmed numbers or stored `weight`. |
| Unattributed HP loss | Negative `H:` HP bracket with zero recognized damage candidates in the window since the previous `H:` line. Stored for review, not in `damage_events`. Distinct from ambiguous batches (N≥2 candidates) and from melee `weapon_family = unknown` in rollups. |
| Context window | All plain incoming lines between the previous `H:` and a triggering `H:` line (exclusive of both `H:` lines). Saved verbatim on unattributed triggers; not stored on attributed rows (`message_text` only). |
| Riposte | Two-line enemy skill: `<name> parries.` then `...AND counterattacks.` or `...AND ripostes.` — `skill` / `riposte`. Parry line alone is setup, not a candidate. |

## Unattributed HP loss review

When a negative `H:` line arrives with **zero** filtered damage candidates, the collector writes one row to `unattributed_hp_events` (schema v4) with `recorded_at`, `player`, `hp_delta`, `hp_before`, `hp_after`, `h_line_text`, and ordered `context_lines` (JSON array). Empty context is valid (silent damage between `H:` lines).

- **Not** ambiguous batches: N≥1 candidates still follow the existing `damage_events` path only.
- **Not** `damage_category = unknown`: confirmed/estimated rollups remain `candidate_count ≥ 1` only.
- Context window includes every line passed to `handle_line` while logged in except `H:` delimiters — including gagged scan lines (collector runs before combat-awareness gag).
- Lifecycle: context window clears on every `H:` line, `reset_buffer()`, logout, and `FreshSessionReset::DamageCollector` — same as the candidate buffer.
- Write failure: `tracing::warn!`, discard pending context, continue play.

HTTP viewer adds an **Unattributed HP loss** section: table of triggers (`recorded_at`, `player`, `hp_delta`, line count, reviewed); drill-down lists context lines in order plus the triggering `H:` line. Opening drill-down sets `reviewed_at` (schema v5, write-once); landing header shows unreviewed count when &gt; 0; reviewed rows render muted. **Previous** / **Next** on drill-down walk the filtered landing list (`recorded_at DESC, id DESC`); links omitted at ends or when the current trigger is outside the filter. **Remove reviewed** on the landing section deletes `reviewed_at IS NOT NULL` rows within the current `range` and `player` filters (`POST /unattributed/remove-reviewed`, 303 redirect). Filters (`range`, `player`) match the attribution dashboard. Always on when the collector is active.

## Module boundary

`src/combat_damage/` owns matchers (skills → spells → melee catalog), `DamageCollector` buffer + SQLite inserts, aggregate queries, and HTTP viewer wiring.

**Riposte** is the first two-line skill pattern: enemy `parries.` sets pending state; the immediately following `...AND counterattacks.` or `...AND ripostes.` line becomes a `skill` / `riposte` candidate. Any intervening line clears pending state. Parry lines alone are never candidates (Q6 footnote — distinct from riposte follow-up).

Combat Damage does **not** own combat round state, short-score stats mutation, or combat-awareness gagging (collector runs **before** gag/continue so gagged scan lines still buffer).

## Catalog rank vs weight

- **`weight`** answers: "How sure is this line the cause?" — crowding after filtering damage candidates only.
- **`catalog_rank`** answers: "How hard does this verb usually hit?" — domain prior from catalog order when isolated data has not yet tightened bounds.

Isolated `known_min`/`known_max` from confirmed observations always override rank. Rank never narrows stored `damage_min`/`damage_max` at insert time.

## Landing layout

- Three top-level damage tables: **Melee**, **Skill**, **Spell**.
- **Melee** groups rows into `weapon_family` sub-sections (catalog file order from `hit_messages.md`; families with no rows under current filters are hidden). Each sub-section is one table with the same confirmed/estimated columns. Default verb order within a family: `catalog_rank` ascending, then `message_verb` ascending. Column sort applies within each family only.
- Skill and spell remain flat single tables keyed on `damage_category` + `message_verb`.

## Drill-down and batch siblings

Verb drill-down (`/events/{category}/{verb}`) lists focal rows for that verb. Optional `?family=` filters melee focal rows by `weapon_family` (e.g. disambiguate `savagely strike` in `bash` vs `claw`); page title includes the family when set. When a focal row belongs to an ambiguous attribution batch (`candidate_count > 1`), sibling rows from the same `batch_id` appear inline immediately after it — including cross-category siblings. Focal rows sort by the chosen column; siblings follow their focal row, sub-sorted by category then verb. Sibling rows are styled distinctly and link to their own verb drill-down.

## Further reading

- PRD: `docs/features/combat-damage-tracking/prd.md`
- Hit catalog: `docs/hit_messages.md`
- Closed tickets: `docs/features/combat-damage-tracking/tickets/how-should-attribution-weight-work.md`, `how-should-catalog-rank-inform-estimated-extrapolation.md`, `how-should-unattributed-hp-loss-be-captured.md`
