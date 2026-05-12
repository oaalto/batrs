//! Kharim guild.

mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

/// Kharim shortcuts aligned with common BatMUD Kharim automation.
#[derive(Default)]
pub struct KharimGuild {}

impl Guild for KharimGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }
}
