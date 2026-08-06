---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

Beyond battle-listen melee (`<monster> lightly strikes you.` and catalog verbs in [`docs/hit_messages.md`](../../hit_messages.md)), what **incoming damage line shapes** must v1 recognize?

From the Holy-man fight log we already see non-catalog incoming lines:

- `Holy man's bash sends you sprawling.`
- `Holy man pushes you.`

Enumerate and classify for v1:

- Enemy spells/skills (`A magic missile hits you.`, breath attacks, guild specials)
- Environmental / ambient damage
- Damage without a standard hit template (bleed, hurt, fall, drowning)
- Player-name vs generic `you` targeting

For each class: example line, proposed `damage_category` label, and whether it belongs in v1 or fog.

### Q1 — v1 skill catalog (resolved)

**Decision:** v1 skills — each its own `message_verb`, multi-regexp where needed:

| `message_verb` | Example lines |
|----------------|---------------|
| `bash` | `Holy man's bash sends you sprawling.` |
| `push` | `Holy man pushes you.` |
| `kick` | `Salvatore kicks you in the groin very hard. You gasp with pain and double up.` |
| | `Salvatore performs a quick kick to your stomach, almost making you lose your breakfast.` |
| | `Salvatore's kick lashes at you with speed, but you manage to partly deflect it in time.` |
| `stab` | `With a quick flick, Akeem knocks your weapon aside and stabs your stomach!` |
| | `You watch helplessly as Akeem smashes your kneecap!` |
| | `OOF!  Akeem feints, throwing you offguard as he PUMMELS your midriff!` |
| `scythe swipe` | `Reaver slashes a ragged wound across your chest.` |

All: `damage_category = skill`. `source_name` = attacker name extracted from line. Other guild/enemy skill lines ignored until matchers added.

### Q2 — spell line shape (resolved)

**Decision:** v1 spells match a single regex: `^An? (.+) hits you\.$`

| Field | Rule |
|-------|------|
| `damage_category` | `spell` |
| `message_verb` | spell name from capture (`magic missile`, `chill touch`, `firebolt`, …) |
| `source_name` | empty on hit line (caster on prior line; not stored in v1) |
| Cast lines | ignored — not damage candidates |

Covers spell hit lines such as `A magic missile hits you.` New spell names auto-discovered via capture group.

### Q3 — patterns explicitly out of v1 (resolved)

**Decision:** v1 matchers = battle-listen melee catalog + five skills + spell regex. Everything else → no attribution candidate → skip row on HP loss.

| Pattern class | Example | v1 |
|---------------|---------|-----|
| Environmental / ambient | room traps, drowning, fall damage | Ignore (fog) |
| DoT / bleed / hurt | periodic ticks without a hit line | Ignore (fog) |
| Player-name targeting | `Orc hits Fueryon.` | Ignore (fog) — battle listen uses `you` |
| Breath natural attacks | `<monster> breath lightly you.` | Melee catalog only (verbs in `hit_messages.md` breath section) |
| Outgoing lines | `You puncture Holy man.` | Never candidates (filtered out) |
| Other guild/enemy skills | unrecognized specials | Ignore until matcher added |

## Resolution

**v1 non-melee matcher catalog** — each entry: `damage_category`, `message_verb`, one or more regexes with `source_name` capture group 1 where applicable.

### Skills (`damage_category = skill`)

| `message_verb` | Regex(es) | `source_name` |
|----------------|-----------|---------------|
| `bash` | `^(.+)'s bash sends you sprawling\.$` | group 1 |
| `push` | `^(.+) pushes you\.$` | group 1 |
| `kick` | `^(.+) kicks you in the groin very hard\. You gasp with pain and double up\.$` | group 1 |
| | `^(.+) performs a quick kick to your stomach, almost making you lose your breakfast\.$` | group 1 |
| | `^(.+)'s kick lashes at you with speed, but you manage to partly deflect it in time\.$` | group 1 |
| `stab` | `^With a quick flick, (.+) knocks your weapon aside and stabs your stomach!$` | group 1 |
| | `^You watch helplessly as (.+) smashes your kneecap!$` | group 1 |
| | `^OOF!\s+(.+) feints, throwing you offguard as he PUMMELS your midriff!$` | group 1 |
| `scythe swipe` | `^(.+) slashes a ragged wound across your chest\.$` | group 1 |

### Spells (`damage_category = spell`)

| `message_verb` | Regex | `source_name` |
|----------------|-------|---------------|
| *(from capture)* | `^An? (.+) hits you\.$` | empty |

Cast lines (`utters the magic words`, `claps his hands`, etc.) are not damage candidates.

### Melee (`damage_category = melee`)

Handled in separate ticket — battle-listen catalog from [`docs/hit_messages.md`](../../hit_messages.md); template `<name> <verb> you.`

### Extensibility

New skills/spells: add regex entry with `message_verb` key; row shape unchanged. Unrecognized lines → no candidate → skip on HP loss.
