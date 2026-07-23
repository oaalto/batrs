# 01 — Extract Secondary Status module from Stats

## Parent

`prd.md`

## What to build

Move all guild HUD rows below the main stats line (Animist soul, Riftwalker entity, Tzarakk mount, Nergal resource + minions) out of Stats into a **Secondary Status** module with its own effect path, rendering, and lifecycle. Guild triggers emit `SecondaryStatusEffect`; the application applies them separately from stats effects. HUD rows show only when the matching guild is selected; deselecting a guild clears its stored state via `sync_guild_selection`. Connect Command resets Secondary Status via `FreshSessionReset::SecondaryStatus`. Draw calls one `render_lines(width, guild_selection)`. Include any Nergal duplicate core-trigger cleanup from the superseded Nergal ownership PRD. Land as one green slice — no phased dual-ownership in Stats.

## Blocked by

None — can start immediately.

## Status

ready-for-agent

## Acceptance criteria

- [ ] Secondary Status module owns guild HUD state types, `SecondaryStatusEffect`, `apply_effect`, `render_lines(width, &GuildSelection)`, and `sync_guild_selection`
- [ ] `TriggerEffects.secondary_status` vector and builder helper; guild triggers (Animist, Riftwalker, Tzarakk, Nergal) emit `SecondaryStatusEffect` instead of guild `StatsEffect` variants
- [ ] All guild HUD variants removed from `StatsEffect`; stats no longer stores or renders soul, mount, entity, or Nergal HUD data
- [ ] Application owns `secondary_status` field; `apply_secondary_status_effects` on incoming trigger results; `draw` uses single `render_lines` call (no per-guild guards in draw)
- [ ] Guild-selected-only rendering for all four domains; OR “show when data observed but guild deselected” removed for soul, Riftwalker, Tzarakk
- [ ] `sync_guild_selection` clears state for deselected guilds; application removes per-guild Nergal-only clear blocks from guild selection application
- [ ] `FreshSessionReset::SecondaryStatus` added to connect manifest; fresh session clears secondary status
- [ ] Nergal resource parsing remains guild-module-only; remove duplicate core/global Nergal resource trigger if still registered
- [ ] Nergal resource line gagged when Nergal selected; not applied to HUD when Nergal not selected
- [ ] Prompt and short-score updates do not clear secondary status (decoupled from stats update paths)
- [ ] Relevant unit tests migrated from stats to Secondary Status; guild trigger and application integration tests updated
- [ ] `cargo test` passes
