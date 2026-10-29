# Slice 01: Render only the selected output viewport

## Status

done

Completed on 2026-10-29 in commit `16b0fe0`. All acceptance criteria passed risk-first and standards/spec review. Format, build, warning-clean Clippy, 796 automated tests, wiki lint, and strict docs build passed; the existing manual `backfill_user_damage_db` test remains ignored. The user confirmed the game still works. The implementation handoff is recorded in slice 02; incremental append wrapping remains pending.

## Suggested branch name

`feature/output-history-performance`

## Parent

[Output History Performance PRD](prd.md)

## Type

AFK

## User stories covered

2–7, 9–23, 27–33 from the parent PRD. This slice delivers the rendering and scrolling portion; the final append/update performance target depends on slice 02.

## What to build

Make cached redraws of the batrs client prepare and render only the selected screenful of output, regardless of retained history size. Deliver the complete path from `Scrollback` position through the batrs client view preparation to the existing paragraph renderer. Copying viewport rows is acceptable; copying all history is not. Do not apply paragraph scrolling a second time after selecting the viewport.

Use clamped `usize` wrapped-row offsets so histories exceeding 65,535 wrapped rows can follow the actual latest output and display earlier selected rows. Keep viewport dimensions consistent with the renderer's actual output area, accounting for combat rows, primary status/clock, Secondary Status, and input. Reuse a minimal shared layout calculation only if needed to avoid disagreement.

Preserve text and styles, wrapping rules, history retention, PageUp/PageDown, follow-latest behavior, command-triggered following, stable selected rows during append, dialogs, cursor placement, and Session Lifecycle history disposition. Preserve existing wrapped-row/clamping resize semantics rather than introducing source-text anchoring.

Include regression checks, real release benchmark measurements, and aligned documentation. Keep the existing full-history wrapping behavior for appended output in this slice; slice 02 removes that separate cost. No new dependencies, custom widget, workers, or unrelated input/trigger changes.

## Acceptance criteria

- [x] View preparation and paragraph rendering copy or traverse only the selected viewport rows for presentation, not hidden history; no second scroll offset is applied to the selected rows.
- [x] Cached redraw cost is bounded by viewport size rather than retained history, with no material history-size trend in repeated isolated release measurements at 100, 1,000, and 10,000 initial lines.
- [x] Wrapped-row offsets are not truncated to `u16`; a regression with more than 65,535 wrapped rows and distinguishable content proves both latest and selected earlier output are reachable. Use a small-content fixture to limit test memory.
- [x] Actual screen-content tests cover latest and scrolled-back positions, PageUp/PageDown, incoming output while scrolled back, returning to follow-latest, and command-triggered following.
- [x] Range selection is safe and correct for empty output, insufficient history, zero width, zero visible height, and tiny terminals; selected rows match the actual output area as HUD row counts and terminal geometry change.
- [x] Text, ANSI styles, blank lines, Unicode content, and existing width-wrap behavior remain unchanged; height changes adjust visible rows without unnecessary wrapping.
- [x] Dialog overlays, combat and Secondary Status placement, input placement, and cursor behavior remain correct in app/renderer `TestBackend` checks.
- [x] Same-character reconnect preserves history and scrollback; different-character and connect-before-login clearing remain correct; `/clear` remains a terminal redraw rather than a history wipe.
- [x] All affected view-model constructions, renderer call sites, and tests use the revised viewport/offset contract; no legacy full-history presentation path remains.
- [x] Keep `cargo test --release benchmark_output_history -- --nocapture --test-threads=1` runnable. Update its isolated copy stage to measure the real viewport preparation, preserve initial history sizes and the fixed 80×24 geometry, and record cached-draw and append-and-draw results after this slice. Do not assert machine-dependent timings in ordinary CI.
- [x] Benchmark evidence and client subsystem documentation describe viewport-limited rendering and the still-existing full-history append wrapping cost. Reconcile affected older profiling/PRD claims without marking unrelated work complete; update the UI manual for corrected long-history scrolling.
- [x] Format → build/typecheck → warning-clean Clippy → complete automated tests pass, followed by wiki lint and strict docs build for documentation changes. Carry regression and performance evidence into risk-first and standards/spec review; report any existing manual-test skips.

## Blocked by

None - can start immediately.
