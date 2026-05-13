//! "Analysis of magic lore" combat-line highlighting shared by guilds (Civmage, Mage).

use crate::ansi::{StyledLine, TextStyle};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use unicode_segmentation::UnicodeSegmentation;

/// Paints tiered damage-reaction text on `styled_line` when `line` matches an analysis pattern.
/// Returns `true` when a rule matched.
pub fn paint_magic_lore_analysis(styled_line: &mut StyledLine, line: &str) -> bool {
    let tiers: &[(&Regex, TextStyle)] = &[
        (&ANALYSIS_SCREAMS, TextStyle::GREEN),
        (&ANALYSIS_WRITHES, TextStyle::BLUE),
        (&ANALYSIS_SHUDDERS, TextStyle::CYAN),
        (&ANALYSIS_GRUNTS, TextStyle::YELLOW),
        (&ANALYSIS_WINCES, TextStyle::MAGENTA),
        (&ANALYSIS_SHRUGS, TextStyle::RED),
    ];
    for (regular_expression, style) in tiers {
        if let Some(captures) = regular_expression.captures(line) {
            apply_capture_hilite(styled_line, &captures, 2, *style);
            return true;
        }
    }
    false
}

fn apply_capture_hilite(
    styled_line: &mut StyledLine,
    captures: &Captures<'_>,
    index: usize,
    style: TextStyle,
) {
    let Some(matched) = captures.get(index) else {
        return;
    };

    let start = byte_to_grapheme_index(&styled_line.plain_line, matched.start());
    let end = byte_to_grapheme_index(&styled_line.plain_line, matched.end());
    let length = styled_line.styled_chars.len();
    let start = start.min(length);
    let end = end.min(length);

    for grapheme_index in start..end {
        styled_line.styled_chars[grapheme_index].color = style.color;
        styled_line.styled_chars[grapheme_index].bold = style.bold;
    }
}

fn byte_to_grapheme_index(textual: &str, byte_index: usize) -> usize {
    textual
        .get(..byte_index)
        .map(|slice| slice.graphemes(true).count())
        .unwrap_or_default()
}

lazy_static! {
    static ref ANALYSIS_SCREAMS: Regex = Regex::new(r"(.+) (screams in pain\.)").unwrap();
    static ref ANALYSIS_WRITHES: Regex = Regex::new(r"(.+) (writhes in agony\.)").unwrap();
    static ref ANALYSIS_SHUDDERS: Regex =
        Regex::new(r"(.+) (shudders from the force of the attack\.)").unwrap();
    static ref ANALYSIS_GRUNTS: Regex = Regex::new(r"(.+) (grunts from the pain\.)").unwrap();
    static ref ANALYSIS_WINCES: Regex =
        Regex::new(r"(.+) (winces a little from the pain\.)").unwrap();
    static ref ANALYSIS_SHRUGS: Regex = Regex::new(r"(.+) (shrugs off the attack\.)").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn analysis_colors_second_capture_green() {
        let line = "The orc screams in pain.";
        let mut styled = StyledLine::new(line);
        assert!(paint_magic_lore_analysis(&mut styled, line.trim()));
        let pain_start = "The orc ".len();
        let pain_graphemes = "screams in pain.".graphemes(true).count();
        for offset in 0..pain_graphemes {
            let index = byte_to_grapheme_index(styled.plain_line.as_str(), pain_start) + offset;
            let cell = &styled.styled_chars[index];
            assert_eq!(cell.color, AnsiCode::Green);
        }
    }
}
