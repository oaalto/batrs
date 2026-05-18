# Domain Context

## Guild Catalog

The Guild Catalog is the canonical Rust-source list of BatMUD guild keywords known to batrs. It includes playable guilds that can be enabled for a player and unimplemented BatMUD guild keywords that still matter for thematic grouping.

The Guild Catalog owns persisted guild keys, display names, grouping membership, playability, and playable guild construction.

## Player Profile

The Player Profile is the per-player runtime configuration loaded from the user's batrs player file. It includes selected guilds, the active guild primary background, settings, and generic command preferences.

The Player Profile owns the interpretation of persisted player settings into runtime profile effects, while configuration file I/O and TOML migration remain owned by the config module.
