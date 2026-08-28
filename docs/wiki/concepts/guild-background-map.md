---
title: Guild Background Map
type: concept
status: current
updated: 2026-08-28
sources:
  - CONTEXT.md
  - src/guilds/catalog/mod.rs
  - src/guilds/catalog/selection.rs
  - src/guilds/good_religious/commands.rs
---

# Guild Background Map

## Summary

BatMUD characters have a **background** (civilized, magical, good religious, evil religious, or nomad) that gates which guilds they can join. batrs mirrors this in the Guild Catalog: each thematic background has a **background-only** guild module auto-injected from the primary theme keyword, plus playable guild entries grouped under that theme.

Short names are used in `/guilds` and persisted keys; long names are shown in some UI contexts (for example guild selection).

## Background-only guild modules

When the player's guild primary background keyword matches, `GuildSelection::build_guilds` prepends the matching background-only catalog entry before player-selected guilds. These entries are buildable for command dispatch but excluded from `/guilds` drill toggles and `playable_entries()`.

| Primary keyword | Catalog key | Module | Commands |
|-----------------|-------------|--------|----------|
| `civilized` | `civilized` | `CivilizedGuild` | stub (empty) |
| `magical` | `magical` | `MagicalGuild` | stub (empty) |
| `good_religious` | `good_religious` | `GoodReligiousGuild` | background spell aliases (see below) |
| `evil_religious` | `evil_religious` | `EvilReligiousGuild` | stub (empty) |
| `nomad` | `nomad` | `NomadGuild` | stub (empty) |

### Good Religious background spell aliases

`GoodReligiousGuild` is the only background-only module with real commands today. It is activated solely by the primary theme keyword `good_religious` — it is never a `/guilds` drill toggle. It registers five aliases (`src/guilds/good_religious/commands.rs`):

| Alias | Spell | Send-line semantics |
|-------|-------|--------------------|
| `ccs` | celestial spark | bare: `cast 'celestial spark'`; with `<target>`: targeted cast form |
| `clw` | cure light wounds | bare defaults target to `me` |
| `csw` | cure serious wounds | bare defaults target to `me` |
| `ccw` | cure critical wounds | bare defaults target to `me` |
| `ccf` | create food | always `cast 'create food'`; silently ignores args |

`build_guilds` merges the background guild **before** player-selected guilds, and first registration wins, so Good Religious wins alias conflicts with Mage (`ccf`), Spider (`csw`), Triad (`ccw`), and Riftwalker (`ccs`). Non-Good-Religious characters are unchanged.

## Thematic guild membership

### Good Religious (`good_religious`)

| Short | Long |
|-------|------|
| Animist | The Guild of Animists |
| Druids | The Humble Druids |
| Liberator | The Order of Ghost Liberator Paladins |
| Monk | The Warrior Brotherhood |
| Nun | Sisters of Las |
| Tarmalen | The Followers of Tarmalen |
| Templar | The Templars of Faerwon |

### Evil Religious (`evil_religious`)

| Short | Long |
|-------|------|
| Curate | The Monastic School of Draen-Dalar |
| Nergal | Bearers of the True Rot |
| Reaver | The Cult of Reavers |
| Seminary | The Polytheistic Seminary |
| Triad | Triad of Darkness |
| Tzarakk | Slaves of the Beastmaster |
| Tiger | The Brotherhood of the Black Tiger |
| Spider | The Blades of the Spider Queen |

### Nomad (`nomad`)

| Short | Long |
|-------|------|
| Archers | The Flight of Archers |
| Barbarian | Barbarian Guild |
| Beastmaster | The Herd of Beastmasters |
| Crimson | The Crimson Brigade |
| Ranger | Rangers |

### Civilized (`civilized`)

| Short | Long |
|-------|------|
| Alchemist | Guild of Alchemy |
| Bard | The Bards' guild |
| Civilized Fighters | The Civilized Fighters |
| Civmage | The Fellowship of Wizardry |
| Folklorist | The School of Folklorists |
| Knight | The Legion of Knights |
| Merchant | The Master Merchants |
| Runemages | The Faculte of Runemagi |
| Sabres | The Order of the Shadow Sabres |

### Magical (`magical`)

| Short | Long |
|-------|------|
| Channellers | The Guild of Channellers |
| Inner Circle | Inner Circle of Sorcery |
| Mage | Brotherhood of Sorcery |
| Psionicist | The Psionicists |
| Riftwalker | The Guild of Riftwalkers |

## Multi-background guilds

Guilds joinable from more than one background. Catalog grouping is `Multi`. `/guilds` drill lists only multi guilds eligible for the active thematic background (or primary background on the Multi-Background browse row); see table below.

| Guild | Backgrounds |
|-------|-------------|
| Cavalier | Civilized, Nomad |
| Squire | Civilized, Nomad |
| Disciple | Civilized, Evil Religious, Nomad |
| Kharim | Nomad, Evil Religious |
| Navigators | Good Religious, Evil Religious, Nomad, Civilized |
| Explorer | any background |
| Inf | any background |
| Sailor | any background |
| Treenav | any background |

## Implementation status

- **Playable with commands/triggers:** guilds that existed before this map (for example Animist, Monk, Mage, Disciple, Kharim).
- **Playable stub:** catalog entry builds an empty `Guild` implementation (`src/guilds/stub.rs`) — selectable in `/guilds`, no shortcuts yet.
- **Background-only:** auto-injected from primary keyword; Good Religious has spell shortcuts, others are stubs until background spells are modeled.

Persisted keys use snake_case (for example `civilized_fighters`, `inner_circle`, `tzarakk`). `good_religious` is a buildable-but-not-playable background key; it is not a persisted guild selection.

## Related

- [Guild Catalog](guild-catalog.md)
- `CONTEXT.md` — Guild Catalog section
