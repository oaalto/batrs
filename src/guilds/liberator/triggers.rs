//! Line highlights for Liberator guild output.

use crate::ansi::{StyledLine, TextStyle};
use crate::guilds::LiberatorGuild;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref GHOST_FAREWELL: Regex =
        Regex::new(r"Ghost of (.+) whispers 'I must leave now. Good luck.'").unwrap();
}

impl LiberatorGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::liberator_highlight_trigger]
    }

    pub fn liberator_highlight_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain = styled_line.plain_line.trim_end_matches('\r').trim();

        match plain {
            "You swing and miss your mark!"
            | "Your greater light spell flickers briefly and disappears."
            | "Your holy glow fades." => {
                styled_line.set_line_style(TextStyle::BRIGHT_RED);
            }
            _ => {
                if GHOST_FAREWELL.is_match(plain) {
                    styled_line.set_line_style(TextStyle::RED);
                }
            }
        }

        TriggerOutput::default()
    }
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
    fn miss_line_bold_red() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You swing and miss your mark!");

        LiberatorGuild::liberator_highlight_trigger(&mut trigger_ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(line.styled_chars[0].bold);
    }

    #[test]
    fn ghost_farewell_red_not_bold() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut line =
            StyledLine::new("Ghost of the fallen knight whispers 'I must leave now. Good luck.'");

        LiberatorGuild::liberator_highlight_trigger(&mut trigger_ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(!line.styled_chars[0].bold);
    }
}
