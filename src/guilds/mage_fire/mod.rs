//! Mage fire sub-guild (TinyFugue parity: `tf/done_mage_fire.tf`; full spell list: `tf/mage_fire.txt`).

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MageFireGuild {}

impl Guild for MageFireGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
