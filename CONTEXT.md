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

The Connect Command is the client slash command, available before and after login, that starts a fresh BatMUD login session after the existing game connection is unusable.

The Connect Command has relaunch semantics for login-dependent state: it clears active runtime state immediately, and Player Profile loading happens again only after the next successful login.

If the Connect Command cannot open a fresh BatMUD connection, the client remains in the fresh-session state and reports the failure rather than preserving the previous session.

The Connect Command is consumed by Command Dispatch as a client command and is never sent to BatMUD as game input.

Only one Connect Command attempt may be active at a time; repeated requests report that reconnect is already in progress.

## Combat Awareness

Combat Awareness is batrs' interpretation of whether the player is currently in BatMUD combat. It begins when combat round output is observed and ends when BatMUD reports that the player is not in combat.

A Combat Scan Snapshot is the latest observed set of combatants and their health from a completed scan result. Each completed scan result replaces the previous snapshot rather than appending to it.
