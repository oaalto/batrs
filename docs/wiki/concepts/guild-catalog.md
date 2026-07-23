---
title: Guild Catalog
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/guilds/catalog/mod.rs
---

# Guild Catalog

## Summary

The Guild Catalog is the canonical Rust-source list of BatMUD guild keywords known to batrs, including playable guilds and unimplemented keywords used for thematic grouping.

## Verified Facts

- Owns persisted guild keys, display names, grouping membership, playability, and playable guild construction (`CONTEXT.md`).
- Implementation: `src/guilds/catalog/` and `src/guilds/grouping.rs`.
- Per-guild behavior lives under `src/guilds/<guild>/` (commands and triggers).

## Related

- [batrs client application](../subsystems/batrs-client.md)
- `CONTEXT.md` — Guild Catalog section
- [Guild overview in user manual](../../guilds/index.md)
