//! Sect cultivation and mantra lines shared by Tiger and Monk.

use crate::ansi::TextStyle;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref FINISHED_CULTIVATING: Regex =
        Regex::new(r"^You have finished cultivating (.+)\.$").unwrap();
    static ref MANTRA_NO_LONGER_ACTIVE: Regex =
        Regex::new(r"^Your mantra of (.+) is no longer active\.$").unwrap();
    static ref RECITE_WITHOUT_CULTIVATION: Regex = Regex::new(
        r"^You decide not to recite the (.+) as you are not actively cultivating anything\.$",
    )
    .unwrap();
}

pub fn sect_cultivation_hilite_trigger(
    line: &TriggerLine<'_>,
    _facts: &TriggerFacts,
) -> TriggerEffects {
    let line = line.plain_line;
    if FINISHED_CULTIVATING.is_match(line) {
        TriggerEffects::none().style_line(TextStyle::GREEN)
    } else if MANTRA_NO_LONGER_ACTIVE.is_match(line) {
        TriggerEffects::none().style_line(TextStyle::YELLOW)
    } else if RECITE_WITHOUT_CULTIVATION.is_match(line) {
        TriggerEffects::none().style_line(TextStyle::RED)
    } else {
        TriggerEffects::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn run(line: &str) -> StyledLine {
        let output =
            sect_cultivation_hilite_trigger(&TriggerLine::new(line), &TriggerFacts::default());
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        styled
    }

    #[test]
    fn finished_cultivating_is_green_not_bold() {
        let styled = run("You have finished cultivating serenity.");
        for c in &styled.styled_chars {
            assert_eq!(c.color, AnsiCode::Green);
            assert!(!c.bold);
        }
    }

    #[test]
    fn mantra_inactive_is_yellow_not_bold() {
        let styled = run("Your mantra of calm focus is no longer active.");
        for c in &styled.styled_chars {
            assert_eq!(c.color, AnsiCode::Yellow);
            assert!(!c.bold);
        }
    }

    #[test]
    fn recite_without_cultivation_is_red_not_bold() {
        let styled = run(
            "You decide not to recite the calm focus as you are not actively cultivating anything.",
        );
        for c in &styled.styled_chars {
            assert_eq!(c.color, AnsiCode::Red);
            assert!(!c.bold);
        }
    }

    #[test]
    fn unrelated_line_unchanged_default_color() {
        let styled = run("You perform the kata.");
        for c in &styled.styled_chars {
            assert_eq!(c.color, AnsiCode::DefaultColor);
            assert!(!c.bold);
        }
    }
}
