/**
 * Tests for the /improvement-scan routing and summary contract (issues #180,
 * #181): reading the completion result as an AssistantMessage, validating the
 * shared summary envelope when the model answers in prose, and resolving
 * clarification answers and score ties by explicit rules.
 *
 * Node built-in test runner — no external dependencies.
 * Run with: node --test --experimental-strip-types tests/improvementScanner.test.ts
 */
import { test, describe } from "node:test";
import assert from "node:assert/strict";
import type { AssistantMessage, TextContent, ToolCall } from "@earendil-works/pi-ai";
import {
  collectLaneScores,
  LANE_TIEBREAK_PRIORITY,
  parseClarificationAnswer,
  summaryTextFromResult,
  validateSummaryEnvelope
} from "../extensions/improvementScanner.ts";

function textResult(content: string): AssistantMessage {
  return { content: [{ type: "text", text: content }] } as unknown as AssistantMessage;
}

function toolCallResult(arguments_: Record<string, unknown>): AssistantMessage {
  const call = { type: "toolCall", name: "improvement_scan_summary", arguments: arguments_ } as ToolCall;
  return { content: [call] } as unknown as AssistantMessage;
}

const ENVELOPE_TEXT = `# Improvement scan

- Lane: architecture (Architecture)
- Confidence: high
- Scope: backend/src
- Research boundary: No files changed.

## Findings

- fragile seam

## Evidence

- package boundary

## Recommendations

- extract the seam

## Details

### Components

- a
`;

describe("summaryTextFromResult", () => {
  test("renders a structured summary from the improvement_scan_summary tool call", () => {
    const result = toolCallResult({
      lane: "architecture",
      confidence: "high",
      scope: "backend/src",
      researchBoundary: "No files changed.",
      findings: ["fragile seam at project boundary"],
      evidence: ["packages/ boundary coupling"],
      recommendations: ["consider extracting the seam"],
      details: { components: ["a"], seams: ["b"], boundaryPressure: [], structuralOptions: [] }
    });

    const text = summaryTextFromResult(result);
    assert.match(text, /# Improvement scan/);
    assert.match(text, /Lane: architecture/);
    assert.match(text, /fragile seam at project boundary/);
    assert.match(text, /No files changed\./);
    assert.equal(validateSummaryEnvelope(text), true);
  });

  test("accepts free-form text that still carries the shared summary envelope", () => {
    const text = summaryTextFromResult(textResult(ENVELOPE_TEXT));
    assert.equal(text, ENVELOPE_TEXT.trim());
  });

  test("rejects unstructured text without the summary envelope as a scan result", () => {
    const text = summaryTextFromResult(textResult("Improvements found: nothing to change."));
    assert.match(text, /could not produce a structured summary/);
  });

  test("rejects free-form text that echoes the envelope markers out of order", () => {
    const outOfOrder = `## Findings\n\n- x\n\n# Improvement scan\n\n- Lane: x\n- Confidence: high\n## Details\n`;
    assert.equal(validateSummaryEnvelope(outOfOrder), false);
  });

  test("rejects free-form text that omits required envelope fields", () => {
    const missingLane = `# Improvement scan\n\n- Confidence: high\n## Findings\n## Evidence\n## Recommendations\n## Details\n`;
    assert.equal(validateSummaryEnvelope(missingLane), false);
  });

  test("coalesces multiple text blocks and returns the fallback when empty", () => {
    const multi = {
      content: [
        { type: "text", text: "a" },
        { type: "text", text: "b" }
      ]
    } as unknown as AssistantMessage;

    const empty: AssistantMessage = { content: [] } as unknown as AssistantMessage;

    assert.match(summaryTextFromResult(multi), /could not produce a structured summary/);
    assert.equal(summaryTextFromResult(empty), "No improvement scan summary returned.");
  });
});

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
    hasContext: false,
    hasPiSubagents: false
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
