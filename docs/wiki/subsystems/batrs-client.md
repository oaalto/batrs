---
title: batrs client application
type: subsystem
status: current
updated: 2026-10-29
sources:
  - src/main.rs
  - src/app/mod.rs
  - src/app/telnet_buffer.rs
  - src/app/output_buffer.rs
  - src/app/scrollback.rs
  - src/ui/mod.rs
  - src/app/session_lifecycle/
  - README.md
  - CONTEXT.md
---

# batrs client application

## Summary

batrs is a Rust terminal client for [BatMUD](https://www.bat.org). It connects over telnet, renders a ratatui TUI, dispatches slash commands locally, and runs guild-specific automation through triggers.

## Verified Facts

- Entry point: `src/main.rs` — sets up crossterm/ratatui, tokio runtime, and `BatApp` (`src/app/mod.rs`). Fatal errors print their display message to stderr without Rust debug wrappers and exit with status 1. Initial connection failure occurs before terminal setup; reconnect error handling is unchanged.
- Core modules under `src/`: `app` (session/UI loop), `app/session_lifecycle/` (Connect Command fresh-session transitions, reconnect guard, stale-event filtering, scrollback disposition), `command` (slash command dispatch), `config` (player TOML under `~/.batrs/`), `guilds/` (per-guild commands and triggers), `triggers/` (shared telnet line parsers), `ui/` (view rendering), `player_profile`, `automation`.
- Network: `libmudtelnet` telnet parser; TCP connection coordinated from `main.rs` / `app`. After queueing received events, the connection notifies the UI via a shared Tokio `Notify`, reused across reconnects. The UI selects between that notification, crossterm `EventStream`, and a one-second clock deadline rather than blocking on terminal-only polling.
- Connection setup and socket I/O use the existing Tokio runtime. Reconnect schedules async setup rather than nesting `Runtime::block_on` inside the TUI runtime; a oneshot completion and shared `Notify` wake the client on either success or failure. Command channels use Tokio unbounded mpsc, so socket writers await input without blocking a runtime worker. Reader/writer lifetimes are tied by `tokio::select!`: EOF, write failure, or command-channel closure cancels the sibling future.
- `src/app/telnet_buffer.rs` releases the prefix through the last complete CRLF on every data event, retaining only the incomplete suffix. Go-Ahead releases the suffix; CRLF may span multiple data events.
- The [text input profiling note](../source-notes/text-input-slow-profiling.md) describes older loop/render behavior. Current code uses dirty redraws and a wrapped-output cache; its terminal-poll timing claim no longer applies.
- Output presentation: `BatApp::prepare_output` copies only the selected wrapped-row viewport into `ViewModel`; the existing paragraph renderer clones only those rows and applies no second scroll offset. `Scrollback` tracks clamped `usize` row ranges, so latest and earlier output remain reachable beyond 65,535 wrapped rows. Shared `Renderer::layout` keeps selection aligned with combat rows, primary status/clock, Secondary Status, and input even on tiny terminals.
- `OutputBuffer` owns a width-keyed wrapping cache and incorporated source-line count (`src/app/output_buffer.rs`). At unchanged width it wraps only pending retained lines, including multiple batches before a draw; unchanged requests, empty/gagged batches, and height-only changes do no old wrapping. Initial population and width changes wrap all history. Clear resets rows and source progress even before an equal-count refill; zero width returns no rows without consuming pending output. Source and wrapped memory still grow without a history cap; vector growth can occasionally move existing rows, so ordinary extension is amortized, not allocation-free.
- [Release benchmark evidence](../source-notes/output-history-benchmark.md) records final 10,000-line append-and-draw medians of 0.163–0.168 ms versus the original 101.132 ms (about 602–620× faster). Ordinary append-cache updates and fixed-viewport cached redraws show no material history-size trend across 100/1,000/10,000 lines. A separate extension probe reports occasional outliers rather than hiding allocation-growth limits; offline results are not live-session timing claims.
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
