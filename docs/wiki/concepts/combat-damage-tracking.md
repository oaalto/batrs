---
title: Combat Damage Tracking
type: concept
status: current
updated: 2026-10-29
sources:
  - CONTEXT.md
  - docs/hit_messages.md
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

HTTP viewer adds an **Unattributed HP loss** section: table of triggers (`recorded_at`, `player`, `hp_delta`, line count, reviewed); drill-down lists context lines in order with the triggering `H:` line appended as the final transcript item, even when context is empty. The triggering line remains in the metadata table too; storage is unchanged, so existing records benefit. Opening drill-down sets `reviewed_at` (schema v5, write-once); landing header shows unreviewed count when &gt; 0; reviewed rows render muted. **Previous** / **Next** on drill-down walk the filtered landing list (`recorded_at DESC, id DESC`); links omitted at ends or when the current trigger is outside the filter. **Remove reviewed** on the landing section deletes `reviewed_at IS NOT NULL` rows within the current `range` and `player` filters (`POST /unattributed/remove-reviewed`, 303 redirect). Filters (`range`, `player`) match the attribution dashboard. Time/player controls appear only on the landing page; both unattributed and attributed drill-downs show a read-only inherited-scope summary and a **Change filters on main page** link, right-aligned in the same row as navigation, with wrapping on narrow screens. Existing URL parameters carry the selection through entry, sibling, sort, Back, Previous/Next, and deletion-return links; no shared browser or server state is added. Direct links without parameters default to All time / All players, and already-open tabs keep their URL selection. Last 24h / Last 7d remain rolling windows. An unattributed record outside the active filters still displays its complete saved transcript, with an explicit explanation and no Previous/Next links; attributed lists continue to filter their events. Always on when the collector is active.

**Delete** appears in an Actions column on every unattributed landing row and beside the single-record page heading. Both forms submit `POST /unattributed/{id}/delete`: permanently remove that ID from `unattributed_hp_events`, regardless of review state, including its context and triggering line. There is a visible permanence warning on the landing page and a Delete-button tooltip on the single-record page, but no confirmation dialog or undo. The ID selects the record even if it falls outside the submitted filters; `range` and `player` are return context only. Success (including an already-absent ID) redirects with 303 to the filtered landing page's `#unattributed` section, never to another drill-down, so no neighboring record is automatically marked reviewed. Counts and navigation are recomputed from remaining rows. `damage_events` and attributed aggregates are untouched.

The single-delete handler rejects cross-origin browser requests using Origin/Fetch Metadata and rejects non-loopback Host names to guard against DNS rebinding. It accepts the viewer's `127.0.0.1` and `localhost` hosts (including custom ports); headerless non-browser clients are allowed. This is request-origin protection, not authentication. The existing bulk-removal route is unchanged. Single-delete database failures log the record ID and error, return HTTP 503 without a redirect, and do not claim success. Viewer writes reuse `open_validated_db`, without migrations or backfills.

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

## Heading anchors and navigation

All visible `h1`–`h3` headings in the local viewer have native `#` permalinks with accessible labels and keyboard focus styling. Stable targets are `combat-damage`, `melee`, `skill`, `spell`, `unattributed`, `melee-{family_id}`, `events`, and `context-lines`, as applicable. The landing page retains its existing `unattributed` wrapper ID; its heading links to that wrapper without a duplicate ID. The filter form additionally owns `filters`, without a new heading.

Heading permalinks use fragment-only URLs, preserving the path, filters, family selection, and sorting. Cross-page and sort links append fragments after their existing query strings:

- Attributed Back links target the landing category, or the selected melee family; family-less melee returns to `#melee`.
- Unattributed Back and both deletion returns target the filtered landing page's `#unattributed` section.
- **Change filters on main page** targets `#filters`, independently of Back. Apply and the general viewer entry URL still land at the top.
- Landing sort links target the category/family whose header was clicked; sort selection still applies across tables. Event sort links target `#events`.
- Landing verb links and drill-down verb/sibling links target `#events`; unattributed entry and Previous/Next links target the record's `#unattributed` heading, not its transcript. `#context-lines` is available as an explicit permalink.
- Empty top-level sections retain targets; melee families without rows remain hidden. A bookmarked family fragment may have no target under different filters.

These are rendering/navigation changes only: routes, query-selection semantics, database storage, review writes, and deletion scope remain unchanged. Sources: `src/combat_damage/viewer.rs`, `assets/combat_damage/style.css`.

## Drill-down and batch siblings

Verb drill-down (`/events/{category}/{verb}`) lists focal rows for that verb. Optional `?family=` filters melee focal rows by `weapon_family` (e.g. disambiguate `savagely strike` in `bash` vs `claw`); page title includes the family when set. When a focal row belongs to an ambiguous attribution batch (`candidate_count > 1`), sibling rows from the same `batch_id` appear inline immediately after it — including cross-category siblings. Focal rows sort by the chosen column; siblings follow their focal row, sub-sorted by category then verb. Sibling rows are styled distinctly and link to their own verb drill-down.

## Verified Facts

- Incoming hit lines between short-score `H:` updates are attributed on a negative `H:` HP bracket; rows are written to `~/.batrs/combat_damage.db` and served by the HTTP viewer (default `127.0.0.1:6464`).
- Attribution `weight` is `1.0` for an isolated candidate (`candidate_count = 1`) and `1.0/N` for ambiguous batches; it is confidence, not hit severity (`src/combat_damage/attribution.rs`).
- A negative `H:` with zero recognized candidates writes one `unattributed_hp_events` row (schema v4/v5) holding an ordered JSON `context_lines` array (ADR `0002`).
- Riposte is a two-line enemy skill — `<name> parries.` then `...AND counterattacks.`/`...AND ripostes.` — producing a `skill` / `riposte` candidate (`matcher.rs::match_skill`); a standalone follow-up or parry line alone is never a candidate.
- Riposte and skill attribution of previously unattributed rows is backfilled on every `open_db()` (`migrate()` → `backfill_riposte_from_unattributed` / `backfill_skills_from_unattributed`, `src/combat_damage/storage.rs`); converted rows are deleted.

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Combat Damage section
- [Secondary Status](../concepts/secondary-status.md)
- Hit catalog: `docs/hit_messages.md`
- ADR `0002-unattributed-hp-loss-outside-damage-events.md`
