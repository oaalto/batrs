# Guild Catalog browse logic in Guild Dialog

## Status

ready-for-agent — grilled 2026-07-23

## Problem Statement

Guild Catalog owns persisted keys, display names, grouping, and playability — but Guild Dialog re-implements browse and drill logic: thematic bucket rows, multi-background drill, drill row rebuilding, cursor placement over row lists, and coordination with thematic-bucket selection clearing. The dialog module mixes UI state (focus, cursors, text inputs) with catalog topology (which definition indices appear in which drill, in what order, with which banners).

Adding a playable guild or changing thematic grouping currently requires touching both grouping data and dialog row builders. Banner strings and empty-state messaging live in the dialog layer alongside keystroke handling. Browse row rules are difficult to test without constructing dialog state and simulating navigation.

Recent deepen work on guild catalog selection improved selection semantics; browse/drill row construction did not move with it.

## Solution

Extract a **Guild Catalog browse** submodule that owns PickBackground label ordering and drill row structure. Guild Dialog becomes a shallow adapter: keystrokes mutate selection and focus state, call browse for labels and rows, enrich toggles with display title and selection at the view-model boundary, and render.

Grouping remains the source for thematic bucket indices, multi-background indices, and `clear_selected_outside_thematic_bucket`. Browse composes grouping into ordered row lists; it does not mutate player selection.

Ship now as one focused slice. If extraction surfaces genuine selection or persistence bugs, fix them in follow-on commits within the same pull request (separate commits per concern).

## User Stories

1. As a player, I want to pick my thematic background from a fixed ordered list, so that my saved primary background matches my character concept.
2. As a player, I want to open a drill list for my chosen thematic background, so that I can toggle playable guilds in that bucket.
3. As a player, I want multi-background guilds listed in thematic drills when applicable, so that I can enable them across themes.
4. As a player, I want a dedicated multi-background drill entry, so that I can browse multi guilds without picking a thematic background first.
5. As a player, I want unimplemented thematic groups to show a clear empty state, so that I am not confused about why no guilds appear.
6. As a player, I want thematic-only empty buckets to explain that no playable guild exists in that group yet while still showing multi-background guilds when available, so that the drill layout matches expectations.
7. As a player who changes thematic primary, I want guilds outside the new bucket cleared from selection, so that saved preferences stay mutually consistent with thematic rules.
8. As a player, I want mount, sabre weapon, and Riftwalker entity inputs to behave as today, so that guild-specific configuration is unchanged by this refactor.
9. As a maintainer adding a playable guild, I want drill rows to appear automatically from catalog grouping, so that I do not edit dialog banner logic.
10. As a maintainer changing thematic UX order, I want one browse module to update, so that picker labels and drill structure stay aligned.
11. As a maintainer, I want thematic bucket indices and mutual-exclusion clearing rules defined in grouping, not duplicated in dialog, so that catalog data and dialog behavior stay aligned.
12. As a maintainer, I want empty-state banner copy colocated with row-ordering logic, so that messaging changes do not require dialog edits.
13. As a test author, I want to test browse label and drill row generation without simulating key events, so that catalog UX rules have a single test home.
14. As a test author, I want dialog tests to remain focused on cursor, focus, and keystroke behavior, so that interaction regressions are caught without duplicating row-structure coverage.
15. As a maintainer, I want toggle definition indices in drill rows always within the playable entry list length, so that out-of-range indices never reach the UI.
16. As a maintainer reviewing the pull request, I want browse extraction in a dedicated refactor commit separate from any discovered fixes, so that the structural change is easy to review and revert.
17. As a maintainer, I want domain context updated to record browse ownership, so that future readers know where catalog topology UX lives.

## Implementation Decisions

### Module ownership

- Add a **Guild Catalog browse** submodule under Guild Catalog.
- Browse owns: PickBackground label list (`browse_labels`), drill source discriminant (`GuildDrillSource`: thematic index or multi-only), drill row type (`GuildBrowseRow`), and drill row generation (`drill_rows`).
- `guilds/grouping` retains thematic bucket indices, multi-background indices, and `clear_selected_outside_thematic_bucket`. Browse reads grouping; it does not own bucket membership rules.
- Guild Dialog retains focus, cursors, keystroke handling, drill/b browse mode state, cached drill row list for cursor indexing, and all guild-specific text inputs (Tzarakk mount, sabre weapon, Riftwalker entities) including optional-input visibility.

### Browse API

- **`browse_labels()`** — returns ordered PickBackground labels (five thematic rows plus multi-background entry). Dialog stops assembling labels from grouping constants at presentation time.
- **`GuildDrillSource`** — `Thematic(usize)` for a thematic bucket index (0–4) or `MultiOnly` for the multi-background drill. Lives in browse; dialog imports it.
- **`GuildBrowseRow`** — structural DTO:
  - `Banner(&'static str)` — includes empty-state and section-header copy; strings owned by browse
  - `Toggle { definition_index }` — index into playable catalog entries
- **`drill_rows(source, entry_count)`** — returns ordered `GuildBrowseRow` list for the given drill source. Filters toggle indices to `definition_index < entry_count` (preserves existing defensive guard). Read-only; no selection input.

Row ordering and banner rules match current dialog behavior for thematic drills (thematic toggles, optional multi section header, multi toggles) and multi-only drills (empty banner when none implemented, otherwise multi toggles).

### Dialog adaptation

- Replace inline `rebuild_drill_rows` body with a call to `drill_rows`, passing `self.entries.len()` as `entry_count`.
- Replace inline browse label assembly with `browse_labels()`.
- Dialog continues calling `clear_selected_outside_thematic_bucket` directly on thematic primary change and at dialog construction — not routed through browse.
- Dialog maps `GuildBrowseRow::Toggle` to UI view model guild lines by resolving `definition_index` against entries and `selected[]`; banners pass through as-is.

### Test seam

- **Primary seam:** Guild Catalog browse module — unit-test `browse_labels` and `drill_rows` directly. This is the highest seam that covers all browse topology rules in one place.
- Dialog remains the seam for interaction tests (cursor movement, focus routing, text-input keystrokes). Do not duplicate row-structure assertions through dialog integration unless a regression requires it.

### Behavior fixes (in scope if discovered)

- Minor player-visible fixes allowed during extraction: banner wording improvements, edge-case selection bugs.
- `GuildSelection` output fixes allowed if a genuine bug is found.
- TOML migration allowed if a persistence bug requires it.
- Fixes ship in separate commits after the browse extraction commit, within the same pull request.

### Delivery

- One pull request.
- Commit order: `refactor:` browse extraction first; then `fix:` / `migrate:` commits only if issues are discovered during implementation.

## Testing Decisions

### What makes a good test

- Test browse **outputs** (label order, row sequence, banner text, toggle indices) given drill source and entry count — not dialog keystrokes or ratatui rendering.
- Assert all toggle `definition_index` values are `< entry_count`.
- Assert empty thematic drill (no thematic playables, no multis) yields the full empty banner.
- Assert thematic drill with no thematic playables but multis yields the partial-empty banner plus multi section.
- Assert multi-only drill with no implemented multis yields the multi empty banner.
- Dialog tests: keep existing interaction coverage; do not add redundant drill-row structure tests once browse unit tests exist.

### Prior art

- Guild Dialog interaction tests (cursor, focus, mount/tab routing).
- Guild grouping tests (bucket classification, multi indices).
- Guild catalog selection tests (thematic index helpers).

## Out of Scope

- Changing thematic group membership rules or adding guilds to the catalog.
- Guild Dialog text-input behavior changes beyond bug fixes discovered during extraction.
- Optional-input visibility logic moving into browse.
- Settings dialog or generic commands dialog.
- In-game guild command behavior.
- Moving `clear_selected_outside_thematic_bucket` into browse or selection modules (grouping keeps it; dialog keeps calling it).

## Further Notes

- `CONTEXT.md` Guild Catalog section updated during grilling to record browse ownership, row DTO shape, test placement, and fix/migration scope.
- Related deepen: guild catalog selection semantics (`088ec17`).
- No ADR required unless implementation discovers a hard-to-reverse trade-off not captured here.
