---
title: batrs client application
type: subsystem
status: current
updated: 2026-07-23
sources:
  - src/main.rs
  - src/app/mod.rs
  - src/app/session_lifecycle/
  - README.md
  - CONTEXT.md
---

# batrs client application

## Summary

batrs is a Rust terminal client for [BatMUD](https://www.bat.org). It connects over telnet, renders a ratatui TUI, dispatches slash commands locally, and runs guild-specific automation through triggers.

## Verified Facts

- Entry point: `src/main.rs` — sets up crossterm/ratatui, tokio runtime, and `BatApp` (`src/app/mod.rs`).
- Core modules under `src/`: `app` (session/UI loop), `app/session_lifecycle/` (Connect Command fresh-session transitions, reconnect guard, stale-event filtering, scrollback disposition), `command` (slash command dispatch), `config` (player TOML under `~/.batrs/`), `guilds/` (per-guild commands and triggers), `triggers/` (shared telnet line parsers), `ui/` (view rendering), `player_profile`, `automation`.
- Network: `libmudtelnet` telnet parser; TCP connection coordinated from `main.rs` / `app`.
- User manual: MkDocs site under `docs/` (`mkdocs.yml`, Material theme).
- Domain glossary: `CONTEXT.md` at repo root.

## Agent Synthesis

- Guild code is organized by guild keyword (`src/guilds/<name>/`) rather than vertical slices; treat each guild folder as a capability module with commands + triggers.

## Related

- [Command Dispatch](../concepts/command-dispatch.md)
- [Session Lifecycle](../concepts/session-lifecycle.md)
- [Guild Catalog](../concepts/guild-catalog.md)
- `CONTEXT.md`
- [User manual](../../manual/ui.md)
