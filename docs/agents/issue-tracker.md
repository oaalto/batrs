# Issue tracker: Repo-local Markdown (`docs/features/<feature_name>/`)

Planning artifacts for this repo live as markdown under `docs/features/<feature_name>/`. The PRD for a feature is `prd.md` in that folder; implementation slices live as sibling markdown files in the same folder. Skills that need a "ticket" should prefer these paths unless the human points elsewhere.

## Conventions

- **Create a spec**: `/to-spec` writes `docs/features/<feature_name>/prd.md` using the process PRD template.
- **Read a PRD**: open `docs/features/<feature_name>/prd.md`; treat content as **historical for behavior claims** until verified against code, tests, and `CONTEXT.md`.
- **Split work**: `/to-tickets` produces implementation slices under `docs/features/<feature_name>/<slice-slug>.md`; link each slice back to the feature's `prd.md` in its **Parent** section.
- **Status**: record planning status in the PRD (`draft` / `in review` / `accepted` / `superseded`) in a `## Status` section near the top.
- **One folder per feature**: PRD and slices share the same `docs/features/<feature_name>/` folder, so a feature's plan is discoverable in one place.

## When a skill says "publish to the issue tracker"

Create or update `docs/features/<feature_name>/prd.md` and its slice files (not GitHub/GitLab unless the human explicitly redirects).

## When a skill says "fetch the relevant ticket"

Read `docs/features/<feature_name>/prd.md` for the parent spec and `docs/features/<feature_name>/<slice-slug>.md` for implementation slices. If the human passes an external issue URL, treat it as supplementary historical context only.

## Related configuration

See `docs/agents/domain.md` for `CONTEXT.md` and ADR layout. See `.agentic-config/USAGE.md` for slash commands.
