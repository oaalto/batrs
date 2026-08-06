# 02 — Damage collector and storage

## Parent

`prd.md`

## What to build

Implement **DamageCollector** on top of the existing matcher: buffer recognized incoming-damage candidates between `H:` lines; on negative HP bracket, allocate `batch_id`, compute `candidate_count`, `weight`, `damage_min`, `damage_max`, and persist rows in `~/.batrs/combat_damage.db` inside one transaction per trigger. Wire the collector on the application shell: `handle_line` before combat-awareness gag when logged in; `FreshSessionReset::DamageCollector` and login-transition `reset_buffer()`; DB open at init, close on drop. Share short-score regex for `H:` parsing. Log and continue on write failure.

**First launch:** at `BatApp` init (before HTTP thread), ensure `~/.batrs/` exists and `open_db` creates `combat_damage.db` with v1 schema if missing. Zero rows is valid. HTTP viewer depends on this file existing — do not defer creation to first insert.

## Blocked by

`01-damage-line-matcher.md` (matcher API stable).

## Status

done

## Acceptance criteria

- [x] SQLite schema v1: `damage_events` + `schema_version` at `~/.batrs/combat_damage.db` with columns per PRD (including `batch_id`, `candidate_count`, `weight`, `damage_min`, `damage_max`).
- [x] Four indexes created on first migrate: `batch_id`, `recorded_at`, `(damage_category, message_verb)`, `candidate_count`.
- [x] Inline migration from version 1; clear error if DB schema newer than binary.
- [x] `handle_line`: non-`H:` lines run through matcher; recognized candidates appended to buffer; unrelated lines ignored.
- [x] `handle_line`: negative `H:` with zero candidates → no rows, buffer cleared.
- [x] `handle_line`: negative `H:` with one candidate → one row, `weight = 1.0`, `damage_min = damage_max = hp_delta`, `candidate_count = 1`.
- [x] `handle_line`: negative `H:` with N candidates → N rows, shared `batch_id`, `weight = 1.0/N`, `damage_min = 0`, `damage_max = hp_delta`, `candidate_count = N`.
- [x] `handle_line`: positive or empty HP bracket on `H:` → no rows; buffer cleared.
- [x] SP/EP/exp/gold brackets on same `H:` line ignored for `hp_delta`.
- [x] `reset_buffer()` clears candidates only; DB connection persists.
- [x] `FreshSessionReset::DamageCollector` and logout transition call `reset_buffer()`.
- [x] Failed batch write: warn log, buffer still cleared, session continues (`log::warn!`; see implementation notes).
- [x] First launch: `~/.batrs/` created if missing; `open_db` at init creates empty v1 schema before any `handle_line` or HTTP start.
- [x] `cargo test --all-targets --all-features` passes.

## Tests (required)

All tests use a temp SQLite path; no `~/.batrs` in tests.

### Schema and migrations

- [x] Fresh open creates `damage_events`, `schema_version`, all columns, four indexes.
- [x] Second open on same file: no error, schema unchanged.
- [x] Open file with `schema_version` > binary `CURRENT`: returns clear error (no panic).
- [x] `open_db` on path whose parent dir does not exist: creates parent (or uses test temp dir) and succeeds with zero rows.

### `handle_line` scenarios (table-driven or individual tests)

- [x] Isolated melee: 1 candidate → 1 row, `weight = 1.0`, `damage_min = damage_max = hp_delta`, `candidate_count = 1`.
- [x] Isolated skill: bash, push, kick (one line), stab (one line), scythe swipe — each produces correct `damage_category` and `message_verb`.
- [x] Isolated spell: `A magic missile hits you.` → `message_verb = magic missile`, `source_name` empty.
- [x] Ambiguous N=2: shared `batch_id`, `weight = 0.5`, `damage_min = 0`, `damage_max = hp_delta` on both rows.
- [x] Ambiguous N=3: `weight ≈ 0.333` (or exact `1/3` float).
- [x] Zero candidates + negative `H:` → 0 rows, buffer empty after.
- [x] Empty buffer + negative `H:` → 0 rows.
- [x] Positive HP bracket on `H:` → 0 rows, buffer cleared.
- [x] `H:` with `[]` HP and negative SP only → 0 rows.
- [x] Negative HP + negative SP on same `H:` → row `hp_delta` from HP bracket only.
- [x] Miss / outgoing / dodge lines in buffer do not become candidates; `H:` with only those → 0 rows.
- [x] Kick partial-deflect line is a candidate when followed by `H:` loss.
- [x] Between-round bash (no round header) still attributes and writes row.
- [x] `Hp:` prompt line does not trigger flush or corrupt buffer.
- [x] `reset_buffer()` discards pending candidates; subsequent isolated hit still works.
- [x] Two players in sequence: `player` column matches `handle_line` argument at flush time.

### Integration fixtures

- [x] Replay inline Holy-man fight line sequence (with synthetic `H:` lines where needed): assert row count, verbs (`bitchslaps`, `lightly strikes`, `bash`, `push`, …), no rows on miss-only windows.
- [x] Replay inline kick skill lines + synthetic `H:` → kick rows with `message_verb = kick`.
- [x] Replay inline spell hit lines + synthetic `H:` → spell rows per spell name.

### Wiring (if testable without full TUI)

- [ ] `FreshSessionReset::DamageCollector` clears buffer (unit test on collector + reset call, or app-level test).
- [ ] Logout transition clears buffer (same).

### Write failure

- [x] Document or test: failed insert logs warning and does not panic (mock/inject if practical; otherwise note in PR checklist).

## Testing seam

**Primary:** `DamageCollector::handle_line` sequence → query `damage_events`. **Secondary:** schema open/migrate helpers. Matcher unit tests remain separate — do not re-test full 286-verb catalog here.

## Implementation notes

- `rusqlite` sync; one transaction per `H:` trigger.
- `player` from `session.login_name()` at flush time.
- `recorded_at` UTC ISO-8601 at flush time.
- `batch_id` seeded from `MAX(batch_id) + 1` on open (stable across restarts).
- Write failures: `log::warn!` (project logging stack; ticket originally mentioned `tracing`).
- `#![allow(dead_code)]` removed from `combat_damage` module when wired.
- App wiring (`FreshSessionReset::DamageCollector`, logout `reset_buffer()`) implemented in `BatApp`; covered by collector `reset_buffer` unit test only — no app-level reset-plan test yet.
