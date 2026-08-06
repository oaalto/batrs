# 03 — HTTP damage viewer

## Parent

`prd.md`

## What to build

Add a read-only HTTP dashboard served by `axum`, auto-started when batrs launches (background thread), bind `127.0.0.1`, port from `--port` flag (default **6464**). Server-rendered HTML with bundled CSS and small inline JS for filters and column sorting.

**Landing (`/`):** three tables (melee, skill, spell). Each row = one `message_verb` with side-by-side **confirmed** and **estimated** columns (obs count, min, max, avg/bounds). Filters: time range (`24h`, `7d`, `all`) and player dropdown; defaults all; query params preserve state. All columns sortable; default sort verb ascending. No total-damage line, no by-monster table.

**Drill-down (`/events/{category}/{verb}`):** event list with `recorded_at`, `player`, `hp_delta` (or min–max for ambiguous), `source_name`, `weight`, `candidate_count`, `verb` (linked), `message_text`; ambiguous batches show inline **batch siblings** (other candidates from the same `batch_id`, including cross-category). Default sort `recorded_at` descending. Same filters as landing.

**Aggregates:** confirmed from `candidate_count = 1` only; estimated applies conservative constraint extrapolation at read time per PRD (no write-back). Aggregation key: `damage_category` + `message_verb`.

**Static assets:** `style.css` — zebra rows, hover, grouped confirmed/estimated headers, distinct wide-bound styling.

**Empty state (zero rows):** see PRD § Empty state and first launch. Landing and drill-down always **200** with full headers, filter form, three section tables (landing) or event table (drill-down), zero data rows, per-section empty hint. Player dropdown = *All players* only. **503** only for DB open failure or schema newer than binary — not for empty data.

**Startup:** HTTP thread starts after `open_db` has created/opened the database at batrs init (slice 02). Viewer uses read-only connection to same path.

## Blocked by

`02-damage-collector-and-storage.md` (database populated with real schema).

## Status

done

## Acceptance criteria

- [ ] HTTP server starts automatically on batrs launch; stops on batrs exit.
- [ ] `--port` flag with default `6464`; binds `127.0.0.1` only.
- [ ] Read-only DB access; concurrent with collector writes (SQLite WAL or separate read connection acceptable).
- [ ] `/` renders three category tables with confirmed + estimated columns.
- [ ] `/events/{category}/{verb}` renders drill-down with batch grouping for shared `batch_id`.
- [ ] Filters `range` and `player` via query params on landing and drill-down.
- [ ] Column header click toggles sort asc/desc; sort state in query params; landing default verb asc; drill-down default `recorded_at` desc.
- [ ] Bundled CSS served; tables readable (zebra, hover, column groups).
- [ ] Estimated columns reflect extrapolation rules from PRD (conservative; loose bounds visually distinct).
- [ ] Empty DB: `GET /` → 200; Melee/Skill/Spell headings; confirmed + estimated headers; zero data rows; empty-hint text per section.
- [ ] Empty DB: player filter shows *All players* only.
- [ ] Empty DB: `GET /events/melee/{verb}` → 200 (not 404); empty event table + back link to `/`.
- [ ] Empty DB: `GET /style.css` → 200 with non-empty CSS.
- [ ] DB schema newer than binary → 503 with clear message (no panic).
- [ ] DB unreadable → 503; batrs TUI continues.
- [ ] `cargo test --all-targets --all-features` passes.

## Tests (required)

Fixture DB builder helper: insert rows with known `batch_id`, `candidate_count`, `weight`, `damage_min`, `damage_max`, `recorded_at`, `player`, category, verb — reused across aggregate and HTTP tests.

### Aggregate / extrapolation (pure functions + SQL)

- [ ] Confirmed rollup: only `candidate_count = 1` rows contribute min/max/avg/count per verb.
- [ ] Estimated rollup: ambiguous rows contribute `[0, hp_delta]` bounds before extrapolation.
- [ ] Extrapolation: when one candidate's isolated known-min equals batch `hp_delta`, estimated view assigns full delta to that verb.
- [ ] Extrapolation: when sum of known-min across candidates exceeds `hp_delta`, handled per PRD (cap or flag — assert chosen behavior).
- [ ] Extrapolation: unresolved ambiguous batch keeps loose bounds in estimated columns.
- [ ] Filter `range=24h`: rows older than 24h excluded from rollups.
- [ ] Filter `range=7d`: seven-day boundary.
- [ ] Filter `player=Odefu`: only that player's rows in rollups and drill-down.
- [ ] Default filters: all time, all players when query params absent.
- [ ] Sort landing by verb asc/desc changes result order.
- [ ] Sort drill-down by `recorded_at` asc/desc changes result order.
- [ ] Melee / skill / spell verbs appear only in their category rollup.

### Empty state and first launch

- [ ] Fresh temp DB (v1 schema, zero rows) → `GET /` 200; body contains `Melee`, `Skill`, `Spell`; confirmed and estimated column headers; no fixture verb string; empty-hint text present.
- [ ] Same empty DB → player dropdown HTML contains only *All players* (no stale player options).
- [ ] Same empty DB → `GET /events/melee/bitchslaps` 200 (not 404); empty tbody or hint; link to `/`.
- [ ] Same empty DB → `GET /style.css` 200, body contains table-related CSS.
- [ ] DB with `schema_version` > binary CURRENT → `GET /` 503 (or app refuses to serve — assert documented behavior).

### HTTP handler tests (`axum` test service)

- [ ] `GET /` with fixture DB → 200; body contains melee/skill/spell table headings and a fixture verb string.
- [ ] `GET /?player=X&range=7d` → 200; filter controls or links reflect params in HTML.
- [ ] `GET /events/melee/{verb}` → 200; body contains fixture `message_text`.
- [ ] `GET /events/skill/bash` → 200 with bash fixture rows.
- [ ] `GET /events/spell/magic%20missile` → 200 with spell fixture rows.
- [ ] Ambiguous `batch_id` siblings both appear on drill-down for that verb (focal verb rows plus inline batch siblings with linked Verb column).
- [ ] `GET /style.css` → 200, non-empty CSS (contains table or zebra-related rule).
- [ ] Sort links on landing preserve `player` and `range` query params.
- [ ] Confirmed and estimated column headers present in landing HTML.

### Server lifecycle (light tests)

- [ ] Port parsing: default 6464, `--port` override (unit test on argv helper if HTTP thread not started in test).
- [ ] Bind address restricted to `127.0.0.1` (config constant test or integration note).

## Testing seam

**Layer 1:** aggregate query functions + extrapolation pure functions on fixture DB. **Layer 2:** HTTP routes via `axum` test client with same fixture DB. Both layers required — not optional.

## Implementation notes

- Prefer testing extrapolation and rollup in pure functions fed by SQL query results.
- Inline JS only for sort and filter controls — no frontend build step.
- Do not add charts, JSON-only mode, or CLI `damage-serve` subcommand in this slice.
