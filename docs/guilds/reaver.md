# Reaver

Heavy use of **reaver threaten** combined with skills and words, loud highlights on combat lines, and a prayer gate for some big spells.

## Line highlights

Many attack, miss, killing blow, blight, and gear-break lines are colored (blue / green / magenta / red) so fights are easier to read. Threaten-related and “ancient word” lines get special treatment.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `uss` | Scythe swipe | `reaver threaten` + `use` when args present; otherwise `use` only. |
| `urc` | Rampant cutting | Same threaten compound rule. |
| `urs` | Reaver strike | Same. |
| `ubh` | Blood harvest | Same. |
| `utr` | True reaving | Same. |
| `ucc`, `uccut` | Corrosive cut | Same (`uccut` is an alias of `ucc`). |
| `ubd` | Breath of doom | Same. |
| `res` | Reave shield | Plain `use` (no threaten compound). |
| `rew` | Reave weapon | Plain `use`. |
| `rea` | Reave armour | Plain `use`. |
| `upd` | Prayer to destruction | **`use 'prayer to destruction' at <target>`** — target required. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cws` | Word of spite | Threaten + `cast` when args present. |
| `cb`, `cwb` | Word of blasting | Duplicate shortcuts for the same spell. |
| `cwd` | Word of destruction | Threaten + `cast` when args present. |
| `cwa` | Word of attrition | **`cast 'word of attrition' at <target>`** only (no threaten); args required. |
| `cwsl` | Word of slaughter | Threaten + `cast` when args present. |
| `cwg` | Word of genocide | Threaten + `cast` when args present. |
| `csf` | Shattered feast | If prayer flag set: `cast 'shattered feast' at amount 100`. Else queues prayer at spell first. |
| `cbh` | Black hole | If prayer done: `cast 'black hole'`. Else prayer gate. |
| `cbs` | Blood seeker | If prayer done: `cast 'blood seeker' at amount 100`. Else prayer gate. |
| `crb` | Reaping of bile | Sends `cast 'reaping of bile'`. |
| `cca` | Call armour | Needs amount arg; may stage through `use 'prayer to destruction' at spell` first. |
| `csd` | Spirit drain | Needs target; after prayer, `cast 'spirit drain' at <target> amount 100`. |

### Other

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `rt` | Reaver threaten | Sends `reaver threaten <target>` when args present. |

Exact targeting follows BatMUD usage; several spells wait on **`prayer_done`** automation before the real cast.

## Profile

Enable **`reaver`** in **`/guilds`**.
