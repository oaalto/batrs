---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

What exactly is one **incoming damage event** row?

Pin down boundaries for v1:

- Trigger: only negative `H:` diff on short-score lines, or also `Hp:` status lines?
- **Out-of-round** HP loss (e.g. `Holy man's bash sends you sprawling.` between round headers and the next `H:` line) — same event type as round melee?
- Fields required on every row (minimum viable metadata): timestamp, player, hp_delta, hp_before/after, damage category, source name, message verb/text, weight, combat/session id, round number?
- Context storage: full text of lines between the previous and current status line, or only the attributed line(s)?
- Lines that change HP but are not hits (`bash sends you sprawling`, `pushes you`) — category labels?
- Confirm: misses, dodges, parries, tumble messages never create events even at weight 0.

## Progress

### Q1 — trigger (resolved)

**Decision:** one event candidate per `H:` line with a negative HP bracket (`[-N]`). `Hp:` prompt lines do not trigger rows (stats may still update elsewhere).

### Q2 — out-of-round vs in-round (resolved)

**Decision:** one unified row shape for all incoming HP loss. No round number or round-scoped metadata — only per-hit/spell/skill damage amounts matter; timing relative to round headers is irrelevant.

### Q3 — minimum fields (resolved)

**Decision:** every row carries `recorded_at`, `player`, `hp_delta`, `hp_before`, `hp_after`, `damage_category`, `source_name`, `message_verb`, `message_text`, `weight`. No round number, fight id, or combat session id in v1. `session_id` (batrs login) deferred — add later if evening-level filters are needed.

### Q4 — context storage (resolved)

**Decision:** store only the attributed line(s). `message_text` holds the full line; no buffer blob, no neighbor window. Smallest DB footprint for v1.

### Q5 — category labels (resolved)

**Decision:** three top-level `damage_category` values for v1:

| `damage_category` | Attribution pattern | `message_verb` |
|-------------------|---------------------|----------------|
| `melee` | `<name> <catalog verb> you.` — verbs from [`docs/hit_messages.md`](../../hit_messages.md) | catalog verb (`bitchslaps`, `lightly strikes`, …) |
| `skill` | guild/enemy skill output lines | skill name (`bash`, `push`, `kick`, …) |
| `spell` | `A/An <spell name> hits you.` (e.g. `A magic missile hits you.`) | spell name (`magic missile`, `chill touch`, …) |

Skill examples locked for v1:

- **bash:** `Holy man's bash sends you sprawling.` → `message_verb = bash`
- **push:** `Holy man pushes you.` → `message_verb = push`
- **kick:** multiple output lines (`kicks you in the groin very hard`, `performs a quick kick to your stomach`, partial-deflect variant, …) → `message_verb = kick`

**Extensibility:** new damage sources are added by extending the matcher catalog (new skill patterns, spell names, melee verbs) — not by changing the row shape. `damage_category` + `message_verb` is the extension point.

**v1 scope limit:** patterns not yet catalogued are not attributed. Environmental, bleed, ambient, and other unknown shapes are out of scope until documented — no guessing.

### Q6 — non-damage lines and unattributed HP loss (resolved)

**A — lines that never create rows:** misses, dodges, parries, tumble messages, and outgoing player lines never create event rows. Only a negative `H:` HP bracket triggers a row; these lines are attribution candidates at most.

**Footnote — enemy parry vs riposte:** enemy `<name> parries.` alone is never a damage candidate. When the immediately following line is `...AND counterattacks.` or `...AND ripostes.`, the follow-up is a `skill` / `riposte` candidate (`source_name` from the parry line). Player `You parry.` / `...AND riposte.` are outgoing counter-damage and excluded.

**Exception — partial kick deflect:** `Salvatore's kick lashes at you with speed, but you manage to partly deflect it in time.` is a kick skill output line — valid attribution candidate with `damage_category = skill`, `message_verb = kick`.

**B — unattributed HP loss:** skip `damage_events` rows. If `H:` shows `[-N]` but no recognized melee/skill/spell line exists in the buffer, **no attribution row is written**. No `unknown` category rows in v1.

> **Addendum (2026-08-07):** [How should unattributed HP loss be captured?](how-should-unattributed-hp-loss-be-captured.md) — zero-candidate triggers now persist a parallel `unattributed_hp_events` row with the full **context window** for review. Attribution skip unchanged; see slice `04-unattributed-hp-review.md`.

### Q7 — co-occurring stat changes (resolved)

**Decision:** each row records **only `hp_delta` from the HP bracket**. SP, EP, exp, and gold changes on the same `H:` line are ignored. No row when the HP bracket is empty or positive (healing).

## Resolution

One **incoming damage event** row is written when all of the following hold:

1. An `H:` short-score line arrives with a negative HP bracket (`[-N]`).
2. The buffer since the previous `H:` contains at least one recognized attribution line (melee catalog, skill pattern, or spell pattern).
3. That line is attributed to the row; `hp_delta` is the positive magnitude of `N`.

**Row fields (v1):** `recorded_at`, `player`, `hp_delta`, `hp_before`, `hp_after`, `damage_category` (`melee` | `skill` | `spell`), `source_name`, `message_verb`, `message_text`, `weight`.

**Not stored on attributed rows:** round number, fight/session id, full context buffer, non-HP stat deltas, `unknown` category in `damage_events`.

**Unattributed HP loss:** no `damage_events` row; context window stored in `unattributed_hp_events` (see addendum on Q6-B).

**Categories:** melee via [`docs/hit_messages.md`](../../hit_messages.md); skills (bash, push, kick, extensible catalog); spells via `A/An <name> hits you.`

**Extensibility:** new matchers plug into the catalog; row shape stays fixed.
