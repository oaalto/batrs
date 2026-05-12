//! Mage asphyxiation sub-guild (`tf/mage_asphyxiation.txt`).

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MageAsphyxiationGuild {}

impl Guild for MageAsphyxiationGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
