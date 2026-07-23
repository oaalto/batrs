# 05 — Companion trigger cache panic hardening

**Parent:** `prd.md` (Slice E)

**What to build:** The soul-companion dynamic trigger rules cache does not panic on mutex poison or regex compile failure during normal play. Highlight and send behavior for existing companion lines stays the same.

**Blocked by:** None — can start immediately

**Status:** ready-for-agent

## Acceptance criteria

- [ ] `COMPANION_RULES_CACHE.lock().unwrap()` replaced with poison-safe handling (recover inner state or return empty rules)
- [ ] Dynamic `Regex::new` in `build_companion_rules` cannot panic on escaped player names (keep infallible patterns or handle error without panic)
- [ ] All existing common trigger tests pass (soul companion announcement, avatar hits, lich drain, etc.)
- [ ] No new lint suppressions
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Test seam

- Existing common triggers module tests (`run_trigger` helpers)
