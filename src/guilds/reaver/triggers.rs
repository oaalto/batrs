use crate::ansi::{AnsiCode, StyledLine};
use crate::guilds::ReaverGuild;
use crate::triggers::Trigger;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use unicode_segmentation::UnicodeSegmentation;

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

    pub fn scythe_swipe_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if SCYTHE_SWIPE_REGEX.is_match(&styled_line.plain_line) {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        TriggerOutput::default()
    }

    pub fn rampant_cutting_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if RAMPANT_CUTTING_REGEXS
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        TriggerOutput::default()
    }

    pub fn reaver_strike_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if REAVER_STRIKE_REGEXS
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        TriggerOutput::default()
    }

    pub fn attack_fails_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if ATTACK_FAILS
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, true);
        }
        TriggerOutput::default()
    }

    pub fn killing_blow_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if KILLING_BLOW.is_match(&styled_line.plain_line) {
            styled_line.set_block_color("KILLING BLOW", AnsiCode::Red, true);
        }
        TriggerOutput::default()
    }

    pub fn speak_ancient_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain_line = styled_line.plain_line.clone();
        if let Some(captures) = SPEAK_ANCIENT.captures(&plain_line) {
            apply_capture_hilite(styled_line, &captures, 1, AnsiCode::White, true);
            apply_capture_hilite(styled_line, &captures, 2, AnsiCode::White, true);
        }
        TriggerOutput::default()
    }

    pub fn destructive_energy_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain_line = styled_line.plain_line.clone();
        if let Some(captures) = DESTRUCTIVE_ENERGY
            .iter()
            .find_map(|r| r.captures(&plain_line))
        {
            apply_capture_hilite(styled_line, &captures, 1, AnsiCode::Cyan, false);
        }
        TriggerOutput::default()
    }

    pub fn blue_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if BLUE_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        TriggerOutput::default()
    }

    pub fn magenta_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if MAGENTA_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Magenta, true);
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

    pub fn red_hilites_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if RED_HILITES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, false);
        }
        TriggerOutput::default()
    }

    pub fn threaten_usage_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if "Can only be used once per 10 minutes." == styled_line.plain_line {
            styled_line.gag = true;
        }
        TriggerOutput::default()
    }
}

fn apply_capture_hilite(
    styled_line: &mut StyledLine,
    captures: &Captures<'_>,
    index: usize,
    color: AnsiCode,
    bold: bool,
) {
    let Some(m) = captures.get(index) else {
        return;
    };

    let start = byte_to_grapheme_index(&styled_line.plain_line, m.start());
    let end = byte_to_grapheme_index(&styled_line.plain_line, m.end());
    let len = styled_line.styled_chars.len();
    let start = start.min(len);
    let end = end.min(len);

    for i in start..end {
        styled_line.styled_chars[i].color = color;
        styled_line.styled_chars[i].bold = bold;
    }
}

fn byte_to_grapheme_index(text: &str, byte_index: usize) -> usize {
    text.get(..byte_index)
        .map(|slice| slice.graphemes(true).count())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;

    #[test]
    fn speak_ancient_highlights_only_matches() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut line = StyledLine::new("You speak the ancient Ruun 'kael'");

        let _ = ReaverGuild::speak_ancient_trigger(&mut ctx, &mut line);

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
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut line =
            StyledLine::new("You feel you have released 42 amount of destructive energy.");

        let _ = ReaverGuild::destructive_energy_trigger(&mut ctx, &mut line);

        let amount_start = line.plain_line.find("42").unwrap();
        assert_eq!(line.styled_chars[amount_start].color, AnsiCode::Cyan);
        assert!(!line.styled_chars[amount_start].bold);
        assert_eq!(line.styled_chars[0].color, AnsiCode::DefaultColor);
    }
}
