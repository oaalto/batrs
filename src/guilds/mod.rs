mod reaver;

pub use reaver::ReaverGuild;

use crate::automation::Automation;
use crate::command::{Command, Data};
use crate::triggers::Trigger;
use std::collections::{HashMap, HashSet};

pub trait Guild {
    fn commands(&self) -> HashMap<String, Command>;
    fn triggers(&self) -> Vec<Trigger>;
    fn register_automation(&self, _automation: &mut Automation) {}
}

#[derive(Clone, Debug)]
pub struct GuildDefinition {
    pub key: &'static str,
    pub name: &'static str,
}

pub fn guild_definitions() -> Vec<GuildDefinition> {
    vec![GuildDefinition {
        key: "reaver",
        name: "Reaver",
    }]
}

pub fn default_guild_keys() -> Vec<String> {
    vec!["reaver".to_string()]
}

pub fn build_guilds(keys: &[String]) -> Vec<Box<dyn Guild>> {
    let mut guilds: Vec<Box<dyn Guild>> = Vec::new();
    let mut seen = HashSet::new();

    for key in keys {
        if !seen.insert(key.to_string()) {
            continue;
        }
        match key.as_str() {
            "reaver" => guilds.push(Box::new(ReaverGuild::default())),
            _ => {}
        }
    }

    guilds
}

pub fn use_skill(skill_name: &str, data: &Data) -> String {
    if data.args.is_empty() {
        format!("@use '{skill_name}'")
    } else {
        format!("@target {};use '{}' {}", &data.args, skill_name, &data.args)
    }
}

pub fn cast_spell(spell_name: &str, data: &Data) -> String {
    if data.args.is_empty() {
        format!("@cast '{spell_name}'")
    } else {
        format!(
            "@target {};cast '{}' {}",
            &data.args, spell_name, &data.args
        )
    }
}
