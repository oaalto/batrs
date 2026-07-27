# Configurable Trigger Chain

## Status

done

## Problem Statement

The trigger processing pipeline always runs every built-in trigger group in a fixed order: guild triggers, spell vocals, common rules, and core rules (prompt, short score, recovery bracket). There is no way to disable a group while debugging, reduce highlight noise during focused play, or temporarily turn off prompt/stats parsing without editing code or restarting with a forked build.

Power users who already customize other per-character settings through the client cannot tune which trigger groups run for their character. The pipeline is opaque: guild selection in `/guilds` controls which guild modules are active, but there is no equivalent switch for the non-guild groups, and no single place to turn off all guild trigger modules without deselecting guilds.

## Solution

Add per-group enable/disable configuration for the four built-in trigger groups, persisted in the player profile TOML and editable in-session through a `/triggers` toggle dialog (patterned after `/genericcommands`). Execution order stays fixed; only on/off switches change. Changes take effect immediately on successful save without restart. Default configuration enables all groups, preserving today's behavior for existing players and omitted config.

## User Stories

1. As a player, I want to disable common trigger highlights, so that I can reduce visual noise while debugging other client features.
2. As a player, I want to disable guild trigger modules without changing my guild selection in `/guilds`, so that I can keep my character build configured while turning off guild-specific line effects.
3. As a player, I want to disable core triggers (prompt, short score, recovery bracket parsing), so that I can isolate whether stats or HUD issues come from core parsing versus guild or common rules.
4. As a player, I want to disable spell vocal triggers, so that I can turn off that highlight group independently of other groups.
5. As a player, I want all four built-in groups toggleable including core, so that I have full control over the fixed pipeline without needing a code change.
6. As a player, I want a `/triggers` slash command that opens a toggle dialog, so that I can change trigger settings without hand-editing TOML.
7. As a player, I want trigger settings saved to my player profile file, so that my choices persist across sessions for this character.
8. As a player, I want trigger changes to take effect immediately after I save in the dialog, so that I do not need to restart batrs to see the difference on incoming game lines.
9. As a player, I want the default configuration to enable all trigger groups, so that upgrading batrs does not change my current experience.
10. As a player, I want human-readable labels in the dialog (Guild triggers, Spell vocals, Common triggers, Core triggers), so that I understand what each toggle controls.
11. As a player, I want a warning when core triggers are turned off in the dialog, so that I know prompt and stats parsing will stop.
12. As a player, I want to cancel the dialog with Esc without saving, so that mistaken toggles do not apply.
13. As a player, I want Enter to save my toggles and close the dialog on success, so that the flow matches other profile dialogs.
14. As a player, I want Enter to close the dialog without writing when I did not change anything, so that I do not trigger unnecessary disk writes.
15. As a player, I want a clear error message when save fails, so that I know settings were not applied.
16. As a player, I want the dialog to stay open after a save failure, so that I can fix the issue or retry without reopening `/triggers`.
17. As a player, I want save errors to clear when I edit toggles again, so that stale error text does not linger after I change my mind.
18. As a player, I want `/triggers` to require login like `/genericcommands`, so that trigger settings are only edited when a character profile is loaded.
19. As a player, I want `/triggers` to load my player config if needed before opening, so that the dialog reflects my saved settings.
20. As a player, I want key help in the dialog footer matching `/genericcommands` (Up/Down, Space, Enter, Esc), so that navigation is consistent across profile dialogs.
21. As a player, I want to hand-edit `[triggers]` in player TOML, so that I can batch-change settings outside the client.
22. As a player, I want omitted `[triggers]` in TOML to mean all groups enabled, so that minimal config files stay small.
23. As a player, I want only disabled groups written back to TOML when I save non-default settings, so that my player file stays sparse.
24. As a player, I want guild selection in `/guilds` unchanged by trigger group toggles, so that guild build configuration and trigger execution are separate concerns.
25. As a player with guild triggers disabled, I want guild trigger modules skipped entirely (including secondary status effects from guild triggers), so that disabling the group is a real off switch.
26. As a player with guild triggers enabled, I want the same guild trigger behavior as today when my guilds are selected, so that the default path is unchanged.
27. As a maintainer, I want `process()` to accept trigger configuration and skip disabled groups via plain conditional gates, so that v1 stays simple without a trait registry.
28. As a maintainer, I want trigger configuration on the player runtime profile, so that the application reads one source of truth at the trigger call site.
29. As a maintainer, I want save to be atomic (persist first, then update in-memory profile on success only), so that a failed disk write never leaves runtime and disk out of sync.
30. As a maintainer, I want successful save to assign the new config directly to the runtime profile without re-reading the file, so that the hot path avoids redundant I/O.
31. As a maintainer, I want the dialog seeded from in-memory profile state on open, so that open is cheap and consistent with `/genericcommands`.
32. As a maintainer, I want hand-edited TOML to apply on the next profile load (login or explicit load path), so that external edits are not lost but do not require re-read on every dialog open.
33. As a test author, I want a unit test proving `process()` skips a disabled guild trigger group, so that the primary behavioral contract is enforced.
34. As a test author, I want serde round-trip tests for sparse trigger config, so that TOML shape and defaults stay correct.
35. As a test author, I want a persist test for saving trigger config, so that sparse write behavior is verified against real file I/O.
36. As a player, I want `/triggers` listed and reachable through Command Dispatch like other profile commands, so that discovery and gating follow existing patterns.
37. As a player, I want the dialog title to read "Triggers", so that the command name and UI title align.
38. As a player with core triggers off, I want the footer warning "Prompt/stats parsing disabled", so that the consequence of that toggle is explicit.
39. As a maintainer, I want fixed user-facing save error strings (not raw I/O errors), so that failures are readable in the TUI footer.
40. As a maintainer, I want missing player config manager on save to surface "Player config not available", so that the failure mode is distinguishable from write errors.
41. As a maintainer, I want failed persist to show "Failed to save trigger settings", so that players know the write did not succeed.
42. As a player, I want no "enable all groups" master row in the dialog, so that I toggle only the four concrete groups.
43. As a maintainer, I want all existing trigger tests updated to pass default config, so that CI continues to assert today's behavior under defaults.
44. As a maintainer, I want PRD and domain docs to reflect that in-session enable/disable is in scope while reordering and new groups are not, so that future work does not re-litigate v1 boundaries.

## Implementation Decisions

### Scope and pipeline behavior

- v1 supports **enable/disable per built-in group only**. Execution order is fixed: guild triggers → spell vocals → common triggers → core triggers. Reordering and adding new built-in groups are out of scope.
- v1 uses **plain conditional gates** around existing group calls in `process()`. No `TriggerGroup` trait or dynamic group registry in this slice; a longer-term trait-based design may be documented as deferred direction only.
- Disabling **guild triggers** skips all guild trigger module execution for the line, including secondary status effects emitted by guild triggers. Guild selection via `/guilds` is unchanged.
- Disabling **core triggers** skips prompt, short score, and recovery bracket processing for the line.
- Default `TriggerConfig` has all four booleans `true`, producing behavior identical to the current hardcoded pipeline.

### Configuration model

- Introduce `TriggerConfig` with fields: `guild_triggers`, `spell_vocals`, `common_triggers`, `core_triggers` (all `bool`, default `true`). TOML keys match these names under a `[triggers]` section on the player profile document.
- `TriggerConfig` lives in the triggers module with `Default`, `Serialize`, and `Deserialize`.
- **Sparse serialization (two layers):** omit the entire `[triggers]` section when config equals defaults (all true). Within a non-default section, serialize only keys whose value is `false` (enabled groups are omitted from disk).
- On deserialize, missing keys default to `true`.
- Configuration is stored on **player profile TOML only** (not user-wide settings). It is loaded into **player runtime profile** and read at the trigger `process()` call site. No separate parallel field on the application shell.

### Runtime integration

- Extend `process()` to accept `&TriggerConfig` (or equivalent) and wrap each group block in the corresponding gate.
- The application passes `&player_runtime_profile.trigger_config` (or equivalent) when invoking `process()` on each incoming line.
- Existing call sites and tests use `TriggerConfig::default()` unless explicitly testing disabled groups.

### Persistence and atomic save

- Add a config-manager save path for trigger settings (e.g. `save_trigger_config`) that writes the sparse `[triggers]` section into the player file, omitting the section when defaults are restored.
- **Atomic save semantics:** on dialog save, attempt persist first; update in-memory runtime profile only on `Ok`. On failure, runtime profile and disk remain at the last committed state.
- On successful save, assign the saved config directly to the runtime profile (no full profile re-load from disk).
- If the player config manager is unavailable after the login/load gate, treat as save failure with the fixed "Player config not available" message; dialog stays open.

### `/triggers` command and access gates

- Register `/triggers` as a builtin command with **`requires_login: true`**, mirroring `/genericcommands`.
- Opening the dialog: return early if not logged in; call user config load when profile config has not been loaded yet, then open.
- Handler opens the triggers toggle dialog; does not send text to BatMUD.

### Triggers dialog (UI)

- Clone the **generic commands toggle dialog** interaction model: list of rows, Up/Down to move, Space to toggle, Enter to save, Esc to cancel.
- **Title:** `Triggers`.
- **Rows (pipeline order):** `Guild triggers`, `Spell vocals`, `Common triggers`, `Core triggers`.
- **State:** at open, `saved = runtime_profile.trigger_config.clone()`, `draft = saved.clone()`. Space/Up/Down mutate `draft` only. Esc discards `draft` and closes. Enter: if `draft == saved`, close without I/O; else persist `draft`, on success update runtime profile and close, on failure keep dialog open.
- **Footer (two lines):** row 1 shows key help (`Up/Down: move  Space: toggle  Enter: save  Esc: cancel`, matching generic commands) or replaces it with a save error string; row 2 shows `Prompt/stats parsing disabled` when `draft.core_triggers` is false, otherwise hidden/blank.
- **Footer error state** lives on the dialog struct; cleared on Space or Up/Down after a failure.
- **Fixed error strings:** missing config manager → `Player config not available`; persist failure → `Failed to save trigger settings`. No raw I/O error text in the footer.
- View model exposes computed `footer_line1` and optional `footer_line2` for the renderer.
- No master "all groups" row.

### Deferred extensibility (documentation only)

- A future `TriggerGroup` trait and loop-over-groups design may remain documented as longer-term direction. v1 does not implement it.

### Type shape (from grill)

```rust
pub struct TriggerConfig {
    #[serde(default, skip_serializing_if = "is_true")]
    pub guild_triggers: bool,
    #[serde(default, skip_serializing_if = "is_true")]
    pub spell_vocals: bool,
    #[serde(default, skip_serializing_if = "is_true")]
    pub common_triggers: bool,
    #[serde(default, skip_serializing_if = "is_true")]
    pub core_triggers: bool,
}
// is_true(b) => *b; is_default() => all true
```

## Testing Decisions

### Test seam (primary)

**One primary seam:** the public `process()` entry point on the triggers module. Tests supply `TriggerFacts`, guild list, input line, and `TriggerConfig`; assert on `TriggerEffects` (line styling, secondary status, stats, etc.). This is the highest behavioral seam — it covers group gating without driving the full application event loop or TUI.

Confirm this seam matches expectations before implementation slices are cut.

### What makes a good test

- Assert **observable effects** (hilite applied or not, secondary status emitted or not), not internal branch counters or private helpers.
- Prefer **inverting existing fixtures** over new elaborate setups.
- Persistence and serde tests assert **document shape on disk and round-trip**, not dialog widget state.

### Test scope (grilled)

1. **Process gating (Q16/Q20):** With Animist guild active and a companion combat line fixture, assert companion hilite **does not** run when `guild_triggers = false` — inverse of the existing Animist companion hilite test. Same facts, guilds, and line; only config differs.
2. **Serde round-trip:** `TriggerConfig` / player profile document: defaults omit `[triggers]`; sparse section writes only `false` keys; deserialize restores expected booleans.
3. **Persist:** `save_trigger_config` (or equivalent) via a test-only config manager constructor pointing at a temp file under the system temp directory (unique filename; no new tempfile dependency). Write, read file back, assert sparse TOML content.

### Modules under test

- Triggers module (`process()` + `TriggerConfig` serde defaults).
- Config module (save path + test helper).
- Dialog and Command Dispatch: **no dedicated UI tests in v1** unless an existing dialog test pattern is trivial to extend; behavior is covered indirectly via save/persist and process gates.

### Prior art

- Existing `process_with_animist_applies_companion_combat_hilite` / `process_without_animist_skips_companion_combat_hilite` tests in the triggers module.
- Generic commands save/persist and dialog flow in the application and config layers.
- Sparse TOML patterns used elsewhere on the player profile document.

## Out of Scope

- **Runtime reordering** of trigger groups or inserting new built-in groups in v1.
- **Per-rule or custom-group editor** — no GUI to edit individual triggers or define new groups; only enable/disable of the four built-in groups via `/triggers` (TOML hand-edits for the same four keys remain valid).
- **`TriggerGroup` trait / dynamic registry** implementation in v1 (documented as deferred only).
- Changing the `Trigger` function signature or individual trigger implementations.
- Master "all groups" toggle row in the dialog.
- Re-reading player TOML from disk on every dialog open.
- Full save on Enter when draft equals saved (no-op close without I/O).
- Raw `io::Error` text in the dialog footer.
- User-wide / `UserSettings` storage for trigger config.
- Confirmation modal when disabling core triggers (footer warning only).

## Further Notes

- **PRD reconciliation (grilled):** In-session enable/disable via `/triggers` **is** in scope and takes effect on successful save without restart. This replaces the earlier non-goal wording that implied config was startup-only. Reordering and new groups remain non-goals.
- **Migration:** Add config types and gates first with default config at all call sites so existing tests pass unchanged; then wire profile load and dialog. No TOML migration required — absent section means all enabled.
- **Wiki / CONTEXT:** After implementation, update domain docs if trigger pipeline or player profile vocabulary changes; optional wiki log entry per project definition-of-done.
- **Slices:** Use `/to-tickets` to split config+persistence, `process()` gating, dialog/command, and PRD/doc sweep if desired.
