//! Mage electricity sub-guild (TinyFugue parity: `tf/done_mage_lightning.tf`;
//! spell list: `tf/mage_electricity.txt`).

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MageElectricityGuild {}

impl Guild for MageElectricityGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
