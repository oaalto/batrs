# Extract Trigger Rule Engine from triggers/common.rs

## Status

ready-for-agent — grilled 2026-07-27

## Problem Statement

The common triggers module is over 1300 lines and mixes three concerns that are hard to reason about separately:

1. **Rule engine** — generic types (`Rule`, `RuleMatcher`, `RuleCondition`, `RuleAction`, `HiliteTarget`, `MatchData`) and their matching, condition evaluation, and effect application logic.
2. **Rule content** — the static `RULES` list with 60+ game-specific rules, plus programmatic soul-companion combat hiliting rules built from the player name.
3. **Trigger entry point** — `trigger()` running `RULES` and companion rules against a line and facts.

Soul-companion combat hiliting is Animist-specific game content but lives in the generic common layer and runs for any profile that has a `player_name`, including non-Animist characters. That makes the module boundary fuzzy, obscures what is engine versus content, and prevents testing the rule engine without constructing the full rule set.

## Solution

Extract a dedicated **rule engine** module with reusable types and a shared `apply_rules` loop. Keep the common triggers module as the content layer for the static `RULES` list and `trigger()` entry point. Move soul-companion combat hiliting to the Animist guild as a standard guild trigger, gated by the existing `process()` pipeline (guild triggers run only when Animist is in the active guild list). Extract money-summary helpers into a sibling module consumed by the rule engine's `MoneySummary` action.

One slice delivers all of the above. Companion hiliting becomes Animist-only — an intentional behavior change from today.

## User Stories

1. As a batrs maintainer, I want the rule engine separated from rule definitions, so that I can understand matching and effect application without reading 60+ game rules.
2. As a batrs maintainer, I want the rule engine testable without `LazyLock` or game content, so that matcher and condition bugs have a fast, isolated test surface.
3. As a batrs maintainer, I want a shared `apply_rules` loop, so that common triggers and guild triggers that use rules do not duplicate the match-condition-apply iteration.
4. As a batrs maintainer, I want soul-companion combat hiliting owned by Animist, so that Animist-specific content does not live in generic common triggers.
5. As a player with an Animist character, I want soul-companion combat lines hilited as today, so that companion combat remains readable in scrollback.
6. As a player without Animist active, I want soul-companion combat hiliting not to run, so that non-Animist profiles are not affected by Animist-only display rules.
7. As a batrs maintainer, I want money-summary parsing in its own module, so that coin-type domain logic is not buried inside the rule engine.
8. As a test author, I want unit tests on the rule engine for matchers, conditions, and actions, so that engine regressions are caught without the full `RULES` static.
9. As a test author, I want companion hilite behavior tested through the Animist trigger function directly, so that guild trigger tests follow the same pattern as existing soul-companion status tests.
10. As a test author, I want a `process()` integration test proving companion hiliting does not run without Animist in the guild list, so that the gating behavior change is locked in.
11. As a batrs maintainer, I want `RULES` integration tests to remain in the common triggers module, so that existing common-rule coverage is not disrupted by the extraction.
12. As a batrs maintainer, I want the companion rule builder and cache colocated with Animist, so that programmatic rule construction stays next to the guild that owns the content.
13. As a batrs maintainer, I want the trigger pipeline orchestrator unchanged in shape, so that guild → spell vocals → common → core ordering is preserved.
14. As a batrs maintainer, I want `common::trigger()` to call `apply_rules` over `RULES` only (no companion chain), so that common triggers have a single clear responsibility.
15. As a player, I want all non-companion common trigger behavior unchanged, so that the extraction does not regress existing hiliting, echoes, sends, and money summaries.

## Implementation Decisions

### Module layout

Introduce three new modules under the triggers package:

- **rule_engine** — engine types (`Rule`, `RuleMatcher`, `RuleCondition`, `RuleAction`, `HiliteTarget`, `MatchData`), matcher/condition/apply impls, capture hilite helper, rule-builder helpers (`tf_hilite`, `tf_echo`, `tf_style`, `push_rule`), and the shared **`apply_rules`** function.
- **money_summary** — `CoinType`, `push_money_summary`, and related parsing; imported by rule_engine for `MoneySummary` action application.
- **companion_combat_rules** (Animist guild) — `build_companion_rules`, companion regex helpers, name normalization, and the `COMPANION_RULES_CACHE`; constructs `Vec<Rule>` from a player name.

After extraction, **common** retains: the `RULES` static, `trigger()` entry point, and rule registration only. **common::trigger()** calls `rule_engine::apply_rules` over `RULES` — no companion rule chain.

### Shared apply loop

```rust
// Behavioral contract (names illustrative)
fn apply_rules(
    rules: impl IntoIterator<Item = &Rule>,
    plain_line: &str,
    facts: &TriggerFacts,
    output: &mut TriggerEffects,
)
```

Both `common::trigger()` and the Animist companion hilite trigger call this function. The loop: match line → check condition → apply actions. No duplication of the iteration logic.

### Companion combat hiliting → Animist guild trigger

- Add **`soul_companion_combat_hilite_trigger`** to Animist `get_triggers()`, alongside existing soul-companion triggers (status, training, sword hit).
- Trigger shape matches existing guild triggers: `(line, facts) → TriggerEffects`.
- Internally: read `facts.player_name()` → resolve cached `build_companion_rules` → call `rule_engine::apply_rules` over the returned rules.
- **Gating is structural, not per-trigger:** Animist triggers are collected only when Animist is in the active guild list passed to `process()`. No additional guild check inside the trigger function.
- Remove companion rule construction, cache, and chaining from common triggers entirely.

### Behavior change (intentional)

**Before:** Companion combat hiliting runs for any profile with `player_name` set, via common `trigger()` chaining companion rules after `RULES`.

**After:** Companion combat hiliting runs only when Animist is among active guilds, via the Animist guild trigger collected by `process()`.

Non-Animist profiles with a player name no longer receive soul-companion combat hiliting. This is accepted and documented.

### What moves where

| Concern | Destination |
| --- | --- |
| Rule types and impls | rule_engine |
| `apply_rules` loop | rule_engine |
| `tf_hilite`, `tf_echo`, `tf_style`, `push_rule` | rule_engine |
| `apply_rule_action`, `apply_capture_hilite` | rule_engine |
| `CoinType`, `push_money_summary` | money_summary (rule_engine imports for MoneySummary) |
| `RULES` static and registration | common (unchanged responsibility) |
| `trigger()` entry point | common (calls `apply_rules` over `RULES` only) |
| `build_companion_rules`, cache, regex helpers | Animist companion_combat_rules |
| Companion hilite trigger fn | Animist triggers |

### Pipeline (unchanged shape)

`process()` order remains: guild triggers → spell vocals → common triggers → core triggers. Guild triggers (including the new companion hilite trigger) run first and apply line effects before subsequent stages, consistent with existing Animist soul-companion triggers.

### Re-exports and visibility

- Engine types used by common rule registration remain accessible to common (via `use` or `pub(crate)` re-export as needed).
- `Rule` and builder helpers used by companion_combat_rules import from rule_engine.
- No public API surface change beyond the intentional companion gating behavior change.

### Line-count targets

Relax or drop the prior `< 200 lines` cap on rule_engine — money_summary extraction and shared `apply_rules` make a strict line budget misleading. Success is measured by separation of concerns and testability, not a line count.

## Testing Decisions

### Primary test seams (agreed during grilling)

Test at the **highest seam that proves the behavior**, preferring existing patterns:

| Seam | What it proves | Location |
| --- | --- | --- |
| **rule_engine unit tests** | Matcher matching, condition gating, action application (hilite, echo, send) without game content or `LazyLock` | rule_engine test module |
| **money_summary unit tests** | Coin parsing and summary line generation | money_summary test module |
| **common integration tests** | Full `RULES` behavior through `trigger()` / existing `run_trigger` helpers | common test module (companion tests **move out**) |
| **Animist trigger fn tests** | Companion hilite colors and capture groups via direct call to `soul_companion_combat_hilite_trigger` | Animist triggers test module |
| **process() integration test** | Companion hiliting does **not** run when Animist is absent from the guild list, even with `player_name` set | triggers orchestrator test module |

### Good tests (external behavior)

- `RuleMatcher::match_line` returns `None` / `Some` for simple and regex patterns.
- `Rule::condition_met` respects `FlagSet` conditions against `TriggerFacts`.
- Hilite actions produce correct `LineEffect` ranges and colors on styled output.
- Money summary action parses coin lists into expected summary lines.
- Companion announcement line hilited blue when player name matches (Animist trigger, direct call).
- Companion announcement not hilited when player name mismatches (Animist trigger, direct call).
- Avatar hit lines hilite correct capture groups (green player name, colored hit count).
- `process()` with non-Animist guilds + matching companion line → no companion hilite effects.
- `process()` with Animist in guild list + matching companion line → companion hilite effects present.
- All existing non-companion `RULES` tests continue to pass unchanged in common.

### Prior art

- Existing common `run_trigger` / `run_trigger_with_setup` helpers for `RULES` integration tests.
- Existing Animist trigger tests (`soul_companion_status_trigger`, etc.) calling guild trigger fns directly with `TriggerLine` and `TriggerFacts`.
- Companion hilite tests currently in common (to be relocated, not rewritten from scratch).

### Avoid

- Testing internal cache mutex behavior or `ProbePhase`-style enum names.
- Duplicating every `RULES` test in rule_engine — engine tests cover mechanisms; common tests cover content.

## Out of Scope

- Restructuring the full trigger pipeline orchestrator beyond removing companion chaining from common.
- Adding new trigger rules or new rule action types.
- Changing the `Trigger` type signature or `process()` parameter list.
- Per-trigger `facts` guild checks (gating is via guild collection only).
- Moving other Animist soul-companion triggers (status, training, sword hit) — they already live in Animist.
- Configurable trigger chain ordering (separate PRD).
- Improving line effect application (separate PRD).

## Further Notes

- **Label:** `ready-for-agent`
- Grilling decisions Q1–Q10 locked 2026-07-27; this spec supersedes the pre-grill PRD architecture (which proposed keeping companion rules in common/rule_engine and preserving all behavior).
- Related: Animist guild already owns soul-companion status/training/sword-hit triggers; companion combat hiliting joins that set.
- `code-hygiene-cleanup` slice `05-companion-cache-unwrap` may overlap with companion cache relocation — implementer should reconcile if both are pending.
- Wiki update not required for this planning artifact; record `skip` in wiki log if implementation lands without durable wiki changes.
