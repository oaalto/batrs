/**
 * Tests for the /improvement-scan routing contract (issues #180, #181): lane
 * scoring, clarification-answer resolution, and tie-break rules. The command
 * now routes by sending a prompt into the normal agent turn, so the pure logic
 * that remains testable here is the routing/lane selection.
 *
 * Node built-in test runner — no external dependencies.
 * Run with: node --test --experimental-strip-types tests/improvementScanner.test.ts
 */
import { test, describe } from "node:test";
import assert from "node:assert/strict";
import {
  collectLaneScores,
  LANE_TIEBREAK_PRIORITY,
  parseClarificationAnswer
} from "../extensions/improvementScanner.ts";

describe("parseClarificationAnswer", () => {
  const options = ["architecture", "documentation"] as const;
  const build = (): [string, string] => [...options] as [string, string];

  test("matches the second option explicitly by label", () => {
    assert.equal(parseClarificationAnswer("I want documentation", build()), "documentation");
  });

  test("matches the first option explicitly by lane id", () => {
    assert.equal(parseClarificationAnswer("architecture please", build()), "architecture");
  });

  test("does not collapse free-form answers to the first option", () => {
    assert.equal(parseClarificationAnswer("both", build()), null);
    assert.equal(parseClarificationAnswer("not sure", build()), null);
    assert.equal(parseClarificationAnswer("", build()), null);
    assert.equal(parseClarificationAnswer("   ", build()), null);
  });

  test("does not misroute an answer that mentions both options", () => {
    assert.equal(parseClarificationAnswer("architecture or documentation", build()), null);
    assert.equal(parseClarificationAnswer("both architecture and documentation", build()), null);
  });

  test("matches an explicit option only as a whole word, not a substring", () => {
    assert.equal(parseClarificationAnswer("architectural review", build()), null);
    assert.equal(parseClarificationAnswer("documented please", build()), null);
  });
});

describe("collectLaneScores", () => {
  // backend + frontend only, and a request whose lone path signal "packages/"
  // hits both architecture and refactor-style identically -> an exact tie.
  const tieSignals = {
    hasBackend: true,
    hasFrontend: true,
    hasDocs: false,
    hasAdr: false,
    hasContext: false
  };

  test("resolves an equal-scoring tie by the documented priority order, not declaration order", () => {
    const scores = collectLaneScores("packages/", tieSignals);
    const architecture = scores.find((s) => s.lane === "architecture");
    const refactor = scores.find((s) => s.lane === "refactor-style");
    assert.ok(architecture && refactor);
    assert.equal(architecture!.score, refactor!.score, "expected architecture and refactor-style to tie");
    // The lane earlier in the documented priority must sort ahead of the other.
    const expectedFirst =
      LANE_TIEBREAK_PRIORITY.indexOf("architecture") < LANE_TIEBREAK_PRIORITY.indexOf("refactor-style")
        ? "architecture"
        : "refactor-style";
    const rank = (id: string) => scores.map((s) => s.lane).indexOf(id);
    assert.ok(
      rank(expectedFirst) < rank(expectedFirst === "architecture" ? "refactor-style" : "architecture"),
      "tie resolved by LANE_TIEBREAK_PRIORITY, not declaration order"
    );
  });

  test("places the catch-all generic lane last in the documented priority", () => {
    assert.equal(
      LANE_TIEBREAK_PRIORITY[LANE_TIEBREAK_PRIORITY.length - 1],
      "generic-improvement-audit",
      "generic audit must never win a tie against a concrete lane"
    );
  });
});
