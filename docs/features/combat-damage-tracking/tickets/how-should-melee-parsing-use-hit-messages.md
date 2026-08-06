---
type: task
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

How should melee parsing **compile** [`docs/hit_messages.md`](../../hit_messages.md) into runtime matchers?

Deliverable for this ticket (planning artifact, not production code):

- Proposed line templates for incoming (`<name> <verb> you.`) and outgoing (`You <verb> <target>.`) sanity checks
- Strategy: one regex per weapon family, verb alternation from catalog, or parse verb token after actor prefix
- Case sensitivity (`BRUTALLY TEAR`, multi-word verbs like `lightly strikes`)
- Generated test cases: at least one line per weapon family from the catalog plus Holy-man fight examples (inline in matcher tests)

### Q1 — matching strategy (resolved)

**Decision:** flat catalog suffix scan (option A), with weapon-family recency optimization.

1. **Compile** `hit_messages.md` at build time into a flat verb list grouped by weapon family (11 families: slash, bash, pierce, shield, whip, unarmed, tiger, monk, bite, claw, breath).
2. **Default order:** all verbs sorted **longest-first** within the full catalog (so `lightly strike` wins over `strike`).
3. **Recency optimization:** track last matched weapon family; on next incoming melee match, try that family's verbs first (still longest-first within the family), then fall back to the global longest-first list. Monsters tend to use one damage type per fight — this avoids scanning unrelated families on most hits.
4. **Incoming template:** line ends with ` you.` — try suffix `{verb}s you.` then `{verb} you.` (case-insensitive); first hit → `message_verb` = catalog form (canonical), prefix = `source_name`.
5. **Outgoing sanity check only:** prefix `You ` + verb + ` ` + target — not stored as damage events; used in tests.
6. **Case:** match case-insensitively; store `message_verb` in catalog canonical form (preserve ALL-CAPS catalog entries like `BRUTALLY TEAR`).

### Q2 — third-person conjugation (resolved)

**Decision:** precomputed **dual suffix** per catalog verb at compile time (option A).

1. Conjugate last word of multi-word verbs (`lightly strike` → `lightly strikes`).
2. Conjugation rule: last word ends in `s`, `x`, `z`, `ch`, `sh` → add `es`; else add `s`.
3. Try **conjugated** suffix first (`{conjugated} you.`), then **bare** catalog (`{catalog} you.`) — covers `breath lightly you.` and irregular forms that skip `+s`.

### Q3 — catalog compilation (resolved)

**Decision:** `build.rs` generates the catalog module (option A).

1. Parse `docs/hit_messages.md` at compile time → emit `combat_damage/catalog.rs` (or equivalent) with static verb lists grouped by weapon family.
2. Each verb entry includes: canonical form, weapon family id, precomputed conjugated form, dual suffix strings.
3. Compile fails if catalog is malformed. `hit_messages.md` stays the human-edited source; generated file is a build artifact.
4. Zero runtime md parsing; recency optimization uses typed family index per verb.

### Q4 — matcher order (resolved)

**Decision:** check order is **skills → spells → melee**.

1. **Skills** — specific regexes (bash, push, kick, stab, scythe swipe); longest/most-specific first within the skill table.
2. **Spells** — `^An? (.+) hits you\.$`
3. **Melee** — catalog suffix scan with family recency optimization.
4. **No match** — line not added to candidate buffer.

Outgoing lines (`You puncture Holy man.`) fail all three — never buffered. Skills and spells win over the broad melee suffix scan (e.g. stab lines like `You watch helplessly as Akeem smashes your kneecap!`).

### Q5 — test fixtures (resolved)

**Decision:** comprehensive test suite in `src/combat_damage/`:

| Layer | Coverage |
|-------|----------|
| **Build** | `build.rs` parses `hit_messages.md`; asserts 11 families × 26 verbs |
| **Conjugation** | `+s`/`+es`, multi-word, ALL-CAPS last word |
| **Catalog** | every one of 286 verbs matches synthetic incoming (melee-only path); one sample per weapon family |
| **Outgoing** | sanity check for all 286 catalog verbs |
| **Fixtures** | Holy-man fight (10 damage lines), kick variants, spell examples, skill patterns — inline in matcher tests |
| **Edge cases** | longest-match beats embedded shorter verbs; push skill vs monk melee `push`; comma phrases; case-insensitive |

Implementation: `build.rs` → generated catalog; `Matcher` with skills → spells → melee order; longest verb wins on ties.

## Resolution

**Compile:** `build.rs` parses [`docs/hit_messages.md`](../../hit_messages.md) → `OUT_DIR/combat_damage_catalog.rs` with `CatalogEntry` (canonical, family, conjugated_suffix, bare_suffix). Compile fails on malformed catalog.

**Match incoming melee:** line ends with ` you.`; dual suffix try (conjugated then bare), case-insensitive; **longest matching verb wins** across catalog; family recency reorders search (recency family first, then global longest-first).

**Conjugation:** last word `+s`/`+es`; ALL-CAPS words get ALL-CAPS suffix.

**Order with skills/spells:** skills → spells → melee (see non-melee ticket). `match_melee_for_test` available for catalog-only tests.

**Outgoing sanity:** `You <verb> <target>.` — test-only validation, not damage events.

**Tests:** 35 unit tests in `combat_damage` module covering full catalog, fixtures, and edge cases.
