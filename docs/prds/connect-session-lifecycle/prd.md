# Connect Command session lifecycle

## Status

ready-for-agent — grilled 2026-07-23

## Problem Statement

The Connect Command's relaunch semantics are scattered across the application shell. Fresh-session reset manually clears a dozen fields in one method, reconnect orchestration interleaves connection coordination with session reset, stale-event filtering lives in input reading, and login-dependent Player Profile reload happens on a separate login transition path. `CONTEXT.md` documents precise Connect Command behavior (single active attempt, fresh-session on failure, Player Profile reload only after login), but no module owns that lifecycle as a whole. Adding a new session-scoped field requires remembering to reset it in the manual clear list.

## Solution

Extract **Session Lifecycle** as a bounded context under the application module. Session Lifecycle owns fresh-session transitions triggered by the Connect Command: what runtime state clears immediately, reconnect-in-progress guard, connection generation counter, stale-event filtering rules, and reconnect orchestration. Command Dispatch continues emitting reconnect effects; Session Lifecycle applies Connect Command semantics. The application shell remains a thin adapter: it applies `FreshSessionPlan` to its owned fields, keeps `ConnectionCoordinator` injection, handles login-gated Player Profile reload, and implements conditional output/scrollback retention based on whether the post-connect login name matches the pre-connect character.

Connect Command stays the player-facing slash command name; its implementation file holds only the `/connect` entry point and delegates to Session Lifecycle.

## User Stories

1. As a player, I want `/connect` to start a fresh BatMUD login when my session is stuck, so that I can recover without restarting batrs.
2. As a player, I want a failed reconnect to leave me in a fresh-session state with an error message, so that I am not stuck believing the old session is active.
3. As a player, I want repeated `/connect` while reconnect is in progress to be rejected clearly, so that I do not spawn parallel connection attempts.
4. As a player logging in after connect, I want my Player Profile reloaded from disk, so that guild and settings match my saved config.
5. As a player who reconnects as the same character, I want my output scrollback preserved through the reconnect gap, so that I can still read what happened before the connection died.
6. As a player who reconnects as a different character, I want output and scrollback cleared on login, so that I do not see another character's session text.
7. As a player who runs `/connect` before logging in, I want output cleared when I complete login, so that pre-login noise does not carry into the new session.
8. As a maintainer, I want one authoritative manifest of session-scoped state cleared on Connect Command, so that new fields do not leak across reconnects.
9. As a maintainer, I want connection generation and stale-event filtering colocated with Session Lifecycle, so that the concurrency model is obvious.
10. As a maintainer, I want reconnect orchestration (guard, coordinator call, channel install handoff) in Session Lifecycle, so that `BatApp` does not interleave connection and reset concerns.
11. As a test author, I want to exercise reconnect semantics through Session Lifecycle at a high seam, so that tests do not require constructing the full UI application where possible.
12. As a maintainer, I want Connect Command implementation limited to the slash-command entry, so that reconnect and fresh-session mechanics live in dedicated sibling modules.
13. As a player, I want automation fully reset on connect, so that stale waiters from a dead connection cannot fire on a new telnet stream.
14. As a maintainer, I want `ConnectionCoordinator` to remain an injected seam (production network vs. test fake), so that lifecycle tests do not depend on real telnet.
15. As a maintainer, I want Session Lifecycle documented in `CONTEXT.md`, so that domain vocabulary matches implementation ownership.

## Implementation Decisions

### Module boundary (medium)

Session Lifecycle owns:

- Fresh-session reset manifest (`FreshSessionPlan`)
- Reconnect-in-progress guard
- Active connection generation counter
- Stale-event filtering (`is_stale(connection_id)`)
- Reconnect orchestration (guard → plan → coordinator call → channel install handoff)

The application shell keeps:

- `ConnectionCoordinator` injection (not owned by lifecycle)
- Applying `FreshSessionPlan` to owned fields (`Stats`, `Automation`, `SessionState`, etc.)
- Login-gated Player Profile reload (`load_user_config` on login transition)
- Conditional output/scrollback clear on post-connect login (character name compare)
- UI rendering, dialogs, loggers, `should_quit`, config manager handle

Command Dispatch unchanged: emits `CommandEffect::Reconnect`; application routes to Connect Command entry → Session Lifecycle.

### Module layout

Introduce a `session_lifecycle` folder module under the application layer with sibling files by concern:

- **Connect Command entry** — `/connect` trigger only; delegates to lifecycle
- **Reconnect** — guard, coordinator invocation, success/failure outcomes, channel handoff
- **Fresh session** — `FreshSessionPlan` and authoritative reset manifest
- **Stale events** — connection id state and stale check
- **Module root** — `SessionLifecycle` struct holding coordination state; re-exports

Exact file split may shift during implementation; rule is Connect Command entry does not absorb reconnect, plan, or stale-event logic.

### Public interface (plan struct)

Session Lifecycle exposes roughly:

- `begin_fresh_session(pre_connect_login_name) → FreshSessionPlan` — bumps connection id, records pre-connect character name for later scrollback decision, returns what the application must reset
- `try_start_reconnect(coordinator) → ReconnectOutcome` — enforces single active attempt; runs fresh session plan; calls coordinator; returns connected channels or failure message
- `is_stale(connection_id) → bool` — stale-event drop for incoming app events
- `on_post_connect_login(login_name) → OutputDisposition` — `KeepScrollback` if names match pre-connect snapshot (including both absent), `ClearOutput` otherwise

`FreshSessionPlan` enumerates session-scoped resets the application applies in one method (session, stats, combat awareness, telnet buffer, guild selection, automation, player profile default, generic commands, dialogs, `user_config_loaded = false`, etc.). Not in plan: output, scrollback, input, loggers, `should_quit`, config manager.

### Player Profile timing

On Connect Command: immediately clear Player Profile to default and set `user_config_loaded = false`. Do not read disk until next successful login transition. Matches existing `CONTEXT.md` Connect Command relaunch semantics.

### Output and scrollback

- Through connect: preserve output buffer and scrollback (do not clear in fresh-session plan).
- Snapshot pre-connect login name before session reset.
- On post-connect login transition: if new login name equals pre-connect name, keep output and scrollback; if different (or pre-connect had no name and post-connect does), clear output and reset scrollback to follow-latest.
- Connect while never logged in this session: no pre-connect name → clear on first login.

### Automation reset

Full `Automation::new()` on fresh session (waiters, flags, vars wiped). Guild automation re-registers after Player Profile reload on login. No waiter preservation across connect.

### Preserved invariants

- Only one Connect Command attempt active at a time; duplicate reports "reconnect already in progress."
- Failed reconnect leaves fresh-session state; does not restore prior session.
- Stale events from superseded connection ids are dropped before telnet processing.
- `CommandEffect::Reconnect` and `ConnectionCoordinator` remain existing seams.

### CONTEXT.md update

Add **Session Lifecycle** section defining ownership (fresh-session transitions, reconnect guard, connection generation, stale events). Connect Command section remains the player-facing trigger under Command Dispatch; cross-reference Session Lifecycle for implementation ownership.

## Testing Decisions

### Primary test seam

Test through **Session Lifecycle public interface** at the highest seam — reconnect guard, fresh-session plan contents, stale-event filtering, and reconnect outcomes. Use `FakeConnectionCoordinator` (existing pattern); do not test production telnet in lifecycle tests.

### Good tests (external behavior)

- Connect while logged in clears stats, guilds, automation (full reset); profile default until re-login.
- Double `/connect` while in progress → rejection message; single coordinator call.
- Failed reconnect → `reconnect_in_progress` false; fresh-session state retained; error surfaced.
- Stale connection events ignored after reconnect bumps connection id.
- Same character after connect → output/scrollback preserved on login.
- Different character after connect → output cleared, scrollback reset on login.
- Connect before login → output cleared on first login.
- Successful reconnect installs new event receiver and command sender.

### Prior art

- Application reconnect tests with `FakeConnectionCoordinator` in `app/mod.rs`.
- Command dispatch tests for `/connect` in `command/mod.rs`.

### Avoid

- Testing `ConnectionCoordinator` production implementation inside lifecycle unit tests.
- Asserting internal field order inside `FreshSessionPlan` application beyond manifest completeness.

## Out of Scope

- Changing BatMUD login protocol or telnet layer.
- Auto-reconnect on disconnect (only explicit `/connect`).
- Persisting session state across batrs restarts.
- Network retry/backoff policy inside coordinator.
- Clearing input buffer or command history on connect (unless added in a follow-up).
- Moving Player Profile reload into Session Lifecycle (stays in application shell on login transition).

## Further Notes

- Recommendation strength: **Strong** — `CONTEXT.md` already specifies behavior; code spread is the friction.
- Related: Combat Awareness PRD user story 14 (fresh-session reset clears combat state) — manifest must include combat awareness.
- Grilling decisions: boundary B (medium), profile timing keep, conditional output keep, plan struct A, hybrid naming (`session_lifecycle` + thin `connect_command`), full automation reset, file split by concern.
- Prior commit: `f1c0732 Add connect command reconnect flow`.
