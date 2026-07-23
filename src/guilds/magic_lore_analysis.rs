//! "Analysis of magic lore" combat-line highlighting shared by guilds (Civmage, Mage).

#[cfg(test)]
use crate::ansi::StyledLine;
use crate::ansi::TextStyle;
use crate::triggers::LineEffect;
use regex::Regex;
use std::sync::LazyLock;
#[cfg(test)]
use unicode_segmentation::UnicodeSegmentation;

/// Returns a tiered damage-reaction style effect when `line` matches an analysis pattern.
pub fn magic_lore_analysis_effect(line: &str) -> Option<LineEffect> {
    let tiers: &[(&Regex, TextStyle)] = &[
        (&ANALYSIS_SCREAMS, TextStyle::GREEN),
        (&ANALYSIS_WRITHES, TextStyle::BLUE),
        (&ANALYSIS_SHUDDERS, TextStyle::CYAN),
        (&ANALYSIS_GRUNTS, TextStyle::YELLOW),
        (&ANALYSIS_WINCES, TextStyle::MAGENTA),
        (&ANALYSIS_SHRUGS, TextStyle::RED),
    ];
    for (regular_expression, style) in tiers {
        if let Some(captures) = regular_expression.captures(line)
            && let Some(matched) = captures.get(2)
        {
            return Some(LineEffect::StylePlainByteRange {
                range: matched.range(),
                style: *style,
            });
        }
    }
    None
}

/// Paints tiered damage-reaction text on `styled_line` when `line` matches an analysis pattern.
/// Returns `true` when a rule matched.
#[cfg(test)]
pub fn paint_magic_lore_analysis(styled_line: &mut StyledLine, line: &str) -> bool {
    if let Some(effect) = magic_lore_analysis_effect(line) {
        effect.apply_to(styled_line);
        true
    } else {
        false
    }
}

#[cfg(test)]
fn byte_to_grapheme_index(textual: &str, byte_index: usize) -> usize {
    textual
        .get(..byte_index)
        .map(|slice| slice.graphemes(true).count())
        .unwrap_or_default()
}

static ANALYSIS_SCREAMS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) (screams in pain\.)").unwrap());
static ANALYSIS_WRITHES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) (writhes in agony\.)").unwrap());
static ANALYSIS_SHUDDERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) (shudders from the force of the attack\.)").unwrap());
static ANALYSIS_GRUNTS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) (grunts from the pain\.)").unwrap());
static ANALYSIS_WINCES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) (winces a little from the pain\.)").unwrap());
static ANALYSIS_SHRUGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) (shrugs off the attack\.)").unwrap());

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
