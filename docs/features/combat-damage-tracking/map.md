# Combat damage tracking — wayfinder map

`wayfinder:map`

**Spec:** [`prd.md`](prd.md) — implementation slices [`01`](01-damage-line-matcher.md) · [`02`](02-damage-collector-and-storage.md) · [`03`](03-http-damage-viewer.md) · [`04`](04-unattributed-hp-review.md) · [`05`](05-riposte-skill-matcher.md) · [`06`](06-unattributed-review-state.md) · [`07`](07-unattributed-drill-navigation.md)

## Destination

Live incoming-damage tracking while playing: batrs records HP-loss events into a single global SQLite database at `~/.batrs/combat_damage.db` (player name and other session facts as metadata only). On batrs startup a small HTTP server (default port 6464, `127.0.0.1`) serves the accumulated data in a readable dashboard. Parsing targets the **battle listen** hit-message format catalogued in [`docs/hit_messages.md`](../../hit_messages.md), with melee separated from skill/spell/special incoming lines; misses produce no events. Attribution uses a **weight** reflecting how confidently a context line caused the HP delta (isolated hits between status lines weigh higher than crowded buffers).

## Notes

- Domain: [`CONTEXT.md`](../../../CONTEXT.md), [`docs/hit_messages.md`](../../hit_messages.md)
- Prior analysis: `~/.batrs/COMBAT_DAMAGE_ANALYSIS.md`, `~/.batrs/analyze_combat_damage.py` (reference only — old log format)
- Skills: `wayfinder`, `domain-modeling`, `grilling`, `prototype`, `workflow`, `ponytail`
- Standing preferences: global DB not per-player; lazy/minimal deps; no log backfill in v1 (live only)

## Decisions so far

- [What counts as an incoming damage event?](tickets/what-counts-as-an-incoming-damage-event.md) — row on negative `H:` HP bracket only; HP-only delta; attributed line stored (no buffer on attributed rows); `melee`/`skill`/`spell` categories; unattributed loss captured separately (slice 04); no round/session ids in v1
- [How should attribution weight work?](tickets/how-should-attribution-weight-work.md) — isolated vs ambiguous by filtered candidate count; `weight` 1.0 or 1/N; `damage_min`/`damage_max` stored per row; aggregation by category+verb; confirmed vs estimated viewer modes; conservative extrapolation
- [How should catalog rank inform estimated extrapolation?](tickets/how-should-catalog-rank-inform-estimated-extrapolation.md) — `catalog_rank` from `hit_messages.md` order; schema v2 `catalog_rank` + `weapon_family` on melee rows; rank-proportional estimated avg only; `weight` unchanged; confirmed view unchanged
- [How should events be stored?](tickets/how-should-events-be-stored.md) — flat `damage_events` + `schema_version` at `~/.batrs/combat_damage.db`; `batch_id` per trigger; `rusqlite` + transaction per trigger; four indexes; inline migrations from v1; unlimited retention
- [Where should collection live in batrs?](tickets/where-should-collection-live-in-batrs.md) — `combat_damage` module on `BatApp`; early `handle_line` before CA gag; shared `SC_REGEX`; buffer reset via `FreshSessionReset` + login transition; DB at init; always on; log-and-continue on write failure
- [What non-melee incoming patterns exist?](tickets/what-non-melee-incoming-patterns-exist.md) — v1 skills: bash, push, kick, stab, scythe swipe (multi-regexp each); spells: `A/An <name> hits you.`; environmental/DoT/player-name/outgoing ignored; breath via melee catalog only
- [How should melee parsing use hit messages?](tickets/how-should-melee-parsing-use-hit-messages.md) — `build.rs` catalog from `hit_messages.md`; longest-first suffix match + family recency; dual conjugation suffix; skills→spells→melee; 35 tests covering 286 verbs + fixtures
- [What should the HTTP viewer show?](tickets/what-should-the-http-viewer-show.md) — three verb tables (melee/skill/spell) with confirmed+estimated columns; sortable columns; bundled CSS; verb drill-down with batch grouping; time+player filters; axum HTML auto-start on batrs launch (`--port` default 6464); no charts, no total/by-monster aggregates
- [How should unattributed HP loss be captured?](tickets/how-should-unattributed-hp-loss-be-captured.md) — zero-candidate negative `H:` only; context window (all non-`H:` lines including gagged); `unattributed_hp_events` table (schema v4); no `damage_events` / no `unknown` category; HTTP unattributed section + drill-down; always on
- [How should unattributed review state be tracked?](tickets/how-should-unattributed-review-state-be-tracked.md) — `reviewed_at` on drill-down open (schema v5); landing Reviewed column + unreviewed count; write-once idempotent mark; no collector changes
- [How should unattributed drill navigation work?](tickets/how-should-unattributed-drill-navigation-work.md) — Previous/Next on `/unattributed/{id}` within filtered landing order; omit at ends; preserve `range`/`player`; mark reviewed on nav GET

## Not yet specified
- Opt-out toggle and performance guardrails (write batching, max buffer size)

## Out of scope

- **Backfill import** of existing `~/.batrs/*/logs/*.log` into the DB (live capture only for v1; user chose destination A without batch import)
- **Per-player databases** — one global `combat_damage.db`; player is row metadata
- **Outgoing damage analytics** — player hits on monsters (`You puncture …`) are context for melee catalog validation, not incoming-damage events unless explicitly reframed later
- **Old battle-listen log format** — parsers target the new listen format only
