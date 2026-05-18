use crate::ansi::TextStyle;
use crate::guilds::TigerGuild;
use crate::guilds::sects_triggers;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
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

    pub fn red_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if RED_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_RED);
        }
        TriggerEffects::none()
    }

    pub fn green_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if GREEN_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::GREEN);
        }
        TriggerEffects::none()
    }
}
