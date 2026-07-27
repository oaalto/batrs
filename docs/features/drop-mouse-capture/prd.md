# Drop Mouse Capture

## Status

done

## Problem Statement

batrs enables terminal mouse capture at startup so it can handle scroll-wheel events. That steals all mouse input from the terminal emulator: right-click no longer opens the terminal context menu, and drag-to-select requires holding **Shift**. Players who rely on native terminal copy/paste and context-menu workflows hit friction on every session.

## Solution

Stop capturing mouse events. Delegate right-click, drag-select, and copy to the terminal emulator. Keep keyboard scrollback navigation (**PageUp** / **PageDown**) and command history on **Up** / **Down** when logged in. Remove in-app mouse-wheel scrolling — it cannot coexist with native mouse delegation without conflicting with command history (alternate-scroll mode sends wheel as **Up**/**Down** key events).

## User Stories

1. As a player, I want to right-click in the batrs terminal window, so that my emulator's context menu (paste, split pane, etc.) works normally.
2. As a player, I want to drag-select game text without holding **Shift**, so that copying output feels like any other terminal application.
3. As a player, I want **PageUp** and **PageDown** to scroll the game text scrollback, so that I can review earlier output without the mouse wheel.
4. As a player logged in, I want **Up** and **Down** to recall prior commands, so that repeating commands stays fast and familiar.
5. As a player, I want scroll position preserved when I scroll up and new game text arrives, so that reading history is not disrupted (unchanged behavior).
6. As a player, I want the view to jump back to the newest output when I send a command with **Enter**, so that live play stays the default focus (unchanged behavior).
7. As a player, I accept that the mouse wheel no longer scrolls batrs scrollback in-app, so that the terminal can own mouse button events.
8. As a player using **Guilds**, **Generic commands**, or **Settings** dialogs, I want dialog keyboard navigation unchanged, so that removing mouse capture does not affect modal workflows.
9. As a maintainer, I want mouse-capture setup and teardown removed from the terminal lifecycle, so that startup/shutdown only manages modes batrs actually needs (raw mode, alternate screen, bracketed paste).
10. As a maintainer, I want the application event loop to stop dispatching mouse events, so that dead input paths are deleted rather than left as no-ops.
11. As a maintainer, I want player-facing UI docs updated to match the new interaction model, so that the manual does not mention wheel scroll or **Shift**+drag workarounds.
12. As a maintainer, I want player-facing UI docs to document **Up**/**Down** command history when logged in, so that players discover history navigation after wheel scroll is removed.
13. As a test author, I want obsolete mouse-wheel unit tests removed, so that the suite does not assert behavior that no longer exists.
14. As a maintainer, I want no new configuration surface for mouse policy, so that behavior is consistent across terminals without a toggle nobody asked for.

## Implementation Decisions

### Grilled trade-offs

- **Alternate scroll mode (`?1007h`) was prototyped and works** for wheel-without-capture, but wheel arrives as **Up**/**Down** key events. Command history on **Up**/**Down** (logged in) is higher priority than arrow-key scrollback scrolling, so alt-scroll is rejected.
- **Mouse capture cannot be partial** in crossterm/xterm: enabling capture for wheel also captures clicks and blocks the terminal context menu. There is no wheel-only capture mode.
- **Chosen approach:** remove capture entirely; sacrifice wheel scroll; keep history on **Up**/**Down**.

### Terminal lifecycle (main entry)

- Remove `EnableMouseCapture` from startup `execute!` block.
- Remove `DisableMouseCapture` from shutdown `execute!` block.
- Drop unused mouse-capture imports from the crossterm event import list.
- Keep existing raw mode, alternate screen, and bracketed paste setup unchanged.

### Event loop (main entry)

- Remove the `Event::Mouse` arm from terminal event dispatch.
- Key and paste event handling unchanged.

### Application shell

- Delete `handle_mouse_event` and the `MOUSE_WHEEL_SCROLL_LINES` constant.
- Remove `MouseEvent` / `MouseEventKind` imports from the application module.
- No replacement scroll path for wheel — intentional regression documented in manual.

### Command history (unchanged)

- **Up** / **Down** continue to call `input.move_history` when `session.is_logged_in()`.
- No change to dialog key handlers (dialogs already use **Up**/**Down** for cursor movement).

### Player manual

- In the game-text section: replace wheel and **Shift**+drag bullets with **PageUp**/**PageDown** scrollback and normal terminal drag-select / right-click.
- Add one line: after login, **Up**/**Down** recall prior commands.

### Seams (test boundaries)

1. **Terminal lifecycle** — startup/shutdown crossterm commands (runtime only; not unit-tested).
2. **Event dispatch** — main loop routes `Event::Key` / `Event::Paste` only (runtime only).
3. **Key handling** — `BatApp::handle_key_event` scroll and history behavior (existing tests cover PageUp/PageDown and history; no new seam required).

Single highest seam for behavioral change: delete mouse handling at the event-dispatch boundary; keyboard contracts stay as today.

## Testing Decisions

### What makes a good test

Test observable keyboard contracts at `BatApp` boundaries. Do not assert on crossterm mouse mode escape sequences or terminal emulator integration.

### Modules to test

- **No new tests.** This change is deletions only.
- **Remove** `mouse_wheel_scrolls_output_without_changing_command_history` — it tests deleted behavior.

### Regression coverage retained

- Existing `handle_key_event` tests for **PageUp**/**PageDown** scrollback (if present) and command-history behavior remain the scroll/history safety net.
- Prior art: scrollback unit tests in the scrollback module; `BatApp` key-event tests in `app/mod.rs` test module.

### Not unit-tested

- Right-click context menu and drag-select in real terminals (manual smoke: run batrs, right-click, drag-copy text).

## Out of Scope

- Alternate scroll mode (`?1007h`) or any wheel-scroll replacement.
- Moving command history to **Ctrl+Up**/**Ctrl+Down**.
- Config toggle to re-enable mouse capture.
- `CONTEXT.md`, ADR, or engineering wiki updates.
- In-app context menus or custom copy UI.
- Mouse interaction inside modal dialogs (dialogs were already keyboard-only for mouse).
- Windows-specific mouse API paths beyond removing the existing capture calls.

## Further Notes

- Grilling explored fixing right-click and selection together; both share the mouse-capture root cause.
- Prototype at `/tmp/batrs-mouse-proto` validated alt-scroll but was rejected due to history conflict — prototype is throwaway, not part of this feature.
- Expected diff is small: three touch points (`main.rs`, `app/mod.rs`, `docs/manual/ui.md`), net lines removed.
