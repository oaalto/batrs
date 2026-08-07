# How should unattributed drill navigation work?

**Status:** closed  
**Slice:** [`07-unattributed-drill-navigation.md`](../07-unattributed-drill-navigation.md)

## Question

When reviewing unattributed HP loss triggers one at a time, how does the player move to the next or previous trigger without returning to the landing table?

## Decision

- **Previous** / **Next** links on `/unattributed/{id}` drill-down only (not verb drill-down).
- Sequence = filtered landing list (`range`, `player`), same sort as landing (`recorded_at DESC, id DESC`).
- **Previous** = newer trigger (row above on landing); **Next** = older trigger (row below).
- Omit links at list ends, for single-item lists, or when current `id` is not in the filtered list.
- Plain link labels; no position counter in v1.
- Nav preserves `range` and `player` only; each GET marks reviewed (write-once).

## Rationale

Mirrors the landing table the user drilled from; no new queries or schema. Index into `list_unattributed` reuses existing filter SQL. Disabled-link stubs omitted to match minimal viewer HTML elsewhere.
