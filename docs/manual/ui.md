# UI and interaction

The client fills the terminal. The **main area** is game text (scrolls as new lines arrive). Below that you get **status lines**, a **clock**, and the **input** line where you type.

## Game text and typing

- Text wraps to the width of your terminal.
- After login, the prompt looks like `> ` plus what you are typing. During password entry the prompt behaves differently for privacy.
- The **clock** shows the current local time (`HH:MM:SS`).

## Main status line

After you are logged in, a **single summary line** can show combat and character info. Before login it stays empty.

## Extra status rows

Sometimes the client adds **another line or two** under the main status:

- **Soul companion** (Animist, or when the game shows companion status): a short soul summary.
- **Nergal minions**: extra rows when you have minion status to show.

When these rows appear, the story window above gets a little shorter so everything still fits.

## Dialogs

When you open **Guilds**, **Generic commands**, or **Settings**, you work in a full-screen dialog. The typing cursor is hidden until you close the dialog.

### Guild selector (`/guilds`)

1. **Pick a theme** (civilized, magical, and similar buckets). That choice is saved and shapes which guilds are suggested.
2. **Turn guilds on or off** for this character. That is saved as your guild list.

**Extra fields** (only when the matching guild is selected):

| Guild | What you can set | Saved as |
|-------|------------------|----------|
| Tzarakk | Mount name | `tzarakk_mount` |
| Sabres | Main-hand weapon | `sabre_weapon` |
| Riftwalker | Names for fire / air / water / earth entities | `riftwalker_entity_fire`, `…_air`, `…_water`, `…_earth` |

Use **Tab** / **arrow keys** / **Enter** as usual to move around the dialog (same as in the app).

### Generic commands (`/generic`)

Turn whole groups of shortcuts (cures, travel spells, and so on) on or off. Changes are saved as described under [Player settings](player-settings.md).

### Settings (`/settings`)

Edit key/value settings in a simple list. Saving updates your profile file on disk.
