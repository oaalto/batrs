//! Mage acid sub-guild (TinyFugue parity: `tf/done_mage_acid.tf`; spells: `tf/mage_acid.txt`).

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MageAcidGuild {}

impl Guild for MageAcidGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
