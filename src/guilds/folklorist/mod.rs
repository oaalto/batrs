//! Folklorist guild.

mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

/// Folklorist shortcuts aligned with common BatMUD Folklorist automation.
///
/// **Alias overlap:** `cpb`, `upl`, and `chb` are also used by Psionicist, Tiger, and Seminary.
/// [`crate::command::dispatch`] keeps the handler from the **first** selected guild in the player list.
#[derive(Default)]
pub struct FolkloristGuild {}

impl Guild for FolkloristGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.get_triggers()
    }
}
