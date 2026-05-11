mod commands;
mod triggers;

use crate::automation::Automation;
use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

pub const RIFTWALKER_SKILL_VAR: &str = "riftwalker_skill";
pub const RIFTWALKER_ELEMENT_VAR: &str = "riftwalker_element";
pub const RIFTWALKER_HAS_ENTITY_FLAG: &str = "riftwalker_has_entity";

pub const FIRE_SKILL: &str = "blazing sunder";
pub const AIR_SKILL: &str = "suffocating embrace";
pub const EARTH_SKILL: &str = "earthen cover";
pub const WATER_SKILL: &str = "subjugating backwash";

/// Settings/automation keys (resolved to `entity` when unset in BatApp).
pub const ENTITY_LABEL_FIRE: &str = "riftwalker_entity_fire";
pub const ENTITY_LABEL_AIR: &str = "riftwalker_entity_air";
pub const ENTITY_LABEL_WATER: &str = "riftwalker_entity_water";
pub const ENTITY_LABEL_EARTH: &str = "riftwalker_entity_earth";

#[derive(Default)]
pub struct RiftwalkerGuild {}

impl Guild for RiftwalkerGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }

    fn register_automation(&self, automation: &mut Automation) {
        automation.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, false);
        automation.set_var(RIFTWALKER_SKILL_VAR, FIRE_SKILL.to_string());
        automation.set_var(RIFTWALKER_ELEMENT_VAR, "fire".to_string());
    }
}
