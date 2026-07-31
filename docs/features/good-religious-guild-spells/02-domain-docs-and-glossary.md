# 02 — Domain docs and glossary

## Parent

`prd.md`

## What to build

Record the Good Religious background-guild pattern in project domain vocabulary so future civilization backgrounds (Evil Religious, etc.) can follow the same seam. A maintainer reading `CONTEXT.md` understands that Good Religious is an auto-injected guild module (not a drill toggle), which spells it owns, how bare vs targeted casts behave, and that background guilds merge before player-selected guilds until a dedicated background→guild map exists.

## Blocked by

- [01 — Good Religious guild spells end-to-end](01-good-religious-guild-spells-end-to-end.md)

## Status

ready-for-agent

## Acceptance criteria

- [ ] `CONTEXT.md` documents Good Religious as an auto-injected background guild: activation via primary theme keyword, not a `/guilds` checkbox
- [ ] `CONTEXT.md` includes the five spell alias table and send-line semantics (`ccs` bare vs targeted, cures default `me`, `ccf` no target / silent arg ignore)
- [ ] `CONTEXT.md` records interim merge-order rule: background guild(s) first, selected guilds after (first registration wins)
- [ ] Optional short player-manual or guild doc note added only if an existing guild-spell doc pattern makes a one-line addition natural; otherwise skip without inventing new pages
- [ ] Wiki log entry recorded per project documentation rules if wiki content changed; `skip` entry if wiki work intentionally omitted
- [ ] No stale references implying `good_religious` is only a gating keyword with no guild module
