---
title: Monk Skill Tracks
type: concept
status: current
updated: 2026-08-05
sources:
  - CONTEXT.md
  - docs/guilds/monk.md
  - src/guilds/monk/skills_config.rs
  - src/guilds/monk/commands.rs
  - src/guilds/monk/triggers.rs
---

# Monk Skill Tracks

## Summary

Monk **skill tracks** are the four rotating combat chains (disrupt, armour, area, avoid). Each track has three **chain slots**. The Player Profile stores which slots the character has unlocked via `[monk_skills]`; `/monk` edits them in-session when monk is selected in `/guilds`.

## Verified Facts

- Config type: `MonkSkillsConfig` in `src/guilds/monk/skills_config.rs`; persisted as `[monk_skills]` on player TOML (sparse when all slots enabled).
- Dialog: `/monk` (`DialogKind::Monk`) — four section headers, twelve skill rows, prefix-chain checkbox rules (select slot *n* enables `1..=n`; deselect slot *n* clears `n..=3`).
- Gating: track shortcuts and dedicated slot aliases check enabled slots before send; rotation triggers still color lines but skip `SetVar` when the target slot is disabled.
- On save or profile load, rotation automation vars are clamped to the first enabled slot per track.
- Monk skill config is independent of the guild-triggers toggle in `/triggers`; disabling guild triggers skips all monk triggers including rotation updates.

## Related

- [Player Profile](player-profile.md)
- [Guild Catalog](guild-catalog.md)
- `docs/guilds/monk.md` — player-facing shortcuts
- `CONTEXT.md` — Player Profile section
