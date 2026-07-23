//! Line highlights for Liberator guild output.

use crate::ansi::TextStyle;
use crate::guilds::LiberatorGuild;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

static GHOST_FAREWELL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Ghost of (.+) whispers 'I must leave now. Good luck.'").unwrap());

impl LiberatorGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::liberator_highlight_trigger]
    }

    pub fn liberator_highlight_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let plain = line.plain_line.trim_end_matches('\r').trim();

        match plain {
            "You swing and miss your mark!"
            | "Your greater light spell flickers briefly and disappears."
            | "Your holy glow fades." => TriggerEffects::none().style_line(TextStyle::BRIGHT_RED),
            _ => {
                if GHOST_FAREWELL.is_match(plain) {
                    TriggerEffects::none().style_line(TextStyle::RED)
                } else {
                    TriggerEffects::none()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn run(line: &str) -> StyledLine {
        let output = LiberatorGuild::liberator_highlight_trigger(
            &TriggerLine::new(line),
            &TriggerFacts::default(),
        );
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        styled
    }

    #[test]
    fn miss_line_bold_red() {
        let line = run("You swing and miss your mark!");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(line.styled_chars[0].bold);
    }

    #[test]
    fn ghost_farewell_red_not_bold() {
        let line = run("Ghost of the fallen knight whispers 'I must leave now. Good luck.'");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(!line.styled_chars[0].bold);
    }
}
