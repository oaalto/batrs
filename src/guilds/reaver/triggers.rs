use crate::ansi::{AnsiCode, StyledLine};
use crate::triggers::TriggerContext;
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
        Regex::new("You attack and swing again").unwrap(),
        Regex::new("You attack and swing a THIRD time").unwrap()
    ];
    static ref ATTACK_FAILS: Vec<Regex> = vec![
        Regex::new("(.+) shifts position and you cannot hit the (.+) time.").unwrap(),
        Regex::new("Your frenzied attempts to destroy (.+) are easily deflected.").unwrap()
    ];
    static ref KILLING_BLOW: Regex =
        Regex::new("You score a \\(?KILLING BLOW\\)? on (.+)!?").unwrap();
    static ref SPEAK_ANCIENT: Regex =
        Regex::new("You speak the ancient (.+) '(.+)'").unwrap();
    static ref DESTRUCTIVE_ENERGY: Vec<Regex> = vec![
        Regex::new("You feel you have released (.+) amount of destructive energy.").unwrap(),
        Regex::new("You feel you have released (.+) amounts of destructive energy.").unwrap(),
    ];
    static ref BLUE_HILITES: Vec<Regex> = vec![
        Regex::new("You make a quick slash across (.+) body with your weapon.").unwrap(),
        Regex::new("You slash upwards across (.+) torso with great force.").unwrap(),
        Regex::new("...and then strike again with a downwards blow!").unwrap(),
        Regex::new("You score a nasty cut on (.+) shoulder.").unwrap(),
        Regex::new("You cut (.+) arm open with a powerful strike.").unwrap(),
        Regex::new("You attack and swing again").unwrap(),
        Regex::new("You attack and swing a THIRD time").unwrap(),
        Regex::new("You follow with a third strike to the cheek, coating (.+) face with blood!")
            .unwrap(),
        Regex::new("You attack and immediately hit (.+) stomach, throwing a crimson spray!")
            .unwrap(),
        Regex::new("You FINALLY shove your weapon right through (.+) chest!").unwrap(),
        Regex::new("You rake your weapon across (.+)").unwrap(),
        Regex::new("You slam your weapon into (.+)").unwrap(),
    ];
    static ref MAGENTA_HILITES: Vec<Regex> = vec![
        Regex::new("You feel the power slip from (.+).").unwrap(),
        Regex::new("You see (.+) revert to its normal shape.").unwrap(),
    ];
    static ref GREEN_HILITES: Vec<Regex> = vec![
        Regex::new("(.+) has been blighted!").unwrap(),
        Regex::new("Targets of race (.+) are added to your list.").unwrap(),
        Regex::new("Targets of type (.+) are added to your list.").unwrap(),
        Regex::new("Weapon type (.+) added to your list.").unwrap(),
        Regex::new("Clothing type (.+) added to your list.").unwrap(),
        Regex::new("The (.+) is destroyed in a mass of sparks!").unwrap(),
        Regex::new("The (.+) is smashed into a million pieces!").unwrap(),
    ];
    static ref RED_HILITES: Vec<Regex> = vec![
        Regex::new("You strike at (.+) but do no significant damage.").unwrap(),
    ];
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
            Self::speak_ancient_trigger,
            Self::destructive_energy_trigger,
            Self::blue_hilites_trigger,
            Self::magenta_hilites_trigger,
            Self::green_hilites_trigger,
            Self::red_hilites_trigger,
        ]
    }

    pub fn scythe_swipe_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if SCYTHE_SWIPE_REGEX.is_match(&styled_line.plain_line) {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        vec![]
    }

    pub fn rampant_cutting_trigger(
        _ctx: &mut TriggerContext<'_>,
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
        _ctx: &mut TriggerContext<'_>,
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
        _ctx: &mut TriggerContext<'_>,
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
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if KILLING_BLOW.is_match(&styled_line.plain_line) {
            styled_line.set_block_color("KILLING BLOW", AnsiCode::Red, true);
        }
        vec![]
    }

    pub fn speak_ancient_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if SPEAK_ANCIENT.is_match(&styled_line.plain_line) {
            styled_line.set_line_color(AnsiCode::White, true);
        }
        vec![]
    }

    pub fn destructive_energy_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if DESTRUCTIVE_ENERGY
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Cyan, false);
        }
        vec![]
    }

    pub fn blue_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if BLUE_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        vec![]
    }

    pub fn magenta_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if MAGENTA_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Magenta, true);
        }
        vec![]
    }

    pub fn green_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if GREEN_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Green, false);
        }
        vec![]
    }

    pub fn red_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if RED_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, false);
        }
        vec![]
    }

    pub fn threaten_usage_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> Vec<StyledLine> {
        if "Can only be used once per 10 minutes." == styled_line.plain_line {
            styled_line.gag = true;
        }
        vec![]
    }
}
