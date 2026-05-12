# Player settings

Everything lives under **`~/.batrs/`**. Each character has a folder named from your login (safe characters only). Inside is a **TOML** file with the same base name — that file is your profile.

You usually edit this through **`/guilds`** and **`/settings`**, but advanced users can edit the file directly when the client is closed.

## Guild list

**`guilds`** — optional list of short ids (`reaver`, `mage_fire`, …). Only listed guilds load their shortcuts and line handling. Until you pick guilds, the list may be empty.

**`guild_primary_background`** — the theme bucket you chose in the guild dialog (civilized, magical, …).

## `[settings]`

| Key | Meaning |
|-----|---------|
| `rig` | Your automation rig name (set in **`/settings`** or edit the profile file) |
| `tzarakk_mount` | Mount name used for Tzarakk features |
| `sabre_weapon` | Main-hand weapon for Sabres shortcuts |
| `riftwalker_entity_fire` | Label you use for your fire entity |
| `riftwalker_entity_air` | Air entity label |
| `riftwalker_entity_water` | Water entity label |
| `riftwalker_entity_earth` | Earth entity label |

Any other **`key = "value"`** lines are kept as extra settings. One common extra:

- **`is_lich`** — set to `1`, `true`, or `yes` to turn on lich-focused behavior in the client.

## `[generic_commands]`

Example:

```toml
[generic_commands]
enabled_groups = ["cure_spells", "navigator"]
disabled_commands = ["clwf"]
```

- **`enabled_groups`**: empty means “all groups”; otherwise only named groups are active.
- **`disabled_commands`**: shortcuts to force off.

## Where changes are saved

- **`/guilds`** — guild list, theme, mount name, sabre weapon, riftwalker labels.
- **`/settings`** — the `[settings]` table (including **`rig`**).

If the file was upgraded from an older format, the client may rewrite it once to match the new layout. A broken file falls back to defaults and prints a short warning in the terminal.
