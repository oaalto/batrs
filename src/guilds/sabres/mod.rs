mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

/// Automation template variable matching player settings key [`crate::config::SettingsTable::sabre_weapon`].
pub const SABRE_WEAPON_VAR: &str = "sabre_weapon";

#[derive(Default)]
pub struct SabresGuild {}

impl Guild for SabresGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }
}
