# Riftwalker

Entity and elemental coloring, syncing labels with score output, and many gem / summon / pulse shortcuts. Uses your **fire, air, water, earth** names from settings.

## Line highlights

Highlights entity hits, stuns, misses, auras, and similar lines; clears “entity lost” style messages so the client drops stale entity state. Colors follow your element theme in game text.

## Shortcuts

### Skills (entity / gem)

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `ufire` | Blazing sunder | Sets entity skill var to **blazing sunder**, element **fire**, marks entity present (no line sent by itself — automation applies on next send). |
| `uair` | Suffocating embrace | Air skill track (**suffocating embrace**). |
| `uearth` | Earthen cover | Earth track (**earthen cover**). |
| `uwater` | Subjugating backwash | Water track (**subjugating backwash**). |
| `ccs` | Current entity skill | Optional target: `target`/`gem cmd target` chain, then `gem cmd use '{riftwalker_skill}'` (with optional target suffix). |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `csum` | Summon rift entity | `cast 'summon rift entity' at <args>`; parses element hints from free text and sets skill/element vars. |
| `cdis` | Dismiss rift entity | Queues **has_entity** clear, then `cast 'dismiss rift entity'`. |
| `cb` | Beckon rift entity | `cast 'beckon rift entity'`. |
| `ctrl` | Establish entity control | No args: `cast 'establish entity control'`. With args: `… at <target>`. |
| `ctrll` | Establish entity control (long) | `cast 'establish entity control' at 10`. |
| `cer` | Regenerate rift entity | `cast 'regenerate rift entity'`. |
| `cte` | Transform rift entity | Expects target/direction text; optional element inferred from remainder. |
| `cs` | Spark birth opener | Optional target: targeting chain, `cast 'spark birth'`, then `gem cmd use` current skill. |
| `css` | Rift pulse opener | Same pattern with **`rift pulse`**. |
| `csd` | Dimensional leech opener | Same pattern with **`dimensional leech`**. |
| `csb` | Spark birth (cast only) | `cast_spell("spark birth", …)`. |
| `crp` | Rift pulse (cast only) | Standard `cast_spell`. |
| `cdl` | Dimensional leech (cast only) | Standard `cast_spell`. |
| `cfa` | Force absorption | No args: `cast 'force absorption' at entity`. With args: normal targeted cast. |
| `cmie` | Mirror image (entity) | `cast 'mirror image' at entity`. |
| `cam` | Absorbing meld | `cast 'Absorbing meld'`. |
| `ciw` | Iron will | No args: `cast 'iron will' at entity`. With args: targeted `cast_spell`. |

### Other

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `estat` | Gem entities | No args: `gem entities {riftwalker_element}`. With args: `gem entities <args>`. |
| `zz` | Stop entity use | Client chain: `zz`; `gem cmd use stop`. |
| `gwield` | Gem wield | `gem cmd wield <args>`. |
| `rwdiag` | Diagnostics | Prints **has_entity** state in the client only. |
| `rwfix` | Fix note | Client-only message about external scripts. |

!!! note "Shared shortcut"
    **`cs`** overlaps Monk, Ranger, Tzarakk.

## Profile

Set **`riftwalker_entity_fire`**, **`…_air`**, **`…_water`**, **`…_earth`** in settings or **`/guilds`**. Enable **`riftwalker`**.
