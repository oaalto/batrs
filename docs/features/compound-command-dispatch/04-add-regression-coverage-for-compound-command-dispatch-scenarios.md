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
- `docs/features/compound-command-dispatch/02-preserve-effect-ordering-and-automation-in-compound-command-dispatch.md`
