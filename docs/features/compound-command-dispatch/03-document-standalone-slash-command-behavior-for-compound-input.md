# Document standalone slash-command behavior for compound input

## Suggested branch name

`feature/compound-command-dispatch`

## Parent

`docs/features/compound-command-dispatch/prd.md`

## What to build

Update the user-facing Command Dispatch documentation so compound input behavior is explicit. Document that logged-in non-slash input supports semicolon-separated compound commands, whitespace around separators is ignored, empty segments are skipped, and slash commands remain standalone client commands rather than valid compound segments.

This slice is complete when the manual explains the new compound-input behavior clearly enough that a player can predict what batrs will send to BatMUD.

## Acceptance criteria

- [ ] The manual describes semicolon-separated compound input for logged-in non-slash commands.
- [ ] The manual explains whitespace trimming around `;` and empty-segment skipping.
- [ ] The manual states that slash commands remain standalone and are not supported as compound segments.
- [ ] The documentation remains aligned with actual Command Dispatch behavior.

## Blocked by

- `docs/features/compound-command-dispatch/02-preserve-effect-ordering-and-automation-in-compound-command-dispatch.md`
