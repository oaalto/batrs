# Improve Line Effect Application

## Problem

The line effect system applies visual modifications to game output lines. The current implementation in `src/triggers/mod.rs` has three related concerns tangled together:

1. **`LineEffect` enum** (4 variants) — how to modify a `StyledLine`:
   ```rust
   pub enum LineEffect {
       StyleLine(TextStyle),
       StyleBlock { text: String, style: TextStyle },
       StylePlainByteRange { range: Range<usize>, style: TextStyle },
       InsertPlainAfterPlainByteIdx { byte_idx: usize, suffix: String },
   }
   ```

2. **`TriggerEffects` struct** — the container that collects all effects from a trigger pass:
   ```rust
   pub struct TriggerEffects {
       pub original: OriginalLineEffects,  // edits to the original line
       pub lines: Vec<StyledLine>,         // new lines to emit
       pub actions: Vec<Action>,           // commands to send
       pub stats: Vec<StatsEffect>,        // stat modifications
       pub secondary_status: Vec<SecondaryStatusEffect>,
   }
   ```

3. **`apply_line_effects_to`** — the method that applies `original.edits` to a `StyledLine`:
   ```rust
   pub fn apply_line_effects_to(&self, line: &mut StyledLine) {
       for edit in &self.original.edits {
           edit.apply_to(line);
       }
       if self.original.gag {
           line.gag = true;
       }
   }
   ```

The problems:
- `LineEffect` and `apply_to` are defined in `mod.rs` but conceptually belong with `StyledLine` (in `src/ansi/`)
- `TriggerEffects` mixes original-line modifications with emitted lines and side-effect actions, making it hard to reason about what constitutes "modifying the line" vs "producing output"
- `apply_to` on `LineEffect` reaches into `StyledLine`'s internals via methods like `set_line_style`, `set_block_style`, `set_plain_byte_range_style`, `insert_plain_after_plain_byte_idx` — this creates a coupling where both `LineEffect` and `StyledLine` need to know about each other's API

## Goals

- Move `LineEffect` and its `apply_to` method to the `ansi` module (co-located with `StyledLine`)
- Clarify the distinction between "line edits" (modifications to the original line) and "emitted lines" (new lines produced by a trigger)
- Keep `TriggerEffects` as the trigger-side aggregation container

## Non-goals

- Adding new `LineEffect` variants
- Changing the trigger pipeline in `mod.rs`
- Making line effects configurable
- Adding effect composition/ordering guarantees

## Proposed Architecture

### Move `LineEffect` to `src/ansi/mod.rs` (or `src/ansi/effect.rs`)

```rust
// In src/ansi/ (where StyledLine lives)

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineEffect {
    StyleLine(TextStyle),
    StyleBlock { text: String, style: TextStyle },
    StylePlainByteRange { range: Range<usize>, style: TextStyle },
    InsertPlainAfterPlainByteIdx { byte_idx: usize, suffix: String },
}

impl LineEffect {
    pub fn apply_to(&self, line: &mut StyledLine) {
        match self {
            LineEffect::StyleLine(style) => line.set_line_style(*style),
            LineEffect::StyleBlock { text, style } => line.set_block_style(text, *style),
            LineEffect::StylePlainByteRange { range, style } => {
                line.set_plain_byte_range_style(range.clone(), *style);
            }
            LineEffect::InsertPlainAfterPlainByteIdx { byte_idx, suffix } => {
                line.insert_plain_after_plain_byte_idx(*byte_idx, suffix);
            }
        }
    }
}
```

### Update `src/triggers/mod.rs`

- Remove `LineEffect` enum and `apply_to` impl
- Re-export from ansi: `pub use crate::ansi::LineEffect;`
- `OriginalLineEffects` and `TriggerEffects` stay in `triggers/mod.rs` since they're trigger-domain types

### Updated `mod.rs` structure

```rust
// src/triggers/mod.rs

pub use crate::ansi::LineEffect;  // re-exported from ansi

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OriginalLineEffects {
    pub gag: bool,
    pub edits: Vec<LineEffect>,
}

// TriggerEffects unchanged, but apply_line_effects_to now uses the imported LineEffect
impl TriggerEffects {
    pub fn apply_line_effects_to(&self, line: &mut StyledLine) {
        for edit in &self.original.edits {
            edit.apply_to(line);  // LineEffect::apply_to is now in ansi module
        }
        if self.original.gag {
            line.gag = true;
        }
    }
}
```

## Migration Plan

1. Move `LineEffect` enum definition and `impl LineEffect { apply_to }` from `src/triggers/mod.rs` to `src/ansi/mod.rs` (or a new `src/ansi/effect.rs`)
2. Update `src/triggers/mod.rs` to `pub use crate::ansi::LineEffect;`
3. Remove the duplicate `LineEffect` definition from `mod.rs`
4. Update any other files that import `LineEffect` from triggers (grep for imports)
5. Run `cargo test` to verify

## Success Criteria

- `LineEffect` is defined in the `ansi` module alongside `StyledLine`
- No duplicate definitions of `LineEffect`
- All existing tests pass
- No change to behavior — line effects apply identically
- Build succeeds with no warnings

## Risks

- **Import churn**: Other crates/modules may import `LineEffect` from `crate::triggers::LineEffect`. Need to grep all import sites and update them. Since `LineEffect` is `pub` and used in `OriginalLineEffects` and `TriggerEffects`, external consumers exist.
- **Module boundary clarity**: `LineEffect` references `TextStyle` (from ansi) and `StyledLine` (from ansi). Moving it to ansi is correct since all its dependencies are already there. The `triggers` module only needs to re-export it.
- **`apply_to` coupling**: Even after the move, `LineEffect::apply_to` calls `StyledLine` methods. This is an intentional coupling — line effects exist to modify styled lines. Consider whether this should become a method on `StyledLine` instead (e.g., `line.apply_effect(effect)`) for future extensibility.
