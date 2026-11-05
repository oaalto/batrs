# Triad

Cause-wound ladder, mellon collie, damn armament on a configured target, aura of hate, and red/green emphasis on curse drops, fails, and harm procs.

## Line highlights

Curse fading and “fail to reach” in red bold; harm messages in green bold.

## Shortcuts

### Spells

| Shortcut | Full name | Notes |
|----------|-----------|-------|
| `cclw` | Cause light wounds | `cast_spell`. |
| `ccsw` | Cause serious wounds | `cast_spell`. |
| `ccw` | Cause critical wounds | `cast_spell`. |
| `cda` | Damn armament | Casts on the target saved in **`/settings`**; shortcut arguments are ignored. |
| `cmc` | Mellon collie | `cast_spell`. |
| `caoh` | Aura of hate | `cast aura of hate` (unquoted). |

## Profile

Enable **`triad`** in **`/guilds`**.

Set **Damn Armament:** in **`/settings`** to your item target (for example, `weapon1` or `weapon2`). This saves **`damn_armament_target`** in your character’s `[settings]` table and takes effect immediately after saving.

The default is empty, including for existing profiles. Missing, empty, or whitespace-only targets send no command and display **`Set the Damn Armament target in /settings`**. Surrounding whitespace is trimmed; internal spaces and case are preserved. Semicolons and line breaks are rejected with a local error and no command sent.
