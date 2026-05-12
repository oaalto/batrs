mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

#[derive(Default)]
pub struct SpiderGuild {}

impl Guild for SpiderGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }
}
