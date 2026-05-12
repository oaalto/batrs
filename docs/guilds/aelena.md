# Aelena

Poison combos, familiar helpers, rip templates, and highlights for harvest and spell trouble. Some harvest lines trigger **familiar store** for you.

## Line highlights

Harvest lines for **spleen / lung / eye** can queue familiar store commands. Familiar leveling in green; failed spells in red; other situational colors.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `uw` | Wound | Standard `use` targeting. |
| `ut` | Thrust | Standard `use` targeting. |
| `ud` | Dissection | **Two arguments** after the command: `use dissection at corpse try <word> <word>`. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `csa` | Sting of aelena | Standard `cast` targeting. |
| `csd` | Slow death + sting | **`aelena poison slow death`**, then targeted **`cast 'sting of aelena'`**. |
| `crb` | Rusted blade + sting | **`aelena poison rusted blade`**, then sting cast. |
| `cbt` | Black trance + sting | **`aelena poison black trance`**, then sting (defaults sting target to **`me`** if args empty). |
| `cb` | Bite of the black widow | Standard `cast` targeting. |
| `ccb` | Command blade | `cast command blade` (unquoted). |

### Other — familiar

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `fc` | Familiar consume | `familiar consume` or `familiar consume <tail>`. |
| `fssd` | Familiar store | `familiar store slow death`. |
| `fsrb` | Familiar store | `familiar store rusted blade`. |
| `fsbt` | Familiar store | `familiar store black trance`. |

### Other — rip templates

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `rip_consume` | Rip consume | `rip_action` + `familiar consume corpse` + drops. |
| `rip_dissect` | Rip dissect | `rip_action` + drops only. |
| `rip_lung` | Rip harvest lung | `rip_action` + `familiar harvest lung any` + drops. |
| `rip_spleen` | Rip harvest spleen | `rip_action` + `familiar harvest spleen any` + drops. |
| `rip_eye` | Rip harvest eye | `rip_action` + `familiar harvest eye any` + drops. |

!!! note "Shared shortcut"
    **`cb`** is also Reaver **word of blasting** if that guild comes first in your list.

## Profile

Enable **`aelena`** in **`/guilds`**.
