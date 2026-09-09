# Slice 01: Redraw invalidation and wrapped-output cache

## Parent

- `docs/features/text-input-performance/prd.md`

## Goal

Cut the largest likely source of input latency by preventing unnecessary full-frame redraws and by avoiding full output re-wrapping on every draw when neither output nor width changed.

## Why this slice first

The current code redraws every main-loop iteration in `src/main.rs`, and `BatApp::draw` rebuilds wrapped output lines from the full output buffer each frame through `OutputBuffer::wrapped_lines`. That makes typing pay for full-screen work even when only the input changed. This slice targets the highest-value hot path first. Sources: `src/main.rs`, `src/app/mod.rs`, `src/app/output_buffer.rs`.

## Scope

- Add a redraw invalidation mechanism so `terminal.draw(...)` only runs when needed.
- Define and wire the initial invalidators for:
  - input edits and submit,
  - paste,
  - scrollback movement,
  - output append/clear,
  - dialog open/close/update,
  - reconnect/session transitions,
  - stats/secondary-status changes,
  - time/clock updates if the clock display still requires them.
- Add wrapped-output caching keyed by output state and terminal width.
- Keep cache invalidation correct when output is appended, cleared, or the terminal width changes.
- Preserve existing scrollback behavior and visible rendering.

## Likely Seams

- `src/main.rs`
- `src/app/mod.rs`
- `src/app/output_buffer.rs`
- possibly `src/ui/mod.rs` if minor call-site changes are needed

## Non-Goals

- No input-state cloning changes yet.
- No cursor-offset caching yet.
- No scrollback storage redesign.
- No new dependencies.

## Dependencies

- none

## Acceptance Checks

- The app does not redraw continuously when no visible state changed.
- Typing, paging scrollback, and incoming-output rendering still update the screen correctly.
- Wrapped output is not recomputed on every draw when the output buffer and width are unchanged.
- Scrollback offset semantics remain unchanged.
- Cargo gates pass:
  - `cargo fmt --all`
  - `cargo build --all-targets`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`

## Notes

- Keep this slice independently buildable and reviewable.
- Prefer the smallest ownership model that works; a generation counter plus width key is enough unless profiling proves otherwise.
