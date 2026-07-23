# 02 — Conditional output scrollback on post-connect login

## Parent

`prd.md`

## What to build

After `/connect`, output and scrollback are preserved through the reconnect gap. When the player completes login, the client compares the new login name to the character name snapshotted before session reset: same character keeps output and scrollback; different character (or first login after connect with no prior character) clears output and resets scrollback to follow-latest.

## Acceptance criteria

- [ ] Pre-connect login name is snapshotted before session reset on connect.
- [ ] Reconnecting as the same character preserves output buffer and scrollback position through login.
- [ ] Reconnecting as a different character clears output and resets scrollback on login.
- [ ] Connect before any login clears output on first successful login.
- [ ] Application integration tests cover same-character, different-character, and pre-login connect cases.
- [ ] Ticket 01 Session Lifecycle wiring remains intact; no regression to reconnect parity tests.

## Blocked by

- 01 — Extract Session Lifecycle with reconnect parity

**Status:** ready-for-agent
