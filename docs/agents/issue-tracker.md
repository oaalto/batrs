# Issue tracker: GitHub Issues

Planning artifacts for this repo live in GitHub Issues. `/to-spec` creates or updates a parent planning issue. `/to-tickets` creates linked child slice issues. Repo-local planning files under `docs/prds/` and `docs/issues/` are no longer the canonical tracker.

## Conventions

- **Create a spec**: `/to-spec` creates or updates one parent GitHub issue for the feature.
- **Read a spec**: read the parent GitHub issue; treat issue content as **historical for behavior claims** until verified against code, tests, and `CONTEXT.md`.
- **Split work**: `/to-tickets` creates one child GitHub issue per implementation slice and links each slice back to the parent issue.
- **Status**: keep canonical planning status in GitHub issue state, labels, and issue links rather than duplicating it in repo-local files.
- **Dependencies**: represent slice ordering in a `Blocked by` section using GitHub issue references.
- **Labels**: parent planning issues and child slice issues should use distinct labels so agents can query them reliably.
- **Existing feature docs**: `docs/features/` remains historical planning/archive material in this repo.

## Migration from repo-local planning files

- Existing unimplemented items under `docs/prds/` and `docs/issues/` should be migrated to GitHub Issues.
- After migration, delete the local source files so GitHub remains the single canonical tracker.
- Local planning files that are already implemented or obsolete should be deleted instead of migrated.
- Do not keep repo-local planning files and GitHub Issues in parallel as active tracker state.

## When a skill says "publish to the issue tracker"

Create or update the corresponding GitHub issue, not a repo-local Markdown file.

## When a skill says "fetch the relevant ticket"

Read the parent GitHub issue and any linked child slice issues. If repo-local planning docs still exist during migration, treat them as historical source material only.

## Tracker operations

- Use GitHub issue links or issue numbers for parent/child references.
- Put the parent issue reference in every slice issue body.
- Link child slices from the parent issue body or checklist.
- Do not silently fall back to `docs/prds/` or `docs/issues/` if GitHub issue creation fails; fail clearly instead.

## Related configuration

See `docs/agents/domain.md` for `CONTEXT.md` and ADR layout. See `.agentic-config/USAGE.md` for slash commands.
