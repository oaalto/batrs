# 02 — Fix selection/persistence bugs discovered during extraction

## Parent

`prd.md`

## What to build

If ticket 01 surfaces genuine bugs while wiring browse (edge-case selection clearing, `GuildSelection` output, TOML persistence), fix them with reproducing tests in separate `fix:` or `migrate:` commits within the same pull request. If no bugs are found, mark this ticket skipped — do not invent work.

## Blocked by

01 — Extract Guild Catalog browse and delegate Guild Dialog

## Status

ready-for-agent

## Acceptance criteria

- [ ] Each fix is preceded or accompanied by a failing test that demonstrates the bug
- [ ] Fixes land in commits separate from the ticket 01 refactor commit
- [ ] Banner wording-only tweaks, if any, are intentional and covered by browse or dialog tests as appropriate
- [ ] TOML migration included only when a genuine persistence bug requires it
- [ ] Ticket marked skipped with a one-line note if no bugs were found during 01
- [ ] `cargo test` passes
