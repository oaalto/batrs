# 06 — lazy_static to LazyLock migration (optional)

**Parent:** `prd.md` (Slice F)

**What to build:** Static regex initialization in touched trigger/guild modules uses `std::sync::LazyLock` instead of `lazy_static!`, following the guild grouping `OnceLock` prior art. Enables eventual removal of the `lazy_static` crate.

**Blocked by:** None — can start immediately; **recommended after 01–05** to avoid merge churn

**Status:** ready-for-agent

## Acceptance criteria

- [ ] All `lazy_static!` uses migrated to `LazyLock` (or `OnceLock` where appropriate) across trigger and guild modules (~31 files)
- [ ] `lazy_static` removed from `Cargo.toml` if no remaining uses
- [ ] Trigger behavior unchanged: all trigger/guild tests pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Test seam

- Existing trigger tests in each touched guild/common module
- `cargo test --all-targets --all-features`

## Note

Wide mechanical diff. Skip or batch by directory if merge conflict risk is high. Tickets 01–05 deliver the hygiene value; this slice is dependency cleanup only.
