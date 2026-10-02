---
name: codebase-design
description: "Shared discipline and vocabulary for designing deep modules: small interfaces, clean seams, testable through the interface."
---

# Codebase Design

Use this skill when the user wants to design or improve a module's interface, decide where a seam should live, make code more testable, or make the codebase easier to navigate.

Favor deep modules: small interfaces, clear seams, and concentrated behavior.

## Core ideas

- Prefer one clear seam over many tiny pass-through modules.
- Test at the interface, not through internals.
- Keep terminology stable across code, docs, and decisions.
- Delete shallow indirection before adding new abstraction.

## How to work

1. Read the surrounding code and trace the real flow.
2. Identify where callers currently pay too much interface cost.
3. Propose the smallest interface that hides the complexity.
4. Keep adapters local unless there are at least two real implementations.
5. If the design changes project terminology or decisions, use `domain-modeling` too.

## Output

Return a concrete recommendation:

- what seam to keep or introduce
- what code can collapse behind it
- what tests should move to the interface
- what abstraction should be deleted or avoided
