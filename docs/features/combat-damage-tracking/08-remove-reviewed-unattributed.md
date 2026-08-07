# 08 — Remove reviewed unattributed triggers

## Parent

`prd.md`

## What to build

**Aggregate queries** (`aggregate.rs`):

- `delete_reviewed_unattributed(conn, filters) -> Result<usize, String>` — `DELETE FROM unattributed_hp_events WHERE reviewed_at IS NOT NULL` plus existing `filter_clause` (`range`, `player`).

**HTTP viewer** (`viewer.rs`):

- `POST /unattributed/remove-reviewed` with hidden `range` / `player` form fields; **303 See Other** redirect back to `/` preserving filters.
- Landing unattributed section header shows **Remove reviewed** button only when the filtered list has ≥1 reviewed trigger.
- Writable `open_db` for delete; log warning on failure; redirect either way.

No schema, collector, or `damage_events` changes.

## Blocked by

`06-unattributed-review-state.md` (reviewed state and landing UX in place).

## Status

complete

## Acceptance criteria

- [x] `delete_reviewed_unattributed` deletes only rows with `reviewed_at IS NOT NULL` within current filters.
- [x] POST removes reviewed rows; unreviewed triggers in the same filter survive.
- [x] Reviewed row outside `player` filter is not deleted when purge runs with that filter.
- [x] **Remove reviewed** button hidden when filtered list has zero reviewed rows.
- [x] Redirect after POST preserves `range` and `player`.
- [x] Delete failure logs warning; redirect still returns 303.
- [x] Collector and attribution aggregates unchanged.

## Tests

1. HTTP fixture: two reviewed + one unreviewed → POST deletes only reviewed; landing shows one row; button gone.
2. HTTP fixture: reviewed rows for two players → POST with `player=Odefu` deletes only Odefu row.
3. HTTP fixture: all unreviewed → landing has no **Remove reviewed** button.
