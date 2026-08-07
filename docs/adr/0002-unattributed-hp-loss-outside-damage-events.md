# Unattributed HP loss stored outside damage_events

Negative `H:` HP loss with zero recognized damage candidates does not create `damage_events` rows and does not use a `damage_category = unknown` label. Instead, batrs writes one `unattributed_hp_events` row (schema v4) with an ordered JSON `context_lines` array — every plain line between the previous and triggering `H:` lines.

Attribution aggregates (confirmed and estimated) remain `candidate_count ≥ 1` only. Melee rollups already use `weapon_family = unknown` for a different meaning; a third “unknown” sense would confuse dashboards. The separate table keeps matcher-gap review (environmental, DoT, guild specials) available without polluting per-verb damage statistics. Ambiguous batches (N≥2 candidates) stay on the existing `damage_events` path; context windows are not saved when attribution succeeds.
