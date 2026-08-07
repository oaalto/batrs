# 05 — Riposte skill matcher

## Parent

`prd.md`

## What to build

Extend the incoming-damage matcher with **riposte** — a two-line enemy skill pattern:

1. `<name> parries.` — sets pending state; not a candidate.
2. `...AND counterattacks.` or `...AND ripostes.` — `skill` / `riposte` when line 1 was the immediately prior non-`H:` line.

Clear pending riposte state on intervening lines, `Matcher::reset`, `reset_buffer()`, and every `H:` flush.

## Blocked by

None — matcher-only extension.

## Status

done

## Acceptance criteria

- [x] `Barney parries.` + `...AND counterattacks.` → `skill` / `riposte` / `Barney`.
- [x] `Barney parries.` + `...AND ripostes.` → same aggregation bucket (`message_verb = riposte`).
- [x] Parry line alone does not match.
- [x] Orphan follow-up without preceding parry does not match.
- [x] Intervening recognized line clears pending parry.
- [x] `You parry.` / `...AND riposte.` excluded (outgoing).
- [x] Collector resets matcher on `H:` flush and `reset_buffer()`.
