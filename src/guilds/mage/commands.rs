use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::MageGuild;
use crate::guilds::{cast_spell, use_skill};
use std::collections::HashMap;

macro_rules! mage_cast_spell {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &mut command::CommandContext,
        ) -> Option<String> {
            Some(cast_spell($spell, data))
        }
    };
}

impl MageGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cere".to_string(), Self::use_ceremony as Command),
            ("ucs".to_string(), Self::use_create_staff as Command),
            ("cad".to_string(), Self::cast_aura_detection as Command),
            ("cct".to_string(), Self::cast_chill_touch as Command),
            ("ccf".to_string(), Self::cast_create_food as Command),
            ("cd".to_string(), Self::cast_darkness as Command),
            ("cdi".to_string(), Self::cast_disruption as Command),
            ("cfa".to_string(), Self::cast_flame_arrow as Command),
            ("cfab".to_string(), Self::cast_force_absorption as Command),
            ("cf".to_string(), Self::cast_floating as Command),
            ("cfl".to_string(), Self::cast_floating_letters as Command),
            ("chs".to_string(), Self::cast_heal_self as Command),
            ("ci".to_string(), Self::cast_identify as Command),
            ("cinv".to_string(), Self::cast_invisibility as Command),
            ("cl".to_string(), Self::cast_light as Command),
            ("cmm".to_string(), Self::cast_magic_missile as Command),
            ("cmb".to_string(), Self::cast_mana_barrier as Command),
            ("cmi".to_string(), Self::cast_mirror_image as Command),
            ("cms".to_string(), Self::cast_moon_sense as Command),
            ("cpb".to_string(), Self::cast_prismatic_burst as Command),
            ("cr".to_string(), Self::cast_relocate as Command),
            ("csi".to_string(), Self::cast_see_invisible as Command),
            ("csm".to_string(), Self::cast_see_magic as Command),
            ("csg".to_string(), Self::cast_shocking_grasp as Command),
            (
                "ctwe".to_string(),
                Self::cast_teleport_with_error as Command,
            ),
            (
                "ctw".to_string(),
                Self::cast_teleport_without_error as Command,
            ),
            ("cts".to_string(), Self::cast_thorn_spray as Command),
            ("cv".to_string(), Self::cast_vacuumbolt as Command),
            ("cww".to_string(), Self::cast_water_walking as Command),
            ("cwor".to_string(), Self::cast_word_of_recall as Command),
        ])
    }

    pub fn use_ceremony(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("ceremony", data))
    }

    pub fn use_create_staff(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("create staff", data))
    }

    pub fn cast_identify(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        Some(abilities::client_send_line(&format!(
            "cast identify at {at}"
        )))
    }

    pub fn cast_mirror_image(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        Some(abilities::client_send_line(&format!(
            "cast mirror image at {at}"
        )))
    }

    mage_cast_spell!(cast_aura_detection, "aura detection");
    mage_cast_spell!(cast_chill_touch, "chill touch");
    mage_cast_spell!(cast_create_food, "create food");
    mage_cast_spell!(cast_darkness, "darkness");
    mage_cast_spell!(cast_disruption, "disruption");
    mage_cast_spell!(cast_flame_arrow, "flame arrow");
    mage_cast_spell!(cast_floating, "floating");
    mage_cast_spell!(cast_floating_letters, "floating letters");
    mage_cast_spell!(cast_force_absorption, "force absorption");
    mage_cast_spell!(cast_heal_self, "heal self");
    mage_cast_spell!(cast_invisibility, "invisibility");
    mage_cast_spell!(cast_light, "light");
    mage_cast_spell!(cast_magic_missile, "magic missile");
    mage_cast_spell!(cast_mana_barrier, "mana barrier");
    mage_cast_spell!(cast_moon_sense, "moon sense");
    mage_cast_spell!(cast_prismatic_burst, "prismatic burst");
    mage_cast_spell!(cast_relocate, "relocate");
    mage_cast_spell!(cast_see_invisible, "see invisible");
    mage_cast_spell!(cast_see_magic, "see magic");
    mage_cast_spell!(cast_shocking_grasp, "shocking grasp");
    mage_cast_spell!(cast_teleport_with_error, "teleport with error");
    mage_cast_spell!(cast_teleport_without_error, "teleport without error");
    mage_cast_spell!(cast_thorn_spray, "thorn spray");
    mage_cast_spell!(cast_vacuumbolt, "vacuumbolt");
    mage_cast_spell!(cast_water_walking, "water walking");
    mage_cast_spell!(cast_word_of_recall, "word of recall");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandContext;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandContext {
        CommandContext::new(std::collections::HashMap::new(), true, String::new())
    }

    #[test]
    fn ceremony_without_target() {
        assert_eq!(
            MageGuild::use_ceremony(&data("cere", ""), &mut empty_ctx()),
            Some("@use 'ceremony'".to_string())
        );
    }

    #[test]
    fn ceremony_with_target() {
        assert_eq!(
            MageGuild::use_ceremony(&data("cere", "altar"), &mut empty_ctx()),
            Some("@target altar;use 'ceremony' altar".to_string())
        );
    }

    #[test]
    fn create_staff_without_target() {
        assert_eq!(
            MageGuild::use_create_staff(&data("ucs", ""), &mut empty_ctx()),
            Some("@use 'create staff'".to_string())
        );
    }

    #[test]
    fn create_staff_with_target() {
        assert_eq!(
            MageGuild::use_create_staff(&data("ucs", "branch"), &mut empty_ctx()),
            Some("@target branch;use 'create staff' branch".to_string())
        );
    }

    #[test]
    fn identify_defaults_to_me() {
        assert_eq!(
            MageGuild::cast_identify(&data("ci", ""), &mut empty_ctx()),
            Some("@cast identify at me".to_string())
        );
    }

    #[test]
    fn identify_with_target() {
        assert_eq!(
            MageGuild::cast_identify(&data("ci", "sword"), &mut empty_ctx()),
            Some("@cast identify at sword".to_string())
        );
    }

    #[test]
    fn mirror_image_defaults_to_me() {
        assert_eq!(
            MageGuild::cast_mirror_image(&data("cmi", ""), &mut empty_ctx()),
            Some("@cast mirror image at me".to_string())
        );
    }

    #[test]
    fn mirror_image_with_target() {
        assert_eq!(
            MageGuild::cast_mirror_image(&data("cmi", "ally"), &mut empty_ctx()),
            Some("@cast mirror image at ally".to_string())
        );
    }

    #[test]
    fn magic_missile_cast_spell() {
        assert_eq!(
            MageGuild::cast_magic_missile(&data("cmm", ""), &mut empty_ctx()),
            Some("@cast 'magic missile'".to_string())
        );
        assert_eq!(
            MageGuild::cast_magic_missile(&data("cmm", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'magic missile' orc".to_string())
        );
    }

    #[test]
    fn word_of_recall_cast_spell() {
        assert_eq!(
            MageGuild::cast_word_of_recall(&data("cwor", ""), &mut empty_ctx()),
            Some("@cast 'word of recall'".to_string())
        );
    }

    #[test]
    fn vacuumbolt_cast_spell() {
        assert_eq!(
            MageGuild::cast_vacuumbolt(&data("cv", ""), &mut empty_ctx()),
            Some("@cast 'vacuumbolt'".to_string())
        );
        assert_eq!(
            MageGuild::cast_vacuumbolt(&data("cv", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'vacuumbolt' orc".to_string())
        );
    }
}
