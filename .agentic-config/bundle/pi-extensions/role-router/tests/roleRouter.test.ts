import assert from "node:assert/strict";
import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  buildDryRunSummary,
  buildSpawnInput,
  buildTraceSummary,
  decideInputRouting,
  decideInputRoutingWithLaya,
  formatDryRunSummary,
  formatInputRoutingDecision,
  formatTraceSummary,
  formatValidationSummary,
  loadRoleDefinitions,
  maybeEnsureLayaRuntime,
  parseDelegateRoleInput,
  parseDryRunRoleRouterInput,
  parseRoleRouterConfig,
  parseValidateRoleRouterInput,
  resolveRegisteredRole,
  registerRoleAgents,
  validateRoleRouterConfig
} from "../extensions/roleRouter.ts";

test("parseDelegateRoleInput accepts legacy string context plus top-level aliases", () => {
  assert.deepEqual(
    parseDelegateRoleInput(
      '{"role":"planner","task":"plan it","context":"extra","files":["README.md"],"constraints":["keep it small"]}'
    ),
    {
      role: "planner",
      task: "plan it",
      context: "extra",
      files: ["README.md"],
      constraints: ["keep it small"]
    }
  );
});

test("parseDelegateRoleInput accepts canonical structured context", () => {
  assert.deepEqual(
    parseDelegateRoleInput(
      '{"role":"planner","task":"plan it","context":{"summary":"extra","files":["README.md"],"constraints":["keep it small"],"artifacts":["tmp/plan.md"]}}'
    ),
    {
      role: "planner",
      task: "plan it",
      context: {
        summary: "extra",
        files: ["README.md"],
        constraints: ["keep it small"],
        artifacts: ["tmp/plan.md"]
      }
    }
  );
});

test("parseDelegateRoleInput normalizes top-level alias fields into structured context", () => {
  assert.deepEqual(parseDelegateRoleInput('{"role":"planner","task":"plan it","files":["README.md"]}'), {
    role: "planner",
    task: "plan it",
    context: {
      files: ["README.md"]
    }
  });
});

test("parseDelegateRoleInput rejects missing role", () => {
  assert.throws(() => parseDelegateRoleInput('{"task":"plan it"}'), /role-router input error: .*role/);
});

test("decideInputRouting passes through ordinary chat", () => {
  assert.deepEqual(decideInputRouting("hello there", ["planner", "developer", "reviewer"]), {
    action: "pass-through",
    reason: "no-match"
  });
});

test("decideInputRouting launches at most one delegated role for a clear request", () => {
  assert.deepEqual(decideInputRouting("please review this diff for bugs", ["planner", "developer", "reviewer"]), {
    action: "delegate",
    role: "reviewer",
    task: "please review this diff for bugs",
    score: 1
  });
});

test("decideInputRouting passes through lookup-style skill-name requests", () => {
  assert.deepEqual(decideInputRouting("list me the review skill names", ["planner", "developer", "reviewer"]), {
    action: "pass-through",
    reason: "no-match"
  });
});

test("decideInputRouting passes through role-name lookup questions", () => {
  assert.deepEqual(decideInputRouting("what reviewer skills are available", ["planner", "developer", "reviewer"]), {
    action: "pass-through",
    reason: "no-match"
  });
});

test("decideInputRouting carries explicit free-text files constraints conservatively", () => {
  assert.deepEqual(
    decideInputRouting(
      "please check risk in src/app/router.ts and keep the thin-router boundary unchanged; use .scratch/plan.md",
      ["reviewer"]
    ),
    {
      action: "delegate",
      role: "reviewer",
      task: "please check risk in src/app/router.ts and keep the thin-router boundary unchanged; use .scratch/plan.md",
      score: 1,
      context: {
        files: ["src/app/router.ts", ".scratch/plan.md"],
        constraints: ["keep the thin-router boundary unchanged"],
        artifacts: [".scratch/plan.md"]
      }
    }
  );
});

test("decideInputRouting preserves explicit absolute paths when clearly stated", () => {
  assert.deepEqual(
    decideInputRouting("please check risk in /tmp/plan.md and keep the thin-router boundary unchanged", ["reviewer"]),
    {
      action: "delegate",
      role: "reviewer",
      task: "please check risk in /tmp/plan.md and keep the thin-router boundary unchanged",
      score: 1,
      context: {
        files: ["/tmp/plan.md"],
        constraints: ["keep the thin-router boundary unchanged"]
      }
    }
  );
});

test("decideInputRouting avoids fuzzy repository guessing from vague free text", () => {
  assert.deepEqual(
    decideInputRouting("please review the router file and use the latest report", ["planner", "developer", "reviewer"]),
    {
      action: "delegate",
      role: "reviewer",
      task: "please review the router file and use the latest report",
      score: 1
    }
  );
});

test("decideInputRouting leaves natural-language slash phrases in plain task text", () => {
  assert.deepEqual(decideInputRouting("please keep Owner/Member semantics and review this change", ["reviewer"]), {
    action: "delegate",
    role: "reviewer",
    task: "please keep Owner/Member semantics and review this change",
    score: 1
  });
});

test("decideInputRouting requires task phrasing for developer routing", () => {
  assert.deepEqual(decideInputRouting("show developer skill names", ["planner", "developer", "reviewer"]), {
    action: "pass-through",
    reason: "no-match"
  });

  assert.deepEqual(decideInputRouting("please implement this fix", ["planner", "developer", "reviewer"]), {
    action: "delegate",
    role: "developer",
    task: "please implement this fix",
    score: 1
  });
});

test("decideInputRouting ignores disabled roles by only scoring enabled ones", () => {
  assert.deepEqual(decideInputRouting("please review this diff for bugs", ["planner", "developer"]), {
    action: "pass-through",
    reason: "no-match"
  });
});

test("decideInputRouting passes through when multiple roles match the same message", () => {
  assert.deepEqual(decideInputRouting("please review the next steps", ["planner", "reviewer"]), {
    action: "pass-through",
    reason: "ambiguous",
    candidates: ["planner", "reviewer"]
  });
});

test("formatInputRoutingDecision explains automatic decline-to-route outcomes", () => {
  assert.equal(
    formatInputRoutingDecision({
      action: "pass-through",
      reason: "ambiguous",
      candidates: ["planner", "reviewer"]
    }),
    "role-router automatic routing: pass through (ambiguous between planner, reviewer)"
  );

  assert.equal(
    formatInputRoutingDecision({
      action: "pass-through",
      reason: "no-match"
    }),
    "role-router automatic routing: pass through (no high-confidence delegated role matched)"
  );

  assert.equal(
    formatInputRoutingDecision({
      action: "delegate",
      role: "reviewer",
      task: "please review this diff",
      score: 1,
      source: "laya"
    }),
    "role-router automatic routing: route to reviewer (score: 1) via Laya"
  );

  assert.equal(
    formatInputRoutingDecision({
      action: "pass-through",
      reason: "no-match",
      source: "laya-fallback"
    }),
    "role-router automatic routing: pass through (no high-confidence delegated role matched) via heuristic fallback after Laya"
  );
});

test("parseDelegateRoleInput classifies malformed JSON as input error", () => {
  assert.throws(() => parseDelegateRoleInput("{"), /role-router input error: delegate-role input must be valid JSON/);
});

test("parseRoleRouterConfig validates overrides", () => {
  assert.deepEqual(
    parseRoleRouterConfig(
      JSON.stringify({
        roles: {
          planner: {
            agent: "delegate",
            model: "openai/gpt-5",
            prompt: "Custom planner prompt",
            description: "Custom planner",
            thinking: "high",
            async: false,
            context: "fork"
          },
          reviewer: {
            enabled: false
          }
        }
      })
    ),
    {
      roles: {
        planner: {
          agent: "delegate",
          model: "openai/gpt-5",
          prompt: "Custom planner prompt",
          description: "Custom planner",
          thinking: "high",
          async: false,
          context: "fork"
        },
        reviewer: {
          enabled: false
        }
      }
    }
  );
});

test("parseRoleRouterConfig accepts dedicated optional laya startup config", () => {
  assert.deepEqual(
    parseRoleRouterConfig(
      JSON.stringify({
        laya: {
          enabled: true,
          healthUrl: "http://127.0.0.1:8877/health",
          decisionUrl: "http://127.0.0.1:8877/v1/systemone",
          startupCommand: "laya-serve",
          startupTimeoutMs: 250,
          startupModel: "english",
          startupPort: 8877,
          startupHost: "127.0.0.1",
          startupDevice: "cuda",
          startupPreload: true,
          startupThreads: 8
        }
      })
    ),
    {
      laya: {
        enabled: true,
        healthUrl: "http://127.0.0.1:8877/health",
        decisionUrl: "http://127.0.0.1:8877/v1/systemone",
        startupCommand: "laya-serve",
        startupTimeoutMs: 250,
        startupModel: "english",
        startupPort: 8877,
        startupHost: "127.0.0.1",
        startupDevice: "cuda",
        startupPreload: true,
        startupThreads: 8
      }
    }
  );
});

test("parseRoleRouterConfig rejects invalid and removed laya startup config", () => {
  assert.throws(
    () => parseRoleRouterConfig(JSON.stringify({ laya: { startupTimeoutMs: 0 } })),
    /role-router config 'laya.startupTimeoutMs' must be a positive integer when set/
  );
  assert.throws(
    () => parseRoleRouterConfig(JSON.stringify({ laya: { startupArgs: ["--port", "8877"] } })),
    /role-router config 'laya.startupArgs' is no longer supported/
  );
  assert.throws(
    () => parseRoleRouterConfig(JSON.stringify({ laya: { startupModel: "qwen2.5-coder:7b" } })),
    /role-router config 'laya.startupModel' must be one of/
  );
});

test("decideInputRoutingWithLaya returns pass-through for valid pass_through", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-choice-pass-through-"));
  const originalCwd = process.cwd();
  const originalFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        decisionUrl: "http://127.0.0.1:8877/v1/systemone"
      }
    })
  );

  let requestBody: unknown;
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async (_url: string, init?: RequestInit) => {
    requestBody = init?.body;
    return {
      ok: true,
      json: async () => ({ answers: { route: { choice: "pass_through" } } })
    };
  };

  try {
    const decision = await decideInputRoutingWithLaya("hello there", ["planner", "reviewer"]);
    assert.deepEqual(decision, { action: "pass-through", reason: "no-match", source: "laya" });
    assert.deepEqual(JSON.parse(String(requestBody)), {
      state: "hello there",
      questions: {
        route: {
          type: "choice",
          instructions:
            "Choose pass_through when the message should stay in the main session, otherwise choose exactly one enabled configured role.",
          criteria: {
            pass_through: "Keep the message in the main session; do not delegate.",
            planner: "Delegate this message to the configured planner role.",
            reviewer: "Delegate this message to the configured reviewer role."
          }
        }
      }
    });
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalFetch;
  }
});

test("decideInputRoutingWithLaya returns one enabled configured role", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-choice-role-"));
  const originalCwd = process.cwd();
  const originalFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      }
    })
  );

  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: { choice: "reviewer" } } })
  });

  try {
    const decision = await decideInputRoutingWithLaya("please review src/app/router.ts", ["planner", "reviewer"]);
    assert.deepEqual(decision, {
      action: "delegate",
      role: "reviewer",
      task: "please review src/app/router.ts",
      score: 1,
      source: "laya",
      context: {
        files: ["src/app/router.ts"]
      }
    });
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalFetch;
  }
});

test("decideInputRoutingWithLaya falls back on invalid role output", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-choice-invalid-role-"));
  const originalCwd = process.cwd();
  const originalFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      }
    })
  );

  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: { choice: "unknown-role" } } })
  });

  try {
    const decision = await decideInputRoutingWithLaya("please review this diff", ["planner", "reviewer"]);
    assert.equal(decision, undefined);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalFetch;
  }
});

test("decideInputRoutingWithLaya falls back on malformed response shape", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-choice-malformed-"));
  const originalCwd = process.cwd();
  const originalFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      }
    })
  );

  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: {} } })
  });

  try {
    const decision = await decideInputRoutingWithLaya("please review this diff", ["planner", "reviewer"]);
    assert.equal(decision, undefined);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalFetch;
  }
});

test("loadRoleDefinitions falls back to defaults when no config exists", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-defaults-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    const roles = await loadRoleDefinitions();
    assert.ok(roles.planner);
    assert.ok(roles.developer);
    assert.ok(roles.reviewer);
  } finally {
    process.chdir(originalCwd);
  }
});

test("loadRoleDefinitions supports custom roles with full overrides", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-custom-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(
      join(tempDir, ".pi", "role-router.json"),
      JSON.stringify({
        roles: {
          qa: {
            agent: "qa-agent",
            model: "openai/gpt-5",
            prompt: "You are QA.",
            description: "QA reviewer"
          }
        }
      })
    );
    const roles = await loadRoleDefinitions();
    assert.equal(roles.qa?.agent, "qa-agent");
    assert.equal(roles.qa?.model, "openai/gpt-5");
    assert.equal(roles.qa?.prompt, "You are QA.");
  } finally {
    process.chdir(originalCwd);
  }
});

test("parseRoleRouterConfig rejects invalid enabled value with a clear error", () => {
  assert.throws(
    () =>
      parseRoleRouterConfig(
        JSON.stringify({
          roles: {
            planner: {
              enabled: "no"
            }
          }
        })
      ),
    /enabled must be a boolean/
  );
});

test("parseRoleRouterConfig rejects invalid runtime defaults with clear errors", () => {
  assert.throws(
    () =>
      parseRoleRouterConfig(
        JSON.stringify({
          roles: {
            planner: {
              async: "no"
            }
          }
        })
      ),
    /async must be a boolean/
  );

  assert.throws(
    () =>
      parseRoleRouterConfig(
        JSON.stringify({
          roles: {
            planner: {
              context: "sideways"
            }
          }
        })
      ),
    /context must be one of 'fresh', 'fork', or 'profile'/
  );
});

test("parseDryRunRoleRouterInput accepts the same structured contract as delegate-role", () => {
  assert.deepEqual(
    parseDryRunRoleRouterInput(
      '{"role":"planner","task":"plan it","context":{"summary":"extra","files":["README.md"],"constraints":["keep it small"],"artifacts":["tmp/plan.md"]}}'
    ),
    {
      role: "planner",
      task: "plan it",
      context: {
        summary: "extra",
        files: ["README.md"],
        constraints: ["keep it small"],
        artifacts: ["tmp/plan.md"]
      }
    }
  );
});

test("parseDryRunRoleRouterInput classifies malformed JSON as input error", () => {
  assert.throws(
    () => parseDryRunRoleRouterInput("{"),
    /role-router input error: dry-run-role-router input must be valid JSON/
  );
});

test("trace-role-router classifies malformed JSON as input error", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();
  const notices: Array<{ message: string; level: string }> = [];
  roleRouter({
    registerCommand: (name: string, definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }) => {
      commands.set(name, definition);
    }
  } as never);

  const command = commands.get("trace-role-router");
  assert.ok(command);

  await command.handler("{", {
    mode: "tui",
    ui: {
      notify: (message: string, level: string) => {
        notices.push({ message, level });
      }
    }
  });

  assert.deepEqual(notices, [
    {
      message: "role-router input error: trace-role-router input must be valid JSON.",
      level: "error"
    }
  ]);
});

test("parseValidateRoleRouterInput defaults to the project-local path", () => {
  assert.deepEqual(parseValidateRoleRouterInput(""), { path: ".pi/role-router.json" });
});

test("parseValidateRoleRouterInput accepts an explicit path", () => {
  assert.deepEqual(parseValidateRoleRouterInput('{"path":"tmp/role-router.json"}'), {
    path: "tmp/role-router.json"
  });
});

test("validateRoleRouterConfig reports defaults when the config file is missing", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-validate-default-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    const summary = await validateRoleRouterConfig();
    assert.equal(summary.source, "default");
    assert.deepEqual(
      summary.roles.map(({ role, enabled }) => ({ role, enabled })),
      [
        { role: "developer", enabled: true },
        { role: "planner", enabled: true },
        { role: "reviewer", enabled: true }
      ]
    );
  } finally {
    process.chdir(originalCwd);
  }
});

test("validateRoleRouterConfig includes disabled roles in the summary", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-validate-disabled-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(
      join(tempDir, ".pi", "role-router.json"),
      JSON.stringify({
        roles: {
          planner: {
            agent: "delegate",
            prompt: "Custom planner prompt",
            description: "Custom planner",
            async: false,
            context: "profile"
          },
          reviewer: {
            enabled: false
          }
        }
      })
    );
    const summary = await validateRoleRouterConfig();
    assert.equal(summary.source, "config");
    assert.deepEqual(
      summary.roles.map(({ role, enabled, agent }) => ({ role, enabled, agent })),
      [
        { role: "developer", enabled: true, agent: "developer" },
        { role: "planner", enabled: true, agent: "delegate" },
        { role: "reviewer", enabled: false, agent: "reviewer" }
      ]
    );
    const roles = await loadRoleDefinitions();
    assert.deepEqual(
      buildDryRunSummary(
        {
          role: "planner",
          dispatchAgent: roles.planner?.agent ?? "role-router-planner",
          definition: roles.planner
        },
        { role: "planner", task: "Plan the rollout" }
      ).runtime,
      {
        async: false,
        context: "profile",
        timeoutMs: 1800000,
        thinking: "medium"
      }
    );
  } finally {
    process.chdir(originalCwd);
  }
});

test("validateRoleRouterConfig supports an explicit config path", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-validate-explicit-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, "tmp"), { recursive: true });
    const explicitPath = "tmp/role-router.json";
    await writeFile(
      join(tempDir, explicitPath),
      JSON.stringify({
        roles: {
          qa: {
            agent: "qa-agent",
            prompt: "You are QA.",
            description: "QA reviewer"
          }
        }
      })
    );
    const summary = await validateRoleRouterConfig(explicitPath);
    assert.equal(summary.path, explicitPath);
    assert.equal(summary.source, "config");
    assert.deepEqual(
      summary.roles.find((entry) => entry.role === "qa"),
      {
        role: "qa",
        enabled: true,
        agent: "qa-agent",
        description: "QA reviewer"
      }
    );
  } finally {
    process.chdir(originalCwd);
  }
});

test("formatValidationSummary renders a stable human-readable report", () => {
  assert.equal(
    formatValidationSummary({
      path: ".pi/role-router.json",
      source: "config",
      roles: [
        {
          role: "planner",
          enabled: true,
          agent: "delegate",
          description: "Custom planner"
        },
        {
          role: "reviewer",
          enabled: false,
          agent: "reviewer",
          description: "Disabled role"
        }
      ]
    }),
    [
      "role-router config valid: .pi/role-router.json",
      "source: project config",
      "resolved roles:",
      "- planner: enabled (agent: delegate) — Custom planner",
      "- reviewer: disabled (agent: reviewer) — Disabled role"
    ].join("\n")
  );
});

test("validateRoleRouterConfig surfaces malformed JSON as a config error", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-validate-error-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(join(tempDir, ".pi", "role-router.json"), "{");
    await assert.rejects(() => validateRoleRouterConfig(), /role-router role\/config error: invalid JSON/);
  } finally {
    process.chdir(originalCwd);
  }
});

test("registerRoleAgents uses configured agent override as the subagent dispatch target", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-register-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(
      join(tempDir, ".pi", "role-router.json"),
      JSON.stringify({
        roles: {
          planner: {
            agent: "delegate",
            prompt: "Custom planner prompt",
            description: "Custom planner",
            async: false,
            context: "profile"
          }
        }
      })
    );

    const roles = await registerRoleAgents({} as never);
    const planner = roles.find((entry) => entry.role === "planner");
    assert.equal(planner?.dispatchAgent, "delegate");
  } finally {
    process.chdir(originalCwd);
  }
});

test("loadRoleDefinitions surfaces malformed config as role/config error", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-load-error-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(join(tempDir, ".pi", "role-router.json"), "{");
    await assert.rejects(() => loadRoleDefinitions(), /role-router role\/config error: invalid JSON/);
  } finally {
    process.chdir(originalCwd);
  }
});

test("resolveRegisteredRole reports unknown roles as role/config errors", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-resolve-missing-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await assert.rejects(
      () => resolveRegisteredRole({ role: "missing", task: "Plan the rollout" }),
      /role-router role\/config error: unknown role 'missing'\./
    );
  } finally {
    process.chdir(originalCwd);
  }
});

test("resolveRegisteredRole reports disabled roles as role/config errors", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-resolve-disabled-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(
      join(tempDir, ".pi", "role-router.json"),
      JSON.stringify({
        roles: {
          reviewer: {
            enabled: false
          }
        }
      })
    );
    await assert.rejects(
      () => resolveRegisteredRole({ role: "reviewer", task: "Review the rollout" }),
      /role-router role\/config error: role 'reviewer' exists but is disabled\./
    );
  } finally {
    process.chdir(originalCwd);
  }
});

test("buildDryRunSummary reuses the live spawn payload seam", () => {
  assert.deepEqual(
    buildDryRunSummary(
      {
        role: "planner",
        dispatchAgent: "delegate",
        definition: {
          description: "Custom planner",
          prompt: "Custom planner prompt",
          model: "openai/gpt-5",
          thinking: "high"
        }
      },
      {
        role: "planner",
        task: "Plan the rollout",
        context: {
          summary: "Use the existing checklist",
          files: ["pi-extensions/role-router/extensions/roleRouter.ts"],
          constraints: ["Keep the thin-router boundary unchanged"],
          artifacts: ["tmp/plan.md"]
        }
      }
    ),
    {
      requestedRole: "planner",
      dispatchAgent: "delegate",
      runtime: {
        async: true,
        context: "fresh",
        timeoutMs: 1800000,
        model: "openai/gpt-5",
        thinking: "high"
      },
      taskPreview:
        "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nSummary:\nUse the existing checklist\n\nFiles:\n- pi-extensions/role-router/extensions/roleRouter.ts\n\nConstraints:\n- Keep the thin-router boundary unchanged\n\nArtifacts:\n- tmp/plan.md"
    }
  );
});

test("buildDryRunSummary adds role context to the handoff when command context is omitted", () => {
  assert.deepEqual(
    buildDryRunSummary(
      {
        role: "planner",
        dispatchAgent: "delegate",
        definition: {
          description: "Custom planner",
          prompt: "Custom planner prompt",
          thinking: "high",
          async: false,
          context: "fork"
        }
      },
      {
        role: "planner",
        task: "Plan the rollout"
      }
    ),
    {
      requestedRole: "planner",
      dispatchAgent: "delegate",
      runtime: {
        async: false,
        context: "fork",
        timeoutMs: 1800000,
        thinking: "high"
      },
      taskPreview: "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nRole context:\nfork"
    }
  );
});

test("formatDryRunSummary renders a stable human-readable report", () => {
  assert.equal(
    formatDryRunSummary({
      requestedRole: "planner",
      dispatchAgent: "delegate",
      runtime: {
        async: false,
        context: "fork",
        timeoutMs: 1800000,
        model: "openai/gpt-5",
        thinking: "high"
      },
      taskPreview: "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nRole context:\nfork"
    }),
    [
      "role-router dry run: planner",
      "agent: delegate",
      "model: openai/gpt-5",
      "thinking: high",
      "async: false",
      "context: fork",
      "timeoutMs: 1800000",
      "task preview:",
      "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nRole context:\nfork"
    ].join("\n")
  );
});

test("buildTraceSummary reports built-in defaults when no config override exists", async () => {
  const summary = await buildTraceSummary(
    {
      role: "developer",
      dispatchAgent: "developer",
      definition: {
        description: "Developer",
        prompt: "Developer prompt",
        thinking: "low"
      }
    },
    {
      role: "developer",
      task: "Implement the rollout"
    },
    "tmp/missing-role-router.json"
  );

  assert.deepEqual(summary, {
    requestedRole: "developer",
    configSource: "default",
    dispatchAgent: "developer",
    fieldSources: {
      agent: "default",
      model: "default",
      thinking: "default",
      async: "default",
      context: "default"
    },
    runtime: {
      async: true,
      context: "fresh",
      timeoutMs: 1800000,
      thinking: "low"
    },
    taskPreview: "Developer prompt\nRole: developer\nTask:\nImplement the rollout"
  });
});

test("parseDelegateRoleInput rejects malformed structured handoff fields", () => {
  assert.throws(
    () => parseDelegateRoleInput('{"role":"planner","task":"plan it","files":"README.md"}'),
    /role-router input error: delegate-role 'files' must be an array of non-empty strings when set/
  );
  assert.throws(
    () => parseDelegateRoleInput('{"role":"planner","task":"plan it","constraints":[""]}'),
    /role-router input error: delegate-role 'constraints' must be an array of non-empty strings when set/
  );
  assert.throws(
    () => parseDelegateRoleInput('{"role":"planner","task":"plan it","context":{"summary":""}}'),
    /role-router input error: delegate-role context 'summary' must be a non-empty string when set/
  );
  assert.throws(
    () => parseDelegateRoleInput('{"role":"planner","task":"plan it","context":{}}'),
    /role-router input error: delegate-role context object must include at least one non-empty supported field/
  );
  assert.throws(
    () => parseDelegateRoleInput('{"role":"planner","task":"plan it","context":{"unknown":"value"}}'),
    /role-router input error: delegate-role context contains unknown key 'unknown'/
  );
  assert.throws(
    () =>
      parseDelegateRoleInput(
        '{"role":"planner","task":"plan it","context":{"files":["README.md"]},"files":["other.md"]}'
      ),
    /role-router input error: delegate-role cannot set both top-level 'files' and context 'files'/
  );
  assert.throws(
    () => parseDelegateRoleInput('{"role":"planner","task":"plan it","context":{"artifacts":[""]}}'),
    /role-router input error: delegate-role 'artifacts' must be an array of non-empty strings when set/
  );
});

test("buildDryRunSummary omits structured sections when fields are absent", () => {
  const summary = buildDryRunSummary(
    {
      role: "planner",
      dispatchAgent: "delegate",
      definition: {
        description: "Custom planner",
        prompt: "Custom planner prompt",
        thinking: "high"
      }
    },
    {
      role: "planner",
      task: "Plan the rollout"
    }
  );

  assert.doesNotMatch(summary.taskPreview, /Files:/);
  assert.doesNotMatch(summary.taskPreview, /Constraints:/);
});

test("buildTraceSummary reports config overrides for delegated runtime defaults", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-trace-config-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(
      join(tempDir, ".pi", "role-router.json"),
      JSON.stringify({
        roles: {
          planner: {
            agent: "delegate",
            model: "openai/gpt-5",
            prompt: "Custom planner prompt",
            description: "Custom planner",
            thinking: "high",
            async: false,
            context: "fork"
          }
        }
      })
    );

    const summary = await buildTraceSummary(
      {
        role: "planner",
        dispatchAgent: "delegate",
        definition: {
          description: "Custom planner",
          prompt: "Custom planner prompt",
          model: "openai/gpt-5",
          thinking: "high",
          async: false,
          context: "fork"
        }
      },
      {
        role: "planner",
        task: "Plan the rollout",
        context: {
          summary: "Use the existing checklist",
          files: ["pi-extensions/role-router/extensions/roleRouter.ts"],
          constraints: ["Keep the thin-router boundary unchanged"],
          artifacts: ["tmp/plan.md"]
        }
      }
    );

    assert.deepEqual(summary, {
      requestedRole: "planner",
      configSource: "config",
      dispatchAgent: "delegate",
      fieldSources: {
        agent: "config",
        model: "config",
        thinking: "config",
        async: "config",
        context: "config"
      },
      runtime: {
        async: false,
        context: "fork",
        timeoutMs: 1800000,
        model: "openai/gpt-5",
        thinking: "high"
      },
      taskPreview:
        "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nSummary:\nUse the existing checklist\n\nFiles:\n- pi-extensions/role-router/extensions/roleRouter.ts\n\nConstraints:\n- Keep the thin-router boundary unchanged\n\nArtifacts:\n- tmp/plan.md"
    });
  } finally {
    process.chdir(originalCwd);
  }
});

test("formatTraceSummary renders a stable human-readable report", () => {
  assert.equal(
    formatTraceSummary({
      requestedRole: "planner",
      configSource: "config",
      dispatchAgent: "delegate",
      fieldSources: {
        agent: "config",
        model: "config",
        thinking: "config",
        async: "config",
        context: "config"
      },
      runtime: {
        async: false,
        context: "fork",
        timeoutMs: 1800000,
        model: "openai/gpt-5",
        thinking: "high"
      },
      taskPreview:
        "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nSummary:\nUse the existing checklist\n\nFiles:\n- pi-extensions/role-router/extensions/roleRouter.ts\n\nConstraints:\n- Keep the thin-router boundary unchanged\n\nArtifacts:\n- tmp/plan.md"
    }),
    [
      "role-router trace: planner",
      "config source: project config",
      "agent: delegate (config)",
      "model: openai/gpt-5 (config)",
      "thinking: high (config)",
      "async: false (config)",
      "context: fork (config)",
      "timeoutMs: 1800000",
      "task preview:",
      "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nSummary:\nUse the existing checklist\n\nFiles:\n- pi-extensions/role-router/extensions/roleRouter.ts\n\nConstraints:\n- Keep the thin-router boundary unchanged\n\nArtifacts:\n- tmp/plan.md"
    ].join("\n")
  );
});

test("buildTraceSummary keeps thinking provenance default when only prompt changed", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-trace-prompt-only-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    await mkdir(join(tempDir, ".pi"), { recursive: true });
    await writeFile(
      join(tempDir, ".pi", "role-router.json"),
      JSON.stringify({
        roles: {
          planner: {
            prompt: "Custom planner prompt",
            description: "Planner with custom prompt"
          }
        }
      })
    );

    const summary = await buildTraceSummary(
      {
        role: "planner",
        dispatchAgent: "planner",
        definition: {
          description: "Planner with custom prompt",
          prompt: "Custom planner prompt",
          thinking: "medium"
        }
      },
      {
        role: "planner",
        task: "Plan the rollout"
      },
      "tmp/missing-role-router.json"
    );

    assert.equal(summary.configSource, "default");
    assert.deepEqual(summary.fieldSources, {
      agent: "default",
      model: "default",
      thinking: "default",
      async: "default",
      context: "default"
    });
  } finally {
    process.chdir(originalCwd);
  }
});

test("trace-role-router handler stays dependency-free", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-trace-handler-"));
  const originalCwd = process.cwd();
  process.chdir(tempDir);
  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();
    const notices: Array<{ message: string; level: string }> = [];

    roleRouter({
      registerCommand: (
        name: string,
        definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }
      ) => {
        commands.set(name, definition);
      }
    } as never);

    const command = commands.get("trace-role-router");
    assert.ok(command);

    await command.handler('{"role":"planner","task":"Plan the rollout"}', {
      mode: "tui",
      ui: {
        notify: (message: string, level: string) => {
          notices.push({ message, level });
        }
      }
    });

    assert.equal(notices.length, 1);
    assert.equal(notices[0]?.level, "info");
    assert.match(notices[0]?.message ?? "", /^role-router trace: planner/);
  } finally {
    process.chdir(originalCwd);
  }
});

test("buildSpawnInput carries the delegated pi-subagents payload", () => {
  assert.deepEqual(
    buildSpawnInput(
      {
        role: "planner",
        dispatchAgent: "delegate",
        definition: {
          description: "Custom planner",
          prompt: "Custom planner prompt",
          model: "openai/gpt-5",
          thinking: "high"
        }
      },
      {
        role: "planner",
        task: "Plan the rollout",
        context: {
          summary: "Use the existing checklist",
          files: ["pi-extensions/role-router/extensions/roleRouter.ts"],
          constraints: ["Keep the thin-router boundary unchanged"],
          artifacts: ["tmp/plan.md"]
        }
      }
    ),
    {
      role: "planner",
      agent: "delegate",
      task: "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nSummary:\nUse the existing checklist\n\nFiles:\n- pi-extensions/role-router/extensions/roleRouter.ts\n\nConstraints:\n- Keep the thin-router boundary unchanged\n\nArtifacts:\n- tmp/plan.md",
      async: true,
      context: "fresh",
      model: "openai/gpt-5",
      thinking: "high",
      timeoutMs: 1800000
    }
  );
});

test("buildSpawnInput applies per-role async and context defaults while preserving command context precedence", () => {
  assert.deepEqual(
    buildSpawnInput(
      {
        role: "planner",
        dispatchAgent: "delegate",
        definition: {
          description: "Custom planner",
          prompt: "Custom planner prompt",
          model: "openai/gpt-5",
          thinking: "high",
          async: false,
          context: "fork"
        }
      },
      {
        role: "planner",
        task: "Plan the rollout",
        context: "Use the existing checklist",
        files: ["README.md"],
        constraints: ["keep it small"]
      }
    ),
    {
      role: "planner",
      agent: "delegate",
      task: "Custom planner prompt\nRole: planner\nTask:\nPlan the rollout\n\nFiles:\n- README.md\n\nConstraints:\n- keep it small\n\nExtra context:\nUse the existing checklist",
      async: false,
      context: "fork",
      model: "openai/gpt-5",
      thinking: "high",
      timeoutMs: 1800000
    }
  );
});

test("delegated launch failures mention the selected role", async () => {
  const spawnInput = buildSpawnInput(
    {
      role: "planner",
      dispatchAgent: "delegate",
      definition: {
        description: "Planner",
        prompt: "Planner prompt",
        thinking: "medium"
      }
    },
    {
      role: "planner",
      task: "Plan the rollout"
    }
  );

  const directSpawn = async (_pi: unknown, input: typeof spawnInput) => {
    try {
      throw new Error("child launch exploded");
    } catch (error) {
      throw new Error(
        `role-router delegated launch failed for role '${input.role}': ${error instanceof Error ? error.message : String(error)}`
      );
    }
  };

  await assert.rejects(
    () => directSpawn({}, spawnInput),
    /role-router delegated launch failed for role 'planner': child launch exploded/
  );
});

test("delegate-role fallback subagents RPC emits the spawn envelope and waits for the reply", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();
  const notices: Array<{ message: string; level: string }> = [];
  const emitted: Array<{ event: string; payload: unknown }> = [];
  const listeners = new Map<string, (payload: unknown) => void>();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalImport = (globalThis as Record<string, unknown>).__testPiPackageImport;
  const originalDateNow = Date.now;
  const originalMathRandom = Math.random;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testPiPackageImport = async (_pkg: string, filePath: string) => {
    if (filePath === "src/api/agents.js") {
      throw new Error("test direct spawn bridge unavailable");
    }
    if (filePath === "src/extension/rpc.js") {
      return { SUBAGENT_RPC_REQUEST_EVENT: "subagents:rpc:v1:request" };
    }
    throw new Error(`unexpected import path: ${filePath}`);
  };

  Date.now = () => 1_700_000_000_000;
  Math.random = () => 0.123456789;

  try {
    roleRouter({
      registerCommand: (
        name: string,
        definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }
      ) => {
        commands.set(name, definition);
      },
      events: {
        on: (event: string, handler: (payload: unknown) => void) => {
          listeners.set(event, handler);
          return () => {
            listeners.delete(event);
          };
        },
        emit: async (event: string, payload: unknown) => {
          emitted.push({ event, payload });
          if (event === "subagents:rpc:v1:request") {
            const request = payload as { requestId: string };
            listeners.get(`subagents:rpc:v1:reply:${request.requestId}`)?.({
              success: true,
              data: { runId: "run_123" }
            });
          }
        }
      }
    } as never);

    const command = commands.get("delegate-role");
    assert.ok(command);

    await command.handler('{"role":"planner","task":"Plan the rollout"}', {
      mode: "tui",
      ui: {
        notify: (message: string, level: string) => {
          notices.push({ message, level });
        }
      }
    });

    assert.deepEqual(emitted, [
      {
        event: "subagents:rpc:v1:request",
        payload: {
          version: 1,
          requestId: "role-router-loyw3v28-4fzzzx",
          method: "spawn",
          params: {
            role: "planner",
            agent: "planner",
            task: "You are the planner role. Clarify the task, produce a compact plan, call out trade-offs, and end with the next concrete step.\nRole: planner\nTask:\nPlan the rollout",
            async: true,
            context: "fresh",
            timeoutMs: 1800000,
            thinking: "medium"
          }
        }
      }
    ]);
    assert.deepEqual(notices, [
      {
        message: "role-router launch receipt: planner\nagent: planner\nhandle: run_123",
        level: "info"
      }
    ]);
  } finally {
    Date.now = originalDateNow;
    Math.random = originalMathRandom;
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testPiPackageImport = originalImport;
  }
});

test("delegate-role fallback subagents RPC times out without a reply", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();
  const notices: Array<{ message: string; level: string }> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalImport = (globalThis as Record<string, unknown>).__testPiPackageImport;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testPiPackageImport = async (_pkg: string, filePath: string) => {
    if (filePath === "src/api/agents.js") {
      throw new Error("test direct spawn bridge unavailable");
    }
    if (filePath === "src/extension/rpc.js") {
      return { SUBAGENT_RPC_REQUEST_EVENT: "subagents:rpc:v1:request" };
    }
    throw new Error(`unexpected import path: ${filePath}`);
  };

  try {
    roleRouter({
      registerCommand: (
        name: string,
        definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }
      ) => {
        commands.set(name, definition);
      },
      events: {
        on: () => () => undefined,
        emit: async () => undefined
      }
    } as never);

    const command = commands.get("delegate-role");
    assert.ok(command);

    await command.handler('{"role":"planner","task":"Plan the rollout"}', {
      mode: "tui",
      ui: {
        notify: (message: string, level: string) => {
          notices.push({ message, level });
        }
      }
    });

    assert.deepEqual(notices, [
      {
        message:
          "role-router delegated launch failed for role 'planner': timed out waiting 5000ms for subagents RPC spawn acknowledgement",
        level: "error"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testPiPackageImport = originalImport;
  }
});

test("delegate-role reports a launch receipt from the delegated seam", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();
  const notices: Array<{ message: string; level: string }> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });

  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({
    runId: "child-run-123",
    outputReference: "artifact://role-router/launch"
  });

  try {
    roleRouter({
      registerCommand: (
        name: string,
        definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }
      ) => {
        commands.set(name, definition);
      },
      events: {
        emit: async () => undefined
      }
    } as never);

    const command = commands.get("delegate-role");
    assert.ok(command);

    await command.handler('{"role":"planner","task":"Plan the rollout"}', {
      mode: "tui",
      ui: {
        notify: (message: string, level: string) => {
          notices.push({ message, level });
        }
      }
    });

    assert.deepEqual(notices, [
      {
        message:
          "role-router launch receipt: planner\nagent: planner\nhandle: child-run-123\noutput: artifact://role-router/launch",
        level: "info"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("delegate-role keeps output references separate from the delegated handle", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();
  const notices: Array<{ message: string; level: string }> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });

  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({
    outputReference: "artifact://role-router/output-only"
  });

  try {
    roleRouter({
      registerCommand: (
        name: string,
        definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }
      ) => {
        commands.set(name, definition);
      },
      events: {
        emit: async () => undefined
      }
    } as never);

    const command = commands.get("delegate-role");
    assert.ok(command);

    await command.handler('{"role":"planner","task":"Plan the rollout"}', {
      mode: "tui",
      ui: {
        notify: (message: string, level: string) => {
          notices.push({ message, level });
        }
      }
    });

    assert.deepEqual(notices, [
      {
        message: "role-router launch receipt: planner\nagent: planner\noutput: artifact://role-router/output-only",
        level: "info"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("automatic routing explains low-confidence pass-through without launching", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const inputHandlers: Array<
    (event: { text: string; source: string; images?: unknown[] }, ctx: unknown) => Promise<unknown>
  > = [];
  const notices: Array<{ message: string; level: string }> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });

  try {
    roleRouter({
      on: (event: string, handler: (event: unknown, ctx: unknown) => Promise<unknown>) => {
        if (event === "input") {
          inputHandlers.push(handler as never);
        }
        return () => undefined;
      },
      registerCommand: () => undefined
    } as never);

    assert.equal(inputHandlers.length, 1);
    const result = await inputHandlers[0]?.(
      { text: "hello there", source: "interactive" },
      {
        mode: "tui",
        ui: {
          notify: (message: string, level: string) => {
            notices.push({ message, level });
          }
        }
      }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.deepEqual(notices, [
      {
        message:
          "role-router automatic routing: pass through (no high-confidence delegated role matched) via heuristic fallback after Laya",
        level: "info"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
  }
});

test("automatic routing stays pass-through outside tui mode", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const inputHandlers: Array<
    (event: { text: string; source: string; images?: unknown[] }, ctx: unknown) => Promise<unknown>
  > = [];
  const spawnCalls: Array<unknown> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input);
    return { runId: "unexpected-run" };
  };

  try {
    roleRouter({
      on: (event: string, handler: (event: unknown, ctx: unknown) => Promise<unknown>) => {
        if (event === "input") {
          inputHandlers.push(handler as never);
        }
        return () => undefined;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.equal(inputHandlers.length, 1);
    const result = await inputHandlers[0]?.(
      { text: "please review this diff for bugs", source: "interactive" },
      {
        mode: "rpc",
        ui: { notify: () => undefined }
      }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.deepEqual(spawnCalls, []);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("automatic routing passes through ambiguous multi-role requests without launching", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const inputHandlers: Array<
    (event: { text: string; source: string; images?: unknown[] }, ctx: unknown) => Promise<unknown>
  > = [];
  const notices: Array<{ message: string; level: string }> = [];
  const spawnCalls: Array<unknown> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input);
    return { runId: "unexpected-run" };
  };

  try {
    roleRouter({
      on: (event: string, handler: (event: unknown, ctx: unknown) => Promise<unknown>) => {
        if (event === "input") {
          inputHandlers.push(handler as never);
        }
        return () => undefined;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.equal(inputHandlers.length, 1);
    const result = await inputHandlers[0]?.(
      { text: "please review the next steps", source: "interactive" },
      {
        mode: "tui",
        ui: {
          notify: (message: string, level: string) => {
            notices.push({ message, level });
          }
        }
      }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.deepEqual(spawnCalls, []);
    assert.deepEqual(notices, [
      {
        message:
          "role-router automatic routing: pass through (ambiguous between planner, reviewer) via heuristic fallback after Laya",
        level: "info"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("automatic routing passes through lookup-style requests without launching", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const inputHandlers: Array<
    (event: { text: string; source: string; images?: unknown[] }, ctx: unknown) => Promise<unknown>
  > = [];
  const notices: Array<{ message: string; level: string }> = [];
  const spawnCalls: Array<unknown> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input);
    return { runId: "unexpected-run" };
  };

  try {
    roleRouter({
      on: (event: string, handler: (event: unknown, ctx: unknown) => Promise<unknown>) => {
        if (event === "input") {
          inputHandlers.push(handler as never);
        }
        return () => undefined;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.equal(inputHandlers.length, 1);
    const result = await inputHandlers[0]?.(
      { text: "list me the review skill names", source: "interactive" },
      {
        mode: "tui",
        ui: {
          notify: (message: string, level: string) => {
            notices.push({ message, level });
          }
        }
      }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.deepEqual(spawnCalls, []);
    assert.deepEqual(notices, [
      {
        message:
          "role-router automatic routing: pass through (no high-confidence delegated role matched) via heuristic fallback after Laya",
        level: "info"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("automatic routing launches exactly one delegated run for a clear request", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const inputHandlers: Array<
    (event: { text: string; source: string; images?: unknown[] }, ctx: unknown) => Promise<unknown>
  > = [];
  const notices: Array<{ message: string; level: string }> = [];
  const spawnCalls: Array<unknown> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input);
    return { runId: "auto-run-123" };
  };

  try {
    roleRouter({
      on: (event: string, handler: (event: unknown, ctx: unknown) => Promise<unknown>) => {
        if (event === "input") {
          inputHandlers.push(handler as never);
        }
        return () => undefined;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.equal(inputHandlers.length, 1);
    const result = await inputHandlers[0]?.(
      { text: "please review this diff for bugs", source: "interactive" },
      {
        mode: "tui",
        ui: {
          notify: (message: string, level: string) => {
            notices.push({ message, level });
          }
        }
      }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.equal(spawnCalls.length, 1);
    assert.deepEqual(spawnCalls[0], {
      role: "reviewer",
      agent: "reviewer",
      task: "You are the reviewer role. Review the work critically, focus on correctness and risk, and report findings before suggestions.\nRole: reviewer\nTask:\nplease review this diff for bugs",
      async: true,
      context: "fresh",
      thinking: "low",
      timeoutMs: 1800000
    });
    assert.deepEqual(notices, [
      {
        message: "role-router automatic routing: route to reviewer (score: 1) via heuristic fallback after Laya",
        level: "info"
      },
      {
        message: "role-router launch receipt: reviewer\nagent: reviewer\nhandle: auto-run-123",
        level: "info"
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("automatic routing carries explicit free-text context into the delegated handoff", async () => {
  const { default: roleRouter } = await import("../extensions/roleRouter.ts");
  const inputHandlers: Array<
    (event: { text: string; source: string; images?: unknown[] }, ctx: unknown) => Promise<unknown>
  > = [];
  const spawnCalls: Array<unknown> = [];
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input);
    return { runId: "auto-run-ctx" };
  };

  try {
    roleRouter({
      on: (event: string, handler: (event: unknown, ctx: unknown) => Promise<unknown>) => {
        if (event === "input") {
          inputHandlers.push(handler as never);
        }
        return () => undefined;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.equal(inputHandlers.length, 1);
    const result = await inputHandlers[0]?.(
      {
        text: "please review src/app/router.ts and look for bugs; keep the thin-router boundary unchanged; use tmp/plan.md",
        source: "interactive"
      },
      {
        mode: "tui",
        ui: { notify: () => undefined }
      }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.deepEqual(spawnCalls, [
      {
        role: "reviewer",
        agent: "reviewer",
        task: "You are the reviewer role. Review the work critically, focus on correctness and risk, and report findings before suggestions.\nRole: reviewer\nTask:\nplease review src/app/router.ts and look for bugs; keep the thin-router boundary unchanged; use tmp/plan.md\n\nFiles:\n- src/app/router.ts\n- tmp/plan.md\n\nConstraints:\n- keep the thin-router boundary unchanged\n\nArtifacts:\n- tmp/plan.md",
        async: true,
        context: "fresh",
        thinking: "low",
        timeoutMs: 1800000
      }
    ]);
  } finally {
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("automatic routing falls back unchanged when laya startup fails", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-fallback-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const originalLayaDecisionFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  const spawnCalls: Array<Record<string, unknown>> = [];
  const notices: Array<{ message: string; level: string }> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: { enabled: true, startupTimeoutMs: 50 },
      roles: {
        reviewer: {
          prompt:
            "You are the reviewer role. Review the work critically, focus on correctness and risk, and report findings before suggestions.",
          description: "Reviewer"
        }
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => false;
  (globalThis as Record<string, unknown>).__testLayaStartup = async () => {
    throw new Error("missing laya");
  };
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input as Record<string, unknown>);
    return { id: "fallback-run-123" };
  };
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => {
    throw new Error("should not call laya decision api when startup fails");
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: (message: string, level: string) => notices.push({ message, level }) } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.equal(spawnCalls.length, 1);
    assert.equal((spawnCalls[0]?.agent as string) ?? "", "reviewer");
    assert.deepEqual(notices, [
      {
        message: "role-router automatic routing: route to reviewer (score: 1) via heuristic fallback after Laya",
        level: "info"
      },
      { message: "role-router launch receipt: reviewer\nagent: reviewer\nhandle: fallback-run-123", level: "info" }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalLayaDecisionFetch;
  }
});

test("automatic routing skips laya startup when already healthy", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-healthy-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const startupCalls: Array<{ command: string; args: string[] }> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        healthUrl: "http://127.0.0.1:8877/health"
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => true;
  (globalThis as Record<string, unknown>).__testLayaStartup = async (command: string, args: string[]) => {
    startupCalls.push({ command, args });
  };
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({ id: "healthy-no-start-run-123" });

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: () => undefined } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.deepEqual(startupCalls, []);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
  }
});

test("automatic routing falls back to heuristic routing when laya preflight startup stays unhealthy", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-auto-pass-through-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const originalLayaDecisionFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  const notices: Array<{ message: string; level: string }> = [];
  let spawnCount = 0;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => false;
  (globalThis as Record<string, unknown>).__testLayaStartup = async () => ({ running: false });
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: { choice: "pass_through" } } })
  });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => {
    spawnCount += 1;
    return { id: "should-not-run" };
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: (message: string, level: string) => notices.push({ message, level }) } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.equal(spawnCount, 1);
    assert.deepEqual(notices, [
      {
        message: "role-router automatic routing: route to reviewer (score: 1) via heuristic fallback after Laya",
        level: "info"
      },
      {
        message: "role-router launch receipt: reviewer\nagent: reviewer\nhandle: should-not-run",
        level: "info"
      }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalLayaDecisionFetch;
  }
});

test("automatic routing falls back to heuristic routing when laya decision fetch is unreachable", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-auto-role-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const originalLayaDecisionFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  const notices: Array<{ message: string; level: string }> = [];
  const spawnCalls: Array<unknown> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      },
      roles: {
        planner: {
          enabled: false
        },
        reviewer: {
          prompt:
            "You are the reviewer role. Review the work critically, focus on correctness and risk, and report findings before suggestions.",
          description: "Reviewer"
        }
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({
    dispose() {},
    name,
    definition
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => true;
  (globalThis as Record<string, unknown>).__testLayaStartup = async () => {
    throw new Error("should not start");
  };
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => {
    throw new Error("network down");
  };
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input);
    return { id: "laya-role-run-123" };
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review src/app/router.ts", source: "user" },
      { mode: "tui", ui: { notify: (message: string, level: string) => notices.push({ message, level }) } }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.equal(spawnCalls.length, 0);
    assert.deepEqual(notices, [
      {
        message:
          "role-router automatic routing: pass through (no high-confidence delegated role matched) via heuristic fallback after Laya",
        level: "info"
      }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalLayaDecisionFetch;
  }
});

test("automatic routing falls back to heuristic routing when healthy laya returns an unknown role", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-auto-invalid-role-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const originalLayaDecisionFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  const notices: Array<{ message: string; level: string }> = [];
  const spawnCalls: Array<Record<string, unknown>> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => true;
  (globalThis as Record<string, unknown>).__testLayaStartup = async () => {
    throw new Error("should not start");
  };
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: { choice: "unknown-role" } } })
  });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input as Record<string, unknown>);
    return { id: "laya-invalid-fallback-run-123" };
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: (message: string, level: string) => notices.push({ message, level }) } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.equal(spawnCalls.length, 1);
    assert.equal((spawnCalls[0]?.agent as string) ?? "", "reviewer");
    assert.deepEqual(notices, [
      {
        message: "role-router automatic routing: route to reviewer (score: 1) via heuristic fallback after Laya",
        level: "info"
      },
      {
        message: "role-router launch receipt: reviewer\nagent: reviewer\nhandle: laya-invalid-fallback-run-123",
        level: "info"
      }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalLayaDecisionFetch;
  }
});

test("role-router defers laya startup until routing input with the dedicated env-backed contract", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-extension-start-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const startupCalls: Array<{
    command: string;
    args: string[];
    env: Partial<Record<string, string | null>>;
  }> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        startupCommand: "laya-serve",
        startupPort: 8877,
        startupTimeoutMs: 50,
        startupModel: "english",
        startupHost: "127.0.0.1",
        startupDevice: "cuda",
        startupPreload: true,
        startupThreads: 8
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => false;
  (globalThis as Record<string, unknown>).__testLayaStartup = async (
    command: string,
    args: string[],
    env?: NodeJS.ProcessEnv
  ) => {
    startupCalls.push({
      command,
      args,
      env: {
        LAYA_DEFAULT_MODEL: env?.LAYA_DEFAULT_MODEL ?? null,
        LAYA_PORT: env?.LAYA_PORT ?? null,
        LAYA_HOST: env?.LAYA_HOST ?? null,
        LAYA_DEVICE: env?.LAYA_DEVICE ?? null,
        LAYA_PRELOAD: env?.LAYA_PRELOAD ?? null,
        LAYA_THREADS: env?.LAYA_THREADS ?? null
      }
    });
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    roleRouter({
      on: () => undefined,
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    await new Promise((resolve) => setTimeout(resolve, 0));

    assert.deepEqual(startupCalls, []);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
  }
});

test("automatic routing reuses healthy laya without spawning when health goes green during probes", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-start-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const healthChecks: string[] = [];
  const startupCalls: Array<{ command: string; args: string[] }> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        healthUrl: "http://127.0.0.1:8877/health",
        startupCommand: "laya-serve",
        startupPort: 8877,
        startupTimeoutMs: 250
      }
    })
  );

  let probeCount = 0;
  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async (healthUrl: string) => {
    healthChecks.push(healthUrl);
    probeCount += 1;
    return probeCount >= 3;
  };
  (globalThis as Record<string, unknown>).__testLayaStartup = async (command: string, args: string[]) => {
    startupCalls.push({ command, args });
  };
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({ id: "healthy-run-123" });

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: () => undefined } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.deepEqual(startupCalls, []);
    assert.deepEqual(healthChecks, []);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
  }
});

test("automatic routing caps laya readiness wait to a tiny probe window", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-timeout-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const startupCalls: Array<{ command: string; args: string[]; port: string | null }> = [];
  const spawnCalls: Array<Record<string, unknown>> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        startupTimeoutMs: 50
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => false;
  (globalThis as Record<string, unknown>).__testLayaStartup = async (
    command: string,
    args: string[],
    env?: NodeJS.ProcessEnv
  ) => {
    startupCalls.push({ command, args, port: env?.LAYA_PORT ?? null });
  };
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async (_pi: unknown, input: unknown) => {
    spawnCalls.push(input as Record<string, unknown>);
    return { id: "timeout-run-123" };
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: () => undefined } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.deepEqual(startupCalls, []);
    assert.equal(spawnCalls.length, 1);
    assert.equal((spawnCalls[0]?.agent as string) ?? "", "reviewer");
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
  }
});

test("automatic routing rechecks laya health on every input and restarts it when it went down", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-restart-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const originalLayaDecisionFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  const startupCalls: Array<{ command: string; args: string[]; port: string | null }> = [];
  let healthCheckCount = 0;
  let spawnCount = 0;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        startupCommand: "laya-serve",
        startupTimeoutMs: 50
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => {
    healthCheckCount += 1;
    return healthCheckCount === 2 || healthCheckCount >= 4;
  };
  (globalThis as Record<string, unknown>).__testLayaStartup = async (
    command: string,
    args: string[],
    env?: NodeJS.ProcessEnv
  ) => {
    startupCalls.push({ command, args, port: env?.LAYA_PORT ?? null });
  };
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: { choice: "reviewer" } } })
  });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => {
    spawnCount += 1;
    return { id: `restart-run-${spawnCount}` };
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);

    const first = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: () => undefined } }
    );
    const second = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: () => undefined } }
    );

    assert.deepEqual(first, { action: "handled" });
    assert.deepEqual(second, { action: "handled" });
    assert.deepEqual(startupCalls, []);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalLayaDecisionFetch;
  }
});

test("automatic routing does not respawn laya while an earlier start is still coming up", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-no-respawn-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const originalLayaDecisionFetch = (globalThis as Record<string, unknown>).__testLayaDecisionFetch;
  const startupCalls: Array<{ command: string; args: string[]; port: string | null }> = [];
  let healthCheckCount = 0;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true,
        startupCommand: "laya-serve",
        startupTimeoutMs: 5000
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => {
    healthCheckCount += 1;
    return healthCheckCount >= 3;
  };
  (globalThis as Record<string, unknown>).__testLayaStartup = async (
    command: string,
    args: string[],
    env?: NodeJS.ProcessEnv
  ) => {
    startupCalls.push({ command, args, port: env?.LAYA_PORT ?? null });
  };
  (globalThis as Record<string, unknown>).__testLayaDecisionFetch = async () => ({
    ok: true,
    json: async () => ({ answers: { route: { choice: "reviewer" } } })
  });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({ id: "no-respawn-run-123" });

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.ok(inputHandler);

    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: () => undefined } }
    );

    assert.deepEqual(result, { action: "handled" });
    assert.deepEqual(startupCalls, []);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
    (globalThis as Record<string, unknown>).__testLayaDecisionFetch = originalLayaDecisionFetch;
  }
});

test("automatic routing keeps unmatched prompts on pass-through when laya is enabled", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-pass-through-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const originalLayaHealthCheck = (globalThis as Record<string, unknown>).__testLayaHealthCheck;
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;
  const notices: Array<{ message: string; level: string }> = [];
  let spawnCount = 0;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        enabled: true
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testLayaHealthCheck = async () => true;
  (globalThis as Record<string, unknown>).__testLayaStartup = async () => {
    throw new Error("should not start");
  };
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => {
    spawnCount += 1;
    return { id: "should-not-run" };
  };

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "hello there", source: "user" },
      { mode: "tui", ui: { notify: (message: string, level: string) => notices.push({ message, level }) } }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.equal(spawnCount, 0);
    assert.deepEqual(notices, [
      {
        message:
          "role-router automatic routing: pass through (no high-confidence delegated role matched) via heuristic fallback after Laya",
        level: "info"
      }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
    (globalThis as Record<string, unknown>).__testLayaHealthCheck = originalLayaHealthCheck;
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
  }
});

test("maybeEnsureLayaRuntime surfaces malformed laya config when role config is otherwise valid", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-invalid-block-"));
  const originalCwd = process.cwd();
  const originalLayaStartup = (globalThis as Record<string, unknown>).__testLayaStartup;

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      laya: {
        startupTimeoutMs: 0
      },
      roles: {
        reviewer: {
          prompt:
            "You are the reviewer role. Review the work critically, focus on correctness and risk, and report findings before suggestions.",
          description: "Reviewer"
        }
      }
    })
  );

  let startupCalled = false;
  (globalThis as Record<string, unknown>).__testLayaStartup = async () => {
    startupCalled = true;
  };

  try {
    await assert.rejects(
      maybeEnsureLayaRuntime(),
      /role-router config 'laya.startupTimeoutMs' must be a positive integer when set\./
    );
    assert.equal(startupCalled, false);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testLayaStartup = originalLayaStartup;
  }
});

test("automatic routing ignores malformed role-router config when laya is optional", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-laya-invalid-config-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const notices: Array<{ message: string; level: string }> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(join(tempDir, ".pi", "role-router.json"), "{");

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({ name }: { name: string }) => ({
    dispose() {},
    name
  });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({ id: "invalid-config-run-123" });

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    let inputHandler:
      | ((
          event: { text: string; source: string },
          ctx: { mode: string; ui: { notify: (message: string, level: string) => void } }
        ) => Promise<{ action: string }>)
      | undefined;

    roleRouter({
      on: (name: string, handler: typeof inputHandler) => {
        if (name === "input") inputHandler = handler;
      },
      registerCommand: () => undefined,
      events: { emit: async () => undefined }
    } as never);

    assert.ok(inputHandler);
    const result = await inputHandler(
      { text: "please review this diff for bugs", source: "user" },
      { mode: "tui", ui: { notify: (message: string, level: string) => notices.push({ message, level }) } }
    );

    assert.deepEqual(result, { action: "continue" });
    assert.deepEqual(notices, [
      { message: "role-router role/config error: invalid JSON in .pi/role-router.json.", level: "error" }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});

test("delegate-role receipt works with project-local agent overrides", async () => {
  const tempDir = await mkdtemp(join(tmpdir(), "role-router-receipt-override-"));
  const originalCwd = process.cwd();
  const originalRegisterAgent = (globalThis as Record<string, unknown>).__testRegisterAgent;
  const originalTestSubagentsSpawn = (globalThis as Record<string, unknown>).__testSubagentsSpawn;
  const notices: Array<{ message: string; level: string }> = [];

  process.chdir(tempDir);
  await mkdir(join(tempDir, ".pi"), { recursive: true });
  await writeFile(
    join(tempDir, ".pi", "role-router.json"),
    JSON.stringify({
      roles: {
        planner: {
          agent: "delegate-planner",
          prompt: "Custom planner prompt",
          description: "Custom planner"
        }
      }
    })
  );

  (globalThis as Record<string, unknown>).__testRegisterAgent = ({
    name,
    definition
  }: {
    name: string;
    definition: Record<string, unknown>;
  }) => ({ dispose() {}, name, definition });
  (globalThis as Record<string, unknown>).__testSubagentsSpawn = async () => ({ id: "override-run-123" });

  try {
    const { default: roleRouter } = await import("../extensions/roleRouter.ts");
    const commands = new Map<string, { handler: (args: string, ctx: unknown) => Promise<void> | void }>();

    roleRouter({
      registerCommand: (
        name: string,
        definition: { handler: (args: string, ctx: unknown) => Promise<void> | void }
      ) => {
        commands.set(name, definition);
      },
      events: {
        emit: async () => undefined
      }
    } as never);

    const command = commands.get("delegate-role");
    assert.ok(command);

    await command.handler('{"role":"planner","task":"Plan the rollout"}', {
      mode: "tui",
      ui: {
        notify: (message: string, level: string) => {
          notices.push({ message, level });
        }
      }
    });

    assert.deepEqual(notices, [
      {
        message: "role-router launch receipt: planner\nagent: delegate-planner\nhandle: override-run-123",
        level: "info"
      }
    ]);
  } finally {
    process.chdir(originalCwd);
    (globalThis as Record<string, unknown>).__testRegisterAgent = originalRegisterAgent;
    (globalThis as Record<string, unknown>).__testSubagentsSpawn = originalTestSubagentsSpawn;
  }
});
