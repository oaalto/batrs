//! Line-level visual edits applied to a [`StyledLine`].
//!
//! `LineEffect` is defined here in `ansi` alongside [`StyledLine`], its mutation target.
//! Trigger code typically imports via [`crate::triggers::LineEffect`] (re-export); the
//! canonical definition is [`crate::ansi::LineEffect`].
//!
//! ## Line edits vs emitted lines
//!
//! Triggers collect two kinds of line output in [`TriggerEffects`]:
//!
//! - **Line edits** — [`TriggerEffects::original`]: [`OriginalLineEffects::edits`]
//!   (`Vec<LineEffect>`) plus [`OriginalLineEffects::gag`]. These mutate the incoming game
//!   line in place when [`TriggerEffects::apply_line_effects_to`] runs during the trigger
//!   pipeline.
//! - **Emitted lines** — [`TriggerEffects::lines`]: new [`StyledLine`] values appended to
//!   output; they are **not** applied through `LineEffect`.
//!
//! [`OriginalLineEffects::gag`] is a sibling flag (not a `LineEffect` variant). It is
//! applied after all `LineEffect` edits by [`TriggerEffects::apply_line_effects_to`].
//!
//! ```
//! use crate::ansi::{LineEffect, StyledLine, TextStyle};
//! use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
//!
//! fn example_trigger(_line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
//!     TriggerEffects::none()
//!         .style_line(TextStyle::GREEN) // pushes LineEffect::StyleLine into original.edits
//!         .gag() // sets original.gag — separate from LineEffect
//!         .emit(StyledLine::new("extra line")) // emitted line, not a LineEffect
//! }
//! ```

use crate::ansi::{StyledLine, TextStyle};
use std::ops::Range;

/// One visual edit to apply to an existing [`StyledLine`].
///
/// Build effects through [`TriggerEffects`] helpers (`style_line`, `style_block`, etc.)
/// or push directly into [`OriginalLineEffects::edits`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineEffect {
    /// Apply a single style to every grapheme on the line.
    StyleLine(TextStyle),
    /// Style every occurrence of `text` as a substring match on [`StyledLine::plain_line`].
    StyleBlock { text: String, style: TextStyle },
    /// Style graphemes whose plain-text UTF-8 bytes fall in `range` (`[start, end)`).
    ///
    /// Indices are **plain-byte** offsets into [`StyledLine::plain_line`], not grapheme
    /// indices. Out-of-range bounds are clamped to the line length.
    StylePlainByteRange {
        range: Range<usize>,
        style: TextStyle,
    },
    /// Insert `suffix` after the plain-byte index `byte_idx`.
    ///
    /// `byte_idx` is a UTF-8 byte offset into [`StyledLine::plain_line`]; it is clamped
    /// to the line end. New graphemes copy style from the grapheme before the insertion
    /// point (or the first grapheme when inserting at the front).
    InsertPlainAfterPlainByteIdx { byte_idx: usize, suffix: String },
}

impl LineEffect {
    /// Apply this effect to `line` by dispatching to the matching [`StyledLine`] mutator:
    /// [`StyledLine::set_line_style`], [`StyledLine::set_block_style`],
    /// [`StyledLine::set_plain_byte_range_style`], or
    /// [`StyledLine::insert_plain_after_plain_byte_idx`].
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::{AnsiCode, TextColor};

    #[test]
    fn style_line_apply_to_sets_line_color() {
        let mut line = StyledLine::new("hello");
        LineEffect::StyleLine(TextStyle::GREEN).apply_to(&mut line);
        assert!(line.styled_chars.iter().all(|c| c.color == AnsiCode::Green));
    }

    #[test]
    fn style_block_apply_to_styles_matching_text() {
        let mut line = StyledLine::new("foo bar");
        LineEffect::StyleBlock {
            text: "foo".to_string(),
            style: TextStyle::RED,
        }
        .apply_to(&mut line);
        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
    }

    #[test]
    fn style_plain_byte_range_apply_to_styles_substring() {
        let mut line = StyledLine::new("alpha beta gamma");
        let start = line.plain_line.find("beta").unwrap();
        let end = start + "beta".len();
        LineEffect::StylePlainByteRange {
            range: start..end,
            style: TextStyle::BRIGHT_RED,
        }
        .apply_to(&mut line);
        assert_eq!(line.styled_chars[6].color, AnsiCode::Red);
        assert!(line.styled_chars[6].bold);
        assert_eq!(line.styled_chars[0].color, TextColor::Default);
    }

    #[test]
    fn insert_plain_after_plain_byte_idx_apply_to_inserts_suffix() {
        let mut line = StyledLine::new("ab");
        LineEffect::InsertPlainAfterPlainByteIdx {
            byte_idx: 1,
            suffix: "X".to_string(),
        }
        .apply_to(&mut line);
        assert_eq!(line.plain_line, "aXb");
    }
}
