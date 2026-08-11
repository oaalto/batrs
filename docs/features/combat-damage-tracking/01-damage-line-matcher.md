# 01 — Damage line matcher

## Parent

`prd.md`

## What to build

Compile the battle-listen melee catalog from `hit_messages.md` at build time and implement the incoming-damage line matcher: skills (bash, push, kick, stab, scythe swipe), spells (`A/An <name> hits you.`), then melee catalog suffix scan with longest-verb-wins and weapon-family recency. Expose `Matcher::match_incoming(line) → Option<DamageMatch>` with `damage_category`, `message_verb`, `source_name`, and `message_text`. Outgoing `You <verb> <target>` lines must not match.

## Blocked by

None — can start immediately.

## Status

done

## Acceptance criteria

- [x] `build.rs` parses `hit_messages.md` into a generated catalog (11 families, 286 verbs); compile fails on malformed catalog.
- [x] Conjugation: `+s`/`+es` on last word; dual suffix (conjugated then bare); ALL-CAPS handling.
- [x] Matcher order: skills → spells → melee.
- [x] Skill patterns: bash, push, kick (3 lines), stab (4 lines), scythe swipe — each with correct `message_verb` and `source_name` capture.
- [x] Spell regex: `^An? (.+) hits you\.$` with `message_verb` from capture; `source_name` empty.
- [x] Melee: longest verb wins; family recency reorders search; case-insensitive; incoming template ends with ` you.`.
- [x] Outgoing catalog lines (`You <verb> <target>`) do not match `match_incoming`.
- [x] Unit tests: full catalog synthetic incoming (286 verbs); per-family samples; conjugation edge cases; Holy-man fight lines, kick variants, spell examples; longest-match and push-skill-vs-monk-melee edge cases.
- [x] `cargo test combat_damage` passes; clippy clean.

## Tests (maintain)

- [x] Full catalog synthetic incoming (286 verbs) + outgoing sanity per verb.
- [x] Per-family sample lines (11 families).
- [x] Conjugation: `+s`, `+es`, multi-word, ALL-CAPS.
- [x] Inline examples: Holy-man fight lines, kick variants, spell hit lines.
- [x] Edge: longest-match (`heavily bash` vs `bash`), push skill vs monk melee, case-insensitive.
- [x] When `hit_messages.md` changes, catalog count test must still pass (add if missing).

## Testing seam

**`Matcher::match_incoming(&str)`** — single-line in, `DamageMatch` out. No database or application shell required.
