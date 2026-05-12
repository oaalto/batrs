use crate::ansi::{AnsiCode, StyledLine};
use crate::guilds::TigerGuild;
use crate::guilds::sects_triggers;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RED_HILITES: Vec<Regex> = vec![
        Regex::new(r"(.+) manages to resist your claws!").unwrap(),
        Regex::new(r"^Your fists are no longer surrounded by Curath's black flames\.$").unwrap(),
        Regex::new(r"^You do a complex attack maneuver but miss\.$").unwrap(),
    ];
    static ref GREEN_HILITES: Vec<Regex> = vec![
        Regex::new(r"As (.+) drops to (.+) knees you leap in for the kill!").unwrap(),
        Regex::new(r"You manage to stun (.+)\.$").unwrap(),
    ];
}

impl TigerGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![
            Self::red_hilites_trigger,
            Self::green_hilites_trigger,
            sects_triggers::sect_cultivation_hilite_trigger,
        ]
    }

    pub fn red_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if RED_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, true);
        }
        TriggerOutput::default()
    }

    pub fn green_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if GREEN_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Green, false);
        }
        TriggerOutput::default()
    }
}
