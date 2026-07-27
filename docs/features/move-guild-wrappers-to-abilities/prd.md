# Move Guild Command Wrappers to Abilities Module

## Status

`ready-for-agent` — synthesized from grill session (Jul 2026).

## Problem Statement

Guild modules format BatMUD `use` and `cast` command lines through two pass-through functions re-exported from the Guild module (`use_skill`, `cast_spell`). Those functions delegate to the Abilities module, which already owns the canonical implementations. The re-export adds no logic but creates confusion about which module is authoritative and implies a tighter coupling than actually exists.

Twenty-four guild command modules import these wrappers from Guild. Guild module tests exercise the wrappers locally. Domain documentation names Command Dispatch and Guild Catalog but does not record Abilities as the bounded context for ability-line formatting. The existing feature brief also misstates the dependency as circular and lists guild module changes as out of scope despite the migration requiring import updates.

## Solution

Remove the pass-through wrappers from Guild. Guild command modules import `use_skill` and `cast_spell` directly from Abilities using named imports; call sites stay unchanged. Move public-API test coverage for `cast_spell` into Abilities (symmetric with existing `use_skill` coverage). Delete the wrapper tests from Guild.

Ship as one atomic pull request: code migration, test relocation, PRD correction, domain glossary update, and engineering-wiki ingest land together. No runtime behavior change.

## User Stories

1. As a maintainer editing a guild command, I want `use_skill` and `cast_spell` to import from Abilities, so that the canonical formatting module is obvious at the call site.
2. As a maintainer reading Guild module code, I want no pass-through re-exports of Abilities functions, so that Guild does not pretend to own ability-line formatting.
3. As a maintainer adding a new guild command that casts or uses, I want to follow the same Abilities import pattern as existing guilds, so that conventions stay consistent.
4. As a maintainer changing targeted-command formatting rules, I want a single module to edit, so that guild modules do not need duplicate logic.
5. As a test author, I want public-API tests for both `use_skill` and `cast_spell` in Abilities, so that formatting regressions are caught at the authoritative seam.
6. As a test author, I want Guild module tests to stop testing Abilities behavior, so that test ownership matches module ownership.
7. As an agent navigating the codebase, I want `CONTEXT.md` to name Abilities as the bounded context for `use`/`cast` line formatting, so that the guild→abilities boundary is documented.
8. As an agent routing from `src/abilities/**`, I want a wiki concept page with formatting rules and a path-map entry, so that durable knowledge lives outside source comments.
9. As an agent routing from `src/guilds/**`, I want Guild Catalog wiki facts to state that command modules import formatting helpers from Abilities, so that catalog docs do not imply Guild owns formatting.
10. As a player reading per-guild manual pages, I want command behavior descriptions unchanged, so that a module-path refactor does not churn player-facing docs.
11. As a reviewer, I want the feature PRD to match agreed scope (guild import updates in scope, one-way dependency, test migration), so that the spec is trustworthy for implementation.
12. As a reviewer, I want one pull request with code and docs together, so that code and glossary/wiki do not temporarily disagree about the seam.
13. As a maintainer running the test suite, I want all existing tests to pass with no new warnings, so that the refactor is behavior-preserving.
14. As a maintainer using secondary Abilities helpers (`cast_quoted_tail`, `compound_send`, etc.), I want those exports unchanged, so that only the two primary wrappers move off the Guild re-export path.
15. As an agent, I want wiki documentation to record targeted-command formatting rules explicitly, so that empty-args vs targeted forms are discoverable without reading every test.
16. As an agent, I want a helper inventory on the Abilities wiki page, so that secondary exports are orienting without duplicating full API reference.
17. As a maintainer, I want Command Dispatch documentation left unchanged, so that slash-command precedence docs are not cluttered with guild ability formatting (orthogonal seams).

## Implementation Decisions

### Dependency and ownership

- **Abilities** owns canonical BatMUD `use`/`cast` command-line formatting (`use_skill`, `cast_spell`, `targeted_use`, `targeted_cast`, `client_send_line`, suffix helpers, `compound_send`, `repeat_inf_cast_heal_self`, `floating_disc`).
- **Guild** owns the `Guild` trait, per-guild command/trigger modules, and catalog metadata — not ability-line formatting.
- **Command Dispatch** does not own guild `use`/`cast` formatting (orthogonal to slash-command precedence).
- The current Guild→Abilities re-export is **one-way**, not circular. Abilities does not import Guild.

### Code migration

- Remove `use_skill` and `cast_spell` pass-through functions from the Guild root module.
- Update imports in all twenty-four guild command modules that currently pull `use_skill` / `cast_spell` from Guild:
  - Use **named imports** from Abilities (`use crate::abilities::use_skill`, `use crate::abilities::cast_spell`, or combined `use crate::abilities::{cast_spell, use_skill}`).
  - Split combined Guild imports so only the `Guild` type (and other Guild-owned items) remain on `crate::guilds`.
  - **No call-site edits** — bare `use_skill(...)` / `cast_spell(...)` calls stay as-is.
  - Files that already import the Abilities module for lower-level helpers keep existing qualified calls (`abilities::targeted_cast`, etc.) unchanged.
- Secondary Abilities API (`cast_quoted_tail`, `use_quoted_tail`, `cast_quoted_with_suffix`, `use_quoted_with_suffix`, `compound_send`, `repeat_inf_cast_heal_self`, `floating_disc`) — no changes; many guild modules already call these via Abilities directly.

### Import style examples (decision, not exhaustive inventory)

- Combined Guild import `use crate::guilds::{FooGuild, use_skill}` → split into `use crate::guilds::FooGuild` + `use crate::abilities::use_skill`.
- Separate `use crate::guilds::cast_spell` → `use crate::abilities::cast_spell`.
- Files without an existing Abilities import add only the needed named import(s).

### Test migration

- **Delete** both wrapper tests from the Guild root module (`use_skill_builds_targeted_commands`, `cast_spell_builds_targeted_commands`).
- **Add** `cast_spell_matches_targeted_form` in Abilities tests — symmetric with existing `use_skill_matches_targeted_form` (empty args → `@cast '<name>'`; with args → `@target <t>;cast '<name>' <t>`).
- Do **not** rewire deleted Guild tests to call Abilities through Guild; coverage belongs in Abilities.

### Domain documentation (`CONTEXT.md`)

- Add an **Abilities** bounded-context section (~3–5 sentences):
  - Owns canonical BatMUD `use`/`cast` command-line formatting.
  - Guild command modules consume it for `use_skill` / `cast_spell` and related helpers.
  - Command Dispatch does not own this concern.

### Engineering wiki

- **Create** new concept page `docs/wiki/concepts/abilities.md`:
  - Standard concept frontmatter (`title`, `type: concept`, `status: current`, `updated`, `sources` including `CONTEXT.md`, Abilities module source, and this feature PRD).
  - **Summary** — Abilities bounded context for `use`/`cast` line formatting.
  - **Verified Facts** — implementation location; primary exports and their roles; **targeted-command formatting rules**:
    - Empty target args → `use '<skill>'` / `cast '<spell>'` (logical line; `client_send_line` adds `@`).
    - Non-empty target args → `target <t>;use '<skill>' <t>` / `target <t>;cast '<spell>' <t>`.
  - **Other exports** — name + one-line purpose inventory for secondary helpers (`cast_quoted_tail`, `use_quoted_tail`, `cast_quoted_with_suffix`, `use_quoted_with_suffix`, `compound_send`, `repeat_inf_cast_heal_self`, `floating_disc`); no full API reference.
  - **Related** — link to Guild Catalog, Command Dispatch boundary note (dispatch does not own this), `CONTEXT.md`.
- **Update** `docs/wiki/index.md` — add Abilities concept entry.
- **Update** `docs/wiki/path-map.json` — map `src/abilities/**` → `docs/wiki/concepts/abilities.md`.
- **Update** `docs/wiki/concepts/guild-catalog.md`:
  - Add Related link to Abilities.
  - Add Verified Facts bullet: guild command modules import `use_skill` / `cast_spell` from Abilities, not Guild.
- **No change** to `docs/wiki/concepts/command-dispatch.md`.
- **Append** `ingest` entry to `docs/wiki/log.md`.

### Player/manual documentation

- **No change** to `docs/guilds/*.md` — function names and player-visible behavior are unchanged; module path is an implementation detail.

### Delivery

- **One atomic pull request** — code, tests, PRD, `CONTEXT.md`, wiki pages, index, path-map, and wiki log together.
- No slice tickets; single PR from this spec.
- No ADR — naming an existing module boundary, not a reversible architectural trade-off.

## Testing Decisions

### What makes a good test

- Test **external behavior** — formatted command strings sent to the client — not import paths or module structure.
- Assert targeted-command rules at the **Abilities public API** (`use_skill`, `cast_spell`) for both empty and non-empty `Data.args`.
- Do not duplicate Abilities formatting assertions in Guild module tests after wrappers are removed.
- Existing lower-level Abilities tests (`targeted_use`, `targeted_cast`, `client_send_line`, suffix helpers, `compound_send`) remain sufficient for non-primary-path helpers; no new tests required for secondary exports unless a regression is found.

### Test seam

- **Primary seam:** Abilities module unit tests — highest seam covering `use_skill` / `cast_spell` public behavior in one place.
- Guild command modules: no new tests; mechanical import swap only. Full `cargo test` gate confirms no regressions across all guild callers.

### Prior art

- Existing `use_skill_matches_targeted_form` in Abilities tests.
- Existing `targeted_use_and_cast` and `quoted_tails_and_suffix_helpers` in Abilities tests.
- Guild wrapper tests being deleted (were testing Abilities behavior through the wrong module).

## Out of Scope

- Restructuring the guild module hierarchy or the `Guild` trait.
- Adding new command formatting functions or changing formatting rules.
- Changing secondary Abilities helper implementations.
- Updating per-guild player manual pages (`docs/guilds/*.md`).
- Updating Command Dispatch wiki page.
- Full API reference documentation for every Abilities export.
- Slice tickets or multi-PR delivery.
- ADR authorship.

## Further Notes

- Caller count verified at twenty-four guild command modules (import-only changes; no other callers of Guild re-exported `use_skill` / `cast_spell`).
- Guild root module shrinks by the two wrapper functions and their tests (~10 lines of implementation plus test code).
- Wiki `sources` frontmatter should cite `CONTEXT.md`, Abilities module source, and this PRD.
- Label: `ready-for-agent`.
