use crate::ansi::StyledLine;
use crate::guilds::MageGuild;
use crate::guilds::magic_lore_analysis::paint_magic_lore_analysis;
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};

impl MageGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::mage_trigger]
    }

    pub fn mage_trigger(
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
