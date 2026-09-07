---
name: wiki
description: Query, ingest, update, and lint the engineering wiki in docs/wiki/. Load before narrative or subsystem overview questions, path-map/index consultation, and pre-exploration tasks — not only wiki writes.
---

# Wiki

## Purpose

Maintain the engineering wiki as an agent-owned, human-reviewed memory layer.

Engineering wiki at `docs/wiki/`. See the `documentation` rule for the four-tier ladder: glossary (`CONTEXT.md`) → wiki → user guide → generated reference.

## Operations

Use one of these operations:

- `/wiki-query`: answer a question using the wiki and verified sources.
- `/wiki-ingest`: process a source into the wiki.
- `/wiki-update`: update existing wiki pages after durable knowledge changes.
- `/wiki-lint`: health-check the wiki.

## Pre-task consultation

Apply before reading, searching, or editing task-relevant paths — not only when making file changes.

1. **Tier 1 (always):** Read `docs/wiki/path-map.json` and `docs/wiki/index.md`. List candidate pages from path-map `sources` matching paths you plan to read, search, or edit **or** index entries matching domain terms in the task.
2. **Tier 2 (when candidates exist):** Read up to **3** candidate pages (subsystem → concept → workflow priority).
3. **Tier 3 (before implementing from wiki):** Verify claims against code, tests, or ADRs; treat unverified synthesis as hypothesis.
4. **No match:** Proceed; note no wiki coverage.

### Architecture and exploration questions

When the user asks for a package, slice, or subsystem overview (read-only — not implementation), Tier 1–2 are **required** before broad source-code exploration. See **Narrative overview questions** in `.agents/skills/repo-navigation/SKILL.md` for the full source order. Do not skip because the task has no file edits.

For **structural topology** questions (call chains, cross-slice imports, impact analysis), load the `graphify` skill when shell confirms `graphify-out/graph.json` exists — see **Structural topology questions** in `.agents/skills/repo-navigation/SKILL.md`. Wiki deep-reads do not replace graphify for those tasks.

## Read First

1. `docs/wiki/schema.md`.
2. `docs/wiki/index.md`.
3. `docs/wiki/log.md`.
4. `docs/wiki/path-map.json` when present.
5. `CONTEXT.md`.
6. Relevant ADRs.

## Source Rules

- Treat code, tests, accepted ADRs, `CONTEXT.md`, current runbooks, and current official external docs as live sources.
- Treat PRDs, issue discussions, PR discussions, and chat history as historical sources unless verified against live sources.
- Treat agent synthesis as synthesis, not fact.
- Verify wiki claims against live sources before relying on them for implementation.
- Record URLs and fetch dates for external sources.
- Prefer raw Markdown or plain-text URLs for external fetches.

## Write obligations

- Update wiki pages when subsystem behavior, workflows, traps, or debugging paths change and will matter to a future agent.
- Act by default in the same change; ask the user only when unsure which page owns the change or scope is ambiguous.
- Record a `skip` log entry in `docs/wiki/log.md` only when there is no behavioral delta (formatting, comments, test-only, pure refactor with identical behavior).
- Propose before changing rules, skills, or `docs/wiki/schema.md`.
## Before Commit

Wiki completion gates before commit (path-map check, skip log policy, mechanical lint) are in the `definition-of-done` rule. Use the operations in this skill (`/wiki-update`, `/wiki-ingest`, `/wiki-lint`) to satisfy them.

## `/wiki-query`

1. Read `docs/wiki/index.md`.
2. Read relevant wiki pages.
3. Verify critical claims against live sources.
4. Answer with citations to files, URLs, or wiki pages.
5. If the answer contains durable new synthesis, ask whether to file it back into the wiki.

## `/wiki-ingest`

1. Identify source type: live, historical, or external.
2. Read the source.
3. Extract durable facts, historical context, open questions, and synthesis.
4. Update or create relevant pages.
5. Update `docs/wiki/index.md`.
6. Append to `docs/wiki/log.md`.
7. Propose promotion to `CONTEXT.md` or ADRs when appropriate.

## `/wiki-update`

1. Identify which durable knowledge changed.
2. Find affected wiki pages through the index and `path-map.json`.
3. Update pages with evidence status.
4. Preserve historical context instead of overwriting it silently.
5. Update related links.
6. Update index and log.

## `/wiki-lint`

Run mechanical lint first when `scripts/wiki-lint.mjs` exists, then check semantic drift:

- stale claims contradicted by live sources,
- pages without evidence,
- orphan pages,
- missing inbound links,
- duplicate pages for the same concept,
- ADR/wiki contradictions,
- `CONTEXT.md` terms missing from wiki maps,
- wiki concepts that should be promoted to `CONTEXT.md`,
- PRD claims being used as live truth.

Report findings before making broad edits. Fix clear mechanical issues when safe.

## Output

For queries, answer with citations and verification notes.

For ingests and updates, summarize:

- pages created,
- pages updated,
- sources used,
- open questions,
- promotion candidates.

For lint, list findings by severity and recommend fixes.