# 01 — Good Religious guild spells end-to-end

## Parent

`prd.md`

## What to build

A player whose guild primary background is **Good Religious** can cast civilization background spells from short aliases — whether or not any other guilds are toggled in `/guilds`. Good Religious is modeled as a normal guild module (Seminary-shaped: commands submodule, empty triggers) and is auto-injected whenever the primary background keyword is `good_religious`. It does not appear as a separate checkbox in the guild drill.

Spell shortcuts and send-line contracts:

| Alias | Bare input | With args |
|-------|------------|-----------|
| `ccs` | Celestial spark, no target | Targeted Celestial spark |
| `clw` | Cure light wounds on self | Cure light wounds on named target |
| `csw` | Cure serious wounds on self | Cure serious wounds on named target |
| `ccw` | Cure critical wounds on self | Cure critical wounds on named target |
| `ccf` | Create food | Create food (args silently ignored) |

When Good Religious is injected alongside other guilds that share an alias (Mage `ccf`, Spider `csw`, Triad `ccw`, Riftwalker `ccs`), Good Religious handlers win because background guilds merge before player-selected guilds. Players whose primary is not Good Religious see no change — generic cures, Mage, Spider, Triad, and Riftwalker shortcuts behave as today.

## Blocked by

None — can start immediately.

## Status

ready-for-agent

## Acceptance criteria

- [ ] Good Religious guild module implements `Guild` with five command handlers matching the PRD send-line contracts (bare and with args); triggers vector is empty
- [ ] Catalog registers `good_religious` as **Good Religious**, thematic Good Religious bucket; entry is buildable for auto-injection but excluded from playable drill toggles and `playable_entries()` browse
- [ ] `GuildSelection::build_guilds` prepends Good Religious when `primary_background_keyword() == "good_religious"`, including when the persisted guild key list is empty; no `good_religious` key required in saved preferences
- [ ] Guild drill and browse do not show Good Religious as a selectable toggle
- [ ] Bare `ccs` sends `cast 'celestial spark'`; `ccs <target>` uses targeted-cast form (`target <t>;cast 'celestial spark' <t>`)
- [ ] `clw`, `csw`, `ccw` default bare target to `me`; args replace the target (generic cure pattern)
- [ ] `ccf` always sends `cast 'create food'`; trailing args produce no client message and no target prefix
- [ ] Civilized (or other non–Good Religious) primary does not register Good Religious aliases on the merged command map
- [ ] When Good Religious and Mage are both in the guild vector, `ccf` resolves to Good Religious (no targeted-cast prefix)
- [ ] When Good Religious and Spider are both in the guild vector, `csw` resolves to Good Religious cure serious wounds
- [ ] When Good Religious and Triad are both in the guild vector, `ccw` resolves to Good Religious cure critical wounds (not cause critical wounds)
- [ ] Per-handler unit tests assert exact `CommandEffect::Send` strings (Seminary `chb` pattern); `build_guilds` injection/ordering test; dispatch or merge test covers at least one alias-precedence case
- [ ] `cargo test --all-targets --all-features` passes; format and clippy gates pass
