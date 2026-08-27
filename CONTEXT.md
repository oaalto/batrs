# batrs domain context

batrs is a Rust terminal client for BatMUD. This glossary exists to keep planning, implementation, review, and wiki updates using the same domain language instead of drifting into file- or module-specific synonyms.

## Language

**batrs client**:
The terminal client application in this repository: it connects to BatMUD over telnet, renders the TUI, dispatches slash commands locally, and runs game-specific automation.
_Avoid_: app shell, frontend

**Command Dispatch**:
The client-local command interpreter that turns slash-command input into client effects such as output, dialogs, reconnect actions, terminal redraw, or forwarded game input.
_Avoid_: command parser when the meaning includes effect routing and login gating

**Session Lifecycle**:
The reconnect and fresh-session state model that decides what runtime state survives, resets, or is ignored across connect/login transitions.
_Avoid_: reconnect helper, login glue

**Player Profile**:
The per-player runtime configuration loaded from the player TOML under `~/.batrs/`, including guild selection, settings, generic command preferences, and trigger toggles.
_Avoid_: config blob, user settings when the per-player meaning matters

**Guild Catalog**:
The catalog of supported guild capabilities and metadata that command discovery, selection, and guild-specific behavior rely on.
_Avoid_: guild list when the richer capability/catalog meaning matters

**Combat Damage**:
The incoming-damage capture and reporting capability that records HP-loss events into `~/.batrs/combat_damage.db` and serves the local HTTP damage viewer.
_Avoid_: parser only, damage logs

**Secondary Status**:
The guild-specific HUD state rendered below the main stats line, separate from the primary stats ownership path.
_Avoid_: extra stats, lower HUD rows when the distinct domain concept is intended

## Relationships

- **Command Dispatch** produces client effects that the **batrs client** applies.
- A **Player Profile** belongs to one player login and lives under `~/.batrs/`.
- The **Guild Catalog** informs what guild-specific commands, triggers, and status ownership the **batrs client** exposes.
- **Combat Damage** and **Secondary Status** are distinct capabilities inside the **batrs client** and should not be conflated with general stats or raw logs.

## Example Dialogue

> **Dev:** "Should this reconnect path clear Command Dispatch state or Session Lifecycle state?"
> **Domain expert:** "Command Dispatch stays the command seam; Session Lifecycle owns what runtime state resets on reconnect."

## Flagged Ambiguities

- "stats" is used in multiple senses; resolved: use **Secondary Status** for guild HUD rows below the main stats line, and reserve stats for the primary stats ownership path.
- "config" is too broad; resolved: use **Player Profile** for per-player TOML-backed runtime configuration.

## To Complete

Agent instruction: When this section lists items, offer the user LLM-assisted follow-up to resolve them. Do not invent definitions silently.

- Confirm whether any additional canonical domain terms from `docs/wiki/concepts/` should be promoted into this root glossary now.
