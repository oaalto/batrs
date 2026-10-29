---
title: Output history scaling benchmark
type: source-note
status: current
updated: 2026-10-29
sources:
  - src/app/mod.rs
  - src/app/output_buffer.rs
  - src/ansi/styled_line.rs
  - src/ui/mod.rs
---

# Output history scaling benchmark

## Summary

Viewport-limited rendering and incremental append wrapping remove sustained retained-history work from ordinary append/draw and cached redraws at a fixed 80×24 viewport. At 10,000 initial lines, final append-and-draw medians are 0.163–0.168 ms versus the original 101.132 ms. Initial population and width changes still wrap all history, memory still grows, and vector growth can occasionally move old rows. Offline measurements establish local scaling behavior, not the cause of every live-session slowdown.

## Verified Facts

Run the real output buffer and `BatApp::draw` through ratatui's in-memory `TestBackend`:

```bash
cargo test --release benchmark_output_history -- --nocapture --test-threads=1
```

The test uses the existing fake app with no config, loggers, damage database, or network. History consists of copies of a short ANSI-colored ASCII combat line. Setup and initial cache population are excluded. Each result is the median of five samples; history grows by two lines per sample. Assertions verify wrapped row counts and the fixed viewport. Timings are reported, not asserted, to avoid machine-dependent test failures. The final benchmark also prints the largest append-cache update among the five samples and runs a separate 10,000-single-line extension probe starting from 10,000 lines. The probe times cache updates only (source append/setup excluded), reports first-/second-half medians and the slowest update, and checks final content length; it does not inspect cache fields or assert allocator behavior.

### Original baseline

One measured run on 2026-10-29, before viewport-limited rendering, in milliseconds:

| Initial history | Cache lookup | Re-wrap after append | Two history copies¹ | Cached draw | Process one new line + draw |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 0.000027 | 0.308 | 0.385 | 0.644 | 1.018 |
| 1,000 | 0.000019 | 2.916 | 3.632 | 5.725 | 9.408 |
| 10,000 | 0.000038 | 37.441 | 44.146 | 62.836 | 101.132 |

¹ Replays the view-model `to_vec()` and renderer's `clone()` into `Text`/`Paragraph`, including destruction. Other columns overlap these operations; do not add all columns together.

Repeated release runs retained the scaling pattern; the 10,000-line append-and-draw median ranged from approximately 98 to 109 ms across three invocations. One repeat ran concurrently with the full test suite and is not an isolated timing baseline.

### Slice 01: viewport-limited rendering

Three sequential isolated release invocations on 2026-10-29 (five samples per history per invocation), with no concurrent agent tests or builds:

| Run | Initial history | Re-wrap after append | Viewport preparation¹ | Cached draw | Process one new line + draw |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 100 | 0.284 | 0.110 | 0.163 | 0.495 |
| 1 | 1,000 | 3.240 | 0.512 | 0.191 | 3.649 |
| 1 | 10,000 | 37.367 | 0.081 | 0.186 | 37.170 |
| 2 | 100 | 0.402 | 0.132 | 0.226 | 0.791 |
| 2 | 1,000 | 2.884 | 0.471 | 0.167 | 3.488 |
| 2 | 10,000 | 36.908 | 0.078 | 0.178 | 38.344 |
| 3 | 100 | 0.327 | 0.118 | 0.174 | 0.578 |
| 3 | 1,000 | 2.902 | 0.506 | 0.169 | 3.685 |
| 3 | 10,000 | 36.908 | 0.079 | 0.180 | 37.681 |

¹ Measures the actual shared layout and `BatApp::prepare_output`, plus the renderer's viewport-only clone into `Text`/`Paragraph` and destruction. This replaces the obsolete two-full-history-copy stage. Preparation is measured just after a full re-wrap; its non-monotonic medians should not be added to other overlapping stages or interpreted as a sustained history-size trend.

- At the slice 01 checkpoint, `OutputBuffer::append_lines` still changed the generation; `wrapped_lines` rebuilds every retained line when generation or width differs (`src/app/output_buffer.rs`). Slice 02 addresses ordinary append wrapping separately.
- `BatApp::prepare_output` updates `Scrollback` with actual output dimensions and copies only `visible_range()`. `Renderer::render` receives only those rows and does not apply paragraph scrolling (`src/app/mod.rs`, `src/app/scrollback.rs`, `src/ui/mod.rs`).
- `Renderer::layout` supplies identical output/HUD/input geometry to preparation and rendering, including tiny-terminal constraint handling.
- `Scrollback::offset` and its clamped range use `usize`. The 70,000-row small-content screen regression reaches both latest and earlier distinguishable rows beyond 65,535.
- Cache hits and height-only changes avoid wrapping; width changes and appended output still wrap full history. All history is retained, so source and wrapped cache memory still grow with output.

### Slice 02: incremental append wrapping

A pre-change release invocation in the same checkout on 2026-10-29 reproduced slice 01: at 10,000 lines, re-wrap was 36.118 ms, cached draw 0.173 ms, and append-and-draw 37.864 ms. The original baseline above is preserved unchanged.

Three final sequential isolated release invocations on the same machine on 2026-10-29, with five samples per history and no concurrent agent builds, tests, or review agents (milliseconds):

| Run | Initial history | Append-cache update | Viewport preparation | Cached draw | Process one new line + draw | Largest sampled cache update |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 100 | 0.002568 | 0.073 | 0.167 | 0.170 | 0.005239 |
| 1 | 1,000 | 0.002269 | 0.072 | 0.180 | 0.173 | 0.002432 |
| 1 | 10,000 | 0.002505 | 0.071 | 0.159 | 0.166 | 0.003422 |
| 2 | 100 | 0.002223 | 0.066 | 0.154 | 0.162 | 0.002788 |
| 2 | 1,000 | 0.002101 | 0.069 | 0.157 | 0.164 | 0.002458 |
| 2 | 10,000 | 0.002183 | 0.072 | 0.158 | 0.168 | 0.002707 |
| 3 | 100 | 0.004129 | 0.101 | 0.209 | 0.199 | 0.006541 |
| 3 | 1,000 | 0.002126 | 0.068 | 0.159 | 0.165 | 0.002267 |
| 3 | 10,000 | 0.002146 | 0.068 | 0.152 | 0.163 | 0.002583 |

The fixed history sizes, ANSI combat input, 80×24 geometry, five-sample medians, real viewport preparation, and `process_input_lines` → `draw` path are unchanged. The wrapping column now measures append-cache update, not full re-wrap. Cache-lookup medians ranged from 0.000030 to 0.000149 ms. Stages overlap and must not be summed.

Separate extension probe, from 10,000 to 20,000 retained lines at width 80:

| Run | First 5,000 updates median | Last 5,000 updates median | Slowest update | Slowest append number |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 0.003359 | 0.003870 | 0.281842 | 2,020 |
| 2 | 0.003355 | 0.003811 | 0.284951 | 6,385 |
| 3 | 0.003364 | 0.003283 | 0.250917 | 6,385 |

These occasional 0.251–0.285 ms outliers are separate from sustained full-history wrapping. Runs 2 and 3 peak at retained row 16,385, consistent with vector capacity growth; the probe does not instrument the allocator, so it cannot attribute every spike (including run 1's earlier peak) to allocation rather than scheduling/other noise. Half-run medians stay at roughly 3–4 microseconds instead of growing with history. No allocation-free or per-update latency guarantee is claimed.

Verified contracts (`src/app/output_buffer.rs`, `src/app/mod.rs`):

- The owned cache incorporates only the pending immutable source suffix at unchanged width; multiple appends before a draw preserve order exactly once. Cached draws and height-only changes do no old wrapping.
- Empty/fully gagged batches preserve existing rows; mixed batches, blank lines, ANSI styling, Unicode, and existing wrap rules are preserved.
- Clear discards cached rows and source progress, including same-count refill; zero width does not consume pending content, including append/clear/refill between requests.
- App `TestBackend` regressions cover real batched append/draw while following and scrolled back, styles, selected row positions, resize, 70,000-row access, HUD geometry, dialogs/cursor, cached reconnect preservation/clearing, and terminal-only `/clear`.
- Formatting, all-target build, warning-denied all-feature Clippy, and the complete all-target/all-feature suite pass: 799 passed, one existing manual `combat_damage::storage::tests::backfill_user_damage_db` ignored. It requires an explicitly supplied database and was not run.
- Raw and wrapped history memory remain unbounded. Initial population and terminal-width changes still wrap all retained content; ordinary extension is amortized because vector growth can move existing entries. No storage redesign or Unicode/span/gag-filter change was made.

## Agent Synthesis

At the slice 01 checkpoint, 10,000-line append-and-draw was 37.170–38.344 ms (about 2.6–2.7× faster than baseline); full-history append wrapping dominated the remainder. Slice 02 reduces it to 0.163–0.168 ms, approximately 602–620× faster than the original 101.132 ms, exceeding the combined 10× target (10.1 ms). Fixed-viewport cached draws are 0.152–0.180 ms at 1,000/10,000 lines; ordinary append-cache updates are about 2–3 microseconds, with no material retained-history trend across the three sizes. The remaining measured ordinary cost is viewport preparation/rendering, not old-history wrapping. The separate extension probe makes occasional growth/outlier costs visible without mistaking them for sustained history-dependent work.

## Open Questions

- How closely does this synthetic history match the user's actual line lengths, styling, terminal size, and build mode?
- How much additional time do logged-in triggers, persistence, and the real terminal backend contribute? They are intentionally excluded here.
- No live BatMUD session, user-local configuration/log/database access, or real terminal-backend validation was performed. Offline acceptance is not proof of every live-session performance contributor.

## Related

- [batrs client application](../subsystems/batrs-client.md)
- [Older text input profiling note](text-input-slow-profiling.md)
