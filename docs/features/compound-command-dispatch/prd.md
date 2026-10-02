# Compound Command Dispatch PRD

## Status

accepted

## Suggested branch name

`feature/compound-command-dispatch`

## Problem Statement

BatMUD allows a player to type multiple commands on one line by separating them with `;`. batrs currently treats the whole input as one Command Dispatch line, so a mixed line like `say Hello;cere` is not split into separate commands before shortcut expansion. That means guild shortcuts and generic shortcuts only work when the player types them as standalone input, not when they are part of a compound BatMUD command line.

From the player's perspective, the problem is that batrs does not honor the normal BatMUD workflow of typing several commands in one line while still benefiting from batrs shortcut expansion and Command Dispatch behavior.

## Solution

Extend Command Dispatch so that, for logged-in non-slash input, batrs treats `;` as a compound-command separator. The client will split the typed line into command segments, trim whitespace around each separator, skip empty segments, run each segment through the existing Command Dispatch resolution flow, and then reassemble the resulting effect stream in order.

Adjacent game-send effects will be joined back into one send payload with `;`, so the final text sent to BatMUD preserves normal compound-command behavior. Automation and other non-send Command Dispatch effects will remain in their normal relative order so shortcut functionality continues to work when a segment needs both a send and client-side state changes.

Slash commands remain standalone client commands and are not supported inside semicolon-separated compound input.

## User Stories

1. As a BatMUD player, I want to type multiple commands on one line with `;`, so that batrs matches the game behavior I already use.
2. As a BatMUD player, I want a raw command followed by a shortcut on the same line to work, so that `say Hello;cere` becomes `say Hello;use ceremony`.
3. As a BatMUD player, I want a shortcut followed by another shortcut on the same line to work, so that I can chain several guild or generic shortcuts in one input.
4. As a BatMUD player, I want whitespace around `;` to be ignored, so that `say Hello ; cere` behaves the same as `say Hello;cere`.
5. As a BatMUD player, I want unknown commands inside a compound line to pass through unchanged, so that batrs only expands the parts it understands.
6. As a BatMUD player, I want guild shortcut precedence inside a compound line to match standalone input, so that the first selected guild still wins alias conflicts.
7. As a BatMUD player, I want generic shortcuts inside a compound line to match standalone input, so that existing shortcut habits still work when I chain commands.
8. As a BatMUD player, I want compound lines to preserve the order I typed, so that sends and shortcut-triggered actions happen in the expected sequence.
9. As a BatMUD player, I want existing shortcut outputs that already contain `;` to keep their current meaning, so that batrs does not break established target/use or rip-action flows.
10. As a BatMUD player, I want shortcuts that rely on automation flags or queued follow-up actions to keep working inside a compound line, so that semicolon support does not silently drop client-side behavior.
11. As a BatMUD player, I want empty segments like `look;;score` or `;look;` to ignore the blank parts, so that accidental extra separators do not send empty commands.
12. As a BatMUD player, I want slash commands to remain client-local and predictable, so that `/clear`, `/guilds`, and similar commands are not mixed into game compound lines.
13. As a BatMUD player, I want compound input to apply only after login, so that login prompts and Session Lifecycle input keep their current behavior.
14. As a maintainer, I want compound input support to live in Command Dispatch, so that command parsing, precedence, and login gating stay owned by the existing seam.
15. As a maintainer, I want the single-segment dispatch rules to stay reusable, so that compound support does not fork shortcut behavior into a second code path.
16. As a maintainer, I want effect ordering to remain stable when a segment produces both send and automation effects, so that guild-specific command behavior does not regress.
17. As a maintainer, I want adjacent send effects to collapse into one BatMUD send with `;`, so that the client preserves one-line compound-command semantics instead of sending separate unrelated lines.
18. As a maintainer, I want non-send effects such as automation, output, dialogs, reconnect, redraw, raw-log toggle, and quit to keep their relative order, so that command semantics remain explicit and testable.
19. As a maintainer, I want the implementation to reuse current guild and generic command handlers unchanged where possible, so that semicolon support is a shared dispatch enhancement rather than a wave of per-guild edits.
20. As a maintainer, I want regression tests around compound input behavior, so that future Command Dispatch changes do not break shortcut expansion inside semicolon-separated input.
21. As a reviewer, I want clear boundaries on what is out of scope, so that this feature does not turn into a redesign of slash commands, telnet transport, or Session Lifecycle.
22. As a future contributor, I want the effect-composition rule documented, so that new shortcut handlers understand how compound input preserves automation and send ordering.

## Implementation Decisions

- The feature belongs to **Command Dispatch**. Compound-command parsing will be implemented at the command-interpreter seam that already owns slash-command routing, guild shortcut precedence, generic shortcut fallback, passthrough behavior, and login gating.

- Semicolon splitting applies only to logged-in non-slash user input. Pre-login input keeps its current direct-send behavior because shortcut expansion is not active there and Session Lifecycle login handling should not change.

- The canonical parsing rule is: split the typed line on `;`, trim whitespace around each segment, discard empty segments, and run each non-empty segment through the existing single-segment Command Dispatch resolution flow.

- Slash commands remain standalone client commands. Compound input that contains slash-command segments is not supported. The implementation should reject or bypass compound processing in a predictable way instead of partially mixing client-local slash effects with game-command segments.

- Compound input composition must preserve the full effect stream, not just final send text. Each segment will produce the same ordered effects it would produce as standalone input. Compound support will flatten those segment effect lists in order and then collapse only adjacent send payloads into one send joined with `;`.

- The send-collapsing rule is intentionally narrow: only neighboring game-send effects are joined. Non-send effects remain separate and keep their relative position. This preserves automation functionality for shortcuts that set flags, queue follow-up actions, emit local output, or trigger other client-side behavior.

- Existing shortcut send payloads that already contain `;` are treated as final resolved command text. Compound support must not recursively re-parse or re-split those payloads. This preserves established behavior for helpers and guild commands that intentionally build multi-part BatMUD sends.

- Unknown non-slash segments keep current passthrough behavior. If a segment does not match a builtin slash command, guild shortcut, or generic shortcut, its trimmed original text is sent unchanged as that segment’s send contribution.

- Guild shortcut precedence and generic shortcut fallback remain unchanged inside compound input. The feature is an input-shaping change, not a precedence redesign.

- The preferred module shape is a reusable single-segment dispatch function plus a thin compound-line wrapper. The wrapper handles segment splitting and effect composition; the single-segment path continues to own the current resolution logic. This keeps the implementation deep at the Command Dispatch seam and avoids scattering semicolon awareness across guild modules.

- The major modules affected are:
  - the Command Dispatch module that interprets user input into command effects,
  - the application command-submission path only insofar as it continues to call Command Dispatch for logged-in user input,
  - existing guild and generic shortcut handlers as behavioral dependents that must continue working without semantic changes.

- No new dependencies are required. The feature should use the standard library and existing Command Dispatch data structures.

## Testing Decisions

- Good tests for this feature verify external Command Dispatch behavior, not internal helper structure. They should assert what effects are produced and in what order for a given typed input, rather than how the dispatcher internally splits or merges data.

- Tests should cover all affected modules at the correct seam, with the main focus on Command Dispatch as the public behavior seam for this feature.

- Required regression coverage should include:
  - raw passthrough plus shortcut expansion in one compound line,
  - whitespace trimming around separators,
  - empty-segment skipping,
  - multiple shortcut segments in one line,
  - compound input where a segment produces both send and automation effects,
  - preservation of existing shortcut send payloads that already contain `;`,
  - slash-command rejection or standalone-only behavior inside compound input,
  - unchanged pre-login behavior.

- Tests should preserve current precedence expectations for guild shortcuts over generic shortcuts and first-selected-guild wins behavior inside compound input just as in standalone dispatch.

- Prior art already exists in the codebase in the Command Dispatch tests and in guild-command tests that assert exact send payloads and effect lists. New tests should follow those styles instead of inventing a new harness.

- Validation for implementation work derived from this PRD should follow the repo workflow gates: format, build/typecheck, lint, and test.

## Out of Scope

- Supporting slash commands as segments inside semicolon-separated compound input.
- Changing pre-login input handling or Session Lifecycle login behavior.
- Reinterpreting or recursively splitting existing shortcut outputs that already contain `;`.
- Redesigning guild shortcut precedence, generic shortcut fallback, or client send prefix behavior.
- Any telnet transport change, BatMUD protocol change, or server-side parsing change.
- Any broader command-language feature beyond semicolon-separated compound input.

## Further Notes

- The motivating example is `say Hello;cere`, which should resolve to a single final game send equivalent to `say Hello;use ceremony` while preserving the normal shortcut-expansion path.
- The riskiest behavior boundary is effect ordering when a shortcut segment emits both send and automation effects; the implementation should keep this rule explicit and tested.
- The preferred next step is `to-tickets` so this accepted PRD can be split into one or more implementation slices.
