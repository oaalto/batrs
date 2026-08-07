# How should reviewed unattributed triggers be purged?

**Status:** closed  
**Slice:** [`08-remove-reviewed-unattributed.md`](../08-remove-reviewed-unattributed.md)

## Question

After reviewing unattributed HP loss triggers, how does the player clear inspected rows from the landing table?

## Decision

- Landing **Remove reviewed** button when the filtered list has ≥1 reviewed trigger (`reviewed_at IS NOT NULL`).
- `POST /unattributed/remove-reviewed` with hidden `range` / `player` fields; **303** redirect to `/` preserving filters.
- Permanent `DELETE` of matching `unattributed_hp_events` rows scoped to current `range` and `player` filters.
- No confirmation dialog in v1; no schema bump; collector and attribution aggregates unchanged.
- Slice-06 review UX (`reviewed_at`, auto-mark on drill-down, Reviewed column, prev/next) unchanged.

## Rationale

Reviewed triggers are inspection-complete; bulk delete keeps the landing inbox manageable without hiding rows behind extra filter state. Filter-scoped delete matches landing table semantics and avoids surprising cross-player data loss. POST + redirect is the viewer's first mutation route and avoids destructive GET.
