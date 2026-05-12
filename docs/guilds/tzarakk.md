# Tzarakk

Mount detection, round reminders, charge outcomes, mount status, and rip modes that chain corpse commands. **Meditation** and **sleep** dismount first.

## Line highlights

Tracks mount summoned / dismounted / riding / banish / charge hit or miss / mount HP-style status, and similar lines so automation and flags stay in sync with what the game prints.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `ut` | Trample | Standard `use` targeting. |
| `ur` | Rampage | Standard `use` targeting. |
| `cs` | Charge | Standard `use` targeting. |
| `uht` | Create hunting trophy | Sends `use 'create hunting trophy' at corpse`. |
| `uhs` | Harvest soul | Sends `use 'harvest soul' at corpse`. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cpc` | Preserve corpse | Sends `cast 'preserve corpse' at corpse`. |
| `cst` | Steed of tzarakk | Sends `cast 'steed of tzarakk'`. |
| `cban` | Banish mount | Sends `cast 'banish mount'`. |

### Other

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `med` | Meditation | **`dismount`**, then `use 'meditation'`. |
| `sleep` | Sleep | **`dismount`**, then `sleep`. |
| `feed_mode` | Rip: feed | Sets `rip_action` chain: loot, `tzarakk chaosfeed corpse` (twice), drop zinc/mowgles. |
| `heal_mode` | Rip: heal | Sets `rip_action`: loot, `use 'harvest soul' at corpse`, drops. |
| `hunt_mode` | Rip: hunt | Same loot/`chaosfeed` pattern as `feed_mode`. |

!!! note "Shared shortcuts"
    **`cs`** overlaps other guilds. **`med`** overlaps Monk, Tiger, Psionicist.

## Profile

- **`tzarakk_mount`** — mount name (also in **`/guilds`**).
- Enable **`tzarakk`** in **`/guilds`**.
