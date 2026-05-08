mod disciple;
mod monk;
mod reaver;

pub use disciple::DiscipleGuild;
pub use monk::MonkGuild;
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
    vec![
        GuildDefinition {
            key: "reaver",
            name: "Reaver",
        },
        GuildDefinition {
            key: "disciple",
            name: "Disciple",
        },
        GuildDefinition {
            key: "monk",
            name: "Monk",
        },
    ]
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
        if key.as_str() == "reaver" {
            guilds.push(Box::new(ReaverGuild::default()));
        }
        if key.as_str() == "disciple" {
            guilds.push(Box::new(DiscipleGuild::default()));
        }
        if key.as_str() == "monk" {
            guilds.push(Box::new(MonkGuild::default()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_skill_builds_targeted_commands() {
        let empty = Data {
            cmd: "use".to_string(),
            args: "".to_string(),
        };
        let with_args = Data {
            cmd: "use".to_string(),
            args: "orc".to_string(),
        };

        assert_eq!(use_skill("scythe swipe", &empty), "@use 'scythe swipe'");
        assert_eq!(
            use_skill("scythe swipe", &with_args),
            "@target orc;use 'scythe swipe' orc"
        );
    }

    #[test]
    fn cast_spell_builds_targeted_commands() {
        let empty = Data {
            cmd: "cast".to_string(),
            args: "".to_string(),
        };
        let with_args = Data {
            cmd: "cast".to_string(),
            args: "goblin".to_string(),
        };

        assert_eq!(cast_spell("word of spite", &empty), "@cast 'word of spite'");
        assert_eq!(
            cast_spell("word of spite", &with_args),
            "@target goblin;cast 'word of spite' goblin"
        );
    }
}
