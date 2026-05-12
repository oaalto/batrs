//! Line highlights for Kharim guild output.

use crate::ansi::{AnsiCode, StyledLine};
use crate::guilds::KharimGuild;
use crate::triggers::{TriggerContext, TriggerOutput};
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
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain = styled_line.plain_line.trim_end_matches('\r').trim();

        match plain {
            "Chaotic force closes the bleeding wound in your body." => {
                styled_line.set_line_color(AnsiCode::Green, true);
            }
            "Your blood circulation normalizes."
            | "Your thirst for blood is growing insatiable." => {
                styled_line.set_line_color(AnsiCode::Yellow, true);
            }
            "The flames surrounding your chaos blade subside." => {
                styled_line.set_line_color(AnsiCode::Yellow, false);
            }
            _ => {
                if CHAOS_AURA.is_match(plain) {
                    styled_line.set_line_color(AnsiCode::Green, false);
                } else if FOUL_INTENTIONS.is_match(plain) || ATTACK_FUTILE.is_match(plain) {
                    styled_line.set_line_color(AnsiCode::Red, false);
                }
            }
        }

        TriggerOutput::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn heals_line_bold_green() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Chaotic force closes the bleeding wound in your body.");

        KharimGuild::kharim_highlight_trigger(&mut trigger_ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Green);
        assert!(line.styled_chars[0].bold);
    }

    #[test]
    fn chaos_aura_green_not_bold() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Your chaos aura of fire reacts to the assault!");

        KharimGuild::kharim_highlight_trigger(&mut trigger_ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Green);
        assert!(!line.styled_chars[0].bold);
    }

    #[test]
    fn foul_intentions_red() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut line =
            StyledLine::new("The orc notices your foul intentions and evades your attempt.");

        KharimGuild::kharim_highlight_trigger(&mut trigger_ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(!line.styled_chars[0].bold);
    }
}
