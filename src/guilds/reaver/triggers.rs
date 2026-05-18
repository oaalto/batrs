use crate::ansi::TextStyle;
use crate::guilds::ReaverGuild;
use crate::triggers::Trigger;
use crate::triggers::{LineEffect, TriggerEffects, TriggerFacts, TriggerLine};
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
    static ref SPEAK_ANCIENT: Regex = Regex::new("You speak the ancient (.+) '(.+)'").unwrap();
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
    static ref RED_HILITES: Vec<Regex> =
        vec![Regex::new("You strike at (.+) but do no significant damage.").unwrap(),];
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

    pub fn scythe_swipe_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if SCYTHE_SWIPE_REGEX.is_match(line.plain_line) {
            return TriggerEffects::none().style_line(TextStyle::BLUE);
        }
        TriggerEffects::none()
    }

    pub fn rampant_cutting_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        if RAMPANT_CUTTING_REGEXS
            .iter()
            .any(|r| r.is_match(line.plain_line))
        {
            return TriggerEffects::none().style_line(TextStyle::BLUE);
        }
        TriggerEffects::none()
    }

    pub fn reaver_strike_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if REAVER_STRIKE_REGEXS
            .iter()
            .any(|r| r.is_match(line.plain_line))
        {
            return TriggerEffects::none().style_line(TextStyle::BLUE);
        }
        TriggerEffects::none()
    }

    pub fn attack_fails_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if ATTACK_FAILS.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_RED);
        }
        TriggerEffects::none()
    }

    pub fn killing_blow_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if KILLING_BLOW.is_match(line.plain_line) {
            return TriggerEffects::none().style_block("KILLING BLOW", TextStyle::BRIGHT_RED);
        }
        TriggerEffects::none()
    }

    pub fn speak_ancient_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let plain_line = line.plain_line;
        let mut output = TriggerEffects::none();
        if let Some(captures) = SPEAK_ANCIENT.captures(plain_line) {
            if let Some(effect) = capture_hilite_effect(&captures, 1, TextStyle::BRIGHT_WHITE) {
                output.original.edits.push(effect);
            }
            if let Some(effect) = capture_hilite_effect(&captures, 2, TextStyle::BRIGHT_WHITE) {
                output.original.edits.push(effect);
            }
        }
        output
    }

    pub fn destructive_energy_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let plain_line = line.plain_line;
        let mut output = TriggerEffects::none();
        if let Some(captures) = DESTRUCTIVE_ENERGY
            .iter()
            .find_map(|r| r.captures(plain_line))
            && let Some(effect) = capture_hilite_effect(&captures, 1, TextStyle::CYAN)
        {
            output.original.edits.push(effect);
        }
        output
    }

    pub fn blue_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if BLUE_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::BLUE);
        }
        TriggerEffects::none()
    }

    pub fn magenta_hilites_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        if MAGENTA_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_MAGENTA);
        }
        TriggerEffects::none()
    }

    pub fn green_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if GREEN_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::GREEN);
        }
        TriggerEffects::none()
    }

    pub fn red_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if RED_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::RED);
        }
        TriggerEffects::none()
    }

    pub fn threaten_usage_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if "Can only be used once per 10 minutes." == line.plain_line {
            return TriggerEffects::none().gag();
        }
        TriggerEffects::none()
    }
}

fn capture_hilite_effect(
    captures: &regex::Captures<'_>,
    index: usize,
    style: TextStyle,
) -> Option<LineEffect> {
    let m = captures.get(index)?;
    Some(LineEffect::StylePlainByteRange {
        range: m.range(),
        style,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::triggers::{TriggerFacts, TriggerLine};

    #[test]
    fn speak_ancient_highlights_only_matches() {
        let mut line = StyledLine::new("You speak the ancient Ruun 'kael'");
        let output = ReaverGuild::speak_ancient_trigger(
            &TriggerLine::new("You speak the ancient Ruun 'kael'"),
            &TriggerFacts::default(),
        );

        output.apply_line_effects_to(&mut line);

        let ruun_start = line.plain_line.find("Ruun").unwrap();
        let kael_start = line.plain_line.find("kael").unwrap();
        let ruun_idx = ruun_start;
        let kael_idx = kael_start;

        assert_eq!(line.styled_chars[0].color, AnsiCode::DefaultColor);
        assert!(!line.styled_chars[0].bold);
        assert_eq!(line.styled_chars[ruun_idx].color, AnsiCode::White);
        assert!(line.styled_chars[ruun_idx].bold);
        assert_eq!(line.styled_chars[kael_idx].color, AnsiCode::White);
        assert!(line.styled_chars[kael_idx].bold);
    }

    #[test]
    fn destructive_energy_highlights_amount() {
        let mut line =
            StyledLine::new("You feel you have released 42 amount of destructive energy.");
        let output = ReaverGuild::destructive_energy_trigger(
            &TriggerLine::new("You feel you have released 42 amount of destructive energy."),
            &TriggerFacts::default(),
        );

        output.apply_line_effects_to(&mut line);

        let amount_start = line.plain_line.find("42").unwrap();
        assert_eq!(line.styled_chars[amount_start].color, AnsiCode::Cyan);
        assert!(!line.styled_chars[amount_start].bold);
        assert_eq!(line.styled_chars[0].color, AnsiCode::DefaultColor);
    }
}
