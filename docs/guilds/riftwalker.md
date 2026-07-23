# Riftwalker

Entity and elemental coloring, syncing labels with score output, and many gem / summon / pulse shortcuts. Uses your **fire, air, water, earth** names from settings.

## Entity status (battle listen)

When the MUD sends **battle-listen** entity status lines (same idea as TinyFugue `riftwalker.tf`), batrs updates a **secondary status line** after the main prompt stats when Riftwalker is in your guild selection:

- `--=  <text>  HP:<n>(…` — **gags** the line whenever this pattern matches (whitespace between tokens does not need to match TinyFugue exactly). If automation **has_entity** is set, stores HP and echoes **low-HP notices** at the same thresholds as TF: under 250 / 200 / 150 / 100. Combined lines that end with `  =--` are still recognized.
- `--=  <text>  =--` — stores the **label** fragment for the status bar.

Processing of **HP** lines does not require **has_entity** for gagging (so listen spam is hidden even if the flag is out of sync). **Label** lines and stat/notice updates still require **has_entity**. Entity death lines clear the stored status.

You must enable battle listen on the MUD for these lines to appear (e.g. `battle listen all 1`).

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
| `csum` | Summon rift entity | `cast 'summon rift entity' <args>`; parses element hints from free text and sets skill/element vars. |
| `cdis` | Dismiss rift entity | Queues **has_entity** clear, then `cast 'dismiss rift entity'`. |
| `cb` | Beckon rift entity | `cast 'beckon rift entity'`. |
| `ctrl` | Establish entity control | No args: `cast 'establish entity control'`. With args: `… <target>`. |
| `ctrll` | Establish entity control (long) | `cast 'establish entity control' 10`. |
| `cer` | Regenerate rift entity | `cast 'regenerate rift entity'`. |
| `cte` | Transform rift entity | Expects target/direction text; optional element inferred from remainder. |
| `cs` | Spark birth opener | Optional target: targeting chain, `cast 'spark birth'`, then `gem cmd use` current skill. |
| `css` | Rift pulse opener | Same pattern with **`rift pulse`**. |
| `csd` | Dimensional leech opener | Same pattern with **`dimensional leech`**. |
| `csb` | Spark birth (cast only) | `cast_spell("spark birth", …)`. |
| `crp` | Rift pulse (cast only) | Standard `cast_spell`. |
| `cdl` | Dimensional leech (cast only) | Standard `cast_spell`. |
| `cfa` | Force absorption | No args: `cast 'force absorption' entity`. With args: normal targeted cast. |
| `cmie` | Mirror image (entity) | `cast 'mirror image' entity`. |
| `cam` | Absorbing meld | `cast 'Absorbing meld'`. |
| `ciw` | Iron will | No args: `cast 'iron will' entity`. With args: targeted `cast_spell`. |

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
