use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{CivmageGuild, cast_spell};
use std::collections::HashMap;

impl CivmageGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cld".to_string(), Self::cast_lift_of_load as Command),
            ("cid".to_string(), Self::cast_identify as Command),
            ("cfd".to_string(), Self::cast_floating_disc as Command),
            ("gad".to_string(), Self::get_all_from_disc as Command),
            (
                "gaad".to_string(),
                Self::get_all_armour_from_disc as Command,
            ),
            (
                "gawd".to_string(),
                Self::get_all_weapon_from_disc as Command,
            ),
            ("pd".to_string(), Self::put_noeq_in_disc as Command),
            ("ch".to_string(), Self::cast_heal_self as Command),
            ("chf".to_string(), Self::repeat_heal_self as Command),
            ("cmi".to_string(), Self::cast_mirror_image as Command),
        ])
    }

    pub fn cast_lift_of_load(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        Some(abilities::client_send_line(&format!(
            "cast lift of load at {at}"
        )))
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

    pub fn cast_floating_disc(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::floating_disc::send_cast_floating_disc())
    }

    pub fn get_all_from_disc(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::floating_disc::send_get_all_from_disc())
    }

    pub fn get_all_armour_from_disc(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::floating_disc::send_get_all_armour_from_disc())
    }

    pub fn get_all_weapon_from_disc(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::floating_disc::send_get_all_weapon_from_disc())
    }

    pub fn put_noeq_in_disc(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::floating_disc::send_put_noeq_in_disc())
    }

    pub fn repeat_heal_self(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::repeat_inf_cast_heal_self())
    }

    pub fn cast_heal_self(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("heal self", data))
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
    fn lift_of_load_default_me() {
        assert_eq!(
            CivmageGuild::cast_lift_of_load(&data("cld", ""), &mut empty_ctx()),
            Some("@cast lift of load at me".to_string())
        );
    }

    #[test]
    fn lift_of_load_with_target() {
        assert_eq!(
            CivmageGuild::cast_lift_of_load(&data("cld", "orc"), &mut empty_ctx()),
            Some("@cast lift of load at orc".to_string())
        );
    }

    #[test]
    fn identify_defaults_to_me_when_no_args() {
        assert_eq!(
            CivmageGuild::cast_identify(&data("cid", ""), &mut empty_ctx()),
            Some("@cast identify at me".to_string())
        );
    }

    #[test]
    fn identify_with_target() {
        assert_eq!(
            CivmageGuild::cast_identify(&data("cid", "sword"), &mut empty_ctx()),
            Some("@cast identify at sword".to_string())
        );
    }

    #[test]
    fn floating_disc() {
        assert_eq!(
            CivmageGuild::cast_floating_disc(&data("cfd", ""), &mut empty_ctx()),
            Some("@cast floating disc".to_string())
        );
    }

    #[test]
    fn disc_inventory_commands() {
        assert_eq!(
            CivmageGuild::get_all_from_disc(&data("gad", ""), &mut empty_ctx()),
            Some("@get all from my disc".to_string())
        );
        assert_eq!(
            CivmageGuild::get_all_armour_from_disc(&data("gaad", ""), &mut empty_ctx()),
            Some("@get all armour from my disc".to_string())
        );
        assert_eq!(
            CivmageGuild::get_all_weapon_from_disc(&data("gawd", ""), &mut empty_ctx()),
            Some("@get all weapon from my disc".to_string())
        );
        assert_eq!(
            CivmageGuild::put_noeq_in_disc(&data("pd", ""), &mut empty_ctx()),
            Some("@put noeq in my disc".to_string())
        );
    }

    #[test]
    fn repeat_heal_self() {
        assert_eq!(
            CivmageGuild::repeat_heal_self(&data("chf", ""), &mut empty_ctx()),
            Some("@repeat inf cast heal self".to_string())
        );
    }

    #[test]
    fn cast_heal_self_ch() {
        assert_eq!(
            CivmageGuild::cast_heal_self(&data("ch", ""), &mut empty_ctx()),
            Some("@cast 'heal self'".to_string())
        );
        assert_eq!(
            CivmageGuild::cast_heal_self(&data("ch", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'heal self' orc".to_string())
        );
    }

    #[test]
    fn mirror_image_default_me() {
        assert_eq!(
            CivmageGuild::cast_mirror_image(&data("cmi", ""), &mut empty_ctx()),
            Some("@cast mirror image at me".to_string())
        );
    }

    #[test]
    fn mirror_image_with_target() {
        assert_eq!(
            CivmageGuild::cast_mirror_image(&data("cmi", "gripe"), &mut empty_ctx()),
            Some("@cast mirror image at gripe".to_string())
        );
    }
}
