import type { ExtensionAPI, ToolDefinition } from "@earendil-works/pi-coding-agent";
import type { AssistantMessage, ToolCall } from "@earendil-works/pi-ai";
import { BorderedLoader } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";

const COMMAND_NAME = "improvement-scan";
const SUMMARY_TOOL_NAME = "improvement_scan_summary";

type LaneId =
  | "bug-diagnosis"
  | "architecture"
  | "domain-modeling"
  | "documentation"
  | "refactor-style"
  | "generic-improvement-audit";

type QuestionReason = "lane-shape" | "next-step";

type ImprovementScanDetails = {
  lane: LaneId;
  confidence: "high" | "medium" | "low";
  scope: string;
  researchBoundary: string;
  findings: string[];
  evidence: string[];
  recommendations: string[];
  details: LaneDetails;
  toSpecRecommendation?: string;
};

type LaneDefinition = {
  label: string;
  intent: string;
  detailDescription: string;
  keywords: string[];
  pathSignals: string[];
  repoSignals: string[];
};

type BugDiagnosisDetails = {
  symptoms: string[];
  likelyRootCause?: string;
  reproClues: string[];
  debuggingGaps: string[];
};

type ArchitectureDetails = {
  components: string[];
  seams: string[];
  boundaryPressure: string[];
  structuralOptions: string[];
};

type DomainModelingDetails = {
  glossary: string[];
  ambiguities: string[];
  missingConcepts: string[];
  namingRecommendations: string[];
};

type DocumentationDetails = {
  gaps: string[];
  stalePages: string[];
  missingReferences: string[];
  updateTargets: string[];
};

type RefactorStyleDetails = {
  duplication: string[];
  localComplexity: string[];
  cleanupCandidates: string[];
  lowRiskRefactors: string[];
};

type GenericImprovementAuditDetails = {
  rankedOpportunities: string[];
  candidateAreas: string[];
  rationale: string[];
};

type LaneDetails =
  | BugDiagnosisDetails
  | ArchitectureDetails
  | DomainModelingDetails
  | DocumentationDetails
  | RefactorStyleDetails
  | GenericImprovementAuditDetails;

type RepoSignals = {
  hasBackend: boolean;
  hasFrontend: boolean;
  hasDocs: boolean;
  hasAdr: boolean;
  hasContext: boolean;
  hasPiSubagents: boolean;
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

const BUG_DIAGNOSIS_DETAILS_SCHEMA = Type.Object({
  symptoms: Type.Array(Type.String()),
  likelyRootCause: Type.Optional(Type.String()),
  reproClues: Type.Array(Type.String()),
  debuggingGaps: Type.Array(Type.String())
});

const ARCHITECTURE_DETAILS_SCHEMA = Type.Object({
  components: Type.Array(Type.String()),
  seams: Type.Array(Type.String()),
  boundaryPressure: Type.Array(Type.String()),
  structuralOptions: Type.Array(Type.String())
});

const DOMAIN_MODELING_DETAILS_SCHEMA = Type.Object({
  glossary: Type.Array(Type.String()),
  ambiguities: Type.Array(Type.String()),
  missingConcepts: Type.Array(Type.String()),
  namingRecommendations: Type.Array(Type.String())
});

const DOCUMENTATION_DETAILS_SCHEMA = Type.Object({
  gaps: Type.Array(Type.String()),
  stalePages: Type.Array(Type.String()),
  missingReferences: Type.Array(Type.String()),
  updateTargets: Type.Array(Type.String())
});

const REFACTOR_STYLE_DETAILS_SCHEMA = Type.Object({
  duplication: Type.Array(Type.String()),
  localComplexity: Type.Array(Type.String()),
  cleanupCandidates: Type.Array(Type.String()),
  lowRiskRefactors: Type.Array(Type.String())
});

const GENERIC_IMPROVEMENT_AUDIT_DETAILS_SCHEMA = Type.Object({
  rankedOpportunities: Type.Array(Type.String()),
  candidateAreas: Type.Array(Type.String()),
  rationale: Type.Array(Type.String())
});

const DETAILS_SCHEMA = Type.Union([
  Type.Object({
    lane: Type.Literal("bug-diagnosis"),
    confidence: Type.Union([Type.Literal("high"), Type.Literal("medium"), Type.Literal("low")]),
    scope: Type.String({ description: "What the scan covered." }),
    researchBoundary: Type.String({ description: "How the command stayed research-only." }),
    findings: Type.Array(Type.String(), { description: "Improvement opportunities found during the scan." }),
    evidence: Type.Array(Type.String(), { description: "Repository evidence that supports the findings." }),
    recommendations: Type.Array(Type.String(), { description: "Concrete next steps, still research-only." }),
    details: BUG_DIAGNOSIS_DETAILS_SCHEMA,
    toSpecRecommendation: Type.Optional(Type.String({ description: "When /to-spec is warranted, and why." }))
  }),
  Type.Object({
    lane: Type.Literal("architecture"),
    confidence: Type.Union([Type.Literal("high"), Type.Literal("medium"), Type.Literal("low")]),
    scope: Type.String({ description: "What the scan covered." }),
    researchBoundary: Type.String({ description: "How the command stayed research-only." }),
    findings: Type.Array(Type.String(), { description: "Improvement opportunities found during the scan." }),
    evidence: Type.Array(Type.String(), { description: "Repository evidence that supports the findings." }),
    recommendations: Type.Array(Type.String(), { description: "Concrete next steps, still research-only." }),
    details: ARCHITECTURE_DETAILS_SCHEMA,
    toSpecRecommendation: Type.Optional(Type.String({ description: "When /to-spec is warranted, and why." }))
  }),
  Type.Object({
    lane: Type.Literal("domain-modeling"),
    confidence: Type.Union([Type.Literal("high"), Type.Literal("medium"), Type.Literal("low")]),
    scope: Type.String({ description: "What the scan covered." }),
    researchBoundary: Type.String({ description: "How the command stayed research-only." }),
    findings: Type.Array(Type.String(), { description: "Improvement opportunities found during the scan." }),
    evidence: Type.Array(Type.String(), { description: "Repository evidence that supports the findings." }),
    recommendations: Type.Array(Type.String(), { description: "Concrete next steps, still research-only." }),
    details: DOMAIN_MODELING_DETAILS_SCHEMA,
    toSpecRecommendation: Type.Optional(Type.String({ description: "When /to-spec is warranted, and why." }))
  }),
  Type.Object({
    lane: Type.Literal("documentation"),
    confidence: Type.Union([Type.Literal("high"), Type.Literal("medium"), Type.Literal("low")]),
    scope: Type.String({ description: "What the scan covered." }),
    researchBoundary: Type.String({ description: "How the command stayed research-only." }),
    findings: Type.Array(Type.String(), { description: "Improvement opportunities found during the scan." }),
    evidence: Type.Array(Type.String(), { description: "Repository evidence that supports the findings." }),
    recommendations: Type.Array(Type.String(), { description: "Concrete next steps, still research-only." }),
    details: DOCUMENTATION_DETAILS_SCHEMA,
    toSpecRecommendation: Type.Optional(Type.String({ description: "When /to-spec is warranted, and why." }))
  }),
  Type.Object({
    lane: Type.Literal("refactor-style"),
    confidence: Type.Union([Type.Literal("high"), Type.Literal("medium"), Type.Literal("low")]),
    scope: Type.String({ description: "What the scan covered." }),
    researchBoundary: Type.String({ description: "How the command stayed research-only." }),
    findings: Type.Array(Type.String(), { description: "Improvement opportunities found during the scan." }),
    evidence: Type.Array(Type.String(), { description: "Repository evidence that supports the findings." }),
    recommendations: Type.Array(Type.String(), { description: "Concrete next steps, still research-only." }),
    details: REFACTOR_STYLE_DETAILS_SCHEMA,
    toSpecRecommendation: Type.Optional(Type.String({ description: "When /to-spec is warranted, and why." }))
  }),
  Type.Object({
    lane: Type.Literal("generic-improvement-audit"),
    confidence: Type.Union([Type.Literal("high"), Type.Literal("medium"), Type.Literal("low")]),
    scope: Type.String({ description: "What the scan covered." }),
    researchBoundary: Type.String({ description: "How the command stayed research-only." }),
    findings: Type.Array(Type.String(), { description: "Improvement opportunities found during the scan." }),
    evidence: Type.Array(Type.String(), { description: "Repository evidence that supports the findings." }),
    recommendations: Type.Array(Type.String(), { description: "Concrete next steps, still research-only." }),
    details: GENERIC_IMPROVEMENT_AUDIT_DETAILS_SCHEMA,
    toSpecRecommendation: Type.Optional(Type.String({ description: "When /to-spec is warranted, and why." }))
  })
]);

const SUMMARY_SYSTEM_PROMPT = `You are improvement-scanner, a research-only repository improvement analyst.

Your job is to inspect the repository and produce a concise improvement scan summary.

Hard boundaries:
- Research only.
- Do not implement, refactor, edit, or write files.
- Do not produce direct spec text.
- You may recommend next steps, including /to-spec, only when warranted.
- Prefer existing evidence from repository inspection over guesses.

Routing contract:
- Respect the provided routed lane, confidence, and repo signals.
- Use the lane as the primary mode for the investigation.
- Mention when pi-subagents are available and worth using, but degrade gracefully when they are not.
- Ask a clarifying question only when the prompt explicitly says ambiguity would materially change the answer shape or next-step recommendation.

Required output:
- End by calling the improvement_scan_summary tool.
- Keep findings evidence-backed and concise.
- State the research-only boundary explicitly.
- Return the shared summary envelope: lane, confidence, findings, evidence, recommendations, details, and optional /to-spec guidance.
- Mention /to-spec only as a recommendation, never as a generated spec.`;

const TOOL_PROMPT_SNIPPET =
  "Emit the final research-only /improvement-scan summary with lane, confidence, findings, evidence, recommendations, details, and /to-spec guidance.";

const TOOL_PROMPT_GUIDELINES = [
  "Use improvement_scan_summary as the final action for /improvement-scan results.",
  "Do not call the tool until repository inspection is complete.",
  "Keep the result research-only: no implementation steps phrased as completed work, no patches, no direct spec text.",
  "Preserve the routed lane and return lane-specific information under details."
];

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
    hasContext: fileExists("CONTEXT.md"),
    hasPiSubagents:
      fileExists(".pi/extensions/pi-subagents/package.json") || fileExists("pi-extensions/pi-subagents/package.json")
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

function confidenceFromScores(scores: LaneScore[]): ImprovementScanDetails["confidence"] {
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
  confidence: ImprovementScanDetails["confidence"],
  clarification: ClarificationDecision
): string {
  const laneDefinition = LANE_DEFINITIONS[lane];
  const signalLines = [
    signals.hasBackend ? "- backend package present" : null,
    signals.hasFrontend ? "- frontend package present" : null,
    signals.hasDocs ? "- docs/help-book surfaces present" : null,
    signals.hasAdr ? "- ADRs present" : null,
    signals.hasContext ? "- CONTEXT.md present" : null,
    signals.hasPiSubagents
      ? "- pi-subagents likely available; prefer them when they help"
      : "- pi-subagents not detected; degrade gracefully"
  ].filter((line): line is string => line !== null);

  return [
    "Run /improvement-scan.",
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
    "1. Stay strictly on the research side of the line.",
    "2. Use the routed lane as the primary frame unless the evidence clearly forces a nearby lane.",
    "3. Prefer native Pi subagents when present and useful; otherwise continue without them.",
    "4. Do not edit files or write specs.",
    "5. End with a structured summary via improvement_scan_summary.",
    "6. Put lane-specific structure inside details.",
    "7. Recommend /to-spec only if the destination is clear, the findings are synthesized, and a spec adds coordination value.",
    "",
    `Lane-specific details should cover: ${laneDefinition.detailDescription}`,
    "",
    "Return only evidence-backed findings."
  ].join("\n");
}

function renderDetailSection(label: string, values: string[]): string[] {
  return [`### ${label}`, ...(values.length > 0 ? values.map((value) => `- ${value}`) : ["- None"]), ""];
}

function renderLaneDetails(details: ImprovementScanDetails): string[] {
  switch (details.lane) {
    case "bug-diagnosis": {
      const laneDetails = details.details as BugDiagnosisDetails;
      return [
        ...renderDetailSection("Symptoms", laneDetails.symptoms),
        ...renderDetailSection("Likely root cause", laneDetails.likelyRootCause ? [laneDetails.likelyRootCause] : []),
        ...renderDetailSection("Repro clues", laneDetails.reproClues),
        ...renderDetailSection("Debugging gaps", laneDetails.debuggingGaps)
      ];
    }
    case "architecture": {
      const laneDetails = details.details as ArchitectureDetails;
      return [
        ...renderDetailSection("Components", laneDetails.components),
        ...renderDetailSection("Seams", laneDetails.seams),
        ...renderDetailSection("Boundary pressure", laneDetails.boundaryPressure),
        ...renderDetailSection("Structural options", laneDetails.structuralOptions)
      ];
    }
    case "domain-modeling": {
      const laneDetails = details.details as DomainModelingDetails;
      return [
        ...renderDetailSection("Glossary", laneDetails.glossary),
        ...renderDetailSection("Ambiguities", laneDetails.ambiguities),
        ...renderDetailSection("Missing concepts", laneDetails.missingConcepts),
        ...renderDetailSection("Naming recommendations", laneDetails.namingRecommendations)
      ];
    }
    case "documentation": {
      const laneDetails = details.details as DocumentationDetails;
      return [
        ...renderDetailSection("Gaps", laneDetails.gaps),
        ...renderDetailSection("Stale pages", laneDetails.stalePages),
        ...renderDetailSection("Missing references", laneDetails.missingReferences),
        ...renderDetailSection("Update targets", laneDetails.updateTargets)
      ];
    }
    case "refactor-style": {
      const laneDetails = details.details as RefactorStyleDetails;
      return [
        ...renderDetailSection("Duplication", laneDetails.duplication),
        ...renderDetailSection("Local complexity", laneDetails.localComplexity),
        ...renderDetailSection("Cleanup candidates", laneDetails.cleanupCandidates),
        ...renderDetailSection("Low-risk refactors", laneDetails.lowRiskRefactors)
      ];
    }
    case "generic-improvement-audit": {
      const laneDetails = details.details as GenericImprovementAuditDetails;
      return [
        ...renderDetailSection("Ranked opportunities", laneDetails.rankedOpportunities),
        ...renderDetailSection("Candidate areas", laneDetails.candidateAreas),
        ...renderDetailSection("Rationale", laneDetails.rationale)
      ];
    }
  }
}

function renderSummaryText(details: ImprovementScanDetails): string {
  const laneDefinition = LANE_DEFINITIONS[details.lane];
  const sections = [
    "# Improvement scan",
    "",
    `- Lane: ${details.lane} (${laneDefinition.label})`,
    `- Confidence: ${details.confidence}`,
    `- Scope: ${details.scope}`,
    `- Research boundary: ${details.researchBoundary}`,
    ...(details.toSpecRecommendation ? [`- /to-spec: ${details.toSpecRecommendation}`] : []),
    "",
    "## Findings",
    ...(details.findings.length > 0 ? details.findings.map((item) => `- ${item}`) : ["- None"]),
    "",
    "## Evidence",
    ...(details.evidence.length > 0 ? details.evidence.map((item) => `- ${item}`) : ["- None"]),
    "",
    "## Recommendations",
    ...(details.recommendations.length > 0 ? details.recommendations.map((item) => `- ${item}`) : ["- None"]),
    "",
    "## Details",
    ...renderLaneDetails(details)
  ];
  return sections.join("\n");
}

/**
 * Markers every /improvement-scan summary shares, regardless of lane, in the
 * order `renderSummaryText` emits them. Used to distinguish a conforming
 * summary from free-form model output.
 */
const SUMMARY_ENVELOPE_MARKERS = [
  "- Lane:",
  "- Confidence:",
  "## Findings",
  "## Evidence",
  "## Recommendations",
  "## Details"
] as const;

const SUMMARY_TITLE = "# Improvement scan";

/**
 * True when `text` looks like a rendered /improvement-scan summary — i.e. it
 * carries the shared envelope: the title leads and the required fields and
 * sections appear in the fixed rendered order. Free-form prose that merely
 * echoes the heading strings out of order (or omits the contract) is not a
 * valid scan result.
 */
export function validateSummaryEnvelope(text: string): boolean {
  let cursor = text.indexOf(SUMMARY_TITLE);
  if (cursor === -1) {
    return false;
  }
  for (const marker of SUMMARY_ENVELOPE_MARKERS) {
    cursor = text.indexOf(marker, cursor + 1);
    if (cursor === -1) {
      return false;
    }
  }
  return true;
}

/**
 * Extract the rendered /improvement-scan summary from a completed `AssistantMessage`.
 * Prefers the `improvement_scan_summary` tool call (the model's required final action).
 * When the model answers only in prose without invoking the tool, the text is
 * validated against the shared envelope rather than accepted as a successful scan.
 */
export function summaryTextFromResult(result: AssistantMessage): string {
  const summaryCall = result.content.find(
    (part): part is ToolCall => part.type === "toolCall" && part.name === SUMMARY_TOOL_NAME
  );

  if (summaryCall) {
    return renderSummaryText(summaryCall.arguments as ImprovementScanDetails);
  }

  const text = result.content
    .filter((part): part is { type: "text"; text: string } => part.type === "text")
    .map((part) => part.text)
    .join("\n")
    .trim();

  if (text && !validateSummaryEnvelope(text)) {
    return (
      "Improvement scan could not produce a structured summary: the model returned " +
      "free-form output instead of the required summary envelope (findings, evidence, " +
      "recommendations, details)."
    );
  }

  return text || "No improvement scan summary returned.";
}

export default function improvementScanner(pi: ExtensionAPI) {
  // Shared summary tool definition: the pi-ai `Context.tools` entry mirrors the
  // registered tool so the completion sees the same name/description/schema.
  const summaryTool: ToolDefinition = {
    name: SUMMARY_TOOL_NAME,
    label: "Improvement scan summary",
    description:
      "Return the final research-only /improvement-scan summary. Use as the last action after repository inspection.",
    promptSnippet: TOOL_PROMPT_SNIPPET,
    promptGuidelines: TOOL_PROMPT_GUIDELINES,
    parameters: DETAILS_SCHEMA,
    async execute(_toolCallId, params) {
      const details = params as ImprovementScanDetails;
      return {
        content: [{ type: "text", text: renderSummaryText(details) }],
        details,
        terminate: true
      };
    }
  };

  pi.registerTool(summaryTool);

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

      const userMessage = {
        role: "user" as const,
        content: [
          {
            type: "text" as const,
            text: buildUserPrompt(
              request,
              repoSignals,
              effectiveLane,
              confidence,
              clarificationAnswer ? { shouldAsk: false } : clarification
            )
          }
        ],
        timestamp: Date.now()
      };

      const response = await ctx.ui.custom<string | null>((tui, theme, _kb, done) => {
        const loader = new BorderedLoader(tui, theme, `Running /${COMMAND_NAME} with ${ctx.model!.id}...`);
        loader.onAbort = () => done(null);

        const run = async (): Promise<void> => {
          try {
            const result = await ctx.modelRegistry.complete(
              ctx.model!,
              {
                systemPrompt: SUMMARY_SYSTEM_PROMPT,
                messages: [userMessage],
                tools: [
                  {
                    name: summaryTool.name,
                    description: summaryTool.description,
                    parameters: summaryTool.parameters
                  }
                ]
              },
              {
                signal: loader.signal,
                cacheRetention: "none"
              }
            );

            if (result.stopReason === "aborted") {
              done(null);
              return;
            }

            done(summaryTextFromResult(result));
          } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            done(`Improvement scan failed: ${message}`);
          }
        };

        void run();
        return loader;
      });

      if (response === null) {
        ctx.ui.notify("Improvement scan cancelled", "info");
        return;
      }

      ctx.ui.setEditorText(response);
      ctx.ui.notify("Improvement scan loaded into the editor.", "info");
    }
  });
}
