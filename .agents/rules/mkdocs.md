# MkDocs Documentation Workflow

Use this rule when working on repositories that publish docs with MkDocs.

## When Documentation Must Be Updated

- Update the manual when behavior, configuration, CLI/API usage, test commands, or troubleshooting steps change.
- Keep navigation in sync with content changes (for example, update `nav:` in `mkdocs.yml` for new sections).
- Prefer snippet/includes from committed sources (for example, `pymdownx.snippets`) to avoid duplicated docs that drift.

## Build and Validation

- Run this project's docs dependency install step before building: `.venv-docs/bin/pip install -r requirements-docs.txt` (or create the venv first with `python3 -m venv .venv-docs`).
- Run this project's MkDocs build command after docs changes: `.venv-docs/bin/mkdocs build --strict`.
- If a change is docs-only, require docs build validation; do not force unrelated build pipelines unless project policy says so.

## Configuration and Hosting Guardrails

- Treat existing theme/plugin selection as intentional; do not switch themes or add/remove MkDocs plugins without explicit request.
- Preserve existing URL mode unless explicitly changed. If a project depends on static hosting or `file://` browsing, keep `use_directory_urls: false`.
- Preserve and improve existing documentation; avoid removing docs unless replacing outdated content with clearer material.
