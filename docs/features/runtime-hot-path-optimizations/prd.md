# Runtime Hot Path Optimizations PRD

## Status

draft

## Problem Statement

The batrs client is already acceptable at startup, but the improvement scan found avoidable runtime work in the hot paths that run during normal play. The clearest remaining costs are in the batrs client render/input path, Command Dispatch trigger processing, and a few secondary per-line data-flow paths. From the user's perspective, the problem is not incorrect behavior; it is that normal gameplay can spend CPU on repeated cloning, repeated trigger-list rebuilding, and other local work that does not improve visible output or game semantics.

## Solution

Optimize the highest-value runtime hot paths without changing visible batrs client behavior, Command Dispatch semantics, Session Lifecycle ownership, Player Profile behavior, or Combat Damage semantics. The feature will focus on the runtime paths that are both evidence-backed and still relevant after excluding startup work:

1. reduce clone churn in the batrs client draw/render path,
2. reduce unnecessary input-buffer copying during ordinary editing,
3. cache active guild-trigger lookup data instead of rebuilding it for every processed line,
4. keep lower-priority follow-ups available for `Automation` waiter scaling and output-buffer append cleanup only if measurement later shows they matter.

The intended user-visible outcome is a batrs client that stays more responsive during longer sessions and heavier output while preserving the same Command Dispatch, Session Lifecycle, and Combat Damage behavior.

## User Stories

1. As a BatMUD player, I want the batrs client to stay responsive during long sessions, so that local performance does not get in the way of play.
2. As a BatMUD player, I want typing commands to feel immediate, so that local input handling does not lag behind my actions.
3. As a BatMUD player, I want typing to stay responsive even when the output area is busy, so that incoming game lines do not cause avoidable local slowdown.
4. As a BatMUD player, I want the visible terminal output to remain unchanged while local performance improves, so that optimization work does not alter gameplay presentation.
5. As a BatMUD player, I want slash-command and trigger behavior to stay the same after optimization, so that improved speed does not come with changed semantics.
6. As a BatMUD player, I want guild-specific automation and trigger behavior to remain correct, so that optimization work does not break my selected guild setup.
7. As a BatMUD player, I want Command Dispatch to remain predictable across active guild combinations, so that command precedence and fallback still work as before.
8. As a BatMUD player, I want scrollback, dialogs, stats rows, and the input cursor to behave the same while the client does less repeated work, so that performance work feels invisible except for responsiveness.
9. As a maintainer, I want runtime optimizations to land at high seams instead of scattered micro-fixes, so that the code stays reviewable and understandable.
10. As a maintainer, I want render-path optimization to happen where the batrs client already prepares and renders view data, so that the fix reduces repeated cloning without redesigning the whole event model.
11. As a maintainer, I want the input-edit optimization to stay inside the existing input-state seam, so that history behavior and grapheme correctness remain local to one owner.
12. As a maintainer, I want guild-trigger processing to avoid rebuilding the same trigger vector for every processed line, so that per-line cost tracks actual trigger execution instead of repeated collection work.
13. As a maintainer, I want the active guild-trigger list to update only when the selected Guild Catalog changes, so that the runtime hot path reuses stable state.
14. As a maintainer, I want optimization work to preserve the domain vocabulary from `CONTEXT.md`, so that Command Dispatch, Session Lifecycle, Player Profile, and Combat Damage keep their current ownership boundaries.
15. As a maintainer, I want to avoid new dependencies for this work, so that the first pass stays surgical and low-risk.
16. As a maintainer, I want the feature to separate primary from secondary opportunities, so that high-value fixes can land without waiting for speculative cleanup.
17. As a maintainer, I want follow-up work on `Automation` waiter scaling to remain optional, so that we do not overbuild before measurement says it matters.
18. As a maintainer, I want follow-up work on output-buffer append cleanup to remain optional, so that we focus first on the hot paths with the strongest evidence.
19. As a reviewer, I want tests to confirm preserved external behavior rather than internal cache shapes or helper structure, so that implementation details can stay flexible.
20. As a reviewer, I want the feature to preserve Command Dispatch ownership boundaries, so that optimization does not become an accidental architecture rewrite.
21. As a reviewer, I want the feature to preserve Session Lifecycle behavior, so that reconnect and login transitions do not drift during runtime optimization work.
22. As a future contributor, I want the client to have cheaper default hot paths, so that later features inherit a better baseline.
23. As a future contributor, I want the plan to prefer the laziest high-leverage fixes first, so that we stop once the meaningful bottlenecks are removed.
24. As a future contributor, I want the parent PRD to support small implementation slices, so that render-path, input-state, and trigger-cache work can be reviewed independently.
25. As a future contributor, I want low-confidence opportunities to stay clearly out of the first pass, so that the optimization effort does not sprawl.

## Implementation Decisions

- This feature intentionally excludes startup optimization work because current startup is acceptable and no startup-focused implementation is needed from this conversation.

- The work will preserve the current domain vocabulary and ownership from `CONTEXT.md`:
  - the **batrs client** remains the terminal application that renders the TUI and handles local input,
  - **Command Dispatch** remains the seam that interprets slash-command input and related local command behavior,
  - **Session Lifecycle** remains the owner of reconnect and fresh-session transitions,
  - **Player Profile** remains the per-player runtime configuration source,
  - **Combat Damage** semantics remain unchanged.

- The preferred implementation order is:
  1. render-path clone reduction in the batrs client draw/render path,
  2. input-state copy reduction for ordinary editing,
  3. cached active guild-trigger list for trigger processing,
  4. optional follow-up only if measurement later justifies `Automation` waiter or output-buffer append optimization.

- The highest proposed seam for render-path optimization is the existing batrs client view/render boundary. The intended direction is to reduce repeated ownership conversion and repeated `clone()` work for output lines and input text during drawing, while preserving the current renderer responsibilities and visible layout.

- The highest proposed seam for input editing optimization is `InputState`. The intended direction is to preserve current grapheme-aware editing and history behavior while reducing whole-string copying during ordinary edits. The optimization should not change visible cursor movement, history recall, paste behavior, or Unicode correctness.

- The highest proposed seam for guild-trigger optimization is the trigger-processing boundary owned by Command Dispatch/application state, not individual guild modules. The intended direction is to flatten or cache the active guild-trigger list when selected guilds change, then reuse that stable list while processing incoming lines.

- Guild-trigger caching must preserve current trigger precedence and current guild selection semantics. Optimization should change only when the active trigger list is rebuilt, not how the triggers behave once selected.

- Render/input optimizations must preserve current visible behavior for:
  - output rendering,
  - scrollback movement,
  - dialog rendering and interaction,
  - stats and Secondary Status rendering,
  - cursor placement,
  - command submission.

- No new dependency is needed for the first pass. Existing standard-library and current crate capabilities are sufficient for the work described here.

- `Automation` waiter scaling is explicitly secondary. The current design is linear in waiter count, but this feature will not redesign it unless later measurement shows waiter volume is a real runtime bottleneck.

- Output-buffer append cleanup is explicitly secondary. The current gag-removal path is not the first optimization target because the stronger evidence points at draw-path cloning and per-line trigger-list rebuilding.

- This feature prefers reuse and narrower data flow over new abstraction. If a cache or precomputed list is introduced, it should live at the existing owner seam rather than behind a new abstraction layer unless that abstraction becomes clearly necessary.

## Testing Decisions

- Good tests for this feature verify external behavior and stable contracts, not implementation details such as the exact cache container, exact clone counts, or helper names.

- batrs client render/input tests should verify that visible output, input editing behavior, cursor placement, dialog behavior, and scrollback behavior remain unchanged from the user’s perspective after optimization.

- Input-state tests should verify grapheme-aware editing, history navigation, paste handling, and mid-line editing behavior, while avoiding assertions about how snapshots or caches are stored internally.

- Command Dispatch / trigger-processing tests should verify that active guild-trigger behavior remains unchanged for the same selected guild set and incoming lines, while avoiding assertions about the exact cached representation.

- If render-path changes are not directly testable at a high seam, prefer behavior checks that cover preserved drawing outcomes instead of adding low-value scaffolding around internal renderer structure.

- Prior art already exists in the codebase for the main seams involved here:
  - app-level tests in the batrs client application module,
  - input-state tests in the input module,
  - trigger-processing tests in the triggers module,
  - guild-specific trigger tests inside guild modules.
  New tests should follow those existing styles.

- Validation for implementation slices derived from this PRD remains the repo workflow gates:
  - `cargo fmt --all`
  - `cargo build --all-targets`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`

## Out of Scope

- Startup optimization work, including Combat Damage database backfill/open-path changes, because startup is already acceptable for this feature.
- Any redesign of Session Lifecycle, Command Dispatch semantics, Player Profile semantics, or Combat Damage semantics.
- Any change to visible BatMUD gameplay behavior, slash-command meaning, guild capability meaning, or trigger meaning.
- Any new dependency added solely for runtime optimization in the first pass.
- Any speculative redesign of the event loop, renderer architecture, trigger architecture, or automation architecture beyond the smallest changes needed to remove measured repeated work.
- Any optimization that depends on reading or changing user-local `~/.batrs/` data outside the current runtime behavior.
- Any promise that secondary opportunities such as `Automation` waiter scaling or output-buffer append cleanup must ship in the first implementation round.

## Further Notes

- This PRD is the runtime-only follow-up to the improvement scan: startup was intentionally excluded after user direction.
- The strongest evidence from the scan points first at render/input clone churn and per-line guild-trigger vector rebuilding.
- `/to-tickets` should split this parent PRD into small slices, ideally one for render/input work and one for guild-trigger caching, with secondary follow-ups only if later measurement justifies them.
- If implementation evidence shows a lower-priority runtime opportunity is noise, that slice can be dropped without invalidating this parent PRD.
