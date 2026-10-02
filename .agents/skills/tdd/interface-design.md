# Interface Design

TDD quality depends on interface quality. If the public interface is awkward, the tests become awkward too.

## Design for testability by designing for clarity

A testable interface is usually just a clear one:

- Inputs are explicit
- Outputs are explicit
- Side effects are visible at the boundary
- Failure modes are understandable

Prefer interfaces that let tests assert on returned values, emitted domain events, or observable state transitions rather than internal calls.

## Good interface properties

### 1. Few inputs

A call that needs many knobs usually exposes too much of the internal design.

### 2. Stable vocabulary

Use the project’s domain language so tests read like real behavior, not plumbing.

### 3. Narrow side-effect boundary

Push I/O to the edges. Keep the core behavior in pure or mostly pure functions when practical.

### 4. Observable outcomes

A caller should be able to tell what happened without reaching into internals.

## TDD questions to ask first

- What is the smallest public entry that can express this behavior?
- What should a caller get back on success?
- What should a caller get back on failure?
- Which details should stay hidden?

## Smells

- Tests need private helpers to verify behavior
- Tests assert on internal collaborator calls instead of results
- Public methods mirror internal steps one-for-one
- Every new behavior requires widening the API

When those show up, redesign the interface before writing more tests.
