# 01 — Extract Session Lifecycle with reconnect parity

## Parent

`prd.md`

## What to build

A player running `/connect` gets the same reconnect behavior as today, but fresh-session reset, reconnect guard, connection generation, coordinator orchestration, stale-event filtering, and channel install are owned by Session Lifecycle instead of scattered methods on the application shell.

The application shell applies a `FreshSessionPlan` to its fields and delegates reconnect to Session Lifecycle. Connect Command entry stays thin and routes into lifecycle. Session Lifecycle unit tests cover guard, plan, stale events, and reconnect outcomes using a fake connection coordinator; existing application reconnect integration tests continue to pass.

## Acceptance criteria

- [ ] Session Lifecycle module exists with separate concerns (connect entry, fresh session plan, reconnect orchestration, stale events).
- [ ] `/connect` clears session-scoped runtime state immediately (stats, guilds, combat awareness, automation full reset, profile default, dialogs closed, `user_config_loaded` false).
- [ ] Only one reconnect attempt at a time; duplicate `/connect` reports reconnect already in progress.
- [ ] Failed reconnect leaves fresh-session state and surfaces an error; guard clears so retry works.
- [ ] Successful reconnect installs new event receiver and command sender.
- [ ] Events from superseded connection ids are dropped before telnet processing.
- [ ] Player Profile reload still happens only on the next successful login (application shell).
- [ ] Output and scrollback behavior unchanged from today (conditional clear deferred to ticket 02).
- [ ] Session Lifecycle public-interface tests pass with fake coordinator; existing app reconnect tests pass.

## Blocked by

None — can start immediately.

**Status:** ready-for-agent
