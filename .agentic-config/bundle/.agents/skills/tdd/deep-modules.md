# Deep Modules

The best design work in TDD is often interface work. A deep module has a small, simple public surface that hides meaningful internal complexity. The tests for a deep module stay stable because they talk to that small surface instead of the moving parts underneath.

## What to look for

When a TDD cycle starts exposing too many internal details, stop and ask whether the interface is too wide.

Signals:

- Callers need to know too much about sequencing
- Tests need to set up lots of incidental state
- A single behavior requires many collaborators at the boundary
- Internal refactors force unrelated test rewrites

## TDD implication

A good red→green loop often deepens the module one behavior at a time:

1. Write the smallest behavior test through the public entry
2. Add only enough implementation to make that behavior real
3. Notice repeated setup or leaking concepts
4. Move that complexity behind the boundary before the next cycle

If the test can stay the same while internals simplify, the module got deeper.

## Heuristic

Prefer tests that name a capability over tests that name internal mechanics.

- Good: `creates a Project API Key for an Owner`
- Bad: `calls validateMembership before buildInsertPayload`

The first style encourages a deep module. The second style freezes internals in place.

## Practical check

Before adding another mock or another test helper, ask:

- Can this disappear behind the public interface?
- Can the setup move into one domain-level factory/helper?
- Can one smaller boundary replace three leaky ones?

If yes, deepen the module first, then continue the TDD loop.
