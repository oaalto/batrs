mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

/// Psionicist guild: slash-command parity with TinyFugue psionicist script (marked `done_` in `tf/` when complete).
///
/// The **`med`** alias overlaps Monk, Tiger, and Tzarakk; [`crate::command::process`] registers the
/// first loaded guild's handler for duplicate keys (`or_insert`).
#[derive(Default)]
pub struct PsionicistGuild {}

impl Guild for PsionicistGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        vec![PsionicistGuild::psionicist_trigger]
    }
}
