---
name: improve-documentation
description: Audit human-facing project documentation (README, USAGE, operational guides) against repo evidence; present a markdown improvement report before applying edits. Excludes docs/wiki/ and other agent-maintained engineering memory — use the wiki skill for those.
---

<what-to-do>

Audit **human-facing project documentation** for accuracy, staleness, duplication, and risky instructions. Cross-check against repo evidence. Present findings as a markdown report and **stop** until the user accepts, revises, or rejects.

**Do not edit any files during the audit phase** — read-only discovery and cross-check only.

When the report is complete, wait for the user to **reject**, **revise** selected rows, or **accept**.

- **Reject** — discard the report; do not edit files. Offer to restart if useful.
- **Revise** — update only the rows the user specifies; show the revised report; still no file writes until they accept.
- **Accept** — apply accepted rows to in-scope documentation files only (see [After acceptance](#after-acceptance)).

</what-to-do>

## Documentation improvement report

Present every finding in one table:

| ID  | File | Section | Problem | Proposed change | Strength                               | Evidence |
| --- | ---- | ------- | ------- | --------------- | -------------------------------------- | -------- |
| 1   | …    | …       | …       | …               | Strong / Worth exploring / Speculative | …        |

Rules:

- **ID** — stable row identifier for accept/revise references.
- **File** — path relative to repo root; cite the section or heading affected.
- **Problem** — what is wrong, stale, duplicated, or missing.
- **Proposed change** — concrete wording or structural fix; preserve the file's existing tone.
- **Strength** — **Strong** (clear contradiction or high-impact risk), **Worth exploring** (likely improvement), **Speculative** (nice-to-have).
- **Evidence** — pointer to repo fact (manifest, compose file, CI workflow, env example, code path).

After the table, add:

- **Duplication map** — cross-file overlap; recommend link targets instead of copy-paste blocks.
- **Out of scope** — items that need code changes or wiki updates (separate issues); do not apply here.

<supporting-info>

## Scope

### In scope (discover — do not assume a fixed layout)

Locate documentation surfaces the project actually uses:

- Root `README.md`
- Root human entry doc (`USAGE.md` or equivalent — find the file the repo treats as the daily agent/human entry point)
- Root backlog when present (`TODO.md`, `TODO`, `BACKLOG.md`)
- `docs/LOCAL_TEST_AND_CI_INSTRUCTIONS.md` and `docs/DEPLOY.md` when present
- Other top-level operational guides under `docs/*.md`
- MkDocs manual entry points and operator-facing docs such as `docs/index.md` and `docs/manual/*.md`

### Explicit exclusions

- `docs/wiki/**` and other agent-maintained engineering memory — owned by the `wiki` skill
- Generated help-book / static-site trees
- `.agentic-config/**` and upstream-installed skill bodies
- Runtime code, tests, and config unless the user explicitly expands scope in conversation

### Default policies

- **README as front door** — keep README concise; prefer links to detailed guides over duplicating setup blocks.
- **No renames or moves** — do not rename or relocate documentation files unless the user explicitly requests it in the acceptance step.
- **Terminology** — align proposed wording with `CONTEXT.md`; surface conflicts in the report.
- **Command safety** — documented commands must match evidenced tooling; flag infra-destructive or restricted commands and recommend read-only alternatives or explicit permission prompts. Do not run mutating shell commands while fixing docs unless the user explicitly approves.
- **Agent command alignment** — when docs describe developer workflows, cross-check wording and commands against `docs/agent-commands.md` and the repo's slash-command conventions.

## Workflow phases

### 1. Discover

- Locate in-scope documentation files.
- Read the domain glossary (`CONTEXT.md`, `CONTEXT-MAP.md`, or project equivalent). In this repo, `CONTEXT.md` is the canonical terminology source.
- When present, read `docs/agent-commands.md` or the install-time command map for slash-command conventions.
- For this repo, inspect `README.md`, `docs/index.md`, and `docs/manual/*.md` before deciding the doc surface is complete.

### 2. Audit (read-only)

Cross-check documentation against repo evidence:

- Package manifests and workspace layout
- Docker Compose / deployment manifests
- CI workflows and test scripts
- Env examples (`.env.example`, sample configs)
- Port numbers, auth setup, model alias wording, and infra mutation commands
- For this repo, Cargo/MkDocs workflow evidence such as `Cargo.toml`, `README.md`, `mkdocs.yml`, `requirements-docs.txt`, and `scripts/serve-manual.sh`

Flag contradictions, staleness, missing links, duplication, and doc-debt items in backlog files when tracked.

### 3. Report

Produce the [Documentation improvement report](#documentation-improvement-report) table and supplementary sections. **Stop** — no file edits.

### 4. Human gate

Wait for reject, revise, or accept. On revise, update only specified rows and re-present the table.

### 5. Apply (after acceptance only)

- Edit **only** accepted rows in allowed doc paths.
- Preserve each file's existing tone and structure — no generic template overwrite.
- Do not rename, move, or delete documentation files unless explicitly requested.
- Do not touch excluded paths or runtime code.

## Common doc-risk scenarios

Prioritize findings that match these patterns when evidence supports them:

- Wrong ports, URLs, or service names vs compose/k8s manifests
- Auth or secret setup that does not match env examples
- Model alias or API wording that does not match deployment catalog or proxy config
- `docker compose up`, `kubectl apply`, `terraform apply`, or other restricted/infra-mutating commands presented without permission prompts
- Test or build commands that do not match `package.json` scripts, Cargo workflows, or CI workflows
- README duplicating long sections already present in operational guides or the MkDocs manual
- Rust/MkDocs repos where docs mention `npm`/frontend tooling despite repo evidence showing Cargo + Python docs tooling instead

## After acceptance

Apply accepted rows to the in-scope files only. When a change affects cross-references, update links in the same edit pass. If accepted edits reveal code/runtime drift that is out of scope, note follow-up issues in the closing summary — do not expand into code changes.

</supporting-info>
