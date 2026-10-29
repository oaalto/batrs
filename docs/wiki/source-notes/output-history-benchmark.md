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

Viewport-limited rendering removes the retained-history trend from cached redraws at a fixed 80×24 viewport. Appends still re-wrap all retained output in slice 01. Offline measurements establish local scaling behavior, not the cause of every live-session slowdown.

## Verified Facts

Run the real output buffer and `BatApp::draw` through ratatui's in-memory `TestBackend`:

```bash
cargo test --release benchmark_output_history -- --nocapture --test-threads=1
```

The test uses the existing fake app with no config, loggers, damage database, or network. History consists of copies of a short ANSI-colored ASCII combat line. Setup and initial cache population are excluded. Each result is the median of five samples; history grows by two lines per sample. Assertions verify wrapped row counts and the fixed viewport. Timings are reported, not asserted, to avoid machine-dependent test failures.

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

- `OutputBuffer::append_lines` still changes the generation; `wrapped_lines` rebuilds every retained line when generation or width differs (`src/app/output_buffer.rs`). Slice 02 addresses ordinary append wrapping separately.
- `BatApp::prepare_output` updates `Scrollback` with actual output dimensions and copies only `visible_range()`. `Renderer::render` receives only those rows and does not apply paragraph scrolling (`src/app/mod.rs`, `src/app/scrollback.rs`, `src/ui/mod.rs`).
- `Renderer::layout` supplies identical output/HUD/input geometry to preparation and rendering, including tiny-terminal constraint handling.
- `Scrollback::offset` and its clamped range use `usize`. The 70,000-row small-content screen regression reaches both latest and earlier distinguishable rows beyond 65,535.
- Cache hits and height-only changes avoid wrapping; width changes and appended output still wrap full history. All history is retained, so source and wrapped cache memory still grow with output.

## Agent Synthesis

The original 10,000-line cached-draw median of 62.836 ms is now 0.178–0.186 ms (approximately 338–353× faster). Cached draws across all histories are 0.163–0.226 ms, with no material history-size trend. Append-and-draw remains 37.170–38.344 ms at 10,000 lines, approximately 2.6–2.7× faster than baseline, not the combined feature's 10× target. Full-history append wrapping remains the dominant measured cost and is intentionally reserved for slice 02.

## Open Questions

- How closely does this synthetic history match the user's actual line lengths, styling, terminal size, and build mode?
- How much additional time do logged-in triggers, persistence, and the real terminal backend contribute? They are intentionally excluded here.
- Slice 02 must retain this real viewport preparation stage and the long-history, resize, scrolling, terminal-redraw, and reconnect regressions when replacing append wrapping. No live BatMUD session was run.

## Related

- [batrs client application](../subsystems/batrs-client.md)
- [Older text input profiling note](text-input-slow-profiling.md)
