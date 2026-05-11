mod animist;
mod disciple;
mod monk;
mod reaver;
mod riftwalker;
mod tiger;
mod tzarakk;

pub use animist::AnimistGuild;
pub use disciple::DiscipleGuild;
pub use monk::MonkGuild;
pub use reaver::ReaverGuild;
pub use riftwalker::RiftwalkerGuild;
pub use tiger::TigerGuild;
pub use tzarakk::TzarakkGuild;

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
            key: "animist",
            name: "Animist",
        },
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
        GuildDefinition {
            key: "tzarakk",
            name: "Tzarakk",
        },
        GuildDefinition {
            key: "tiger",
            name: "Tiger",
        },
        GuildDefinition {
            key: "riftwalker",
            name: "Riftwalker",
        },
    ]
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
        if key.as_str() == "animist" {
            guilds.push(Box::new(AnimistGuild::default()));
        }
        if key.as_str() == "disciple" {
            guilds.push(Box::new(DiscipleGuild::default()));
        }
        if key.as_str() == "monk" {
            guilds.push(Box::new(MonkGuild::default()));
        }
        if key.as_str() == "tzarakk" {
            guilds.push(Box::new(TzarakkGuild::default()));
        }
        if key.as_str() == "tiger" {
            guilds.push(Box::new(TigerGuild::default()));
        }
        if key.as_str() == "riftwalker" {
            guilds.push(Box::new(RiftwalkerGuild::default()));
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

    #[test]
    fn animist_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "animist" && definition.name == "Animist")
        );

        let guilds = build_guilds(&["animist".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn tzarakk_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "tzarakk" && definition.name == "Tzarakk")
        );

        let guilds = build_guilds(&["tzarakk".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn tiger_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "tiger" && definition.name == "Tiger")
        );

        let guilds = build_guilds(&["tiger".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn riftwalker_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "riftwalker" && definition.name == "Riftwalker")
        );

        let guilds = build_guilds(&["riftwalker".to_string()]);

        assert_eq!(guilds.len(), 1);
    }
}
