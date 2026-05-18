# Domain Context

## Guild Catalog

The Guild Catalog is the canonical Rust-source list of BatMUD guild keywords known to batrs. It includes playable guilds that can be enabled for a player and unimplemented BatMUD guild keywords that still matter for thematic grouping.

The Guild Catalog owns persisted guild keys, display names, grouping membership, playability, and playable guild construction.

## Player Profile

The Player Profile is the per-player runtime configuration loaded from the user's batrs player file. It includes selected guilds, the active guild primary background, settings, and generic command preferences.

The Player Profile owns the interpretation of persisted player settings into runtime profile effects, while configuration file I/O and TOML migration remain owned by the config module.

## Command Dispatch

Command Dispatch is the runtime interpretation of a command input line into client effects. These effects include sending text to BatMUD, opening dialogs, emitting output, applying automation actions, toggling logging, and quitting.

Command Dispatch returns effects for the application to apply. It does not own the concrete adapters that send text, render dialogs, write logs, or persist state.

Command Dispatch owns command precedence and login gating for command input. It does not own the login conversation, guild ability definitions, or Player Profile persistence.

Command Dispatch may use command environment facts, such as runtime flags and variables, but those facts are snapshots supplied by the application rather than Player Profile data owned by Command Dispatch.
