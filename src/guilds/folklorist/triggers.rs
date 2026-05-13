use crate::ansi::{StyledLine, TextStyle};
use crate::guilds::FolkloristGuild;
use crate::triggers::{TriggerContext, TriggerOutput};

const MINOR_PROTECTION_FADES: &str = "The minor protection fades away.";

impl FolkloristGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![Self::folklorist_highlight_trigger]
    }

    pub fn folklorist_highlight_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let plain = styled_line.plain_line.trim_end_matches('\r').trim();

        if plain == MINOR_PROTECTION_FADES {
            styled_line.set_line_style(TextStyle::BRIGHT_RED);
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

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
        }
    }

    #[test]
    fn minor_protection_fade_prompt_red_bold() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut trigger_ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new(MINOR_PROTECTION_FADES);

        FolkloristGuild::folklorist_highlight_trigger(&mut trigger_ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(line.styled_chars[0].bold);
    }
}
