//! Kharim guild (TinyFugue parity: `tf/done_kharim.tf`).

mod commands;
mod triggers;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

/// Kharim shortcuts mirrored from TinyFugue `tf/done_kharim.tf`.
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
