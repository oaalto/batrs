use crate::ansi::TextStyle;
use crate::guilds::TriadGuild;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

impl TriadGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::triad_highlight_trigger]
    }

    pub fn triad_highlight_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let line = line.plain_line.trim_end_matches('\r').trim();

        if CURSE_FADE.is_match(line) || FAIL_REACH.is_match(line) {
            TriggerEffects::none().style_line(TextStyle::BRIGHT_RED)
        } else if HARM.is_match(line) {
            TriggerEffects::none().style_line(TextStyle::BRIGHT_GREEN)
        } else {
            TriggerEffects::none()
        }
    }
}

lazy_static! {
    static ref CURSE_FADE: Regex = Regex::new(r"^The curse on your (.+) fades away\.$").unwrap();
    static ref FAIL_REACH: Regex = Regex::new(r"^You fail to reach (.+)\.$").unwrap();
    static ref HARM: Regex =
        Regex::new(r"^You harm (.+) (a little|some|a good bit|a lot|really much)\.$",).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn run(line: &str) -> StyledLine {
        let output =
            TriadGuild::triad_highlight_trigger(&TriggerLine::new(line), &TriggerFacts::default());
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        styled
    }

    #[test]
    fn curse_fade_red() {
        let styled = run("The curse on your weapon fades away.");

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn fail_to_reach_red() {
        let styled = run("You fail to reach the goblin.");

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn harm_green() {
        let styled = run("You harm goblin a little.");

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
        assert!(styled.styled_chars[0].bold);
    }
}
