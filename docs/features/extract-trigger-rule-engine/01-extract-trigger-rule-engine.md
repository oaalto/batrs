# 01 — Extract trigger rule engine and relocate Animist companion hiliting

## Parent

`prd.md`

## What to build

Separate the trigger rule engine from common rule content in one green slice. Introduce a **rule_engine** module (types, matchers, conditions, actions, shared `apply_rules` loop), a **money_summary** module (coin parsing for `MoneySummary` actions), and slim **common** down to the `RULES` static plus `trigger()` calling `apply_rules` over `RULES` only.

Move soul-companion combat hiliting out of common into Animist: a **companion_combat_rules** builder/cache module and a new **`soul_companion_combat_hilite_trigger`** in Animist `get_triggers()` that calls `apply_rules` over cached companion rules. Gating is structural — companion hiliting runs only when Animist is in the active guild list collected by `process()`, not for every profile with `player_name`. This behavior change is intentional.

All tests land with the slice: engine unit tests, money_summary unit tests, relocated companion hilite tests on the Animist trigger fn, retained `RULES` integration tests in common, and one `process()` integration test proving no companion hilite without Animist active.

Reconcile overlap with `code-hygiene-cleanup/05-companion-cache-unwrap.md` — companion cache moves to Animist in this slice; mark hygiene slice superseded or fold poison-safe cache handling here if still pending.

## Blocked by

None — can start immediately.

## Status

done

## Acceptance criteria

- [x] **rule_engine** module exists with `Rule`, `RuleMatcher`, `RuleCondition`, `RuleAction`, `HiliteTarget`, `MatchData`, matcher/condition/apply impls, capture hilite helper, builder helpers (`tf_hilite`, `tf_echo`, `tf_style`, `push_rule`), and shared **`apply_rules(rules, plain_line, facts, output)`**
- [x] **money_summary** module owns `CoinType` and `push_money_summary`; rule_engine imports it for `MoneySummary` action application
- [x] **common** retains `RULES` static and registration; `trigger()` calls `apply_rules` over `RULES` only — no companion rule chain
- [x] Companion rule builder, cache, and regex helpers removed from common; live in Animist **companion_combat_rules** module
- [x] **`soul_companion_combat_hilite_trigger`** added to Animist `get_triggers()`; reads `facts.player_name()`, resolves cached `build_companion_rules`, calls `apply_rules`
- [x] No per-trigger guild check inside the companion hilite trigger — gating via `process()` guild collection only
- [x] **Behavior change verified:** non-Animist profile with `player_name` set does not get companion combat hiliting through `process()`
- [x] **Behavior preserved:** Animist-active profile with matching `player_name` gets same companion hilite colors/capture groups as before
- [x] All non-companion `RULES` behavior unchanged; existing common integration tests pass
- [x] Companion hilite tests moved from common to Animist triggers test module (direct trigger fn calls, same assertions)
- [x] rule_engine unit tests: matcher match/no-match, condition gating, hilite/echo/send actions without `LazyLock` or full `RULES`
- [x] money_summary unit tests: coin list parsing and summary line output
- [x] One `process()` integration test in triggers orchestrator: companion line + `player_name` + guild list without Animist → no companion hilite effects
- [x] `mod rule_engine` and `mod money_summary` registered in triggers package; Animist `mod companion_combat_rules` registered
- [x] `cargo test --all-targets --all-features` passes; workflow gates (format, clippy) pass
