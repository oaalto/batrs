# 01 — Trigger config, pipeline gating, and profile persistence

## Parent

`prd.md`

## What to build

Introduce per-group trigger configuration on the player profile and make the trigger pipeline respect it on every incoming line. A player (or maintainer testing via TOML) can disable any of the four built-in groups — guild triggers, spell vocals, common triggers, core triggers — and see the effect on line processing without restarting the client, once the profile is loaded with the new settings.

Configuration lives on the player profile document under `[triggers]`, loads into the player runtime profile, and can be saved back with sparse serialization (omit the section when all groups are enabled; within the section, persist only `false` keys). The public `process()` entry point accepts trigger configuration and skips disabled groups via plain conditional gates while preserving fixed execution order. Disabling guild triggers skips all guild trigger modules for the line, including secondary status effects from guild triggers, without changing guild selection in `/guilds`.

This slice does not add `/triggers` or a dialog — hand-edited TOML and programmatic save are the verification paths.

## Blocked by

None — can start immediately.

## Status

done

## Acceptance criteria

- [x] `TriggerConfig` exists with four boolean fields (`guild_triggers`, `spell_vocals`, `common_triggers`, `core_triggers`), all defaulting to `true`, with sparse serde (omit section when default; omit `true` keys within section).
- [x] Player profile document includes optional `[triggers]`; missing section and missing keys deserialize as all enabled.
- [x] Player runtime profile carries `trigger_config`; profile interpretation populates it from the loaded document.
- [x] `process()` accepts `&TriggerConfig` and gates each group independently; default config preserves current behavior for all existing tests.
- [x] Application passes runtime profile trigger config into `process()` on incoming lines.
- [x] Config manager can save trigger settings to the player file with the same sparse shape; test helper constructs a manager against a unique temp file (no new tempfile dependency).
- [x] Unit test: Animist companion combat hilite does **not** run when `guild_triggers` is false (inverse of existing Animist hilite test).
- [x] Unit test: serde round-trip — defaults omit `[triggers]`; partial disable writes only `false` keys; deserialize restores expected booleans.
- [x] Unit test: save trigger config, read file back, assert sparse TOML content.
- [x] `cargo test --all-targets --all-features` passes.
