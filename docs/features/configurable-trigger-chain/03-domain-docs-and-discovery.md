# 03 — Domain docs and command discovery

## Parent

`prd.md`

## What to build

Document the configurable trigger chain in domain vocabulary and make `/triggers` discoverable through the same channels as other client slash commands. Future maintainers and agents should find a single source of truth for: four fixed trigger groups, enable/disable via player profile TOML or `/triggers`, immediate effect on successful save, and v1 boundaries (no reordering, no per-rule editor, no trait registry).

## Blocked by

- [02 — `/triggers` dialog and in-session save](02-triggers-dialog-and-command.md)

## Status

done

## Acceptance criteria

- [x] `CONTEXT.md` updated under Player Profile and/or trigger pipeline vocabulary: trigger group toggles, `[triggers]` on player profile, `/triggers` dialog, fixed pipeline order, guild selection independent of guild-trigger toggle.
- [x] `/help` includes `/triggers` with a one-line description consistent with the PRD (toggle built-in trigger groups).
- [x] Engineering wiki updated if path-map or concepts cover command dispatch or player profile (per wiki skill); wiki log entry recorded per project documentation rules.
- [x] Repo search finds no stale claims that trigger config is startup-only or TOML-only with no in-app editor.
- [x] Parent `prd.md` status remains `ready-for-agent` or is updated to reflect tickets published (no contradiction with implemented behavior).
