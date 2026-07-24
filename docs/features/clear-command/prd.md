# Clear Command

## Status

ready-for-agent — grilled 2026-07-24

## Problem Statement

Terminal display corruption — ghost characters, stale cells, or other visual artifacts — can make the batrs TUI hard to read during a session. Players need a quick client-side way to force a clean repaint without losing scrollback, HUD state, or in-progress input context. Today `terminal.clear()` runs only at startup; there is no player-facing command to trigger a mid-session full redraw.

## Solution

Add the **Clear Command** (`/clear`), a client slash command that performs an atomic visual refresh: clear the terminal surface, then immediately redraw the entire UI from in-memory state (output scrollback, live HUD, input line, scroll position). The command does **not** erase game text, reset stats, or change session state. It is silent, available before and after login, and reachable only from the main `>` prompt when no modal dialog is open.

Help text: `/clear - Redraws the display from memory (fixes screen artifacts).`

## User Stories

1. As a player, I want to type `/clear` when the screen looks corrupted, so that visual artifacts disappear without restarting batrs.
2. As a player, I want `/clear` to redraw my current scrollback exactly as it was, so that I do not lose game text I was reading.
3. As a player who has scrolled up with PageUp, I want `/clear` to preserve my scroll position, so that a redraw does not jump me to the latest output.
4. As a player, I want `/clear` to repaint my stats and guild HUD rows from their current in-memory values as part of the full-screen redraw — without resetting or clearing that HUD state — so that live status information stays accurate and visible after fixing screen artifacts.
5. As a player, I want `/clear` to produce no client echo line, so that the refresh does not add noise to my output buffer.
6. As a player at the login prompt, I want `/clear` to work before I am logged in, so that I can fix display issues during connection or login.
7. As a player logged in, I want `/clear` to work without requiring a special login state, so that I can refresh the display at any time during play.
8. As a player, I want `/clear` never sent to BatMUD, so that the game does not receive a nonsense command.
9. As a player, I want `/clear` listed in `/help`, so that I can discover the command without reading external docs.
10. As a player, I want `/clear` to ignore extra arguments (e.g. `/clear foo`), so that typos do not produce errors or unexpected behavior.
11. As a player with a modal dialog open (`/guilds`, `/settings`, `/generic`), I accept that `/clear` is unavailable until I close the dialog, so that dialog keystrokes are not conflated with main-prompt commands.
12. As a player, I want `/clear` to complete in one action — terminal clear followed immediately by the normal draw pass — so that I never see a prolonged blank screen.
13. As a maintainer, I want Clear Command semantics documented in domain vocabulary, so that "clear" is not confused with output-buffer wipe or Connect Command scrollback reset.
14. As a maintainer, I want Command Dispatch to emit a dedicated redraw effect, so that terminal I/O stays in the main loop and not inside `BatApp`.
15. As a test author, I want to verify `/clear` dispatch and effect application through existing Command Dispatch and `BatApp` test seams, so that redraw behavior is covered without a full terminal integration harness.
16. As a maintainer, I want player-facing command docs updated, so that the manual matches `/help` and the true redraw semantics.
17. As a player who spams `/clear` during heavy output, I want each invocation to safely repaint from current state, so that repeated refreshes are harmless idempotent visual operations.
18. As a player reconnecting via `/connect`, I want `/clear` independent of Session Lifecycle, so that a redraw does not trigger reconnect or fresh-session reset.
19. As a maintainer, I want the engineering wiki command-dispatch concept updated to list `/clear`, so that agent navigation reflects the builtin set.
20. As a player, I want the input line cleared on submit like any other slash command, so that `/clear` does not remain visible in the prompt after I press Enter.

## Implementation Decisions

### Semantics (grilled)

- **Clear Command** means **terminal redraw**, not data clear. No `output.clear()`, no scrollback reset, no HUD state reset, no input buffer mutation beyond normal submit handling.
- Atomic operation: `terminal.clear()` on the next frame, then the existing `draw()` path repaints from memory.
- Scroll offset preserved; no `follow_latest` on redraw.
- Silent: no `CommandEffect::Output` echo.
- Login gating: none (`requires_login: false`), same class as `/help`, `/quit`, `/connect`, `/raw_logs`.
- Availability: main `>` prompt only; not while guild, generic, or settings dialogs capture keystrokes. No `Ctrl+L` chord in v1.
- Never forwarded to BatMUD.

### Command Dispatch

- Register `/clear` in the builtin command map with `requires_login: false`.
- Builtin handler returns a single new effect variant (name: **`Redraw`** recommended — describes behavior; player-facing name stays `/clear`).
- Add `/clear` to `HELP_LINES` with the agreed one-liner.
- Extra arguments ignored (match `/quit`, `/raw_logs` pattern).

### Command effect and application shell

- Extend `CommandEffect` with `Redraw`.
- In `apply_command_effects`, handle `Redraw` by setting a one-shot boolean on `BatApp` (e.g. `pending_terminal_clear`). Do not mutate output, scrollback, stats, combat awareness, secondary status, or session state — the next `draw()` repaints HUD rows from that unchanged state. Return `false` for `sent_command` (no game traffic).
- Expose a consume method on `BatApp` for the main loop (e.g. `take_pending_terminal_clear() → bool`), mirroring the `should_quit` polling pattern.

### Main event loop

- Before `terminal.draw()` each iteration, if `app.take_pending_terminal_clear()` is true, call `terminal.clear()?`.
- Then run `terminal.draw(|frame| app.draw(frame))` as today. No other loop changes.

### Domain documentation

- Add **Clear Command** to `CONTEXT.md` under Command Dispatch: client-only slash command; terminal redraw from in-memory UI state; no login gate; never sent to BatMUD; distinct from Session Lifecycle output clear on character change.
- Update player manual client-command list and wiki command-dispatch concept.

### Naming note

Player command is `/clear`. Domain term is **Clear Command**. Internal effect is **`Redraw`** to avoid implying buffer wipe in code review.

## Testing Decisions

### What makes a good test

Test observable contracts at module boundaries: dispatch input → effect list; effect application → side effects on owned state. Do not assert on `terminal.clear()` calls (terminal handle lives only in `main`). Do not test ratatui paint details.

### Modules to test

1. **Command Dispatch** — `/clear` before login and after login returns exactly `[CommandEffect::Redraw]`; `/clear` with trailing args still returns `Redraw`; `/clear` is not `Send`.
2. **BatApp effect application** — applying `Redraw` sets the pending terminal-clear flag; output line count and scrollback offset unchanged when pre-seeded with output and a non-default scroll position.

### Prior art

- `dispatch_handles_builtin_quit`, `dispatch_handles_connect_before_login_as_client_reconnect` in Command Dispatch tests.
- `command_effect_quit_sets_app_quit_flag`, `command_effect_toggle_raw_logs_reports_unavailable_logger`, `command_effect_output_appends_to_output_buffer` in `BatApp` tests.

### Not unit-tested in v1

- Actual crossterm `terminal.clear()` invocation in `main` (runtime handoff; manual smoke: corrupt screen → `/clear` → clean repaint).

## Out of Scope

- Wiping output scrollback or resetting HUD/session state (that is Connect Command / Session Lifecycle territory, not Clear Command).
- Global key chord (`Ctrl+L`) while dialogs are open.
- `/clear` from inside modal dialogs.
- Renaming the player command to `/redraw`.
- Confirmation echo or status messages.
- Automatic redraw on corruption detection; player must invoke `/clear` explicitly.
- Raw log file changes, Player Profile persistence, or automation side effects.

## Further Notes

- **Label:** `ready-for-agent`
- Grilling corrected an initial misread: "clear the screen" is repaint-only, not `output.clear()`.
- Implementation is intentionally minimal: one new `CommandEffect` variant, one boolean flag, one `terminal.clear()` call site in the main loop — reuses existing dispatch → apply → draw architecture.
- If display corruption inside modal dialogs becomes a reported issue, a follow-up can add a global redraw chord without changing Clear Command semantics.
