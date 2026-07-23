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
3. **Skip graphify** for this track.
4. **Code (targeted):** Open source only to verify wiki claims or fill gaps — not directory sweeps. For batrs: start from `src/app/mod.rs`, `src/command/mod.rs`, and `src/guilds/` before sweeping all guild folders.

## Structural topology questions

When the question needs **cross-file or cross-slice** relationships: call/import chains, caller maps, dependency paths, impact analysis, or which files connect A to B. Common triggers: "call chain", "import path", "what calls", "what connects", "cross-slice", "facade", "shortest path", "who depends on", "what breaks if".

When shell confirms the graph exists (`test -f graphify-out/graph.json`) or `graphify-out/.graphify_semantic_marker` is readable — `graphify-out/` is often gitignored; Read/Glob alone are not reliable:

1. **Graphify (required):** Load `.agents/skills/graphify/SKILL.md` if not already loaded. Check graph freshness, then run `graphify query`, `graphify path`, or graphify MCP via **shell or MCP** **before** opening implementation files or running broad search for topology. Do not wait for the user to say "use graphify".
2. **Wiki (optional, brief):** At most path-map + index, or one subsystem page for domain terms — do not substitute wiki deep-reads for graphify.
3. **Code (targeted):** Open only files graphify names to verify **EXTRACTED** edges; prefer public entry files and cited hop files — not parallel sweeps across many internal files.

If the graph is missing or stale, propose `graphify .` or `graphify update .` before deep structural work; fall back to targeted search only then.

## Both tracks

Do not skip consultation because the task is question-only with no file edits. If wiki has no match on the narrative track, say so and proceed with ADRs and code.
