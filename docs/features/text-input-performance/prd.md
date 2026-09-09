# Text Input Performance PRD

## Status

draft

## Summary

Text input in the batrs client can feel slow because the current input/render path does more work per keystroke than necessary. The goal of this feature is to make text entry feel responsive under normal BatMUD use by reducing per-keystroke work in the batrs client without changing visible behavior, Command Dispatch semantics, or Session Lifecycle behavior.

This PRD covers five candidate improvements that should be validated and then implemented in the smallest useful order:

1. stop redrawing when nothing changed,
2. cache wrapped output lines,
3. avoid cloning the full input string for render,
4. stop cloning `current_typed_input` on every edit,
5. cache cursor display offset.

## Problem

The batrs client redraws the terminal every main-loop iteration via `terminal.draw(|frame| app.draw(frame))`, and `BatApp::draw` rebuilds wrapped output lines from the full output buffer on every frame. The input path also clones the current input string during editing and rendering. Together, these make local typing latency scale with work outside the immediate input field, especially when output history is large or the app is processing incoming BatMUD lines at the same time. Sources: `src/main.rs`, `src/app/mod.rs`, `src/app/output_buffer.rs`, `src/app/input_state.rs`, `src/ui/mod.rs`.

## Desired Outcomes

- Typing short commands feels responsive even after long play sessions with large output history.
- Typing responsiveness degrades less under heavy output.
- The batrs client keeps the same visible behavior for output, scrollback, dialogs, command submission, and cursor placement.
- The implementation stays within the current Rust crate and Cargo workflow; no new profiling or runtime dependencies are required for the first pass.

## Non-Goals

- No redesign of Command Dispatch, Player Profile, or Session Lifecycle.
- No new terminal UI library or input subsystem.
- No speculative scrollback storage rewrite unless the first five changes prove insufficient.
- No behavior change to Unicode/grapheme correctness.
- No remote-runtime/network optimization work beyond reducing local shared-loop pressure when processing inbound lines.

## Current Evidence

- `run_app` redraws the terminal every loop iteration before polling for terminal events. Source: `src/main.rs`.
- `BatApp::draw` calls `self.output.wrapped_lines(output_area_width)` on every frame. Source: `src/app/mod.rs`.
- `OutputBuffer::wrapped_lines` iterates all stored lines and collects a fresh wrapped vector. Source: `src/app/output_buffer.rs`.
- `InputState::insert_char` and `insert_str` both call `sync_current_typed_input()`, which clones the whole displayed input into `current_typed_input`. Source: `src/app/input_state.rs`.
- `InputState::displayed_text(false)` returns a cloned `String`, and `BatApp::draw` builds `input_text` with `format!(">{}", ...)`. Sources: `src/app/input_state.rs`, `src/app/mod.rs`.
- `InputState::cursor_offset(false)` counts graphemes in the current prefix on demand. Source: `src/app/input_state.rs`.
- Scrollback currently stores an offset into the rendered line count; it does not limit wrapping work before `wrapped_lines()` builds the full rendered list. Source: `src/app/scrollback.rs`.

## User Stories

- As a player, I want typing a short command like `look` or `kill orc` to feel immediate even when the output buffer is large.
- As a player, I want editing a longer command in the middle of the line to remain usable without visible lag.
- As a player, I want typing to remain responsive while BatMUD is producing output.
- As a maintainer, I want the fix to be measurable with repo-native Cargo workflows and targeted timing/profiling.

## Constraints

- Preserve existing domain behavior and terminology from `CONTEXT.md`, especially Command Dispatch and Session Lifecycle ownership boundaries.
- Keep the first pass dependency-free beyond the existing crate set in `Cargo.toml`.
- Prefer surgical changes in current seams: `src/main.rs`, `src/app/mod.rs`, `src/app/output_buffer.rs`, `src/app/input_state.rs`, `src/ui/mod.rs`.
- Validation language should match the repo’s Cargo gates: `cargo fmt --all`, `cargo build --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`.

## Proposed Scope

### 1. Stop redrawing when nothing changed

Introduce a redraw-invalidating mechanism in the main loop so `terminal.draw(...)` only runs when a state change requires a new frame.

Likely invalidators:
- keyboard input edits,
- paste,
- scrollback movement,
- output append/clear,
- dialog open/close/update,
- stats or secondary status changes,
- reconnect/session transitions,
- clock changes if the clock remains visible and time-driven.

Primary seam: `src/main.rs` and `src/app/mod.rs`.

### 2. Cache wrapped output lines

Change the output rendering path so wrapped output lines are cached and invalidated only when the output buffer changes or the terminal width changes.

Likely cache inputs:
- output generation/version,
- terminal width.

Primary seam: `src/app/output_buffer.rs`, with call-site adjustments in `src/app/mod.rs`.

### 3. Avoid cloning the full input string for render

Remove unnecessary per-frame input cloning from the render path.

Likely direction:
- expose visible input as `&str` when not hidden,
- render prompt and input text without `format!(">{}", cloned_string)`.

Primary seam: `src/app/input_state.rs`, `src/app/mod.rs`, `src/ui/mod.rs`.

### 4. Stop cloning `current_typed_input` on every edit

Reduce or eliminate whole-input cloning during ordinary edits while preserving history-navigation behavior.

Likely direction:
- update the saved typed-input snapshot only when history navigation needs it,
- or otherwise narrow when `current_typed_input` must diverge from `displayed_input`.

Primary seam: `src/app/input_state.rs`.

### 5. Cache cursor display offset

Avoid recalculating grapheme-based cursor display offset on every draw when the cursor/input state has not changed.

Likely direction:
- maintain cached display-column offset alongside cursor state,
- update it only on edits and cursor movement.

Primary seam: `src/app/input_state.rs`.

## Acceptance Checks

- In a release build, typing remains visually correct and no worse than before for:
  - short ASCII commands,
  - longer Unicode-containing commands,
  - history navigation,
  - mid-line insert/delete,
  - paste handling.
- Profiling or direct timing shows a measurable reduction in per-keystroke work in at least one of these hot paths:
  - `terminal.draw(...)`,
  - `self.output.wrapped_lines(...)`,
  - input render-string construction,
  - `sync_current_typed_input()` / input edit path,
  - `cursor_offset(false)` / cursor math.
- Output, scrollback offset behavior, dialog behavior, and command submission semantics remain unchanged.
- Cargo gates pass:
  - `cargo fmt --all`
  - `cargo build --all-targets`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`

## Validation Approach

Before implementation, verify the relative cost of the current seams with targeted timing or system profiling in a release build.

Suggested confirmation order:
1. compare typing when idle vs under heavy inbound output,
2. compare typing with short vs large output history,
3. add temporary timing around `terminal.draw(...)`, `wrapped_lines(...)`, input edit paths, and `process_input_lines(...)`,
4. if needed, sample the release binary with `perf`.

## Rollout / Follow-Up Notes

- The five changes do not need to land as one diff; they can be split into reviewable slices as long as intermediate behavior stays correct.
- Preferred implementation order is the laziest likely win first:
  1. redraw invalidation,
  2. wrapped-output cache,
  3. render-path input clone removal,
  4. edit-path input clone removal,
  5. cached cursor display offset.
- If these changes do not materially improve responsiveness, a later follow-up can investigate bounded or segmented scrollback storage, but that is explicitly out of scope for this PRD.

## Open Questions

- Should the clock remain truly time-driven when no other UI state changed, or is minute-level redraw acceptable?
- Should wrapped-output caching live inside `OutputBuffer` or in the app/render layer that already knows the active width?
- Does the team want this work delivered as one feature branch or as several small slices in the order above?

## References

- `CONTEXT.md`
- `docs/adr/0004-repo-local-markdown-files-are-the-canonical-planning-tracker.md`
- `docs/wiki/source-notes/text-input-slow-profiling.md`
- `src/main.rs`
- `src/app/mod.rs`
- `src/app/output_buffer.rs`
- `src/app/input_state.rs`
- `src/app/scrollback.rs`
- `src/ui/mod.rs`
