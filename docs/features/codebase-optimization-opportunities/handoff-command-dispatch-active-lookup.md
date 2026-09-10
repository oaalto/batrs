# Handoff: command-dispatch-active-lookup

**Implementation knowledge from `command-dispatch-active-lookup`** for the sibling
`main-loop-redraw-poll-narrowing` ticket (and any future work touching the app input loop / dispatch).

## Contracts / seams

- `command::dispatch` now takes `guild_command_lookup: &HashMap<String, Command>` in addition to
  `guilds: &[Box<dyn Guild>]`. The lookup is a prebuilt merged guild-command map; `guilds` is still
  required for `/show` (iterates active guild triggers/shortcuts).
- `command::build_guild_command_lookup(&guilds)` builds the merged map. First selected guild wins per
  alias (`or_insert`), identical to prior per-dispatch behavior.
- `BatApp` owns `guild_command_lookup`, rebuilt only in `apply_guild_selection` and on
  `FreshSessionReset::GuildSelection`. Keep these two mutation points in sync; never rebuild the map
  inline per dispatch in `submit_input`.

## Decisions that changed the plan

- None diverged. Kept the two-param `dispatch` signature (no struct wrapper) per PRD "favor deletion,
  reuse, or narrower data flow over new abstraction"; `guilds` stays because `/show` needs it.

## Traps / gotchas

- `GuildSelection::from_playable_keys` and `from_persisted_keys` both filter to `is_playable()`.
  `background_only` guilds (e.g. `GoodReligious`) can never be selected and never own active commands.
  Do not write app-level dispatch tests that select a `background_only` guild via these builders.
  The good_religious dispatch tests drive `command::dispatch` directly with ad-hoc guild vectors
  instead.

## File / API map

- `src/command/mod.rs` — `dispatch`, `build_guild_command_lookup`.
- `src/app/mod.rs` — `BatApp.guild_command_lookup` field; rebuilt in `apply_guild_selection` and
  `FreshSessionReset::GuildSelection`; both `dispatch` call sites in `submit_input` pass
  `&self.guild_command_lookup`.
- Regression test: `app::tests::guild_command_lookup_refreshes_on_guild_selection_change`
  (Tzarakk `med` → `@dismount;use 'meditation'` vs Tiger `med` → `@use 'meditation'` proves cache refresh).