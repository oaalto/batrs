# 04 — Unattributed HP loss review

## Parent

`prd.md`

## What to build

Extend **DamageCollector** with a parallel **context window** buffer: append every plain incoming line (except `H:` delimiters) while logged in. On negative `H:` with **zero** recognized damage candidates, persist one `unattributed_hp_events` row (schema v4) with ordered `context_lines` (JSON array) and `h_line_text`. On negative `H:` with N≥1 candidates, existing `damage_events` path only — no unattributed row.

**Schema v4 migration** (`storage.rs`): add `unattributed_hp_events` table with columns `id`, `recorded_at`, `player`, `hp_delta`, `hp_before`, `hp_after`, `h_line_text`, `context_lines` (TEXT JSON). Indexes on `recorded_at` and `player`. Bump `schema_version` to 4.

**Collector changes** (`collector.rs`):

- Dual buffer: candidate buffer (unchanged) + context window (all non-`H:` lines).
- `flush_on_h_line`: when `candidates.is_empty()` and `hp_delta > 0`, insert unattributed row instead of no-op.
- Clear both buffers on every `H:` line, `reset_buffer()`, and session reset.

**HTTP viewer** (`viewer.rs`):

- Landing: **Unattributed HP loss** section — table of triggers (`recorded_at`, `player`, `hp_delta`, line count).
- Drill-down route (e.g. `/unattributed/{id}`): ordered context lines + triggering `h_line_text`.
- Filters `range` and `player` apply; empty section shows hint text (200, not error).

## Blocked by

`03-http-damage-viewer.md` (viewer framework and filter patterns in place).

## Status

complete

## Acceptance criteria

- [x] Schema v4 migration creates `unattributed_hp_events`; fresh DB at v4 includes both `damage_events` and unattributed table.
- [x] Negative `H:` with zero candidates → 0 `damage_events` rows, 1 `unattributed_hp_events` row with `context_lines` matching lines since previous `H:`.
- [x] Negative `H:` with empty context window → unattributed row with `context_lines = []`.
- [x] Negative `H:` with N≥1 candidates → N `damage_events` rows, 0 unattributed rows (unchanged).
- [x] Context window includes gagged scan lines and outgoing `You …` lines when zero candidates match.
- [x] Every `H:` line (positive, negative, empty bracket) clears context window.
- [x] `reset_buffer()` and `FreshSessionReset::DamageCollector` clear context window without closing DB.
- [x] Write failure logs warning, clears buffers, does not panic.
- [x] HTTP landing shows unattributed section; drill-down lists context in order.
- [x] HTTP filters (`range`, `player`) apply to unattributed queries.
- [x] Attributed aggregates (confirmed/estimated) unchanged — no unattributed rows in rollups.

## Tests

Per PRD § Unattributed review test matrix:

1. Miss-only window → unattributed row with miss lines in context.
2. Silent loss (empty window) → unattributed row, empty JSON array.
3. Recognized hit → `damage_events` only.
4. Ambiguous N=2 → two `damage_events`, no unattributed.
5. Gagged line in context when zero candidates.
6. Reset lifecycle clears context.
7. HTTP fixture → 200, trigger table + drill-down content.
