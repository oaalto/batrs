---
title: Session Lifecycle
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/app/session_lifecycle/mod.rs
  - src/app/session_lifecycle/fresh_session.rs
  - src/app/session_lifecycle/output_disposition.rs
  - src/app/session_lifecycle/connect_command.rs
  - src/app/mod.rs
---

# Session Lifecycle

## Summary

Session Lifecycle is the application-owned bounded context for fresh-session transitions triggered by the Connect Command (`/connect`). It owns the reset manifest, reconnect-in-progress guard, connection generation counter, stale-event filtering, reconnect orchestration, and scrollback disposition after reconnect login.

## Verified Facts

- Implementation: `src/app/session_lifecycle/` — `SessionLifecycle` struct in `mod.rs`; submodules for connect preparation, fresh-session plan, output disposition, reconnect execution, and stale-event checks.
- `FreshSessionPlan` (`fresh_session.rs`) is the authoritative manifest of session-scoped state cleared on `/connect`: session, stats, combat awareness, telnet buffer, guild selection, automation, user-config-loaded flag, player profile, generic commands, and open dialogs.
- `prepare_connect` / `complete_connect` (`connect_command.rs`) set the reconnect guard, bump the connection id, and delegate socket work to an injected `ConnectionCoordinator`.
- `SessionLifecycle::is_stale(connection_id)` drops events from superseded connections after a fresh session begins (`stale_events.rs`).
- Failed reconnect clears the guard and retains the fresh-session connection id; retry uses the next id (`mod.rs` tests).
- Scrollback disposition (`output_disposition.rs`): on first login after reconnect, compare the pre-connect login name (snapshotted at first `begin_fresh_session`) to the post-connect login name. Same character (case-insensitive ASCII) keeps scrollback; different character or connect-before-login clears output. `BatApp` applies `ClearOutput` in `process_input_lines` (`src/app/mod.rs`).
- Session Lifecycle does not own Player Profile disk I/O, login parsing, UI rendering, output buffers, or the production telnet adapter (`CONTEXT.md`).
- Command Dispatch emits `CommandEffect::Reconnect`; `BatApp::start_reconnect` calls Session Lifecycle and applies `FreshSessionPlan` resets.

## Related

- [Command Dispatch](command-dispatch.md)
- [Player Profile](player-profile.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Session Lifecycle and Connect Command sections
