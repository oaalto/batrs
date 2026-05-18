//! Line highlights for Kharim guild output.

use crate::ansi::TextStyle;
use crate::guilds::KharimGuild;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref CHAOS_AURA: Regex =
        Regex::new(r"Your chaos aura of (.+) reacts to the assault!").unwrap();
    static ref FOUL_INTENTIONS: Regex =
        Regex::new(r"(.+) notices your foul intentions and evades your attempt\.",).unwrap();
    static ref ATTACK_FUTILE: Regex =
        Regex::new(r"Your attempts to attack (.+) become futile\.").unwrap();
}

impl KharimGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::kharim_highlight_trigger]
    }

    pub fn kharim_highlight_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let plain = line.plain_line.trim_end_matches('\r').trim();

        match plain {
            "Chaotic force closes the bleeding wound in your body." => {
                TriggerEffects::none().style_line(TextStyle::BRIGHT_GREEN)
            }
            "Your blood circulation normalizes."
            | "Your thirst for blood is growing insatiable." => {
                TriggerEffects::none().style_line(TextStyle::BRIGHT_YELLOW)
            }
            "The flames surrounding your chaos blade subside." => {
                TriggerEffects::none().style_line(TextStyle::YELLOW)
            }
            _ => {
                if CHAOS_AURA.is_match(plain) {
                    TriggerEffects::none().style_line(TextStyle::GREEN)
                } else if FOUL_INTENTIONS.is_match(plain) || ATTACK_FUTILE.is_match(plain) {
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
        let output = KharimGuild::kharim_highlight_trigger(
            &TriggerLine::new(line),
            &TriggerFacts::default(),
        );
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        styled
    }

    #[test]
    fn heals_line_bold_green() {
        let line = run("Chaotic force closes the bleeding wound in your body.");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Green);
        assert!(line.styled_chars[0].bold);
    }

    #[test]
    fn chaos_aura_green_not_bold() {
        let line = run("Your chaos aura of fire reacts to the assault!");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Green);
        assert!(!line.styled_chars[0].bold);
    }

    #[test]
    fn foul_intentions_red() {
        let line = run("The orc notices your foul intentions and evades your attempt.");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(!line.styled_chars[0].bold);
    }
}
