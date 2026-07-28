# 04 — Application shell logging migration

**Parent:** `prd.md` (Slice D)

**What to build:** Operational messages in the application shell, main connection thread, and ANSI styled-line helper route through `log` so `RUST_LOG` controls verbosity. No remaining production `eprintln!` outside tests.

**Blocked by:** None — can start immediately (may land after 03 without conflict)

**Status:** done — no production `eprintln!` in `src/`; app shell, main, and config use `log` macros.

## Acceptance criteria

- [x] Production `eprintln!` in application shell (~15), main connection setup (~2), and styled-line helper (~1) replaced with appropriate `log` levels
- [x] Message text preserved where practical for grep continuity
- [x] `env_logger` already initialized in main — no new logging dependency
- [x] Test code may keep `eprintln!`
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] Full test suite passes

## Test seam

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- Manual spot-check: `RUST_LOG=batrs=debug cargo run` shows migrated messages (human)
