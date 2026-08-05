---
title: Player Profile
type: concept
status: current
updated: 2026-07-27
sources:
  - CONTEXT.md
  - src/player_profile.rs
  - src/config.rs
---

# Player Profile

## Summary

The Player Profile is the per-player runtime configuration loaded from the user's batrs player TOML file. It includes selected guilds, the active guild primary background, settings, generic command preferences, and trigger group toggles (`[triggers]`).

## Verified Facts

- Runtime type: `PlayerRuntimeProfile` in `src/player_profile.rs` — guild selection, settings map, generic command config, `trigger_config`, `monk_skills_config`.
- `[monk_skills]` on player TOML: four tracks × three slot booleans; omitted section means all enabled. Edited via `/monk` when monk is in guild selection.
- `[triggers]` on player TOML: four booleans (`guild_triggers`, `spell_vocals`, `common_triggers`, `core_triggers`); omitted section or keys mean enabled. Sparse write omits `true` keys.
- Trigger toggles editable via `/triggers` dialog or hand-edited TOML; successful dialog save updates runtime profile immediately (`CONTEXT.md` Trigger Chain section).
- Interpretation: `interpret_player_toml` converts persisted `PlayerToml` into runtime effects; config file I/O and TOML migration remain in `src/config.rs` (`CONTEXT.md`).
- Player files live under `~/.batrs/` (see `config.rs` `base_dir`).
- Settings include guild-specific keys (e.g. `tzarakk_mount`, `sabre_weapon`, riftwalker entity labels, `is_lich`).
- Player Profile reload is deferred until the next successful login after a `/connect` reconnect; `FreshSessionReset::PlayerProfile` clears runtime profile immediately on connect (`CONTEXT.md`, [Session Lifecycle](session-lifecycle.md)).

## Related

- [Guild Catalog](guild-catalog.md)
- [Monk Skill Tracks](monk-skill-tracks.md)
- [Command Dispatch](command-dispatch.md)
- [Session Lifecycle](session-lifecycle.md)
- `CONTEXT.md` — Player Profile section
