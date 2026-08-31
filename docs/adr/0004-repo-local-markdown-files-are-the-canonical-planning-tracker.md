# Repo-local markdown files are the canonical planning tracker

## Status

Accepted (supersedes ADR-0003)

## Context

ADR-0003 moved the planning tracker to GitHub Issues. That made `/to-spec` and `/to-tickets` depend on GitHub access and authentication, and split planning work into an external tracker while the repo already keeps durable engineering memory locally. The project wants planning artifacts back in the repository as markdown files.

## Decision

Repo-local markdown files under `docs/features/<feature_name>/` are the canonical planning tracker for this repository.

- `/to-spec` writes the PRD for a feature at `docs/features/<feature_name>/prd.md`.
- `/to-tickets` writes implementation slice files in the same `docs/features/<feature_name>/` folder, linked back to the folder's `prd.md`.
- PRD status is recorded in a `## Status` section (`draft` / `in review` / `accepted` / `superseded`).
- GitHub issues are treated as historical context, not the canonical tracker.
- `docs/features/` therefore holds both historical archive features and the canonical planning folders for current work.

## Consequences

### Positive

- Planning lives in Git alongside the code, with no external authentication dependency.
- PRD and slices for a feature are discoverable in one folder.
- Status is version-controlled in the file instead of split across an external tracker.

### Negative

- `/to-spec` and `/to-tickets` no longer surface work in the GitHub issue UI.
- Feature planning history is not browsable through GitHub's issue views.
