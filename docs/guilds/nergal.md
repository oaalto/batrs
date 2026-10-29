# Nergal

Minion block on the status bar, potentia/vitae “full” banners, auto **`nergal sc`** on death lines, harvest coloring, and parasite / enthralled commands.

## Line highlights

Minion status lines are absorbed into the HUD row (original line hidden). Unsummon clears the block. Loud notices when potentia or vitae cap. Greens on harvest-style gains.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `aura` | Embrace the gifts (aura) | Args required: `use embrace the gifts at start aura <args>`. |
| `mutation` | Embrace the gifts (mutation) | Args required: `use embrace the gifts at start mutation <args>`. |
| `udh` | Dreary hibernation | `use dreary hibernation`. |
| `us` | Stab | Target required: `target`; `use 'stab' <target>`. |

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

- **Start:** “Dark mist floods the area around you as twisted, rotting roots spew forth from it, enveloping you in a cocoon.” hides `d`.
- **Ready again:** “You feel in harmony with yourself, the universe and life in general.” restores `d`. This also restores meditation readiness when core triggers are enabled.

Both messages remain in game output. Login and newly selecting Nergal optimistically initialize `d` as ready; no server cooldown query is made. Saving an unchanged selection preserves readiness, deselecting clears it, and reselecting initializes it again. Disabling guild triggers freezes message-driven updates without changing guild-selected visibility.

## Profile

Enable **`nergal`** in **`/guilds`** so the minion row appears when the game sends status. The client sends **`nergal sc`** once after login (including login after `/connect`) when Nergal is selected, and when you newly select Nergal while logged in. Saving an unchanged selection does not resend it; deselecting and reselecting does. No BatMUD `nc` alias is required.
