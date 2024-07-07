mod reaver;

pub use reaver::ReaverGuild;

use crate::command::Command;
use crate::triggers::Trigger;
use std::collections::HashMap;

pub trait Guild {
    fn commands(&self) -> HashMap<String, Box<dyn Command>>;
    fn triggers(&self) -> Vec<Box<dyn Trigger>>;
}
