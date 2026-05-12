# Kharim

Chaos blade skills and spells, plus many **direction** shortcuts for shrine navigation (see in-game routes). Line highlights for chaos aura, evades, and circulation messages.

## Line highlights

Green/yellow/red emphasis on chaos aura, blade fire ending, circulation messages, evades, and similar lines.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `ufp` | Foul play | Prefixes **`kharim observe`**. No args: `use 'foul play'`. With target: `target`; `use 'foul play' <target>`. |
| `uam` | Deceitful act of mercy | Standard `use` targeting. |
| `ufr` | Feigned remorse | Standard `use` targeting. |
| `usd` | Scourge of dark steel | No args: bare `use`. With target: `kharim observe`; `target`; `use` on target. |
| `uvb` | Vampiric blow | Standard `use` targeting. |
| `ucc` | Chaotic circulation | `use Chaotic circulation at me` (casing as sent by Batrs). |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cfa` | Flame arrow | No args: **`cast flame arrow at device`**. With args: `cast flame arrow at <target>`. |
| `cbf` | Blade of fire | `cast blade of fire`. |
| `cac` | Aura of chaos | `cast aura of chaos`. |

### Other — combat helpers

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `kharim_rip` | Rip macro | `rip_action set get all from corpse; kharim drain corpse; drop zinc; drop mowgles`. |
| `kharim_help` | Route help | Prints route hints **only in the client** (nothing sent to the game). |

### Other — travel (fixed walks)

Each shortcut sends one semicolon-separated movement chain (exact steps match your install).

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `tositwar` | To sitwar trainer | From device toward shield trainer. |
| `fromsitwar` | From sitwar | Return chain. |
| `tomelee` | To melee trainer | Longer west run to general fighting trainer. |
| `frommelee` | From melee | Return. |
| `tosw` | To spell trainer | Device → SW spell trainer route. |
| `fromsw` | From SW | Return. |
| `tose` | To SE trainer | Device → SE specialist route. |
| `fromse` | From SE | Return. |
| `tonw` | To NW (scout) | Long NW walk including scout trainer. |
| `fromnw` | From NW | Return. |
| `tone` | To NE (attack skills) | NE attack-skills trainer route. |
| `fromne` | From NE | Return. |
| `tokitan` | To Kitan | Extended NW route ending **`enter`**. |
| `fromkitan` | From Kitan | **`out`** then return east run. |
| `tosouls` | To souls room | Includes **`ask man about services`** and **`kharim souls`**. |
| `fromsouls` | From souls | Return from souls room. |
| `tocloud` | To cloud | West run ending **`cloud`**. |
| `fromcloud` | From cloud | **`descend`** then east return. |
| `toswords` | To sword hotel | Separate compass chain ending **`enter`**. |
| `fromswords` | From sword hotel | **`out`** and return. |
| `todevice` | To device | From elevator toward device. |
| `fromdevice` | From device | Return toward elevator entrance. |

## Profile

Enable **`kharim`** in **`/guilds`**.
