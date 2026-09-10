# Borrow automation state across trigger evaluation instead of cloning snapshots

## Parent

`docs/features/codebase-optimization-opportunities/prd.md`

## What to build

Make incoming-line trigger evaluation read the current automation flags and vars without cloning whole automation snapshots for each logged-in line. From the user's perspective, trigger behavior, automation effects, and Player Profile-derived trigger facts stay the same while line processing does less repeated allocation.

## Acceptance criteria

- [ ] Trigger evaluation still sees the correct automation flags, vars, login name, and Player Profile-derived facts for incoming lines.
- [ ] Logged-in line processing no longer clones whole automation flag and var maps merely to construct trigger facts.
- [ ] Existing trigger and automation behavior remains unchanged from the player's perspective.
- [ ] Existing automation or trigger tests, or equivalent behavior checks, cover the preserved semantics.

## Blocked by

- None (can start immediately).

## Status

ready-for-agent
