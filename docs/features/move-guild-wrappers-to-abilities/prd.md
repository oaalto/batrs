# Move Guild Command Wrappers to Abilities Module

## Problem

`src/guilds/mod.rs` (633 lines) contains:
- The `Guild` trait definition
- 30+ guild implementation modules (aelena, animist, barbarian, channellers, civmage, ...)
- Two forwarding functions that delegate to `crate::abilities`:

```rust
pub fn use_skill(skill_name: &str, data: &Data) -> String {
    crate::abilities::use_skill(skill_name, data)
}

pub fn cast_spell(spell_name: &str, data: &Data) -> String {
    crate::abilities::cast_spell(spell_name, data)
}
```

These two functions are pure pass-throughs — they add zero logic. They exist because guild modules call them (e.g., a guild command might do `guilds::use_skill("scythe swipe", &data)`). This creates a circular dependency: `abilities` exports the canonical implementation, and `guilds` re-exports it.

`src/abilities/mod.rs` (107 lines) already has the canonical implementations:
```rust
pub fn use_skill(skill_name: &str, data: &Data) -> String { ... }
pub fn cast_spell(spell_name: &str, data: &Data) -> String { ... }
```

Having duplicate function names across modules creates confusion about which is the "real" implementation and adds maintenance overhead (if the implementation changes, both copies must be kept in sync — though currently guilds just delegates so there's no risk of drift).

## Goals

- Remove the pass-through functions from `guilds/mod.rs`
- Callers go directly to `crate::abilities::use_skill` / `crate::abilities::cast_spell`
- Reduce `guilds/mod.rs` by ~10 lines
- Eliminate the circular re-export confusion

## Non-goals

- Restructuring the guild module hierarchy
- Adding new command formatting functions
- Changing the `Guild` trait
- Modifying any guild implementation modules

## Proposed Architecture

### Before
```
guilds/mod.rs:
    pub fn use_skill(skill_name: &str, data: &Data) -> String {
        crate::abilities::use_skill(skill_name, data)
    }
    pub fn cast_spell(spell_name: &str, data: &Data) -> String {
        crate::abilities::cast_spell(spell_name, data)
    }

Callers: guilds::use_skill("scythe swipe", &data)
```

### After
```
No change to abilities/mod.rs.

Callers: abilities::use_skill("scythe swipe", &data)
```

## Migration Plan

1. Find all callers of `guilds::use_skill` and `guilds::cast_spell`
2. Replace each with `crate::abilities::use_skill` / `crate::abilities::cast_spell` (or `abilities::use_skill` depending on import scope)
3. Remove the two pass-through functions from `guilds/mod.rs`
4. Run `cargo test` to verify nothing breaks

## Success Criteria

- No callers remain that go through `guilds::use_skill` or `guilds::cast_spell`
- `guilds/mod.rs` no longer contains the two forwarding functions
- All existing tests pass
- Build succeeds with no warnings

## Risks

- **Caller count**: Need to verify how many callers exist. If dozens of guild modules use these, the search-and-replace is mechanical but needs care to get the module path right for each caller's location.
- **Test code**: The tests in `guilds/mod.rs` already use `use_skill` and `cast_spell` directly (not through the module path), so they should be unaffected.
