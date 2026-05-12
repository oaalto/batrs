# Inner Circle

Short commands for blurred image, feather weight, shield of protection, and armour of aether.

## Line highlights

None from this guild.

## Shortcuts

All of these send **unquoted** `cast … at …` lines (no `'...'` spell wrapper in the client output).

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cbi` | Blurred image | Args empty: **`me`**, or **`entity`** if the inner-circle entity flag is set; otherwise explicit target text. |
| `cfw` | Feather weight | Args empty → **`me`**; else cast at args. |
| `csp` | Shield of protection | Same default-target rule as **`cbi`** (me vs entity flag). |
| `caoa` | Armour of aether | Same default-target rule as **`cbi`**. |

For **`cbi`**, **`csp`**, and **`caoa`**, an empty argument usually means **you**, unless automation marks an **entity** target—then Batrs substitutes **`entity`**.

## Profile

Enable **`inner_circle`** in **`/guilds`**.
