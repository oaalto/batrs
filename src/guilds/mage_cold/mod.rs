//! Mage cold sub-guild — spells from `tf/mage_cold.txt` (no TinyFugue script in-repo).

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MageColdGuild {}

impl Guild for MageColdGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
