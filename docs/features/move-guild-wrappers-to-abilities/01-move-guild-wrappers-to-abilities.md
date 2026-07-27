# 01 — Move guild ability wrappers to Abilities

**Parent:** `prd.md`

**What to build:** Guild command modules import `use_skill` and `cast_spell` directly from Abilities instead of through Guild pass-through re-exports. Remove the wrappers and their Guild-module tests; add symmetric `cast_spell` public-API coverage in Abilities. Land as one green slice with no runtime behavior change — formatted `use`/`cast` command strings stay identical.

Document the Abilities bounded context in the domain glossary and engineering wiki so agents and maintainers see the guild→abilities seam in code and docs together: new Abilities concept page (formatting rules + helper inventory), wiki index and path-map entry, Guild Catalog cross-link and consumption fact, wiki ingest log entry. Player manual pages and Command Dispatch wiki stay unchanged.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [x] Guild root module no longer exports `use_skill` or `cast_spell` pass-through functions; no callers import them from Guild
- [x] All guild command modules that used Guild wrappers now use named Abilities imports; combined Guild imports split so only Guild-owned items remain; call sites unchanged
- [x] Guild wrapper tests deleted; Abilities gains `cast_spell_matches_targeted_form` symmetric with existing `use_skill` coverage (empty args → `@cast '<name>'`; with args → `@target <t>;cast '<name>' <t>`)
- [x] Secondary Abilities helpers (`cast_quoted_tail`, `compound_send`, etc.) unchanged
- [x] `CONTEXT.md` gains Abilities bounded-context section: owns `use`/`cast` line formatting; guild command modules consume it; Command Dispatch does not own it
- [x] Engineering wiki: new Abilities concept page (Summary, Verified Facts with targeted-command rules, Other exports inventory, Related); index entry; path-map mapping for Abilities source tree; Guild Catalog Related link + Verified Facts bullet on Abilities imports; ingest log entry
- [x] Command Dispatch wiki and per-guild player manual pages unchanged
- [x] `cargo test --all-targets --all-features` passes; format and clippy gates pass
