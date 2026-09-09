---
title: Text input slowness profiling note
type: source-note
status: current
updated: 2026-09-09
sources:
  - src/main.rs
  - src/app/mod.rs
  - src/app/input_state.rs
  - src/ui/mod.rs
  - Cargo.toml
  - docs/agent-commands.md
  - /home/oaalto/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-0.29.0/Cargo.toml
  - /home/oaalto/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossterm-0.29.0/Cargo.toml
---

# Text input slowness profiling note

## Summary

The most likely repo-specific cause of slow typing is not network I/O but per-keystroke work on the UI path: the app redraws the terminal every loop iteration, rebuilds wrapped output lines on every draw, clones the entire visible input string for rendering, and does `String::insert` plus full-string clone/sync on each typed character. The fastest way to confirm the real bottleneck is to profile the local input/render path while disconnected or otherwise idle, then compare that against runs with heavy server output. Sources: `src/main.rs`, `src/app/mod.rs`, `src/app/input_state.rs`, `src/ui/mod.rs`.

## Verified Facts

- The main loop redraws the terminal every iteration with `terminal.draw(|frame| app.draw(frame))?` before waiting for new input events. It then polls crossterm events with a timeout derived from a fixed `tick_rate = Duration::from_millis(200)` and drains any immediately available terminal events with `while event::poll(Duration::ZERO)? { ... }`. Source: `src/main.rs`.
- Key presses are handled synchronously in `BatApp::handle_key_event`; printable characters call `self.input.insert_char(c)`, and then the next loop iteration redraws the full frame. Source: `src/app/mod.rs`.
- `InputState::insert_char` does `self.displayed_input.insert(self.cursor_position, c)`, advances the byte cursor, then calls `sync_current_typed_input()`. `sync_current_typed_input()` clones the whole `displayed_input` into `current_typed_input` with `clone_from`. That means each typed character mutates the string in-place and then duplicates the whole current input buffer. Source: `src/app/input_state.rs`.
- `InputState::insert_str` also inserts into the middle of the string and then clones the whole current input into `current_typed_input`. Source: `src/app/input_state.rs`.
- Cursor movement and deletion over Unicode graphemes use `unicode-segmentation` helpers such as `grapheme_indices`, `unicode_word_indices`, and `graphemes(true).count()`. The direct crate dependency is present in `Cargo.toml`; `ratatui` also depends on `unicode-segmentation`. Sources: `src/app/input_state.rs`, `Cargo.toml`, `ratatui-0.29.0/Cargo.toml`, `cargo tree -i unicode-segmentation --depth 2`.
- The draw path rebuilds presentation data on every frame. `BatApp::draw` computes combat status lines, secondary status lines, then calls `self.output.wrapped_lines(output_area_width)` to produce a fresh `Vec<Line<'_>>`, builds `input_text` with `format!(">{}", self.input.displayed_text(hide_input))`, computes `cursor_offset`, builds a `ViewModel`, and passes it to `Renderer::render`. Source: `src/app/mod.rs`.
- `InputState::displayed_text(false)` returns `self.displayed_input.clone()`, so rendering the input line clones the entire current input string every frame when input is visible. Source: `src/app/input_state.rs`.
- `Renderer::render` clones `view.output_lines` into a new `Text` for the output paragraph and clones line values for combat/secondary rows before rendering. It also clones `view.input_text` into a `Paragraph`. Source: `src/ui/mod.rs`.
- The output area is rendered from a wrapped paragraph: `Paragraph::new(Text::from(view.output_lines.clone())).scroll((view.scroll_offset, 0))`. If output history is large, rebuilding and cloning wrapped output lines every frame can dominate the cost of typing because typing causes redraws too. Sources: `src/app/mod.rs`, `src/ui/mod.rs`.
- The app currently depends on `ratatui = "0.29"` and `crossterm = "0.29"`. `crossterm` 0.29 enables terminal events by default and includes examples for `event-poll-read` and async event streams; `ratatui` ships criterion benches and a `tracing` example, which indicates these libraries support profiling-oriented workflows even though this repo does not currently include profiling crates. Sources: `Cargo.toml`, `crossterm-0.29.0/Cargo.toml`, `ratatui-0.29.0/Cargo.toml`.
- The repo’s documented mechanical gates are `cargo fmt --all`, `cargo build --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features`. Source: `docs/agent-commands.md`.

## Plausible root causes in this codebase

### 1. Full-frame redraw cost on every keypress

Because `run_app` calls `terminal.draw(...)` every loop iteration, typing does not only update the input widget; it redraws output, stats, secondary status, dialogs, and cursor placement too. If the output area is large or wrapping is expensive, this can make input feel laggy even when `insert_char` itself is cheap. Sources: `src/main.rs`, `src/app/mod.rs`, `src/ui/mod.rs`.

### 2. Re-wrapping / cloning output history during typing

`BatApp::draw` calls `self.output.wrapped_lines(output_area_width)` on every frame, then `Renderer::render` clones `view.output_lines` again to build the rendered paragraph. If the output buffer is long, per-keystroke redraw cost scales with scrollback/output size rather than only with current input size. Sources: `src/app/mod.rs`, `src/ui/mod.rs`.

### 3. Per-keystroke string copying in input state

Each character insertion updates `displayed_input` and then clones the whole string into `current_typed_input`. Rendering also clones the displayed input string again through `displayed_text(false)` and `format!`. This is small for short commands, but it grows with longer command lines and pasted text. Sources: `src/app/input_state.rs`, `src/app/mod.rs`.

### 4. Grapheme-aware cursor math on every relevant edit

The code correctly handles Unicode graphemes and word motion, but grapheme scans and counts are more expensive than raw byte indexing. This is probably not the first bottleneck for plain short ASCII commands, but it is a valid profiling candidate for long inputs or heavy cursor movement. Sources: `src/app/input_state.rs`, `Cargo.toml`.

### 5. Input lag caused by contention with incoming MUD output

The main loop handles both local key events and channel-driven telnet input. `app.read_input()` drains queued `AppEvent`s and may call `process_input_lines(lines)` before the loop returns to polling keyboard input. If the server is producing lots of lines, typing can feel slow because local input shares the same single-threaded app/update/render loop. Sources: `src/main.rs`, `src/app/mod.rs`.

## How to confirm each cause

### A. Separate local typing cost from network/output cost

1. Start the app in a state with no incoming telnet traffic, or temporarily instrument a disconnected/quiet scenario.
2. Type into the input box and compare that against typing while the server is spamming output.
3. If lag appears only under heavy output, the hot path is more likely `read_input`/`process_input_lines`/wrapping/render, not raw key handling. Sources: `src/main.rs`, `src/app/mod.rs`.

### B. Measure whether redraw time scales with scrollback size

1. Compare typing responsiveness with a nearly empty output buffer versus after a large amount of accumulated output.
2. If lag grows with output history, the likely culprit is `self.output.wrapped_lines(output_area_width)` plus render-time cloning, not `InputState` alone. Sources: `src/app/mod.rs`, `src/ui/mod.rs`.

### C. Measure whether lag scales with input length

1. Type a short command like `look` repeatedly.
2. Then type or paste a very long command line and edit within the middle.
3. If lag grows with input length even when output is quiet, the likely culprit is `InputState` string mutation plus cloning and grapheme scans. Source: `src/app/input_state.rs`.

### D. Time specific phases directly

Add temporary `Instant::now()` timing around:

- `terminal.draw(...)` in `run_app`,
- `self.output.wrapped_lines(...)` inside `BatApp::draw`,
- `self.input.insert_char(c)` handling path,
- `self.process_input_lines(lines)` when telnet input arrives.

This is a low-risk way to decide whether the time is going into rendering, input editing, or remote-output processing. Sources: `src/main.rs`, `src/app/mod.rs`, `src/app/input_state.rs`.

## Recommended profiling tools and commands

## Repo-safe baseline

- Build a release binary before profiling so measurements reflect optimized code; debug builds can exaggerate redraw and string-processing cost. The repo’s standard build gate is `cargo build --all-targets`. Source: `docs/agent-commands.md`.

Suggested commands:

```bash
cargo build --release
cargo run --release
```

## Linux system profiler route

For this terminal app, the simplest external profiler is usually Linux `perf` against a release build. It can show whether time is going into ratatui rendering, wrapping, string cloning, Unicode segmentation, or app-specific processing.

Suggested commands:

```bash
cargo build --release
perf record --call-graph dwarf target/release/batrs
perf report
```

Focus on stacks involving:

- `batrs::app::BatApp::draw`
- `batrs::ui::Renderer::render`
- `batrs::app::input_state::*`
- `batrs::app::BatApp::process_input_lines`
- Unicode segmentation helpers and ratatui paragraph/layout code

Repo evidence for those hot-path symbols: `src/main.rs`, `src/app/mod.rs`, `src/app/input_state.rs`, `src/ui/mod.rs`.

## Sampling with repeated repros

If the slowdown is intermittent, record while reproducing a concrete scenario:

1. idle typing,
2. typing with large scrollback,
3. typing during heavy inbound output.

Compare the reports. This often identifies whether the problem is render-bound or input-processing-bound. Sources: `src/main.rs`, `src/app/mod.rs`.

## Built-in microbenchmark / focused timing route

This repo does not currently declare `criterion` or `pprof` in its own `Cargo.toml`, so the lowest-friction route is temporary in-code timing or ad hoc microbenches rather than immediately adding new dependencies. Source: `Cargo.toml`.

Useful local timing targets:

- `InputState::insert_char` on short vs long strings,
- `InputState::cursor_offset(false)` on long strings,
- `self.output.wrapped_lines(width)` with realistic output-buffer sizes,
- full `BatApp::draw` with synthetic long output.

## Low-risk speedup options likely applicable here

### 1. Stop redrawing when nothing changed

The biggest likely win is a dirty-flag approach: redraw only after state changes, not every pass through the loop. Right now `terminal.draw(...)` runs unconditionally each iteration. A dirty flag set by key events, incoming lines, dialog changes, scrollback changes, and clock ticks would avoid extra full-frame work. Source: `src/main.rs`.

### 2. Cache wrapped output lines until output or width changes

`self.output.wrapped_lines(output_area_width)` appears on the hot draw path. If wrapping results are cached and invalidated only when the output buffer changes or the terminal width changes, typing into the input line would no longer force re-wrapping the whole output history. Source: `src/app/mod.rs`.

### 3. Avoid cloning the full input string multiple times per keypress

Low-risk improvements inside `InputState` / draw path:

- keep `current_typed_input` untouched while editing history-free current input, or update it only on transitions where history navigation needs it,
- return `&str` for visible input rendering instead of cloning in `displayed_text(false)`,
- avoid `format!(">{}", cloned_string)` if the UI can render prompt and input separately.

These are plausible improvements because the current implementation clones the input buffer in both `sync_current_typed_input()` and `displayed_text(false)`. Sources: `src/app/input_state.rs`, `src/app/mod.rs`, `src/ui/mod.rs`.

### 4. Defer expensive cursor calculations unless the cursor moved

`cursor_offset(false)` counts graphemes in the prefix every draw. Caching the display-column cursor offset and updating it only on input edits/cursor movement can reduce repeated scans. Source: `src/app/input_state.rs`.

### 5. Reduce shared-loop pressure from inbound output
n
If profiling shows `process_input_lines` dominates while typing under spam, batch or coalesce inbound-line processing and redraw less often than input arrival. The current code drains all queued channel events in `read_input()`, so bursts of incoming lines can monopolize the loop. Sources: `src/app/mod.rs`, `src/main.rs`.

## Practical diagnosis order

1. Reproduce in `--release`.
2. Compare idle typing vs typing during heavy output.
3. Compare empty scrollback vs large scrollback.
4. Add temporary timings around `terminal.draw`, `wrapped_lines`, `insert_char`, and `process_input_lines`.
5. Run `perf record --call-graph dwarf target/release/batrs` on the slow scenario.
6. Fix the biggest bucket first; likely order is redraw/wrapping, then input-string cloning, then finer-grained Unicode/cursor costs. Sources: `src/main.rs`, `src/app/mod.rs`, `src/app/input_state.rs`, `src/ui/mod.rs`, `docs/agent-commands.md`.

## Agent Synthesis

The repo-specific evidence points first at render-path cost, not exotic terminal-event latency. The unconditional `terminal.draw(...)` in the main loop plus output re-wrapping inside `BatApp::draw` means a single typed character can trigger work proportional to the whole visible screen and possibly the entire wrapped output state. `InputState` cloning is also real, but it is more likely a secondary bottleneck unless users commonly type very long commands or paste large blocks. Sources: `src/main.rs`, `src/app/mod.rs`, `src/app/input_state.rs`, `src/ui/mod.rs`.

## Open Questions

- How large does `self.output.wrapped_lines(output_area_width)` get in a realistic long BatMUD session? The answer depends on `src/app/output_buffer.rs`, which should be profiled next if redraw cost dominates.
- Does `ratatui::Terminal::draw` diff efficiently enough for this layout, or is most time spent preparing `ViewModel` data before the backend write? A stack sample will answer this.
- Is the slowness present in release builds, or mainly in debug? This has to be verified by running the app locally.

## Related

- [batrs client application](../subsystems/batrs-client.md)
- [Command Dispatch](../concepts/command-dispatch.md)
- [Session Lifecycle](../concepts/session-lifecycle.md)
