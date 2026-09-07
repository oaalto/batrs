# Agent Commands

Agent instruction: Read this file before code-changing work.

## Format

`Not configured yet.`

## Build / Typecheck

`Not configured yet.`

## Lint

`Not configured yet.`

## Test

`Not configured yet.`

## Knip config hygiene

`Not configured yet.`

See `knip-hygiene` rule for policy. Run from repo root when knip is configured.

## Strict codebase health (fallow)

`Not configured yet.`

See `fallow-strict` rule for policy. Run from repo root when fallow is configured.

## Wiki Lint

When `scripts/wiki-lint.mjs` (or a ported equivalent) is present, run mechanical wiki lint before commit. Node reference implementation: `node scripts/wiki-lint.mjs --staged`.

## Docs Checks

`Not configured yet.`

## Runtime-Restricted Checks

Checks requiring credentials, root, Docker, cloud access, paid services, hardware, or local-only infrastructure:

None known.

## To Complete

Agent instruction: When this section lists items, offer the user LLM-assisted follow-up to resolve them. Do not invent commands silently.

- Confirm format command
- Confirm build/typecheck command
- Confirm lint command
- Confirm test command
- Confirm knip command (if using knip-hygiene)
- Confirm fallow command (if using fallow-strict)
- Confirm docs check command
- Confirm runtime-restricted check commands (if any apply)
