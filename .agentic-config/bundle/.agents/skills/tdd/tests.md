# Tests

TDD tests should read like behavior promises.

## What a good test does

A good test:

- names one behavior clearly
- uses the public interface
- sets up only the state that behavior needs
- asserts on observable outcomes
- survives internal refactors

## Structure

Keep the shape boring:

1. arrange the smallest meaningful state
2. act through the public interface
3. assert on the externally visible result

## Naming

Use names that describe capability and context.

Examples:

- `creates a Project API Key for an Owner`
- `rejects a Profiling Run when the Model Alias is unavailable`
- `returns cached prompt tokens in the usage summary`

Avoid names that describe implementation steps.

## Scope

Prefer one behavior per test. If a test has many unrelated assertions, split it.

## Tracer bullet standard

The first test in a flow should prove the path end to end, even if the behavior is small. That keeps the suite anchored to reality.

## When a test feels hard to write

Do not immediately add helpers, fixtures, or mocks.

Ask first:

- Is the interface wrong?
- Is the setup exposing internals?
- Is this really the behavior that matters?

Usually the smallest fix is to simplify the design, not the test harness.
