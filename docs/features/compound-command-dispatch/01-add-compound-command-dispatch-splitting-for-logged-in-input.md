# Add compound Command Dispatch splitting for logged-in input

## Suggested branch name

`feature/compound-command-dispatch`

## Parent

`docs/features/compound-command-dispatch/prd.md`

## What to build

Teach Command Dispatch to treat semicolon-separated logged-in non-slash input as compound user input instead of one raw line. The slice should split a typed line into command segments, trim whitespace around separators, skip empty segments, run each non-empty segment through the existing single-segment Command Dispatch flow, and preserve current passthrough behavior for unknown non-slash segments.

This slice establishes the core compound-input path while keeping pre-login behavior and standalone slash-command behavior unchanged.

## Acceptance criteria

- [ ] Logged-in non-slash input containing `;` is treated as compound input instead of one unsplit Command Dispatch line.
- [ ] Whitespace around `;` is ignored and empty segments are skipped.
- [ ] Unknown non-slash segments still pass through unchanged in their original order.
- [ ] Pre-login input handling remains unchanged.
- [ ] Standalone slash-command behavior remains unchanged.

## Blocked by

None - can start immediately.
