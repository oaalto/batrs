//! Mage magical sub-guild (TinyFugue parity: `tf/done_mage_magical.tf`; spell list:
//! `tf/mage_magical.txt`). Knowledge of magic lore uses shared analysis line coloring.

mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct MageMagicalGuild {}

impl Guild for MageMagicalGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }
}
