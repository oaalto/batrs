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
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::client_send_line(&format!(
            "cast lift of load at {at}"
        )))
    }

    pub fn cast_identify(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::client_send_line(&format!(
            "cast identify at {at}"
        )))
    }

    pub fn cast_floating_disc(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::floating_disc::send_cast_floating_disc())
    }

    pub fn get_all_from_disc(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::floating_disc::send_get_all_from_disc())
    }

    pub fn get_all_armour_from_disc(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::floating_disc::send_get_all_armour_from_disc())
    }

    pub fn get_all_weapon_from_disc(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::floating_disc::send_get_all_weapon_from_disc())
    }

    pub fn put_noeq_in_disc(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::floating_disc::send_put_noeq_in_disc())
    }

    pub fn repeat_heal_self(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::repeat_inf_cast_heal_self())
    }

    pub fn cast_heal_self(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("heal self", data))
    }

    pub fn cast_mirror_image(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::client_send_line(&format!(
            "cast mirror image at {at}"
        )))
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
    fn lift_of_load_default_me() {
        assert_eq!(
            CivmageGuild::cast_lift_of_load(&data("cld", ""), &empty_ctx()),
            command::send("@cast lift of load at me".to_string())
        );
    }

    #[test]
    fn lift_of_load_with_target() {
        assert_eq!(
            CivmageGuild::cast_lift_of_load(&data("cld", "orc"), &empty_ctx()),
            command::send("@cast lift of load at orc".to_string())
        );
    }

    #[test]
    fn identify_defaults_to_me_when_no_args() {
        assert_eq!(
            CivmageGuild::cast_identify(&data("cid", ""), &empty_ctx()),
            command::send("@cast identify at me".to_string())
        );
    }

    #[test]
    fn identify_with_target() {
        assert_eq!(
            CivmageGuild::cast_identify(&data("cid", "sword"), &empty_ctx()),
            command::send("@cast identify at sword".to_string())
        );
    }

    #[test]
    fn floating_disc() {
        assert_eq!(
            CivmageGuild::cast_floating_disc(&data("cfd", ""), &empty_ctx()),
            command::send("@cast floating disc".to_string())
        );
    }

    #[test]
    fn disc_inventory_commands() {
        assert_eq!(
            CivmageGuild::get_all_from_disc(&data("gad", ""), &empty_ctx()),
            command::send("@get all from my disc".to_string())
        );
        assert_eq!(
            CivmageGuild::get_all_armour_from_disc(&data("gaad", ""), &empty_ctx()),
            command::send("@get all armour from my disc".to_string())
        );
        assert_eq!(
            CivmageGuild::get_all_weapon_from_disc(&data("gawd", ""), &empty_ctx()),
            command::send("@get all weapon from my disc".to_string())
        );
        assert_eq!(
            CivmageGuild::put_noeq_in_disc(&data("pd", ""), &empty_ctx()),
            command::send("@put noeq in my disc".to_string())
        );
    }

    #[test]
    fn repeat_heal_self() {
        assert_eq!(
            CivmageGuild::repeat_heal_self(&data("chf", ""), &empty_ctx()),
            command::send("@repeat inf cast heal self".to_string())
        );
    }

    #[test]
    fn cast_heal_self_ch() {
        assert_eq!(
            CivmageGuild::cast_heal_self(&data("ch", ""), &empty_ctx()),
            command::send("@cast 'heal self'".to_string())
        );
        assert_eq!(
            CivmageGuild::cast_heal_self(&data("ch", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'heal self' orc".to_string())
        );
    }

    #[test]
    fn mirror_image_default_me() {
        assert_eq!(
            CivmageGuild::cast_mirror_image(&data("cmi", ""), &empty_ctx()),
            command::send("@cast mirror image at me".to_string())
        );
    }

    #[test]
    fn mirror_image_with_target() {
        assert_eq!(
            CivmageGuild::cast_mirror_image(&data("cmi", "gripe"), &empty_ctx()),
            command::send("@cast mirror image at gripe".to_string())
        );
    }
}
