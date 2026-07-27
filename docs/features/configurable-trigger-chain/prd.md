# Configurable Trigger Chain

## Problem

`src/triggers/mod.rs` has a hardcoded trigger processing pipeline in the `process()` function:

```rust
pub fn process(facts: &TriggerFacts, guilds: &[Box<dyn Guild>], line: &str) -> TriggerEffects {
    let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();
    let mut current_line = StyledLine::new(line);
    let mut output = TriggerEffects::default();

    // 1. Guild triggers
    for trigger in guild_triggers.iter() { ... }

    // 2. Spell vocals
    let result = spell_vocals::trigger(...);

    // 3. Common triggers (from common.rs)
    for trigger in COMMON_TRIGGERS.iter() { ... }

    // 4. Core triggers
    for trigger in CORE_TRIGGERS.iter() { ... }

    output
}
```

The pipeline is fixed:
- Guild triggers always run first
- Spell vocals always run second
- Common rules always run third
- Core rules (prompt, short_score, recovery_bracket) always run last

There is no way to:
- Disable a trigger group (e.g., disable common rules while debugging)
- Change execution order
- Add custom trigger groups
- Skip trigger groups conditionally

This limits testing flexibility and prevents power users from customizing their experience (e.g., disabling certain highlights, adding custom rule groups, or reordering for specific workflows).

## Goals

- Allow enabling/disabling trigger groups via configuration
- Keep the default pipeline order (guilds → spell_vocals → common → core)
- Make it easy to add new trigger groups in the future
- Support per-guild trigger control (already partially exists via the Guild trait)

## Non-goals

- Runtime (hot) reconfiguration of the trigger chain — config is loaded at startup
- Adding new built-in trigger groups
- Changing the `Trigger` function signature
- A GUI editor for trigger groups — TOML config only

## Proposed Architecture

### Configuration

Add a `TriggerConfig` to `PlayerToml` (or `UserSettings`):

```rust
pub struct TriggerConfig {
    pub guild_triggers: bool,          // default: true
    pub spell_vocals: bool,            // default: true
    pub common_triggers: bool,         // default: true
    pub core_triggers: bool,           // default: true
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            guild_triggers: true,
            spell_vocals: true,
            common_triggers: true,
            core_triggers: true,
        }
    }
}
```

TOML representation in `player.toml`:

```toml
[triggers]
guild_triggers = true
spell_vocals = true
common_triggers = true
core_triggers = true
```

### Runtime integration

Pass `TriggerConfig` into `process()`:

```rust
pub fn process(facts: &TriggerFacts, guilds: &[Box<dyn Guild>], line: &str, config: &TriggerConfig) -> TriggerEffects {
    let mut output = TriggerEffects::default();
    let mut current_line = StyledLine::new(line);

    if config.guild_triggers {
        for trigger in guild_triggers.iter() { ... }
    }

    if config.spell_vocals {
        let result = spell_vocals::trigger(...);
        result.apply_line_effects_to(&mut current_line);
        output.extend(result);
    }

    if config.common_triggers {
        for trigger in COMMON_TRIGGERS.iter() { ... }
    }

    if config.core_triggers {
        for trigger in CORE_TRIGGERS.iter() { ... }
    }

    output
}
```

The caller (`BatApp` or wherever `process()` is invoked) constructs the `TriggerConfig` from the player profile and passes it in.

### Extensibility: trigger group abstraction

For future-proofing, introduce a lightweight trigger group trait:

```rust
pub trait TriggerGroup {
    fn name(&self) -> &'static str;
    fn process(&self, line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects;
}
```

Then `process()` becomes a loop over configured groups:

```rust
let groups: Vec<Box<dyn TriggerGroup>> = {
    let mut groups = Vec::new();
    if config.guild_triggers { groups.push(Box::new(GuildTriggerGroup)); }
    if config.spell_vocals { groups.push(Box::new(SpellVocalsGroup)); }
    if config.common_triggers { groups.push(Box::new(CommonTriggerGroup)); }
    if config.core_triggers { groups.push(Box::new(CoreTriggerGroup)); }
    groups
};

for group in &groups {
    let result = group.process(&TriggerLine::new(&current_line.plain_line), facts);
    result.apply_line_effects_to(&mut current_line);
    output.extend(result);
}
```

This is a longer-term direction; the immediate implementation can skip the trait and just use if-gates on the existing function calls.

## Migration Plan

1. Add `TriggerConfig` struct to `src/triggers/mod.rs`
2. Add `triggers` section to `PlayerToml` config with default values
3. Update `process()` signature to accept `&TriggerConfig`
4. Update all call sites of `process()` to pass `TriggerConfig::default()` (preserves existing behavior)
5. Wire the config from the player profile into the call site
6. Run `cargo test` to verify no behavioral changes

## Success Criteria

- `process()` respects `TriggerConfig` — disabling a group skips it entirely
- Default config produces identical behavior to current code
- All existing tests pass
- Config is deserialized from TOML with sensible defaults

## Risks

- **Performance**: The if-gate approach adds a few branch predictions per process call. Negligible — line processing is already dominated by regex matching.
- **Call site updates**: Every caller of `process()` needs the config. Currently the trigger pipeline is called from one place (the event processor). Verify this is the case.
- **Testing**: Existing tests call `process()` directly. They'll need to pass `TriggerConfig::default()` or the tests should be updated to a helper.
