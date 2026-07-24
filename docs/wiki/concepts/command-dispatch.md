---
title: Command Dispatch
type: concept
status: current
updated: 2026-07-24
sources:
  - CONTEXT.md
  - src/command/mod.rs
---

# Command Dispatch

## Summary

Command Dispatch interprets a command input line into client effects (send text to BatMUD, open dialogs, emit output, automation, logging, terminal redraw via `/clear`, quit). It owns precedence and login gating; it does not own telnet I/O adapters or player persistence.

## Verified Facts

- Implementation: `src/command/mod.rs` — builtin slash commands include `/help`, `/quit`, `/connect`, `/clear`, `/guilds`, `/generic`, `/settings`, `/raw_logs`.
- Clear Command (`/clear`): client-only terminal redraw from in-memory UI state — not an output-buffer wipe; no login gate; never forwarded to BatMUD. Distinct from Session Lifecycle scrollback clear on character change after reconnect.
- Returns effects for `BatApp` to apply; does not send game input for client-only commands (e.g. `/connect` is never forwarded to BatMUD).
- Connect Command (`/connect`): relaunches login-dependent state; only one connect attempt active at a time; never forwarded to BatMUD. On failure the client stays in fresh-session state (`CONTEXT.md`). Reconnect orchestration and reset manifest live in [Session Lifecycle](session-lifecycle.md).
- Command environment facts (runtime flags, variables) are snapshots from the application, not Player Profile data.

## Related

- [Session Lifecycle](session-lifecycle.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Command Dispatch section
