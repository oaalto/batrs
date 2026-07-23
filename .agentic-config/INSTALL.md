# Agentic Development Configurator setup

This bundle includes **upstream** skills and/or Pi extensions. Repo-local agent files (`docs/`, rules, skills) are already at the project root; run the installer below to pull external dependencies.

## Prerequisites

`install.sh` runs an **environment check** before any install step. It verifies every tool required by your `install-plan.json` and prints docs URLs plus example install commands for anything missing.

| Tool                                   | Needed for                                                                       | Install docs                                                                                      |
| -------------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `jq`                                   | Reading `install-plan.json`                                                      | https://jqlang.org/download/                                                                      |
| [Node.js](https://nodejs.org/) + `npx` | `npx skills add …` ([vercel-labs/skills](https://github.com/vercel-labs/skills)) | https://nodejs.org/en/download/                                                                   |
| [Pi](https://pi.dev) CLI               | `pi install …`                                                                   | https://pi.dev/docs/latest/usage#cli-reference — `npm install -g @earendil-works/pi-coding-agent` |
| [uv](https://docs.astral.sh/uv/)       | `uv tool install "graphifyy[mcp]"` (Graphify CLI; uv manages Python)             | https://docs.astral.sh/uv/getting-started/installation/                                           |

Other tools (`curl`, `wget`, `git`) appear only if your plan includes custom shell steps that need them.

When your plan includes Graphify install steps, add `graphify-out/` to `.gitignore` so generated graph artifacts are not committed. Allow agent-visible markers (for example `graphify-out/*` with `!graphify-out/.graphify_semantic_marker` and `!graphify-out/GRAPH_REPORT.md`). On Pi, add `!graphify-out` and `!graphify-out/**` to `.piignore` when using the pi-ignore extension so gitignored graph paths are discoverable.

## Graphify extraction backend

`graphifyy[mcp]` installs the **CLI and MCP only**. Semantic extraction of docs, papers, and images requires a separate **Graphify extraction backend** — an LLM provider the operator configures before the first `graphify .` build. This is independent of your target agent's chat model (configuring Pi or Cursor for Aura chat does **not** configure graphify).

When Graphify is enabled, the bundle includes `graphify.env.example` at the project root (placeholders only — no secrets).

### Install the matching pip extra

Re-run with the extra that matches your backend (examples use uv; pipx is an operator alternative — not run by `install.sh`):

```bash
uv tool install 'graphifyy[mcp,openai]' --force    # OpenAI-compatible (cloud or Aura proxy)
uv tool install 'graphifyy[mcp,gemini]' --force    # Gemini
uv tool install 'graphifyy[mcp,anthropic]' --force # Anthropic
uv tool install 'graphifyy[mcp,ollama]' --force    # Ollama (local)
uv tool install 'graphifyy[mcp,deepseek]' --force  # DeepSeek
```

### Curated backend paths

| Backend                 | uv extra    | Typical env vars                                                                                     |
| ----------------------- | ----------- | ---------------------------------------------------------------------------------------------------- |
| **Gemini**              | `gemini`    | `GEMINI_API_KEY` or `GOOGLE_API_KEY`                                                                 |
| **OpenAI-compatible**   | `openai`    | `OPENAI_API_KEY`; optional `OPENAI_BASE_URL`, `OPENAI_MODEL`                                         |
| **Aura proxy**          | `openai`    | `OPENAI_API_KEY` (Project API Key), `OPENAI_BASE_URL` (Aura `/v1`), `OPENAI_MODEL` (**Model Alias**) |
| **Anthropic**           | `anthropic` | `ANTHROPIC_API_KEY`                                                                                  |
| **Ollama**              | `ollama`    | `OLLAMA_BASE_URL` (for example `http://localhost:11434`)                                             |
| **DeepSeek / Moonshot** | `deepseek`  | `DEEPSEEK_API_KEY` or `MOONSHOT_API_KEY`                                                             |
| **Code-only**           | _(none)_    | No LLM env — AST indexing only; **discouraged** on doc-heavy repos                                   |

Copy `graphify.env.example`, fill placeholders, and export vars in your shell before the first build.

### First graph build

After backend env is configured:

```bash
graphify .                  # auto-detect backend from env
graphify . --backend=gemini # or openai, anthropic, ollama, etc. when needed
```

`install.sh` prints a **warn-only** reminder when no extraction backend env is detected after graphify install steps. Install still succeeds — AST-only indexing remains possible.

### pipx alternative (operator-managed)

`install.sh` uses uv only. With pipx: `pipx install 'graphifyy[mcp]'` then `pipx inject graphifyy openai` (or the extra matching your backend). Add pipx's bin directory to PATH.

## Install steps

From your **project root** (parent of `.agentic-config/`):

```bash
chmod +x .agentic-config/install.sh
./.agentic-config/install.sh
```

The script prints `[ok]` or `[MISSING]` for each required tool. Fix missing tools, then run again.

Preview commands without executing (env check still runs):

```bash
./.agentic-config/install.sh --dry-run
```

Skip the environment check (not recommended):

```bash
./.agentic-config/install.sh --skip-env-check
```

Upstream `npx skills add` and `pi install` steps are **non-fatal by default**: a failed step prints `[warn]`, the installer continues with bundled rules and skills, and exits **0** with a warning count when no hard failures occurred. Use strict mode when you want fail-fast behavior (for example CI):

```bash
./.agentic-config/install.sh --strict
```

On Windows PowerShell: `.\.agentic-config\install.ps1 -Strict`

Headroom and Graphify tool install steps still fail hard when they error - those are operator-chosen runtime dependencies, not best-effort catalog fan-out.

Windows (PowerShell, from project root):

```powershell
.\.agentic-config\install.ps1
```

Use `-SkipEnvCheck` to bypass the upfront check on Windows.

## What gets installed

See `.agentic-config/install-plan.json` for the exact command list. Typical entries:

- **Pi packages** — `pi install -l npm:…` (project-local under `.pi/settings.json`)
- **Engineering skills** — `npx -y skills@latest add <owner/repo> --skill <name> --agent <agent> -y`
- **Graphify** — `uv tool install "graphifyy[mcp]"` then `graphify <platform> install --project` (when Graphify is enabled on the Memory step)
- **Custom shell** — freeform commands you added in the Agentic Development Configurator (rules fetched via `curl`, hooks, etc.)

Custom **rules** marked “bundle” in the Agentic Development Configurator are already inside the zip (not run by this script).

Edit `docs/agents/*.md` directly later; re-run `./.agentic-config/install.sh` only when adding new upstream skills or Pi packages.

## Headroom (context optimization)

When your bundle includes Headroom install steps, **uv** must be on PATH. ADC installs the PyPI package via:

```bash
uv tool install 'headroom-ai[proxy,mcp]'
```

Ensure `~/.local/bin` is on PATH so the `headroom` CLI is discoverable after install.

### Native Windows (`install.ps1`)

PyPI `headroom-ai` publishes prebuilt wheels for **Linux and macOS only** — not for native Windows. On Windows, `uv tool install` builds the Rust extension from source and needs **Rust (`cargo` on PATH)** and **Visual Studio Build Tools** with the **Desktop development with C++** workload (`link.exe`). `install.ps1` exits before `headroom-tool-install` when either prerequisite is missing.

Without that toolchain you will see `linker 'link.exe' not found` / maturin failures if you bypass the check.

Alternatives:

- Run Headroom under **WSL** or another Linux environment and point the Pi extension/proxy at that runtime.
- Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the C++ workload, then re-run `.\.agentic-config\install.ps1`.

| Step kind               | Command                                                | Required when on                            |
| ----------------------- | ------------------------------------------------------ | ------------------------------------------- |
| `headroom-tool-install` | `uv tool install 'headroom-ai[proxy,mcp]'`             | runtime, MCP, or Pi extension               |
| `headroom-mcp-install`  | `headroom mcp install`                                 | MCP sub-option (Cursor, Claude Code, Codex) |
| `headroom-pi-extension` | `pi install -l npm:@ryan_nookpi/pi-extension-headroom` | Pi extension on Pi target                   |

**Docs:** https://github.com/headroomlabs-ai/headroom

**Pi alternative (not ADC default):** `npm:pi-headroom` — ADC v1 uses `@ryan_nookpi/pi-extension-headroom`.

### Proxy and wrap usage

After install, verify `headroom` responds (`headroom --help`). Use the local proxy or `headroom wrap` launchers per upstream docs for your target agent. On Pi, prefer the installed extension for tool-result compression instead of Headroom MCP in v1.

Consider ignoring Headroom cache directories in `.gitignore` when upstream documents a local cache path.

The bundled `headroom-consultation` rule (when selected) is zip-only — it does not gate install success. Rule and guide sub-options never abort `install.sh`.

## Reload your agent

- **Pi**: run `/reload` in an interactive session.

## Post-install review

After reload, run a **post-install review** with your coding agent in this repository.

| File                                 | Use when                                          |
| ------------------------------------ | ------------------------------------------------- |
| `post-install-review-integration.md` | Tailoring to an **existing repository**           |
| `post-install-review-greenfield.md`  | **Greenfield install** with little domain context |
| `post-install-review-update.md`      | **Update** to a prior ADC installation (detected) |

**Start with:** `post-install-review-integration.md` (`integration` variant)

Open the file, select all, and paste into your coding agent. Review the Setup completion report when the run finishes.


See `.agentic-config/USAGE.md` for slash commands and the human review checklist.

### Auto-detection

`install.sh` / `install.ps1` detect the install type from git history:

- **No commits** → **greenfield** — suggests `post-install-review-greenfield.md`
- **Has commits, no prior `.agentic-config/` in git** → **integration** — suggests `post-install-review-integration.md`
- **Has commits + prior `.agentic-config/` tree in git** (manifest or other bootstrap files) → **update** — suggests `post-install-review-update.md`

Update detection inspects **committed** git history only. An uncommitted prior manifest is not visible after you unzip the new bundle; commit `.agentic-config/manifest.json` (or re-run install from a clean tree) when you want update guidance.

After the install script finishes, it prints the detected type and the recommended review file path. Use that to pick the right variant above.

## Cleaning up

Bootstrap files live under `.agentic-config/` (`INSTALL.md`, `manifest.json`, `post-install-review-integration.md`, `post-install-review-greenfield.md`, `post-install-review-update.md`, `install.sh`, `install.ps1`, `install-plan.json`, `USAGE.md`). These files are not removed automatically.

When you no longer need install metadata or onboarding docs:

```bash
rm -rf .agentic-config
```

Keep `.agentic-config/` if you expect to re-run the installer or audit selections via `manifest.json`.