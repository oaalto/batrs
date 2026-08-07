# Combat damage tracking

## Status

accepted

**Triage:** `ready-for-agent`

## Problem Statement

BatMUD combat output is noisy: many lines appear between short-score (`H:`) status updates, and HP loss on an `H:` line does not always map cleanly to a single hit message. The player wants to learn how much damage each incoming melee verb, skill, and spell actually deals over time — not guess from crowded combat scrollback.

Prior offline analysis (`analyze_combat_damage.py`) proved the concept on old log formats, but batrs does not yet capture incoming damage live. The player has switched to a **battle listen** format where melee verbs are catalogued (`docs/hit_messages.md`), skills and spells have distinct line shapes, and misses produce no HP change. There is no persistent store, no attribution confidence model, and no local dashboard to review per-verb damage ranges after play.

## Solution

While batrs is running and the player is logged in, a **Combat Damage** module observes every incoming plain line, buffers recognized incoming-damage candidates between `H:` triggers, and when an `H:` line shows a negative HP bracket, writes one or more attributed rows to a global SQLite database at `~/.batrs/combat_damage.db`. Player name is metadata on each row, not a partition key.

On batrs startup, a read-only HTTP server (default port **6464**, bind `127.0.0.1`) starts automatically in the background and serves a styled HTML dashboard: three sortable tables (melee, skill, spell) with side-by-side **confirmed** and **estimated** damage columns, filters by time range and player, and drill-down to individual events grouped by `batch_id`. Collection is always on in v1; write failures log and do not interrupt play.

## User Stories

1. As a player, I want batrs to record incoming HP loss while I play, so that I do not need to manually parse combat logs afterward.
2. As a player, I want damage stored in one global database on my machine, so that all characters contribute to the same dataset with player as metadata.
3. As a player, I want attributed HP loss in `damage_events` and unexplained HP drops captured separately for review, so that matcher gaps do not pollute attribution aggregates but I can still inspect what happened.
4. As a player, I want events triggered only by negative HP brackets on `H:` short-score lines, so that `Hp:` prompt snapshots and healing do not create rows.
5. As a player, I want each row to record HP before, HP after, and HP delta magnitude, so that I can see exact loss per observation.
6. As a player, I want SP, EP, exp, and gold changes on the same `H:` line ignored for damage rows, so that my own skill costs are not mixed with incoming damage.
7. As a player, I want battle-listen melee hits (`<monster> <verb> you.`) recognized via the hit-message catalog, so that catalog verbs like `bitchslaps` and `lightly strikes` aggregate correctly.
8. As a player, I want incoming skills (bash, push, kick, stab, scythe swipe, riposte) recognized by their output lines, so that non-melee damage is separated from catalog melee.
9. As a player, I want incoming spells recognized by `A/An <name> hits you.`, so that each spell name becomes its own aggregation bucket.
10. As a player, I want misses, dodges, parries, tumble lines, and outgoing `You <verb> <target>` lines excluded from the candidate buffer, so that only plausible incoming damage lines are considered.
11. As a player, I want kick partial-deflect lines counted as kick skill output, so that reduced kick damage is still attributed to kick.
12. As a player, I want isolated hits between two `H:` lines to receive full confidence (`weight = 1.0`), so that clean observations tighten per-verb damage ranges fastest.
13. As a player, I want ambiguous buffers with multiple candidates to produce one row per candidate with fractional weight, so that uncertain splits are preserved rather than guessed at insert time.
14. As a player, I want `damage_min` and `damage_max` stored per row, so that ambiguous events keep loose bounds until enough isolated data exists.
15. As a player, I want sibling rows from the same `H:` trigger linked by `batch_id`, so that I can see which lines competed for the same HP loss.
16. As a player, I want the dashboard to show **confirmed** aggregates from isolated observations only, so that I can trust the primary numbers.
17. As a player, I want an **estimated** view that applies conservative constraint extrapolation to ambiguous batches, so that I can explore upper/lower bounds without false precision.
17b. As a player, I want ambiguous melee batches with loose bounds to show a **rank-estimated avg** skewed toward higher-catalog-rank verbs within the batch, so that estimated numbers are more informative before isolated data exists — without changing confirmed aggregates or attribution `weight`.
18. As a player, I want melee, skill, and spell damage in separate summary tables, so that I can compare verbs within each category without cross-noise.
19. As a player, I want confirmed and estimated min/max/avg (or bounds) side by side per verb, so that I can see how much ambiguous data shifts the range.
20. As a player, I want to click a verb row and see individual events, so that I can verify outliers and read the attributed `message_text`.
21. As a player, I want drill-down events grouped or linked by `batch_id`, so that I can inspect multi-candidate `H:` triggers.
22. As a player, I want to filter the dashboard by time range (`24h`, `7d`, `all time`), so that recent sessions are easy to review.
23. As a player, I want to filter by player name, so that multi-character play on one machine stays separable.
24. As a player, I want all tables sortable by column, so that I can rank verbs by observation count or damage without exporting data.
25. As a player, I want the dashboard styled with CSS (zebra rows, clear headers, distinct confirmed/estimated groups), so that local review is pleasant without a separate frontend build.
26. As a player, I want the HTTP server to start automatically when batrs starts, so that I can open the dashboard without remembering a subcommand.
27. As a player, I want to override the HTTP port via `--port` (default 6464), so that I can avoid local port conflicts.
28. As a player, I want the HTTP server bound to localhost only, so that damage data is not exposed on the network.
29. As a player, I want damage collection to survive reconnect within a login but clear its buffer on `/connect` or logout, so that mis-attribution does not carry across sessions.
30. As a player, I want combat play to continue if the database write fails, so that analytics never block my session.
31. As a maintainer, I want a single top-level Combat Damage module on the application shell, so that buffer, matchers, and persistence have one locality.
32. As a maintainer, I want the collector called before combat-awareness gagging, so that gagged scan and prompt lines still participate in buffering and `H:` detection.
33. As a maintainer, I want `H:` parsing shared with short-score via the same regex, so that HP deltas stay consistent without coupling to stats effects.
34. As a maintainer, I want buffer reset wired through existing fresh-session reset and login-transition paths, so that session lifecycle stays consistent with Combat Awareness and stats.
35. As a maintainer, I want the melee catalog compiled from `hit_messages.md` at build time, so that runtime matching stays fast and catalog edits are validated at compile time.
36. As a maintainer, I want matcher order skills → spells → melee, so that distinctive skill and spell lines are not misclassified as melee.
37. As a maintainer, I want longest verb match and weapon-family recency for melee, so that multi-word verbs and same-monster weapon type are handled efficiently.
38. As a maintainer, I want inline numbered schema migrations starting at version 1, so that future column additions do not require external migration tooling.
39. As a maintainer, I want batrs to fail clearly if the database schema is newer than the binary, so that downgrade corruption is avoided.
40. As a maintainer, I want new skills and spells addable by extending the matcher catalog without changing the row shape, so that the system grows with documented line patterns.
41. As a test author, I want matcher behavior covered by fixture-driven unit tests (catalog, conjugation, example fight, kick, spells), so that parser regressions are caught without a live BatMUD session.
42. As a test author, I want collector behavior testable by feeding lines and asserting database rows, so that buffer, trigger, weight, and batch semantics are verified at the highest practical seam.
43. As a player opening the dashboard before my first fight, I want a readable empty page with table headers and filters, so that I know the server is running and what will appear after combat.
44. As a player, I want the database created automatically on first batrs launch, so that I do not need to run a setup command or play a fight before the dashboard works.
45. As a player, when HP drops on an `H:` line but no recognized hit line exists in the window, I want the lines before that `H:` saved for later review, so that I can discover new damage patterns (environmental, DoT, guild specials) without losing scrollback context.
46. As a player, I want unattributed HP loss shown in a separate HTTP dashboard section (not mixed into melee/skill/spell tables), so that attribution rollups stay trustworthy.
47. As a player, I want unattributed review to include misses, outgoing hits, and gagged lines in the saved context, so that clues for unexplained loss are not hidden by combat-awareness filtering.
48. As a player, I want unattributed triggers marked reviewed when I open their drill-down page, so that I can tell which unexplained HP losses I have already inspected.

## Implementation Decisions

### Module boundary

- Top-level **Combat Damage** module on the application shell with a **DamageCollector** field.
- Collector owns: in-memory candidate buffer between `H:` lines, line matchers (skills, spells, melee catalog), SQLite connection, monotonic `batch_id` allocation, and insert transactions.
- Collector does **not** own: combat round state, short-score stats mutation, HTTP rendering templates, or trigger pipeline effects.

### Pipeline hook

- Call `handle_line(plain_line, player_name)` on every logged-in incoming line in input processing, **before** combat-awareness gag/continue.
- `player_name` from session login name (same as other session-scoped subsystems).
- Re-parse `H:` lines internally using the shared short-score regex; do not hook `StatsEffect::UpdateShortScore`.

### Event trigger and row shape

A row is written only when **all** hold:

1. An `H:` line arrives with negative HP bracket `[-N]`.
2. The buffer since the previous `H:` contains at least one recognized incoming-damage candidate (melee catalog, skill regex, or spell regex).
3. Each recognized candidate in that window becomes one row sharing the same `batch_id`.

Per row fields:

| Field | Semantics |
| --- | --- |
| `recorded_at` | Wall-clock UTC when the `H:` line was seen (ISO-8601 text) |
| `player` | Login name metadata |
| `hp_delta` | Positive magnitude of HP loss (same value on each sibling row in a batch) |
| `hp_before`, `hp_after` | From the `H:` line HP fields |
| `damage_category` | `melee`, `skill`, or `spell` |
| `source_name` | Attacker from line capture; empty string for spell hit lines |
| `message_verb` | Aggregation key: catalog verb, skill id, or spell name |
| `message_text` | Full attributed line only — no context buffer blob |
| `candidate_count` | Count of filtered candidates in the batch |
| `weight` | `1.0` if isolated; `1.0 / N` if ambiguous |
| `damage_min`, `damage_max` | `hp_delta`/`hp_delta` if isolated; `0`/`hp_delta` if ambiguous |
| `batch_id` | Integer shared by all rows from one `H:` trigger |
| `catalog_rank` | Melee only: `1`–`26` from `hit_messages.md` line order; `NULL` for skill/spell (schema v2) |
| `weapon_family` | Melee only: catalog family id (`slash`, `bash`, …); `NULL` for skill/spell (schema v2) |

Skip `damage_events` row when HP loss is unattributed (zero candidates). Persist a parallel **unattributed review** record instead (see [Unattributed HP loss](#unattributed-hp-loss)). No `unknown` category in `damage_events`. No round number, fight id, or session id.

### Attribution and aggregation

- **Crowding:** only filtered damage-candidate lines count toward `candidate_count`; misses, outgoing hits, scan output, round headers, `Hp:` prompts, concentration lines, and gags are ignored for weight.
- **Aggregation key:** `damage_category` + `message_verb` for skill and spell. **Melee** adds `weapon_family` — verbs that collide across families (e.g. `savagely strike` in `bash` and `claw`) roll up separately. `known_min`/`known_max` for estimated extrapolation use the same melee key.
- **Confirmed view:** aggregates from `candidate_count = 1` rows only (exact `hp_delta` per observation).
- **Estimated view:** applies conservative constraint extrapolation at read time to ambiguous batches using isolated-derived `known_min`/`known_max` per key; no even-split fallback; loose `[0, hp_delta]` when constraints do not resolve. When bounds stay loose and batch rows carry `catalog_rank`, **estimated avg** uses rank-proportional split (`hp_delta × rankᵢ / Σ rankⱼ` over ranked melee candidates; unranked candidates share remainder equally; equal avg fallback if no ranks). Rank never overrides isolated constraints or stored per-row bounds. Extrapolation is not written back to stored rows.

### Unattributed HP loss

**Trigger:** negative HP bracket on an `H:` short-score line with **zero** recognized damage candidates in the attribution window since the previous `H:` line. `Hp:` prompt lines do not trigger. Healing or empty HP brackets do not trigger.

**Not in scope for this path:** ambiguous batches (N≥2 candidates) — those continue on the `damage_events` path only.

**Context window:** every plain line passed to `DamageCollector::handle_line` while logged in between the previous `H:` and the triggering `H:` (exclusive of both `H:` lines). Includes misses, outgoing hits, round headers, and gagged scan lines. Empty context is valid.

**Storage (schema v4):**

| Table / field | Semantics |
| --- | --- |
| `unattributed_hp_events` | One row per trigger: `recorded_at`, `player`, `hp_delta`, `hp_before`, `hp_after`, `h_line_text` |
| `context_lines` | Ordered JSON array of strings on the same row |

One transaction per trigger. Indexes on `recorded_at` and `player`. No duplication of attributed-row context when candidates exist.

**Lifecycle:** context window clears on every `H:` line, `reset_buffer()`, logout, and `FreshSessionReset::DamageCollector`. Write failure: `tracing::warn!`, discard pending context, continue play.

**HTTP viewer:** new **Unattributed HP loss** section on landing (or dedicated route) — table of triggers with `recorded_at`, `player`, `hp_delta`, line count; drill-down lists `context_lines` in order plus `h_line_text`. Same `range` and `player` filters as attribution tables. Always on when collector is active.

### Matcher catalog (v1)

**Order:** skills → spells → melee.

**Skills** (`damage_category = skill`), each own `message_verb`:

| `message_verb` | Patterns (summary) |
| --- | --- |
| `bash` | `<name>'s bash sends you sprawling.` |
| `push` | `<name> pushes you.` |
| `kick` | groin kick, stomach kick, partial-deflect kick lines |
| `stab` | weapon aside + stab; kneecap smash; PUMMELS midriff |
| `scythe swipe` | `slashes a ragged wound across your chest` |
| `riposte` | two-line: `<name> parries.` then `...AND counterattacks.` or `...AND ripostes.` |

**Spells** (`damage_category = spell`): single regex `^An? (.+) hits you\.$`; `message_verb` = captured spell name; `source_name` empty; cast lines ignored.

**Melee** (`damage_category = melee`): battle-listen template `<name> <verb> you.` against catalog compiled from `hit_messages.md` (11 weapon families, 286 verbs). Longest matching verb wins. Dual suffix: conjugated (`+s`/`+es` on last word) then bare catalog phrase. Case-insensitive match; canonical `message_verb` from catalog. Weapon-family recency: after a melee match, try that family's verbs first on subsequent lines.

**Outgoing** `You <verb> <target>` lines are never candidates.

**Out of v1 matchers:** environmental, DoT/bleed, player-name targeting (`hits Fueryon`), unrecognized guild specials. Breath attacks only via melee catalog when battle listen emits catalog breath verbs.

### Storage

- Path: `~/.batrs/combat_damage.db` via existing config directory helper.
- Library: `rusqlite` (sync).
- Tables: `damage_events` (flat log) + `unattributed_hp_events` (review capture, schema v4) + `schema_version` (single integer row, starts at `1`).
- Writes: one transaction per `H:` trigger (`BEGIN` → N inserts with shared `batch_id` → `COMMIT`).
- Indexes: `batch_id`, `recorded_at`, `(damage_category, message_verb)`, `candidate_count`.
- Migrations: inline numbered Rust steps on open; fail with clear error if DB schema newer than binary. No downgrade.
- Retention: unlimited in v1.

### Empty state and first launch

**Startup ordering (batrs process start):**

1. Ensure `~/.batrs/` exists (reuse existing config `init_base` / `create_dir_all` pattern).
2. Open or create `combat_damage.db` with v1 schema (`open_db`) — **before** the HTTP background thread starts.
3. Collector holds the read-write connection; HTTP viewer opens a **read-only** connection to the same path (WAL mode acceptable for concurrent read/write).
4. HTTP server starts in a background thread; batrs TUI continues regardless of HTTP bind success (log warning if port unavailable).

The database file **must exist** (empty schema, zero rows) after step 2 even if the player has never logged in or taken damage. HTTP must not depend on a first insert to create the file.

**Landing page (`/`) when `damage_events` has zero rows:**

- HTTP **200** (database opens successfully).
- Page structure always present:
  - Document title / header: **Combat damage** (or equivalent).
  - Filter form: time range (`24h`, `7d`, `all time`) and player dropdown.
  - Three sections: **Melee**, **Skill**, **Spell**.
  - Each section: full table **header row** with confirmed and estimated column groups (verb, obs count, min, max, avg/bounds as designed).
  - Each section: **zero data rows** in `<tbody>`.
  - Each section: one short empty hint below the table, e.g. *No melee damage recorded yet.* (skill/spell variants).
- No total-damage summary line; no by-monster table (unchanged).
- Bundled CSS applied (page is styled, not raw browser defaults).
- Sort links on column headers still render (clicking with no data is a no-op or reload with sort param — no error).

**Player filter when empty:**

- Dropdown contains **All players** only (no other options until at least one distinct `player` value exists in `damage_events`).

**Drill-down (`/events/{category}/{verb}`) when no matching rows:**

- HTTP **200** (not 404) — verb may never have been observed yet.
- Same filter form as landing.
- Event table with headers, zero rows, empty hint e.g. *No events for this verb.*
- Link back to `/` preserving current `range` and `player` query params.

**HTTP error responses (not empty state):**

| Condition | Response |
| --- | --- |
| DB schema newer than binary | **503** — plain text or minimal HTML: *Database schema newer than batrs; upgrade batrs.* |
| DB file unreadable / corrupt open | **503** — plain text or minimal HTML: *Cannot open combat damage database.* Batrs process continues; collector logs warning. |
| Port already in use | Log warning at startup; TUI continues; no crash. |

Empty state is **not** an error — distinguish 200 empty dashboard from 503 DB failure.

### Session lifecycle

- Open DB once at application init; close on drop.
- `reset_buffer()` on `FreshSessionReset::DamageCollector` and on login-state transition to logged-out; do not close DB on reconnect.
- Write failure: `tracing::warn!`, drop batch, continue play.

### HTTP viewer

- Framework: `axum`, server-rendered HTML, bundled `style.css`, small inline JS for filters and column sorting (no JS framework, no charts).
- Auto-start on batrs launch in background thread; `--port` flag default **6464**; bind `127.0.0.1`; read-only DB access; stops when batrs exits.
- Routes: `/` landing (three attribution tables + unattributed section + filters), `/events/{category}/{verb}` drill-down, `/unattributed` (or equivalent) drill-down for context review, static CSS route.
- Landing: three tables (melee, skill, spell); each row one `message_verb` with side-by-side confirmed and estimated columns (obs count, min, max, avg/bounds). No total-damage line, no by-monster table.
- Filters: time (`24h`, `7d`, `all`) and player dropdown; default all; query params preserve filter and sort state.
- Sorting: all columns clickable asc/desc; landing default verb ascending; drill-down default `recorded_at` descending.
- Drill-down columns: `recorded_at`, `player`, `hp_delta` (or min–max for ambiguous), `source_name`, `weight`, `candidate_count`, `message_text`; `batch_id` grouping or sibling links.
- CSS: zebra rows, hover, grouped confirmed/estimated headers, distinct styling for wide estimated bounds.

### Build-time catalog

- `build.rs` parses `hit_messages.md` → generated catalog module in `OUT_DIR` with `CatalogEntry` (canonical verb, weapon family id, `catalog_rank` 1–26, conjugated suffix, bare suffix).
- Compile fails on malformed catalog. Human-edited source remains `hit_messages.md`.

### Current implementation state

- **Done:** matcher module with build-time catalog, conjugation, skills/spells/melee order, 35+ unit tests; DamageCollector buffer + SQLite persistence; HTTP viewer and auto-start wiring.
- **Not done:** unattributed HP loss dual-buffer, schema v4 tables, unattributed HTTP viewer section (slice 04).

## Testing Decisions

### Philosophy

**More tests, better.** Cover every layer with observable-behavior tests — no live BatMUD, no network, no full CSS layout assertions. Prefer table-driven tests with inline line vectors in test modules over external fixture files. Test boundaries (matcher → collector → storage → aggregates → HTTP) independently so a failure localizes quickly; also run cross-layer integration tests so wiring regressions surface.

Good tests assert **inputs → outputs** at module seams: lines → `DamageMatch`; line sequences → `damage_events` rows; fixture DB → aggregate columns; HTTP routes → status + key substrings in HTML. Do not assert private buffer fields, internal call order, or pixel layout.

### Test layers (all required)

| Layer | Seam | Minimum coverage |
| --- | --- | --- |
| **Matcher** | `match_incoming(line)` | Done: full catalog (286 verbs), per-family samples, conjugation, fixtures, edge cases. Maintain when catalog changes. |
| **Conjugation** | `conjugate_last_word` | `+s`, `+es`, multi-word, ALL-CAPS — isolated unit tests (already present). |
| **Catalog build** | `build.rs` output | Compile-time validation; optional test that generated catalog count matches `hit_messages.md` verb count. |
| **Schema / migrations** | `open_db(path)` | Fresh DB creates v1 schema + indexes; `schema_version = 1`; reopen is no-op; newer-schema DB returns clear error string. |
| **Collector** | `handle_line` sequence → query rows | Every acceptance path in slice 02 (see below). |
| **Collector integration** | Inline Holy-man fight line sequence | Row count, categories, verbs, `batch_id` siblings for ambiguous batches, no rows on misses/outgoing-only buffers. |
| **Aggregates** | Query functions on fixture DB | Confirmed rollups, estimated extrapolation, filters, sort keys — table-driven per category. |
| **Extrapolation** | Pure function on batch tuples | Conservative constraint pass: known-min exceeds delta, single known-min equals delta, loose bounds when unresolved. |
| **HTTP handlers** | `axum` test service + fixture DB | `/` and `/events/...` return 200; HTML contains expected verb labels, filter params, sort links. |
| **HTTP assets** | `GET /style.css` | 200 and non-empty body. |
| **App wiring** | `FreshSessionReset` / logout | Buffer cleared without closing DB (light integration or dedicated test on reset plan). |

### Collector test matrix (slice 02)

Each scenario is a separate test or table row; query `damage_events` after the sequence:

1. **Isolated melee** — one catalog hit between two `H:` lines → 1 row, `weight = 1.0`, min = max = delta.
2. **Isolated skill** — bash / push / kick / stab / scythe swipe each with following `H:` loss.
3. **Isolated spell** — `A magic missile hits you.` + `H:` loss.
4. **Ambiguous (N=2)** — two candidates, one `H:` → 2 rows, same `batch_id`, `weight = 0.5`, min = 0, max = delta.
5. **Ambiguous (N=3)** — three candidates → `weight = 1/3` each.
6. **No candidates** — `H:` loss with only misses/outgoing in buffer → 0 `damage_events` rows; 1 `unattributed_hp_events` row with context; buffer cleared.
7. **Unattributed** — `H:` loss with empty buffer → 0 `damage_events` rows; 1 `unattributed_hp_events` row with empty context.
8. **Healing** — positive HP bracket on `H:` → 0 rows, buffer cleared.
9. **SP-only loss** — `H:` with `[]` HP bracket and negative SP → 0 rows.
10. **Mixed stat line** — negative HP + negative SP on same `H:` → row uses HP delta only.
11. **Gagged-line candidate** — damage candidate line that would be gagged by combat awareness still buffered and attributed.
12. **Between-round bash** — skill line outside round block + `H:` (no round header required).
13. **`reset_buffer`** — partial buffer then reset → next `H:` with no candidates → 0 rows.
14. **`Hp:` lines** — prompt snapshots do not trigger or clear incorrectly.
15. **Write failure** — mock or inject DB error → warn logged, buffer cleared, no panic (if testable without excessive mocking, otherwise manual checklist).
16. **Multi-player metadata** — two sequences with different `player` strings → rows retain correct `player` column.
17. **Holy-man fight replay** — inline line sequence end-to-end row inventory vs expected hits (melee + bash + push).

### Aggregate and viewer test matrix (slice 03)

Fixture SQLite with hand-built rows covering isolated and ambiguous batches:

1. **Confirmed min/max/avg/count** per `damage_category` + `message_verb` from `candidate_count = 1` only.
2. **Estimated bounds** — ambiguous batch contributes `[0, delta]` per candidate; extrapolation tightens when isolated known-min exists.
2b. **Rank-estimated avg** — ambiguous melee batch with ranks 2 vs 20 and loose bounds → estimated avg skews toward high-rank verb; confirmed avg unchanged (isolated only).
3. **Constraint: sum of known-min > delta** — flag or cap per PRD rules.
4. **Constraint: one candidate known-min = delta** — others assigned 0 for estimated view.
5. **Filter `range=24h`** — excludes old `recorded_at`.
6. **Filter `range=7d`** — boundary at seven days.
7. **Filter `player=`** — only matching rows in rollups and drill-down.
8. **Filter defaults** — `all` + all players when params absent.
9. **Landing sort** — verb asc/desc via query param changes row order in HTML or query result.
10. **Drill-down sort** — `recorded_at` desc default; toggle asc.
11. **Drill-down batch grouping** — sibling rows share `batch_id` visible or linked.
12. **Three tables** — melee / skill / spell verbs appear only in their table.
13. **HTTP `/`** — 200, contains verb from fixture, confirmed and estimated column headers.
14. **HTTP `/events/melee/bitchslaps`** — 200, contains `message_text` from fixture.
15. **HTTP `/style.css`** — 200.
16. **Sort link preservation** — filter query params preserved in sort URL query string.

### Empty state and first launch (slice 02 + 03)

1. **Fresh DB at init** — `open_db` on temp path creates file + v1 schema with zero rows; second open succeeds.
2. **`~/.batrs/` creation** — open on missing parent dir creates directory (integration or unit with temp home).
3. **HTTP `/` on empty DB** — 200; body contains `Melee`, `Skill`, `Spell` section headings; confirmed and estimated column headers; **no** `<tbody>` data rows (or empty tbody); empty-hint text present.
4. **HTTP `/` player dropdown** — only *All players* when DB empty.
5. **HTTP `/events/melee/nonexistent`** — 200; empty event table; not 404.
6. **HTTP `/style.css` on empty DB** — 200; non-empty CSS.
7. **Schema newer than binary** — HTTP or `open_db` returns 503 / error string per PRD (no panic).

### Unattributed review test matrix (slice 04)

1. **Miss-only window** — miss/outgoing lines then `H:[-N]` → 0 `damage_events`; 1 unattributed row; context contains miss lines.
2. **Silent loss** — empty window then `H:[-N]` → unattributed row with empty `context_lines`.
3. **Attributed path unchanged** — recognized hit in window → `damage_events` only; no unattributed row.
4. **Ambiguous batch unchanged** — N≥2 candidates → N `damage_events` rows; no unattributed row.
5. **Context includes gagged line** — candidate-visible line that CA would gag still appears in unattributed context when zero candidates match.
6. **Reset lifecycle** — `reset_buffer` / logout clears context window without closing DB.
7. **HTTP unattributed section** — fixture DB → 200; trigger table and drill-down show ordered context + `h_line_text`.

### Prior art

- Matcher: `combat_damage/matcher.rs` (35+ tests).
- Line-sequence integration: Combat Awareness `handle_incoming_line` tests.
- Short-score parsing: `short_score` trigger tests / `SC_REGEX` sharing.
- Table-driven fixtures: trigger and guild test modules.
- HTTP: if no existing `axum` tests in repo, introduce `axum` test utilities pattern for this feature.

### Coverage target

No formal % gate, but **every acceptance criterion in slices 02 and 03 must have at least one automated test** unless explicitly marked manual (write-failure mock). New matchers or schema columns add tests in the same PR.

## Out of Scope

- Backfill import of existing `~/.batrs/*/logs/*.log` into the database (live capture only).
- Per-player database files (one global `combat_damage.db`).
- Outgoing damage analytics as events (`You puncture …` is matcher sanity only).
- Old battle-listen log format (new listen format only).
- Round number, fight id, or session id on rows.
- `unknown` category rows in `damage_events` for unattributed HP loss (review capture is a separate table).
- Context buffer on attributed `damage_events` rows (`message_text` only remains).
- Environmental, DoT/bleed, and player-name-targeting patterns until documented.
- Opt-out toggle, write batching tuning, max buffer size guardrails (fog — future ticket).
- Charts, total-damage summary line, damage-by-monster landing table.
- CLI subcommand or slash command to start viewer (auto-start only).
- Network exposure beyond `127.0.0.1`.
- Spell caster name on hit lines (empty `source_name` in v1).
- Automatic prune / retention policy.
- JSON-only API without HTML dashboard.

## Further Notes

- Wayfinder map and closed decision tickets live under `docs/features/combat-damage-tracking/`; this PRD supersedes them for implementation handoff but tickets remain the audit trail for grilling decisions.
- Reference catalog: `docs/hit_messages.md`. Example lines live in matcher/collector test tables (Holy-man fight, kick variants, spell hit lines).
- Prior offline analysis (`~/.batrs/COMBAT_DAMAGE_ANALYSIS.md`, `analyze_combat_damage.py`) informed design but targets an older log format — not a v1 dependency.
- Implementation slices: `01-damage-line-matcher.md`, `02-damage-collector-and-storage.md`, `03-http-damage-viewer.md`, `04-unattributed-hp-review.md`.
