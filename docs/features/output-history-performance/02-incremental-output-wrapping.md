# Slice 02: Wrap appended output incrementally

## Suggested branch name

`feature/output-history-performance`

## Parent

[Output History Performance PRD](prd.md)

## Type

AFK

## Triage label

`ready-for-agent`

## User stories covered

1, 4–8, 11–12, 15–16, 20–21, 24–33 from the parent PRD. Combined with slice 01, this completes the parent output-history performance target.

## What to build

Make the real incoming-output append-and-draw path wrap only newly retained source lines when terminal width is unchanged, then render through the viewport-limited path from slice 01. Keep `OutputBuffer` responsible for storage and owned wrapping-cache mechanics, with the existing cached wrapped-row interface.

Record the source-line count already incorporated at the cached width. Extend the cache from the pending suffix instead of using output-generation changes to rebuild all history on ordinary append. Multiple batches before drawing accumulate as one pending suffix; repeated draws without additions do no wrapping work. Existing source lines remain immutable under the append/clear contract.

Populate the whole cache initially and rebuild it when width changes. Output clear resets cached rows and source progress, even if output is refilled to the same source count before drawing. Height-only changes, empty batches, and fully gagged batches do not require wrapping old output. A zero-width request returns no rows without losing pending content or advancing source progress incorrectly.

Preserve ordering, gagging, blank lines, Unicode and ANSI styling, existing wrapping rules, scrollback following/selection, and Session Lifecycle history disposition. Ordinary extension may occasionally move existing vector entries as capacity grows; the steady-state complexity guarantee is amortized, not an allocation-free guarantee.

Include output and screen-content regressions, final before/after release measurements, and documentation of the measured improvement and remaining limits. Retain all history; memory still grows with history and terminal-width changes still perform full-history wrapping. No storage redesign, new dependencies, workers, span/grapheme cleanup, or gag-filter refactor.

## Acceptance criteria

- [ ] At unchanged width, cache updates incorporate only source lines appended after the last completed update; subsequent unchanged requests return cached rows without wrapping old content.
- [ ] Multiple appends before a draw produce every retained wrapped row exactly once and in order, with no omitted or duplicated output.
- [ ] Initial population and terminal-width changes produce correct full-history wrapping in both resize directions; height-only changes do not invalidate wrapping.
- [ ] Clear empties cached rows and resets source progress; tests cover clear/refill to the same count before a draw, different counts, and clear followed by an empty draw.
- [ ] Empty and fully gagged batches do not add rows or force old wrapping; mixed batches preserve retained order, text, styles, blank lines, and Unicode behavior.
- [ ] Zero-width requests leave pending output intact; restoring a nonzero width produces correct content without stale rows or duplication, including when pending appends and clear occur between draws.
- [ ] App-level `TestBackend` checks exercise real append-and-draw behavior while following latest and while scrolled back; viewport content and selected row positions remain correct.
- [ ] Existing reconnect preservation/clearing and terminal-redraw regressions pass with the new cache contract. Keep the greater-than-65,535-row viewport regression from slice 01 passing.
- [ ] Replace existing cache pointer-inequality assertions with output correctness checks. Tests cover contracts and real rendering, not allocation movement, cache field names, or container choices.
- [ ] Keep `cargo test --release benchmark_output_history -- --nocapture --test-threads=1` runnable with the baseline's synthetic input, 100/1,000/10,000 initial histories, and fixed 80×24 geometry. Label the wrapping stage as append-cache update, retain viewport preparation measurements, and preserve the recorded original baseline.
- [ ] In repeated isolated release runs on the same machine, the 10,000-line median append-and-draw time is at least 10× faster than the approximately 101 ms baseline (approximately 10.1 ms or less). Cached redraw and ordinary append/update costs show no material retained-history trend; report occasional allocation growth separately from sustained full-history work. Keep timing thresholds out of ordinary CI assertions.
- [ ] If the benchmark still shows sustained history-dependent work, investigate the measured remainder at the existing seams before widening scope or declaring the target met.
- [ ] Update benchmark evidence and the client subsystem documentation with before/after timings, methodology, and limitations. Reconcile historical profiling/PRD statements about full-history re-wrapping; retain unrelated plans. Explicitly document growing history memory and full-history work on width changes.
- [ ] Format → build/typecheck → warning-clean Clippy → complete automated tests pass, followed by wiki lint and strict docs build for documentation changes. Carry regression and performance evidence into risk-first and standards/spec review; report any existing manual-test skips.
- [ ] No live BatMUD connection, user-local config/log/database access, or runtime session is needed for automated acceptance. Offline success is not reported as proof of every live-session performance contributor.

## Implementation handoff

**Implementation knowledge from slice 01**

- **Contracts/seams:** `Scrollback::offset()` now returns `usize`; `visible_range()` selects clamped wrapped rows. Breaking view-model contract change: `ViewModel` no longer accepts a scroll offset. Slice 02 depends on this viewport-only presentation contract; rows are already selected, so do not scroll the paragraph again.
- **Decisions:** No plan divergence or ADR candidate. Full-history append wrapping remains intentionally unchanged.
- **Traps:** Use `Renderer::layout` for exact output dimensions, including tiny terminals. Preserve the 70,000-row regression, height-only cache reuse, and cached reconnect checks.
- **File/API map:** `src/app/mod.rs::prepare_output`, `src/app/scrollback.rs`, and `src/ui/mod.rs`. Keep the benchmark's real viewport-preparation stage. At 10,000 initial lines, append-and-draw still costs 37.170–38.344 ms in three isolated release runs; slice 02 must address that remainder. See the [benchmark evidence](../../wiki/source-notes/output-history-benchmark.md).

## Blocked by

- [Slice 01: Render only the selected output viewport](01-viewport-limited-rendering.md)
