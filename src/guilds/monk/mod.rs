mod commands;
mod triggers;

use crate::automation::Automation;
use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

pub const KATA_DONE_FLAG: &str = "monk_kata_done";
pub const DOING_MEDITATION_FLAG: &str = "monk_doing_meditation";

pub const CURRENT_ARMOUR_SKILL_VAR: &str = "monk_current_armour_skill";
pub const CURRENT_DISRUPT_SKILL_VAR: &str = "monk_current_disrupt_skill";
pub const CURRENT_AREA_SKILL_VAR: &str = "monk_current_area_skill";
pub const CURRENT_AVOID_SKILL_VAR: &str = "monk_current_avoid_skill";

pub const ARMOUR_SKILL_1: &str = "falling boulder strike";
pub const ARMOUR_SKILL_2: &str = "earthquake kick";
pub const ARMOUR_SKILL_3: &str = "falling boulder strike";

pub const DISRUPT_SKILL_1: &str = "wave crest strike";
pub const DISRUPT_SKILL_2: &str = "geyser force kick";
pub const DISRUPT_SKILL_3: &str = "tsunami push";

pub const AREA_SKILL_1: &str = "hydra fang strike";
pub const AREA_SKILL_2: &str = "winged horse kick";
pub const AREA_SKILL_3: &str = "hydra fang strike";

pub const AVOID_SKILL_1: &str = "falcon talon strike";
pub const AVOID_SKILL_2: &str = "elder cobra kick";
pub const AVOID_SKILL_3: &str = "falcon talon strike";

#[derive(Default)]
pub struct MonkGuild {}

impl Guild for MonkGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }

    fn register_automation(&self, automation: &mut Automation) {
        automation.set_flag(KATA_DONE_FLAG, false);
        automation.set_flag(DOING_MEDITATION_FLAG, false);
        automation.set_var(CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1.to_string());
        automation.set_var(CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1.to_string());
        automation.set_var(CURRENT_AREA_SKILL_VAR, AREA_SKILL_1.to_string());
        automation.set_var(CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1.to_string());
    }
}
