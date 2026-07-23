# 01 — Extract Guild Catalog browse and delegate Guild Dialog

## Parent

`prd.md`

## What to build

Players see the same guild dialog as today: PickBackground labels, thematic and multi-background drills, empty-state banners, guild toggles, and mount/sabre/Riftwalker text inputs. Maintainers change browse topology in one Guild Catalog browse submodule instead of editing dialog row builders.

Introduce browse with `browse_labels`, `GuildDrillSource`, `GuildBrowseRow`, and `drill_rows(source, entry_count)`. Guild Dialog delegates label and drill row generation, enriches toggles at the view-model boundary, and keeps focus, cursors, keystroke handling, and `clear_selected_outside_thematic_bucket` calls via grouping. Land as one green vertical slice with browse unit tests; any bugs found during wiring are left for ticket 02.

## Blocked by

None — can start immediately.

## Status

ready-for-agent

## Acceptance criteria

- [ ] Guild Catalog browse submodule exports `browse_labels`, `GuildDrillSource`, `GuildBrowseRow`, and `drill_rows(source, entry_count)`
- [ ] Row ordering, banner copy, and toggle index bounds match current dialog behavior (thematic drills, multi section, multi-only drill, empty states)
- [ ] `drill_rows` filters toggle indices to `definition_index < entry_count`
- [ ] Guild Dialog delegates browse label and drill row generation; no duplicate row-topology logic remains in dialog
- [ ] Dialog still calls `clear_selected_outside_thematic_bucket` on thematic primary change; browse does not mutate selection
- [ ] Browse unit tests cover label order, empty thematic drill, partial thematic empty with multis, multi-only empty, and index bounds — without keystroke simulation
- [ ] Existing guild dialog interaction tests (cursor, focus, text-input routing) still pass; no redundant drill-row structure tests added to dialog
- [ ] Refactor commit is separate from any fix commits (fixes belong in ticket 02)
- [ ] `cargo test` passes
