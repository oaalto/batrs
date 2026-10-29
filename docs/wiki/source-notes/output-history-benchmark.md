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

An offline release benchmark reproduces increasing append-and-draw cost as output history grows, even with a fixed 80×24 viewport. Full-history wrapping and copying both contribute. This demonstrates a local scaling problem, not yet a diagnosis of a particular live session.

## Verified Facts

Run the real output buffer and `BatApp::draw` through ratatui's in-memory `TestBackend`:

```bash
cargo test --release benchmark_output_history -- --nocapture --test-threads=1
```

The test uses the existing fake app with no config, loggers, damage database, or network. History consists of copies of a short ANSI-colored ASCII combat line. Setup and initial cache population are excluded. Each result is the median of five samples; history grows by two lines per sample. Assertions verify wrapped row counts and the fixed viewport. Timings are reported, not asserted, to avoid machine-dependent test failures.

One measured run on 2026-10-29, in milliseconds:

| Initial history | Cache lookup | Re-wrap after append | Two history copies¹ | Cached draw | Process one new line + draw |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 0.000027 | 0.308 | 0.385 | 0.644 | 1.018 |
| 1,000 | 0.000019 | 2.916 | 3.632 | 5.725 | 9.408 |
| 10,000 | 0.000038 | 37.441 | 44.146 | 62.836 | 101.132 |

¹ Replays the view-model `to_vec()` and renderer's `clone()` into `Text`/`Paragraph`, including destruction. Other columns overlap these operations; do not add all columns together.

Repeated release runs retained the scaling pattern; the 10,000-line append-and-draw median ranged from approximately 98 to 109 ms across three invocations. One repeat ran concurrently with the full test suite and is not an isolated timing baseline.

- `OutputBuffer::append_lines` changes the generation; `wrapped_lines` rebuilds every stored line when generation or width differs (`src/app/output_buffer.rs`).
- `BatApp::draw` copies the complete wrapped slice into the view model; `Renderer::render` copies it again (`src/app/mod.rs`, `src/ui/mod.rs`).
- A cache hit avoids wrapping but does not avoid those draw-time copies.

## Agent Synthesis

Increasing initial history 100-fold raised append-and-draw cost approximately 99-fold in the recorded baseline. At 10,000 lines, cached drawing alone cost about 63 ms. Incremental wrapping alone therefore cannot eliminate all history-dependent redraw cost; full-history copies also need investigation at the existing view/render seam.

## Open Questions

- How closely does this synthetic history match the user's actual line lengths, styling, terminal size, and build mode?
- How much additional time do logged-in triggers, persistence, and the real terminal backend contribute? They are intentionally excluded here.
- A `/fix` should carry this benchmark forward alongside regression checks for visible output, scrollback, append, clear, and width changes; no production optimization was applied in this benchmark change.

## Related

- [batrs client application](../subsystems/batrs-client.md)
- [Older text input profiling note](text-input-slow-profiling.md)
