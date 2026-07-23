---
title: Player Profile
type: concept
status: current
updated: 2026-07-23
sources:
  - CONTEXT.md
  - src/player_profile.rs
  - src/config.rs
---

# Player Profile

## Summary

The Player Profile is the per-player runtime configuration loaded from the user's batrs player TOML file. It includes selected guilds, the active guild primary background, settings, and generic command preferences.

## Verified Facts

- Runtime type: `PlayerRuntimeProfile` in `src/player_profile.rs` — guild selection, settings map, generic command config.
- Interpretation: `interpret_player_toml` converts persisted `PlayerToml` into runtime effects; config file I/O and TOML migration remain in `src/config.rs` (`CONTEXT.md`).
- Player files live under `~/.batrs/` (see `config.rs` `base_dir`).
- Settings include guild-specific keys (e.g. `tzarakk_mount`, `sabre_weapon`, riftwalker entity labels, `is_lich`).
- Player Profile reload is deferred until the next successful login after a `/connect` reconnect (`CONTEXT.md` — Connect Command).

## Related

- [Guild Catalog](guild-catalog.md)
- [Command Dispatch](command-dispatch.md)
- `CONTEXT.md` — Player Profile section
