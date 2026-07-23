# 02 — Telnet buffer explicit UTF-8 line handling

**Parent:** `prd.md` (Slice B)

**What to build:** Telnet line assembly documents and implements an explicit policy for invalid UTF-8 in received bytes. The clippy `lines_filter_map_ok` suppression is removed; skipped invalid lines are observable at debug log level under `RUST_LOG`.

**Blocked by:** None — can start immediately

**Status:** ready-for-agent

## Acceptance criteria

- [ ] `#[allow(clippy::lines_filter_map_ok)]` removed from telnet buffer line processing
- [ ] Invalid UTF-8 lines are skipped (same effective behavior as today) with `debug!` when a line is dropped
- [ ] Module doc comment states the UTF-8 policy
- [ ] New or extended unit test: valid CRLF input yields expected lines; invalid UTF-8 byte sequence does not panic and omits the bad line
- [ ] GA / CRLF framing logic unchanged
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Test seam

- New telnet buffer unit test for UTF-8 edge case
- `cargo clippy --all-targets --all-features -- -D warnings`
