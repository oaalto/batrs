---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

How should events be **stored** in `~/.batrs/combat_damage.db`?

Once event shape and weight are defined, lock:

- Table layout (events, sessions, raw context — normalized vs JSON blob)
- Indexes for viewer queries (by time, player, category, verb)
- `rusqlite` (sync) vs async SQLite crate — ponytail default?
- Write pattern: per-event insert vs batch on status line
- Schema version table and migration approach
- Retention: unlimited growth vs prune policy

### Q1 — table layout + batch grouping (resolved)

**Decision:** one flat **`damage_events`** table + **`schema_version`** table (single version integer row). No `sessions` table, no JSON blob column, no materialized aggregate tables in v1.

**`damage_events` columns (v1):**

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK AUTOINCREMENT | row id |
| `batch_id` | INTEGER NOT NULL | shared by all rows from one `H:` trigger; index for grouping ambiguous siblings |
| `recorded_at` | TEXT NOT NULL | ISO-8601 UTC when `H:` line seen |
| `player` | TEXT NOT NULL | metadata |
| `hp_delta` | INTEGER NOT NULL | positive magnitude |
| `hp_before` | INTEGER NOT NULL | from `H:` line |
| `hp_after` | INTEGER NOT NULL | from `H:` line |
| `damage_category` | TEXT NOT NULL | `melee` \| `skill` \| `spell` |
| `source_name` | TEXT NOT NULL | attacker; empty string when unknown (e.g. spell hit line) |
| `message_verb` | TEXT NOT NULL | aggregation key half |
| `message_text` | TEXT NOT NULL | full attributed line |
| `candidate_count` | INTEGER NOT NULL | filtered candidates in this batch |
| `weight` | REAL NOT NULL | `1.0` or `1/N` |
| `damage_min` | INTEGER NOT NULL | per-row bounds at insert |
| `damage_max` | INTEGER NOT NULL | per-row bounds at insert |

**`batch_id` usage:** one integer per negative-`H:` trigger; all sibling rows in an ambiguous batch share it; isolated rows still get a `batch_id` (sole member). Estimated-view extrapolation groups by `batch_id`; drill-down fetches `WHERE batch_id = ?`.

**`schema_version`:** one row, integer `version` — bumped on future migrations.

### Q2 — library and write pattern (resolved)

**Decision:** **`rusqlite`** (sync) + **one transaction per trigger** (`BEGIN` → N inserts sharing `batch_id` → `COMMIT`). DB opened once at collector init at `~/.batrs/combat_damage.db` via existing config dir helper.

### Q3 — indexes (resolved)

**Decision:** four indexes on `damage_events`:

| Index | Columns |
|-------|---------|
| `idx_damage_events_batch_id` | `(batch_id)` |
| `idx_damage_events_recorded_at` | `(recorded_at)` |
| `idx_damage_events_category_verb` | `(damage_category, message_verb)` |
| `idx_damage_events_candidate_count` | `(candidate_count)` |

No `player` index in v1.

### Q4 — schema versioning and migrations (resolved)

**Decision:** start at **`version = 1`**. On DB open: read `schema_version`; if missing, run `CREATE TABLE` + seed `version = 1`. If `version < CURRENT`: run numbered inline Rust migration steps (`1 → 2`, `2 → 3`, …) — no external migration tool. **No downgrade** — batrs versions that see a newer schema fail with a clear error (“DB schema newer than batrs; upgrade batrs”).

### Q5 — retention (resolved)

**Decision:** **unlimited growth** — no prune policy in v1. Rows accumulate indefinitely; manual delete if needed later.

## Resolution

**Path:** `~/.batrs/combat_damage.db` (global, via existing config dir helper).

**Tables:**

- `schema_version` — single row, integer `version` (starts at `1`)
- `damage_events` — flat event log; columns per Q1; `batch_id` groups sibling rows from one `H:` trigger

**Library / writes:** `rusqlite` (sync); DB opened once at collector init; one transaction per trigger (`BEGIN` → N inserts sharing `batch_id` → `COMMIT`).

**Indexes:** `batch_id`, `recorded_at`, `(damage_category, message_verb)`, `candidate_count`.

**Migrations:** inline numbered Rust steps on open; fail with clear error if DB schema is newer than batrs.

**Retention:** unlimited; no automatic prune in v1.
