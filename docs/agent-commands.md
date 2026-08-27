# Agent Commands

Agent instruction: Read this file before code-changing work.

## Format

`cargo fmt --all`

## Build / Typecheck

`cargo build --all-targets`

## Lint

`cargo clippy --all-targets --all-features -- -D warnings`

## Test

`cargo test --all-targets --all-features`

## Knip config hygiene

`Not configured yet.`

See `knip-hygiene` rule for policy. Run from repo root when knip is configured.

## Strict codebase health (fallow)

`Not configured yet.`

See `fallow-strict` rule for policy. Run from repo root when fallow is configured.

## Wiki Lint

When `scripts/wiki-lint.mjs` (or a ported equivalent) is present, run mechanical wiki lint before commit. Node reference implementation: `node scripts/wiki-lint.mjs --staged`.

## Docs Checks

```bash
python3 -m venv .venv-docs
.venv-docs/bin/pip install -r requirements-docs.txt
.venv-docs/bin/mkdocs build --strict
```

## Runtime-Restricted Checks

Checks requiring credentials, root, Docker, cloud access, paid services, hardware, or local-only infrastructure:

- `cargo run` / live BatMUD session verification
- Manual verification of the local combat damage viewer while the app is connected to BatMUD

## To Complete

Agent instruction: When this section lists items, offer the user LLM-assisted follow-up to resolve them. Do not invent commands silently.

- Confirm whether docs checks should stay as strict MkDocs build, `./scripts/serve-manual.sh --no-livereload`, or both
