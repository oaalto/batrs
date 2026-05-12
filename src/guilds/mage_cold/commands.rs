//! Slash commands for Mage Cold.

use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::MageColdGuild;
use crate::guilds::cast_spell;
use std::collections::HashMap;

macro_rules! mage_cold_targeted_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &mut command::CommandContext,
        ) -> Option<String> {
            Some(cast_spell($spell, data))
        }
    };
}

impl MageColdGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cht".to_string(), Self::cast_chill_touch as Command),
            ("ccray".to_string(), Self::cast_cold_ray as Command),
            ("ccoc".to_string(), Self::cast_cone_of_cold as Command),
            (
                "cfrostw".to_string(),
                Self::cast_create_frost_weapon as Command,
            ),
            ("cdfire".to_string(), Self::cast_darkfire as Command),
            ("cfice".to_string(), Self::cast_flaming_ice as Command),
            ("cfins".to_string(), Self::cast_frost_insulation as Command),
            ("cfrshield".to_string(), Self::cast_frost_shield as Command),
            ("chail".to_string(), Self::cast_hailstorm as Command),
            ("cib".to_string(), Self::cast_icebolt as Command),
            (
                "ctogw".to_string(),
                Self::cast_touch_of_glacial_winds as Command,
            ),
        ])
    }

    mage_cold_targeted_cast!(cast_chill_touch, "chill touch");
    mage_cold_targeted_cast!(cast_cold_ray, "cold ray");
    mage_cold_targeted_cast!(cast_cone_of_cold, "cone of cold");
    mage_cold_targeted_cast!(cast_create_frost_weapon, "create frost weapon");
    mage_cold_targeted_cast!(cast_darkfire, "darkfire");
    mage_cold_targeted_cast!(cast_flaming_ice, "flaming ice");
    mage_cold_targeted_cast!(cast_frost_insulation, "frost insulation");
    mage_cold_targeted_cast!(cast_hailstorm, "hailstorm");
    mage_cold_targeted_cast!(cast_icebolt, "icebolt");
    mage_cold_targeted_cast!(cast_touch_of_glacial_winds, "touch of glacial winds");

    pub fn cast_frost_shield(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        Some(abilities::client_send_line(&format!(
            "cast frost shield at {at}"
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
    fn cold_ray_without_target() {
        assert_eq!(
            MageColdGuild::cast_cold_ray(&data("ccray", ""), &mut empty_ctx()),
            Some("@cast 'cold ray'".to_string())
        );
    }

    #[test]
    fn cold_ray_with_target() {
        assert_eq!(
            MageColdGuild::cast_cold_ray(&data("ccray", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'cold ray' orc".to_string())
        );
    }

    #[test]
    fn frost_shield_defaults_to_me() {
        assert_eq!(
            MageColdGuild::cast_frost_shield(&data("cfrshield", ""), &mut empty_ctx()),
            Some("@cast frost shield at me".to_string())
        );
    }

    #[test]
    fn frost_shield_with_target() {
        assert_eq!(
            MageColdGuild::cast_frost_shield(&data("cfrshield", "ally"), &mut empty_ctx()),
            Some("@cast frost shield at ally".to_string())
        );
    }
}
