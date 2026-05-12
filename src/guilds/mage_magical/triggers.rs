use crate::ansi::StyledLine;
use crate::guilds::MageMagicalGuild;
use crate::guilds::magic_lore_analysis::paint_magic_lore_analysis;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};

impl MageMagicalGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::mage_magical_trigger]
    }

    pub fn mage_magical_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let line = styled_line
            .plain_line
            .trim_end_matches('\r')
            .trim()
            .to_string();

        if paint_magic_lore_analysis(styled_line, line.as_str()) {
            return TriggerOutput::default();
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
    use unicode_segmentation::UnicodeSegmentation;

    fn run_trigger(line: &str) -> StyledLine {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut styled = StyledLine::new(line);
        let _ = MageMagicalGuild::mage_magical_trigger(&mut ctx, &mut styled);
        styled
    }

    fn byte_to_grapheme_index(textual: &str, byte_index: usize) -> usize {
        textual
            .get(..byte_index)
            .map(|slice| slice.graphemes(true).count())
            .unwrap_or_default()
    }

    #[test]
    fn analysis_screams_paints_second_capture_green() {
        let styled = run_trigger("The orc screams in pain.");
        let pain_start = "The orc ".len();
        let pain_graphemes = "screams in pain.".graphemes(true).count();
        for offset in 0..pain_graphemes {
            let index = byte_to_grapheme_index(styled.plain_line.as_str(), pain_start) + offset;
            let cell = &styled.styled_chars[index];
            assert_eq!(cell.color, AnsiCode::Green);
        }
    }
}
