# Complete agent setup — greenfield install

You are helping a human **finish installing** an agent setup bundle in the **current repository**. The user already unzipped files into the repo root and ran `./.agentic-config/install.sh` when an installer was included.

## Setup context (from install — do not re-ask)

**Mode:** Greenfield install — validate new agent setup without host merge
- **Agent platform:** Pi — main file `AGENTS.md`; invoke skills as `/skill:name`; reload with `/reload`
- **Main rules file:** `AGENTS.md`
- **Files that may have replaced earlier versions:** `docs/wiki/index.md`, `docs/wiki/schema.md`, `docs/wiki/log.md`, `docs/wiki/path-map.json`, `docs/wiki/concepts/.gitkeep`, `docs/wiki/subsystems/.gitkeep`, `docs/wiki/workflows/.gitkeep`, `docs/wiki/debugging/.gitkeep`, `docs/wiki/known-traps/.gitkeep`, `docs/wiki/source-notes/.gitkeep`, `docs/wiki/synthesis/.gitkeep`, `AGENTS.md`
- **Seed doc paths in bundle:** `CONTEXT.md`, `docs/wiki/index.md`, `docs/agent-commands.md`
- **Documentation kept from repo (bundle did not replace):** detect in repo — for each seed path below, use `git show HEAD:{path}`; when it succeeds, treat as pre-existing (bundle did not replace) and review under **Existing documentation review**
- **Engineering wiki:** enabled — `docs/wiki/` installed (index, schema, log, topic directories)
- **Graphify:** not installed in this bundle
- **Headroom:** enabled — runtime install steps in `.agentic-config/install.sh`; see `.agentic-config/INSTALL.md` for proxy, MCP, or Pi extension workflow
- **Skills added:** wiki, code-review, codebase-design, diagnosing-bugs, domain-modeling, grill-with-docs, grilling, handoff, implement, improve-codebase-architecture, ponytail, prototype, repo-navigation, research, resolving-merge-conflicts, review, tdd, teach, to-spec, to-tickets, triage, vertical-slice-migration, wayfinder, workflow, zoom-out
- **Rules added:** ponytail, commit, decision-making, definition-of-done, role, signature, strict-output-execution, adr-discipline, documentation, domain-language, dependency-boundaries, vertical-slice-boundaries, current-state, restricted-operations, testing, api-design-basics, code-format, functional-programming, result-handling, warning-hygiene, logging-practices, runtime-handoff, headroom-consultation, rust-api-semver, rust-dependency-hygiene, rust-error-handling, rust-observability, rust-testing-strategy, rust-workflow-gates
- **Install record:** `.agentic-config/manifest.json` (files and selections), `.agentic-config/install-plan.json` (installer steps when present)

## Constraints (mandatory)

- **Application code:** Do not modify product source (application packages, services, or UI code outside agent-setup paths).
- - **What you may edit directly:** agent-setup paths the bundle added; and **initial wiki bootstrap** when creating new wiki pages from verified repo sources.
- **Propose before editing:** `CONTEXT.md`, `CONTEXT-MAP.md`, changes to existing non-empty wiki pages, `docs/adr/`, and command lines in `docs/agent-commands.md`. List changes under **Proposed changes** in the report; wait for human approval before applying.
- **Evidence:** Use on-disk files and `.agentic-config/manifest.json`. Mark unknowns under **Remaining follow-ups**.
- **Workflow commands:** Read gates from `docs/agent-commands.md`. Run format → build/typecheck → lint → test when commands are present. Skip human-only or runtime-restricted steps. Stop on first failure unless the doc marks a gate as optional.

## How to work with the human

This is a **setup interview**, not a silent one-shot pass.

- **One question at a time** when a decision materially affects the setup. Include your **recommended answer** with each question.
- **Investigate first** when the answer is likely in the codebase, docs, or git history; do not ask what you can verify.
- **Act without asking** when constraints and evidence make the action clear.
- **Wait** for the human's reply before the next question.
- **Checkpoint before domain edits:** present **Proposed changes** and ask which to apply before editing pre-existing `CONTEXT.md`, wiki pages, or `docs/agent-commands.md`. **Exception:** obvious vendored seed stubs (placeholders only, no substantive git version) may be filled directly when repo evidence is clear — see **Seed docs fill**.

**Context to gather** (when not already clear from repo discovery):

- Tie questions to **selected skills and rules** from `manifest.json` — how they should behave in this repo, stack-specific conventions, or gaps the wizard cannot capture.
- When filling `docs/agent-commands.md`: ask about **tools and frameworks** (test runner, linter, formatter, bundler, monorepo tool, package manager) — **not** exact npm/pnpm script names unless not visible in the repo.
- When seed doc paths are present: ask about **domain language** — project or product name, core domain terms, bounded contexts when multi-context applies, and initial wiki concepts worth documenting.
- When you have enough context, you may **close without a question** — say briefly that you have no more questions for now. Do not rush to close while meaningful gaps remain.

Do not assume any particular slash command or skill is installed — follow the rules above directly.

Work through the task sections below iteratively. Pause for questions at decision points; produce the **Setup completion report** when the human is ready to finish (or when remaining work is only follow-ups).

## Your task (in order)

1b. Verify Headroom is installed and healthy: `command -v headroom`, proxy/MCP/Pi-extension status for the target agent, and target-specific `headroom wrap` or proxy workflow per `.agentic-config/INSTALL.md`. Note optional cache-directory `.gitignore` entries when upstream documents a local cache path.

### Duplicate content audit

1. For each newly installed skill (`.agents/skills/*/SKILL.md`), scoped rule (`.agents/rules/*.md`), and the host file (`AGENTS.md`), scan for **identical or near-identical blocks** — especially sections like `## Repo Context`, `## Project overview`, or bullet lists describing stack, monorepo layout, or team conventions.
   - **Ponytail Pi overlap:** When `ponytail` is selected (skill or rule) and the target is Pi, compare the bundled Ponytail rule (inline `### ponytail` in `AGENTS.md` for compiled hosts, or `.cursor/rules/ponytail.mdc` for Cursor) with Ponytail content injected by the Pi extension. Remove duplicate ladder blocks; keep one canonical always-on copy in the bundled rule and replace duplicates with short cross-references. Do not edit upstream Ponytail slash skills delivered by `pi install`.
1c. Scan bundled `headroom-consultation` against upstream RTK-style Headroom blocks in `AGENTS.md` (and other host files). Merge overlap; keep `headroom-consultation` where it fills gaps upstream does not cover.
2. When the same global context block appears in **two or more files**, remove it from the duplicates:
   - Keep it in at most one location (prefer `AGENTS.md` for compiled-markdown targets, or the root-level documentation file).
   - If the block is only a high-level repo overview, remove it from skills and rules entirely — those files should contain their own scoped instructions, not a project summary.
2. When a duplicated block has been customized per-file (e.g., slightly different stack notes in each skill), merge the unique details into one authoritative version and replace the rest with a short cross-reference (e.g., "See project stack details in `CONTEXT.md`" if applicable).
3. Do not delete unique, file-specific content — only remove blocks that are repeated across files with no file-specific variation.

### Seed docs fill

Fill every seed stub path in this bundle:

- `CONTEXT.md`
- `docs/wiki/index.md`
- `docs/agent-commands.md`

For each path:

1. Read the on-disk file and check `git show HEAD:{path}` for a pre-install version.
2. **Obvious vendored stub** (placeholders like `{Term}`, `Not configured yet.`, and no substantive pre-existing git version): write a **complete** replacement grounded in repo discovery or inspect-only evidence — **apply directly** when facts are clear. List what you wrote in the report; no approval gate for these paths.
3. **Ambiguous stub** (mixed placeholder and real content, or key facts still uncertain): draft under **Proposed changes** and wait for human approval before applying.
4. **Pre-existing substantive file** (`git show HEAD:{path}` has real content): do **not** rewrite — record gap-analysis items under **Existing documentation review → Proposed changes** only.

**Rules when filling stubs:**

- **Strip unfilled sections** — omit blocks you cannot ground. Never leave `{placeholder}` or `{{placeholder}}` tokens.
- **Draft Language** — provisional terms you establish from repo evidence (CONTEXT files).
- **To Complete** — include in `CONTEXT.md`, `CONTEXT-MAP.md` when present, and `docs/agent-commands.md`. Each item must remind future agents to offer LLM-assisted follow-up. Do not invent definitions or commands silently.
- **Wiki index** (`docs/wiki/index.md`): include only real link rows backed by evidence; omit empty sections entirely.
- **`docs/agent-commands.md` shape:**
  - One backtick line per standard gate (`## Format`, `## Build / Typecheck`, `## Lint`, `## Test`, `## Docs Checks`) — either the command (e.g. `` `pnpm test` ``) or `` `Not configured yet.` `` when unknown. **No duplicate "If not configured" prose blocks.**
  - **Tiered inference:** when the stack is evident, infer plausible commands for Format, Build/Typecheck, Lint, and Test (respect package manager). **Never infer Docs Checks or Runtime-Restricted Checks** from tooling alone.
  - **Inferred commands** get a **To Complete** confirmation item (e.g. "Confirm test command — inferred `pnpm test` from Vitest").
  - **Runtime-Restricted Checks:** keep heading and intro prose always. Bullet list only for documented commands. When none are known, a single `None known.` line.
  - **Docs Checks:** `Not configured yet.` unless an exact docs command is documented.
- Align terminology with installed skills and rules — do not rewrite skill or rule bodies in this step.

### Catalog tailoring

Rewrite **bundled** skills and rules substantively for this repository. Edit only paths the bundle installed — omit paths that need no change.

**Editable paths:**

- `.agents/skills/*/SKILL.md` for skills included in the bundle (not upstream-installed skills)
- `.agents/rules/*.md` for scoped catalog rules
- `.cursor/rules/*.mdc` for Cursor catalog rules
- Inline `### {rule-key}` sections in `AGENTS.md` for always-apply rules

**When editing:**

- Add repo-specific examples — prioritize **Repo discovery** findings and on-disk evidence, then infer plausible conventions for the stated stack (language, monorepo, package manager, test runner)
- Align terminology with filled seed docs and repo facts
- Substitute commands and tools when evidenced (e.g. `npm test` → `pnpm test`)
- Merge overlapping guidance across selected skills and rules; each file stays a separate path
- Resolve terminology or workflow conflicts between selected items
- When consolidating overlap into one rule or skill, patch the absorbed item with a brief pointer to the canonical path (stub patch)
- Add cross-references between selected skills and rules where useful

**Preserve:**

- Workflow structure and step order in each file
- Frontmatter and skill or rule identity
- Separate files — do not merge skills or rules into one path
- Every selected item remains in the bundle — stub pointers are allowed

**Do not:**

- Edit upstream-installed skills or paths outside the bundle
- Invent specific paths, services, team names, or commands with no grounding in repo evidence or obvious stack conventions — use generic placeholders such as `<package-name>/` when specifics are unknown
- Copy-paste repo overview blocks into every skill/rule — overview belongs in `CONTEXT.md`
- Add a generic `## Repo Context` or project overview section to each file
- Remove required frontmatter or skill or rule identity

**Compiled rules layout (this target):**

- The host file (`AGENTS.md`) has a **Rules index** table, preamble, then **inline** `### {catalogKey}` sections only for always-apply rules.
- Scoped rules live in `.agents/rules/{catalogKey}.md` (body only — no YAML frontmatter). Never place scoped rule bodies in the host file.
- Do not use HTML comments in rule bodies.



### Definition of done wiring

The bundle already includes a tailored `definition-of-done` body with completion gates matching `.agentic-config/manifest.json` `selection.rules` and `selection.skills`. Do not re-add stripped gates during catalog tailoring.

When `definition-of-done` is in `.agentic-config/manifest.json` `selection.rules`:

1. Ensure `AGENTS.md` **Rules index** lists `definition-of-done` as `always` / inline.
2. Merge or update inline `### definition-of-done` from the bundled body.
3. Tailor project-specific bullets in that section when useful.

When `definition-of-done` is **not** bundled but `role`, `changelog`, or `wiki-consultation` is, list installing `definition-of-done` under **Proposed changes**. Do not edit pre-existing host rule sections unless the user is already changing those files.

### Restricted-operations rule tailoring

When `restricted-operations` is in `.agentic-config/manifest.json` `selection.rules`:

1. Open the `restricted-operations` rule — the inline `### restricted-operations` section in `AGENTS.md`.
2. Locate the **Repo-Specific Notes** section at the bottom of that section.
3. Populate it with the target repository's actual infrastructure tooling:
   - **Specific infrastructure CLIs** (e.g. `kubectl`, `helm`, `terraform`, `aws`, `gcloud`, `az`) and their write/subcommands found in the repo (from Dockerfiles, CI configs, README, infra/ directory, or `git log` patterns)
   - **Docker compose commands** used for local development (e.g. `docker compose up -d`, specific service names)
   - **Deployment/publish commands** used by the project (CI scripts, make targets, npm scripts)
   - **Credential paths** specific to this repo (e.g. `infra/secrets/`, environment files referenced in docs)
   - **Any package-manager or tool-specific mutations** that could be destructive (e.g. database migrations, seed scripts)
4. For **integration** variants: check the prior version of `AGENTS.md` via `git show HEAD:AGENTS.md`, extract the prior `### restricted-operations` section, and merge or extend the existing repo-specific notes rather than replacing them.
5. Ground everything in repo evidence — `docker-compose.yml`, CI configs, `infra/`, `deploy/`, `.github/workflows/`, Makefile, package.json scripts, README. Do not invent tooling the repository does not use.

### Ponytail catalog tailoring

When `ponytail` is in `.agentic-config/manifest.json` `selection.skills` or `selection.rules`:

Harmonize Ponytail (lazy minimalism / YAGNI ladder) with other selected catalog items. Edit **bundled** Ponytail rule paths only — do not edit upstream-installed Ponytail slash skills under `.agents/skills/ponytail-*`.

**Known conflict pairs:**

1. **vs `strict-output-execution`:** Ponytail allows brief post-code explanation when it aids clarity; strict-output forbids preamble and filler. In the bundled Ponytail rule (inline `### ponytail` in `AGENTS.md` or `.cursor/rules/ponytail.mdc`), state precedence: default to strict artifact-only output when both apply; allow one short trailing line only when the user explicitly asked for explanation.
2. **vs `workflow-gates` / `testing`:** Ponytail minimizes code but does **not** skip validation gates or meaningful tests. Keep workflow-gates and testing rules authoritative; trim redundant “run tests” prose from Ponytail only where it duplicates gate wording.
3. **vs `functional-programming`:** Deduplicate overlapping style guidance (immutability, small functions, avoid mutation). Keep one canonical home — usually the more specific rule — and stub-pointer the other.
4. **Pi implicit install (rule and/or skill):** On Pi targets, `install.sh` runs implicit `pi install` for Ponytail when the catalog skill or rule is selected. Audit overlap among the bundled rule, Pi extension injection, and upstream slash skills. Consolidate always-on ladder text in the bundled rule; use stub pointers in extension overlap rather than triplicating the full ladder.

Ground edits in repo evidence. Do not remove Ponytail’s non-negotiables (trust boundaries, security, accessibility, required checks).

### Save content hashes (mandatory after editing)

After completing all tailoring and merge work above, save SHA256 content hashes of every edited file back to `.agentic-config/manifest.json`.

1. For each file you edited (host file, `.agents/rules/*.md`, `.cursor/rules/*.mdc`, `.agents/skills/*/SKILL.md`), compute its SHA256 hash.
2. Format each hash as `sha256:<hex>` — for example, `"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"`.
3. Read the current `.agentic-config/manifest.json` from disk. Parse the JSON.
4. Update the `contentHashes` field **in place** — merge, do not replace:
   - If `contentHashes` already exists, **keep all existing entries** and add/update only the files you just edited.
   - If `contentHashes` does not exist yet, create it with the hashes of the files you edited.
   - **Do not remove** entries for files you did not edit in this session.
     Example:
   ```json
   {
     ...existing fields,
     "contentHashes": {
       ...existing contentHashes,
       "AGENTS.md": "sha256:abc123...",
       ".agents/rules/workflow-gates.md": "sha256:def456..."
     }
   }
   ```
5. Write the updated manifest back to disk, preserving all other fields.
6. If the write fails for any reason (permissions, file locked, etc.), log a warning and **continue** — do not block the setup completion.

### Skill body resolution (pi only)

Some skills in the bundle may reference other skills by name (for example a body that says "run a `/grilling` session, using the `/domain-modeling` skill"). Pi does **not** resolve these references — the body is just instructions for the agent, and a bare `/skillName` reference gives the agent nothing to do.

Scan every `*.SKILL.md` in `.pi/skills/` and `.agents/skills/` installed by this bundle:

1. Read the body (everything after the front-matter `---` block).
2. If the body contains a bare reference like `/grilling` or `/domain-modeling` — or any line that says "run a `/X` session" or "use the `/Y` skill" — **resolve it**:
   - Read the referenced skill's body.
   - Replace the referencing skill's body with the referenced skill's actual content (strip the referenced skill's own front-matter, keep the instruction text).
   - If the referencing skill has its own useful content beyond the reference, merge: keep the original body, append the referenced skill's body, and mark the source with `<!-- sourced from: /skillName -->`.
3. If the referenced skill is **not** installed alongside the referencing one, add a comment in the body: `<!-- TODO: /skillName not installed — resolve this reference -->`.
4. Do **not** ask the human about this — it's mechanical and the agent is the reader. Include findings in the **Setup completion report**.

This only applies to skills whose bodies rely on pi resolving `/skillName` references. Skills with substantive bodies (interview questions, domain modeling rules, etc.) leave alone.



### Initial wiki

Skip this section when `docs/wiki/schema.md` is absent.

1. Read `docs/wiki/schema.md`, `docs/wiki/page-template.md`, `docs/wiki/index.md`, and `docs/wiki/log.md`.
2. Decide whether **substantive wiki pages** already exist — any `.md` under `docs/wiki/` besides `index.md`, `log.md`, `schema.md`, and empty placeholders.
3. When **no substantive pages** exist (or the wiki is still sparse):
   - Create **one or more wiki pages** from live repo sources — at least one page so the wiki is usable; add **additional pages** when you have enough **Verified Facts** (READMEs, code layout, `CONTEXT.md`, ADRs, and when Graphify is enabled `graphify-out/GRAPH_REPORT.md`) for another topic. Do not invent content to pad page count.
   - Do **not** hand-write graph topology as markdown pages when `GRAPH_REPORT.md` exists — link to or cite graph clusters instead.
   - Pick sensible paths under `docs/wiki/` per `docs/wiki/schema.md` — getting the wiki running matters more than a prescribed filename.
   - Follow frontmatter and section shape in `page-template.md`. Prefer **Verified Facts** with source paths; mark interpretation under **Agent Synthesis**.
   - Update `docs/wiki/index.md` with links under the right headings.
   - Add entries to `docs/wiki/log.md` for each page created.
4. When substantive pages **already exist**: do not overwrite them. **Ask** whether to add new pages for topics you found, or list them under **Initial wiki → Proposed changes** in the report.

### Wiki automation

Skip when `docs/wiki/schema.md` or `docs/wiki/path-map.json` is absent.

1. Read `docs/wiki/path-map.json`, `scripts/wiki-lint.mjs` (reference implementation), `docs/wiki/schema.md`, and `docs/wiki/log.md`.
2. **Classify the repository** — language(s), layout (monorepo vs single package), and existing validation or git-hook tooling. Do not assume Node, `package.json`, Husky, or any specific stack.
3. **Populate `docs/wiki/path-map.json`** from discovered layout and wiki pages:
   - Add one mapping per top-level package or subsystem that has (or will have) a matching `docs/wiki/subsystems/<name>.md` page.
   - Use `allowSkip: true` for subsystem mappings; keep bundled `allowSkip: false` entries for `docs/adr/**` and `CONTEXT.md`.
   - Do not invent mappings for paths with no wiki page yet — list those under **Proposed changes** instead.
4. **Adopt or port the lint script:**
   - When the repo already uses Node.js, keep `scripts/wiki-lint.mjs` as the mechanical wiki linter.
   - When the repo does not use Node.js, offer a port (Python, shell, Make target, etc.) that enforces the same checks and reads the same `docs/wiki/path-map.json` contract. Use `scripts/wiki-lint.mjs` as the reference implementation.
5. **Propose hook wiring** under **Proposed changes** — recommend how to run mechanical wiki lint before commits using tooling that already exists in the repo (git hooks, task runners, CI, etc.). Apply wiring only when repo evidence is clear; otherwise list options and ask.
6. Append `docs/wiki/log.md` with `## [YYYY-MM-DD] update | Wiki automation setup` summarizing path-map changes, script location, and proposed or applied wiring.



### Workflow validation

1. Read `CONTEXT.md`, `docs/adr/`, `docs/wiki/schema.md` when present, installed skills and rules, and `docs/agent-commands.md`.
2. Run the safe commands listed in `docs/agent-commands.md` in gate order (format → build/typecheck → lint → test).
3. Note which commands work, which are missing, which need a human, and inconsistencies between docs.

## Setup completion report (mandatory — last message)

Use this structure. Omit subsections only when not applicable. Omit **Initial graphify** and **Graphify automation** when Graphify was not installed. Omit **Initial wiki** and **Wiki automation** when wiki was not installed.

After the report body, **end your message with one closing question** — ask whether the human is satisfied with the setup so far or wants to adjust anything (proposed changes, tailoring, wiki, commands, or follow-ups). Include your **recommended default** when useful (for example: proceed with listed proposed changes). One question only.

```markdown
# Setup completion report

## Duplicate content audit

- Duplicates removed: …
- Cross-references added: …
- Needs human decision: …

## Seed docs fill

- Paths filled: …
- Paths skipped (pre-existing): …

#### Proposed changes

- …

## Catalog tailoring

- Skills/rules tailored: …
- Notes: …

## Initial graphify

- First `graphify .` run: …
- `GRAPH_REPORT.md` / `graph.json` verified: …

#### Proposed changes

- …

## Graphify automation

- MCP registration verified: …
- Hook/CI re-index proposed or applied: …

#### Proposed changes

- …

## Initial wiki

- Pages created: …
- Index/log updated: …

#### Proposed changes

- …

## Wiki automation

- Path map populated: …
- Lint script adopted or ported: …
- Hook wiring proposed or applied: …

#### Proposed changes

- …

## Workflow validation

### Commands that work

- …

### Missing or not configured

- …

### Human-only / runtime-restricted

- …

### Documentation inconsistencies

- …

## Remaining follow-ups

- …
```

When the human replies to the closing question, apply approved **Proposed changes** and continue the setup interview until open decisions are resolved or explicitly deferred to **Remaining follow-ups**.