# Issue tracker: Repo PRDs (`docs/prds/`)

Planning artifacts for this repo live as markdown PRDs under `docs/prds/<feature_name>/`. Implementation slices live under `docs/issues/<feature_name>/`. Skills that need a "ticket" should prefer these paths unless the human points elsewhere.

## Conventions

- **Create a spec**: `/to-spec` writes `docs/prds/<feature_name>/prd.md` using the process PRD template.
- **Read a PRD**: open the file under `docs/prds/<feature_name>/`; treat content as **historical for behavior claims** until verified against code, tests, and `CONTEXT.md`.
- **Split work**: `/to-tickets` produces implementation slices under `docs/issues/<feature_name>/<slice-slug>.md`; link each slice back to the spec in its **Parent** section.
- **Status**: record planning status in the PRD (draft / in review / accepted / superseded) in a `## Status` section near the top.

## When a skill says "publish to the issue tracker"

Create or update a file under `docs/prds/<feature_name>/` (not GitHub/GitLab unless the human explicitly redirects).

## When a skill says "fetch the relevant ticket"

Read the referenced `docs/prds/<feature_name>/prd.md` path. For implementation slices, read `docs/issues/<feature_name>/<slice-slug>.md`. If the human passes an external issue URL, treat it as supplementary historical context only.

## Related configuration

See `docs/agents/domain.md` for `CONTEXT.md` and ADR layout. See `.agentic-config/USAGE.md` for slash commands.
