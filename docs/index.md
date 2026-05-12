# Batrs

Batrs is a terminal client for [BatMUD](https://www.bat.org). It shows the game in your terminal, highlights important lines, and gives you short commands (aliases) instead of typing long spells and skills by hand.

This manual explains the screen layout, the commands that start with `/`, where your settings are saved, and what each supported guild offers.

## Quick start

- Run the client from the project folder: `cargo run`.
- Your options and guild choices live in **`~/.batrs/`** (see [Player settings](manual/player-settings.md)).

## In this manual

| Topic | What it covers |
|-------|----------------|
| [UI](manual/ui.md) | Status bar, extra info lines, dialogs |
| [Commands](manual/commands.md) | `/` commands, shortcuts, overlaps between guilds |
| [Player settings](manual/player-settings.md) | Config file layout and common options |
| [Guilds](guilds/index.md) | One page per guild: highlights, shortcuts, settings |

## Building this manual (for editors)

If your system Python blocks global installs, use a small virtual environment:

```bash
python3 -m venv .venv-docs
.venv-docs/bin/pip install -r requirements-docs.txt
.venv-docs/bin/mkdocs serve
```

If you are allowed to install packages for your user:

```bash
pip install -r requirements-docs.txt
mkdocs serve
```
