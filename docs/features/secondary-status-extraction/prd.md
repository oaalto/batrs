# Secondary status extraction from Stats

## Status

ready-for-agent — grilled 2026-07-23

## Problem Statement

Guild-specific HUD rows below the main stats line (Animist soul companion, Riftwalker entity, Tzarakk mount, Nergal resource status and minions) are stored, updated, and rendered inside the stats module, while the application shell assembles `secondary_status_lines` with per-guild visibility guards and multiple render calls. Stats therefore exposes an interface mixing unrelated concerns (prompt, short score, recovery brackets, combat-round semantics) with four guild HUD domains. UI layout logic (which rows appear and in what order) leaks into the application draw path.

Combat status was extracted above stats with dedicated state and a separate effect path; secondary status repeats the problem incompletely below stats — guild HUD state and rendering never left stats.

Additionally, soul, Riftwalker, and Tzarakk rows use “show if guild selected **or** data already observed” semantics, while Nergal uses guild-selected-only visibility and clear-on-deselect. That inconsistency leaves stale guild HUD data in memory after deselecting Animist, Riftwalker, or Tzarakk.

## Solution

Extract a **Secondary Status** module that owns all four guild HUD domains end to end: state, `SecondaryStatusEffect` application, guild-selected rendering, and lifecycle (clear on deselect, reset on Connect Command). Guild triggers emit `SecondaryStatusEffect` via a new `TriggerEffects.secondary_status` vector; stats sheds all guild HUD types and effect variants.

Tighten visibility for all guild HUD rows to **guild-selected only** (matching Nergal). Clear stored state immediately when a guild is deselected via `sync_guild_selection`.

Absorb the storage/render scope from the Nergal Resource Status ownership feature: include Nergal parsing cleanup (remove any duplicate core resource trigger if present) and move Nergal HUD state into Secondary Status in one vertical slice.

The application draw path calls a single `render_lines(width, guild_selection)` on Secondary Status. `ViewModel.secondary_status_lines` keeps its name; only the producer changes.

## User Stories

1. As a player, I want guild-specific HUD rows below my main stats line, so that mount, minion, and soul info stays visible during play.
2. As a player with a guild selected, I want HUD rows for that guild only when the guild is in my selection, so that unrelated guild data never appears.
3. As a player who deselects a guild mid-session, I want that guild’s HUD rows to disappear immediately, so that stale data does not linger on screen.
4. As a player who deselects a guild mid-session, I want that guild’s stored HUD state cleared from memory, so that re-selecting does not flash outdated values before the next game line.
5. As a player, I want short-score and prompt updates not to clear my guild HUD display, so that the HUD stays stable within a session.
6. As a player who uses Connect Command, I want secondary status cleared on fresh session, so that reconnect does not show stale guild HUD from the prior session.
7. As a maintainer adding a new guild HUD row, I want one module to extend, so that stats does not grow further.
8. As a maintainer, I want guild-selection gating in one place, so that draw does not list per-guild conditions.
9. As a maintainer, I want guild HUD effects separate from stats effects, so that the trigger pipeline mirrors combat awareness.
10. As a maintainer, I want Animist soul, Riftwalker entity, Tzarakk mount, and Nergal resource/minions colocated in Secondary Status, so that the guild HUD band has one owner.
11. As a maintainer, I want deselect clearing centralized in Secondary Status, so that the application does not special-case each guild.
12. As a test author, I want to apply `SecondaryStatusEffect` and render lines without constructing full Stats, so that guild HUD behavior is testable in isolation.
13. As a test author, I want integration tests for guild-selected-only visibility and clear-on-deselect for all four guild domains, so that lifecycle regressions are caught.
14. As a Nergal player, I want Vitae/Potentia/Evolution status and minion rows in the HUD when Nergal is selected, so that resource tracking stays visible without scrollback spam.
15. As a Nergal player, I want the Nergal resource status line gagged when Nergal is selected, so that automation output stays clean.
16. As a player without Nergal selected, I want no Nergal resource line parsed into HUD state and no Nergal rows rendered, so that unrelated play stays clean.
17. As a maintainer, I want one regex owner for the Nergal resource status line (guild module only), so that BatMUD format changes require one edit.
18. As a maintainer, I want Connect Command’s fresh-session manifest to include Secondary Status explicitly, so that session reset boundaries stay honest after the stats split.
19. As a maintainer, I want domain docs updated to record Secondary Status ownership, so that future readers do not look for guild HUD state in stats.

## Implementation Decisions

### Module ownership

- Introduce a **Secondary Status** module on the application shell (parallel to Combat Awareness).
- Move guild HUD state types from stats: soul companion, Tzarakk mount, Riftwalker entity, Nergal minion, Nergal resource status.
- Move render helpers for all four domains into Secondary Status; expose `render_lines(width, &GuildSelection) -> Vec<Line>`.
- Stats retains only main-line concerns: prompt, short score, recovery brackets, combat-round diff semantics.

### Effect routing

- Introduce `SecondaryStatusEffect` enum owned by Secondary Status (soul set, Tzarakk set/clear, Riftwalker merge/clear, Nergal upsert/set/clear variants — migrated from current stats effect shapes).
- Add `secondary_status: Vec<SecondaryStatusEffect>` to `TriggerEffects`; add a builder helper mirroring `.stat()`.
- Guild trigger modules emit `SecondaryStatusEffect` via `.secondary_status(...)` instead of `StatsEffect` guild variants.
- Remove all guild HUD variants from `StatsEffect`.
- Application applies `apply_secondary_status_effects` separately from `apply_stats_effects` (mirrors combat awareness fan-out).

### Visibility and lifecycle

- **Guild-selected only:** render a guild’s rows only when that `GuildKey` is in `GuildSelection`. Remove OR fallback on “data already observed” for soul, Riftwalker, and Tzarakk.
- **Clear on deselect:** `secondary_status.sync_guild_selection(&GuildSelection)` clears stored state for any guild no longer selected. Application calls this once from guild selection application; remove per-guild clear blocks from the application shell.
- **Parsing:** guild triggers already run only for selected guilds; no guild-agnostic parse path for soul/mount/entity. Nergal resource parsing remains guild-module-owned; remove any duplicate core/global Nergal resource trigger if still registered.

### Render and layout

- `render_lines` accepts terminal width for Nergal minion multi-line wrapping (same behavior as today).
- Row order is not a product concern (BatMUD guild membership is exclusive); keep a stable hardcoded order in code (soul → riftwalker → tzarakk → nergal) for predictability.
- `ViewModel.secondary_status_lines` name unchanged.

### Session lifecycle

- Add `FreshSessionReset::SecondaryStatus` to the Connect Command reset manifest.
- Application owns a `secondary_status` field reset to default when that manifest entry runs (parallel to Combat Awareness).

### Nergal PRD absorption

- This feature **supersedes** `docs/features/nergal-resource-status-ownership/prd.md` for storage, effect routing, render, and lifecycle decisions.
- Still in scope from that PRD: guild-only Nergal resource parsing/gagging, no duplicate core trigger, strict field-order rejection, combined Nergal resource + minion render block when Nergal selected.
- Do not implement the superseded PRD’s “keep Nergal in stats” decisions.

### Delivery

- **Single vertical slice:** introduce Secondary Status, rewire triggers and application, delete guild HUD from stats, include Nergal cleanup, land only when fully green. No phased dual-ownership in stats.

### Documentation

- Add **Secondary Status** bounded context to `CONTEXT.md`.
- Update **Nergal Status** in `CONTEXT.md` to record guild-owned parsing and Secondary-Status-owned storage/render/lifecycle.
- Mark `nergal-resource-status-ownership` PRD as superseded by this feature.

## Testing Decisions

### Testing seams (primary verification points)

1. **Secondary Status module tests** (highest seam for render + effect application) — apply `SecondaryStatusEffect` history, call `render_lines` with guild selection; assert row presence, gating, and span content. Migrate relevant unit tests from stats.
2. **Guild trigger tests** — emit `SecondaryStatusEffect` (not `StatsEffect`) for soul, mount, entity, Nergal resource/minion paths; preserve gag and field-order cases.
3. **Application integration tests** (lifecycle seam) — guild selected: lines update HUD; guild not selected: no HUD rows; deselect clears state and hides rows; Connect Command reset clears secondary status.

Prefer these seams over new test infrastructure. One module test file is sufficient.

### Good tests (external behavior)

- Render empty when no selected guild has HUD data.
- Render soul when Animist selected and soul effect applied; empty when Animist not selected.
- Same selected-only pattern for Riftwalker, Tzarakk, Nergal.
- Deselect guild after populated state → rows hidden and state cleared for that guild.
- Prompt and short-score updates do not clear secondary status (verified via Secondary Status or app integration, not stats preservation clones).
- Nergal resource line gagged with Nergal selected; not parsed into HUD without Nergal selected.
- Connect Command fresh session clears secondary status.

### Prior art

- Stats preservation tests for guild HUD (migrate non-duplicative cases to Secondary Status).
- Application Nergal integration tests in app module tests.
- Guild trigger tests for Animist soul, Riftwalker, Tzarakk, Nergal.

### Avoid

- Snapshot-testing ratatui spans unless already established pattern.
- Tests that assert OR “show when data observed but guild deselected” — that behavior is removed.

## Out of Scope

- Main stats line (short score, recovery brackets, combat diffs on the primary stats row).
- Combat scan rows (separate feature: combat-awareness-cohesion).
- Guild dialog profile inputs (mount name, sabre, riftwalker entity labels).
- New guild HUD types not already in stats.
- Configurable per-player HUD row ordering.
- Per-guild render modules (render stays inside Secondary Status, not delegated to each guild package).

## Further Notes

- Recommendation strength: **Strong** — mirrors proven combat-awareness extraction; payoff grows with each new guild HUD row.
- Grilled decisions locked 2026-07-23; all open questions from initial exploration resolved.
- BatMUD guild membership is exclusive; batrs multi-guild selection in the guild dialog is a config quirk, not a product scenario to optimize row ordering for.
- `nergal-resource-status-ownership` tickets should not be implemented separately; fold parsing cleanup into this slice.
