# Codebase Optimization Opportunities PRD

## Status

draft

## Problem Statement

The batrs client works correctly, but several parts of the current implementation do more work than necessary on hot paths. During normal BatMUD play, the batrs client repeatedly processes incoming lines, Command Dispatch requests, combat-damage viewer queries, automation state, and Session Lifecycle transitions. The improvement scan found six evidence-backed optimization opportunities where repeated allocation, repeated recomputation, or repeated database maintenance can be reduced without changing visible player behavior.

From the user’s perspective, the problem is not a single bug. It is a collection of avoidable costs that can make the batrs client less responsive, make the local combat-damage dashboard scale poorly as data grows, and spend CPU on repeated work that does not improve correctness.

## Solution

Create one coordinated optimization feature that addresses the six validated opportunities in the current codebase with the smallest safe changes at the highest useful seams. The work will preserve existing Command Dispatch behavior, Player Profile behavior, Session Lifecycle boundaries, Combat Damage semantics, and ADR-backed combat-damage data modeling.

The feature will cover these six optimization tracks:

1. separate Combat Damage database schema validation/opening from migration and backfill work so ordinary opens do not rerun heavyweight maintenance,
2. cache or precompute active Command Dispatch lookup data instead of rebuilding merged guild command maps on every dispatch,
3. reduce per-line cloning in automation-to-trigger evaluation by borrowing or otherwise sharing automation state snapshots,
4. avoid rebuilding or resorting melee-catalog search order on every incoming damage line,
5. reduce unnecessary intermediate collection in Combat Damage aggregate and backfill paths so query and maintenance work scales better with database size,
6. review the main batrs client event loop for unnecessary idle redraw/poll overhead and narrow any work that is measurably redundant.

The intended user-visible outcome is a batrs client that stays responsive under longer sessions and heavier local state, with the Combat Damage viewer and related maintenance paths doing less wasted work, while preserving current behavior and existing domain terminology.

## User Stories

1. As a BatMUD player, I want the batrs client to stay responsive during long sessions, so that accumulated local state does not make routine play feel sluggish.
2. As a BatMUD player, I want local slash commands to resolve quickly, so that Command Dispatch feels immediate when I use guild shortcuts and client commands.
3. As a BatMUD player, I want client responsiveness to remain stable even when several guild modules are active, so that adding supported guild capabilities does not create unnecessary local overhead.
4. As a BatMUD player, I want incoming line processing to stay cheap, so that large amounts of game output do not degrade the local client more than necessary.
5. As a BatMUD player, I want Combat Damage collection to keep up with incoming combat output, so that damage attribution remains available without adding avoidable processing cost.
6. As a BatMUD player, I want the Combat Damage viewer to open and respond quickly, so that reviewing damage data feels lightweight.
7. As a BatMUD player, I want marking unattributed HP-loss rows reviewed or removing reviewed rows to feel immediate, so that dashboard maintenance actions do not trigger unrelated expensive work.
8. As a BatMUD player, I want the Combat Damage viewer to scale better as the database grows, so that old history does not make current investigation painful.
9. As a BatMUD player, I want the batrs client to preserve existing output, command, and combat behavior while being optimized, so that performance work does not change gameplay semantics.
10. As a maintainer, I want database maintenance work to run only when needed, so that normal startup and normal viewer usage do not pay migration/backfill cost repeatedly.
11. As a maintainer, I want Combat Damage schema handling to distinguish routine open, schema validation, migration, and repair/backfill responsibilities, so that performance and correctness are easier to reason about.
12. As a maintainer, I want Command Dispatch to avoid rebuilding the same merged guild command map on each request, so that the hot path reflects active Guild Catalog state without repeated allocation.
13. As a maintainer, I want the active slash-command lookup to be derived at a stable seam, so that dispatch cost depends less on repeated map construction and more on the actual command lookup.
14. As a maintainer, I want automation state passed into trigger evaluation without cloning whole maps for every line when possible, so that trigger processing pays only for the data it actually needs.
15. As a maintainer, I want melee candidate matching to reuse static catalog structure, so that matching cost is dominated by actual line matching rather than repeated sort/allocation overhead.
16. As a maintainer, I want Combat Damage aggregate code to avoid materializing large intermediate vectors unless necessary, so that the dashboard keeps acceptable performance as data volume grows.
17. As a maintainer, I want Combat Damage backfill code to process existing rows efficiently, so that one-time maintenance and schema transitions do not scale poorly.
18. As a maintainer, I want optimization work to respect existing ADRs for melee aggregation and unattributed HP loss, so that performance changes do not accidentally flatten important Combat Damage semantics.
19. As a maintainer, I want optimization changes to land at a few high seams instead of scattered micro-fixes everywhere, so that the resulting code stays reviewable and understandable.
20. As a maintainer, I want tests to protect external behavior while allowing internal implementation changes, so that optimization work can proceed without freezing today’s internals.
21. As a maintainer, I want the optimization effort grouped into one parent PRD, so that I can later split it into implementation slices with `/to-tickets`.
22. As a maintainer, I want the parent PRD to cover both high-impact and lower-priority opportunities, so that I can stage the work by value instead of losing the scan results.
23. As a reviewer, I want each optimization area to name the seam it changes, so that I can evaluate scope before implementation starts.
24. As a reviewer, I want the feature to preserve Command Dispatch ownership boundaries, so that performance work does not become an accidental redesign.
25. As a reviewer, I want the feature to preserve Session Lifecycle ownership boundaries, so that reconnect and fresh-session behavior do not drift during optimization work.
26. As a reviewer, I want Combat Damage viewer optimizations to keep current landing-page and drill-down semantics intact, so that performance work does not alter established dashboard behavior.
27. As a future contributor, I want the codebase to spend less time on repeated work, so that later features inherit a cheaper baseline instead of building on hot-path waste.
28. As a future contributor, I want the optimization plan to prioritize the laziest high-leverage changes first, so that the project gets value quickly without unnecessary refactoring.
29. As a future contributor, I want the feature plan to distinguish immediate optimization wins from speculative redesigns, so that later work can stop once the real bottlenecks are addressed.
30. As a future contributor, I want the resulting slices to be independently reviewable, so that one optimization can land without waiting for all six.

## Implementation Decisions

- This PRD is a single parent feature covering six related optimization opportunities discovered in one improvement scan. It is intentionally broader than a normal one-problem spec so the work can be split later into implementation slices.

- The work will preserve the current domain vocabulary from `CONTEXT.md`: Command Dispatch remains the seam that interprets client-local command input into client effects; Session Lifecycle remains the owner of reconnect and fresh-session reset behavior; Player Profile remains per-player runtime configuration; Combat Damage remains the incoming HP-loss capture and viewer capability.

- Combat Damage optimizations must preserve ADR-backed semantics:
  - melee aggregation still distinguishes `weapon_family` where required,
  - unattributed HP loss remains separate from `damage_events` and is not collapsed into a synthetic unknown damage category.

- The preferred implementation order is value-first and seam-first:
  1. Combat Damage database open-path split,
  2. Command Dispatch command-lookup caching,
  3. automation snapshot borrowing/reuse,
  4. melee matcher search-order reuse,
  5. Combat Damage aggregate/backfill collection reduction,
  6. main-loop redraw/poll narrowing if profiling still shows meaningful idle overhead.

- The highest proposed seam for the Command Dispatch optimization is the active command catalog owned near the application’s selected-guild state, not inside each individual command handler. The intent is to precompute or cache the merged guild command lookup when the active guild set changes, then reuse that lookup during dispatch. This keeps the optimization at the Command Dispatch seam instead of scattering memoization across guild modules.

- The highest proposed seam for the automation optimization is TriggerFacts construction and the trigger-processing boundary. The intended direction is to let trigger evaluation read current automation flags and variables without cloning full maps for every processed line, provided the borrowing model remains simple and does not leak mutable ownership across unrelated subsystems.

- The highest proposed seam for the Combat Damage database optimization is the storage open API. The intended direction is to separate concerns such as:
  - ordinary writable open for routine insert/update work,
  - readonly validated open for viewer reads,
  - migration path for schema upgrades,
  - explicit backfill or repair path when historical data needs maintenance.
  The exact API shape can be decided during implementation, but ordinary runtime actions should no longer implicitly perform all maintenance work.

- The highest proposed seam for melee matching optimization is the static catalog matcher structure. The intended direction is to reuse precomputed catalog ordering and apply the small dynamic preference for recent `weapon_family` without allocating and sorting fresh vectors for each line.

- The highest proposed seam for aggregate/backfill optimization is the Combat Damage query/storage layer rather than the viewer HTML layer. If intermediate collections can be reduced, that should happen where rows are loaded and transformed, not by changing dashboard presentation behavior.

- The main-loop optimization is explicitly lower priority and contingent on evidence. The current loop may already be acceptable after the higher-value hot paths are reduced. This track should remain small and only address measurable redundant polling or redraw behavior; it is not a license to redesign the batrs client event model.

- No new dependencies are required for the first pass. The work should prefer existing standard-library, Tokio, and rusqlite capabilities.

- The feature should favor deletion, reuse, or narrower data flow over new abstraction. New interfaces are acceptable only where a high seam needs a clear ownership boundary, such as a storage open mode or cached command lookup.

## Testing Decisions

- Good tests for this feature verify external behavior and stable contracts, not internal implementation details like exact cache shapes, exact temporary collection counts, or the presence of a specific helper. The tests should fail when optimization work breaks visible behavior, API-level semantics, or persisted-data expectations.

- Combat Damage storage tests should verify that ordinary database usage still preserves schema correctness, viewer-visible behavior, and existing historical-data semantics, while avoiding tests that require knowledge of internal migration wiring unless that wiring becomes a documented contract.

- Command Dispatch tests should verify that slash-command resolution, login gating, guild-command precedence, and generic-command fallback remain unchanged after lookup caching is introduced.

- Automation/trigger tests should verify that trigger evaluation still sees the correct flags, vars, and Player Profile-derived facts for incoming lines, without asserting how snapshots are internally represented.

- Combat Damage matcher tests should verify identical candidate classification and `weapon_family`/verb semantics before and after matcher optimization, especially around family collisions and riposte handling.

- Combat Damage aggregate/viewer tests should verify that landing-page sections, drill-down ordering rules, unattributed HP-loss behavior, and reviewed-row actions remain unchanged from the player’s perspective.

- Main-loop tests, if added, should focus on externally observable redraw or input-processing behavior only when such behavior is already testable at an existing seam. If the event loop is not easily testable without introducing low-value scaffolding, this track should prefer measurement plus the smallest safe behavioral checks.

- Prior art in the codebase already exists for the major seams:
  - Command Dispatch tests in the command module,
  - Combat Damage storage, aggregate, matcher, and viewer tests in the combat-damage modules,
  - automation tests in the automation module,
  - application and session tests in the app module.
  New tests should follow those existing styles instead of inventing a new harness.

- Validation gates for implementation slices derived from this PRD remain the repo workflow gates: format, build/typecheck, lint, and test.

## Out of Scope

- Any change to visible BatMUD gameplay semantics, slash-command meaning, guild capability meaning, Player Profile meaning, or Session Lifecycle reset policy.
- Any redesign of the Combat Damage data model that would contradict current ADRs.
- Any new dependency added solely for optimization unless later evidence proves the standard library and current dependencies are insufficient.
- Any speculative redesign of the batrs client architecture, event loop, trigger system, or Guild Catalog beyond what is needed to remove measured repeated work.
- Any optimization that depends on reading user-local `~/.batrs/` data outside the current runtime paths.
- Any network/protocol optimization or BatMUD-side latency work.
- Any commitment that all six opportunities must ship together; this PRD is intentionally a parent feature for later splitting.

## Further Notes

- This feature is a coordination artifact, not a claim that every opportunity has equal value. The first slices should target the highest leverage, lowest churn changes.
- The scan that motivated this PRD found the clearest likely wins in Combat Damage database open semantics and Command Dispatch command-map rebuilding.
- `/to-tickets` should split this parent PRD into narrowly scoped slices, ideally one optimization track per slice or one tightly coupled pair where a shared seam makes that cleaner.
- If implementation evidence shows one of the lower-ranked opportunities is noise, that slice can be dropped without invalidating the parent PRD.
