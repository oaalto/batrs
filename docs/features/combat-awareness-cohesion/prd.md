# Combat Awareness cohesion

## Status

ready-for-agent — grilled 2026-07-23

## Problem Statement

Combat Awareness is split across multiple modules with no single interface. Round detection, combat-end detection, probe orchestration, snapshot parsing, and UI rendering each live in different places. The combat-end BatMUD line is duplicated in several matchers. Ending combat can be applied through multiple paths on one incoming line (scan handler, then trigger pipeline), and probe sending is coupled to stats-effect handling in the application shell.

A maintainer tracing "what happens when combat ends?" must read combat scan state, combat round triggers, common trigger rules, application input processing, stats effects, and guild-specific matchers. Recent combat-scan work keeps touching this scatter without a single locality for fixes.

## Solution

Deepen Combat Awareness into one module with a small interface: observe incoming game lines, expose combat-active state, expose the latest Combat Scan Snapshot, and emit `CombatAwarenessEffect` values for the application to fan out. The application shell becomes a thin adapter that maps those effects to stats round semantics, automation flags, outbound probe and short-score commands, line gagging, and UI snapshot data. Rendering moves to the UI layer; stats keeps short-score diff semantics.

Triggers and guild modules call into Combat Awareness for canonical round-header and combat-end matching instead of duplicating regexes or literals. Combat lifecycle is reported once per line through the single CA → app fan-out path.

## User Stories

1. As a batrs maintainer, I want one module to own Combat Awareness, so that combat bug fixes have a single locality.
2. As a batrs maintainer, I want the combat-end BatMUD line defined once, so that text changes do not require a repo-wide search.
3. As a player, I want combat scan snapshots to update reliably each round, so that enemy health in the HUD matches `#scan all` output.
4. As a player, I want probe traffic gagged from my scrollback, so that automation and display stay clean.
5. As a test author, I want to exercise combat start, probe, snapshot, and end through one interface, so that integration tests do not require the full application shell.
6. As a maintainer adding a combat feature (e.g. scan cadence change), I want probe policy behind the Combat Awareness interface, so that the application shell does not grow more combat branches.
7. As a maintainer, I want Combat Awareness to report combat-ended once per line, so that stats and automation are not double-updated through parallel paths.
8. As a player, I want short-score combat diffs to behave as today when rounds start and end, so that deepening does not regress the stats HUD.
9. As a player, I want `@sc` sent automatically when a combat round starts, so that short-score diffs update without manual input.
10. As a player, I want the `in_battle` automation flag set on round start and cleared on combat end, so that combat automation rules fire correctly.
11. As a maintainer, I want monk kata interrupt logic to remain guild-specific while sharing the canonical combat-end line, so that guild behavior does not leak into Combat Awareness.
12. As a maintainer, I want combat status rows rendered from snapshot data in the UI layer, so that Combat Awareness stays free of presentation concerns.
13. As a player, I want probe cadence preserved (initial probe on round start, periodic probe on user game commands while idle), so that scan snapshots stay current without spamming BatMUD.
14. As a maintainer, I want Connect Command fresh-session reset to clear Combat Awareness state, so that reconnect does not show stale combat HUD.
15. As a test author, I want regression tests for double-apply on combat end, probe gagging, and short-score diff behavior across round boundaries, so that cohesion work does not reintroduce known bugs.

## Implementation Decisions

### Module boundary

- Introduce a top-level **Combat Awareness** module promoted from the existing combat scan probe state machine.
- Combat Awareness owns: canonical round-header regex, canonical `NOT_IN_COMBAT_LINE`, probe phase machine, snapshot parsing and storage, combat-active state, and `CombatAwarenessEffect` emission.
- Combat Awareness does **not** own: short-score diff accumulation (`combat_round_active` stays in stats), ratatui rendering, automation flag mutation, or guild-specific kata interrupt behavior.

### Public interface (behavioral contract)

Combat Awareness exposes roughly:

- `handle_incoming_line(line) → LineDisposition` — classifies probe echo rows, organic combat lines, and combat-end; may emit effects and indicate whether the line should be gagged or shown.
- `observe_user_game_command() → Option<ProbeCommand>` — idle-phase probe cadence (every second user game command while combat active).
- `snapshot() → &[CombatScanRow]` — latest completed scan rows (name, condition phrase, health percent).
- `is_active() → bool` — whether combat is currently considered active.
- Exported constants: `NOT_IN_COMBAT_LINE`, round-header matcher, `PROBE_COMMAND` (`#scan all`).

`CombatAwarenessEffect` variants (minimum):

```rust
enum CombatAwarenessEffect {
    RoundStarted,
    CombatEnded,
    SendProbe,      // #scan all
    SendShortScore, // @sc — folded into app fan-out on RoundStarted
}
```

Triggers and guild code import canonical matchers/constants from Combat Awareness rather than redefining them.

### Application fan-out adapter

The application shell maps `CombatAwarenessEffect` to downstream concerns in one place:

| Effect | Fan-out |
| --- | --- |
| `RoundStarted` | stats `StartCombatRound`; send `@sc`; send `#scan all`; `in_battle = true` |
| `CombatEnded` | stats `EndCombat`; clear snapshot / end combat state; `in_battle = false` |
| `SendProbe` | send `#scan all` |
| `SendShortScore` | send `@sc` (if not always bundled with `RoundStarted`) |

Delete the separate global combat-end application path that duplicates stats and automation updates. One CA call per line, one fan-out.

### Trigger pipeline changes

- **Delete** the dedicated combat-round trigger module; round and combat-end detection route through Combat Awareness canonical matchers.
- **Strip** from common trigger rules: round-header regex with `in_battle` SetFlag, combat-end SetFlag, and `@sc` on round start (moved to app fan-out).
- Trigger pipeline may still run for other effects on the same line; Combat Awareness owns lifecycle effects exclusively.

### Rendering

- Combat Awareness exposes snapshot data only (`CombatScanRow` or equivalent).
- UI module gains combat status rendering (condition coloring, width wrapping) from snapshot data.
- Application `draw` passes rendered lines into the view model as today.

### Guild: monk kata interrupt

- Monk guild keeps kata-interrupt behavior when the combat-end line appears.
- Monk imports the canonical `NOT_IN_COMBAT_LINE` / matcher from Combat Awareness; no duplicate literal or regex.

### Deletions and migrations

- Delete the application-local combat scan module after promotion.
- Delete the combat-round trigger module.
- Delete `apply_global_combat_end` and combat-specific branches inside stats-effect application in the application shell.
- Move combat status rendering out of Combat Awareness into the UI layer.

### Ordering and idempotency

- Preserve existing ordering: combat-end from an internal probe response must still clear state when the probe returns "not in combat."
- Combat-ended effects must be idempotent within a single line — fan-out runs once even if multiple subsystems would previously have reacted.
- Preserve probe gagging: internal probe echo and captured scan rows stay out of scrollback and automation input.

### Probe cadence (unchanged)

- Initial `#scan all` on round start.
- Additional `#scan all` on every second non-empty user game command while probe phase is idle and combat is active.
- Gag internal probe lines (`scan all` echo, captured rows, probe-returned combat-end when internal).

## Testing Decisions

### Primary test seam

Test through the **Combat Awareness public interface** at the highest seam — one module boundary, not internal phase enum names or application wiring details.

### Good tests (external behavior)

- Round header → `RoundStarted` + probe command effect; combat becomes active.
- Probe echo and scan rows → snapshot populated; lines marked for gagging.
- Combat-end line (organic and probe-returned) → `CombatEnded`; snapshot cleared; combat inactive.
- User game commands during combat → probe cadence (every second command).
- Next round header while capturing → prior scan completes into snapshot.
- Combat-end reported once per line when fan-out adapter is exercised in application integration tests.

### Prior art

- Existing combat scan unit tests (probe phases, snapshot capture, combat-end from probe).
- Application integration tests: round start correlates probe before short score; correlated scan rows gagged; global not-in-combat clears scan and stops diff accumulation.

### Regression focus

- Double-apply on combat end (stats + automation).
- Probe gagging and automation exclusion.
- Short-score diff behavior across round boundaries (stats tests + application integration).
- Monk kata interrupt still fires on combat-end line.

### Avoid

- Testing internal `ProbePhase` enum transitions by name.
- Snapshot-testing ratatui spans unless following an established UI test pattern.

## Out of Scope

- Changing BatMUD scan output format or probe command (`#scan all`).
- New combat automation or guild combat triggers unrelated to awareness.
- Reworking short-score parsing or recovery bracket logic.
- Combat features not yet in batrs (target selection, aggro tracking).
- Moving `combat_round_active` or diff accumulation into Combat Awareness.
- Secondary guild HUD status extraction (separate PRD).

## Further Notes

- Recommendation strength: **Strong** — highest bug surface among cohesion candidates.
- Related wiki concept: Combat Awareness (`docs/wiki/concepts/combat-awareness.md`) — update when implementation lands.
- `CONTEXT.md` Combat Awareness section updated with `CombatAwarenessEffect`.
- Grilling decisions: hybrid seam (C), own effect type (B), UI renders snapshot (B), app maps automation (A), monk imports constant (A), promote-and-delete (A), `@sc` in app fan-out on `RoundStarted`.
