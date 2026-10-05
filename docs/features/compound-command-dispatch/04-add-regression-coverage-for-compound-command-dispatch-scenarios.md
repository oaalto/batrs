# Add regression coverage for compound Command Dispatch scenarios

## Suggested branch name

`feature/compound-command-dispatch`

## Parent

`docs/features/compound-command-dispatch/prd.md`

## What to build

Add regression coverage that exercises compound Command Dispatch as a player-visible behavior. The coverage should verify raw passthrough plus shortcut expansion, multiple shortcut segments, whitespace handling, empty-segment skipping, preserved effect ordering for shortcut segments that emit automation, and standalone-only slash-command behavior.

This slice is complete when the tests protect the compound-input contract without depending on internal helper shapes.

## Acceptance criteria

- [ ] Regression tests cover passthrough plus shortcut expansion in one compound line.
- [ ] Regression tests cover whitespace handling and empty-segment skipping.
- [ ] Regression tests cover compound input where a segment emits both send and automation effects.
- [ ] Regression tests cover preservation of existing shortcut send payloads that already contain `;`.
- [ ] Regression tests cover standalone-only slash-command behavior for compound input.

## Blocked by

- `docs/features/compound-command-dispatch/01-add-compound-command-dispatch-splitting-for-logged-in-input.md`

**Implementation knowledge from 02-preserve-effect-ordering-and-automation-in-compound-command-dispatch**

- Contracts/seams: the stable regression seam is `command::dispatch` through `dispatch_line`-style tests in `src/command/mod.rs`; `collapse_adjacent_sends` still matters, but only as the helper that collapses sends within one segment result or standalone dispatch result.
- Decisions that changed the plan: the ticket landed part of the regression coverage directly in `src/command/mod.rs` while fixing the behavior. Future coverage should stay player-visible and assert final `CommandEffect` ordering, not helper-internal iteration shape.
- Traps/gotchas: old expectations that compound input becomes one merged send are now wrong for multi-segment input once segment boundaries matter. Also, shortcuts whose send payload already contains `;` must remain opaque and should not be asserted as recursively split segments.
- File/API map: current coverage was added in `src/command/mod.rs` tests `dispatch_preserves_segment_effect_order_with_automation` and `dispatch_collapses_adjacent_send_effects_within_one_segment`; adjacent baseline tests in the same module already cover passthrough ordering, empty-segment skipping, and embedded-`;` payload preservation.
