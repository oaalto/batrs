# Monk

Kata and meditation state, rotating combat skills (disrupt / area / armour / avoid), elemental finishers, and the same **sect cultivation** coloring as Tiger (mantra / cultivation lines).

## Line highlights

- Finishing a **kata** can flip flags and sometimes send **meditation** for you.
- Many skill success/fail lines are colored; skill chains update which “slot” you are on.
- Shared **sect** green/yellow/red for cultivation and mantra messages.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cs` | Kiai-cry | Target required: `target …;use kiai-cry at …`. Resets rotation vars. |
| `ujl` | Joint lock | Resets rotation. Standard `use` targeting. |
| `upw` | Pattern weave | Resets rotation. |
| `usk` | Skulking | Resets rotation. Bare `use 'skulking'`. |
| `ip` | Iron palm | Resets rotation. Standard `use` targeting. |
| `kata` | Kata | Resets rotation. Bare `use 'kata'`. |
| `med` | Meditation | Resets rotation. If kata-not-done flag: queues kata + **doing meditation** flag; if kata done: `use 'meditation'`. |
| `umb` | Mind over body | Resets rotation. No args: self; with args: `use 'mind over body' at …`. |
| `uds` | *(rotation)* | Sends `use '{monk_current_disrupt_skill}'` with optional target (disrupt track). |
| `uaa` | *(rotation)* | Area track (`{monk_current_area_skill}`). |
| `uar` | *(rotation)* | Armour track (`{monk_current_armour_skill}`). |
| `uav` | *(rotation)* | Avoid track (`{monk_current_avoid_skill}`). |
| `uws` | Wave crest strike | Sets disrupt var to **wave crest strike**, then sends disrupt template. |
| `ugk` | Geyser force kick | Sets disrupt var to **geyser force kick**, then sends disrupt template. |
| `uek` | Earthquake kick | Sets armour var to **earthquake kick**, then sends armour template. |
| `uas` | Avalanche slam | Sets armour var (third slot; default alias for **falling boulder strike**), then sends armour template. |

Default rotation labels: **falling boulder strike** (armour), **wave crest strike** (disrupt), **hydra fang strike** (area), **falcon talon strike** (avoid) — triggers can advance these.

!!! note "Shared shortcuts"
    **`cs`** also means **Ranger** “open fight”, **Tzarakk** charge, **Riftwalker** spark opener — put the guild you play **first** in **`/guilds`**.

## Profile

Enable **`monk`** in **`/guilds`**.
