use crate::ansi::{AnsiCode, StyledLine};
use crate::app::BatApp;
use crate::guilds::ReaverGuild;
use crate::triggers::Trigger;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SCYTHE_SWIPE_REGEX: Regex =
        Regex::new("You make a quick slash across (.+) body with your weapon.").unwrap();
    static ref RAMPANT_CUTTING_REGEXS: Vec<Regex> = vec![
        Regex::new("You slash upwards across (.+) torso with great force.").unwrap(),
        Regex::new("...and then strike again with a downwards blow!").unwrap()
    ];
    static ref REAVER_STRIKE_REGEXS: Vec<Regex> = vec![
        Regex::new("You score a nasty cut on (.+) shoulder.").unwrap(),
        Regex::new("You attack and swing again, ripping open (.+) side!").unwrap(),
        Regex::new("You attack and swing a THIRD time").unwrap()
    ];
    static ref ATTACK_FAILS: Vec<Regex> = vec![
        Regex::new("(.+) shifts position and you cannot hit the (.+) time.").unwrap(),
        Regex::new("Your frenzied attempts to destroy (.+) are easily deflected.").unwrap()
    ];
    static ref KILLING_BLOW: Regex = Regex::new("You score a KILLING BLOW on (.+)!").unwrap();
}

impl ReaverGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![
            Self::scythe_swipe_trigger,
            Self::rampant_cutting_trigger,
            Self::reaver_strike_trigger,
            Self::attack_fails_trigger,
            Self::killing_blow_trigger,
            Self::threaten_usage_trigger,
        ]
    }

    pub fn scythe_swipe_trigger(
        _app: &mut BatApp,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if SCYTHE_SWIPE_REGEX.is_match(&styled_line.plain_line) {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        vec![]
    }

    pub fn rampant_cutting_trigger(
        _app: &mut BatApp,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if RAMPANT_CUTTING_REGEXS
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        vec![]
    }

    pub fn reaver_strike_trigger(
        _app: &mut BatApp,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if REAVER_STRIKE_REGEXS
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        vec![]
    }

    pub fn attack_fails_trigger(
        _app: &mut BatApp,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if ATTACK_FAILS
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, true);
        }
        vec![]
    }

    pub fn killing_blow_trigger(
        _app: &mut BatApp,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if KILLING_BLOW.is_match(&styled_line.plain_line) {
            styled_line.set_block_color("KILLING BLOW", AnsiCode::Red, true);
        }
        vec![]
    }

    pub fn threaten_usage_trigger(
        _app: &mut BatApp,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if "Can only be used once per 10 minutes." == styled_line.plain_line {
            styled_line.gag = true;
        }
        vec![]
    }
}
