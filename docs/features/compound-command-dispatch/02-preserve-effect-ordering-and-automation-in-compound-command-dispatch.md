# Preserve effect ordering and automation in compound Command Dispatch

## Suggested branch name

`feature/compound-command-dispatch`

## Parent

`docs/features/compound-command-dispatch/prd.md`

## What to build

Make compound Command Dispatch preserve the full effect stream for each command segment. Each segment should produce the same ordered effects it would produce as standalone input. When the compound line is reassembled, adjacent send effects should collapse into one BatMUD send joined with `;`, while automation and other non-send effects stay in their normal relative order.

This slice is complete when shortcut segments that rely on automation functionality still work inside compound input and existing shortcut send payloads that already contain `;` keep their current meaning.

## Acceptance criteria

- [x] Compound input preserves the same per-segment effect ordering as standalone Command Dispatch.
- [x] Adjacent send effects are collapsed into one send payload joined with `;`.
- [x] Automation and other non-send effects remain separate and keep their relative order.
- [x] Shortcut segments that emit both send and automation effects continue to work inside compound input.
- [x] Existing shortcut send payloads that already contain `;` are not recursively re-split or reinterpreted.

## Status

completed

## Blocked by

- `docs/features/compound-command-dispatch/01-add-compound-command-dispatch-splitting-for-logged-in-input.md`
