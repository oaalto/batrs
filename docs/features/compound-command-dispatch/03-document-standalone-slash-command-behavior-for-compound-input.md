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

- none

**Implementation knowledge from 02-preserve-effect-ordering-and-automation-in-compound-command-dispatch**

- Contracts/seams: `src/command/mod.rs` keeps compound splitting in `dispatch_compound_line`, while per-segment effect shaping now happens by applying `collapse_adjacent_sends` to each segment result before flattening. Compound input therefore preserves each segment's effect order instead of globally re-collapsing sends across segment boundaries.
- Decisions that changed the plan: the implemented behavior keeps per-segment equivalence ahead of whole-line send collapsing. That means docs should describe player-visible compound behavior in terms of segment-by-segment Command Dispatch, not one final merged BatMUD send.
- Traps/gotchas: existing send payloads that already contain `;` are still opaque payload text. Do not document compound input as recursively splitting shortcut-generated send strings.
- File/API map: player-facing behavior lives in `src/command/mod.rs` tests `dispatch_splits_logged_in_compound_input_and_preserves_passthrough_order`, `dispatch_skips_empty_compound_segments`, `dispatch_preserves_segment_effect_order_with_automation`, and `dispatch_does_not_resplit_existing_shortcut_send_payloads`.
