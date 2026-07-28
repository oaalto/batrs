# 03 — Config migration style and logging

**Parent:** `prd.md` (Slice C)

**What to build:** Player config load/migrate path satisfies clippy without style suppressions, and config-side error reporting uses the `log` crate instead of `eprintln!`.

**Blocked by:** None — can start immediately

**Status:** done — `src/config.rs` uses `log::warn!` / `log::error!`; no clippy style suppressions remain.

## Acceptance criteria

- [x] `#[allow(clippy::collapsible_if)]` removed; nested `if` in `load_user` migration rewrite flattened or restructured
- [x] Config migration and invalid-config messages use `log::warn!` / `log::error!` instead of `eprintln!`
- [x] Fallback behavior unchanged: invalid player config still yields defaults
- [x] All existing config module tests pass
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Test seam

- Existing config serde, migration, and guilds-in-settings tests
