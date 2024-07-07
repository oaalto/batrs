use crate::command::Command;
use crate::guilds::reaver::scythe_swipe::ScytheSwipe;
use crate::guilds::Guild;
use crate::triggers::Trigger;
use std::collections::HashMap;

mod scythe_swipe;

#[derive(Default)]
pub struct ReaverGuild {}

impl Guild for ReaverGuild {
    fn commands(&self) -> HashMap<String, Box<dyn Command>> {
        HashMap::from([(
            "uss".to_string(),
            Box::new(ScytheSwipe::default()) as Box<dyn Command>,
        )])
    }

    fn triggers(&self) -> Vec<Box<dyn Trigger>> {
        Vec::new()
    }
}
