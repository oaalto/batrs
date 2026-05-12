# Spider

Demon summon/control, spider walk, toxic/venom lines, stab, and loud highlights for queen help, losing control, heavy weight dropping, and demon power lines.

## Line highlights

Color-coded feedback for queen boons, demon struggles, poison streak successes, blocked stabs, and when **heavy weight** falls off (with a banner).

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `us` | Stab | No args: `use 'stab'`. With target: `target`; `use 'stab' <target>`. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `csw` | Spider wrath | No args: `cast 'spider wrath'`. With target: `cast 'spider wrath' <t>`. |
| `chs` | Hunger of the spider | Target required: `cast hunger of the spider at <t>`. |
| `csum` | Spider demon conjuration | `cast spider demon conjuration at me with <args>` (args can be empty). |
| `ctrl` | Spider demon control | `cast spider demon control at me`. |
| `csac` | Spider demon sacrifice | `cast spider demon sacrifice at <args>`. |
| `cban` | Spider demon banishment | `cast spider demon banishment at me`. |
| `cinq` | Spider demon inquiry | `cast spider demon inquiry at me`. |
| `cchan` | Spider demon channeling | `cast spider demon channeling at me`. |
| `ctd` | Toxic dilution | No args: `cast toxic dilution at me`. With args: `cast 'toxic dilution' <t>`. |
| `cvb` | Venom blade | Target required (`cast_spell` style). |
| `cswalk` | Spider walk | No args: `cast spider walk at me`. With args: `cast 'spider walk' <t>`. |
| `chw` | Heavy weight | No args: `cast heavy weight at me`. With args: `cast 'heavy weight' <t>`. |
| `cmsac` | Spider demon mass sacrifice | `cast spider demon mass sacrifice`. |
| `cpsq` | Prayer to the spider queen | `cast prayer to the spider queen`. |
| `crmp` | Remove poison | No args: `cast remove poison at me`. With args: at given target. |

!!! danger "Name clashes with generic cures"
    Generic cures use **`csw`** for **cure serious wounds** — only one works. Turn off the generic alias or put guilds in an order you like.

!!! note "Heavy weight"
    Generic navigator also defines **`chw`** for heavy weight — same clash idea.

## Profile

Enable **`spider`** in **`/guilds`**.
