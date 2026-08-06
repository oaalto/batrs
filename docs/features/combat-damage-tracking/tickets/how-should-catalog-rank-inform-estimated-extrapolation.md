---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

How should **`catalog_rank`** from [`docs/hit_messages.md`](../../hit_messages.md) improve estimated damage extrapolation when ambiguous batches stay loose after isolated `known_min` constraints?

Decide:

- Relationship to attribution **`weight`** (`1.0` / `1.0/N` confidence — not severity)
- Compile-time vs persisted rank; schema columns for melee rows
- Cross-family rank comparability within one ambiguous batch
- Skills and spells — rank applies or not
- Monotonic-damage assumption vs isolated observation override
- Where rank affects the pipeline (insert bounds vs read-time estimated avg)
- Formula for rank-proportional soft split
- Viewer labeling (confirmed unchanged; estimated avg may use rank)
- Minimal v1 scope

## Resolution

**Purpose:** improve **estimated** attribution when ambiguous batches stay loose after isolated `known_min` constraints — not isolated (confirmed) observations. Confirmed view uses exact isolated `hp_delta` only; rank adds no value there.

**Not `weight`:** keep `weight` as attribution **confidence** per [How should attribution weight work?](how-should-attribution-weight-work.md). Rank reflects expected hit severity from catalog order, not how sure this line caused the HP loss.

**Terminology:**

| Term | Meaning |
| --- | --- |
| `catalog_rank` | Integer `1`–`26` within a weapon family; monotonically non-decreasing expected damage per `hit_messages.md` line order |
| `weapon_family` | Catalog family id (`slash`, `bash`, …) persisted on melee rows for disambiguation |
| Rank-estimated avg | Read-time point estimate `hp_delta × (rankᵢ / Σ rankⱼ)` over ranked candidates in a batch when bounds stay loose |

**Source of truth:** rank is compile-time from numbered line order in `hit_messages.md`, same as verbs today. Extend `build.rs` generation — add `rank: u8` to `CatalogEntry`; no second hand-maintained table.

**Persistence (schema v2):** nullable `catalog_rank` and `weapon_family` on `damage_events` for melee rows only; `NULL` for skill/spell. Persist at insert so aggregate SQL does not re-infer family from verb (four cross-family verb collisions: `whack`, `savagely strike`, etc.).

**Scope:**

- Rank applies to **melee catalog only** — skills/spells are not in `hit_messages.md`.
- Ranks are comparable **within a batch only** as an ordinal prior, not calibrated absolute damage across families or players.
- **Monotonic prior:** file order is authoritative until contradicted by isolated data; isolated `known_min`/`known_max` always override rank.

**Pipeline (conservative extrapolation unchanged):**

1. Exact `known_min == hp_delta` → assign full delta (existing).
2. `sum(known_min) > hp_delta` → loose bounds (existing).
3. Otherwise per-row bounds stay `[0, hp_delta]` at insert (existing).
4. **New:** for **estimated avg only**, when step 3 left bounds loose and batch has ranked melee rows, `estimated_avg = hp_delta × (rankᵢ / Σ rankⱼ)`; unranked candidates in mixed batches share any remaining mass equally; if Σrank = 0, equal split for avg only.

Rank **never** overrides isolated constraints, **never** narrows `damage_min`/`damage_max` on insert, **never** replaces `weight`.

**Viewer:** confirmed columns unchanged (isolated only). Estimated avg may use rank split; label or tooltip when avg came from rank prior not tight bounds. Drill-down: keep `weight` column as attribution confidence; optional `catalog_rank` column for melee.

**Minimal v1:** emit rank in catalog + persist on insert + rank-weighted `estimated_avg` only — no new extrapolation bounds logic, no weight-column change, no cross-verb calibration.

**Cross-links:** [How should attribution weight work?](how-should-attribution-weight-work.md) (weight ≠ rank), [How should melee parsing use hit messages?](how-should-melee-parsing-use-hit-messages.md) (catalog compilation).
