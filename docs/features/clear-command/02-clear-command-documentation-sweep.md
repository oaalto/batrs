# 02 — Clear Command documentation sweep

## Parent

`prd.md`

## What to build

Domain vocabulary and player/agent documentation match the shipped Clear Command: a client-only terminal redraw from in-memory UI state — not an output-buffer wipe and distinct from Session Lifecycle scrollback clear on character change.

## Acceptance criteria

- [ ] `CONTEXT.md` defines **Clear Command** under Command Dispatch (redraw semantics, no login gate, never sent to BatMUD, distinct from Connect Command / Session Lifecycle output clear).
- [ ] Player manual client-command list includes `/clear` with redraw semantics consistent with `/help`.
- [ ] Engineering wiki command-dispatch concept lists `/clear` among builtins and states redraw-not-wipe semantics.
- [ ] Wiki log entry recorded per project documentation rules if wiki content changed.

## Blocked by

- [01 — Clear Command end-to-end redraw](01-clear-command-end-to-end-redraw.md)

**Status:** ready-for-agent
