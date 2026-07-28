# Improve Line Effect Application

## Status

grilled — ready for `/to-tickets` and implementation

## Problem

The line effect system applies visual modifications to game output lines. The current implementation in `src/triggers/mod.rs` tangles three related concerns:

1. **`LineEffect` enum** (4 variants) — how to modify a `StyledLine`:
   ```rust
   pub enum LineEffect {
       StyleLine(TextStyle),
       StyleBlock { text: String, style: TextStyle },
       StylePlainByteRange { range: Range<usize>, style: TextStyle },
       InsertPlainAfterPlainByteIdx { byte_idx: usize, suffix: String },
   }
   ```

2. **`TriggerEffects` struct** — the container that collects all effects from a trigger pass:
   ```rust
   pub struct TriggerEffects {
       pub original: OriginalLineEffects,  // edits to the original line
       pub lines: Vec<StyledLine>,         // new lines to emit
       pub actions: Vec<Action>,
       pub stats: Vec<StatsEffect>,
       pub secondary_status: Vec<SecondaryStatusEffect>,
   }
   ```

3. **`apply_line_effects_to`** — applies `original.edits` (and `gag`) to a `StyledLine`:
   ```rust
   pub fn apply_line_effects_to(&self, line: &mut StyledLine) {
       for edit in &self.original.edits {
           edit.apply_to(line);
       }
       if self.original.gag {
           line.gag = true;
       }
   }
   ```

Problems addressed in this slice:

- `LineEffect` and `apply_to` live in `triggers/mod.rs` but conceptually belong with `StyledLine` (in `src/ansi/`).
- Agents reading `LineEffect` lack module-level guidance on how line edits (`original.edits`, `gag`) differ from emitted lines (`lines`).

Out of scope for this slice (deferred):

- Reshaping `TriggerEffects` to separate line edits from emitted lines at the struct level.
- Inverting `apply_to` to `StyledLine::apply_effect`.

## Goals

1. **Move `LineEffect` to `ansi`** — co-locate `LineEffect` and `LineEffect::apply_to` with `StyledLine` in `src/ansi/effect.rs`; keep `TriggerEffects` and `OriginalLineEffects` in `triggers`.
2. **Document the line-edits vs emitted-lines split** — LLM-oriented module docs in `effect.rs` (prose, cross-refs, one minimal code example) explaining `original.edits` / `gag` vs `lines`; no `TriggerEffects` API changes.

## Non-goals

- Adding new `LineEffect` variants.
- Changing the trigger pipeline in `mod.rs`.
- Reshaping `TriggerEffects` or `OriginalLineEffects` fields.
- Inverting `apply_to` to `StyledLine::apply_effect` (intentional coupling stays on `LineEffect::apply_to`).
- Updating call-site imports — all consumers keep `crate::triggers::LineEffect`.
- Making line effects configurable.
- Adding effect composition/ordering guarantees.
- `CONTEXT.md` or wiki updates (in-code LLM docs cover this scope).
- Slice ticket in this step — run `/to-tickets` after implementation planning.

## Implementation Decisions

### File layout

- **Create** `src/ansi/effect.rs` — `LineEffect` enum and `impl LineEffect { pub fn apply_to(&self, line: &mut StyledLine) }` move here verbatim (same match arms, same method name).
- **Update** `src/ansi/mod.rs`:
  ```rust
  mod effect;
  pub use effect::LineEffect;
  ```
  Canonical path: `crate::ansi::LineEffect`.
- **Update** `src/triggers/mod.rs`:
  - Remove `LineEffect` enum and `apply_to` impl.
  - Add `pub use crate::ansi::LineEffect;` so existing imports unchanged.
  - `OriginalLineEffects`, `TriggerEffects`, and `apply_line_effects_to` stay in `triggers/mod.rs`.

### Import boundary

- **Zero call-site edits.** Five modules import `LineEffect` via `crate::triggers::LineEffect`:
  - `src/triggers/rule_engine.rs`
  - `src/guilds/reaver/triggers.rs`
  - `src/guilds/riftwalker/triggers.rs`
  - `src/guilds/magic_lore_analysis.rs`
  - `src/guilds/psionicist/triggers.rs`
- `src/app/mod.rs` uses `triggers::OriginalLineEffects` only — no change.

### API surface

- Keep `LineEffect::apply_to(&self, line: &mut StyledLine)` as-is; do not add `StyledLine::apply_effect`.
- `TriggerEffects::apply_line_effects_to` behavior unchanged (edits first, then `gag`).

### Documentation (`effect.rs`)

LLM-useful rustdoc — not minimal one-liners, not boilerplate.

**Module doc (`//!`):**

- What `LineEffect` is and that `ansi` is its canonical home.
- Import path for callers: `crate::triggers::LineEffect` (re-export); canonical definition at `crate::ansi::LineEffect`.
- **Line edits** vs **emitted lines**:
  - Line edits: `TriggerEffects::original.edits` (`Vec<LineEffect>`) and `original.gag` — applied to the incoming line by `TriggerEffects::apply_line_effects_to`.
  - Emitted lines: `TriggerEffects::lines` — new `StyledLine` values to output; not applied via `LineEffect`.
- `gag` is a sibling line-edit flag on [`OriginalLineEffects`], applied after all `LineEffect` edits by [`TriggerEffects::apply_line_effects_to`]; it is **not** a `LineEffect` variant.
- Cross-refs: [`StyledLine`], [`TriggerEffects`], [`OriginalLineEffects`], [`TriggerEffects::apply_line_effects_to`].
- One minimal rustdoc code example showing a trigger building `original.edits` (and optionally `gag`) separately from `lines`.

**Item docs (`///`):**

- Type and each variant: when to use it; plain-byte index semantics for `StylePlainByteRange` and `InsertPlainAfterPlainByteIdx`.
- `apply_to`: dispatches to the matching [`StyledLine`] mutator; link to those methods.

### Tests (`effect.rs`)

- Inline `#[cfg(test)] mod tests` at bottom of `src/ansi/effect.rs` (same pattern as `styled_line.rs`).
- **One test per variant** (four total) — dispatch smoke only:
  - Build a `StyledLine`, call `effect.apply_to(&mut line)`, assert one observable change (color, block, range style, or suffix present).
  - Do **not** duplicate UTF-8 / byte-index parity tests from `styled_line.rs`; mutator correctness stays there.

## Migration Plan

1. Create `src/ansi/effect.rs` with `LineEffect`, `apply_to`, module/item docs, and tests.
2. Wire `mod effect; pub use effect::LineEffect;` in `src/ansi/mod.rs`.
3. Remove `LineEffect` definition and `apply_to` impl from `src/triggers/mod.rs`; add `pub use crate::ansi::LineEffect;`.
4. Grep confirms no call-site import changes required.
5. Run workflow gates (`cargo fmt`, build, clippy, `cargo test --all-targets --all-features`).

## Success Criteria

- `LineEffect` defined in `src/ansi/effect.rs`, re-exported from `src/ansi/mod.rs` and `src/triggers/mod.rs`.
- No duplicate `LineEffect` definitions.
- All existing tests pass; four new dispatch-smoke tests in `effect.rs`.
- No behavior change — line effects apply identically.
- Module docs in `effect.rs` explain edits vs emitted lines, document `gag`, include cross-refs and one code example.
- Build succeeds with no new warnings.
- No call-site import edits.

## Risks

- **Behavior regression** — mechanical move only; mitigated by existing test suite plus per-variant dispatch tests in `effect.rs`.
- **Import churn** — mitigated by `pub use crate::ansi::LineEffect` in `triggers/mod.rs`; no call-site edits planned or required.
- **Module boundary clarity** — `LineEffect` depends only on `ansi` types (`TextStyle`, `StyledLine`); `triggers` re-exports for trigger-domain consumers. Documented in `effect.rs` module docs.
