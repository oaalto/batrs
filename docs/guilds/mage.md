# Mage (core)

Large set of core mage spells and a few skills (**ceremony**, **create staff**). Line painting for **magic lore** / analyse output.

## Line highlights

Spell‑lore style lines get color markup when the client recognises them.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cere` | Ceremony | Standard `use` targeting. |
| `ucs` | Create staff | Standard `use` targeting. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cad` | Aura detection | `cast_spell`. |
| `cct` | Chill touch | `cast_spell`. |
| `ccf` | Create food | `cast_spell`. |
| `cd` | Darkness | `cast_spell`. |
| `cdi` | Disruption | `cast_spell`. |
| `cfa` | Flame arrow | `cast_spell`. |
| `cfab` | Force absorption | `cast_spell`. |
| `cf` | Floating | `cast_spell`. |
| `cfl` | Floating letters | `cast_spell`. |
| `chs` | Heal self | `cast_spell`. |
| `ci` | Identify | No args: `cast identify at me`. With args: at target. |
| `cinv` | Invisibility | `cast_spell`. |
| `cl` | Light | `cast_spell`. |
| `cmm` | Magic missile | `cast_spell`. |
| `cmb` | Mana barrier | `cast_spell`. |
| `cmi` | Mirror image | No args: `cast mirror image at me`. With args: at target. |
| `cms` | Moon sense | `cast_spell`. |
| `cpb` | Prismatic burst | `cast_spell`. |
| `cr` | Relocate | `cast_spell`. |
| `csi` | See invisible | `cast_spell`. |
| `csm` | See magic | `cast_spell`. |
| `csg` | Shocking grasp | `cast_spell`. |
| `ctwe` | Teleport with error | `cast_spell`. |
| `ctw` | Teleport without error | `cast_spell`. |
| `cts` | Thorn spray | `cast_spell`. |
| `cv` | Vacuumbolt | `cast_spell`. |
| `cww` | Water walking | `cast_spell`. |
| `cwor` | Word of recall | `cast_spell`. |

!!! note "Shared shortcuts"

    **`cmm`** is also on **Mage Magical** — pick one guild order.

    **`cfa`** matches **Mage Fire** (flame arrow), **`cfl`** matches **Mage Electricity** (forked lightning), and **`csg`** matches **Mage Electricity** (shocking grasp) — same rule: first guild in `/guilds` wins.

## Profile

Enable **`mage`** in **`/guilds`**.
