# Connect Command session lifecycle

## Status

draft — initial exploration for grilling

## Problem Statement

The Connect Command's relaunch semantics are implemented as a scattered reset in `BatApp`: `prepare_fresh_session` manually clears a dozen fields, `start_reconnect` interleaves connection coordination with session reset, and login-dependent reload (`load_user_config`, guild selection) happens elsewhere on login transition. `CONTEXT.md` documents precise Connect Command behavior (single active attempt, fresh-session on failure, Player Profile reload only after login), but no module owns that lifecycle. Adding a new session-scoped field requires remembering to reset it in `prepare_fresh_session`.

## Initial exploration

| Piece | Location | Behavior |
| --- | --- | --- |
| `/connect` command | `command/mod.rs` `builtin_connect` | Emits `CommandEffect::Reconnect` before and after login |
| Reconnect orchestration | `app/mod.rs` `start_reconnect` | Guards `reconnect_in_progress`, increments `connection_id`, calls `prepare_fresh_session`, calls `ConnectionCoordinator::reconnect` |
| Session reset | `app/mod.rs` `prepare_fresh_session` | Resets session, stats, combat_scan, telnet buffer, guilds, automation, player_profile default, dialogs, `user_config_loaded = false` |
| Connection install | `install_connection` | Swaps event receiver and command sender |
| Stale event drop | `read_input` | Ignores events where `connection_id != active_connection_id` |
| Profile reload | `process_input_lines` on login transition | `load_user_config` when `!was_logged_in && is_logged_in` |
| Tests | `app/mod.rs` | Fake `ConnectionCoordinator`; reconnect while logged in clears stats/guilds |

**`ConnectionCoordinator` trait** already exists — good seam for adapter (prod network vs. test fake).

**Not reset in `prepare_fresh_session` (verify during grilling):** output scrollback, loggers, `should_quit`, config_manager handle, raw log toggle state?

## Solution (proposed direction)

Extract a **session lifecycle** module (name TBD in grilling) that owns fresh-session transitions: what clears immediately on Connect Command, what waits for login, connection generation counter, reconnect-in-progress guard. `BatApp` delegates `start_reconnect` and `prepare_fresh_session` to it. Command Dispatch keeps emitting `Reconnect`; the lifecycle module applies CONTEXT semantics.

## User Stories

1. As a player, I want `/connect` to start a fresh BatMUD login when my session is stuck, so that I can recover without restarting batrs.
2. As a player, I want a failed reconnect to leave me in a fresh-session state with an error message, so that I am not stuck believing the old session is active.
3. As a player, I want repeated `/connect` while reconnect is in progress to be rejected clearly, so that I do not spawn parallel connection attempts.
4. As a maintainer, I want one place listing session-scoped state cleared on Connect Command, so that new fields do not leak across reconnects.
5. As a maintainer, I want connection id stale-event filtering colocated with lifecycle, so that the concurrency model is obvious.
6. As a test author, I want to exercise reconnect semantics without constructing the full UI app where possible.
7. As a player logging in after connect, I want my Player Profile reloaded from disk, so that guild and settings match my saved config.

## Open questions (for grilling)

1. **Module boundary:** Lifecycle module owns only reset + reconnect guard, or also `ConnectionCoordinator` wiring?
2. **Player Profile timing:** Confirm reload only post-login matches desired behavior for connect-while-logged-in (current: immediate clear to default, reload on next login).
3. **Output/history:** Should scrollback and styled output clear on connect, or persist across reconnect?
4. **Automation state:** Full `Automation::new()` vs. selective flag clear — any waiters that should survive?
5. **Interface shape:** Single `fn begin_fresh_session(&mut self) -> FreshSessionPlan` returning what app must rewire, or mutable session object held by app?
6. **Naming:** Align module name with `CONTEXT.md` ("Connect Command") vs. generic "session lifecycle".

## Implementation Decisions (tentative)

- Keep `CommandEffect::Reconnect` and `ConnectionCoordinator` as existing seams; deepen behind app shell.
- `reconnect_in_progress` and `active_connection_id` move into lifecycle module state.
- `prepare_fresh_session` field list becomes authoritative manifest; app calls one method.
- Preserve: only one active connect attempt; failed reconnect does not restore prior session.
- Login-gated profile load stays in app or moves to lifecycle callback — decide in grilling.

## Testing Decisions

- **Good tests:** Connect while logged in clears guilds/stats; double connect message; failed reconnect leaves `reconnect_in_progress` false; stale connection events ignored.
- **Prior art:** `app/mod.rs` reconnect tests with `FakeConnectionCoordinator`; `command/mod.rs` dispatch tests for `/connect`.
- **Avoid:** Testing connection coordinator implementation in lifecycle tests — use fake adapter.

## Out of Scope

- Changing BatMUD login protocol or telnet layer.
- Auto-reconnect on disconnect (only explicit `/connect`).
- Persisting session state across batrs restarts.
- Network retry/backoff policy inside coordinator.

## Further Notes

- Recommendation strength: **Strong** — `CONTEXT.md` already specifies behavior; code spread is the friction.
- Recent commit: `f1c0732 Add connect command reconnect flow`.
- No ADR directory in repo; no conflicts found.
