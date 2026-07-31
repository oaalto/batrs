---
title: Guild Catalog
type: concept
status: current
updated: 2026-07-31
sources:
  - CONTEXT.md
  - src/guilds/catalog/mod.rs
  - src/guilds/catalog/browse.rs
---

# Guild Catalog

## Summary

The Guild Catalog is the canonical Rust-source list of BatMUD guild keywords known to batrs, including playable guilds, background-only guild modules, and stub guilds without commands yet.

## Verified Facts

- Owns persisted guild keys, display names, grouping membership, playability, and playable guild construction (`CONTEXT.md`).
- Background→guild membership table: [Guild Background Map](guild-background-map.md).
- Browse submodule (`src/guilds/catalog/browse.rs`) owns PickBackground label ordering, drill source (`GuildDrillSource`), drill row structure (`GuildBrowseRow`), and `drill_rows(source, entry_count, multi_filter_thematic)` — including empty-state banner copy.
- Grouping (`src/guilds/grouping.rs`) owns thematic bucket indices, multi-background eligibility (`multi_guild_eligible_for_thematic`), filtered multi drill indices (`visible_indices_multi_drill_for`), and `clear_selected_outside_thematic_bucket`.
- Guild Dialog (`src/app/dialogs/guild_dialog.rs`) delegates browse labels and drill rows to browse; it keeps focus, cursors, keystroke handling, and guild-specific text inputs.
- Implementation: `src/guilds/catalog/`, `src/guilds/grouping.rs`.
- Per-guild behavior lives under `src/guilds/<guild>/` (commands and triggers).
- Guild command modules import `use_skill` and `cast_spell` from Abilities (`src/abilities/mod.rs`), not from the Guild root module.

## Related

- [Abilities](abilities.md)
- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Guild Catalog section
- [Guild overview in user manual](../../guilds/index.md)
