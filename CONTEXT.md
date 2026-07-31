# Domain Context

## Guild Catalog

The Guild Catalog is the canonical Rust-source list of BatMUD guild keywords known to batrs. It includes playable guilds that can be enabled for a player, background-only guild modules auto-injected from the primary theme keyword, and stub guild modules for BatMUD guilds not yet given commands or triggers.

Each thematic background (`civilized`, `magical`, `good_religious`, `evil_religious`, `nomad`) has a **background-only** catalog entry (`GuildPlayability::BackgroundOnly`). It is buildable for command dispatch but excluded from `/guilds` drill toggles and `playable_entries()`. When the player's guild primary background keyword matches, `GuildSelection::build_guilds` prepends that background guild before guilds built from player-selected keys — including when the persisted guild key list is empty. Background guilds merge first in command dispatch (`or_insert` first registration wins).

See `docs/wiki/concepts/guild-background-map.md` for the full background→guild membership table.

**Good Religious** (`good_religious`) is the only background guild with spell shortcuts implemented so far. The other four background modules (`civilized`, `magical`, `evil_religious`, `nomad`) are empty stubs ready for future background spells.

Good Religious spell shortcuts:

| Alias | Bare send line | With args |
|-------|----------------|-----------|
| `ccs` | `cast 'celestial spark'` | targeted cast (`target <t>;cast 'celestial spark' <t>`) |
| `clw` | `cast 'cure light wounds' me` | `cast 'cure light wounds' <args>` |
| `csw` | `cast 'cure serious wounds' me` | `cast 'cure serious wounds' <args>` |
| `ccw` | `cast 'cure critical wounds' me` | `cast 'cure critical wounds' <args>` |
| `ccf` | `cast 'create food'` | `cast 'create food'` (args ignored) |

Interim rule until per-background spell tables exist: background guild(s) first, player-selected guilds after.

The Guild Catalog browse module (`guilds/catalog/browse.rs`) owns PickBackground labels (`browse_labels()`), drill source (`GuildDrillSource`), drill row structure (`drill_rows(source, entry_count)`), and the `GuildBrowseRow` type (`Banner` + `Toggle { definition_index }`).

`drill_rows(source, entry_count)` filters toggle indices to `definition_index < entry_count`.

`guilds/grouping.rs` remains the source for thematic bucket indices, multi-background indices, and `clear_selected_outside_thematic_bucket`; dialog calls that helper on thematic primary change. `browse.rs` does not mutate selection.

Browse rows are a structural DTO: `Banner(&'static str)` plus `Toggle { definition_index }`. Guild Dialog enriches toggles with display title and selection state when building UI view models.

Browse row-structure tests live in `guilds/catalog/browse.rs`; `guild_dialog.rs` tests cover interaction (cursor, focus, keystrokes) only.

This slice may include minor player-visible fixes discovered during extraction (banner wording, edge-case selection bugs), genuine `GuildSelection` output fixes, and TOML migration if a persistence bug requires it.

Ship as one PR with separate commits per concern (`refactor:` browse extraction, then `fix:` / `migrate:` if discovered).

Guild Dialog owns focus, cursors, keystroke handling, and guild-specific text inputs (mount, sabre weapon, Riftwalker entities); optional-input visibility stays in dialog, not in browse.

## Abilities

Abilities is the bounded context for canonical BatMUD `use` and `cast` command-line formatting. It owns `use_skill`, `cast_spell`, and related helpers that build logical command lines and wrap them with the client send prefix.

Guild command modules consume Abilities for ability-line formatting; they import `use_skill` and `cast_spell` directly from Abilities rather than through Guild re-exports.

Command Dispatch does not own guild `use`/`cast` formatting — that concern is orthogonal to slash-command precedence and login gating.

## Player Profile

The Player Profile is the per-player runtime configuration loaded from the user's batrs player file. It includes selected guilds, the active guild primary background, settings, generic command preferences, and trigger group toggles.

The Player Profile owns the interpretation of persisted player settings into runtime profile effects, while configuration file I/O and TOML migration remain owned by the config module.

## Trigger Chain

The Trigger Chain is the fixed-order pipeline that processes each incoming game line: guild triggers, spell vocals, common triggers, then core triggers (prompt, short score, recovery bracket).

Trigger group toggles live on the Player Profile under `[triggers]` in the player TOML file. Omitted section or omitted keys mean the group is enabled. Players edit toggles in-session via `/triggers` (same login and config-load gates as `/generic`) or by hand-editing TOML; successful save from the dialog updates the in-memory profile immediately without restart.

Disabling guild triggers skips all guild trigger modules for the line, including secondary status effects from guild triggers. Guild selection in `/guilds` is independent of the guild-trigger toggle. v1 supports enable/disable only; reordering, per-rule editing, and a dynamic group registry are out of scope.

## Command Dispatch

Command Dispatch is the runtime interpretation of a command input line into client effects. These effects include sending text to BatMUD, opening dialogs, emitting output, applying automation actions, toggling logging, terminal redraw, and quitting.

Command Dispatch returns effects for the application to apply. It does not own the concrete adapters that send text, render dialogs, write logs, or persist state.

Command Dispatch owns command precedence and login gating for command input. It does not own the login conversation, guild ability definitions, or Player Profile persistence.

Command Dispatch may use command environment facts, such as runtime flags and variables, but those facts are snapshots supplied by the application rather than Player Profile data owned by Command Dispatch.

The Connect Command is the client slash command, available before and after login, that starts a fresh BatMUD login session after the existing game connection is unusable.

The Connect Command has relaunch semantics for login-dependent state: it clears active runtime state immediately, and Player Profile loading happens again only after the next successful login.

If the Connect Command cannot open a fresh BatMUD connection, the client remains in the fresh-session state and reports the failure rather than preserving the previous session.

The Connect Command is consumed by Command Dispatch as a client command and is never sent to BatMUD as game input.

Only one Connect Command attempt may be active at a time; repeated requests report that reconnect is already in progress.

The Clear Command is the client-only slash command `/clear` that triggers a terminal redraw from in-memory UI state. It clears the terminal surface and immediately repaints output scrollback, HUD rows, and the input line from current application state without erasing scrollback or resetting session data.

The Clear Command has no login gate, is available before and after login, and is never sent to BatMUD as game input.

The Clear Command is distinct from Session Lifecycle output scrollback clear on character change after reconnect; a redraw fixes display artifacts without mutating output buffers or session state.

## Session Lifecycle

Session Lifecycle is the application-owned bounded context for fresh-session transitions triggered by the Connect Command. It owns the reset manifest for session-scoped runtime state, the reconnect-in-progress guard, the active connection generation counter, stale-event filtering for superseded connections, and reconnect orchestration against an injected connection coordinator.

Session Lifecycle does not own Player Profile disk I/O, login conversation parsing, UI rendering, output scrollback buffers, or the production telnet adapter. Command Dispatch emits reconnect effects; Session Lifecycle applies Connect Command semantics. The application shell applies the fresh-session plan to its fields, reloads Player Profile after the next successful login, and clears output scrollback on post-connect login when the login name differs from the pre-connect character.

For scrollback disposition, the character name is the BatMUD login name held in session state: snapshotted at Connect Command time and compared again at the first successful login after reconnect. Same character is decided by case-insensitive ASCII equality; connect before any login clears output on first login.

## Combat Awareness

Combat Awareness is batrs' interpretation of whether the player is currently in BatMUD combat. It begins when combat round output is observed and ends when BatMUD reports that the player is not in combat.

A Combat Scan Snapshot is the latest observed set of combatants and their health from a completed scan result. Each completed scan result replaces the previous snapshot rather than appending to it.

Combat Awareness owns canonical round-header and combat-end line matching, probe orchestration, and snapshot state in `src/combat_awareness.rs`. The application calls Combat Awareness once per incoming line and fans out `CombatAwarenessEffect` values: `RoundStarted` (stats round semantics and `in_battle`), `CombatEnded` (stats end-combat and clear `in_battle`), `SendShortScore` (`@sc`), and `SendProbe` (`#scan all`). Stats retains short-score round diff semantics; the UI layer renders combat status rows from snapshot data via `ui::render_combat_status_lines`.

## Secondary Status

Secondary Status is the guild-specific HUD row band rendered below the main stats line. It covers Animist soul companion, Riftwalker entity, Tzarakk mount, and Nergal resource status plus minions.

Secondary Status owns guild HUD state, `SecondaryStatusEffect` application, guild-selected rendering via `render_lines`, and lifecycle: clear stored state for deselected guilds (`sync_guild_selection`) and reset on Connect Command (`FreshSessionReset::SecondaryStatus`). Guild trigger modules emit `SecondaryStatusEffect` values; the application applies them separately from stats effects.

A guild HUD row renders only when that guild's `GuildKey` is in the player's guild selection. Deselecting a guild clears its stored secondary status immediately. Stats retains prompt, short score, recovery brackets, and combat-round diff semantics only.

## Nergal Status

Nergal Resource Status is the player's current Nergal-specific resource state: Vitae, Potentia, and Evolution points.

The Nergal guild module owns parsing and gagging of the Nergal resource status line. Parsing runs only when `GuildKey::Nergal` is in the player's guild selection. Secondary Status owns `NergalResourceStatus` and minion storage, Nergal `SecondaryStatusEffect` variants, and Nergal HUD rendering. The application shows Nergal HUD rows only when Nergal is selected; deselecting Nergal clears Nergal secondary status via `sync_guild_selection`.
