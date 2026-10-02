# Mocking

Mocking is a last resort, not the default TDD technique.

## Default stance

Prefer real collaborators when they are:

- fast
- deterministic
- local to the test
- part of the behavior you actually care about

Prefer simple fakes or in-memory adapters over deep mock trees.

## Good reasons to mock

Mock at a hard boundary you do not want to cross in the test, for example:

- network calls
- time
- randomness
- process execution
- billing/external provider integrations
- slow or flaky infrastructure

Even there, mock the boundary once. Do not mock your whole call graph.

## Bad reasons to mock

- To verify every internal call
- To avoid constructing a real domain object
- To keep a leaky design alive
- To make an implementation-shaped test easier to write

If the test needs many mocks, the design is probably the problem.

## Preferred order

1. Real code through the public interface
2. Small fake at one external boundary
3. Mock only the boundary contract you truly need to control

## Rule of thumb

If changing an internal helper breaks the test while behavior stays the same, you mocked too deep.
