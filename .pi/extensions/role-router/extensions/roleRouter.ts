import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

type RegisterAgentModule = {
  registerAgent: (input: { pi: ExtensionAPI; name: string; definition: Record<string, unknown> }) => {
    dispose(): void;
  };
};

type SpawnSubagentInput = {
  role?: string;
  agent: string;
  task: string;
  async?: boolean;
  context?: string;
  model?: string;
  thinking?: string;
  timeoutMs?: number;
};

type RoutingDecisionSource = "heuristic" | "laya" | "laya-fallback";

type InputRoutingDecision =
  | {
      action: "pass-through";
      reason: "empty" | "slash-command" | "no-match" | "ambiguous";
      candidates?: string[];
      source?: RoutingDecisionSource;
    }
  | {
      action: "delegate";
      role: string;
      task: string;
      score: number;
      context?: StructuredContext;
      source?: RoutingDecisionSource;
    };

type InputRoutingLaunch = {
  role: string;
  dispatchAgent: string;
  receipt?: LaunchReceipt;
};

type RpcRequestEnvelope = {
  version: 1;
  requestId: string;
  method: "spawn";
  params: SpawnSubagentInput;
};

type RpcReplyEnvelope = {
  success?: boolean;
  data?: unknown;
  error?: {
    message?: string;
  };
};

const RPC_ACK_TIMEOUT_MS = 5_000;

type SubagentsRpcModule = {
  spawn: (pi: ExtensionAPI, input: SpawnSubagentInput) => Promise<LaunchReceipt | undefined>;
};

type LayaStartResult = { running?: boolean; exited?: Promise<void>; output?: string; getOutput?: () => string };

const LAYA_STARTUP_OUTPUT_LIMIT = 16 * 1024;

type ValidationSummary = {
  path: string;
  source: "default" | "config";
  roles: Array<{
    role: string;
    enabled: boolean;
    agent: string;
    description: string;
  }>;
};

type LayaStartupModel = "coder" | "english" | "multilingual" | "reasoning" | "small" | "tiny";

type LayaConfig = {
  enabled?: boolean;
  healthUrl?: string;
  startupCommand?: string;
  startupTimeoutMs?: number;
  decisionUrl?: string;
  startupModel?: LayaStartupModel;
  startupPort?: number;
  startupHost?: string;
  startupDevice?: string;
  startupPreload?: boolean;
  startupThreads?: number;
};

type LayaDecisionRequest = {
  state: string;
  questions: {
    route: {
      type: "choice";
      instructions: string;
      criteria: Record<string, string>;
    };
  };
};

type LayaDecisionResponse = {
  answers?: {
    route?: {
      choice?: unknown;
    };
  };
};

type RuntimeSummary = {
  async: boolean;
  context: string;
  timeoutMs: number;
  model?: string;
  thinking?: string;
};

type StructuredContext = {
  summary?: string;
  files?: string[];
  constraints?: string[];
  artifacts?: string[];
};

type TaskPreviewOptions = StructuredContext & {
  roleContext?: string;
};

type LaunchReceipt = {
  runHandle?: string;
  outputReference?: string;
};

type DryRunSummary = {
  requestedRole: string;
  dispatchAgent: string;
  runtime: RuntimeSummary;
  taskPreview: string;
};

type TraceFieldSource = "default" | "config";

type TraceSummary = {
  requestedRole: string;
  configSource: TraceFieldSource;
  dispatchAgent: string;
  fieldSources: {
    agent: TraceFieldSource;
    model: TraceFieldSource;
    thinking: TraceFieldSource;
    async: TraceFieldSource;
    context: TraceFieldSource;
  };
  runtime: RuntimeSummary;
  taskPreview: string;
};

type RoleRouterErrorClass = "input" | "role-config" | "dependency-setup" | "delegated-launch";

type RoleRouterErrorOptions = {
  cause?: unknown;
  role?: string;
};

class RoleRouterError extends Error {
  readonly kind: RoleRouterErrorClass;
  readonly cause?: unknown;

  constructor(kind: RoleRouterErrorClass, message: string, options: RoleRouterErrorOptions = {}) {
    super(message);
    this.name = "RoleRouterError";
    this.kind = kind;
    this.cause = options.cause;
  }
}

async function loadRegisterAgent(): Promise<RegisterAgentModule> {
  const testRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  if (typeof testRegisterAgent === "function") {
    return {
      registerAgent: testRegisterAgent as RegisterAgentModule["registerAgent"]
    };
  }
  try {
    return (await import("pi-subagents/agents")) as RegisterAgentModule;
  } catch (error) {
    try {
      return (await importPiPackageModule("pi-subagents", "src/api/agents.js")) as RegisterAgentModule;
    } catch (fallbackError) {
      throw new RoleRouterError(
        "dependency-setup",
        formatPiSubagentsDependencyMessage("load the pi-subagents role registration bridge", fallbackError),
        { cause: fallbackError }
      );
    }
  }
}

/**
 * Resolve the pi-subagents spawn bridge.
 *
 * Prefer the direct `pi-subagents/extension` export when present; otherwise fall back to
 * emitting the legacy `subagents:rpc:v1:request` event with the same spawn payload.
 */
async function loadSubagentsRpc(): Promise<SubagentsRpcModule> {
  const testSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  if (typeof testSubagentsSpawn === "function") {
    return {
      spawn: async (pi, input) => parseLaunchReceipt(await testSubagentsSpawn(pi, input))
    };
  }

  let loadError: unknown;
  try {
    const extensionModule = (await import("pi-subagents/extension")) as Partial<SubagentsRpcModule>;
    if (typeof extensionModule.spawn === "function") {
      return {
        spawn: async (_pi, input) => {
          try {
            return parseLaunchReceipt(await extensionModule.spawn?.(_pi, input));
          } catch (error) {
            throw new RoleRouterError("delegated-launch", formatLaunchFailure(input.role ?? input.agent, error), {
              cause: error
            });
          }
        }
      };
    }
  } catch (error) {
    loadError = error;
  }

  try {
    const rpcModule = (await importPiPackageModule("pi-subagents", "src/extension/rpc.js")) as {
      SUBAGENT_RPC_REQUEST_EVENT?: string;
    };
    if (typeof rpcModule.SUBAGENT_RPC_REQUEST_EVENT === "string") {
      return {
        spawn: async (pi, input) => emitRpcSpawn(pi, rpcModule.SUBAGENT_RPC_REQUEST_EVENT, input)
      };
    }
  } catch (fallbackError) {
    if (loadError) {
      throw new RoleRouterError(
        "dependency-setup",
        formatPiSubagentsDependencyMessage("load the pi-subagents delegated execution bridge", fallbackError),
        { cause: fallbackError }
      );
    }
  }

  return {
    spawn: async (pi, input) => emitRpcSpawn(pi, "subagents:rpc:v1:request", input)
  };
}

async function emitRpcSpawn(
  pi: ExtensionAPI,
  requestEventName: string,
  input: SpawnSubagentInput
): Promise<LaunchReceipt | undefined> {
  const requestId = `role-router-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
  const replyEventName = `subagents:rpc:v1:reply:${requestId}`;
  const rpcRequest: RpcRequestEnvelope = {
    version: 1,
    requestId,
    method: "spawn",
    params: input
  };

  return await new Promise<LaunchReceipt | undefined>((resolvePromise, rejectPromise) => {
    let settled = false;
    let ackTimeout: ReturnType<typeof setTimeout> | undefined;
    const finish = (settler: () => void) => {
      if (settled) {
        return;
      }
      settled = true;
      if (ackTimeout) {
        clearTimeout(ackTimeout);
      }
      unsubscribe?.();
      settler();
    };
    const unsubscribe = pi.events.on(replyEventName, (rawReply: unknown) => {
      const reply = rawReply as RpcReplyEnvelope;
      finish(() => {
        if (reply?.success === false) {
          rejectPromise(
            new RoleRouterError(
              "delegated-launch",
              formatLaunchFailure(input.role ?? input.agent, reply.error?.message ?? "unknown RPC spawn failure")
            )
          );
          return;
        }
        resolvePromise(parseLaunchReceipt(reply?.data));
      });
    });

    ackTimeout = setTimeout(() => {
      finish(() => {
        rejectPromise(
          new RoleRouterError(
            "delegated-launch",
            formatLaunchFailure(
              input.role ?? input.agent,
              `timed out waiting ${RPC_ACK_TIMEOUT_MS}ms for subagents RPC spawn acknowledgement`
            )
          )
        );
      });
    }, RPC_ACK_TIMEOUT_MS);

    Promise.resolve(pi.events.emit(requestEventName, rpcRequest)).catch((error) => {
      finish(() => {
        rejectPromise(
          new RoleRouterError("delegated-launch", formatLaunchFailure(input.role ?? input.agent, error), {
            cause: error
          })
        );
      });
    });
  });
}

const COMMAND_NAME = "delegate-role";
const DRY_RUN_COMMAND_NAME = "dry-run-role-router";
const TRACE_COMMAND_NAME = "trace-role-router";
const VALIDATE_COMMAND_NAME = "validate-role-router";
const DEFAULT_CONTEXT = "fresh" as const;
const DEFAULT_ASYNC = true;
const DEFAULT_TIMEOUT_MS = 30 * 60 * 1000;
const DEFAULT_LAYA_HEALTH_URL = "http://127.0.0.1:8877/health";
const DEFAULT_LAYA_DECISION_URL = "http://127.0.0.1:8877/v1/systemone";
const DEFAULT_LAYA_STARTUP_COMMAND = "laya-serve";
const DEFAULT_LAYA_STARTUP_TIMEOUT_MS = 5_000;
const LAYA_NON_BLOCKING_PROBE_WAIT_MS = 150;
const ROLE_SETTINGS_PATH = ".pi/role-router.json";
const LAYA_STARTUP_IN_FLIGHT = Symbol.for("role-router:laya-startup-in-flight");

type LayaStartupState = {
  promise?: Promise<boolean>;
  exited?: Promise<void>;
  running?: boolean;
  runtimeOutput?: () => string;
};
const DEFAULT_ROLES = {
  planner: {
    description: "Plan the work, expose trade-offs, and propose a small next step.",
    prompt:
      "You are the planner role. Clarify the task, produce a compact plan, call out trade-offs, and end with the next concrete step.",
    model: undefined,
    thinking: "medium"
  },
  developer: {
    description: "Implement or debug the requested change with a minimal, practical diff.",
    prompt:
      "You are the developer role. Implement or debug the requested change directly, prefer the smallest working diff, and note any risk or follow-up needed.",
    model: undefined,
    thinking: "low"
  },
  reviewer: {
    description: "Review the proposed change for correctness, risk, and missing checks.",
    prompt:
      "You are the reviewer role. Review the work critically, focus on correctness and risk, and report findings before suggestions.",
    model: undefined,
    thinking: "low"
  }
} as const satisfies Record<string, RoleDefinition>;

type RoleContextMode = "fresh" | "fork" | "profile";

type RoleDefinition = {
  description: string;
  prompt: string;
  agent?: string;
  model?: string;
  thinking?: string;
  async?: boolean;
  context?: RoleContextMode;
};

type RoleConfig = Partial<RoleDefinition> & {
  enabled?: boolean;
};

type RoleRouterConfig = {
  roles?: Record<string, RoleConfig>;
  laya?: LayaConfig;
};

const PASS_THROUGH_PATTERNS = [
  /\blist\b/i,
  /\bshow\b/i,
  /\bwhat(?:'s| is| are)?\b/i,
  /\bwhich\b/i,
  /\bnames?\b/i,
  /\bavailable\b/i,
  /\bskills?\b/i,
  /\bskill names?\b/i
] as const;

const AUTO_ROUTE_PATTERNS = {
  planner: [
    /\bmake a plan\b/i,
    /\bplan (?:the|this|that|a)\b/i,
    /\bplan out\b/i,
    /\bdraft a plan\b/i,
    /\brollout plan\b/i,
    /\bimplementation plan\b/i,
    /\bmigration plan\b/i,
    /\bexecution plan\b/i,
    /\bbreak (?:this|it|the work) down\b/i,
    /\bnext steps?\b/i,
    /\btrade-?offs?\b/i,
    /\bproposal\b/i
  ],
  developer: [
    /\bimplement (?:this|that|it|the|a)\b/i,
    /\bfix (?:this|that|it|the|a)\b/i,
    /\bdebug (?:this|that|it|the|a)\b/i,
    /\bpatch (?:this|that|it|the|a)\b/i,
    /\brefactor (?:this|that|it|the|a)\b/i,
    /\bupdate (?:this|that|it|the|a)\b/i,
    /\bwire (?:this|that|it|the|a)\b/i,
    /\bmake (?:this|that|it) work\b/i,
    /\bworking\s+(?:implementation|fix|patch)\b/i,
    /\bapply (?:the|this) patch\b/i,
    /\bship the fix\b/i,
    /\bchange the code\b/i
  ],
  reviewer: [
    /\bcode review\b/i,
    /\breview (?:this|that|it|the|a)\b/i,
    /\breview (?:my|the) diff\b/i,
    /\breview (?:my|the) changes\b/i,
    /\breview for bugs\b/i,
    /\baudit (?:this|that|it|the|a)\b/i,
    /\bcheck\b.*\brisk\b/i,
    /\blook for bugs\b/i,
    /\bsanity-check (?:this|that|it|the|a)\b/i,
    /\breviewer\b.*\bpass\b/i
  ]
} as const satisfies Record<string, readonly RegExp[]>;

type DelegateRoleInput = {
  role: string;
  task: string;
  context?: string | StructuredContext;
  files?: string[];
  constraints?: string[];
};

type DelegateCommandName = typeof COMMAND_NAME | typeof DRY_RUN_COMMAND_NAME | typeof TRACE_COMMAND_NAME;

type RegisteredRole = {
  role: string;
  dispatchAgent: string;
  definition: RoleDefinition;
};

function normalizeRoleKey(value: string): string {
  return value.trim().toLowerCase();
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function getModuleLoadDetail(error: unknown): string {
  if (!(error instanceof Error)) {
    return String(error);
  }
  const code = "code" in error && typeof error.code === "string" ? error.code : undefined;
  return code ? `${code}: ${error.message}` : error.message;
}

function formatPiSubagentsDependencyMessage(action: string, error?: unknown): string {
  const detail = error ? ` Load detail: ${getModuleLoadDetail(error)}` : "";
  return `role-router dependency/setup error: unable to ${action} because pi-subagents is missing, disabled, or failed to load.${detail}`;
}

function formatRoleConfigError(message: string): string {
  return `role-router role/config error: ${message}`;
}

function formatInputError(message: string): string {
  return `role-router input error: ${message}`;
}

function formatLaunchFailure(role: string, error: unknown): string {
  return `role-router delegated launch failed for role '${role}': ${getErrorMessage(error)}`;
}

import { dirname } from "node:path";
import { pathToFileURL } from "node:url";

async function importPiPackageModule(packageName: string, relativeModulePath: string): Promise<unknown> {
  const testImport = (globalThis as Record<string, unknown>).__testPiPackageImport;
  if (typeof testImport === "function") {
    return await testImport(packageName, relativeModulePath);
  }

  const packageRoot = resolvePiExtensionPackageRoot(packageName);
  return import(pathToFileURL(resolve(packageRoot, relativeModulePath)).href);
}

function resolvePiExtensionPackageRoot(packageName: string): string {
  const testPackageRoot = (globalThis as Record<string, unknown>).__testPiExtensionPackageRoot;
  if (typeof testPackageRoot === "function") {
    return String(testPackageRoot(packageName));
  }
  const envRoot = process.env.PI_EXTENSION_NPM_DIR?.trim();
  if (envRoot) {
    return resolve(envRoot, packageName);
  }
  const sessionFile = process.env.PI_SESSION_FILE?.trim();
  if (sessionFile) {
    const sessionRepoRoot = dirname(sessionFile)
      .match(/\/sessions\/--(.+)--$/)?.[1]
      ?.replace(/--/g, "/");
    if (sessionRepoRoot) {
      return resolve("/", sessionRepoRoot, ".pi/npm/node_modules", packageName);
    }
  }
  return resolve(import.meta.dirname, "../../../npm/node_modules", packageName);
}

function normalizeInputText(text: string): string {
  return text.trim();
}

function isSlashCommand(text: string): boolean {
  return text.startsWith("/") || text.startsWith("!");
}

function shouldForcePassThrough(text: string): boolean {
  return PASS_THROUGH_PATTERNS.some((pattern) => pattern.test(text));
}

function scoreAutomaticRole(text: string, role: string): number {
  const patterns = AUTO_ROUTE_PATTERNS[role as keyof typeof AUTO_ROUTE_PATTERNS] ?? [];
  return patterns.reduce((score, pattern) => score + (pattern.test(text) ? 1 : 0), 0);
}

const SLASH_PATH_CANDIDATE_PATTERN = /(?:\.?\.?\/|\/)?[A-Za-z0-9_.][A-Za-z0-9._-]*(?:\/[A-Za-z0-9_.][A-Za-z0-9._-]*)+/g;
const FILELIKE_PATH_PATTERN = /\/?\.?[A-Za-z0-9_./-]+\.[A-Za-z0-9_-]+/;
const NON_PATH_SEGMENT_WORDS = new Set(["for", "with", "from", "into", "about", "under", "using"]);
const CONSTRAINT_PATTERNS = [
  /\b(?:must|should|need to|without|avoid|do not|don't)([^.!?\n;]+)/gi,
  /\b(?:constrained to|constraint:?|guardrails?:?)([^.!?\n;]+)/gi,
  /\bkeep\s+(?![A-Z][a-z]+\/[A-Z][a-z]+)([^.!?\n;]*(?:thin-router boundary|public API|interface|contract|schema)[^.!?\n;]*)/gi
] as const;
const ARTIFACT_PATTERNS = [
  /\b(?:artifact|artifacts|output|outputs|receipt|receipts|report|reports|plan|plans|diff|diffs|patch|patches|log|logs)\b\s*(?:at|in|to|from)?\s*(\/?\.?[A-Za-z0-9_./-]+(?:\.[A-Za-z0-9_-]+)?)/gi,
  /\b(?:write|save|store|append|update|use|reuse|read|inspect)\b[^.!?\n;]*?(\/?\.?[A-Za-z0-9_./-]+\.[A-Za-z0-9_-]+)/gi
] as const;

/** Normalize one explicit slash-delimited path candidate and reject obvious non-path phrases. */
function normalizeExtractedPath(value: string): string | undefined {
  const trimmed = value.trim().replace(/^[('"`\[]+|[)"'`\],:;!?]+$/g, "");
  if (!trimmed || !trimmed.includes("/")) {
    return undefined;
  }
  const segments = trimmed.split("/").filter(Boolean);
  if (segments.length < 2) {
    return undefined;
  }
  if (segments.some((segment) => NON_PATH_SEGMENT_WORDS.has(segment.toLowerCase()))) {
    return undefined;
  }
  const hasStrongPathSignal =
    trimmed.startsWith("./") ||
    trimmed.startsWith("../") ||
    trimmed.startsWith("/") ||
    FILELIKE_PATH_PATTERN.test(trimmed) ||
    segments.some((segment) => segment.includes(".") || segment.includes("-") || segment.includes("_"));
  return hasStrongPathSignal ? trimmed : undefined;
}

/** Keep extracted free-text handoff values stable and duplicate-free. */
function dedupeStrings(values: readonly string[]): string[] | undefined {
  const deduped = [...new Set(values)];
  return deduped.length ? deduped : undefined;
}

/**
 * Extract clearly stated slash-delimited paths from free text without probing the filesystem.
 *
 * ponytail: path-shape-only heuristic; keep explicit cwd, relative, absolute, and temp paths because the user requested all explicit paths to be preserved.
 */
function extractExplicitPaths(text: string): string[] | undefined {
  return dedupeStrings(
    [...text.matchAll(SLASH_PATH_CANDIDATE_PATTERN)]
      .map((match) => normalizeExtractedPath(match[0]))
      .filter((value): value is string => Boolean(value))
  );
}

/** Normalize an extracted constraint phrase and drop vague one-word fragments. */
function normalizeConstraint(value: string): string | undefined {
  const normalized = value.trim().replace(/^[,:\-\s]+|[\s,;:.!?]+$/g, "");
  if (!normalized || normalized.split(/\s+/).length < 4) {
    return undefined;
  }
  return normalized;
}

/** Extract clearly stated guardrails from free text using a small keyword-based heuristic. */
function extractExplicitConstraints(text: string): string[] | undefined {
  return dedupeStrings(
    CONSTRAINT_PATTERNS.flatMap((pattern) =>
      [...text.matchAll(pattern)]
        .map((match) => normalizeConstraint(`${match[0].trim()}`))
        .filter((value): value is string => Boolean(value))
    )
  );
}

/** Normalize an explicit artifact reference while allowing relative, temp, and absolute paths. */
function extractArtifactPath(candidate: string): string | undefined {
  const normalized = candidate.trim().replace(/^[('"`\[]+|[)"'`\],:;!?]+$/g, "");
  if (!normalized || !FILELIKE_PATH_PATTERN.test(normalized.replace(/^\//, ""))) {
    return undefined;
  }
  return normalized;
}

/** Extract clearly stated artifact references from free text without guessing unnamed outputs. */
function extractExplicitArtifacts(text: string): string[] | undefined {
  return dedupeStrings(
    ARTIFACT_PATTERNS.flatMap((pattern) =>
      [...text.matchAll(pattern)]
        .map((match) => extractArtifactPath((match[1] ?? "").trim()))
        .filter((value): value is string => Boolean(value))
    )
  );
}

/** Build the bounded delegated handoff context extracted conservatively from explicit free text only. */
function extractStructuredContextFromFreeText(text: string): StructuredContext | undefined {
  const files = extractExplicitPaths(text);
  const constraints = extractExplicitConstraints(text);
  const artifacts = extractExplicitArtifacts(text);
  if (!files && !constraints && !artifacts) {
    return undefined;
  }
  return {
    ...(files ? { files } : {}),
    ...(constraints ? { constraints } : {}),
    ...(artifacts ? { artifacts } : {})
  };
}

export function decideInputRouting(text: string, enabledRoles: readonly string[]): InputRoutingDecision {
  const normalizedText = normalizeInputText(text);
  if (!normalizedText) {
    return { action: "pass-through", reason: "empty" };
  }
  if (isSlashCommand(normalizedText)) {
    return { action: "pass-through", reason: "slash-command" };
  }
  if (shouldForcePassThrough(normalizedText)) {
    return { action: "pass-through", reason: "no-match" };
  }

  const scoredRoles = enabledRoles
    .map((role) => ({ role, score: scoreAutomaticRole(normalizedText, role) }))
    .filter((entry) => entry.score > 0)
    .sort((left, right) => right.score - left.score || left.role.localeCompare(right.role));

  const bestScore = scoredRoles[0]?.score;
  if (!bestScore) {
    return { action: "pass-through", reason: "no-match" };
  }

  const topMatches = scoredRoles.filter((entry) => entry.score === bestScore);
  if (topMatches.length !== 1) {
    return {
      action: "pass-through",
      reason: "ambiguous",
      candidates: topMatches.map((entry) => entry.role)
    };
  }

  const extractedContext = extractStructuredContextFromFreeText(normalizedText);
  return {
    action: "delegate",
    role: topMatches[0].role,
    task: normalizedText,
    score: topMatches[0].score,
    ...(extractedContext ? { context: extractedContext } : {})
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

const LAYA_STARTUP_MODELS = ["coder", "english", "multilingual", "reasoning", "small", "tiny"] as const;

function validateLayaConfig(value: unknown): LayaConfig {
  if (!isRecord(value)) {
    throw new Error("role-router config 'laya' must be an object.");
  }
  if (value.enabled !== undefined && typeof value.enabled !== "boolean") {
    throw new Error("role-router config 'laya.enabled' must be a boolean when set.");
  }
  if (value.healthUrl !== undefined && (typeof value.healthUrl !== "string" || value.healthUrl.trim() === "")) {
    throw new Error("role-router config 'laya.healthUrl' must be a non-empty string when set.");
  }
  if (value.decisionUrl !== undefined && (typeof value.decisionUrl !== "string" || value.decisionUrl.trim() === "")) {
    throw new Error("role-router config 'laya.decisionUrl' must be a non-empty string when set.");
  }
  if (
    value.startupCommand !== undefined &&
    (typeof value.startupCommand !== "string" || value.startupCommand.trim() === "")
  ) {
    throw new Error("role-router config 'laya.startupCommand' must be a non-empty string when set.");
  }
  if (value.startupArgs !== undefined) {
    throw new Error(
      "role-router config 'laya.startupArgs' is no longer supported; use dedicated startup fields instead."
    );
  }
  if (
    value.startupTimeoutMs !== undefined &&
    (!Number.isInteger(value.startupTimeoutMs) || value.startupTimeoutMs <= 0)
  ) {
    throw new Error("role-router config 'laya.startupTimeoutMs' must be a positive integer when set.");
  }
  if (value.startupModel !== undefined && !LAYA_STARTUP_MODELS.includes(value.startupModel as LayaStartupModel)) {
    throw new Error(
      `role-router config 'laya.startupModel' must be one of ${LAYA_STARTUP_MODELS.map((entry) => `'${entry}'`).join(", ")} when set.`
    );
  }
  if (value.startupPort !== undefined && (!Number.isInteger(value.startupPort) || value.startupPort <= 0)) {
    throw new Error("role-router config 'laya.startupPort' must be a positive integer when set.");
  }
  if (value.startupHost !== undefined && (typeof value.startupHost !== "string" || value.startupHost.trim() === "")) {
    throw new Error("role-router config 'laya.startupHost' must be a non-empty string when set.");
  }
  if (
    value.startupDevice !== undefined &&
    (typeof value.startupDevice !== "string" || value.startupDevice.trim() === "")
  ) {
    throw new Error("role-router config 'laya.startupDevice' must be a non-empty string when set.");
  }
  if (value.startupPreload !== undefined && typeof value.startupPreload !== "boolean") {
    throw new Error("role-router config 'laya.startupPreload' must be a boolean when set.");
  }
  if (value.startupThreads !== undefined && (!Number.isInteger(value.startupThreads) || value.startupThreads <= 0)) {
    throw new Error("role-router config 'laya.startupThreads' must be a positive integer when set.");
  }
  return {
    ...(typeof value.enabled === "boolean" ? { enabled: value.enabled } : {}),
    ...(typeof value.healthUrl === "string" ? { healthUrl: value.healthUrl.trim() } : {}),
    ...(typeof value.decisionUrl === "string" ? { decisionUrl: value.decisionUrl.trim() } : {}),
    ...(typeof value.startupCommand === "string" ? { startupCommand: value.startupCommand.trim() } : {}),
    ...(typeof value.startupTimeoutMs === "number" ? { startupTimeoutMs: value.startupTimeoutMs } : {}),
    ...(typeof value.startupModel === "string" ? { startupModel: value.startupModel as LayaStartupModel } : {}),
    ...(typeof value.startupPort === "number" ? { startupPort: value.startupPort } : {}),
    ...(typeof value.startupHost === "string" ? { startupHost: value.startupHost.trim() } : {}),
    ...(typeof value.startupDevice === "string" ? { startupDevice: value.startupDevice.trim() } : {}),
    ...(typeof value.startupPreload === "boolean" ? { startupPreload: value.startupPreload } : {}),
    ...(typeof value.startupThreads === "number" ? { startupThreads: value.startupThreads } : {})
  };
}

function validateRoleDefinition(role: string, value: unknown): RoleDefinition {
  if (!isRecord(value)) {
    throw new Error(`Role '${role}' must be an object.`);
  }
  if (typeof value.description !== "string" || value.description.trim() === "") {
    throw new Error(`Role '${role}' requires a non-empty description.`);
  }
  if (typeof value.prompt !== "string" || value.prompt.trim() === "") {
    throw new Error(`Role '${role}' requires a non-empty prompt.`);
  }
  if (value.agent !== undefined && (typeof value.agent !== "string" || value.agent.trim() === "")) {
    throw new Error(`Role '${role}' agent must be a non-empty string when set.`);
  }
  if (value.model !== undefined && (typeof value.model !== "string" || value.model.trim() === "")) {
    throw new Error(`Role '${role}' model must be a non-empty string when set.`);
  }
  if (value.thinking !== undefined && (typeof value.thinking !== "string" || value.thinking.trim() === "")) {
    throw new Error(`Role '${role}' thinking must be a non-empty string when set.`);
  }
  if (value.async !== undefined && typeof value.async !== "boolean") {
    throw new Error(`Role '${role}' async must be a boolean when set.`);
  }
  if (
    value.context !== undefined &&
    value.context !== "fresh" &&
    value.context !== "fork" &&
    value.context !== "profile"
  ) {
    throw new Error(`Role '${role}' context must be one of 'fresh', 'fork', or 'profile' when set.`);
  }
  return {
    description: value.description.trim(),
    prompt: value.prompt.trim(),
    ...(typeof value.agent === "string" ? { agent: value.agent.trim() } : {}),
    ...(typeof value.model === "string" ? { model: value.model.trim() } : {}),
    ...(typeof value.thinking === "string" ? { thinking: value.thinking.trim() } : {}),
    ...(typeof value.async === "boolean" ? { async: value.async } : {}),
    ...(typeof value.context === "string" ? { context: value.context } : {})
  };
}

/** Parse `.pi/role-router.json` and validate the supported per-role override shape. */
export function parseRoleRouterConfig(raw: string): RoleRouterConfig {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (error) {
    throw new RoleRouterError("role-config", formatRoleConfigError(`invalid JSON in ${ROLE_SETTINGS_PATH}.`), {
      cause: error
    });
  }
  if (!isRecord(parsed)) {
    throw new Error("role-router config must be a JSON object.");
  }
  const layaValue = parsed.laya;
  const laya = layaValue === undefined ? undefined : validateLayaConfig(layaValue);
  const rolesValue = parsed.roles;
  if (rolesValue === undefined) {
    return { ...(laya ? { laya } : {}) };
  }
  if (!isRecord(rolesValue)) {
    throw new Error("role-router config 'roles' must be an object.");
  }
  const roles: Record<string, RoleConfig> = {};
  for (const [rawRole, rawConfig] of Object.entries(rolesValue)) {
    const role = normalizeRoleKey(rawRole);
    if (!role) {
      throw new Error("role-router config role keys must be non-empty.");
    }
    if (!isRecord(rawConfig)) {
      throw new Error(`Role '${role}' config must be an object.`);
    }
    const { enabled, ...rest } = rawConfig;
    if (enabled !== undefined && typeof enabled !== "boolean") {
      throw new Error(`Role '${role}' enabled must be a boolean when set.`);
    }
    const hasOverrideFields = Object.keys(rest).length > 0;
    roles[role] = {
      ...(hasOverrideFields
        ? validateRoleDefinition(role, { ...(DEFAULT_ROLES[role as keyof typeof DEFAULT_ROLES] ?? {}), ...rest })
        : {}),
      ...(enabled !== undefined ? { enabled } : {})
    };
  }
  return { roles, ...(laya ? { laya } : {}) };
}

/** Read a role-router config file, return null when the file does not exist, and rethrow other filesystem errors. */
async function readTextFile(path: string): Promise<string | null> {
  try {
    return await readFile(resolve(process.cwd(), path), "utf8");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return null;
    }
    throw error;
  }
}

/** Load built-in roles plus any valid project-local overrides from `.pi/role-router.json`. */
export async function loadRoleDefinitions(configPath = ROLE_SETTINGS_PATH): Promise<Record<string, RoleDefinition>> {
  const baseRoles = Object.fromEntries(
    Object.entries(DEFAULT_ROLES).map(([role, definition]) => [role, { ...definition }])
  ) as Record<string, RoleDefinition>;
  const rawConfig = await readTextFile(configPath);
  if (!rawConfig) {
    return baseRoles;
  }
  let parsed: RoleRouterConfig;
  try {
    parsed = parseRoleRouterConfig(rawConfig);
  } catch (error) {
    if (error instanceof RoleRouterError) {
      throw error;
    }
    throw new RoleRouterError("role-config", formatRoleConfigError(getErrorMessage(error)), { cause: error });
  }
  try {
    for (const [role, config] of Object.entries(parsed.roles ?? {})) {
      if (config.enabled === false) {
        delete baseRoles[role];
        continue;
      }
      const current = baseRoles[role];
      if (!current && Object.keys(config).length === 0) {
        throw new Error(`Role '${role}' is configured but missing description/prompt.`);
      }
      const next = {
        ...(current ?? {}),
        ...config
      };
      baseRoles[role] = validateRoleDefinition(role, next);
    }
    return baseRoles;
  } catch (error) {
    if (error instanceof RoleRouterError) {
      throw error;
    }
    throw new RoleRouterError("role-config", formatRoleConfigError(getErrorMessage(error)), { cause: error });
  }
}

function buildAgentName(role: string): string {
  return role;
}

async function canProbeFetch(): Promise<typeof fetch | undefined> {
  const candidate = (globalThis as Record<string, unknown>).fetch;
  return typeof candidate === "function" ? (candidate as typeof fetch) : undefined;
}

/** Probe the optional local Laya health endpoint without surfacing failures into routing. */
async function isLayaHealthy(healthUrl: string): Promise<boolean> {
  const testHealth = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  if (typeof testHealth === "function") {
    const healthy = Boolean(await testHealth(healthUrl));
    (globalThis as Record<string, unknown>).__testLastLayaHealthCheckHealthy = healthy;
    return healthy;
  }
  const fetchImpl = await canProbeFetch();
  if (!fetchImpl) {
    return false;
  }
  try {
    const response = await fetchImpl(healthUrl, { method: "GET" });
    return response.ok;
  } catch {
    return false;
  }
}

/** Start local Laya once as best-effort preflight; launch failures are intentionally swallowed by the caller. */
function buildLayaSpawn(
  command: string,
  laya: LayaConfig
): {
  command: string;
  args: string[];
  env: NodeJS.ProcessEnv;
} {
  const env = { ...process.env };
  if (laya.startupModel) {
    env.LAYA_DEFAULT_MODEL = laya.startupModel;
  }
  if (laya.startupPort !== undefined) {
    env.LAYA_PORT = String(laya.startupPort);
  }
  if (laya.startupHost) {
    env.LAYA_HOST = laya.startupHost;
  }
  if (laya.startupDevice) {
    env.LAYA_DEVICE = laya.startupDevice;
  }
  if (laya.startupPreload !== undefined) {
    env.LAYA_PRELOAD = laya.startupPreload ? "1" : "0";
  }
  if (laya.startupThreads !== undefined) {
    env.LAYA_THREADS = String(laya.startupThreads);
  }
  return { command, args: [], env };
}

async function startLayaProcess(
  laya: Required<Pick<LayaConfig, "startupCommand">> & LayaConfig
): Promise<LayaStartResult> {
  const spawnConfig = buildLayaSpawn(laya.startupCommand, laya);
  const testStart = (globalThis as Record<string, unknown>).__testLayaStartup;
  if (typeof testStart === "function") {
    return (await testStart(spawnConfig.command, spawnConfig.args, spawnConfig.env)) as LayaStartResult;
  }
  const childProcessModule = await import("node:child_process");
  const child = childProcessModule.spawn(spawnConfig.command, spawnConfig.args, {
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
    env: spawnConfig.env
  });
  child.unref();
  let running = true;
  let output = "";
  const appendOutput = (chunk: string) => {
    output = `${output}${chunk}`.slice(-LAYA_STARTUP_OUTPUT_LIMIT);
  };
  child.stdout?.setEncoding("utf8");
  child.stderr?.setEncoding("utf8");
  child.stdout?.on("data", appendOutput);
  child.stderr?.on("data", appendOutput);
  return {
    running,
    output,
    getOutput: () => output,
    exited: new Promise((resolvePromise) => {
      const markStopped = () => {
        running = false;
        resolvePromise();
      };
      child.once("exit", markStopped);
      child.once("error", markStopped);
    })
  };
}

function getLayaStartupState(key: string): LayaStartupState {
  const state = globalThis as Record<PropertyKey, unknown>;
  const existing = state[LAYA_STARTUP_IN_FLIGHT];
  const startupStates = existing instanceof Map ? existing : new Map<string, LayaStartupState>();
  if (!(existing instanceof Map)) {
    state[LAYA_STARTUP_IN_FLIGHT] = startupStates;
  }
  const current = startupStates.get(key);
  if (current) {
    return current;
  }
  const created: LayaStartupState = {};
  startupStates.set(key, created);
  return created;
}

function getLayaStartupKey(laya: LayaConfig): string {
  return JSON.stringify([
    laya.healthUrl,
    laya.startupCommand,
    laya.startupModel,
    laya.startupPort,
    laya.startupHost,
    laya.startupDevice,
    laya.startupPreload,
    laya.startupThreads
  ]);
}

/** Wait only for a tiny local probe window during routing; longer startup continues off the critical path. */
async function waitForLayaReady(healthUrl: string, startupTimeoutMs: number): Promise<boolean> {
  const deadline = Date.now() + Math.min(startupTimeoutMs, LAYA_NON_BLOCKING_PROBE_WAIT_MS);
  while (Date.now() < deadline) {
    if (await isLayaHealthy(healthUrl)) {
      return true;
    }
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 100));
  }
  return false;
}

/** Resolve the optional Laya preflight settings onto one bounded runtime config. */
function getResolvedLayaConfig(config: RoleRouterConfig): Required<
  Pick<LayaConfig, "healthUrl" | "decisionUrl" | "startupCommand" | "startupTimeoutMs">
> &
  Omit<LayaConfig, "healthUrl" | "decisionUrl" | "startupCommand" | "startupTimeoutMs"> & {
    enabled: boolean;
  } {
  return {
    enabled: config.laya?.enabled === true,
    healthUrl: config.laya?.healthUrl ?? DEFAULT_LAYA_HEALTH_URL,
    decisionUrl: config.laya?.decisionUrl ?? DEFAULT_LAYA_DECISION_URL,
    startupCommand: config.laya?.startupCommand ?? DEFAULT_LAYA_STARTUP_COMMAND,
    startupTimeoutMs: config.laya?.startupTimeoutMs ?? DEFAULT_LAYA_STARTUP_TIMEOUT_MS,
    ...(config.laya?.startupModel ? { startupModel: config.laya.startupModel } : {}),
    ...(config.laya?.startupPort !== undefined ? { startupPort: config.laya.startupPort } : {}),
    ...(config.laya?.startupHost ? { startupHost: config.laya.startupHost } : {}),
    ...(config.laya?.startupDevice ? { startupDevice: config.laya.startupDevice } : {}),
    ...(config.laya?.startupPreload !== undefined ? { startupPreload: config.laya.startupPreload } : {}),
    ...(config.laya?.startupThreads !== undefined ? { startupThreads: config.laya.startupThreads } : {})
  };
}

/** Parse role-router config for the optional Laya seam, preserving shared role/config errors. */
async function readLayaConfig(configPath: string): Promise<RoleRouterConfig | null> {
  const rawConfig = await readTextFile(configPath);
  if (!rawConfig) {
    return null;
  }
  return parseRoleRouterConfig(rawConfig);
}

/**
 * Best-effort local Laya preflight for automatic routing only.
 *
 * ponytail: malformed config, missing fetch, failed startup, and readiness timeout all degrade to no-op so the existing
 * heuristic role routing stays in control; startup readiness only gets a tiny probe window to avoid blocking launches.
 */
export async function maybeEnsureLayaRuntime(configPath = ROLE_SETTINGS_PATH): Promise<boolean> {
  const parsed = await readLayaConfig(configPath);
  if (!parsed) {
    return undefined;
  }
  const laya = getResolvedLayaConfig(parsed);
  if (!laya.enabled) {
    return undefined;
  }
  const startupKey = getLayaStartupKey(laya);
  const startupState = getLayaStartupState(startupKey);
  if (await isLayaHealthy(laya.healthUrl)) {
    return true;
  }

  if (!startupState.promise) {
    startupState.promise = (async () => {
      if (startupState.running) {
        return await waitForLayaReady(laya.healthUrl, laya.startupTimeoutMs);
      }
      try {
        const started = await startLayaProcess(laya);
        startupState.exited = started.exited;
        startupState.running = started.running ?? false;
        startupState.runtimeOutput = started.getOutput
          ? () => started.getOutput?.() ?? started.output ?? ""
          : () => started.output ?? "";
        void started.exited?.finally(() => {
          startupState.running = false;
          startupState.runtimeOutput = undefined;
        });
      } catch {
        startupState.exited = undefined;
        startupState.running = false;
        startupState.runtimeOutput = undefined;
        return false;
      }
      return await waitForLayaReady(laya.healthUrl, laya.startupTimeoutMs);
    })();
    void startupState.promise.finally(() => {
      if (startupState.promise) {
        startupState.promise = undefined;
      }
    });
    void startupState.exited?.finally(() => {
      if (startupState.exited) {
        startupState.exited = undefined;
      }
    });
  }

  await startupState.promise;
  return Boolean(await startupState.promise);
}

function getLayaFetch(): typeof fetch | undefined {
  const testFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  if (typeof testFetch === "function") {
    return testFetch as typeof fetch;
  }
  return getGlobalFetch();
}

function buildLayaDecisionRequest(state: string, enabledRoles: readonly string[]): LayaDecisionRequest {
  return {
    state,
    questions: {
      route: {
        type: "choice",
        instructions:
          "Choose pass_through when the message should stay in the main session, otherwise choose exactly one enabled configured role.",
        criteria: Object.fromEntries([
          ["pass_through", "Keep the message in the main session; do not delegate."],
          ...enabledRoles.map((role) => [role, `Delegate this message to the configured ${role} role.`])
        ])
      }
    }
  };
}

function parseLayaChoice(response: unknown): string | undefined {
  if (!isRecord(response)) {
    return undefined;
  }
  const answers = response.answers;
  if (!isRecord(answers)) {
    return undefined;
  }
  const route = answers.route;
  if (!isRecord(route) || typeof route.choice !== "string") {
    return undefined;
  }
  const choice = route.choice.trim();
  return choice || undefined;
}

/** Ask Laya for one typed route choice and accept only pass-through or one enabled configured role. */
export async function decideInputRoutingWithLaya(
  text: string,
  enabledRoles: readonly string[],
  configPath = ROLE_SETTINGS_PATH
): Promise<InputRoutingDecision | undefined> {
  const normalizedText = normalizeInputText(text);
  if (!normalizedText || enabledRoles.length === 0) {
    return undefined;
  }

  const parsed = await readLayaConfig(configPath);
  if (!parsed) {
    return undefined;
  }

  const laya = getResolvedLayaConfig(parsed);
  if (!laya.enabled) {
    return undefined;
  }

  const fetchFn = getLayaFetch();
  if (!fetchFn) {
    return undefined;
  }

  const response = await fetchFn(laya.decisionUrl, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(buildLayaDecisionRequest(normalizedText, enabledRoles))
  });
  if (!response.ok) {
    return undefined;
  }

  const choice = parseLayaChoice((await response.json()) as LayaDecisionResponse);
  if (!choice) {
    return undefined;
  }
  if (choice === "pass_through") {
    return { action: "pass-through", reason: "no-match", source: "laya" };
  }
  if (!enabledRoles.includes(choice)) {
    return undefined;
  }

  const extractedContext = extractStructuredContextFromFreeText(normalizedText);
  return {
    action: "delegate",
    role: choice,
    task: normalizedText,
    score: 1,
    source: "laya",
    ...(extractedContext ? { context: extractedContext } : {})
  };
}

function buildTaskPrompt(
  role: string,
  prompt: string,
  task: string,
  context?: string,
  options: TaskPreviewOptions = {}
): string {
  return [
    prompt,
    `Role: ${role}`,
    `Task:\n${task.trim()}`,
    options.summary?.trim() ? `\nSummary:\n${options.summary.trim()}` : null,
    options.files?.length ? `\nFiles:\n${options.files.map((file) => `- ${file}`).join("\n")}` : null,
    options.constraints?.length
      ? `\nConstraints:\n${options.constraints.map((constraint) => `- ${constraint}`).join("\n")}`
      : null,
    options.artifacts?.length
      ? `\nArtifacts:\n${options.artifacts.map((artifact) => `- ${artifact}`).join("\n")}`
      : null,
    context?.trim()
      ? `\nExtra context:\n${context.trim()}`
      : options.roleContext
        ? `\nRole context:\n${options.roleContext}`
        : null
  ]
    .filter((value): value is string => Boolean(value))
    .join("\n");
}

/** Build the delegated pi-subagents spawn payload for one `/delegate-role` request. */
function getStructuredContext(input: DelegateRoleInput): StructuredContext | undefined {
  return typeof input.context === "object" ? input.context : undefined;
}

export function buildSpawnInput(match: RegisteredRole, input: DelegateRoleInput): SpawnSubagentInput {
  const structuredContext = getStructuredContext(input);
  return {
    role: input.role,
    agent: match.dispatchAgent,
    task: buildTaskPrompt(
      input.role,
      match.definition.prompt,
      input.task,
      typeof input.context === "string" ? input.context : undefined,
      {
        roleContext: input.context ? undefined : match.definition.context,
        summary: structuredContext?.summary,
        files: structuredContext?.files ?? input.files,
        constraints: structuredContext?.constraints ?? input.constraints,
        artifacts: structuredContext?.artifacts
      }
    ),
    async: match.definition.async ?? DEFAULT_ASYNC,
    context: match.definition.context ?? DEFAULT_CONTEXT,
    timeoutMs: DEFAULT_TIMEOUT_MS,
    ...(match.definition.model ? { model: match.definition.model } : {}),
    ...(match.definition.thinking ? { thinking: match.definition.thinking } : {})
  };
}

function parseStringListField(
  value: unknown,
  commandName: DelegateCommandName,
  fieldName: "files" | "constraints" | "artifacts"
) {
  if (!Array.isArray(value)) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} '${fieldName}' must be an array of non-empty strings when set.`)
    );
  }
  const normalized = value.map((entry) => {
    if (typeof entry !== "string" || entry.trim() === "") {
      throw new RoleRouterError(
        "input",
        formatInputError(`${commandName} '${fieldName}' must be an array of non-empty strings when set.`)
      );
    }
    return entry.trim();
  });
  return normalized;
}

function parseStructuredContext(value: unknown, commandName: DelegateCommandName): StructuredContext {
  if (!isRecord(value)) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} 'context' must be a non-empty string or structured object when set.`)
    );
  }

  const allowedKeys = ["summary", "files", "constraints", "artifacts"] as const;
  for (const key of Object.keys(value)) {
    if (!allowedKeys.includes(key as (typeof allowedKeys)[number])) {
      throw new RoleRouterError("input", formatInputError(`${commandName} context contains unknown key '${key}'.`));
    }
  }

  const structuredContext = {
    ...(typeof value.summary === "string" && value.summary.trim() ? { summary: value.summary.trim() } : {}),
    ...(value.files !== undefined ? { files: parseStringListField(value.files, commandName, "files") } : {}),
    ...(value.constraints !== undefined
      ? { constraints: parseStringListField(value.constraints, commandName, "constraints") }
      : {}),
    ...(value.artifacts !== undefined
      ? { artifacts: parseStringListField(value.artifacts, commandName, "artifacts") }
      : {})
  };

  if (value.summary !== undefined && structuredContext.summary === undefined) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} context 'summary' must be a non-empty string when set.`)
    );
  }

  if (Object.keys(structuredContext).length === 0) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} context object must include at least one non-empty supported field.`)
    );
  }

  return structuredContext;
}

/** Parse one role-router v1 JSON command contract. */
function parseRoleCommandInput(args: string, commandName: DelegateCommandName): DelegateRoleInput {
  const trimmed = args.trim();
  if (!trimmed) {
    throw new RoleRouterError(
      "input",
      formatInputError(`Usage: /${commandName} {"role":"planner","task":"...","context":"optional"}`)
    );
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch (error) {
    throw new RoleRouterError("input", formatInputError(`${commandName} input must be valid JSON.`), { cause: error });
  }
  if (!isRecord(parsed)) {
    throw new RoleRouterError("input", formatInputError(`${commandName} input must be a JSON object.`));
  }
  if (typeof parsed.role !== "string" || parsed.role.trim() === "") {
    throw new RoleRouterError("input", formatInputError(`${commandName} input requires a non-empty 'role'.`));
  }
  if (typeof parsed.task !== "string" || parsed.task.trim() === "") {
    throw new RoleRouterError("input", formatInputError(`${commandName} input requires a non-empty 'task'.`));
  }
  if (parsed.context !== undefined && typeof parsed.context !== "string" && !isRecord(parsed.context)) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} 'context' must be a non-empty string or structured object when set.`)
    );
  }

  const nestedStructuredContext = isRecord(parsed.context)
    ? parseStructuredContext(parsed.context, commandName)
    : undefined;
  if (nestedStructuredContext?.files && parsed.files !== undefined) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} cannot set both top-level 'files' and context 'files'.`)
    );
  }
  if (nestedStructuredContext?.constraints && parsed.constraints !== undefined) {
    throw new RoleRouterError(
      "input",
      formatInputError(`${commandName} cannot set both top-level 'constraints' and context 'constraints'.`)
    );
  }

  const topLevelFiles =
    parsed.files !== undefined ? parseStringListField(parsed.files, commandName, "files") : undefined;
  const topLevelConstraints =
    parsed.constraints !== undefined ? parseStringListField(parsed.constraints, commandName, "constraints") : undefined;

  const normalizedContext =
    typeof parsed.context === "string"
      ? parsed.context.trim()
      : nestedStructuredContext || topLevelFiles || topLevelConstraints
        ? {
            ...(nestedStructuredContext ?? {}),
            ...(topLevelFiles ? { files: topLevelFiles } : {}),
            ...(topLevelConstraints ? { constraints: topLevelConstraints } : {})
          }
        : undefined;

  return {
    role: normalizeRoleKey(parsed.role),
    task: parsed.task.trim(),
    ...(normalizedContext !== undefined ? { context: normalizedContext } : {}),
    ...(typeof parsed.context === "string" && topLevelFiles ? { files: topLevelFiles } : {}),
    ...(typeof parsed.context === "string" && topLevelConstraints ? { constraints: topLevelConstraints } : {})
  };
}

/** Parse the `/delegate-role` v1 JSON contract. */
export function parseDelegateRoleInput(args: string): DelegateRoleInput {
  return parseRoleCommandInput(args, COMMAND_NAME);
}

/** Parse the `/dry-run-role-router` v1 JSON contract. */
export function parseDryRunRoleRouterInput(args: string): DelegateRoleInput {
  return parseRoleCommandInput(args, DRY_RUN_COMMAND_NAME);
}

/** Resolve the selected role against the shared role-definition seam. */
export async function resolveRegisteredRole(input: DelegateRoleInput): Promise<RegisteredRole> {
  const roles = await loadRoleDefinitions();
  const selectedRole = roles[input.role];
  if (selectedRole) {
    return {
      role: input.role,
      dispatchAgent: selectedRole.agent ?? buildAgentName(input.role),
      definition: selectedRole
    };
  }

  const rawConfig = await readTextFile(ROLE_SETTINGS_PATH);
  let disabledRole = false;
  if (rawConfig) {
    try {
      disabledRole = parseRoleRouterConfig(rawConfig).roles?.[input.role]?.enabled === false;
    } catch {
      // loadRoleDefinitions already classifies malformed config at the shared seam.
    }
  }

  throw new RoleRouterError(
    "role-config",
    formatRoleConfigError(
      disabledRole ? `role '${input.role}' exists but is disabled.` : `unknown role '${input.role}'.`
    )
  );
}

function readReceiptStringAlias(value: Record<string, unknown>, fieldNames: readonly string[]): string | undefined {
  for (const fieldName of fieldNames) {
    const candidate = value[fieldName];
    if (typeof candidate === "string" && candidate.trim() !== "") {
      return candidate.trim();
    }
  }
  return undefined;
}

function parseLaunchReceipt(value: unknown): LaunchReceipt | undefined {
  if (!isRecord(value)) {
    return undefined;
  }

  const runHandle = readReceiptStringAlias(value, ["runId", "id", "asyncId"]);
  const outputReference = readReceiptStringAlias(value, ["outputReference", "outputPath", "output"]);
  if (!runHandle && !outputReference) {
    return undefined;
  }

  return {
    ...(runHandle ? { runHandle } : {}),
    ...(outputReference ? { outputReference } : {})
  };
}

type LaunchReceiptView = {
  role: string;
  dispatchAgent: string;
  receipt?: LaunchReceipt;
};

function formatInputRoutingReceipt(summary: InputRoutingLaunch): string {
  return formatLaunchReceipt(summary);
}

export function formatInputRoutingDecision(decision: InputRoutingDecision): string {
  const sourceSuffix =
    decision.source === "laya"
      ? " via Laya"
      : decision.source === "laya-fallback"
        ? " via heuristic fallback after Laya"
        : "";

  if (decision.action === "delegate") {
    return `role-router automatic routing: route to ${decision.role} (score: ${decision.score})${sourceSuffix}`;
  }

  if (decision.reason === "ambiguous") {
    return `role-router automatic routing: pass through (ambiguous between ${decision.candidates?.join(", ") ?? "multiple roles"})${sourceSuffix}`;
  }

  if (decision.reason === "no-match") {
    return `role-router automatic routing: pass through (no high-confidence delegated role matched)${sourceSuffix}`;
  }

  if (decision.reason === "slash-command") {
    return `role-router automatic routing: pass through (slash commands are not auto-routed)${sourceSuffix}`;
  }

  return `role-router automatic routing: pass through (empty input)${sourceSuffix}`;
}

function formatLaunchReceipt({ role, dispatchAgent, receipt }: LaunchReceiptView): string {
  const lines = [`role-router launch receipt: ${role}`, `agent: ${dispatchAgent}`];
  if (receipt?.runHandle) {
    lines.push(`handle: ${receipt.runHandle}`);
  }
  if (receipt?.outputReference) {
    lines.push(`output: ${receipt.outputReference}`);
  }
  return lines.join("\n");
}

async function launchSelectedRole(
  pi: ExtensionAPI,
  subagentsRpc: SubagentsRpcModule,
  selectedRole: RegisteredRole,
  input: DelegateRoleInput
): Promise<LaunchReceipt | undefined> {
  return subagentsRpc.spawn(pi, buildSpawnInput(selectedRole, input));
}

function notifyLaunchReceipt(
  ui: { notify: (message: string, level: "info" | "error") => void },
  selectedRole: RegisteredRole,
  role: string,
  receipt?: LaunchReceipt
): void {
  ui.notify(
    formatInputRoutingReceipt({
      role,
      dispatchAgent: selectedRole.dispatchAgent,
      receipt
    }),
    "info"
  );
}

function notifyAutomaticRoutingExplanation(
  ui: { notify: (message: string, level: "info" | "error" | "warning") => void },
  decision: InputRoutingDecision
): void {
  ui.notify(formatInputRoutingDecision(decision), "info");
}

function buildRuntimeSummary(spawnInput: SpawnSubagentInput): RuntimeSummary {
  return {
    async: spawnInput.async ?? DEFAULT_ASYNC,
    context: spawnInput.context ?? DEFAULT_CONTEXT,
    timeoutMs: spawnInput.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    ...(spawnInput.model ? { model: spawnInput.model } : {}),
    ...(spawnInput.thinking ? { thinking: spawnInput.thinking } : {})
  };
}

function readConfigOverride(configPath: string, role: string): Promise<RoleConfig | null> {
  return readTextFile(configPath).then((rawConfig) => {
    if (!rawConfig) {
      return null;
    }
    return parseRoleRouterConfig(rawConfig).roles?.[role] ?? null;
  });
}

/** Build the preview summary from the same resolved role and spawn payload seam used by live execution. */
export function buildDryRunSummary(selectedRole: RegisteredRole, input: DelegateRoleInput): DryRunSummary {
  const spawnInput = buildSpawnInput(selectedRole, input);
  return {
    requestedRole: input.role,
    dispatchAgent: spawnInput.agent,
    runtime: buildRuntimeSummary(spawnInput),
    taskPreview: spawnInput.task
  };
}

/** Build the routing trace summary, including default-vs-config provenance for the effective role. */
export async function buildTraceSummary(
  selectedRole: RegisteredRole,
  input: DelegateRoleInput,
  configPath = ROLE_SETTINGS_PATH
): Promise<TraceSummary> {
  const spawnInput = buildSpawnInput(selectedRole, input);
  const override = await readConfigOverride(configPath, input.role);
  const fieldSources = {
    agent: override?.agent ? "config" : "default",
    model: override?.model ? "config" : "default",
    thinking: override?.thinking ? "config" : "default",
    async: override?.async !== undefined ? "config" : "default",
    context: override?.context !== undefined ? "config" : "default"
  } as const;
  return {
    requestedRole: input.role,
    configSource: override ? "config" : "default",
    dispatchAgent: spawnInput.agent,
    fieldSources,
    runtime: buildRuntimeSummary(spawnInput),
    taskPreview: spawnInput.task
  };
}

/** Format the stable human-readable `/dry-run-role-router` success output. */
export function formatDryRunSummary(summary: DryRunSummary): string {
  return [
    `role-router dry run: ${summary.requestedRole}`,
    `agent: ${summary.dispatchAgent}`,
    `model: ${summary.runtime.model ?? "default"}`,
    `thinking: ${summary.runtime.thinking ?? "default"}`,
    `async: ${summary.runtime.async ? "true" : "false"}`,
    `context: ${summary.runtime.context}`,
    `timeoutMs: ${summary.runtime.timeoutMs}`,
    "task preview:",
    summary.taskPreview
  ].join("\n");
}

/** Format the stable human-readable `/trace-role-router` success output. */
export function formatTraceSummary(summary: TraceSummary): string {
  return [
    `role-router trace: ${summary.requestedRole}`,
    `config source: ${summary.configSource === "config" ? "project config" : "built-in defaults"}`,
    `agent: ${summary.dispatchAgent} (${summary.fieldSources.agent})`,
    `model: ${summary.runtime.model ?? "default"} (${summary.fieldSources.model})`,
    `thinking: ${summary.runtime.thinking ?? "default"} (${summary.fieldSources.thinking})`,
    `async: ${summary.runtime.async ? "true" : "false"} (${summary.fieldSources.async})`,
    `context: ${summary.runtime.context} (${summary.fieldSources.context})`,
    `timeoutMs: ${summary.runtime.timeoutMs}`,
    "task preview:",
    summary.taskPreview
  ].join("\n");
}

/** Parse the `/validate-role-router` input, defaulting to `.pi/role-router.json` when omitted. */
export function parseValidateRoleRouterInput(args: string): { path: string } {
  const trimmed = args.trim();
  if (!trimmed) {
    return { path: ROLE_SETTINGS_PATH };
  }
  const parsed: unknown = JSON.parse(trimmed);
  if (!isRecord(parsed)) {
    throw new Error("validate-role-router input must be a JSON object.");
  }
  if (typeof parsed.path !== "string" || parsed.path.trim() === "") {
    throw new Error("validate-role-router input requires a non-empty 'path' when set.");
  }
  return { path: parsed.path.trim() };
}

/** Validate one role-router config path and summarize the resolved enabled and disabled roles. */
export async function validateRoleRouterConfig(configPath = ROLE_SETTINGS_PATH): Promise<ValidationSummary> {
  const rawConfig = await readTextFile(configPath);
  const parsed = rawConfig ? parseRoleRouterConfig(rawConfig) : {};
  const resolved = await loadRoleDefinitions(configPath);
  const disabledRoles = Object.entries(parsed.roles ?? {})
    .filter(([, config]) => config.enabled === false)
    .map(([role, config]) => ({
      role,
      enabled: false,
      agent: config.agent ?? buildAgentName(role),
      description:
        config.description ?? DEFAULT_ROLES[role as keyof typeof DEFAULT_ROLES]?.description ?? "Disabled role"
    }));
  const enabledRoles = Object.entries(resolved).map(([role, definition]) => ({
    role,
    enabled: true,
    agent: definition.agent ?? buildAgentName(role),
    description: definition.description
  }));

  return {
    path: configPath,
    source: rawConfig ? "config" : "default",
    roles: [...enabledRoles, ...disabledRoles].sort((left, right) => left.role.localeCompare(right.role))
  };
}

/** Format the stable human-readable `/validate-role-router` success output. */
export function formatValidationSummary(summary: ValidationSummary): string {
  const header = [
    `role-router config valid: ${summary.path}`,
    `source: ${summary.source === "config" ? "project config" : "built-in defaults"}`,
    "resolved roles:"
  ];
  const roles = summary.roles.map(
    ({ role, enabled, agent, description }) =>
      `- ${role}: ${enabled ? "enabled" : "disabled"} (agent: ${agent}) — ${description}`
  );
  return [...header, ...roles].join("\n");
}

/** Resolve one delegated target per role and return the actual dispatch target for each role. */
export async function registerRoleAgents(_pi: ExtensionAPI): Promise<RegisteredRole[]> {
  const roles = await loadRoleDefinitions();
  return Object.entries(roles).map(([role, definition]) => ({
    role,
    dispatchAgent: definition.agent ?? role,
    definition
  }));
}

function createCommandPrompt(roles: RegisteredRole[]): string {
  const listedRoles = roles.map(({ role, definition }) => `- ${role}: ${definition.description}`).join("\n");
  return [
    'Use JSON: /delegate-role {"role":"planner","task":"...","context":{"summary":"optional","files":["path/to/file"],"constraints":["keep the diff small"],"artifacts":["tmp/plan.md"]}}',
    "Available roles:",
    listedRoles
  ].join("\n");
}

export default function roleRouter(pi: ExtensionAPI): void {
  let registeredRolesPromise: Promise<RegisteredRole[]> | null = null;
  let subagentsRpcPromise: Promise<SubagentsRpcModule> | null = null;

  if (typeof pi.on === "function") {
    pi.on("input", async (event, ctx) => {
      if (ctx.mode !== "tui" || event.source === "extension" || event.images?.length) {
        return { action: "continue" };
      }

      let roles: RegisteredRole[];
      try {
        roles = await ensureRoles();
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return { action: "continue" };
      }

      const enabledRoleNames = roles.map((entry) => entry.role);
      const heuristicDecision = decideInputRouting(event.text, enabledRoleNames);
      let decision = heuristicDecision;

      try {
        const parsedConfig = await readLayaConfig();
        const resolvedLaya = parsedConfig ? getResolvedLayaConfig(parsedConfig) : undefined;
        const layaReady = resolvedLaya ? await maybeEnsureLayaRuntime() : false;
        const layaDecision = layaReady ? await decideInputRoutingWithLaya(event.text, enabledRoleNames) : undefined;
        if (layaDecision) {
          decision = layaDecision;
        } else {
          decision = {
            ...heuristicDecision,
            source: "laya-fallback"
          };
        }
      } catch {
        // ponytail: optional preflight only; malformed or failed local runtime checks must not block routing.
        decision = {
          ...heuristicDecision,
          source: "laya-fallback"
        };
      }
      if (decision.action === "pass-through") {
        notifyAutomaticRoutingExplanation(ctx.ui, decision);
        return { action: "continue" };
      }

      const selectedRole = roles.find((entry) => entry.role === decision.role);
      if (!selectedRole) {
        return { action: "continue" };
      }

      notifyAutomaticRoutingExplanation(ctx.ui, decision);

      let subagentsRpc: SubagentsRpcModule;
      try {
        subagentsRpc = await ensureSubagentsRpc();
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return { action: "continue" };
      }

      try {
        const receipt = await launchSelectedRole(pi, subagentsRpc, selectedRole, {
          role: decision.role,
          task: decision.task,
          ...(decision.context ? { context: decision.context } : {})
        });
        notifyLaunchReceipt(ctx.ui, selectedRole, decision.role, receipt);
        return { action: "handled" };
      } catch (error) {
        ctx.ui.notify(
          error instanceof RoleRouterError ? error.message : formatLaunchFailure(decision.role, error),
          "error"
        );
        return { action: "continue" };
      }
    });
  }

  const ensureRoles = () => {
    if (!registeredRolesPromise) {
      registeredRolesPromise = registerRoleAgents(pi);
    }
    return registeredRolesPromise;
  };

  const ensureSubagentsRpc = () => {
    if (!subagentsRpcPromise) {
      subagentsRpcPromise = loadSubagentsRpc();
    }
    return subagentsRpcPromise;
  };

  pi.registerCommand(COMMAND_NAME, {
    description: "Delegate work by configured role",
    handler: async (args, ctx) => {
      if (ctx.mode !== "tui") {
        ctx.ui.notify(`/${COMMAND_NAME} requires interactive mode`, "error");
        return;
      }

      let roles: RegisteredRole[];
      try {
        roles = await ensureRoles();
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return;
      }
      if (args.trim() === "") {
        ctx.ui.notify(createCommandPrompt(roles), "info");
        return;
      }

      let input: DelegateRoleInput;
      try {
        input = parseDelegateRoleInput(args);
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return;
      }

      const selectedRole = roles.find((entry) => entry.role === input.role);
      if (!selectedRole) {
        try {
          await resolveRegisteredRole(input);
        } catch (error) {
          ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        }
        return;
      }

      let subagentsRpc: SubagentsRpcModule;
      try {
        subagentsRpc = await ensureSubagentsRpc();
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return;
      }

      try {
        const receipt = await launchSelectedRole(pi, subagentsRpc, selectedRole, input);
        notifyLaunchReceipt(ctx.ui, selectedRole, input.role, receipt);
      } catch (error) {
        ctx.ui.notify(
          error instanceof RoleRouterError ? error.message : formatLaunchFailure(input.role, error),
          "error"
        );
      }
    }
  });

  pi.registerCommand(DRY_RUN_COMMAND_NAME, {
    description: "Preview the delegated target for one role without launching delegated execution",
    handler: async (args, ctx) => {
      if (ctx.mode !== "tui") {
        ctx.ui.notify(`/${DRY_RUN_COMMAND_NAME} requires interactive mode`, "error");
        return;
      }

      let input: DelegateRoleInput;
      try {
        input = parseDryRunRoleRouterInput(args);
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return;
      }

      try {
        const selectedRole = await resolveRegisteredRole(input);
        ctx.ui.notify(formatDryRunSummary(buildDryRunSummary(selectedRole, input)), "info");
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
      }
    }
  });

  pi.registerCommand(TRACE_COMMAND_NAME, {
    description: "Explain one role routing decision without launching delegated execution",
    handler: async (args, ctx) => {
      if (ctx.mode !== "tui") {
        ctx.ui.notify(`/${TRACE_COMMAND_NAME} requires interactive mode`, "error");
        return;
      }

      let input: DelegateRoleInput;
      try {
        input = parseRoleCommandInput(args, TRACE_COMMAND_NAME);
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return;
      }

      try {
        const selectedRole = await resolveRegisteredRole(input);
        ctx.ui.notify(formatTraceSummary(await buildTraceSummary(selectedRole, input)), "info");
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
      }
    }
  });

  pi.registerCommand(VALIDATE_COMMAND_NAME, {
    description: "Validate role-router config without delegated execution",
    handler: async (args, ctx) => {
      let input: { path: string };
      try {
        input = parseValidateRoleRouterInput(args);
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
        return;
      }

      try {
        const summary = await validateRoleRouterConfig(input.path);
        ctx.ui.notify(formatValidationSummary(summary), "info");
      } catch (error) {
        ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
      }
    }
  });
}
