---
name: review
description: Risk-first code review after mechanical gates have run.
---

# Review

## Purpose

Find bugs, regressions, security/privacy issues, missing tests, operability risks, and missing knowledge updates. For batrs: pay extra attention to telnet/session lifecycle, login gating, guild trigger false positives, and TUI input handling (ratatui/crossterm).

## Assumption

Formatting and mechanical gates should run before review. Review is not a formatter.

## Read First

1. Task intent, issue, PRD, or human request.
2. Changed files.
3. Relevant tests.
4. `CONTEXT.md`.
5. Relevant ADRs.
6. Relevant wiki pages.

## Review Priorities

1. Correctness bugs.
2. Behavioral regressions.
3. Security and privacy issues.
4. Missing or weak tests.
5. Operability and observability risks.
6. Architecture conflicts with ADRs.
7. Vocabulary drift from `CONTEXT.md`.
8. Missing or stale wiki, ADR, PRD, or documentation updates.

## Rules

- Findings first, ordered by severity.
- Every finding must cite changed code or a relevant source.
- Do not focus on style unless it creates correctness or readability risk.
- Do not suggest broad refactors unless directly tied to risk.
- Treat PRDs as historical intent, not proof of behavior.
- If no findings exist, say so and mention residual risk or test gaps.

## Output

Use this format:

```md
## Findings

- {Severity}: {Finding}
  - Evidence: {file/source}
  - Impact: {why it matters}
  - Suggested fix: {concise fix}

## Open Questions

- {Question}

## Residual Risk

- {Risk or test gap}
```
