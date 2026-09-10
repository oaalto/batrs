import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const COMMAND_NAME = "improvement-scan";

type LaneId =
  | "bug-diagnosis"
  | "architecture"
  | "domain-modeling"
  | "documentation"
  | "refactor-style"
  | "generic-improvement-audit";

type QuestionReason = "lane-shape" | "next-step";

type Confidence = "high" | "medium" | "low";

type LaneDefinition = {
  label: string;
  intent: string;
  detailDescription: string;
  keywords: string[];
  pathSignals: string[];
  repoSignals: string[];
};

type RepoSignals = {
  hasBackend: boolean;
  hasFrontend: boolean;
  hasDocs: boolean;
  hasAdr: boolean;
  hasContext: boolean;
};

type FileProbe = {
  size: number;
};

type ClarificationDecision = {
  shouldAsk: boolean;
  question?: string;
  reason?: QuestionReason;
  options?: [LaneId, LaneId];
};

type LaneScore = {
  lane: LaneId;
  score: number;
  requestScore: number;
  repoScore: number;
  reasons: string[];
};

type ClarificationAnswer = {
  lane: LaneId;
  answer: string;
};

/**
 * Defines which lane wins when `collectLaneScores` produces an equal-scoring
 * tie. The rubric is explicit and deliberate (declaration order is not the
 * source of truth) so tie resolution never silently shifts with object key
 * order: prefer a more specific lane over the catch-all, and fall back to this
 * fixed documented order. `generic-improvement-audit` ranks last by design;
 * escalating it before a concrete lane would waste the scan on a broad pass.
 */
export const LANE_TIEBREAK_PRIORITY: LaneId[] = [
  "bug-diagnosis",
  "architecture",
  "domain-modeling",
  "documentation",
  "refactor-style",
  "generic-improvement-audit"
];

function tiebreakRank(lane: LaneId): number {
  const index = LANE_TIEBREAK_PRIORITY.indexOf(lane);
  return index === -1 ? LANE_TIEBREAK_PRIORITY.length : index;
}

const LANE_DEFINITIONS: Record<LaneId, LaneDefinition> = {
  "bug-diagnosis": {
    label: "Bug diagnosis",
    intent: "Find likely root causes, repro clues, and debugging next steps.",
    detailDescription: "Bug-diagnosis details such as symptoms, likely root cause, repro clues, and debugging gaps.",
    keywords: [
      "bug",
      "broken",
      "error",
      "failing",
      "failure",
      "stack trace",
      "exception",
      "crash",
      "regression",
      "diagnose",
      "debug",
      "root cause",
      "test failure"
    ],
    pathSignals: ["test", "spec", "__tests__", "backend/src", "frontend/src"],
    repoSignals: ["backend", "frontend"]
  },
  architecture: {
    label: "Architecture",
    intent: "Assess seams, boundaries, module responsibilities, and structural changes.",
    detailDescription: "Architecture details such as components, seams, boundary pressure, and structural options.",
    keywords: [
      "architecture",
      "design",
      "module",
      "boundary",
      "seam",
      "dependency",
      "coupling",
      "slice",
      "layer",
      "interface",
      "ownership",
      "refactor architecture"
    ],
    pathSignals: ["packages/", "backend/src/slices", "frontend/src/slices", "docs/adr"],
    repoSignals: ["backend", "frontend", "adr"]
  },
  "domain-modeling": {
    label: "Domain modeling",
    intent: "Sharpen terminology, concepts, glossary, and model boundaries.",
    detailDescription:
      "Domain-modeling details such as terms, ambiguities, missing concepts, and naming recommendations.",
    keywords: [
      "domain",
      "terminology",
      "language",
      "glossary",
      "naming",
      "concept",
      "conceptual",
      "ubiquitous language",
      "context map",
      "context.md"
    ],
    pathSignals: ["context.md", "CONTEXT.md", "docs/wiki", "docs/adr"],
    repoSignals: ["context", "docs"]
  },
  documentation: {
    label: "Documentation",
    intent: "Find docs gaps, stale guidance, and missing user/developer documentation.",
    detailDescription: "Documentation details such as doc gaps, stale pages, missing references, and update targets.",
    keywords: [
      "docs",
      "documentation",
      "readme",
      "help book",
      "guide",
      "wiki",
      "explain",
      "onboarding",
      "usage",
      "instruction"
    ],
    pathSignals: ["README", "docs/", "help-book/", "CHANGELOG.md"],
    repoSignals: ["docs"]
  },
  "refactor-style": {
    label: "Refactor/style",
    intent: "Find maintainability, readability, duplication, and style improvements without changing behavior.",
    detailDescription:
      "Refactor/style details such as duplication, local complexity, cleanup candidates, and low-risk refactors.",
    keywords: [
      "refactor",
      "cleanup",
      "simplify",
      "style",
      "readability",
      "duplicate",
      "duplication",
      "messy",
      "over-engineered",
      "tech debt"
    ],
    pathSignals: ["src/", "packages/", "pi-extensions/"],
    repoSignals: ["backend", "frontend"]
  },
  "generic-improvement-audit": {
    label: "Generic improvement audit",
    intent: "Run a broad improvement-oriented research pass when no narrower lane clearly fits.",
    detailDescription:
      "Generic-audit details such as ranked opportunities, candidate areas, and rationale for the broad scan.",
    keywords: ["improve", "improvement", "audit", "review this repo", "scan", "opportunities", "what should change"],
    pathSignals: ["README", "docs/", "backend/", "frontend/", "packages/"],
    repoSignals: ["backend", "frontend", "docs"]
  }
};

function normalizeRequest(args: string): string {
  return args.trim() || "Scan this repository for improvement opportunities.";
}

function maybeFile(path: string): FileProbe | null {
  try {
    const globalBun = globalThis as { Bun?: { file: (target: string) => FileProbe } };
    return globalBun.Bun ? globalBun.Bun.file(path) : null;
  } catch {
    return null;
  }
}

function fileExists(path: string): boolean {
  const file = maybeFile(path);
  return file !== null && file.size >= 0;
}

function detectRepoSignals(): RepoSignals {
  return {
    hasBackend: fileExists("backend/package.json"),
    hasFrontend: fileExists("frontend/package.json"),
    hasDocs: fileExists("docs/agent-commands.md") || fileExists("README.md") || fileExists("help-book/README.md"),
    hasAdr: fileExists("docs/adr/0042-bundle-delivered-pi-package.md"),
    hasContext: fileExists("CONTEXT.md")
  };
}

export function collectLaneScores(request: string, signals: RepoSignals): LaneScore[] {
  const lower = request.toLowerCase();
  const scores = Object.entries(LANE_DEFINITIONS).map(([lane, definition]) => {
    let requestScore = lane === "generic-improvement-audit" ? 1 : 0;
    let repoScore = 0;
    const reasons: string[] = [];
    let hasRequestSignal = false;

    for (const keyword of definition.keywords) {
      if (lower.includes(keyword)) {
        requestScore += 3;
        reasons.push(`keyword:${keyword}`);
        hasRequestSignal = true;
      }
    }

    for (const pathSignal of definition.pathSignals) {
      if (lower.includes(pathSignal.toLowerCase())) {
        requestScore += 2;
        reasons.push(`path:${pathSignal}`);
        hasRequestSignal = true;
      }
    }

    if (hasRequestSignal || lane === "generic-improvement-audit") {
      for (const repoSignal of definition.repoSignals) {
        if (
          (repoSignal === "backend" && signals.hasBackend) ||
          (repoSignal === "frontend" && signals.hasFrontend) ||
          (repoSignal === "docs" && signals.hasDocs) ||
          (repoSignal === "adr" && signals.hasAdr) ||
          (repoSignal === "context" && signals.hasContext)
        ) {
          repoScore += 1;
          reasons.push(`repo:${repoSignal}`);
        }
      }
    }

    return {
      lane: lane as LaneId,
      score: requestScore * 100 + repoScore,
      requestScore,
      repoScore,
      reasons
    };
  });

  return scores.sort((left, right) => right.score - left.score || tiebreakRank(left.lane) - tiebreakRank(right.lane));
}

/**
 * Parse a free-form clarification answer. Returns the explicitly matched
 * option only when exactly one option is mentioned; ambiguous, empty, or
 * both-options answers ("both", "not sure", blank, "architecture or
 * documentation") return null so the caller falls back to the best-scored lane
 * instead of silently collapsing to one option.
 */
export function parseClarificationAnswer(answer: string, options: [LaneId, LaneId]): LaneId | null {
  const text = answer.trim().toLowerCase();
  const [first, second] = options;
  if (!text) {
    return null;
  }
  const mentionsFirst =
    textMentionsOption(text, first.toLowerCase()) ||
    textMentionsOption(text, LANE_DEFINITIONS[first].label.toLowerCase());
  const mentionsSecond =
    textMentionsOption(text, second.toLowerCase()) ||
    textMentionsOption(text, LANE_DEFINITIONS[second].label.toLowerCase());
  if (mentionsFirst === mentionsSecond) {
    return null;
  }
  return mentionsSecond ? second : first;
}

function textMentionsOption(text: string, option: string): boolean {
  const escaped = option.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(`\\b${escaped}\\b`).test(text);
}

function decideClarification(scores: LaneScore[]): ClarificationDecision {
  const [first, second] = scores;
  if (!first || !second) {
    return { shouldAsk: false };
  }

  if (first.requestScore <= 1 && second.requestScore <= 1) {
    return {
      shouldAsk: true,
      reason: "lane-shape",
      options: [first.lane, second.lane],
      question: `Which direction should /${COMMAND_NAME} prioritize: ${LANE_DEFINITIONS[first.lane].label.toLowerCase()} or ${LANE_DEFINITIONS[second.lane].label.toLowerCase()}?`
    };
  }

  if (first.requestScore === second.requestScore && first.lane !== second.lane) {
    return {
      shouldAsk: true,
      reason: "next-step",
      options: [first.lane, second.lane],
      question: `Quick clarification: do you want a ${LANE_DEFINITIONS[first.lane].label.toLowerCase()} answer or a ${LANE_DEFINITIONS[second.lane].label.toLowerCase()} answer?`
    };
  }

  return { shouldAsk: false };
}

function confidenceFromScores(scores: LaneScore[]): Confidence {
  const [first, second] = scores;
  if (!first) {
    return "low";
  }
  if (!second || first.requestScore - second.requestScore >= 3) {
    return "high";
  }
  if (first.requestScore - second.requestScore >= 1) {
    return "medium";
  }
  if (first.repoScore - second.repoScore >= 2) {
    return "medium";
  }
  return "low";
}

function buildUserPrompt(
  request: string,
  signals: RepoSignals,
  lane: LaneId,
  confidence: Confidence,
  clarification: ClarificationDecision
): string {
  const laneDefinition = LANE_DEFINITIONS[lane];
  const signalLines = [
    signals.hasBackend ? "- backend package present" : null,
    signals.hasFrontend ? "- frontend package present" : null,
    signals.hasDocs ? "- docs/help-book surfaces present" : null,
    signals.hasAdr ? "- ADRs present" : null,
    signals.hasContext ? "- CONTEXT.md present" : null,
    "- Prefer native Pi subagents for parallel research where useful; fall back to single-agent if unavailable"
  ].filter((line): line is string => line !== null);

  return [
    "Run an improvement scan for /improvement-scan.",
    "",
    `User request: ${request}`,
    `Routed lane: ${lane} (${laneDefinition.label})`,
    `Lane intent: ${laneDefinition.intent}`,
    `Routing confidence: ${confidence}`,
    "",
    "Cheap repo signals:",
    ...signalLines,
    "",
    clarification.shouldAsk && clarification.question
      ? `Clarifying-question policy: Ask exactly one clarifying question only if it would materially change the answer shape or next-step recommendation. If still needed, ask: ${clarification.question}`
      : "Clarifying-question policy: Do not ask a clarifying question unless ambiguity would materially change the answer shape or next-step recommendation.",
    "",
    "Instructions:",
    "1. Actually inspect the repository — read the relevant source, config, and docs.",
    "2. Stay strictly on the research side of the line: no file edits, no writing specs.",
    "3. Use the routed lane as the primary frame unless the evidence clearly forces a nearby lane.",
    "4. Prefer native Pi subagents when present and useful; otherwise continue without them.",
    "5. End with a concise structured improvement report in the transcript with these sections: findings, evidence, recommendations.",
    `6. ${laneDefinition.label} focus areas to cover: ${laneDefinition.detailDescription}`,
    "7. Recommend /to-spec only if the destination is clear, the findings are synthesized, and a spec adds coordination value.",
    "",
    "Return only evidence-backed findings."
  ].join("\n");
}

export default function improvementScanner(pi: ExtensionAPI) {
  pi.registerCommand(COMMAND_NAME, {
    description: "Research-only repository improvement scan",
    handler: async (args, ctx) => {
      if (ctx.mode !== "tui") {
        ctx.ui.notify("/improvement-scan requires interactive mode", "error");
        return;
      }

      if (!ctx.model) {
        ctx.ui.notify("No model selected", "error");
        return;
      }

      const request = normalizeRequest(args);
      const repoSignals = detectRepoSignals();
      const laneScores = collectLaneScores(request, repoSignals);
      const lane = laneScores[0]?.lane ?? "generic-improvement-audit";
      const confidence = confidenceFromScores(laneScores);
      const clarification = decideClarification(laneScores);

      let clarificationAnswer: ClarificationAnswer | null = null;
      if (clarification.shouldAsk && clarification.question && clarification.options) {
        const answer = await ctx.ui.input(clarification.question);
        // A dismissed or empty answer no longer cancels the run: fall back to
        // the best-scored lane only when the answer isn't an explicit option
        // match, then continue the scan.
        const matchedLane = answer ? parseClarificationAnswer(answer, clarification.options) : null;
        if (matchedLane && answer) {
          clarificationAnswer = { lane: matchedLane, answer };
        }
      }

      const effectiveLane = clarificationAnswer?.lane ?? lane;
      ctx.ui.notify("Improvement scan started.", "info");
      // Route the scan prompt into the normal agent turn so the model streams its
      // report into the transcript like any other response — no composer fill.
      pi.sendUserMessage(
        buildUserPrompt(
          request,
          repoSignals,
          effectiveLane,
          confidence,
          clarificationAnswer ? { shouldAsk: false } : clarification
        )
      );
    }
  });
}
