# Nergal

Minion block on the status bar, potentia/vitae “full” banners, auto **`nergal sc`** on death lines, harvest coloring, and parasite / enthralled commands.

## Line highlights

Minion status lines are absorbed into the HUD row (original line hidden). Unsummon clears the block. Loud notices when potentia or vitae cap. Greens on harvest-style gains.

## Shortcuts

Missing required arguments produce a local message without sending a game command: `cee` shows **“No target!”**; `cne` shows **“Need three arguments: cne <host> <a> <b>.”**; `aura` and `mutation` show usage for their missing arguments.

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `aura` | Embrace the gifts (aura) | Args required: `use embrace the gifts at start aura <args>`. |
| `mutation` | Embrace the gifts (mutation) | Args required: `use embrace the gifts at start mutation <args>`. |
| `udh` | Dreary hibernation | `use dreary hibernation`. |
| `us` | Stab | Bare: `use 'stab'`. With a target: `target <target>;use 'stab' <target>`. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cep` | Enthralling parasite | `cast 'enthralling parasite' <suffix>`. |
| `chv` | Harvest vitae | `target`; `cast 'harvest vitae' <target>`. |
| `ccg` | Corrupt ground | `cast corrupt ground`. |
| `ceh` | Evaluate host | `target`; `cast 'evaluate host' <target>`. |
| `crp` | Reap potentia | `target`; `cast 'reap potentia' <target>`. |
| `cps` | Parasitic swarm | `target`; `cast 'parasitic swarm' <target>`. |
| `cee` | End enthrallment | Args required: `cast end enthrallment at <host>`. |
| `cne` | Nourish enthralled | **Three tokens** required: `cast nourish enthralled at <host> consume <a> <b>`. |
| `cce` | Call forth enthralled | `cast call forth enthralled at <args>`. |

!!! note "Overlaps"
    **`cps`** clashes with **Psionicist** **`cps`**. **`us`** overlaps **Liberator slash** / **Spider stab**.

## Dreary hibernation readiness

The main status recovery bracket adds **`d`** after camping (`c`) and meditation (`m`) only while Nergal is selected in `/guilds`. The letter means **ready**, not currently hibernating; the existing `udh` shortcut is unchanged.

- **Start:** “Dark mist floods the area around you as twisted, rotting roots spew forth from it, enveloping you in a cocoon.” clears shared readiness, hiding both `d` and any eligible `m`. Meditation’s “You sit down and start meditating.” does the same.
- **Ready again:** “You feel in harmony with yourself, the universe and life in general.” restores shared readiness, showing both eligible markers.

Both messages remain in game output. Login optimistically initializes shared readiness as ready; no server cooldown query is made. Guild selection changes only hide or reveal markers, preserving readiness even when Nergal is deselected and reselected. Cocoon-start detection requires Nergal guild triggers; meditation-start detection requires core triggers. Harmony restores shared readiness when either matching trigger group is enabled.

## Profile

Enable **`nergal`** in **`/guilds`** so the minion row appears when the game sends status. The client sends **`nergal sc`** once after login (including login after `/connect`) when Nergal is selected, and when you newly select Nergal while logged in. Saving an unchanged selection does not resend it; deselecting and reselecting does. No BatMUD `nc` alias is required.
