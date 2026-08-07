# Engineering Wiki Log

## [2026-08-07] update | Unattributed drill navigation (slice 07)

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md), `CONTEXT.md`, `docs/features/combat-damage-tracking/prd.md`, `docs/features/combat-damage-tracking/map.md`
- Created: `docs/features/combat-damage-tracking/07-unattributed-drill-navigation.md`, `docs/features/combat-damage-tracking/tickets/how-should-unattributed-drill-navigation-work.md`
- Notes: Previous/Next on `/unattributed/{id}` within filtered landing order; preserve `range`/`player`; mark reviewed on nav GET.

## [2026-08-07] update | Unattributed review state (slice 06)

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md), `CONTEXT.md`, `docs/features/combat-damage-tracking/prd.md`, `docs/features/combat-damage-tracking/map.md`
- Created: `docs/features/combat-damage-tracking/06-unattributed-review-state.md`, `docs/features/combat-damage-tracking/tickets/how-should-unattributed-review-state-be-tracked.md`
- Sources: grill-with-docs-batch acceptance; `src/combat_damage/storage.rs`, `aggregate.rs`, `viewer.rs`
- Notes: schema v5 `reviewed_at`; auto-mark on `/unattributed/{id}` drill-down; landing Reviewed column + unreviewed count.

## [2026-08-07] update | Riposte skill matcher (slice 05)

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md), `CONTEXT.md`, `docs/features/combat-damage-tracking/prd.md`, `docs/features/combat-damage-tracking/tickets/what-counts-as-an-incoming-damage-event.md`, `docs/features/combat-damage-tracking/map.md`
- Created: `docs/features/combat-damage-tracking/05-riposte-skill-matcher.md`
- Sources: grill-with-docs-batch acceptance; `src/combat_damage/matcher.rs`, `collector.rs`
- Notes: two-line enemy riposte (`parries.` → `...AND counterattacks.` / `...AND ripostes.`); `skill` / `riposte`; matcher reset on `H:` flush and `reset_buffer()`.

## [2026-08-07] update | Unattributed HP loss implementation (slice 04)

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md) (if needed), `docs/features/combat-damage-tracking/04-unattributed-hp-review.md`
- Sources: `src/combat_damage/storage.rs`, `collector.rs`, `aggregate.rs`, `viewer.rs`
- Notes: schema v4 `unattributed_hp_events`; dual-buffer collector; HTTP `/unattributed/{id}` drill-down; landing unattributed section with `range`/`player` filters.

## [2026-08-07] update | Unattributed HP loss review capture

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md), `CONTEXT.md`, `docs/features/combat-damage-tracking/prd.md`, `docs/features/combat-damage-tracking/map.md`
- Created: `docs/features/combat-damage-tracking/tickets/how-should-unattributed-hp-loss-be-captured.md`, `docs/features/combat-damage-tracking/04-unattributed-hp-review.md`, `docs/adr/0002-unattributed-hp-loss-outside-damage-events.md`
- Sources: grill-with-docs-batch acceptance (unattributed context window plan)
- Notes: zero-candidate negative `H:` triggers save context window to `unattributed_hp_events` (schema v4); no `damage_events` / no `unknown` category; HTTP unattributed section planned in slice 04.

## [2026-08-06] update | Melee landing weapon-family sub-sections

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md), `CONTEXT.md`, `docs/features/combat-damage-tracking/prd.md`, `docs/features/combat-damage-tracking/03-http-damage-viewer.md`, `README.md`
- Created: `docs/adr/0001-melee-aggregation-includes-weapon-family.md`
- Sources: grill-with-docs-batch acceptance; `src/combat_damage/aggregate.rs`, `src/combat_damage/viewer.rs`, `build.rs`
- Notes: melee landing grouped by `weapon_family`; aggregation key includes family; drill-down `?family=` param; `FAMILY_TITLES` from `hit_messages.md` headers.

## [2026-08-06] update | Combat damage drill-down batch siblings

- Updated: [Combat Damage Tracking](concepts/combat-damage-tracking.md), `CONTEXT.md`, `docs/features/combat-damage-tracking/03-http-damage-viewer.md`
- Sources: grill-with-docs-batch acceptance; `src/combat_damage/aggregate.rs`, `src/combat_damage/viewer.rs`
- Notes: verb drill-down shows inline batch siblings for ambiguous attribution batches; cross-category siblings included.

## [2026-08-06] update | Combat damage catalog rank

- Created: [Combat Damage Tracking](concepts/combat-damage-tracking.md)
- Updated: `CONTEXT.md`, `docs/hit_messages.md`, `docs/features/combat-damage-tracking/prd.md`, `docs/features/combat-damage-tracking/map.md`, weight ticket addendum
- Sources: grill-with-docs-batch acceptance; `docs/hit_messages.md`, `docs/features/combat-damage-tracking/`
- Notes: `catalog_rank` vs attribution `weight`; rank-estimated avg for loose ambiguous batches; schema v2 `catalog_rank` + `weapon_family` on melee rows (planned).

## [2026-08-05] update | Monk chain rotation wrap

- Updated: [Monk Skill Tracks](concepts/monk-skill-tracks.md)
- Sources: `src/guilds/monk/triggers.rs`, `src/guilds/monk/skills_config.rs`
- Notes: two-slot chains now wrap rotation to the first enabled skill after a full success on the last enabled slot (was skipping `SetVar` when the next slot was disabled).

## [2026-08-05] update | `/show` command

- Updated: [Command Dispatch](concepts/command-dispatch.md), `CONTEXT.md`, `docs/manual/commands.md`
- Sources: `src/command/show.rs`, `src/command/catalog.rs`, `src/guilds/*/commands.rs`, `src/guilds/*/triggers.rs`
- Notes: login-gated `/show commands|triggers [guild|generic]`; introspection metadata co-located with guild registrations.

## [2026-08-05] update | Monk skill tracks and `/monk` dialog

- Created: [Monk Skill Tracks](concepts/monk-skill-tracks.md)
- Updated: [Player Profile](concepts/player-profile.md), `CONTEXT.md`, `docs/guilds/monk.md`, `docs/manual/ui.md`
- Sources: `src/guilds/monk/skills_config.rs`, `src/app/dialogs/monk_dialog.rs`, `src/config.rs`
- Notes: `[monk_skills]` profile section; prefix-chain UI; rotation gating vs line coloring split.

## [2026-08-04] ingest | Combat scan status suffix

- Updated: [Combat Awareness](concepts/combat-awareness.md)
- Sources: `CONTEXT.md`, `src/combat_awareness.rs`, `src/ui/mod.rs`
- Notes: `#scan all` rows may include an optional `and <status>` suffix; HUD renders it as `[status]`.

## [2026-07-31] skip | Guild dialog save primary on drill open

- Touched: `src/app/dialogs/guild_dialog.rs`
- Reason: bugfix only; no durable wiki concept change.

## [2026-07-31] update | Multi-background guild drill filtering

- Updated: [Guild Catalog](concepts/guild-catalog.md), [Guild Background Map](concepts/guild-background-map.md)
- Sources: `src/guilds/grouping.rs`, `src/guilds/catalog/browse.rs`
- Notes: `/guilds` multi-background section filters by thematic eligibility rules from the background map.

## [2026-07-31] ingest | Guild background map and stub guilds

- Created: [Guild Background Map](concepts/guild-background-map.md)
- Updated: [Guild Catalog](concepts/guild-catalog.md)
- Sources: `CONTEXT.md`, `src/guilds/catalog/mod.rs`, `src/guilds/stub.rs`, BatMUD background→guild reference
- Notes: Documented thematic and multi-background membership; background-only auto-injection for all five themes; stub implementations for previously unimplemented catalog entries.

## [2026-07-31] skip | Good Religious guild spells

- Notes: Background-only guild auto-injection and spell shortcuts documented in `CONTEXT.md`; no wiki concept page updates in this slice.

## [2026-07-28] skip | Code hygiene cleanup backlog close-out

- Notes: Jul 2026 audit slices (stats test gating, telnet UTF-8, config/logging migration, companion cache, LazyLock) verified shipped in code; PRD and tickets marked done. No wiki concept updates required.

## [2026-07-28] skip | Move guild wrappers to Abilities backlog close-out

- Notes: Feature shipped in `a7c2f26`; PRD and ticket status aligned to done. Wiki ingest already recorded 2026-07-27.

## [2026-07-27] ingest | Move guild wrappers to Abilities

- Updated: [Abilities](concepts/abilities.md), [Guild Catalog](concepts/guild-catalog.md)
- Sources: `CONTEXT.md`, `src/abilities/mod.rs`, `docs/features/move-guild-wrappers-to-abilities/prd.md`
- Notes: Documented Abilities bounded context, targeted-command formatting rules, and guild→Abilities import seam.

## [2026-07-27] update | Configurable trigger chain

- Updated: [Command Dispatch](concepts/command-dispatch.md), [Player Profile](concepts/player-profile.md)
- Sources: `CONTEXT.md`, `src/command/mod.rs`, `src/triggers/mod.rs`, `src/player_profile.rs`, `docs/features/configurable-trigger-chain/prd.md`
- Notes: Documented `/triggers`, `[triggers]` player profile section, fixed pipeline order, and in-session save semantics.

## [2026-07-27] skip | Extract trigger rule engine

- Notes: Internal module extraction and Animist companion gating behavior change; no durable wiki concept updates required.

## [2026-07-24] update | Clear Command documentation sweep (ticket 02)

- Updated: [Command Dispatch](concepts/command-dispatch.md)
- Sources: `CONTEXT.md`, `docs/manual/commands.md`, `docs/features/clear-command/prd.md`, `src/command/mod.rs`
- Notes: Documented `/clear` as client-only terminal redraw (not output-buffer wipe); aligned domain vocabulary, player manual, and wiki command-dispatch concept with PRD semantics.

## [2026-07-23] update | Guild Catalog browse extraction (ticket 03)

- Updated: [Guild Catalog](concepts/guild-catalog.md)
- Sources: `src/guilds/catalog/browse.rs`, `src/app/dialogs/guild_dialog.rs`, `CONTEXT.md`
- Notes: Browse submodule ownership documented; dialog no longer implied as owner of drill row topology.

## [2026-07-23] ingest | Initial wiki bootstrap

- Updated: [batrs client application](subsystems/batrs-client.md), [Command Dispatch](concepts/command-dispatch.md), [Guild Catalog](concepts/guild-catalog.md)
- Sources: `CONTEXT.md`, `src/main.rs`, `src/app/mod.rs`, `src/command/mod.rs`, `README.md`
- Notes: Greenfield ADC post-install; first substantive wiki pages from live repo sources.

## [2026-07-23] update | Wiki automation setup

- Updated: `docs/wiki/path-map.json`
- Sources: `src/` layout, `scripts/wiki-lint.mjs`
- Notes: Added subsystem mappings for `src/app`, `src/command`, `src/guilds`. Node `wiki-lint.mjs` kept as reference; no git-hook wiring yet (repo has no Husky/pre-commit).

## [2026-07-23] ingest | CONTEXT.md concept pages

- Updated: [Combat Awareness](concepts/combat-awareness.md), [Nergal Status](concepts/nergal-status.md), [Player Profile](concepts/player-profile.md)
- Sources: `CONTEXT.md`, `src/app/combat_scan.rs`, `src/triggers/combat_round.rs`, `src/player_profile.rs`, `src/config.rs`, `src/triggers/nergal_resource_status.rs`
- Notes: Post-install follow-up; remaining CONTEXT.md concepts now indexed.

## [2026-07-23] update | Combat status UI rendering (ticket 02)

- Updated: [Combat Awareness](concepts/combat-awareness.md), `CONTEXT.md`
- Sources: `src/combat_awareness.rs`, `src/ui/mod.rs`, `src/app/mod.rs`
- Notes: Combat status presentation moved to UI layer; domain exposes snapshot data only.

## [2026-08-05] update | Death line combat-end detection

- Updated: [Combat Awareness](concepts/combat-awareness.md), `CONTEXT.md`
- Sources: `src/combat_awareness.rs`, `src/guilds/monk/triggers.rs`, `src/triggers/common.rs`
- Notes: Added `DEATH_COMBAT_END_LINE` and `is_combat_end_line`; monk kata interrupt and lich drain use both canonical combat-end lines.

## [2026-07-23] update | Combat Awareness cohesion (ticket 01)

- Updated: [Combat Awareness](concepts/combat-awareness.md), `CONTEXT.md`, `docs/wiki/path-map.json`
- Sources: `src/combat_awareness.rs`, `src/app/mod.rs`, `CONTEXT.md`
- Notes: Replaced `combat_scan` / `combat_round` references with unified Combat Awareness module and app fan-out adapter.

## [2026-07-23] update | Combat Awareness docs + stale reference sweep (ticket 03)

- Updated: [Combat Awareness](concepts/combat-awareness.md)
- Sources: `src/combat_awareness.rs`, `src/app/mod.rs`, `src/ui/mod.rs`, `src/guilds/monk/triggers.rs`, `src/triggers/common.rs`
- Notes: Structured module boundary, effect fan-out table, and UI rendering seam; verified canonical `NOT_IN_COMBAT_LINE` imports in monk/common; added app regression test for single combat-end fan-out per line.

## [2026-07-23] update | Nergal guild-gated HUD lifecycle (ticket 02)

- Updated: [Nergal Status](concepts/nergal-status.md), `docs/wiki/path-map.json`
- Sources: `src/app/mod.rs`, `src/stats.rs`, `src/guilds/nergal/triggers.rs`
- Notes: HUD gated on guild selection only; deselect clears Nergal resource status and minions from stats.

## [2026-07-23] update | Nergal resource status ownership (ticket 01)

- Updated: [Nergal Status](concepts/nergal-status.md), `docs/wiki/path-map.json`
- Sources: `src/guilds/nergal/triggers.rs`, `src/triggers/mod.rs`, `src/app/mod.rs`
- Notes: Removed duplicate core trigger; guild module is sole parser when Nergal is selected.

## [2026-07-23] skip | Session Lifecycle login-name comparison

- Sources: `CONTEXT.md`, `src/app/session_lifecycle/output_disposition.rs`
- Notes: One-line CONTEXT.md clarification only; no wiki page change.

## [2026-07-23] update | Session Lifecycle extraction and scrollback disposition

- Created: [Session Lifecycle](concepts/session-lifecycle.md)
- Updated: [batrs client application](subsystems/batrs-client.md), [Command Dispatch](concepts/command-dispatch.md), [Player Profile](concepts/player-profile.md), `docs/wiki/index.md`, `docs/wiki/path-map.json`
- Sources: `CONTEXT.md`, `src/app/session_lifecycle/`, `src/app/mod.rs`
- Notes: Documented extracted bounded context, fresh-session reset manifest, reconnect guard, stale-event filtering, and same-character scrollback preservation on reconnect.

## [2026-07-23] update | Secondary Status extraction docs (ticket 02)

- Created: [Secondary Status](concepts/secondary-status.md)
- Updated: [Nergal Status](concepts/nergal-status.md), [Session Lifecycle](concepts/session-lifecycle.md), `docs/wiki/index.md`, `docs/wiki/path-map.json`, `docs/guilds/riftwalker.md`
- Sources: `src/secondary_status.rs`, `src/app/mod.rs`, `src/app/session_lifecycle/fresh_session.rs`, `src/guilds/*/triggers.rs`, `CONTEXT.md`
- Notes: Guild HUD ownership moved from stats to Secondary Status; Nergal and session-lifecycle pages updated; nergal-resource-status-ownership tickets marked superseded; stale riftwalker guild-selected-only wording fixed.
