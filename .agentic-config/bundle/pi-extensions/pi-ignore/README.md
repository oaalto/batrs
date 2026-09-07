# @oaalto-kodanbce/pi-ignore

Pi extension: make selected gitignored files discoverable via a project `.piignore` file (gitignore negation syntax).

## Install

```bash
pi install npm:@oaalto-kodanbce/pi-ignore@1.1.1
pi install npm:@oaalto-kodanbce/pi-ignore@1.1.1 -l   # project-local
pi -e npm:@oaalto-kodanbce/pi-ignore                 # try once
```

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

`pi update npm:@oaalto-kodanbce/pi-ignore` or bump pinned version in `.pi/settings.json`.
