# Vertical Slice Migration Skill

## Purpose

Workflow for bootstrapping, migrating, or reviewing vertical-slice architecture with deep modules. Supports any codebase shape (CLI, library, API, multi-tier) via `architecture.yaml`.

## When to use

- Greenfield: scaffold slices, kernel, composition, and inventory
- Existing repo: discovery, ADR, strangler migration with shims
- Placement: decide which slice owns new code
- Review: validate a slice PR against boundary invariants

## Files in this package

- `content.md`: main skill workflow (installs as `SKILL.md`)
- `support/LANGUAGE.md`: shared vocabulary
- `support/CHECKLIST.md`: migration step and cleanup checklists
- `support/INSTALL.md`: install and phased-enforcement guidance
- `support/templates/`: `architecture.yaml`, `SLICES.md`, project overlay rule templates

## Related packages

- `rule:vertical-slice-boundaries` — path-scoped import and seam invariants
