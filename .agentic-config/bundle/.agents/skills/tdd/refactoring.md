# Refactoring

Refactor only from green.

Once the current behavior is proven, clean the design while the tests hold the line.

## What to refactor

Look for:

- duplication exposed by multiple red→green cycles
- names that do not match the domain language
- conditionals that want a deeper module boundary
- setup noise repeated across tests
- public APIs that expose implementation steps

## Safe refactor loop

1. Get to green
2. Change one thing
3. Re-run the relevant tests
4. Repeat

Small refactors compound. Large refactors while green are still risky when you change many ideas at once.

## Common moves

- Extract a helper when duplication is real
- Inline a pointless abstraction
- Rename to domain terms
- Move logic behind a narrower public boundary
- Replace mock-heavy tests with behavior tests when the design improves

## Do not call this refactoring

These are new feature work, not refactoring:

- broadening behavior
- adding speculative extension points
- changing external behavior without changing tests first

If behavior needs to change, go back to red.
