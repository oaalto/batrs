# Code hygiene cleanup

## Status

`ready-for-agent` — synthesized from static audit (Jul 2026). `cargo clippy --all-targets --all-features` is clean; findings are latent smells, not active compiler warnings.

## Problem Statement

The codebase compiles and passes clippy with no warnings today, but several patterns hide real issues or add maintenance drag: dead-code suppressions on production APIs that are only used from tests, silent UTF-8 loss in telnet line processing, ad-hoc `eprintln!` instead of the already-initialized logging stack, legacy `lazy_static` regex initialization across triggers/guilds, and widespread `.unwrap()` on runtime paths (mutex locks, dynamic regex compilation). These do not fail CI now, but they violate the project's warning-hygiene discipline, make future regressions harder to spot, and increase panic surface in production paths.

## Solution

A focused hygiene pass that removes suppressions by fixing root causes (not re-allowlisting), aligns test-only APIs with `#[cfg(test)]`, makes telnet line decoding explicit about invalid UTF-8, routes operational messages through `log`, and optionally modernizes static regex init — without changing player-visible behavior except where silent data loss is corrected.

## User Stories

1. As a maintainer, I want zero `#[allow(dead_code)]` on production modules, so that unused API surface is deleted or correctly gated for tests.
2. As a maintainer, I want test-only helper methods on the stats model marked with `#[cfg(test)]`, so that the public stats interface reflects what production code actually uses.
3. As a maintainer, I want `merge_riftwalker_battle_hp` either removed or test-gated, so that production code does not carry a superseded HP-merge path alongside `merge_riftwalker_battle_hp_from_listen`.
4. As a maintainer, I want `has_nergal_minions` and `has_nergal_resource_status` test-gated or inlined in tests, so that dead-code allows are unnecessary.
5. As a maintainer, I want telnet line assembly to handle invalid UTF-8 explicitly, so that clippy `lines_filter_map_ok` suppression is not needed.
6. As a player, I want telnet lines with invalid UTF-8 handled predictably (drop, replace, or error), so that behavior is intentional rather than silent omission.
7. As a maintainer, I want the telnet buffer module's line-splitting behavior covered by a unit test, so that UTF-8 handling cannot regress silently.
8. As a maintainer, I want nested `if` in config migration flattened or justified without `#[allow(clippy::collapsible_if)]`, so that style suppressions do not accumulate.
9. As a maintainer, I want connection and session failures in the application shell logged via `log::warn!` / `log::error!`, so that `RUST_LOG` can filter severity.
10. As a maintainer, I want config migration failures logged via the log crate, so that error reporting is consistent with the rest of the binary.
11. As a maintainer, I want socket read/write failures in the main connection thread logged via `log`, so that production debugging does not depend on stderr alone.
12. As a maintainer, I want ANSI/styled-line parse failures logged at debug or warn level, so that stderr is not the only observability path.
13. As a developer running locally, I want to raise log verbosity with `RUST_LOG=batrs=debug` without recompiling, so that troubleshooting matches standard Rust practice.
14. As a maintainer, I want no new `eprintln!` introduced in application paths after this pass, so that hygiene does not regress.
15. As a maintainer, I want companion trigger regex cache mutex poisoning handled without `.unwrap()` panic, so that a poisoned lock does not brick trigger processing for the session.
16. As a maintainer, I want dynamic regex compilation in companion rules to use infallible patterns or structured error handling, so that runtime `Regex::new(...).unwrap()` cannot panic on cache miss.
17. As a maintainer, I want static trigger regex initialization modernized from `lazy_static!` to `std::sync::LazyLock` where touched, so that the `lazy_static` dependency can eventually be removed.
18. As a maintainer, I want guild grouping's existing `OnceLock` pattern treated as prior art for static init, so that migration is consistent across modules.
19. As a maintainer, I want clippy `-- -D warnings` to remain green after every hygiene slice, so that CI gates stay authoritative.
20. As a maintainer, I want existing stats preservation tests to keep passing after dead-code cleanup, so that prompt/short-score HUD behavior is unchanged.
21. As a maintainer, I want existing trigger behavior tests in common triggers to keep passing, so that highlight/send side effects are unchanged.
22. As a maintainer, I want config serde and migration tests to keep passing after collapsible-if cleanup, so that player TOML round-trips are unchanged.
23. As an AFK agent, I want work sliced by smell category (stats allows, telnet UTF-8, logging, lazy_static/unwrap hotspots), so that each PR is reviewable and buildable.
24. As a maintainer, I want out-of-scope items explicitly listed, so that a hygiene pass does not balloon into a full error-handling rewrite.
25. As a maintainer, I want no new lint suppressions added to land this work, so that warning-hygiene rules are honored.
26. As a maintainer, I want production `unwrap()` in static regex literals left alone unless mechanically migrating the block, so that effort focuses on runtime panic paths first.
27. As a maintainer, I want test-module `unwrap()` left alone, so that test ergonomics are not degraded.
28. As a maintainer, I want documentation of the chosen UTF-8 policy in the telnet buffer module doc comment, so that the next contributor knows why lines are dropped or replaced.
29. As a maintainer, I want logging migration to preserve message text where possible, so that grepping logs still finds familiar strings.
30. As a maintainer, I want the stats test helpers `end_combat_invocations` / `start_combat_round_invocations` pattern replicated for other test-only accessors, so that the module is internally consistent.

## Implementation Decisions

### Slice A — Stats dead-code suppressions (highest priority, smallest diff)

- **Modules:** stats model and its unit tests.
- **Decision:** Remove `#[allow(dead_code)]` from `merge_riftwalker_battle_hp`, `has_nergal_minions`, and `has_nergal_resource_status`.
- **Decision:** Gate `merge_riftwalker_battle_hp` with `#[cfg(test)]` — it is only referenced from stats tests; production uses `merge_riftwalker_battle_hp_from_listen`.
- **Decision:** Gate `has_nergal_minions` and `has_nergal_resource_status` with `#[cfg(test)]`, matching existing `end_combat_invocations` / `start_combat_round_invocations` accessors in the same impl block.
- **Decision:** Do not change stats effect routing, render methods, or HUD assembly — behavior is unchanged.

### Slice B — Telnet buffer UTF-8 handling

- **Modules:** telnet buffer (app layer), possibly its unit tests.
- **Decision:** Remove `#[allow(clippy::lines_filter_map_ok)]` from line processing.
- **Decision:** Replace `filter_map(Result::ok)` with explicit handling. Default policy: **skip invalid UTF-8 lines** (current effective behavior) but document that choice; log at `debug!` when a line is skipped so silent loss is observable under `RUST_LOG`.
- **Decision:** Alternative policies (lossy UTF-8 replacement, propagate error) are out of scope unless audit reveals player-visible need — document in module comment.
- **Decision:** No change to GA / CRLF framing logic.

### Slice C — Config style cleanup

- **Modules:** config manager (player TOML load/migrate).
- **Decision:** Flatten nested `if` in `load_user` migration rewrite path to satisfy `clippy::collapsible_if` without allow attribute.
- **Decision:** Replace `eprintln!` for migration/config errors with `log::warn!` or `log::error!` as appropriate; keep user-facing fallback behavior (defaults on invalid config).

### Slice D — Application and connection logging

- **Modules:** application shell, main entry connection setup, styled-line ANSI helper.
- **Decision:** Replace production `eprintln!` (~20 call sites: ~15 app shell, ~2 main, ~2 config, ~1 styled-line) with `log` macros at appropriate levels.
- **Decision:** `env_logger` is already initialized in main — no new logging dependency.
- **Decision:** Do not add structured tracing spans in this pass; plain log lines only.

### Slice E — Runtime unwrap hotspots (companion trigger cache)

- **Modules:** common triggers (companion rules cache).
- **Decision:** Replace `COMPANION_RULES_CACHE.lock().unwrap()` with poison recovery or `expect` with a clear message only if poison is deemed impossible; prefer `lock().unwrap_or_else(|e| e.into_inner())` or early return empty rules on poison.
- **Decision:** Dynamic `Regex::new` inside `build_companion_rules` remains infallible for escaped player names — keep `unwrap` on compile only if patterns are structurally safe; otherwise use `LazyLock` per compiled pattern set.
- **Decision:** Do not rewrite all guild trigger static regex blocks in this slice.

### Slice F — lazy_static modernization (optional, wide diff)

- **Modules:** triggers, guild trigger modules, automation, combat awareness, command, session state, ANSI styled line (~31 files).
- **Decision:** Defer full migration unless bundled with Slice E touch points.
- **Decision:** When migrating a touched file, use `std::sync::LazyLock` (edition 2024) following guild grouping's `OnceLock` prior art.
- **Decision:** Removing the `lazy_static` crate from `Cargo.toml` is a follow-up only after all `lazy_static!` uses are gone.

### Architectural constraints

- No new dependencies for hygiene work.
- No new `#![allow(...)]` or `#[allow(...)]` to land changes.
- Player-visible HUD, trigger highlights, and automation sends must remain unchanged except documented telnet UTF-8 observability (debug log only by default).
- Respect existing ADR and domain language; this is internal quality, not a feature.

## Testing Decisions

### What makes a good test

- Assert **observable behavior**: rendered stats lines, trigger outputs, telnet line vectors, config round-trips — not private field layout or allow-attribute presence.
- Mechanical hygiene (no dead_code allows) is enforced by **clippy with `-D warnings`**, not custom tests.

### Primary test seam (one gate)

- **Repository workflow gate:** `cargo clippy --all-targets --all-features -- -D warnings` after each slice. This is the single mechanical seam proving suppressions were removed without new warnings.

### Behavioral seams by slice

| Slice | Module tests | What to assert |
| --- | --- | --- |
| A — Stats | Existing stats unit tests | Riftwalker HP merge tests still pass via test-gated `merge_riftwalker_battle_hp`; Nergal presence checks use test-gated helpers or direct field assertions in tests |
| B — Telnet | New or extended telnet buffer unit test | Valid CRLF input yields expected line vec; invalid UTF-8 byte sequence yields skipped line (and does not panic) |
| C — Config | Existing config `mod tests` | Serde round-trip, legacy migration, guilds-in-settings parsing unchanged |
| D — Logging | No new tests required | Manual/local: `RUST_LOG=batrs=debug cargo run` shows migrated messages; clippy green |
| E — Companion cache | Existing common trigger tests | Soul companion / avatar hit highlight tests unchanged; no panic on repeated companion name |
| F — LazyLock | Existing trigger tests in touched files | Regex-dependent trigger tests still pass |

### Prior art

- Stats preservation tests (prompt/short-score/combat round) in stats module tests.
- Common triggers integration-style tests (`run_trigger` helpers) in triggers common tests.
- Config serde and `parse_or_migrate` tests in config module tests.
- Guild grouping `OnceLock` static init pattern.

### Seam check (for implementer confirmation)

Proposed order: **clippy gate** (repo-wide) + **stats tests** (Slice A) + **telnet buffer unit test** (Slice B) + **config tests** (Slice C). Slices D–F rely on clippy and existing trigger/config tests unless a behavior change is introduced.

## Out of Scope

- Full removal of `.unwrap()` / `.expect()` in static `lazy_static!` / `LazyLock` regex literals across all guild trigger files (~200+ occurrences, mostly compile-time-safe patterns).
- Rewriting all `eprintln!` in test code.
- Introducing `tracing` spans or structured logging.
- Removing the `lazy_static` crate dependency in the initial pass (requires Slice F completion across all 31 files).
- Refactoring stats secondary-status extraction (separate PRD: secondary-status-extraction).
- Changing trigger rule semantics, priorities, or send actions.
- Player config schema changes.
- Adding `ponytail:` debt comments for deferred work — fix or defer via this PRD's slices instead.
- Fallow/knip dead-code audit tooling integration.

## Further Notes

### Audit inventory (baseline)

| Category | Count / location | Severity |
| --- | --- | --- |
| `#[allow(dead_code)]` | 3 — stats model | Medium — hides test-only API |
| `#[allow(clippy::...)]` | 2 — telnet buffer, config | Medium (telnet), Low (config) |
| `eprintln!` | 20 — app shell, main, config, styled-line | Medium — inconsistent observability |
| `lazy_static!` | 31 files | Low — modernization, not correctness |
| `.unwrap()` in src | ~229 total; production hotspots in common triggers companion cache | Medium on runtime paths |
| Absent smells | No `unsafe`, `todo!`, `FIXME`, crate-level allows, `ponytail:` comments | — |

### Suggested implementation order

1. Slice A (stats `#[cfg(test)]`) — smallest, zero behavior change.
2. Slice B (telnet UTF-8 + test) — fixes only real behavioral smell.
3. Slice C (config clippy + logging) — small.
4. Slice D (remaining logging) — mechanical, wide but safe.
5. Slice E (companion cache unwrap) — targeted panic reduction.
6. Slice F (lazy_static → LazyLock) — optional batch when touching trigger files anyway.

### Triage

- **Label:** `ready-for-agent`
- **Parent feature:** `code-hygiene-cleanup`
- **Slices:** `docs/features/code-hygiene-cleanup/` (01–06; 01–05 parallel, 06 optional wide refactor)
