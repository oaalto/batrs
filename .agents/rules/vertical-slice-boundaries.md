# Vertical slice boundaries

When editing files under a slice, kernel, or composition layer (paths defined in `architecture.yaml` when present):

## Public entry only

- Import sibling modules only via their **public entry** (e.g. `index.ts`, `mod.rs`, `__init__.py`).
- Do **not** import from `internal/`, `impl/`, `private/`, or deep paths inside another slice.

## Kernel and composition

- **Kernel** (if present): platform plumbing only — config, logging, process bootstrap, shared error/result types, connection pools. No domain logic.
- **Composition** (if present): wiring only — registers slice entrypoints, parses argv, mounts adapters, starts runtime. No domain rules.

## Adapters vs domain

- HTTP handlers, CLI subcommands, queue consumers, and UI routes are **adapters** — thin layers inside or registered by a slice. They are not slice identity.
- Slice identity is the **capability** (domain), not the transport.

## Cross-slice calls

- Call other slices only through their **public entry** exports (facades).
- Do not import persistence models, repositories, or internal types from another slice.

## Duplicate over shared (default)

- Prefer duplicating small helpers at slice boundaries over creating shared `utils/` or cross-slice libraries.
- Promote to kernel only when **two or more slices genuinely need the same plumbing** — not when two slices integrate with the same external system.

## Tests

- Unit tests mirror slice folders (e.g. `test/slices/<name>/`).
- Cross-slice behavior tests live in a dedicated integration area — not inside a single slice's unit tests.

## Project overlay

If the repo defines layout or slice inventory, read and follow before adding imports or new modules:

- `architecture.yaml`
- `SLICES.md` or `ARCHITECTURE.md`
- `docs/adr/*` about slices or modules
- Project-specific overlay rule

If none exist and the change is non-trivial, propose a slice map before large moves.

**batrs:** Guild folders (`src/guilds/<name>/`) are capability boundaries; shared telnet parsers live in `src/triggers/`. Cross-guild reuse goes through `guilds/catalog` and public module exports — not deep imports into another guild's internals.
