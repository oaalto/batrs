---
title: Command Dispatch
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/command/mod.rs
---

# Command Dispatch

## Summary

Command Dispatch interprets a command input line into client effects (send text to BatMUD, open dialogs, emit output, automation, logging, quit). It owns precedence and login gating; it does not own telnet I/O adapters or player persistence.

## Verified Facts

- Implementation: `src/command/mod.rs` — builtin slash commands include `/help`, `/quit`, `/connect`, `/guilds`, `/generic`, `/settings`, `/raw_logs`.
- Returns effects for `BatApp` to apply; does not send game input for client-only commands (e.g. `/connect` is never forwarded to BatMUD).
- Connect Command: relaunches login-dependent state; only one connect attempt active at a time (`CONTEXT.md`).
- Command environment facts (runtime flags, variables) are snapshots from the application, not Player Profile data.

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Command Dispatch section
