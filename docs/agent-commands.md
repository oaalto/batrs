# Agent Commands

Agent instruction: Read this file before code-changing work.

## Format

`cargo fmt --all --check`

## Build / Typecheck

`cargo build`

## Lint

`cargo clippy --all-targets --all-features -- -D warnings`

## Test

`cargo test --all-targets --all-features`

## Docs Checks

`.venv-docs/bin/mkdocs build --strict`

Fallback when the docs venv is not set up: `Not configured yet.` (see README for `python3 -m venv .venv-docs` setup).

Mechanical wiki lint (when wiki pages change): `node scripts/wiki-lint.mjs --staged`

## Runtime-Restricted Checks

Checks requiring credentials, root, Docker, cloud access, paid services, hardware, or local-only infrastructure:

- BatMUD telnet session / live game connection (`cargo run` against `batmud.bat.org`) — requires network and a BatMUD account; user-only.
- Player profile and logs under `~/.batrs/` — writes local config; user-only for verification against real characters.
