//! Slash commands for Mage Fire, including spells beyond the minimal core set.

use crate::abilities;
use crate::abilities::cast_spell;
use crate::command;
use crate::command::Command;
use crate::command::ShortcutEntry;
use crate::guilds::MageFireGuild;
use std::collections::HashMap;

macro_rules! mage_fire_targeted_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &command::CommandEnvironment,
        ) -> Vec<command::CommandEffect> {
            command::send(cast_spell($spell, data))
        }
    };
}

impl MageFireGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cfa".to_string(), Self::cast_flame_arrow as Command),
            ("cf".to_string(), Self::cast_firebolt as Command),
            ("cfb".to_string(), Self::cast_fire_blast as Command),
            ("cflshield".to_string(), Self::cast_flame_shield as Command),
            ("chred".to_string(), Self::cast_heat_reduction as Command),
            ("clvb".to_string(), Self::cast_lava_blast as Command),
            ("clvs".to_string(), Self::cast_lava_storm as Command),
            ("cmetb".to_string(), Self::cast_meteor_blast as Command),
            ("cmets".to_string(), Self::cast_meteor_swarm as Command),
            ("cserf".to_string(), Self::cast_searing_fervor as Command),
        ])
    }

    pub fn get_shortcut_catalog(&self) -> Vec<ShortcutEntry> {
        vec![
            ShortcutEntry::new("cfa", "Cast flame arrow."),
            ShortcutEntry::new("cf", "Cast firebolt."),
            ShortcutEntry::new("cfb", "Cast fire blast."),
            ShortcutEntry::new("cflshield", "Cast flame shield on self or a target."),
            ShortcutEntry::new("chred", "Cast heat reduction."),
            ShortcutEntry::new("clvb", "Cast lava blast."),
            ShortcutEntry::new("clvs", "Cast lava storm."),
            ShortcutEntry::new("cmetb", "Cast meteor blast."),
            ShortcutEntry::new("cmets", "Cast meteor swarm."),
            ShortcutEntry::new("cserf", "Cast searing fervor."),
        ]
    }

    mage_fire_targeted_cast!(cast_flame_arrow, "flame arrow");
    mage_fire_targeted_cast!(cast_firebolt, "firebolt");
    mage_fire_targeted_cast!(cast_fire_blast, "fire blast");
    mage_fire_targeted_cast!(cast_heat_reduction, "heat reduction");
    mage_fire_targeted_cast!(cast_lava_blast, "lava blast");
    mage_fire_targeted_cast!(cast_lava_storm, "lava storm");
    mage_fire_targeted_cast!(cast_meteor_blast, "meteor blast");
    mage_fire_targeted_cast!(cast_meteor_swarm, "meteor swarm");
    mage_fire_targeted_cast!(cast_searing_fervor, "searing fervor");

    pub fn cast_flame_shield(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::cast_quoted_with_suffix("flame shield", at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandEnvironment;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandEnvironment {
        CommandEnvironment::empty()
    }

    #[test]
    fn flame_arrow_without_target() {
        assert_eq!(
            MageFireGuild::cast_flame_arrow(&data("cfa", ""), &empty_ctx()),
            command::send("@cast 'flame arrow'".to_string())
        );
    }

    #[test]
    fn flame_arrow_with_target() {
        assert_eq!(
            MageFireGuild::cast_flame_arrow(&data("cfa", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'flame arrow' orc".to_string())
        );
    }

    #[test]
    fn firebolt_with_target() {
        assert_eq!(
            MageFireGuild::cast_firebolt(&data("cf", "troll"), &empty_ctx()),
            command::send("@target troll;cast 'firebolt' troll".to_string())
        );
    }

    #[test]
    fn fire_blast_without_target() {
        assert_eq!(
            MageFireGuild::cast_fire_blast(&data("cfb", ""), &empty_ctx()),
            command::send("@cast 'fire blast'".to_string())
        );
    }

    #[test]
    fn flame_shield_defaults_to_me() {
        assert_eq!(
            MageFireGuild::cast_flame_shield(&data("cflshield", ""), &empty_ctx()),
            command::send("@cast 'flame shield' me".to_string())
        );
    }

    #[test]
    fn flame_shield_with_target() {
        assert_eq!(
            MageFireGuild::cast_flame_shield(&data("cflshield", "ally"), &empty_ctx()),
            command::send("@cast 'flame shield' ally".to_string())
        );
    }

    #[test]
    fn heat_reduction_cast_spell() {
        assert_eq!(
            MageFireGuild::cast_heat_reduction(&data("chred", ""), &empty_ctx()),
            command::send("@cast 'heat reduction'".to_string())
        );
    }
}
