use crate::guilds::MageMagicalGuild;
use crate::guilds::magic_lore_analysis::magic_lore_analysis_effect;
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};

impl MageMagicalGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::mage_magical_trigger]
    }

    pub fn mage_magical_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let line = line.plain_line.trim_end_matches('\r').trim().to_string();

        if let Some(effect) = magic_lore_analysis_effect(line.as_str()) {
            let mut output = TriggerEffects::none();
            output.original.edits.push(effect);
            return output;
        }

        TriggerEffects::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::triggers::{TriggerFacts, TriggerLine};
    use unicode_segmentation::UnicodeSegmentation;

    fn run_trigger(line: &str) -> StyledLine {
        let output = MageMagicalGuild::mage_magical_trigger(
            &TriggerLine::new(line),
            &TriggerFacts::default(),
        );
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
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
