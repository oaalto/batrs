# Engineering Wiki Schema

The wiki is an agent-owned, human-reviewed engineering memory layer.

## Source Of Truth

1. Code and tests define implemented behavior.
2. `CONTEXT.md` defines canonical domain language.
3. ADRs define durable decisions.
4. Current runbooks and command maps define current workflows.
5. Wiki pages compile, explain, cross-reference, and synthesize.
6. PRDs, issue discussions, PR discussions, and chat outcomes are historical unless verified against live sources.

## Page Types

- `concept`: domain or architecture concept explanation.
- `subsystem`: module or subsystem map.
- `workflow`: repeated developer or operational workflow.
- `debugging`: root cause, repro loop, diagnostic path, or bug class.
- `trap`: known pitfall future agents should avoid.
- `source-note`: extracted notes from one source.
- `synthesis`: cross-source analysis or comparison.

## Frontmatter

Every wiki page must start with:

```yaml
---
title: Human-readable title
type: concept | subsystem | workflow | debugging | trap | source-note | synthesis
status: current | historical | draft | needs-verification
updated: YYYY-MM-DD
sources:
  - path-or-url
---
```

## Page Body

Use this structure:

```md
# Title

## Summary

One short paragraph.

## Verified Facts

- Fact backed by live source links.

## Historical Context

- Claim backed by historical source links.

## Agent Synthesis

- Clearly marked interpretation or cross-source synthesis.

## Open Questions

- Unresolved ambiguity or stale claim to verify.

## Related

- [Related Page](../concepts/related-page.md)
```

Sections may be omitted when empty, except `Summary`, at least one of `Verified Facts` or `Agent Synthesis`, and `Related`.

## Links

Use portable Markdown links:

```md
[Display Title](../concepts/example.md)
```

Agent instruction: Do not use Obsidian links unless the repository explicitly adopts them.

## Evidence Rules

- Live claims require live sources.
- Historical claims must be marked as historical.
- Agent synthesis must be marked as synthesis.
- PRDs are historical by default.
- Chat history is not a source unless a human confirms the conclusion should be preserved.
- External sources require URL and fetch date in a source note.

## Update Rules

Agent instruction: Update the wiki when durable knowledge changes.

Agent instruction: Do not update the wiki for trivial edits, formatting-only changes, or speculative chat.

Agent instruction: When a wiki concept becomes stable domain language, propose promotion to `CONTEXT.md`.

Agent instruction: When a wiki explanation describes a durable hard-to-reverse trade-off, propose an ADR.

## Log Format

Every `docs/wiki/log.md` entry uses:

```md
## [YYYY-MM-DD] update | Short title

## [YYYY-MM-DD] ingest | Short title

## [YYYY-MM-DD] skip | Short title
```

Use **skip** when durable-knowledge paths changed but the wiki does not need an update. Include touched paths and a one-line reason.

## Path Map

`docs/wiki/path-map.json` maps source paths to expected wiki pages. Each mapping has `sources`, `wikiPages`, and `allowSkip`. Mappings with `allowSkip: false` (for example `docs/adr/**` and `CONTEXT.md`) require a wiki **update** or **ingest** log entry — not **skip**.

Populate subsystem mappings for the target repository during post-install review or when new packages gain wiki pages.

## Mechanical Lint

When `scripts/wiki-lint.mjs` (or a ported equivalent) is present, run it before commit. It checks frontmatter, required sections, index coverage, internal wiki links, and log entry headers. With `--staged`, it warns when staged files match `path-map.json` without a matching wiki touch or log entry.

Wire the script using whatever git-hook or validation tooling the repository already uses. Do not assume Node, `package.json`, or Husky unless the repo already does.

## Semantic Lint Checks

Agent `/wiki-lint` should also check:

- stale claims contradicted by live sources,
- pages without evidence,
- missing inbound links to major concepts,
- duplicate pages for the same concept,
- ADR/wiki contradictions,
- `CONTEXT.md` terms missing from wiki maps,
- wiki concepts that should be promoted to `CONTEXT.md`,
- old PRD claims being used as live truth.
