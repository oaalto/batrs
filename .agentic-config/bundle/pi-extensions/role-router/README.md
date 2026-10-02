# role-router

Pi extension package bundled by ADC as a **Bundle-installed Pi extension/package**.

## Purpose

ADC stages this package under `.agentic-config/bundle/pi-extensions/role-router/` and the installer places it into the target project's `.pi/extensions/role-router/` (no `pi install`, no network).

`role-router` is a thin role router: **Automatic role selection** is the default ordinary-chat path, so it can auto-route ordinary chat text to one configured delegated role when the request is a high-confidence delegated-work candidate, or route a caller-selected role through the explicit command path. It does **not** replace the main session as the orchestrator, and it does **not** embed a second orchestration framework.

This package depends on `pi-subagents` for delegated execution. The router package owns only the role-to-target runtime seam; `pi-subagents` owns the actual child-run launch machinery. When Pi installs packages under the local `.pi/npm/node_modules/` tree, `role-router` falls back to Pi's local package tree if ordinary package resolution does not see `pi-subagents` from the extension directory.

## How to Use

Ordinary free-text chat is now the default input seam. `role-router` inspects incoming chat text before normal handling and chooses between two outcomes only:

- pass the message through unchanged to the main session, or
- route the whole message to exactly one enabled delegated role when one high-confidence match is clear

Auto-routing stays intentionally narrow in v1:

- only enabled configured roles are considered
- one user message launches at most one delegated run
- ambiguous or low-confidence multi-role matches pass through unchanged
- automatic routing runs only for TUI chat input; slash commands, bash input, and non-TUI input stay on the normal pass-through path
- free-text context extraction is conservative: clearly stated slash-delimited paths, explicit constraints, and explicit artifact references can be carried into delegated handoff fields, including absolute paths and paths under `tmp/` when the user names them explicitly, but vague text does not trigger broad repository search or fuzzy guessing

`/delegate-role` remains available as the explicit override path and accepts a small additive JSON contract:

```text
/delegate-role {"role":"planner","task":"Plan the rollout","context":{"summary":"optional extra brief","files":["pi-extensions/role-router/extensions/roleRouter.ts"],"constraints":["keep the thin-router boundary unchanged"],"artifacts":["tmp/plan.md"]}}
```

Built-in roles:

- `planner`
- `developer`
- `reviewer`

The extension resolves one delegated target per role and routes the delegated run through that target instead of implementing its own orchestration layer. Built-in roles default to the same-named pi-subagents builtin agents (`planner` uses `planner` when configured, `reviewer` uses `reviewer`, and so on), while `.pi/role-router.json` may override the delegated target with `agent`. On successful live launch, `/delegate-role` reports a compact launch receipt with the requested role, resolved delegated agent, the delegated child-run handle when the delegated seam returns one, and any immediate output reference or output path when that is what the delegated seam returns.

Project-local overrides live in `.pi/role-router.json`:

```json
{
  "laya": {
    "enabled": true,
    "healthUrl": "http://127.0.0.1:8877/health",
    "decisionUrl": "http://127.0.0.1:8877/v1/systemone",
    "startupCommand": "laya-serve",
    "startupTimeoutMs": 5000,
    "startupModel": "english",
    "startupPort": 8877,
    "startupHost": "127.0.0.1",
    "startupDevice": "cuda",
    "startupPreload": true,
    "startupThreads": 8
  },
  "roles": {
    "planner": {
      "agent": "delegate",
      "model": "openai/gpt-5",
      "prompt": "You are the planner role. Keep plans compact.",
      "description": "Custom planner",
      "thinking": "high",
      "async": false,
      "context": "fork"
    },
    "reviewer": {
      "enabled": false
    }
  }
}
```

`agent` selects the actual delegated target name for that role; `model` and `prompt` override the generated run settings for that same delegated target. `async` and `context` are per-role delegated runtime defaults: when omitted they keep the built-in launch defaults (`true` and `fresh`). The optional `laya` block only affects ordinary-chat **Automatic role selection**: when `enabled: true`, role-router makes one best-effort `laya-serve` startup attempt during extension startup, remembers that in-flight local process while it is still coming up, probes the local Laya health URL again on every routed input, retries startup only after the previous process has exited or the runtime is still unhealthy later, waits only for a tiny local probe window per check, and when Laya is healthy sends one typed choice decision to the configured decision URL. Startup settings are a dedicated env-backed seam only for a local `laya-serve` process that `role-router` launches itself: `startupModel` maps to `LAYA_DEFAULT_MODEL`, `startupPort` to `LAYA_PORT`, `startupHost` to `LAYA_HOST`, `startupDevice` to `LAYA_DEVICE`, `startupPreload` to `LAYA_PRELOAD`, and `startupThreads` to `LAYA_THREADS`. Those fields do not change delegated role `model` overrides: role `model` still controls the child agent that `pi-subagents` launches, while `laya.startupModel` only selects which local routing checkpoint `laya-serve` should boot for automatic role selection. A healthy already-running external Laya process stays authoritative; these startup fields do not introspect or reconcile its effective runtime. The valid outcomes are exactly `pass_through` plus the currently enabled configured roles. `pass_through` keeps the message in the main session, a valid enabled role launches through the existing delegated `pi-subagents` seam, and invalid, disabled, unknown, malformed, or failed Laya results fall back unchanged to the existing heuristic routing path. For broader official Laya model background, see `docs/research/laya_models_overview.md` and `docs/research/laya_serve_configuration_options.md`, but treat the installed Laya runtime as the source of truth for accepted startup values. Command-supplied free-text `context` still wins for the rendered task handoff text; when the command omits it, the role's delegated `context` default is used for both child-run launch mode and previewed handoff text.

The public command contract stays intentionally small and additive:

- `role` — required non-empty string
- `task` — required non-empty string
- `context` — optional non-empty string or structured object
  - `summary` — optional non-empty string
  - `files` — optional array of non-empty strings
  - `constraints` — optional array of non-empty strings
  - `artifacts` — optional array of non-empty strings

The structured `context` object is the canonical rich handoff shape. For backward compatibility, top-level `files` and `constraints` are still accepted as temporary aliases and normalize into the same structured `context` object. Mixed duplicate sources for the same field fail as input errors instead of silently choosing precedence.

When present, structured fields are rendered into the same delegated task preview and live handoff used by launch, dry run, and trace in stable order: `Summary`, `Files`, `Constraints`, `Artifacts`. Omitted fields disappear cleanly with no empty sections.

`role-router` now reports four user-facing failure classes at the existing runtime seam:

- **Input errors** — malformed `/delegate-role` JSON or missing required fields
- **Role/config errors** — unknown roles, disabled roles, and malformed `.pi/role-router.json`
- **Dependency/setup errors** — `pi-subagents` missing, disabled, or failing to load
- **Delegated launch failures** — the selected role resolved, but the delegated child-run launch failed

Example mappings:

- `role-router role/config error: unknown role 'missing'.`
- `role-router role/config error: role 'reviewer' exists but is disabled.`
- `role-router role/config error: invalid JSON in .pi/role-router.json.`
- `role-router dependency/setup error: unable to load the pi-subagents delegated execution bridge because pi-subagents is missing, disabled, or failed to load. Load detail: MODULE_NOT_FOUND: Cannot find module 'pi-subagents/agents' ...`
- `role-router delegated launch failed for role 'planner': <child launch error>`

The live `/delegate-role` receipt and the automatic-routing receipt are launch observability only. They are not result aggregation, child completion tracking, or a second orchestration API. Automatic routing now emits the existing parent-session explanation seam plus, when a launch happens, the same compact launch receipt used by the explicit command path. That explanation makes the final outcome observable: users can tell whether Laya made the routing choice directly or whether `role-router` fell back to the built-in heuristic path after optional Laya consultation. When the delegated seam is the RPC bridge, `role-router` now waits for the matching `subagents:rpc:v1:reply:<requestId>` response instead of treating fire-and-forget event emission as a successful launch, so a visible receipt means the subagent owner actually acknowledged the spawn request. Receipts only show a `handle:` line when the delegated seam actually returns a child-run handle; output-only launch responses show just the `output:` line instead of a placeholder missing handle. When free text clearly names slash-delimited paths, constraints, or artifact references, the same delegated handoff preview carries them into `Files`, `Constraints`, and `Artifacts`; that includes explicit absolute paths and explicit temp paths such as `tmp/plan.md`. Vague mentions stay as plain task text. Ambiguous and low-confidence automatic pass-through outcomes now leave an inspectable explanation in the parent session instead of silently disappearing. Preview-only `/dry-run-role-router` and `/trace-role-router` remain unchanged in intent and do not invent real handles before a delegated spawn happens.

`/trace-role-router` explains one routing decision without delegated execution. It accepts the same JSON contract as `/delegate-role`:

```text
/trace-role-router {"role":"planner","task":"Plan the rollout","context":{"summary":"optional extra brief","files":["pi-extensions/role-router/extensions/roleRouter.ts"],"constraints":["keep the thin-router boundary unchanged"],"artifacts":["tmp/plan.md"]}}
```

Success reports the requested role, whether the effective route stayed on built-in defaults or used project config, whether `agent`, `model`, `thinking`, `async`, and delegated `context` came from defaults or config overrides, the resolved delegated agent, the effective model and thinking values when present, the live runtime defaults used by delegated launch, and the rendered delegated task payload preview. It stays config-only, does not launch a child run, and does not require `pi-subagents` to load.

`/dry-run-role-router` previews the delegated target without delegated execution. It accepts the same JSON contract as `/delegate-role`:

```text
/dry-run-role-router {"role":"planner","task":"Plan the rollout","context":{"summary":"optional extra brief","files":["pi-extensions/role-router/extensions/roleRouter.ts"],"constraints":["keep the thin-router boundary unchanged"],"artifacts":["tmp/plan.md"]}}
```

Success reports the resolved role, delegated agent, effective model and thinking overrides when present, the effective delegated `async` and `context` defaults used by live launch, and the rendered delegated task payload preview. It stays config-only and does not require `pi-subagents` to load.

Malformed command JSON stays an input error. Unknown roles, disabled roles, malformed `.pi/role-router.json`, unknown structured `context` keys, duplicate alias-plus-nested fields, all-empty structured objects, and malformed `summary` / `files` / `constraints` / `artifacts` shapes stay input or role/config errors at the same existing boundary.

`/validate-role-router` validates the current role-router config without delegated execution. With no arguments it checks `.pi/role-router.json`; with an explicit path it accepts `{"path":"tmp/role-router.json"}`. Success reports the resolved role set, including disabled roles, and malformed JSON or invalid override fields report a role/config error.

## Deferred improvements backlog

The first version stays intentionally small. Deferred follow-up work lives in the dedicated backlog artifact [`docs/research/role-router-deferred-improvements-backlog.md`](../../docs/research/role-router-deferred-improvements-backlog.md), which records the earlier explicit-routing backlog lineage from parent spec [#261](https://github.com/kodanbce/aura-v2/issues/261). The default-on ordinary-chat routing boundary is the newer parent spec [#281](https://github.com/kodanbce/aura-v2/issues/281). The boundary rationale for why default-on **Automatic role selection** still stays thin is recorded in [ADR 0045](../../docs/adr/0045-role-router-default-on-automatic-role-selection.md).

Grouped deferred themes:

- **Dispatch intelligence**
- **Runtime policy knobs**
- Richer orchestration or metadata beyond the shipped structured `context` object
- **Orchestration ownership**

The one-launch-per-message limit is a v1 boundary, not a claim that broader routing can never exist.

## Update

Snapshotted per ADC release — re-download the ADC bundle to take a newer version. `pi update` does not apply.
