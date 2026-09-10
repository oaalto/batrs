# Reuse melee matcher search order in Combat Damage collection

## Parent

`docs/features/codebase-optimization-opportunities/prd.md`

## What to build

Make Combat Damage melee attribution reuse static catalog structure instead of rebuilding or resorting melee search order for each incoming line. From the user's perspective, melee candidate matching, verb attribution, and weapon-family semantics remain unchanged while Combat Damage collection does less repeated matcher work.

## Acceptance criteria

- [ ] Melee candidate matching preserves current verb attribution and `weapon_family` semantics, including ADR-backed family-sensitive aggregation behavior.
- [ ] Incoming melee matching no longer allocates and sorts a fresh search-order structure for each line.
- [ ] Existing Combat Damage matcher behavior remains unchanged for recent-family preference and family-collision cases.
- [ ] Existing matcher tests, or equivalent behavior checks, cover the preserved classification semantics.

## Blocked by

- None (can start immediately).

## Status

ready-for-agent
