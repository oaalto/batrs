# 01 — Clear Command end-to-end redraw

## Parent

`prd.md`

## What to build

A player types `/clear` at the main `>` prompt (before or after login, with no modal dialog open) and the terminal performs an atomic visual refresh: clear the terminal surface, then immediately redraw the full UI from in-memory state. Scrollback content, scroll position, stats and guild HUD rows (repainted from unchanged HUD state), and session data stay as they were. The command is silent, never sent to BatMUD, and listed in `/help` with: `Redraws the display from memory (fixes screen artifacts).`

Command Dispatch emits a dedicated `Redraw` effect; the application shell sets a one-shot pending flag without mutating buffers or state; the main event loop consumes that flag and calls terminal clear before the normal draw pass.

## Acceptance criteria

- [ ] `/clear` is registered as a builtin with no login gate; extra arguments are ignored.
- [ ] `/clear` before login and after login dispatches to `[CommandEffect::Redraw]` only — never `Send`.
- [ ] Applying `Redraw` sets a one-shot pending terminal-clear flag; output line count and scrollback offset are unchanged when output and a non-default scroll position are pre-seeded.
- [ ] Main loop calls terminal clear when the flag is pending, then draws as today; flag is consumed (one-shot).
- [ ] No client echo line is appended to output on `/clear`.
- [ ] `/help` includes the agreed Clear Command one-liner.
- [ ] Command Dispatch and BatApp unit tests cover dispatch and effect-application contracts.
- [ ] `cargo test --all-targets --all-features` passes.

## Blocked by

None — can start immediately.

**Status:** ready-for-agent
