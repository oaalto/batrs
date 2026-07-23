# Guild Catalog browse logic in Guild Dialog

## Status

draft — initial exploration for grilling

## Problem Statement

Guild Catalog owns persisted keys, display names, grouping, and playability — but Guild Dialog re-implements browse and drill logic: thematic bucket rows, multi-background drill, `rebuild_drill_rows`, cursor placement, and `clear_selected_outside_thematic_bucket` coordination. `guild_dialog.rs` is ~1000 lines mixing UI state (focus, cursors, text inputs) with catalog topology (which indices appear in which drill). Adding a playable guild or changing thematic grouping requires touching both `guilds/grouping.rs` and dialog row builders.

Recent deepen work (`088ec17 Deepen guild catalog selection`) improved selection semantics; browse/drill row construction did not move with it.

## Initial exploration

**Guild Catalog / grouping (data):**

- `guilds/catalog/mod.rs` — `GuildCatalogEntry`, `GuildKey`, playability, `playable_entries_list`
- `guilds/catalog/selection.rs` — `GuildSelection`, persistence, thematic index helpers
- `guilds/grouping.rs` — `guild_grouping()`, thematic buckets, `visible_indices_multi_drill()`, `clear_selected_outside_thematic_bucket`

**Guild Dialog (UI + browse logic):**

- `app/dialogs/guild_dialog.rs` — modes `PickBackground` / `DrillGuild`; `rebuild_drill_rows` builds `DrillRow::Banner | Toggle { definition_index }`
- Thematic drill: bucket playable indices + multi indices with banner separators
- Multi-only drill: `visible_indices_multi_drill()` filtered
- `app/mod.rs` — opens dialog with catalog entries, applies keystrokes, persists on close
- `ui/mod.rs` — `GuildDialogPresentation` view models for ratatui render

**Friction:**

- `DrillRow` and `rebuild_drill_rows` encode catalog UX rules duplicating grouping knowledge
- Banner strings ("Nothing implemented yet…", "Multi-background guilds") live in dialog
- `THEMES_UX_ORDER` indexed by cursor in dialog — coupling UX order to browse state
- Tests in `guild_dialog.rs` likely cover drill rows — grep if implementing

## Solution (proposed direction)

Deepen Guild Catalog (or a `catalog/browse.rs` submodule) with an interface that returns browse/drill **structure** — ordered list of rows (banner, toggle at definition index, thematic picker labels) — given catalog entries + active thematic index. Guild Dialog becomes a shallow adapter: keystrokes mutate selection state, call catalog for row model, render view model.

Grilling should decide: does browse logic belong in Guild Catalog module or adjacent `guilds/grouping`?

## User Stories

1. As a player, I want to pick my thematic background and toggle playable guilds in a drill list, so that my saved guild set matches my character.
2. As a player, I want multi-background guilds listed separately, so that I can enable them across themes.
3. As a maintainer adding a playable guild, I want drill rows to appear automatically from catalog grouping, so that I do not edit dialog banner logic.
4. As a maintainer, I want thematic bucket rules (mutual exclusion on save) defined once, so that dialog and `GuildSelection` stay aligned.
5. As a test author, I want to test browse row generation without simulating key events, so that catalog UX rules have locality.
6. As a maintainer, I want unimplemented thematic groups to show a clear empty state, so that players are not confused — wording may stay in dialog or move to catalog.

## Open questions (for grilling)

1. **Ownership:** `GuildCatalog` vs. `guilds/grouping` vs. new `guild_browse` module — where does the seam live?
2. **Row model:** Shared type between dialog and catalog, or dialog-specific VM built from catalog DTO?
3. **Empty states:** Are banner messages catalog responsibility or presentation (dialog/UI)?
4. **Mount/sabre/riftwalker inputs:** Stay in dialog only — confirm out of scope for catalog browse extraction?
5. **Selection clearing:** `clear_selected_outside_thematic_bucket` — call from catalog browse interface or remain dialog action on background change?
6. **Speculative threshold:** Is ~1000-line dialog worth splitting now, or wait until next guild UX change?

## Implementation Decisions (tentative)

- Introduce `GuildBrowsePlan` (name TBD): `browse_labels()`, `drill_rows(source: Thematic | MultiOnly) -> Vec<DrillRowKind>`.
- `DrillRowKind` carries `definition_index` for toggles; banners are data or enum variants.
- Guild Dialog `rebuild_drill_rows` delegates to catalog/browse module; keeps focus/cursor/input state.
- `visible_indices_multi_drill` and thematic bucket indices remain sourced from `guild_grouping()`.
- No change to persistence format or `GuildSelection` semantics in first slice.

## Testing Decisions

- **Good tests:** Thematic drill with only multi guilds shows expected banners; multi-only drill empty when none implemented; indices always within entries length.
- **Prior art:** `guild_dialog.rs` tests; `guilds/grouping.rs` tests if any; `catalog/selection.rs` tests.
- **Avoid:** Full TUI keystroke integration unless regressions require it.

## Out of Scope

- Changing thematic group membership rules or adding guilds to catalog.
- Guild Dialog text inputs (Tzarakk mount, sabre weapon, Riftwalker entity labels).
- Settings dialog or generic commands dialog.
- In-game guild command behavior.

## Further Notes

- Recommendation strength: **Speculative** — real duplication, but dialog works; payoff on next catalog UX change.
- Related deepen: `088ec17 Deepen guild catalog selection`.
- `CONTEXT.md` Guild Catalog section defines ownership — browse UX should not contradict selection persistence rules.
