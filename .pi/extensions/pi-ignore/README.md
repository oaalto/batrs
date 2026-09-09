# pi-ignore

Pi extension: make selected gitignored files discoverable via a project `.piignore` file (gitignore negation syntax).

## Delivery

Bundle-installed Pi extension/package (`file-placement`): ADC vendors this source at repo root `pi-extensions/pi-ignore/`, stages it into the generated bundle under `.agentic-config/bundle/pi-extensions/pi-ignore/`, and the installer places it into the target project's `.pi/extensions/pi-ignore/` (no `pi install`, no network). Pi auto-discovers it after the project is trusted, then `/reload` hot-loads it. Select **pi-ignore** on the ADC **Pi packages** step to emit the placement step.

## Setup

Add `.piignore` at your project root (commit this file; keep referenced docs gitignored):

```gitignore
!docs/issues/
!docs/issues/**
!docs/*prd*.md
```

## Verify

Start pi → info notification about `.piignore` → `@docs/...` suggests gitignored files → `find`/`grep` include them.

## Requirements

- [pi](https://pi.dev) with `fd` and `ripgrep` (pi installs managed copies under `~/.pi/agent/bin/`).

## Limitations

- Skills discovery still uses `.gitignore` only.
- Extensions run with full system access; review source before installing.

## Update

Snapshotted per ADC release — re-download the ADC bundle to take a newer version. `pi update` does not apply.
