---
name: vertical-slice-migration
description: >-
  Bootstrap, migrate, or review vertical-slice architecture with deep modules.
  Use for any codebase shape (CLI, library, API, multi-tier). Use when creating
  slices, migrating layer-first code, placing new code, or reviewing a slice PR.
disable-model-invocation: true
---

# Vertical slice migration

**batrs note:** This repo uses guild capability folders under `src/guilds/<name>/`, not `src/slices/`. Apply slice-boundary thinking to guild modules and shared `triggers/` — do not scaffold `architecture.yaml` unless migrating layout.

Use vocabulary from [LANGUAGE.md](./support/LANGUAGE.md) exactly.

## Step 0 — Detect layout and mode

1. Read `architecture.yaml` if present (tiers, paths, composition entrypoints).
2. If absent, infer:
   - **Single-tier** — one source tree; default `src/slices/`, optional `src/kernel/`, optional composition at `src/main/` or project convention.
   - **Multi-tier** — separate roots (e.g. api + ui); each tier may have its own slices.
3. Detect **NEW** vs **EXISTING**:
   - **NEW:** no slices yet, greenfield or early project.
   - **EXISTING:** layer-first legacy (`services/`, `handlers/`, etc.) or partial slices.

Load project overlay: `architecture.yaml` → `SLICES.md` → slice ADR → step PRDs.

---

## Mode NEW — Greenfield

See [INSTALL.md](./support/INSTALL.md#greenfield).

1. Scaffold slice root(s) per architecture config.
2. Add kernel/composition only if needed (single-tier CLI may use composition only).
3. Create minimal `SLICES.md`.
4. New work: capability → slice → public entry only.

---

## Mode EXISTING — Strangler migration

See [INSTALL.md](./support/INSTALL.md#existing-repo).

1. Discovery: capabilities, cross-imports, tests, jobs, entrypoints (any transport).
2. ADR + `SLICES.md` + optional step PRDs.
3. Per step: extract slice → public entry → register adapters in composition → move tests → shims → delete shims.
4. Review against [CHECKLIST.md](./support/CHECKLIST.md).

---

## Where does this code belong?

1. Name the **capability** — not the HTTP route, CLI flag, or screen name.
2. Match to `SLICES.md`.
3. Adapters stay inside the owning slice or composition wiring only.
4. Cross-slice need → facade on public entry.

---

## Deep module check

- [ ] Public entry is small relative to implementation
- [ ] No external imports of slice internals
- [ ] Cross-slice calls use facades only
- [ ] Tests assert behavior, not folder layout
