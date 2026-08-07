---
type: grilling
status: closed
claimed_by: oaalto
blocked_by: []
parent: map.md
---

## Question

When an `H:` line shows negative HP loss but **no** recognized damage candidate exists in the attribution window, should batrs discard the loss, write an `unknown` `damage_events` row, or capture it elsewhere for review?

Pin down:

- Trigger boundary (zero candidates only vs ambiguous batches)
- What lines constitute the saved window
- Storage shape and schema version
- Viewer surface
- Relationship to v1 “skip unattributed” decision

## Progress

### Q1 — trigger boundary (resolved)

**Decision:** only **zero-candidate** negative `H:` triggers. Ambiguous batches (N≥2 recognized candidates) stay on the existing `damage_events` path with fractional `confidence`/`weight`. Mixing the two would blur “unmatched pattern” vs “crowded attribution.”

### Q2 — context window contents (resolved)

**Decision:** **context window** — every plain line passed to `DamageCollector::handle_line` while logged in between the previous `H:` and the triggering `H:` (exclusive of both `H:` lines). Includes misses, outgoing hits, round headers, and gagged scan lines (collector runs before combat-awareness gag). Empty context is valid.

### Q3 — storage (resolved)

**Decision:** separate table in `~/.batrs/combat_damage.db` (schema **v4** migration):

- `unattributed_hp_events`: one row per trigger (`recorded_at`, `player`, `hp_delta`, `hp_before`, `hp_after`, `h_line_text`, `context_lines` JSON array)
- **Not** `damage_events` with `damage_category = unknown` (melee rollups already use `weapon_family = unknown` for a different meaning)

One transaction per trigger; indexes on `recorded_at` and `player`. No log-file correlation in v1 — store verbatim context in DB.

### Q4 — lifecycle and failure (resolved)

**Decision:** context window clears on every `H:` line, `reset_buffer()`, logout, and `FreshSessionReset::DamageCollector` — same as candidate buffer. Write failure: `tracing::warn!`, discard pending context, continue play.

### Q5 — viewer (resolved)

**Decision:** HTTP viewer **Unattributed HP loss** section — trigger table (`recorded_at`, `player`, `hp_delta`, line count); drill-down shows ordered `context_lines` plus `h_line_text`. Same `range`/`player` filters as attribution dashboard. Always on when collector is active.

### Q6 — relationship to attributed rows (resolved)

**Decision:** when N≥1 candidates exist, **only** `damage_events` rows are written — no parallel context capture. `message_text` on attributed rows remains the single attributed line only.

## Resolution

**Unattributed HP loss** is captured when a negative `H:` short-score line arrives with zero recognized damage candidates since the previous `H:` line.

- Writes **no** `damage_events` rows.
- Writes one `unattributed_hp_events` row with ordered `context_lines` (JSON) for later pattern discovery.
- Does **not** supersede attributed-batch semantics; extends v1 “skip unattributed” to “skip attribution rows, add review capture.”

**Not stored in this path:** ambiguous multi-candidate batches, `unknown` damage category, round/session ids, per-line timestamps in context.

**Implementation:** slice `04-unattributed-hp-review.md`.
