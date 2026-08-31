# GitHub Issues are the canonical planning tracker

## Status

Superseded by ADR-0004 (repo-local markdown files are the canonical planning tracker).

## Context

This repo originally treated `docs/prds/` and `docs/issues/` as the canonical planning tracker for `/to-spec` and `/to-tickets`. That kept planning artifacts in Git, but it split planning away from the repository's actual GitHub issue workflow and required agents to maintain parallel local files as active tracker state.

The project now wants `/to-spec` and `/to-tickets` to use GitHub Issues directly. The change also needs a rule for legacy local planning docs: unimplemented items should be migrated, while implemented or obsolete items should be deleted.

## Decision

GitHub Issues are the canonical planning tracker for this repository.

- `/to-spec` creates or updates one parent GitHub planning issue.
- `/to-tickets` creates linked child GitHub slice issues.
- GitHub issue state, labels, and links are the canonical planning status.
- Repo-local planning files under `docs/prds/` and `docs/issues/` are no longer active tracker state.
- Legacy unimplemented local planning artifacts should be migrated to GitHub Issues, then deleted locally.
- Legacy implemented or obsolete local planning artifacts should be deleted instead of migrated.
- Skills must fail clearly if GitHub issue creation is unavailable; they must not silently fall back to repo-local planning files.

## Consequences

### Positive

- Planning lives in the same tracker humans already use.
- Parent and slice work stays visible without opening repo-local Markdown files.
- Agents no longer need to maintain parallel local issue state.
- Status drift between Markdown files and GitHub issue state is reduced.

### Negative

- Planning now depends on GitHub issue access and authentication.
- Historical local planning docs require a migration pass or cleanup.
- Issue structure must be expressed with labels and issue links instead of directory layout.

## Alternatives considered

### Keep `docs/prds/` and `docs/issues/` as the canonical tracker

Rejected because the project explicitly wants GitHub Issues, and keeping local files as canonical state would preserve the current split-brain workflow.

### Keep both GitHub Issues and repo-local planning files active in parallel

Rejected because parallel active trackers invite drift and duplicate maintenance.

### Migrate only new work and leave old unimplemented local issues in place

Rejected because open work would remain split across two trackers, making the transition incomplete.
