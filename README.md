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

**Filters:** change time range and player only on the landing page. All drill-downs inherit that applied selection through their links and show a read-only summary with **Change filters on main page**, right-aligned beside the navigation links (wrapping on narrow screens). Back, sorting, record navigation, and deletion returns retain the filters. Direct links without filters use All time / All players; already-open tabs retain their own URL selection. Last 24h and Last 7d remain rolling windows, not snapshots.

**Section links:** every page and section heading has a `#` permalink. Use normal browser link-copy controls to bookmark the current URL, including its filters and sorting: `/#melee`, `/#skill`, `/#spell`, `/#unattributed`, or `/unattributed/{id}#context-lines`. Melee-family targets use `#melee-{family_id}` (for example, `#melee-unarmed`); a family hidden by the current filters has no target. Sorting stays at the relevant category, family, or event heading. Back returns to the originating category/family or Unattributed section, while **Change filters on main page** goes directly to the filter controls. Record and verb links land at the destination heading; Previous / Next keep record details above the transcript. Both deletion actions return to `#unattributed`. Applying filters still returns to the page top.

**Drill-down (`/events/{category}/{verb}`):** individual events with timestamp, player, HP delta, source, weight, and original message text. Rows that share a `batch_id` (one HP loss split across multiple candidate lines) are grouped.

**Unattributed HP loss (`/unattributed/{id}`):** review the saved context followed by the triggering `H:` HP-loss line. This includes existing records and losses with no preceding context. The triggering line also appears in the event details table. Inherited filters scope **Previous / Next**, not the saved transcript. A directly linked record outside those filters still displays in full, with an explanation and no Previous / Next links. **Delete** is available on each landing-table row and on the single-record page, for both reviewed and unreviewed records. It permanently removes only that record and its saved context, without confirmation or undo, then returns to the landing section with the current time-range and player filters. Other records and attributed damage statistics are unchanged. **Remove reviewed** still removes only reviewed records within the current filters.

An empty database still shows the full page structure with zero rows. If the database cannot be opened, the viewer returns HTTP 503; the TUI keeps running.
