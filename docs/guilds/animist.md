# Animist

Soul companion on the status bar, ceremony-first casting for several spells, and a little automation when your spirit shows up.

## Line highlights

- **Soul line**: game text that shows your companion’s percent and stance is turned into the compact soul row (the raw line is hidden).
- When your **spirit answers your call**, the client sends **lead my spirit** for you.
- Training message about fighting with your companion is highlighted in blue.

## Shortcuts

### Skills

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cere` | Ceremony | Sends `use 'ceremony'`. |

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `csoul` | Separate soul | Sends `cast 'separate soul'` after ceremony when the client’s ceremony flag is not set yet (otherwise casts immediately). |
| `cjoin` | Join soul | Same ceremony gate as `csoul`. |
| `csum` | Conjure animal soul | Same ceremony gate. |
| `cdis` | Animal soul link (dismiss) | Sends `cast 'animal soul link' at dismiss` with the same ceremony gate. |

## Profile

Enable **`animist`** in **`/guilds`** so the soul row and these shortcuts are active.
