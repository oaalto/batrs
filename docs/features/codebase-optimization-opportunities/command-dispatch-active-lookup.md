# Cache active Command Dispatch lookups for the selected Guild Catalog

## Parent

`docs/features/codebase-optimization-opportunities/prd.md`

## What to build

Make Command Dispatch reuse a stable active slash-command lookup for the current Guild Catalog selection instead of rebuilding merged guild command maps on every dispatch. From the user's perspective, slash-command behavior, login gating, precedence, and generic-command fallback remain unchanged while local command handling does less repeated work.

## Acceptance criteria

- [x] Slash-command behavior remains unchanged for builtin commands, guild commands, precedence, and generic-command fallback.
- [x] Changing the active Guild Catalog selection updates the active command lookup correctly.
- [x] Command Dispatch no longer rebuilds the same merged guild command map on each dispatch for an unchanged guild selection.
- [x] Existing Command Dispatch tests or equivalent behavior checks cover the preserved resolution semantics.

## Blocked by

- None (can start immediately).

## Status

done
