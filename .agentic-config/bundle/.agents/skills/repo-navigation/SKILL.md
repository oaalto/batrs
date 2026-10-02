---
name: repo-navigation
description: Classify read-only exploration questions (narrative overview vs structural topology) and route to wiki or graphify skills. Load before broad repository exploration on question-only tasks.
---

# Repository navigation

Classify read-only exploration first, then follow the matching track **before** broad source-code exploration.

## Narrative overview questions

When the user asks what a package, slice, or subsystem **is** or **owns** (layout summary, responsibilities, stack — not a file-level call chain):

1. **Wiki (required):** Load `.agents/skills/wiki/SKILL.md` if not already loaded. Read `docs/wiki/path-map.json` and `docs/wiki/index.md`. Match to path-map `sources` or index entries. Read up to **3** candidate pages (subsystem → concept → workflow).
2. **ADRs:** Read cited or task-relevant accepted ADRs when the wiki or question is architectural.
3. **Stop when answered:** Once the current question is answered and verified, stop. Do not widen into nearby subsystems, extra workflows, or nice-to-know background unless the user asks.
4. **Skip graphify** for this track.
5. **Code (targeted):** Open source only to verify wiki claims or fill gaps — not directory sweeps.

## Structural topology questions

When the question needs **cross-file or cross-slice** relationships: call/import chains, caller maps, dependency paths, impact analysis, or which files connect A to B. Common triggers: "call chain", "import path", "what calls", "what connects", "cross-slice", "facade", "shortest path", "who depends on", "what breaks if".

When shell confirms the graph exists (`test -f graphify-out/graph.json`) — `graphify-out/` is often gitignored; Read/Glob alone are not reliable:

1. **Graphify (required):** Load `.agents/skills/graphify/SKILL.md` if not already loaded. Check graph freshness, then run `graphify query`, `graphify path`, or graphify MCP via **shell or MCP** **before** opening implementation files or running broad search for topology. Do not wait for the user to say "use graphify".
2. **Wiki (optional, brief):** At most path-map + index, or one subsystem page for domain terms — do not substitute wiki deep-reads for graphify.
3. **Code (targeted):** Open only files graphify names to verify **EXTRACTED** edges; prefer public entry files and cited hop files — not parallel sweeps across many internal files.
4. **Stop at the current answer:** Once you can answer the current topology question with evidence, stop. Do not keep exploring adjacent paths or broader impact unless the user asked for that wider scope.

If the graph is missing or stale, propose `graphify update .` before deep structural work; fall back to targeted search only then.

## Both tracks

Do not skip consultation because the task is question-only with no file edits. If wiki has no match on the narrative track, say so and proceed with ADRs and code. Lead with the single answer or next missing fact the current question needs, not a sprawling exploration plan.
