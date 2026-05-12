mod aelena;
mod animist;
mod barbarian;
mod channellers;
mod curate;
mod disciple;
pub mod grouping;
mod grouping_catalog;
mod monk;
mod psionicist;
mod ranger;
mod reaver;
mod riftwalker;
mod tiger;
mod tzarakk;

pub use aelena::AelenaGuild;
pub use animist::AnimistGuild;
pub use barbarian::BarbarianGuild;
pub use channellers::ChannellersGuild;
pub use curate::CurateGuild;
pub use disciple::DiscipleGuild;
pub use monk::MonkGuild;
pub use psionicist::PsionicistGuild;
pub use ranger::RangerGuild;
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
        GuildDefinition {
            key: "ranger",
            name: "Ranger",
        },
        GuildDefinition {
            key: "aelena",
            name: "Aelena",
        },
        GuildDefinition {
            key: "barbarian",
            name: "Barbarian",
        },
        GuildDefinition {
            key: "channellers",
            name: "Channeller",
        },
        GuildDefinition {
            key: "curate",
            name: "Curate",
        },
        GuildDefinition {
            key: "psionicist",
            name: "Psionicist",
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
        if key.as_str() == "ranger" {
            guilds.push(Box::new(RangerGuild::default()));
        }
        if key.as_str() == "aelena" {
            guilds.push(Box::new(AelenaGuild::default()));
        }
        if key.as_str() == "barbarian" {
            guilds.push(Box::new(BarbarianGuild::default()));
        }
        if key.as_str() == "channellers" {
            guilds.push(Box::new(ChannellersGuild::default()));
        }
        if key.as_str() == "curate" {
            guilds.push(Box::new(CurateGuild::default()));
        }
        if key.as_str() == "psionicist" {
            guilds.push(Box::new(PsionicistGuild::default()));
        }
    }

    guilds
}

pub fn use_skill(skill_name: &str, data: &Data) -> String {
    crate::abilities::use_skill(skill_name, data)
}

pub fn cast_spell(spell_name: &str, data: &Data) -> String {
    crate::abilities::cast_spell(spell_name, data)
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

    #[test]
    fn ranger_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "ranger" && definition.name == "Ranger")
        );

        let guilds = build_guilds(&["ranger".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn aelena_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "aelena" && definition.name == "Aelena")
        );

        let guilds = build_guilds(&["aelena".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn barbarian_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "barbarian" && definition.name == "Barbarian")
        );

        let guilds = build_guilds(&["barbarian".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn channellers_is_registered_and_builds() {
        assert!(guild_definitions().iter().any(|definition| {
            definition.key == "channellers" && definition.name == "Channeller"
        }));

        let guilds = build_guilds(&["channellers".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn curate_is_registered_and_builds() {
        assert!(
            guild_definitions()
                .iter()
                .any(|definition| definition.key == "curate" && definition.name == "Curate")
        );

        let guilds = build_guilds(&["curate".to_string()]);

        assert_eq!(guilds.len(), 1);
    }

    #[test]
    fn psionicist_is_registered_and_builds() {
        assert!(guild_definitions().iter().any(|definition| {
            definition.key == "psionicist" && definition.name == "Psionicist"
        }));

        let guilds = build_guilds(&["psionicist".to_string()]);

        assert_eq!(guilds.len(), 1);
    }
}
