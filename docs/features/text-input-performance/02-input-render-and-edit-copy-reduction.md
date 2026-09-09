# Slice 02: Input render and edit copy reduction

## Parent

- `docs/features/text-input-performance/prd.md`

## Blocked by

- `docs/features/text-input-performance/01-redraw-invalidation-and-output-cache.md`

## Goal

Reduce per-keystroke work inside `InputState` and the input render path by removing unnecessary whole-string clones while preserving current history-navigation and Unicode behavior.

## Why this slice second

After redraw/wrapping work is reduced, the next likely avoidable cost is repeated cloning in `InputState` and input rendering. The current implementation clones input during ordinary edits and again during rendering. Sources: `src/app/input_state.rs`, `src/app/mod.rs`, `src/ui/mod.rs`.

## Scope

- Change visible input rendering so the app does not clone the full displayed input string every frame just to build the prompt line.
- Narrow or eliminate `current_typed_input` cloning on every edit while preserving history-navigation behavior.
- Keep existing semantics for:
  - normal typing,
  - mid-line insert/delete,
  - history up/down,
  - submit/reset behavior,
  - hidden password input,
  - paste handling.
- Add or update focused tests for any changed `InputState` behavior.

## Likely Seams

- `src/app/input_state.rs`
- `src/app/mod.rs`
- `src/ui/mod.rs`

## Non-Goals

- No wrapped-output caching work; that belongs to Slice 01.
- No cursor display-offset caching yet.
- No behavior change to Unicode/grapheme correctness.

## Dependencies

- `docs/features/text-input-performance/01-redraw-invalidation-and-output-cache.md`

## Acceptance Checks

- Rendering the input line no longer requires cloning the full visible input string per frame in the common visible-input path.
- Ordinary edits no longer clone the full input into `current_typed_input` on every keystroke unless required for preserved semantics.
- Existing input behavior remains correct for history navigation, hidden input, and paste.
- Unit tests cover the preserved input semantics touched by the refactor.
- Cargo gates pass:
  - `cargo fmt --all`
  - `cargo build --all-targets`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`

## Notes

- Prefer deleting redundant state transitions over adding new abstraction.
- If a tiny helper or cached field is needed, keep it local to `InputState`.
