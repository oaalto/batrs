# 07 — Unattributed drill-down navigation

## Parent

`prd.md`

## What to build

**HTTP viewer** (`viewer.rs`):

- On `/unattributed/{id}` drill-down, show **Previous** and **Next** links in the top nav row beside **Back** when neighbors exist in the **filtered landing list** (`list_unattributed` with current `range` and `player`).
- Neighbor order matches the landing table: `ORDER BY recorded_at DESC, id DESC` — **Previous** = row above (newer trigger), **Next** = row below (older trigger).
- Omit a link when there is no neighbor (ends of list, single-item list, or current `id` not in the filtered list).
- Nav links preserve `range` and `player` only (`build_filter_query`); no `sort`/`dir`/`family`.
- Opening via Previous/Next marks the target trigger reviewed (same as direct drill-down).

No schema, collector, or aggregate query changes.

## Blocked by

`06-unattributed-review-state.md` (drill-down and landing review UX in place).

## Status

complete

## Acceptance criteria

- [x] Middle trigger in a three-item filtered list shows both Previous and Next with correct target ids.
- [x] Newest trigger shows Next only; oldest shows Previous only.
- [x] Single trigger in filtered list shows Back only (no Previous/Next).
- [x] Deep link to trigger excluded by `player` filter still renders detail but omits both nav links.
- [x] Nav hrefs include current `range` and `player` query params when set.
- [x] Sequential nav GETs mark each visited trigger reviewed (existing `mark_unattributed_reviewed`).

## Tests

1. Unit: `unattributed_neighbor_ids` index logic for first/middle/last/missing id.
2. HTTP fixture: three triggers — newest/middle/oldest nav link presence and href targets.
3. HTTP fixture: single trigger — no nav links.
4. HTTP fixture: `?player=` excludes current id — no nav links.
