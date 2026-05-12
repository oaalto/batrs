//! Mage poison sub-guild — spells from `tf/mage_poison.txt` (no TinyFugue script in-repo).

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MagePoisonGuild {}

impl Guild for MagePoisonGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
