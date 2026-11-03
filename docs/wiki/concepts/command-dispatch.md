---
title: Command Dispatch
type: concept
status: current
updated: 2026-11-03
sources:
  - CONTEXT.md
  - src/command/mod.rs
  - src/guilds/nergal/commands.rs
  - src/guilds/disciple/commands.rs
  - src/guilds/liberator/commands.rs
  - src/guilds/monk/commands.rs
  - src/guilds/reaver/commands.rs
  - src/guilds/spider/commands.rs
---

# Command Dispatch

## Summary

Command Dispatch interprets a command input line into client effects (send text to BatMUD, open dialogs, emit output, automation, logging, terminal redraw via `/clear`, quit). It owns precedence and login gating; it does not own telnet I/O adapters or player persistence.

## Verified Facts

- Implementation: `src/command/mod.rs` — builtin slash commands include `/help`, `/quit`, `/connect`, `/clear`, `/guilds`, `/generic`, `/triggers`, `/show`, `/settings`, `/raw_logs`.
- `/show` (`requires_login: true`): lists active guild/generic shortcuts (`/show commands`) or guild/common line triggers (`/show triggers`); optional filter is a guild `persisted_key` or `generic`; output via `CommandEffect::Output`; handled in `dispatch` with guild selection and trigger config context, not a stateless builtin handler.
- `/triggers` opens the trigger-group toggle dialog (`requires_login: true`); same discovery and gating pattern as `/generic`. Changes persist to player TOML and apply on the next line after successful save.
- Clear Command (`/clear`): client-only terminal redraw from in-memory UI state — not an output-buffer wipe; no login gate; never forwarded to BatMUD. Distinct from Session Lifecycle scrollback clear on character change after reconnect.
- Returns effects for `BatApp` to apply; does not send game input for client-only commands (e.g. `/connect` is never forwarded to BatMUD).
- Connect Command (`/connect`): relaunches login-dependent state; only one connect attempt active at a time; never forwarded to BatMUD. On failure the client stays in fresh-session state (`CONTEXT.md`). Reconnect orchestration and reset manifest live in [Session Lifecycle](session-lifecycle.md).
- Command environment facts (runtime flags, variables) are snapshots from the application, not Player Profile data.
- With Nergal owning `us`, bare `us` sends `@use 'stab'`; `us <target>` sends `@target <target>;use 'stab' <target>`. The guild uses the shared Abilities `use_skill` helper; the dispatch regression covers both forms (`src/guilds/nergal/commands.rs`, `src/command/mod.rs`).

- Required-target shortcuts `ukd` (Disciple), `cgs` (Liberator), `cs` (Monk), `cee` (Nergal), `rt`/`cwa`/`csd` (Reaver), and `chs` (Spider) emit only local **“No target!”** output when arguments are missing. Monk validates before creating rotation-reset effects. Nergal `cne` reports its three-argument syntax; `aura`/`mutation` report missing arguments; Reaver `cca` reports its required amount. Missing-input paths neither send game commands nor change automation state; dispatch regression covers every shortcut, incomplete `cne`, and valid-input sends (`src/command/mod.rs`).

## Related

- [Session Lifecycle](session-lifecycle.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Command Dispatch section
