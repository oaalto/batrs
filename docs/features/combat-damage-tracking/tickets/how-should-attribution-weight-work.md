---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

How should **attribution weight** be computed and used?

User intent: a hit/spell/skill line alone between two HP status messages should score **higher** than when many lines sit in the buffer.

Decide:

- Weight scale (e.g. 0.0–1.0 float, 1–100 int, or discrete tiers: high/medium/low)
- Counting rules: what lines count toward "crowding" (gags, scan output, skill concentration lines, round headers)?
- Single candidate vs multiple: if exactly one incoming-damage candidate line in the buffer, is weight maximal?
- Multiple candidates: split `hp_delta` evenly, assign all weight to the strongest match, or store unattributed remainder?
- Should the HTTP viewer **filter or sort** by weight threshold, or only display it as a column?
- Relationship to misses: nearby `misses` lines lower weight or irrelevant?

### Q1 — scale and purpose (resolved)

**Decision:** `0.0`–`1.0` float on every row.

**Purpose (revised):** weight reflects how much of the observed `hp_delta` can be attributed to this specific line — not generic buffer noise. Crowding is measured **after filtering**: only recognized incoming-damage candidate lines (melee/skill/spell matchers) count; unrelated lines (misses, outgoing hits, scan, `Hp:` prompts, round headers, concentration, etc.) are ignored for weight calculation.

**Goal:** over many events, accumulate per-message **min/max damage estimates** (e.g. `bitchslaps`, `bash`, `A magic missile hits you.`) — isolated observations (one candidate between status lines) tighten the range fastest; crowded buffers widen it.

### Q2 — single vs multiple candidates (resolved)

**Decision:** treat **single-candidate** and **multi-candidate** buffers as separate confidence classes — not one blended stream.

| Class | Condition | Rows | `weight` | Per-line bounds from this event |
|-------|-----------|------|----------|----------------------------------|
| **Isolated** | exactly 1 filtered candidate before `H:` loss | 1 row, full `hp_delta` | `1.0` | min = max = `hp_delta` |
| **Ambiguous** | 2+ filtered candidates, one `hp_delta` | 1 row per candidate, each with full `hp_delta` | `1.0 / N` | each line: min `0`, max `hp_delta` |

**Storage:** add `candidate_count` (integer) on every row — count of filtered incoming-damage lines in that buffer window.

**Extrapolation:** multi-candidate bounds are refined using per-verb ranges learned from isolated events (e.g. if `bitchslaps` is consistently ~22 from isolated rows, an ambiguous row containing `bitchslaps` + another hit constrains the split). Algorithm for v1 deferred to Q3.

**Viewer (cross-ticket note):** HTTP dashboard should expose at least two aggregate modes — **confirmed** (isolated/`candidate_count = 1` only) and **estimated** (includes ambiguous rows, with extrapolation applied). See weight ticket resolution + viewer ticket.

### Q3 — extrapolation from ambiguous rows (resolved)

**Decision:** conservative constraint pass for the **estimated** view only; no even-split fallback.

1. **Per verb, from isolated rows only** (`candidate_count = 1`): compute `known_min`, `known_max`, `known_avg` (simple min/max/mean of `hp_delta`).
2. **For each ambiguous row:** start with each candidate's bounds `[0, hp_delta]`.
3. **Greedy constraint pass:** if sum of `known_min` for all candidates exceeds `hp_delta`, cap or flag; if one candidate's `known_min` alone equals `hp_delta`, assign full delta to that line and `0` to others.
4. **Otherwise:** leave bounds loose `[0, hp_delta]` — no guess beyond what isolated data proves.

**Confirmed view** uses step 1 only (isolated events). **Estimated view** applies steps 2–4 at read/aggregate time (not stored per-row beyond raw bounds).

### Q4 — aggregation key for per-message bounds (resolved)

**Decision:** roll up isolated observations by **`damage_category` + `message_verb`**, with category-specific key rules:

| Category | `message_verb` key | Notes |
|----------|-------------------|-------|
| `melee` | catalog verb only | e.g. `bitchslaps`, `lightly strikes` — merged across all source monsters |
| `skill` | one key per skill | e.g. `bash`, `push`, `kick` — multiple line patterns (regexps) map to the same key |
| `spell` | one key per spell | e.g. `magic missile`, `fire blast` — extracted from `A/An <name> hits you.` |

Extrapolation and confirmed/estimated aggregates use this key; `source_name` is drill-down metadata only, not part of the bounds rollup.

### Q5 — per-row bounds at insert time (resolved)

**Decision:** persist **`damage_min`** and **`damage_max`** on every row at insert time.

| Class | `damage_min` | `damage_max` |
|-------|--------------|--------------|
| Isolated (`candidate_count = 1`) | `hp_delta` | `hp_delta` |
| Ambiguous (`candidate_count > 1`) | `0` | `hp_delta` |

Extrapolation (Q3) runs at aggregate/read time for the **estimated** view — not rewritten back into stored rows.

## Resolution

**Weight scale:** `0.0`–`1.0` float. Derived from candidate class: `1.0` when isolated, `1.0 / N` when ambiguous (`N = candidate_count`).

**Crowding:** unrelated lines (misses, outgoing hits, scan, `Hp:` prompts, round headers, concentration, gags, etc.) are **ignored** before counting candidates. Only recognized incoming-damage matcher lines count.

**Candidate classes:**

| Class | Condition | Rows | `weight` | `damage_min` / `damage_max` |
|-------|-----------|------|----------|----------------------------|
| Isolated | exactly 1 filtered candidate | 1 | `1.0` | both = `hp_delta` |
| Ambiguous | 2+ filtered candidates | 1 per candidate | `1.0 / N` | `0` / `hp_delta` each |

**Row fields added (beyond event ticket):** `candidate_count`, `weight`, `damage_min`, `damage_max`.

**Aggregation key:** `damage_category` + `message_verb` — melee grouped by catalog verb; each skill one key (multi-regexp); each spell one key.

**Extrapolation (estimated view only):** conservative constraint pass using isolated-derived `known_min`/`known_max` per key; no even-split fallback; loose `[0, hp_delta]` when constraints don't resolve.

**Viewer (cross-ticket):** **confirmed** mode = isolated rows only; **estimated** mode = all rows with extrapolation applied at read time.

## Addendum — `weight` is not `catalog_rank`

**Attribution `weight`** (`1.0` or `1.0/N`) measures how confidently a filtered candidate line caused the observed `hp_delta` given buffer crowding. It is **not** melee hit severity and must not be replaced by catalog position.

**`catalog_rank`** (1–26 per weapon family from [`hit_messages.md`](../../hit_messages.md) line order) is a separate axis: expected damage severity for melee verbs. It informs **estimated avg** in the read-time extrapolation path when ambiguous bounds stay loose — see [How should catalog rank inform estimated extrapolation?](how-should-catalog-rank-inform-estimated-extrapolation.md).

| Field | Meaning | Used for |
| --- | --- | --- |
| `weight` | Attribution confidence | Drill-down column; not severity |
| `catalog_rank` | Catalog severity ordinal (melee only) | Rank-estimated avg when bounds unresolved |
