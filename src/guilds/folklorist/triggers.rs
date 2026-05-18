use crate::ansi::TextStyle;
use crate::guilds::FolkloristGuild;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};

const MINOR_PROTECTION_FADES: &str = "The minor protection fades away.";

impl FolkloristGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::folklorist_highlight_trigger]
    }

    pub fn folklorist_highlight_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let plain = line.plain_line.trim_end_matches('\r').trim();

        if plain == MINOR_PROTECTION_FADES {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_RED);
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

    #[test]
    fn minor_protection_fade_prompt_red_bold() {
        let mut line = StyledLine::new(MINOR_PROTECTION_FADES);
        let output = FolkloristGuild::folklorist_highlight_trigger(
            &TriggerLine::new(MINOR_PROTECTION_FADES),
            &TriggerFacts::default(),
        );

        output.apply_line_effects_to(&mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(line.styled_chars[0].bold);
    }
}
