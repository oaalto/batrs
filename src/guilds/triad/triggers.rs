use crate::ansi::{StyledLine, TextStyle};
use crate::guilds::TriadGuild;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

impl TriadGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::triad_highlight_trigger]
    }

    pub fn triad_highlight_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let output = TriggerOutput::default();
        let line = styled_line.plain_line.trim_end_matches('\r').trim();

        if CURSE_FADE.is_match(line) || FAIL_REACH.is_match(line) {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
        } else if HARM.is_match(line) {
            styled_line.set_line_style(TextStyle::BRIGHT_GREEN);
        }

        output
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
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
        }
    }

    #[test]
    fn curse_fade_red() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("The curse on your weapon fades away.");

        let _ = TriadGuild::triad_highlight_trigger(&mut trigger_ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn fail_to_reach_red() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("You fail to reach the goblin.");

        let _ = TriadGuild::triad_highlight_trigger(&mut trigger_ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn harm_green() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut styled = StyledLine::new("You harm goblin a little.");

        let _ = TriadGuild::triad_highlight_trigger(&mut trigger_ctx, &mut styled);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
        assert!(styled.styled_chars[0].bold);
    }
}
