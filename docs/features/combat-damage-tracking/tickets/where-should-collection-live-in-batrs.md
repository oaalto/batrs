---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

Where should the **collector** live in batrs architecture?

Options to compare:

- New top-level module (`combat_damage` / `damage_tracker`) called from `process_input_lines` alongside combat awareness
- Extension inside `combat_awareness` (same line stream, combat scope already known)
- Trigger-side effects emitting damage events (reuse trigger pipeline)

Also: coupling to `StatsEffect::UpdateShortScore` for HP deltas vs re-parsing status lines in the collector; gag/scrollback interaction; behavior on reconnect and `Connect Command` fresh session.

### Q1 — module boundary (resolved)

**Decision:** **A** — new top-level `combat_damage` module; `DamageCollector` field on `BatApp`; `handle_line(&plain_line, player_name)` called from `process_input_lines` when logged in. Collector owns buffer + DB + matchers; independent of combat-awareness round/probe logic.

### Q2 — HP trigger and line ordering (resolved)

**Decision:** single early `handle_line` call on every logged-in line, **before** combat-awareness gag/continue. Collector **re-parses** `H:` internally (share `SC_REGEX` with `short_score`). No coupling to `StatsEffect::UpdateShortScore`. Gagged lines still enter the candidate buffer; `H:` detection is not skipped by gag.

### Q3 — buffer lifecycle on reconnect / Connect Command (resolved)

**Decision:** clear in-memory candidate buffer on session boundaries; **reuse existing reset paths**:

1. **`FreshSessionReset::DamageCollector`** — new variant in `fresh_session.rs`, wired in `apply_fresh_session_plan` alongside `CombatAwareness` / `Stats`. Calls `damage_collector.reset_buffer()` only (DB connection stays open; no full `Default` reset).
2. **Login-state transition** — in existing `process_input_lines` block that already tracks `was_logged_in` / `update_login_state`: when `was_logged_in && !session.is_logged_in()`, call `reset_buffer()` (covers disconnect / wrong-password without `/connect`).

**Not reset:** SQLite DB (`~/.batrs/combat_damage.db`) — opened once at `BatApp` init, closed on `Drop`. Continuous play on same login keeps buffer between `H:` lines as normal.

### Q4 — always on vs opt-out (resolved)

**Decision:** **always on** — no config flag or slash toggle in v1. Collector active whenever logged in. Opt-out deferred to fog on map.

### Q5 — DB write failures (resolved)

**Decision:** **log and continue** — `tracing::warn!` on failed batch write; drop the batch; keep playing; buffer clears normally. Combat play must not break because analytics failed.

## Resolution

**Module:** new top-level `combat_damage` module; `DamageCollector` field on `BatApp`.

**Pipeline hook:** `damage_collector.handle_line(&plain_line, player_name)` on every logged-in line in `process_input_lines`, **before** combat-awareness gag/continue. `player_name` from `session.login_name()` (same as other session-scoped subsystems).

**HP detection:** collector re-parses `H:` internally; share `SC_REGEX` with `short_score`. No coupling to `StatsEffect::UpdateShortScore`.

**Buffer reset:** `FreshSessionReset::DamageCollector` → `reset_buffer()` in `apply_fresh_session_plan`; also `reset_buffer()` on `was_logged_in && !session.is_logged_in()` in existing login-transition block.

**DB lifecycle:** open `~/.batrs/combat_damage.db` once at `BatApp` init; close on `Drop`. Not reset on reconnect or `/connect`.

**Enabled:** always on when logged in (no v1 toggle).

**Write failures:** log and continue; never block or crash the session.
