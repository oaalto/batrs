# 06 — Unattributed review state

## Parent

`prd.md`

## What to build

**Schema v5 migration** (`storage.rs`): add nullable `reviewed_at TEXT` to `unattributed_hp_events`. Bump `schema_version` to 5. Existing rows default to unreviewed (`reviewed_at IS NULL`).

**Aggregate queries** (`aggregate.rs`):

- `list_unattributed` includes `reviewed` flag (`reviewed_at IS NOT NULL`).
- `mark_unattributed_reviewed` — `UPDATE … SET reviewed_at = now WHERE id = ? AND reviewed_at IS NULL` (idempotent).

**HTTP viewer** (`viewer.rs`):

- Drill-down (`/unattributed/{id}`) opens writable DB, marks reviewed on successful load, logs warning on write failure, still renders page.
- Landing section header shows unreviewed count when &gt; 0 (e.g. “Unattributed HP loss (3 unreviewed)”).
- Landing table adds **Reviewed** column; reviewed rows use muted `tr.reviewed` styling.

## Blocked by

`04-unattributed-hp-review.md` (unattributed table and drill-down in place).

## Status

complete

## Acceptance criteria

- [x] Schema v5 migration adds `reviewed_at`; fresh DB at v5 includes column on `unattributed_hp_events`.
- [x] v4→v5 migration adds column; existing rows have `reviewed_at IS NULL`.
- [x] New collector inserts leave `reviewed_at` unset.
- [x] First GET of `/unattributed/{id}` sets `reviewed_at`; re-open does not overwrite.
- [x] 404 / missing id does not write `reviewed_at`.
- [x] Mark-write failure logs warning; drill-down still returns 200.
- [x] Landing shows Reviewed column, muted styling for reviewed rows, unreviewed count in header when &gt; 0.
- [x] Filters (`range`, `player`) apply to list and unreviewed count.
- [x] Collector and attribution aggregates unchanged.

## Tests

1. v4 DB migrates to v5 with `reviewed_at` column.
2. Drill-down marks row reviewed; landing shows reviewed styling and reduced unreviewed count.
3. Re-open drill-down is idempotent (single `reviewed_at` value).
4. HTTP fixture — landing + drill-down behavior unchanged aside from review metadata.
