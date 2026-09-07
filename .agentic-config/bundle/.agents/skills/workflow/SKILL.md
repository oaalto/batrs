---
name: workflow
description: Validation gate order (format, build/typecheck, lint, test). Load before editing or fixing any source file and run gates before changelog or marking work complete.
---

# Workflow gates

Apply before editing repository source files — including typo fixes and small bugfixes — not only before commit.

Apply validation in order and **stop on failure at each gate**. Fix every finding at the current gate before the next gate, changelog, commit, or marking work complete.

Keep scoped `code-format` and `warning-hygiene` rules for baseline formatting presentation and warning-fix discipline when installed.

## Generic gate order

1. **Format** — formatting checks pass before continuing.
2. **Build / typecheck** — no compile/type errors.
3. **Static analysis / lint** — blocking unless project policy says otherwise.
4. **Tests** — required scope passes.

## Fix-everything-before-continue (mandatory)

**Absolute requirement:** Resolve every finding at the current gate before doing anything else — including the next gate, changelog, commit, or marking work complete.

- **One gate at a time:** Stop on first failure; re-run that gate until green before moving on.
- **No partial green:** Warnings, lint hints, test failures, and format drift are blocking unless project policy documents a carve-out.
- **No deferral:** Do not leave follow-up fixes, baseline allowlists, or suppressions while a gate is red.
- **No suppressions:** Do not add lint/test/tool suppressions or baseline allowlists. Fix the code or config root cause.
- **No bypass:** Do not skip gate order or use `git commit --no-verify`.
- **Pre-commit parity:** When the project runs pre-commit or staged-file checks, fix every finding before commit succeeds.

Do not skip gate order unless project policy documents an exception.

## Optional docs-only path

For docs-only changes, use a reduced validation path (for example docs build + link checks) instead of the full code gate sequence.
