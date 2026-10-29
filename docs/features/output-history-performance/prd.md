# Output History Performance PRD

## Status

accepted

The resolved plan review is the basis of this spec; the user directed the planning lane to `to-spec`. Slice 01 implementation now selects the visible viewport with clamped usize row offsets. Slice 02 incremental append wrapping remains pending; this does not mark either ticket complete.

## Triage label

`ready-for-agent`

## Suggested branch name

`feature/output-history-performance`

## Problem Statement

Original baseline problem: the batrs client becomes less responsive during long play sessions as game output accumulates. New displayed lines cause the entire retained output history to be wrapped again, and drawing copies the complete wrapped history even when only a screenful is visible. Typing and other UI updates also pay the copying cost when wrapping is already cached. Slice 01 now eliminates hidden-history presentation copies; append wrapping is still full-history.

An offline release benchmark with a fixed 80×24 terminal measured the following median costs:

| Initial retained lines | Re-wrap after append | Draw with valid wrapping cache | Process one new line and draw |
| ---: | ---: | ---: | ---: |
| 100 | 0.308 ms | 0.644 ms | 1.018 ms |
| 1,000 | 2.916 ms | 5.725 ms | 9.408 ms |
| 10,000 | 37.441 ms | 62.836 ms | 101.132 ms |

Growing history 100-fold increased append-and-draw cost approximately 99-fold. Repeated release invocations put the 10,000-line update at approximately 98–109 ms. This demonstrates a local output-history scaling problem; it does not establish every contributor to a particular live-session slowdown. The benchmark excludes live networking, logged-in triggers, persistence, and the real terminal backend.

At specification time, the dirty-redraw mechanism and wrapped-output cache avoided some unnecessary work but did not prevent a full re-wrap after append or full-history copies on redraw. After slice 01, cached redraws take 0.163–0.226 ms across 100/1,000/10,000 initial lines in three isolated release invocations. Append-and-draw at 10,000 lines still takes 37.170–38.344 ms, short of the combined 10× target; see the linked benchmark evidence. Game triggers process incoming lines, not the entire stored history; this feature addresses output preparation and presentation.

## Solution

Keep normal gameplay responsive as retained output grows by rendering only the selected screenful and wrapping only newly appended output at an unchanged width. Preserve output text, styles, history retention, paging, input, status rows, dialogs, and reconnect behavior.

Deliver two cohesive implementation slices in order: viewport-limited rendering, then incremental wrapping. Both use existing owner modules and the existing ratatui paragraph renderer, without new dependencies or a custom rendering subsystem.

Normal append/update work should be proportional to newly retained content plus the visible viewport, rather than total history. Cached redraw work should be proportional to the viewport. Initial wrapping, terminal-width changes, and occasional cache allocation growth remain explicit history-dependent costs.

## User Stories

1. As a BatMUD player, I want new game output to appear promptly after long sessions, so that accumulated history does not interrupt play.
2. As a BatMUD player, I want typing short commands to remain responsive with large history, so that editing input does not repeatedly process old output.
3. As a BatMUD player, I want redraws of unchanged output to be inexpensive, so that clock and UI updates do not introduce unnecessary lag.
4. As a BatMUD player, I want all existing scrollback retained, so that performance improvements do not discard earlier game text.
5. As a BatMUD player, I want output text and ANSI styles preserved, so that combat and other highlights remain readable.
6. As a BatMUD player, I want long lines to wrap as before, so that optimization does not change the presentation rules.
7. As a BatMUD player, I want blank output lines preserved, so that transcript spacing remains unchanged.
8. As a BatMUD player, I want gagged lines to remain hidden, so that internal probes and suppressed messages do not appear in scrollback.
9. As a BatMUD player, I want PageUp to display earlier output correctly, so that I can review events during play.
10. As a BatMUD player, I want PageDown to return to follow-latest mode at the bottom, so that normal output following resumes predictably.
11. As a BatMUD player, I want incoming output not to move my selected scrollback position, so that I can keep reading earlier events.
12. As a BatMUD player, I want new output followed automatically when already at the bottom, so that I see the latest events without manual paging.
13. As a BatMUD player, I want command-triggered follow-latest behavior preserved, so that command submission behaves consistently.
14. As a BatMUD player, I want sessions exceeding 65,535 wrapped rows to show the actual latest output, so that long history does not strand the viewport at an old offset.
15. As a BatMUD player, I want terminal-width changes to re-wrap output correctly, so that resizing does not show stale line breaks.
16. As a BatMUD player, I want terminal-height changes to adjust the visible screenful without unnecessary wrapping, so that ordinary resizing stays efficient.
17. As a BatMUD player, I want tiny terminals handled safely, so that limited screen space does not cause a panic or invalid viewport selection.
18. As a BatMUD player, I want combat and Secondary Status rows to keep their placement, so that output optimization does not hide important HUD information.
19. As a BatMUD player, I want dialogs and the input cursor to behave as before, so that faster output rendering does not disrupt interaction.
20. As a BatMUD player, I want reconnecting as the same character to preserve scrollback, so that temporary connection loss does not erase session history.
21. As a BatMUD player, I want different-character reconnects to clear old output as before, so that session history is not mixed between characters.
22. As a BatMUD player, I want `/clear` to remain a terminal redraw rather than a history wipe, so that its established semantics are preserved.
23. As a maintainer, I want rendering work bounded by the selected viewport, so that drawing does not clone hidden history.
24. As a maintainer, I want appended output wrapped incrementally, so that a small update does not rebuild old presentation data.
25. As a maintainer, I want repeated appends before a draw handled together, so that incoming batches do not require redundant cache rebuilds.
26. As a maintainer, I want clear-and-refill to invalidate cached output correctly, so that equal line counts cannot expose stale text.
27. As a maintainer, I want zero-width rendering to preserve pending output, so that temporarily unavailable screen space does not lose lines.
28. As a maintainer, I want existing ownership boundaries retained, so that local performance work does not redesign Command Dispatch or Session Lifecycle.
29. As a maintainer, I want no new dependencies or runtime workers, so that the optimization stays small and understandable.
30. As a maintainer, I want regression tests at the real output and drawing seams, so that correct-looking helper tests do not hide broken scrolling or presentation.
31. As a maintainer, I want repeatable before/after release measurements, so that performance improvement is demonstrated rather than assumed.
32. As a reviewer, I want independently buildable implementation slices, so that viewport rendering and wrapping changes can be reviewed separately.
33. As a future maintainer, I want remaining memory and resize limits documented, so that this optimization is not mistaken for bounded-memory history storage.

## Implementation Decisions

### Delivery and ownership

- Implement viewport-limited rendering first, then incremental wrapping. Each slice must preserve behavior, pass validation, and retain runnable performance checks.
- Reuse the existing deep modules rather than extracting new wrappers: `OutputBuffer` hides storage and cache mechanics; `Scrollback` hides position and follow-latest state; the batrs client draw/view boundary coordinates them; `Renderer` handles presentation.
- Command Dispatch, Session Lifecycle, Player Profile, Guild Catalog, Combat Damage, triggers, and the network event loop retain their current ownership and behavior.
- No schema, persistent configuration, dependency, or remote-system changes are required.

### Viewport-limited rendering

- Select only the visible wrapped-row range before constructing the output view data. Pass the selected rows to the existing paragraph renderer without applying a second scroll offset.
- Copying visible rows is acceptable. Eliminate full-history copies rather than pursuing zero-copy presentation or a custom widget.
- Keep history positions and viewport range arithmetic as `usize`; remove the existing 65,535-row truncation associated with the paragraph's `u16` scrolling interface. Terminal geometry retains its existing native types.
- `Scrollback` continues to track total wrapped-row count, visible height, selected top row, and follow-latest state. Its offset contract returns a clamped wrapped-row index suitable for selecting a slice, not a terminal-widget scroll argument.
- Compute viewport dimensions consistently with the renderer's actual output area, including combat rows, primary status/clock, Secondary Status, and input. Reuse a minimal shared layout calculation only if needed to avoid disagreement between selection and rendering.
- Clamp range bounds to available rows and handle empty output, zero width, zero visible height, and tiny terminals without invalid slicing or subtraction.
- Preserve PageUp/PageDown, command-triggered follow-latest, automatic following at the bottom, and stable selected wrapped-row positions while new output arrives.
- On width or height changes, preserve current wrapped-row/clamping semantics. Do not introduce source-text anchoring across resize.
- Keep dialogs, HUD layout, input placement, cursor behavior, and text/style rendering unchanged, except for correcting long-history offset truncation.

### Incremental wrapping

- Retain the existing cached wrapped-row interface and owned cache storage. Track the source-line count already incorporated at the cached terminal width.
- At an unchanged width, wrap only retained source lines after that count and extend the wrapped-row cache in order. After all pending additions are incorporated, repeated requests return the cache without wrapping work.
- The source count replaces generation-based full rebuilding for ordinary append. Old source lines are immutable under the current append/clear contract; no editing or arbitrary removal API is introduced.
- Initial cache population and terminal-width changes wrap all retained source lines. A height-only change does not invalidate wrapping.
- Output clear resets cached rows and source progress, including clear followed by refill to the same source-line count before the next draw.
- Filter gagged additions as currently required. Empty and fully gagged batches do not add wrapped rows or force unrelated wrapping.
- Multiple appends before a draw accumulate as one pending suffix. Zero-width requests return no rows without discarding retained lines or incorrectly advancing source progress.
- Preserve the existing wrapping algorithm, blank-line handling, Unicode behavior, styles, and row order. Do not combine this change with grapheme/span or gag-filter cleanup.
- Cache extension uses ordinary vector allocation behavior. Work proportional to new content and viewport size is amortized: occasional allocation growth may still move existing cache entries.

### Lifecycle and limits

- Preserve same-character reconnect history and scrollback, different-character or connect-before-login output clearing, and fresh-session reset responsibilities.
- `/clear` remains a terminal redraw, not output-buffer clearing.
- Retain all history. Raw and wrapped history memory still grow with session output; this feature does not promise bounded memory.
- Full-history width-change wrapping remains intentional. Document that ceiling and investigate more complex storage only if subsequent measurement justifies it.

## Testing Decisions

- Good tests exercise output content, style, order, selected viewport, paging, follow-latest, and lifecycle behavior through existing interfaces. Do not freeze cache field names, allocation strategy, or helper decomposition.
- Cover every affected module at the correct seam: output-buffer tests for wrapping/cache behavior, scrollback tests for range and following behavior, and app/renderer tests with ratatui's in-memory `TestBackend` for actual screen content and layout.
- Reuse existing fake-app construction without production configuration, loggers, damage storage, or network connections. No user-local data or live BatMUD connection is required for automated checks.
- Prior art includes output-cache and resize tests, scrollback paging/following tests, app drawing and cursor tests, and same-/different-character reconnect tests.
- Replace cache pointer-inequality assertions with output correctness assertions. Incremental extension can legitimately retain its allocation, so allocator movement is not a behavioral contract.
- Cover append after cache population, multiple append batches before drawing, unchanged draws, mixed gagged/retained additions, fully gagged and empty batches, blank lines, styled and Unicode lines, clear/refill to equal counts, width changes in both directions, height-only changes, and zero-width recovery.
- Cover actual visible content at the latest and scrolled-back positions, append while scrolled back, return to follow-latest, command-triggered following, insufficient history to fill a viewport, zero/tiny viewport geometry, status-row count changes, dialogs, and input cursor placement.
- Add a long-history regression using more than 65,535 wrapped rows and distinguishable text to prove latest and selected rows remain reachable without truncation. Use small source content so this check does not require a huge styled-history fixture.
- Preserve and run reconnect and terminal-redraw regressions; same-character preservation and different-character clearing must still work with the new cache and viewport contracts.
- Keep the existing release benchmark runnable with the same history sizes, synthetic input, fixed terminal dimensions, and real append/draw calls:

```bash
cargo test --release benchmark_output_history -- --nocapture --test-threads=1
```

- Measure after each slice. Update the benchmark's isolated copy stage to exercise the new viewport preparation, not replay obsolete full-history copying. Label the wrapping stage as append-cache update once it becomes incremental, and retain recorded baseline results for comparison.
- Performance acceptance: on the same machine in repeated isolated release runs, the 10,000-line median append-and-draw time is at least ten times faster than the approximately 101 ms baseline. Fixed-viewport cached redraw and ordinary append/update costs should show no material retained-history trend across 100, 1,000, and 10,000 lines; distinguish occasional allocation growth from sustained full-history work.
- Keep machine-dependent timing thresholds out of ordinary CI assertions. Report actual timings, sample counts, build mode, and any concurrent work; do not infer live-session timing from the synthetic result.
- If history-dependent cost remains, investigate the measured remainder at the same seams before widening scope.
- Run project gates in order: formatting, build/typecheck, Clippy with warnings denied, and the complete automated test suite. Follow with mechanical wiki lint, strict documentation build for documentation changes, and risk-first plus standards/spec review.
- Live-session responsiveness is a user-run follow-up, not an agent-run network validation. Ask for targeted observations only if residual behavior requires confirmation.

## Out of Scope

- Capping, deleting, persisting, compressing, or otherwise redesigning scrollback storage.
- Bounded-memory guarantees or eliminating full-history work on terminal-width changes.
- Custom terminal widgets, new UI libraries, background wrapping workers, dirty-redraw redesign, or asynchronous rendering.
- New runtime or profiling dependencies.
- Changes to incoming-line trigger processing, automation, input editing, Unicode wrapping rules, gag-filter algorithms, or span representation.
- Command Dispatch, Player Profile, Guild Catalog, Combat Damage schema/attribution, and Session Lifecycle redesign.
- New source-text scroll anchoring during resize or unrelated UI behavior changes.
- Reading or changing user-local configuration, logs, credentials, or damage databases.
- Claiming that the offline benchmark diagnoses every source of live-session slowdown.

## Further Notes

- Benchmark evidence and limitations are recorded in the [output-history scaling source note](../../wiki/source-notes/output-history-benchmark.md).
- The [client subsystem page](../../wiki/subsystems/batrs-client.md) describes current dirty redraws and cached wrapping. The [older profiling note](../../wiki/source-notes/text-input-slow-profiling.md) contains historical loop/render claims and should be reconciled during implementation.
- This spec narrows the history-scaling work overlapping the [text input performance PRD](../text-input-performance/prd.md) and [runtime hot path optimizations PRD](../runtime-hot-path-optimizations/prd.md). It does not supersede their unrelated input or trigger work. Reconcile affected descriptions when implementation changes current behavior; do not mark unrelated work completed.
- Update benchmark evidence, client subsystem knowledge, and the UI manual's corrected long-history scrolling behavior with the implementation. No new glossary terms or ADR are required: ownership remains unchanged and the implementation choices are local and reversible.
- Slice 01 applies viewport-limited presentation only; the original offline baseline and slice 01 measurements remain in the benchmark evidence. Slice 02 still needs to remove full-history wrapping after ordinary append.
- Next planning step: `to-tickets`, producing viewport-limited rendering and incremental-wrapping slices under this feature, both sharing the suggested feature branch.
