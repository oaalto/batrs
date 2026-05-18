//! Slash commands for Mage Acid.

use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::MageAcidGuild;
use crate::guilds::cast_spell;
use std::collections::HashMap;

macro_rules! mage_acid_targeted_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &command::CommandEnvironment,
        ) -> Vec<command::CommandEffect> {
            command::send(cast_spell($spell, data))
        }
    };
}

impl MageAcidGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cdi".to_string(), Self::cast_disruption as Command),
            ("caw".to_string(), Self::cast_acid_wind as Command),
            ("caa".to_string(), Self::cast_acid_arrow as Command),
            ("car".to_string(), Self::cast_acid_ray as Command),
            ("cab".to_string(), Self::cast_acid_blast as Command),
            ("carain".to_string(), Self::cast_acid_rain as Command),
            ("cas".to_string(), Self::cast_acid_storm as Command),
            ("clb".to_string(), Self::cast_lock_biter as Command),
            (
                "ccshield".to_string(),
                Self::cast_corrosion_shield as Command,
            ),
            ("cashield".to_string(), Self::cast_acid_shield as Command),
        ])
    }

    mage_acid_targeted_cast!(cast_disruption, "disruption");
    mage_acid_targeted_cast!(cast_acid_wind, "acid wind");
    mage_acid_targeted_cast!(cast_acid_arrow, "acid arrow");
    mage_acid_targeted_cast!(cast_acid_ray, "acid ray");
    mage_acid_targeted_cast!(cast_acid_blast, "acid blast");
    mage_acid_targeted_cast!(cast_acid_rain, "acid rain");
    mage_acid_targeted_cast!(cast_acid_storm, "acid storm");

    pub fn cast_lock_biter(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("lock biter", data))
    }

    pub fn cast_corrosion_shield(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::client_send_line(&format!(
            "cast corrosion shield at {at}"
        )))
    }

    pub fn cast_acid_shield(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::client_send_line(&format!(
            "cast acid shield at {at}"
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
    fn disruption_without_target() {
        assert_eq!(
            MageAcidGuild::cast_disruption(&data("cdi", ""), &empty_ctx()),
            command::send("@cast 'disruption'".to_string())
        );
    }

    #[test]
    fn disruption_with_target() {
        assert_eq!(
            MageAcidGuild::cast_disruption(&data("cdi", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'disruption' orc".to_string())
        );
    }

    #[test]
    fn acid_wind_with_target() {
        assert_eq!(
            MageAcidGuild::cast_acid_wind(&data("caw", "troll"), &empty_ctx()),
            command::send("@target troll;cast 'acid wind' troll".to_string())
        );
    }

    #[test]
    fn lock_biter_cast_spell() {
        assert_eq!(
            MageAcidGuild::cast_lock_biter(&data("clb", ""), &empty_ctx()),
            command::send("@cast 'lock biter'".to_string())
        );
        assert_eq!(
            MageAcidGuild::cast_lock_biter(&data("clb", "gate"), &empty_ctx()),
            command::send("@target gate;cast 'lock biter' gate".to_string())
        );
    }

    #[test]
    fn corrosion_shield_defaults_to_me() {
        assert_eq!(
            MageAcidGuild::cast_corrosion_shield(&data("ccshield", ""), &empty_ctx()),
            command::send("@cast corrosion shield at me".to_string())
        );
    }

    #[test]
    fn corrosion_shield_with_target() {
        assert_eq!(
            MageAcidGuild::cast_corrosion_shield(&data("ccshield", "ally"), &empty_ctx()),
            command::send("@cast corrosion shield at ally".to_string())
        );
    }

    #[test]
    fn acid_shield_defaults_to_me() {
        assert_eq!(
            MageAcidGuild::cast_acid_shield(&data("cashield", ""), &empty_ctx()),
            command::send("@cast acid shield at me".to_string())
        );
    }

    #[test]
    fn acid_storm_alias_cas() {
        assert_eq!(
            MageAcidGuild::cast_acid_storm(&data("cas", ""), &empty_ctx()),
            command::send("@cast 'acid storm'".to_string())
        );
    }
}
