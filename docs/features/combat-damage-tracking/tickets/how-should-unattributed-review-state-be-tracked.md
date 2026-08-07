# How should unattributed review state be tracked?

**Status:** closed  
**Slice:** [`06-unattributed-review-state.md`](../06-unattributed-review-state.md)

## Question

When reviewing unattributed HP loss triggers in the HTTP dashboard, how does the player tell which triggers they have already opened?

## Decision

- Add nullable `reviewed_at TEXT` to `unattributed_hp_events` (schema v5).
- First successful GET of `/unattributed/{id}` sets `reviewed_at` to current UTC timestamp; re-open is idempotent (write-once).
- Landing table shows a **Reviewed** column and muted styling for reviewed rows; section header includes unreviewed count when &gt; 0.
- Mark-write failure logs a warning and still renders drill-down.
- No manual toggle; no filter param in v1; collector and attribution aggregates unchanged.

## Rationale

Persistent SQLite state survives refresh and batrs restarts. Drill-down open matches the user's mental model of “I looked at this one.” Separate `localStorage` would not survive across browsers and would be invisible to tests.
