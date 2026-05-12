use crate::ansi::{AnsiCode, StyledLine};
use crate::guilds::CivmageGuild;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use unicode_segmentation::UnicodeSegmentation;

impl CivmageGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::civmage_trigger]
    }

    pub fn civmage_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line
            .plain_line
            .trim_end_matches('\r')
            .trim()
            .to_string();

        if line == "You feel odd. Not weaker, but..." {
            styled_line.set_line_color(AnsiCode::Red, true);
            return output;
        }

        if line == "Your disc wavers dangerously." {
            output.lines.push(disc_notice());
            return output;
        }

        if try_analysis_paint(styled_line, line.as_str()) {
            return output;
        }

        output
    }
}

fn disc_notice() -> StyledLine {
    let mut line = StyledLine::new("FLOATING DISC IS GOING DOWN!");
    line.set_line_color(AnsiCode::Red, true);
    line
}

fn try_analysis_paint(styled_line: &mut StyledLine, line: &str) -> bool {
    let tiers: &[(&Regex, AnsiCode, bool)] = &[
        (&ANALYSIS_SCREAMS, AnsiCode::Green, false),
        (&ANALYSIS_WRITHES, AnsiCode::Blue, false),
        (&ANALYSIS_SHUDDERS, AnsiCode::Cyan, false),
        (&ANALYSIS_GRUNTS, AnsiCode::Yellow, false),
        (&ANALYSIS_WINCES, AnsiCode::Magenta, false),
        (&ANALYSIS_SHRUGS, AnsiCode::Red, false),
    ];
    for (regular_expression, color, bold) in tiers {
        if let Some(captures) = regular_expression.captures(line) {
            apply_capture_hilite(styled_line, &captures, 2, *color, *bold);
            return true;
        }
    }
    false
}

fn apply_capture_hilite(
    styled_line: &mut StyledLine,
    captures: &Captures<'_>,
    index: usize,
    color: AnsiCode,
    bold: bool,
) {
    let Some(matched) = captures.get(index) else {
        return;
    };

    let start = byte_to_grapheme_index(&styled_line.plain_line, matched.start());
    let end = byte_to_grapheme_index(&styled_line.plain_line, matched.end());
    let length = styled_line.styled_chars.len();
    let start = start.min(length);
    let end = end.min(length);

    for grapheme_index in start..end {
        styled_line.styled_chars[grapheme_index].color = color;
        styled_line.styled_chars[grapheme_index].bold = bold;
    }
}

fn byte_to_grapheme_index(textual: &str, byte_index: usize) -> usize {
    textual
        .get(..byte_index)
        .map(|slice| slice.graphemes(true).count())
        .unwrap_or_default()
}

lazy_static! {
    static ref ANALYSIS_SCREAMS: Regex = Regex::new(r"(.+) (screams in pain\.)").unwrap();
    static ref ANALYSIS_WRITHES: Regex = Regex::new(r"(.+) (writhes in agony\.)").unwrap();
    static ref ANALYSIS_SHUDDERS: Regex =
        Regex::new(r"(.+) (shudders from the force of the attack\.)").unwrap();
    static ref ANALYSIS_GRUNTS: Regex = Regex::new(r"(.+) (grunts from the pain\.)").unwrap();
    static ref ANALYSIS_WINCES: Regex =
        Regex::new(r"(.+) (winces a little from the pain\.)").unwrap();
    static ref ANALYSIS_SHRUGS: Regex = Regex::new(r"(.+) (shrugs off the attack\.)").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn run_trigger(line: &str) -> (TriggerOutput, StyledLine) {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut styled = StyledLine::new(line);
        let output = CivmageGuild::civmage_trigger(&mut ctx, &mut styled);
        (output, styled)
    }

    #[test]
    fn lift_side_effect_line_red_bold() {
        let (_output, styled) = run_trigger("You feel odd. Not weaker, but...");
        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
        assert!(styled.styled_chars[0].bold);
    }

    #[test]
    fn disc_waver_inserts_notice() {
        let (output, _) = run_trigger("Your disc wavers dangerously.");
        assert_eq!(output.lines.len(), 1);
        assert_eq!(output.lines[0].plain_line, "FLOATING DISC IS GOING DOWN!");
        assert_eq!(output.lines[0].styled_chars[0].color, AnsiCode::Red);
    }

    #[test]
    fn analysis_colors_second_capture_green() {
        let (_output, styled) = run_trigger("The orc screams in pain.");
        let pain_start = "The orc ".len();
        let pain_graphemes = "screams in pain.".graphemes(true).count();
        for offset in 0..pain_graphemes {
            let index = byte_to_grapheme_index(styled.plain_line.as_str(), pain_start) + offset;
            let cell = &styled.styled_chars[index];
            assert_eq!(cell.color, AnsiCode::Green);
        }
    }
}
