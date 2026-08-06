BatMUD Client
=============

This is a very early version of a client that can connect to [BatMUD](https://www.bat.org). It will be a very opionated as I'm making it for myself and just for fun. There are many other MUD clients that are more suitable for general use. I even have a separate repository of BatMUD tf triggers [here](https://github.com/oaalto/batmud-tf-trigs) that offer a much better experience for now.

## Manual

The user-facing manual is built with [MkDocs](https://www.mkdocs.org/) (Material theme). See `docs/index.md`, `mkdocs.yml`, and `requirements-docs.txt`.

- **Build then serve** (strict build, then dev server): `./scripts/serve-manual.sh`  
  MkDocs **`serve`** already watches files and **live-reloads** the browser (pass **`--no-livereload`** to turn that off).  
  Optional args are forwarded (e.g. `-a 127.0.0.1:8001`, `--no-livereload`).  
  If `127.0.0.1:8000` is busy, the script picks a free port unless you set `MKDOCS_SERVE_ADDR` or `-a`.

Quick local setup:

```bash
python3 -m venv .venv-docs
.venv-docs/bin/pip install -r requirements-docs.txt
./scripts/serve-manual.sh
```

## Combat damage viewer

While batrs is running, it records incoming damage (melee verbs, skills, spells) into `~/.batrs/combat_damage.db` and serves a read-only HTML dashboard on **http://127.0.0.1:6464/** (localhost only). The server starts automatically in the background when batrs launches; it stops when batrs exits.

Override the port with `--port` (default `6464`):

```bash
cargo run -- --port 8080
```

**Landing page (`/`):** three tables — melee, skill, spell — one row per hit verb. The melee table is grouped into **weapon-family** sub-sections (from `docs/hit_messages.md`). Each row has **confirmed** columns (only unambiguous hits) and **estimated** columns (includes ambiguous batches with conservative bounds). Filter by time range (`24h`, `7d`, `all`) and player; click column headers to sort within each section.

**Drill-down (`/events/{category}/{verb}`):** individual events with timestamp, player, HP delta, source, weight, and original message text. Rows that share a `batch_id` (one HP loss split across multiple candidate lines) are grouped.

An empty database still shows the full page structure with zero rows. If the database cannot be opened, the viewer returns HTTP 503; the TUI keeps running.
