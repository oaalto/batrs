# 02 — Telnet buffer explicit UTF-8 line handling

**Parent:** `prd.md` (Slice B)

**What to build:** Telnet line assembly documents and implements an explicit policy for invalid UTF-8 in received bytes. The clippy `lines_filter_map_ok` suppression is removed; skipped invalid lines are observable at debug log level under `RUST_LOG`.

**Blocked by:** None — can start immediately

**Status:** done — `src/app/telnet_buffer.rs` documents UTF-8 policy, skips invalid lines with `debug!`, and includes unit tests.

## Acceptance criteria

- [x] `#[allow(clippy::lines_filter_map_ok)]` removed from telnet buffer line processing
- [x] Invalid UTF-8 lines are skipped (same effective behavior as today) with `debug!` when a line is dropped
- [x] Module doc comment states the UTF-8 policy
- [x] New or extended unit test: valid CRLF input yields expected lines; invalid UTF-8 byte sequence does not panic and omits the bad line
- [x] GA / CRLF framing logic unchanged
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Test seam

- New telnet buffer unit test for UTF-8 edge case
- `cargo clippy --all-targets --all-features -- -D warnings`
