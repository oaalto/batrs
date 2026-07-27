# 02 — `/triggers` dialog and in-session save

## Parent

`prd.md`

## What to build

A logged-in player types `/triggers` at the main prompt and opens a toggle dialog to enable or disable the four built-in trigger groups without hand-editing TOML. The interaction mirrors `/genericcommands`: Up/Down to move, Space to toggle, Enter to save, Esc to cancel. Title is **Triggers**; rows (pipeline order) are **Guild triggers**, **Spell vocals**, **Common triggers**, **Core triggers**.

On open, the dialog seeds `saved` and `draft` from in-memory runtime profile trigger config (no disk re-read). Enter with unchanged draft closes without I/O. Enter with changes persists first; on success updates runtime profile directly and closes; on failure the dialog stays open with a fixed footer error (`Player config not available` or `Failed to save trigger settings`). Save errors clear on Space or Up/Down. When draft has core triggers off, footer row 2 shows **Prompt/stats parsing disabled**; row 1 shows key help or the save error.

`/triggers` requires login, loads user config when not yet loaded (same gates as `/genericcommands`), and never sends text to BatMUD. Changes take effect on the next incoming line after successful save.

## Blocked by

- [01 — Trigger config, pipeline gating, and profile persistence](01-trigger-config-pipeline-and-persistence.md)

## Status

done

## Acceptance criteria

- [x] `/triggers` registered as a builtin command with `requires_login: true`; opening respects login and config-load gates like `/genericcommands`.
- [x] Triggers dialog struct holds `saved`, `draft`, cursor, and footer error state; view model exposes row labels, toggle states, `footer_line1`, and optional `footer_line2`.
- [x] Space toggles the selected row on `draft` only; Up/Down move selection and clear footer error after a failed save.
- [x] Esc closes without persisting or mutating runtime profile.
- [x] Enter: `draft == saved` closes with no disk write; otherwise atomic save (persist then assign runtime profile on `Ok` only).
- [x] Modal captures keystrokes while open; main prompt commands do not run underneath.
- [x] UI renders the dialog using the view model (title, four rows, two-line footer).
- [x] No master "all groups" row; no raw I/O error text in the footer.
- [x] After successful save, disabling a group (e.g. guild triggers) affects `process()` on subsequent lines without restart.
- [x] `cargo test --all-targets --all-features` passes.
