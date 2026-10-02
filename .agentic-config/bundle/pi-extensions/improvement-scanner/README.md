# improvement-scanner

Pi extension: first-class research-only `/improvement-scan` command for repository improvement scans.

## Purpose

**Bundle-installed Pi extension/package** (`file-placement` delivery): ADC vendors `improvement-scanner` at repo root `pi-extensions/improvement-scanner/`, stages it into the zip under `.agentic-config/bundle/pi-extensions/improvement-scanner/`, and the installer places it into the target project's `.pi/extensions/improvement-scanner/` (no `pi install`, no network). Pi auto-discovers it after the project is trusted, then `/reload` hot-loads it. Snapshotted per ADC release — re-download the bundle to update; `pi update` does not apply.

## Command

- `/improvement-scan [optional focus]`

The command is extension-owned (`pi.registerCommand`) and does not depend on a repo-local skill file.

## Behavior

- Uses extension-owned routing plus prompt text; routes the scan prompt into the normal agent turn so the model streams its report into the transcript like any other response.
- Routes deterministically across v1 research lanes: bug diagnosis, architecture, domain modeling, documentation, refactor/style, and generic improvement audit.
- Uses cheap repository signals as tie-breakers after request/path scoring, asks at most one explicit clarification and then continues the same scan flow when ambiguity would materially change the answer shape, and degrades gracefully when `pi-subagents` is absent.
- Stays research-only: scans and summarizes, but does not edit files, execute implementation, refactor code, or write specs.

## Requirements

- Pi interactive/TUI mode.
- An active model selection.

## Update

Snapshotted per ADC release — re-download the ADC bundle to take a newer version. `pi update` does not apply.
