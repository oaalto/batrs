mod reaver;

pub use reaver::ReaverGuild;

use crate::command::{Command, Data};
use crate::triggers::Trigger;
use std::collections::HashMap;

pub trait Guild {
    fn commands(&self) -> HashMap<String, Command>;
    fn triggers(&self) -> Vec<Box<dyn Trigger>>;
}

pub fn use_skill(skill_name: &str, data: &Data) -> String {
    if data.args.is_empty() {
        format!("@use '{}'", skill_name)
    } else {
        format!("@target {};use '{}' {}", &data.args, skill_name, &data.args)
    }
}

pub fn cast_spell(spell_name: &str, data: &Data) -> String {
    if data.args.is_empty() {
        format!("@cast '{}'", spell_name)
    } else {
        format!(
            "@target {};cast '{}' {}",
            &data.args, spell_name, &data.args
        )
    }
}
