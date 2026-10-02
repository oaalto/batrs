---
description: >-
  Two-axis review of the diff since a fixed point: Standards (does it follow the repo's coding standards?) and Spec
  (does it faithfully implement the originating issue/PRD?), run as parallel sub-agents.
---

Review the changes since a fixed point (commit, branch, tag, or merge-base) along two axes: Standards (does the code follow this repo's documented coding standards?) and Spec (does the code match what the originating issue/spec asked for?). Runs both reviews in parallel sub-agents and reports them side by side. Use when the user wants to review a branch, a PR, work-in-progress changes, or asks to "review since X".

Use `/code-review` as the two-axis review gate at the end of the execution flow. Typical input: a completed `/fix` or `/implement` diff plus its originating issue/spec and a pinned fixed point. Typical output: a split Standards-vs-Spec review report that complements `/review` instead of replacing it.
