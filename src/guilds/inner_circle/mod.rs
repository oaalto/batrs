//! Inner Circle of Sorcery — TinyFugue parity with `tf/inner_circle.tf`.
//!
//! `tf/mage_inner_circle.tf` is not present in this repository; add or symlink it locally if needed.

mod commands;

use crate::command::Command;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

pub const INNER_CIRCLE_HAS_ENTITY_FLAG: &str = "inner_circle_has_entity";

#[derive(Default)]
pub struct InnerCircleGuild {}

impl Guild for InnerCircleGuild {
    fn commands(&self) -> HashMap<String, Command> {
        self.get_commands()
    }

    fn triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}
