use crate::guilds::MageGuild;
use crate::guilds::magic_lore_analysis::magic_lore_analysis_effect;
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};

impl MageGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::mage_trigger]
    }

    pub fn mage_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let line = line.plain_line.trim_end_matches('\r').trim().to_string();

        if let Some(effect) = magic_lore_analysis_effect(line.as_str()) {
            let mut output = TriggerEffects::none();
            output.original.edits.push(effect);
            return output;
        }

        TriggerEffects::none()
    }
}
