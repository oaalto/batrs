---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also creates docs (ADRs and glossary) as the model solidifies.
disable-model-invocation: true
---

Use the installed `grilling` and `domain-modeling` skills directly; Pi does not resolve bare `/skill` references for you.

## Process

1. Load the `grilling` skill and run its frontier-based interview loop to stress-test the plan or design.
2. In parallel with that reasoning discipline, apply the `domain-modeling` skill whenever terms, boundaries, or hard-to-reverse trade-offs become concrete.
3. For this repo, read `CONTEXT.md`, relevant ADRs under `docs/adr/`, and existing feature planning in `docs/features/<feature_name>/` before proposing new glossary terms or decisions.
4. When planning artifacts are needed, keep them repo-local: PRDs live at `docs/features/<feature_name>/prd.md`, and sibling slices live in the same feature folder.
5. Do not assume Pi will dispatch another skill from a bare slash command inside this body; carry out the referenced skill instructions yourself.
