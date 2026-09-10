# Streamline Combat Damage aggregate and backfill row processing

## Parent

`docs/features/codebase-optimization-opportunities/prd.md`

## What to build

Make Combat Damage aggregate queries and historical maintenance paths process rows with less unnecessary intermediate collection so the local dashboard scales better with larger datasets. From the user's perspective, landing-page summaries, drill-down behavior, and unattributed HP-loss review behavior remain the same while viewer and maintenance work do less repeated allocation.

## Acceptance criteria

- [ ] Combat Damage landing-page summaries, drill-down behavior, and unattributed HP-loss review behavior remain unchanged from the player's perspective.
- [ ] Aggregate and backfill paths reduce unnecessary intermediate collection or full-row materialization where equivalent streaming or narrower processing is possible.
- [ ] Larger Combat Damage datasets are handled without regressing current semantics for confirmed, estimated, melee-family, or unattributed views.
- [ ] Any dependence on storage open behavior is satisfied by the routine-open split landing first.

## Blocked by

- `docs/features/codebase-optimization-opportunities/combat-damage-open-path.md`

## Status

done
