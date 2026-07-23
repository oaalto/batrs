---
name: workflow
description: Validation gate order (format, build/typecheck, lint, test). Load before editing or fixing any source file and run gates before changelog or marking work complete.
---

# Workflow gates

Apply before editing repository source files — including typo fixes and small bugfixes — not only before commit.

Apply validation in order and **stop on failure at each gate**. Fix every finding at the current gate before the next gate, changelog, commit, or marking work complete. Commands: `docs/agent-commands.md` when present.

Keep scoped `code-format` and `warning-hygiene` rules for baseline formatting presentation and warning-fix discipline when installed.

## Rust crate gate order (batrs)

When `Cargo.toml` is present (this repo), follow `rust-workflow-gates` and `docs/agent-commands.md`:

| Step | Gate              | Command (batrs)                                              |
| ---- | ----------------- | ------------------------------------------------------------ |
| 1    | Format            | `cargo fmt --all --check`                                    |
| 2    | Lint              | `cargo clippy --all-targets --all-features -- -D warnings` |
| 3    | Tests             | `cargo test --all-targets --all-features`                    |
| 4    | Docs (when needed)| `cargo doc --no-deps` and/or `.venv-docs/bin/mkdocs build --strict` |

Build/typecheck: `cargo build` before or with lint as needed. Stop on first failure.

## Node monorepo strict gate order

When the repository uses Prettier, Knip, Fallow, and ESLint as documented in `docs/agent-commands.md`:

| Step | Gate              | Action                                       |
| ---- | ----------------- | -------------------------------------------- |
| 1    | Format (Prettier) | Format changed files (Prettier, root config) |
| 2    | Knip              | Knip manifest/workspace hygiene              |
| 3    | Fallow            | Fallow strict codebase health                |
| 4    | Build / typecheck | Build / typecheck for affected scope         |
| 5    | ESLint            | Static analysis / lint (`--max-warnings=0`)  |
| 6    | Tests             | Tests for changed scope                      |
| 7    | Graphify          | `graphify update .` when graph stale/missing |

## Generic gate order (toolchain not installed)

1. **Format** — formatting checks pass before continuing.
2. **Build / typecheck** — no compile/type errors.
3. **Static analysis / lint** — blocking unless project policy says otherwise.
4. **Tests** — required scope passes.

## Fix-everything-before-continue (mandatory)

**Absolute requirement:** Resolve every finding at the current gate before doing anything else — including the next gate, changelog, commit, or marking work complete.

- **One gate at a time:** Stop on first failure; re-run that gate until green before moving on.
- **No partial green:** Warnings, knip hints, fallow findings, test failures, and format drift are blocking unless project policy documents a carve-out.
- **No deferral:** Do not leave follow-up fixes, baseline allowlists, or suppressions while a gate is red.
- **No suppressions:** Do not add lint/test/tool suppressions (`eslint-disable`, `@ts-ignore`, knip/fallow ignore entries, baselines, allowlists). Fix the code or config root cause.
- **No bypass:** Do not skip gate order or use `git commit --no-verify`.
- **Pre-commit parity:** When the project uses Husky, fix every `lint-staged`, pre-commit, and wiki-lint finding before commit succeeds.

Do not skip gate order unless project policy documents an exception.

## Format (Prettier)

- **Companion:** `code-format` scoped rule for diff presentation and file-ending hygiene.
- **Canonical formatter:** Prettier with root config when the project uses it.
- **Command map:** `docs/agent-commands.md` — read before code-changing work; do not invent commands.
- Formatting checks are blocking for merge and pre-commit.
- Do not hand-format around Prettier for stylistic preferences.

## Knip

- Run knip from repo root when `docs/agent-commands.md` documents it.
- **Scope:** manifest/workspace hygiene only — unlisted binaries and stale config hints.
- **Tool split:** dependency/export/file/duplication findings may be owned by fallow or another tool — do not re-enable overlapping rules.
- **No suppressions:** fix manifests, scripts, or config; do not add knip ignore entries or similar suppressions.

## Fallow

- Run fallow from repo root when documented in `docs/agent-commands.md`.
- **Merge gate:** exit code 0 required — dead code, duplication, complexity health, and audit gates are blocking.
- **No baseline:** do not add allowlists to bypass findings.
- **No suppressions:** fix findings or remove dead code; do not add fallow suppressions or inline ignores.

## ESLint

- **Companion:** `warning-hygiene` scoped rule for warning-fix discipline.
- **Zero-warning gate:** `--max-warnings=0` when project policy requires it.
- **No suppressions:** do not add `eslint-disable`, `@ts-ignore`, or `@ts-expect-error` to land changes — fix the underlying issue.
- **Carve-outs:** read project ESLint deferred-rules docs before changing ESLint config (not for adding suppressions).

## Graphify (step 7)

When graphify is installed (`command -v graphify` or `graphify-out/graph.json`):

- **Verify:** graph build commit matches `git rev-parse HEAD` when the project documents freshness checks.
- **Stale or missing:** `graphify update .` from repository root before marking work complete.
- **Non-blocking for CI:** local development concern only unless project policy says otherwise.

## Optional docs-only path

For docs-only changes, use a reduced validation path (for example docs build + link checks) instead of the full code gate sequence.
