---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

What is the **minimum useful HTTP dashboard** for v1?

Decide MVP pages/sections:

- **Confirmed** view (isolated/`candidate_count = 1` only) vs **estimated** view (ambiguous rows + extrapolation) — see [weight ticket](how-should-attribution-weight-work.md)
- Aggregates: per-message min/max/avg damage by `damage_category` + `message_verb`; total damage taken; damage by monster (`source_name`)?
- Time range filter (session, last N fights, all time)?
- Drill-down to individual events with context lines and weight?
- Charts vs tables (ponytail bias: tables first?)
- Static bundled HTML/JS vs server-rendered templates vs JSON API only?
- Auth/bind: localhost-only, configurable port, read-only?

What would make you actually use this after one evening of play?

### Q1 — primary landing view (resolved)

**Decision:** landing page is **three separate summary tables** — one each for **melee**, **skill**, and **spell**. Each table has **side-by-side confirmed and estimated columns** in a single row per `message_verb`:

| Verb | Confirmed obs | Conf min | Conf max | Conf avg | Est min | Est max | Est obs |
|------|---------------|----------|----------|----------|---------|---------|---------|
| bitchslaps | 12 | 18 | 24 | 21.3 | 15 | 26 | 18 |

- **Confirmed columns:** from `candidate_count = 1` rows only (exact `hp_delta` per observation).
- **Estimated columns:** bounds after conservative constraint extrapolation on ambiguous batches; loose bounds visually distinct (e.g. wide range or italic).
- No charts on landing — tables only.

### Q2 — drill-down (resolved)

**Decision:** click a verb row → **event list page** for that `damage_category` + `message_verb`.

| `recorded_at` | `player` | `hp_delta` | `source_name` | `weight` | `candidate_count` | `message_text` |
|---------------|----------|------------|---------------|----------|-------------------|----------------|
| 2026-08-06 14:32 | Odefu | 22 | Holy man | 1.0 | 1 | Holy man bitchslaps you. |

- Sorted newest-first.
- Ambiguous rows show `damage_min`–`damage_max` in estimated drill-down.
- Rows sharing a `batch_id` grouped or linked (sibling candidates for same `H:` trigger).
- No context buffer — attributed line only (`message_text`).

### Q3 — filters (resolved)

**Decision:** **time range + player** filters on landing tables and drill-down event lists.

| Filter | Options | Default |
|--------|---------|---------|
| **Time range** | `last 24h`, `last 7d`, `all time` | `all time` |
| **Player** | dropdown of distinct `player` values in DB | `all players` |

- No fight/session id filter (not tracked in v1).
- No monster (`source_name`) filter on landing in v1.

### Q4 — delivery (resolved)

**Decision:** **server-rendered HTML** via `axum` — ponytail-friendly, no separate frontend build.

| Route | Purpose |
|-------|---------|
| `/` | Landing — three tables + filters |
| `/events/{category}/{verb}` | Drill-down event list |

- Bind `127.0.0.1` only, read-only.
- HTML tables with bundled CSS; small inline JS for filter dropdowns and column sorting — no charts, no JS framework.
- Filters via query params (`?player=Odefu&range=7d`).

### Q5 — server lifecycle (resolved)

**Decision:** HTTP server **starts automatically** when batrs starts — no CLI subcommand, no slash command in v1.

- **`--port` flag** on batrs startup; default **`6464`**.
- Server runs in a **background thread/task** alongside the TUI; read-only DB access for viewer queries.
- Bind `127.0.0.1` only; serves until batrs exits.

### Q6 — extra aggregates on landing (resolved)

**Decision:** **verb tables only** — no total-damage summary line, no damage-by-monster table on landing. Landing is the three category tables (melee / skill / spell) plus filters. `source_name` visible on drill-down only.

### Q7 — table sorting (resolved)

**Decision:** all summary and drill-down tables are **sortable by column** (click column header to toggle asc/desc).

- Landing tables: sort by verb, confirmed obs/min/max/avg, estimated min/max/obs.
- Drill-down event list: sort by `recorded_at`, `player`, `hp_delta`, `source_name`, `weight`, `candidate_count`.
- Default sort: landing — verb ascending; drill-down — `recorded_at` descending (newest first).
- Implementation: small inline JS on column headers (no JS framework); sort state reflected in query params so filtered views keep their order on reload.

### Q8 — styling (resolved)

**Decision:** server-rendered HTML includes a **bundled CSS stylesheet** — not unstyled browser defaults.

- Single static `style.css` served by axum (or embedded in a shared layout template).
- Readable tables: zebra rows, hover highlight, clear header row, monospace for numeric columns.
- Confirmed vs estimated column groups visually separated (e.g. subtle background bands or header grouping).
- Loose estimated bounds styled distinctly (e.g. italic or muted when min–max span is wide).
- Responsive enough for a local dashboard (no mobile-first requirement); dark-friendly palette optional but not required in v1.

## Resolution

**Landing (`/`):** three summary tables (melee, skill, spell). Each row is one `message_verb` with side-by-side **confirmed** and **estimated** columns (obs count, min, max, avg / bounds). Filters: time range (`24h`, `7d`, `all`) + player dropdown; default all. No charts, no total-damage line, no by-monster aggregate.

**Drill-down (`/events/{category}/{verb}`):** event list newest-first; columns `recorded_at`, `player`, `hp_delta` (or min–max for ambiguous), `source_name`, `weight`, `candidate_count`, `message_text`. Rows with same `batch_id` grouped or linked.

**Delivery:** `axum` server-rendered HTML with bundled **`style.css`**; filters via query params; small inline JS for filter dropdowns and **column-header sorting** (asc/desc, state in query params).

**Sorting:** all tables sortable by column; landing default verb asc; drill-down default `recorded_at` desc.

**Lifecycle:** HTTP server starts automatically when batrs starts; `--port` flag default **6464**; background thread; bind `127.0.0.1`; read-only DB; stops on batrs exit.

**Routes:** `/` (landing), `/events/{category}/{verb}` (drill-down), `/static/style.css` (or equivalent).
