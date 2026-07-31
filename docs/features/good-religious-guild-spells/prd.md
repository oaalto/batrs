# Good Religious guild spells

## Status

ready-for-agent

## Problem Statement

Players whose guild primary background is **Good Religious** have civilization background spells in BatMUD — Celestial spark, cure light/serious/critical wounds, and create food — but batrs today only stores `good_religious` as a thematic primary keyword for guild gating. There is no guild module or slash shortcuts for those spells. Generic cure shortcuts (`clw`, `csw`) and Mage `ccf` exist for other characters, but a Good Religious character needs background-owned aliases that win dispatch precedence and match BatMUD cast semantics (including Celestial spark and create food without a target).

## Solution

Model **Good Religious** as a guild module (same `Guild` trait as Monk, Seminary, Liberator) whose commands register five spell shortcuts. **Auto-inject** that guild whenever the player's primary background keyword is `good_religious`, including when no other guild keys are selected — picking Good Religious as the primary theme is the enable switch; it does not appear as a separate checkbox in `/guilds`. Merge background guild command maps **before** player-selected guilds so Good Religious handlers win alias conflicts (`ccs`, `clw`, `csw`, `ccw`, `ccf`). Commands only in v1 (no triggers).

## User Stories

1. As a Good Religious player, I want `ccs` to cast Celestial spark, so that I can use my background offensive spell from a short alias.
2. As a Good Religious player, I want bare `ccs` to send `cast 'celestial spark'`, so that I can cast without naming a target when the game allows it.
3. As a Good Religious player, I want `ccs orc` to target and cast Celestial spark, so that I can hit a named foe with the standard targeted-cast pattern.
4. As a Good Religious player, I want `clw` to cast cure light wounds defaulting to self, so that I can heal quickly in combat.
5. As a Good Religious player, I want `clw ally` to cast cure light wounds on a named target, so that I can heal party members.
6. As a Good Religious player, I want `csw` to cast cure serious wounds with the same self-default pattern as `clw`, so that serious cures behave like other cures.
7. As a Good Religious player, I want `ccw` to cast cure critical wounds with the same self-default pattern, so that my background critical cure is available without Triad's cause critical wounds stealing the alias.
8. As a Good Religious player, I want `ccf` to send `cast 'create food'`, so that I can create food from a short alias.
9. As a Good Religious player, I want `ccf foo` to still send only `cast 'create food'`, so that mistaken trailing args do not produce invalid target lines.
10. As a Good Religious player with Good Religious primary and zero guilds toggled in `/guilds`, I want all five shortcuts to work, so that background spells do not depend on guild membership.
11. As a Good Religious player who also enabled Mage, I want Good Religious `ccf` to win over Mage `ccf`, so that create food never adds a target prefix.
12. As a Good Religious player who also enabled Spider, I want Good Religious `csw` to win over Spider `csw`, so that cure serious wounds is not replaced by guild-specific behavior.
13. As a Good Religious player who also enabled Triad, I want Good Religious `ccw` to win over Triad `ccw`, so that cure critical wounds is not replaced by cause critical wounds.
14. As a player whose primary background is not Good Religious, I want Good Religious shortcuts not registered, so that Riftwalker `ccs`, Mage `ccf`, Spider `csw`, Triad `ccw`, and generic cures behave as today.
15. As a player with Riftwalker selected and Civilized primary, I want `ccs` to remain Riftwalker's current-skill shortcut, so that non–Good Religious characters are unaffected.
16. As a player, I do not want a separate Good Religious toggle in the guild drill, so that the UI stays uncluttered — primary theme selection is sufficient.
17. As a maintainer, I want Good Religious implemented as a normal guild module with a commands submodule, so that the pattern matches Seminary and Monk.
18. As a maintainer, I want background guilds merged before selected guilds in command dispatch input construction, so that precedence is explicit until a background→guild map exists.
19. As a maintainer, I want the catalog to know about Good Religious as a buildable entry keyed `good_religious`, so that naming and grouping stay consistent with other guilds.
20. As a maintainer, I want Good Religious excluded from playable drill toggles, so that it does not appear as an extra guild checkbox.
21. As a test author, I want unit tests on each spell handler's send lines (bare and with args), so that cast semantics regressions are caught without a live session.
22. As a test author, I want a test that `build_guilds` injects Good Religious first when primary is `good_religious` with an empty key list, so that auto-injection is verified.
23. As a test author, I want a dispatch-level test that Good Religious `ccf` wins when both Good Religious and Mage guilds are in the merged list, so that merge order is enforced.
24. As a maintainer, I want domain vocabulary updated to record that civilization backgrounds can be auto-injected guild modules, so that future backgrounds follow the same pattern.
25. As a maintainer planning Evil Religious and other themes, I want this slice to stay limited to Good Religious spells only, so that a future explicit background→guild map can extend without rework.

## Implementation Decisions

### Modeling

- Add **Good Religious guild** module implementing `Guild`, structurally parallel to Seminary (commands submodule, empty triggers in v1).
- In BatMUD, civilization backgrounds are guilds; do not introduce a separate `background_commands` layer.

### Catalog and activation

- Register a catalog entry with persisted key `good_religious`, display name **Good Religious**, grouping **Thematic(2)** (Good Religious bucket index).
- Entry must be **buildable** for `build_guilds` auto-injection but **not** listed in playable drill toggles — extend playability (or equivalent filter) so `playable_entries()` and guild drill browse exclude it while `build()` still works. Do not add a `/guilds` checkbox for it.
- **`GuildSelection::build_guilds`**: when `primary_background_keyword() == "good_religious"`, prepend the Good Religious guild instance **before** guilds built from selected keys. Works with an empty `keys` vector.
- Do not require `good_religious` in persisted guild keys.

### Command merge precedence

- Application shell passes the guild vector from `build_guilds` to command dispatch. Because dispatch uses `or_insert` (first registration wins), prepending Good Religious in `build_guilds` makes its aliases win over later guilds and prevents generic fallback for those keys.
- Interim rule until a dedicated background→guild map exists: **background guild(s) first, player-selected guilds after**.

### Spell handlers (send-line contracts)

All lines use the existing client `@` prefix via `command::send` / abilities helpers (same as other guild spells).

| Alias | Spell | Bare input | With args |
|-------|-------|------------|-----------|
| `ccs` | Celestial spark | `cast 'celestial spark'` | `target <args>;cast 'celestial spark' <args>` via targeted-cast helper |
| `clw` | Cure light wounds | `cast 'cure light wounds' me` | `cast 'cure light wounds' <args>` |
| `csw` | Cure serious wounds | `cast 'cure serious wounds' me` | `cast 'cure serious wounds' <args>` |
| `ccw` | Cure critical wounds | `cast 'cure critical wounds' me` | `cast 'cure critical wounds' <args>` |
| `ccf` | Create food | `cast 'create food'` | `cast 'create food'` (args silently ignored) |

- Cures follow the generic cure pattern (`append_args_default` semantics with default `me`), not `cast_spell` bare behavior.
- `ccs` with args uses targeted-cast (`cast_spell` / `targeted_cast`) like Seminary harm body and other targeted spells.
- `ccf` never uses targeted-cast; trailing args produce no client message.

### Triggers and automation

- v1: **commands only**, empty triggers vector (Seminary precedent). No spell-failure or cooldown triggers in this slice.

### Documentation

- Update `CONTEXT.md` glossary/domain notes: Good Religious as auto-injected background guild; merge-order rule; spell alias table above.
- Optional: short note in guild or player manual if other guild spell docs exist for similar slices.

## Testing Decisions

### What makes a good test

- Assert **external behavior**: exact `CommandEffect::Send` strings (with `@` prefix) for each alias, bare and with args — not internal map structure.
- Assert **auto-injection**: `GuildSelection` with `good_religious` primary and empty keys yields a guild list whose first merged command map includes `ccs`.
- Assert **precedence**: dispatch with Good Religious + Mage (or Spider/Triad) in the guild vector resolves `ccf` / `csw` / `ccw` to Good Religious handlers.
- Assert **non-injection**: Civilized (or other) primary does not register Good Religious aliases on the merged map.

### Modules to test

- Good Religious commands submodule (per-handler tests, Seminary `chb` pattern).
- `GuildSelection::build_guilds` (injection and ordering).
- Optional thin dispatch integration test if not covered by handler + build_guilds tests.

### Prior art

- `SeminaryGuild` command tests (`harm body` bare vs targeted).
- `abilities` targeted-cast tests.
- Generic commands cure group tests (`append_args_default` with `me`).
- Guild catalog selection tests.

## Out of Scope

- Evil Religious, Nomad, Civilized, Magical background spell guilds (future slices).
- Explicit persisted **background→guild map** configuration (follow-up; this slice uses hard-coded `good_religious` → Good Religious guild).
- Removing or changing generic `clw`/`csw` for non–Good Religious characters.
- Good Religious triggers, HUD rows, or automation registration.
- Showing Good Religious as a selectable guild toggle in `/guilds` drill.
- Spell vocal data / trigger phrase wiring beyond existing game data.
- Changing Riftwalker, Mage, Spider, or Triad guild modules (precedence comes from merge order only).

## Further Notes

- Grilled decisions captured 2026-07-31; alias `ccs` for Celestial spark is intentional — Riftwalker is not a Good Religious guild and does not conflict when primary is Good Religious.
- User plans to produce a background→guild map later so alias precedence is data-driven instead of merge-order ad hoc.
- No ADR unless implementation discovers a hard-to-reverse trade-off (e.g. playability enum extension) not captured here.
