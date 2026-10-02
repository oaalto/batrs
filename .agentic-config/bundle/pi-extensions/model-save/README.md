# model-save

Pi extension package bundled by ADC as a **Bundle-installed Pi extension/package**.

## Purpose

ADC stages this package under `.agentic-config/bundle/pi-extensions/model-save/` and the installer places it into the target project's `.pi/extensions/model-save/` (no `pi install`, no network).

`model-save` is a convenience wrapper over Pi's normal interactive `/model` selection flow: it opens the same picker, switches the current session to the selected model, then persists that choice as a startup default.

## Commands

- `/model-save` writes only `defaultProvider` plus `defaultModel` to the global Pi settings file at `~/.pi/agent/settings.json`.
- `/model-save-local` uses the same picker and session switch, then writes only `defaultProvider` plus `defaultModel` to the project-local Pi settings file at `.pi/settings.json` in the current directory.
- Both commands now support `/model`-style slash-command argument completions in `provider/model` format, and an exact typed `provider/model` argument saves directly without reopening the picker.
- Project-local settings override global settings for that trusted project, so `/model-save-local` is the per-project override and `/model-save` is the fallback default elsewhere.
- Interactive/TUI mode only. If the target settings file contains invalid JSON, the command fails without overwriting it.
- Unrelated settings in either settings file are preserved.

## Update

Snapshotted per ADC release — re-download the ADC bundle to take a newer version. `pi update` does not apply.
