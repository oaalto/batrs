# 02 — Damage collector and storage

## Parent

`prd.md`

## What to build

Implement **DamageCollector** on top of the existing matcher: buffer recognized incoming-damage candidates between `H:` lines; on negative HP bracket, allocate `batch_id`, compute `candidate_count`, `weight`, `damage_min`, `damage_max`, and persist rows in `~/.batrs/combat_damage.db` inside one transaction per trigger. Wire the collector on the application shell: `handle_line` before combat-awareness gag when logged in; `FreshSessionReset::DamageCollector` and login-transition `reset_buffer()`; DB open at init, close on drop. Share short-score regex for `H:` parsing. Log and continue on write failure.

**First launch:** at `BatApp` init (before HTTP thread), ensure `~/.batrs/` exists and `open_db` creates `combat_damage.db` with v1 schema if missing. Zero rows is valid. HTTP viewer depends on this file existing — do not defer creation to first insert.

## Blocked by

`01-damage-line-matcher.md` (matcher API stable).

## Status

pending

## Acceptance criteria

- [ ] SQLite schema v1: `damage_events` + `schema_version` at `~/.batrs/combat_damage.db` with columns per PRD (including `batch_id`, `candidate_count`, `weight`, `damage_min`, `damage_max`).
- [ ] Four indexes created on first migrate: `batch_id`, `recorded_at`, `(damage_category, message_verb)`, `candidate_count`.
- [ ] Inline migration from version 1; clear error if DB schema newer than binary.
- [ ] `handle_line`: non-`H:` lines run through matcher; recognized candidates appended to buffer; unrelated lines ignored.
- [ ] `handle_line`: negative `H:` with zero candidates → no rows, buffer cleared.
- [ ] `handle_line`: negative `H:` with one candidate → one row, `weight = 1.0`, `damage_min = damage_max = hp_delta`, `candidate_count = 1`.
- [ ] `handle_line`: negative `H:` with N candidates → N rows, shared `batch_id`, `weight = 1.0/N`, `damage_min = 0`, `damage_max = hp_delta`, `candidate_count = N`.
- [ ] `handle_line`: positive or empty HP bracket on `H:` → no rows; buffer cleared.
- [ ] SP/EP/exp/gold brackets on same `H:` line ignored for `hp_delta`.
- [ ] `reset_buffer()` clears candidates only; DB connection persists.
- [ ] `FreshSessionReset::DamageCollector` and logout transition call `reset_buffer()`.
- [ ] Failed batch write: `tracing::warn!`, buffer still cleared, session continues.
- [ ] First launch: `~/.batrs/` created if missing; `open_db` at init creates empty v1 schema before any `handle_line` or HTTP start.
- [ ] `cargo test --all-targets --all-features` passes.

## Tests (required)

All tests use a temp SQLite path; no `~/.batrs` in tests.

### Schema and migrations

- [ ] Fresh open creates `damage_events`, `schema_version`, all columns, four indexes.
- [ ] Second open on same file: no error, schema unchanged.
- [ ] Open file with `schema_version` > binary `CURRENT`: returns clear error (no panic).
- [ ] `open_db` on path whose parent dir does not exist: creates parent (or uses test temp dir) and succeeds with zero rows.

### `handle_line` scenarios (table-driven or individual tests)

- [ ] Isolated melee: 1 candidate → 1 row, `weight = 1.0`, `damage_min = damage_max = hp_delta`, `candidate_count = 1`.
- [ ] Isolated skill: bash, push, kick (one line), stab (one line), scythe swipe — each produces correct `damage_category` and `message_verb`.
- [ ] Isolated spell: `A magic missile hits you.` → `message_verb = magic missile`, `source_name` empty.
- [ ] Ambiguous N=2: shared `batch_id`, `weight = 0.5`, `damage_min = 0`, `damage_max = hp_delta` on both rows.
- [ ] Ambiguous N=3: `weight ≈ 0.333` (or exact `1/3` float).
- [ ] Zero candidates + negative `H:` → 0 rows, buffer empty after.
- [ ] Empty buffer + negative `H:` → 0 rows.
- [ ] Positive HP bracket on `H:` → 0 rows, buffer cleared.
- [ ] `H:` with `[]` HP and negative SP only → 0 rows.
- [ ] Negative HP + negative SP on same `H:` → row `hp_delta` from HP bracket only.
- [ ] Miss / outgoing / dodge lines in buffer do not become candidates; `H:` with only those → 0 rows.
- [ ] Kick partial-deflect line is a candidate when followed by `H:` loss.
- [ ] Between-round bash (no round header) still attributes and writes row.
- [ ] `Hp:` prompt line does not trigger flush or corrupt buffer.
- [ ] `reset_buffer()` discards pending candidates; subsequent isolated hit still works.
- [ ] Two players in sequence: `player` column matches `handle_line` argument at flush time.

### Integration fixtures

- [ ] Replay inline Holy-man fight line sequence (with synthetic `H:` lines where needed): assert row count, verbs (`bitchslaps`, `lightly strikes`, `bash`, `push`, …), no rows on miss-only windows.
- [ ] Replay inline kick skill lines + synthetic `H:` → kick rows with `message_verb = kick`.
- [ ] Replay inline spell hit lines + synthetic `H:` → spell rows per spell name.

### Wiring (if testable without full TUI)

- [ ] `FreshSessionReset::DamageCollector` clears buffer (unit test on collector + reset call, or app-level test).
- [ ] Logout transition clears buffer (same).

### Write failure

- [ ] Document or test: failed insert logs warning and does not panic (mock/inject if practical; otherwise note in PR checklist).

## Testing seam

**Primary:** `DamageCollector::handle_line` sequence → query `damage_events`. **Secondary:** schema open/migrate helpers. Matcher unit tests remain separate — do not re-test full 286-verb catalog here.

## Implementation notes

- `rusqlite` sync; one transaction per `H:` trigger.
- `player` from `session.login_name()` at flush time.
- `recorded_at` UTC ISO-8601 at flush time.
- Monotonic `batch_id` (auto-increment or in-memory counter — either is fine if stable per process).
- Remove `#![allow(dead_code)]` from combat_damage module when wired.
