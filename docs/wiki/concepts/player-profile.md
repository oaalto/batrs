---
title: Player Profile
type: concept
status: current
updated: 2026-11-05
sources:
  - CONTEXT.md
  - src/player_profile/mod.rs
  - src/player_profile/registry.rs
  - src/player_profile/runtime.rs
  - src/config.rs
  - src/app/mod.rs
  - src/guilds/triad/commands.rs
  - src/ui/mod.rs
---

# Player Profile

## Summary

The Player Profile is the per-player runtime configuration loaded from the user's batrs player TOML file. It includes selected guilds, the active guild primary background, settings, generic command preferences, and trigger group toggles (`[triggers]`).

## Verified Facts

- Runtime type: `PlayerRuntimeProfile` in `src/player_profile/runtime.rs` — guild selection, known settings, generic command config, `trigger_config`, `monk_skills_config`.
- `[monk_skills]` on player TOML: four tracks × three slot booleans; omitted section means all enabled. Edited via `/monk` when monk is in guild selection.
- `[triggers]` on player TOML: four booleans (`guild_triggers`, `spell_vocals`, `common_triggers`, `core_triggers`); omitted section or keys mean enabled. Sparse write omits `true` keys.
- Trigger toggles editable via `/triggers` dialog or hand-edited TOML; successful dialog save updates runtime profile immediately (`CONTEXT.md` Trigger Chain section).
- Interpretation: `interpret_player_toml` converts persisted `PlayerToml` into runtime effects; config file I/O and TOML migration remain in `src/config.rs` (`CONTEXT.md`).
- Player files live under `~/.batrs/` (see `config.rs` `base_dir`).
- Settings include guild-specific keys (e.g. `tzarakk_mount`, `sabre_weapon`, riftwalker entity labels, `is_lich`).
- `/settings` presents known and custom keys as title-cased words with underscores replaced by spaces (e.g. `tzarakk_mount` → **Tzarakk Mount:**); `damn_armament_target` keeps its **Damn Armament:** label. Row rendering and cursor placement share the display-name function; cursor placement measures label and value terminal widths, including combining and wide characters. Persisted keys and values are unchanged.
- `damn_armament_target` is a registered string setting, persisted under `[settings]`, exported into the command environment, and edited as **Damn Armament:** in `/settings` independently of guild selection. Successful saves refresh the runtime profile immediately. Bulk settings saves persist a candidate before replacing loaded settings, so a failed disk write cannot activate rejected settings during a later refresh.
- Triad `cda` casts directly on the configured item suffix without a separate `target` command and ignores shortcut arguments. Missing, empty, or whitespace-only values produce `Set the Damn Armament target in /settings` without sending; there is no `weapon2` fallback. Surrounding whitespace is trimmed, and semicolons or line breaks produce a local error without sending.
- Player Profile reload is deferred until the next successful login after a `/connect` reconnect; `FreshSessionReset::PlayerProfile` clears runtime profile immediately on connect (`CONTEXT.md`, [Session Lifecycle](session-lifecycle.md)).

## Related

- [Guild Catalog](guild-catalog.md)
- [Monk Skill Tracks](monk-skill-tracks.md)
- [Command Dispatch](command-dispatch.md)
- [Session Lifecycle](session-lifecycle.md)
- `CONTEXT.md` — Player Profile section
