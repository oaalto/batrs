# Slice 03: Cursor display-offset cache

## Parent

- `docs/features/text-input-performance/prd.md`

## Blocked by

- `docs/features/text-input-performance/02-input-render-and-edit-copy-reduction.md`

## Goal

Avoid repeated grapheme-count work on every draw by caching cursor display offset and updating it only when input or cursor state actually changes.

## Why this slice third

`InputState::cursor_offset(false)` currently counts graphemes in the prefix on demand. That is correct but repeated draw-time work. It is a smaller likely win than redraw invalidation and copy reduction, so it follows those slices. Source: `src/app/input_state.rs`.

## Scope

- Introduce a cached cursor display offset in `InputState` or an equally local seam.
- Update the cached value on all operations that can affect cursor position or displayed width, including:
  - insert char/string,
  - backspace/delete,
  - left/right movement,
  - word movement,
  - home/end,
  - history navigation,
  - clear/take/reset paths,
  - hidden-input cases as needed.
- Preserve current Unicode/grapheme correctness.
- Add or update focused unit tests for cursor behavior.

## Likely Seams

- `src/app/input_state.rs`
- `src/app/mod.rs` only if call sites need minor adjustment

## Non-Goals

- No redraw-loop changes.
- No output-buffer caching changes.
- No change to scrollback semantics.

## Dependencies

- `docs/features/text-input-performance/02-input-render-and-edit-copy-reduction.md`

## Acceptance Checks

- The common draw path no longer recomputes cursor display offset by scanning graphemes when input state is unchanged.
- Cursor placement remains correct for ASCII and Unicode-containing input.
- Existing cursor-movement and edit tests still pass, with added coverage where needed.
- Cargo gates pass:
  - `cargo fmt --all`
  - `cargo build --all-targets`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`

## Notes

- Keep the cache local and boring.
- If maintaining the cache proves too error-prone, stop and reassess rather than spreading cursor math across multiple modules.
