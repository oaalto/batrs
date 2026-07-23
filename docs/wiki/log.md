# Engineering Wiki Log

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

## [2026-07-23] update | Combat Awareness cohesion (ticket 01)

- Updated: [Combat Awareness](concepts/combat-awareness.md), `CONTEXT.md`, `docs/wiki/path-map.json`
- Sources: `src/combat_awareness.rs`, `src/app/mod.rs`, `CONTEXT.md`
- Notes: Replaced `combat_scan` / `combat_round` references with unified Combat Awareness module and app fan-out adapter.
