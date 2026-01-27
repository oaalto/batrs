---
globs:
alwaysApply: true
---

## Agent Guide

This project values correctness, functional style, and concise code. Prefer small, focused functions and keep logic easy to reason about.

### Principles
- Prioritize correctness and explicit invariants over cleverness.
- Favor functional patterns: pure functions, immutability, and clear data flow.
- Keep functions short and single-purpose; refactor when they grow.
- Use enums over string or integer keys for domain values.
- Prefer static analysis and compiler guidance; keep types precise.
- Be terse: remove redundancy and avoid unnecessary abstractions.

### Rust Conventions
- Prefer pattern matching over chained conditionals.
- Use `Option`/`Result` to model absence and failure explicitly.
- Avoid `unwrap()` except in tests or unreachable code with justification.
- Keep side effects at the edges; core logic should be pure.

### Quality Checks
- After each feature, run `cargo clippy`, `cargo build`, `cargo fmt`, and `cargo test`.
- Always fix warnings and errors before moving on.
- Remove any unused functions and fields
- Never use #[allow(dead_code)]

### TUI Code Conventions
- Use concise styling helpers from ratatui’s `Stylize` trait.
- Basic spans: use `"text".into()`.
- Styled spans: use `"text".red()`, `"text".green()`, `"text".magenta()`, `"text".dim()`, etc.
- Prefer these over constructing styles with `Span::styled` and `Style` directly.
- Example: patch summary file lines
  - Desired: `vec![" └ ".into(), "M".red(), " ".dim(), "tui/src/app.rs".dim()]`

### TUI Styling (ratatui)
- Prefer `Stylize` helpers: use `"text".dim()`, `.bold()`, `.cyan()`, `.italic()`, `.underlined()` instead of manual `Style` where possible.
- Prefer simple conversions: use `"text".into()` for spans and `vec![…].into()` for lines; when inference is ambiguous (e.g., `Paragraph::new`/`Cell::from`), use `Line::from(spans)` or `Span::from(text)`.
- Computed styles: if the `Style` is computed at runtime, using `Span::styled` is OK (`Span::from(text).set_style(style)` is also acceptable).
- Avoid hardcoded white: do not use `.white()`; prefer the default foreground (no color).
- Chaining: combine helpers by chaining for readability (e.g., `url.cyan().underlined()`).
- Single items: prefer `"text".into()`; use `Line::from(text)` or `Span::from(text)` only when the target type isn’t obvious from context, or when using `.into()` would require extra type annotations.
- Building lines: use `vec![…].into()` to construct a `Line` when the target type is obvious and no extra type annotations are needed; otherwise use `Line::from(vec![…])`.
- Avoid churn: don’t refactor between equivalent forms (`Span::styled` ↔ `set_style`, `Line::from` ↔ `.into()`) without a clear readability or functional gain; follow file‑local conventions and do not introduce type annotations solely to satisfy `.into()`.
- Compactness: prefer the form that stays on one line after rustfmt; if only one of `Line::from(vec![…])` or `vec![…].into()` avoids wrapping, choose that. If both wrap, pick the one with fewer wrapped lines.

### TinyFugue (tf)
- Any files with the extension of `.tf` are TinyFugue script files, *not* Terraform.
- @http://flaprider.dyndns.org/~hair/tf/commands/index.html
- @http://flaprider.dyndns.org/~hair/tf/topics/index.html
