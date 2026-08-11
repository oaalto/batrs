# 05 — Riposte skill matcher

## Parent

`prd.md`

## What to build

Extend the incoming-damage matcher with **riposte** — a two-line enemy skill pattern:

1. `<name> parries.` — sets pending state; not a candidate.
2. `..AND counterattacks.` / `..AND ripostes.` (optional leading space, two or three dots) — `skill` / `riposte` when line 1 was the immediately prior non-`H:` line.

Clear pending riposte state on intervening lines, `reset_buffer()`, and `Matcher::reset`. `H:` flush clears melee family recency only — pending parry survives until a follow-up or intervening line clears it.

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
- [x] Collector resets matcher family recency on `H:` flush; full reset on `reset_buffer()`.
