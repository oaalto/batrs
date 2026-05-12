//! Slash commands for Mage Asphyxiation.

use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::MageAsphyxiationGuild;
use crate::guilds::cast_spell;
use std::collections::HashMap;

macro_rules! mage_asphyxiation_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &mut command::CommandContext,
        ) -> Option<String> {
            Some(cast_spell($spell, data))
        }
    };
}

impl MageAsphyxiationGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cairshield".to_string(), Self::cast_air_shield as Command),
            ("caowind".to_string(), Self::cast_aura_of_wind as Command),
            ("cbvac".to_string(), Self::cast_blast_vacuum as Command),
            ("cchaosb".to_string(), Self::cast_chaos_bolt as Command),
            ("cetherb".to_string(), Self::cast_ether_boundary as Command),
            ("cstrang".to_string(), Self::cast_strangulation as Command),
            ("csuff".to_string(), Self::cast_suffocation as Command),
            ("cvball".to_string(), Self::cast_vacuum_ball as Command),
            ("cvglobe".to_string(), Self::cast_vacuum_globe as Command),
            ("cvacb".to_string(), Self::cast_vacuumbolt as Command),
        ])
    }

    pub fn cast_air_shield(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        Some(abilities::client_send_line(&format!(
            "cast air shield at {at}"
        )))
    }

    mage_asphyxiation_cast!(cast_aura_of_wind, "aura of wind");
    mage_asphyxiation_cast!(cast_blast_vacuum, "blast vacuum");
    mage_asphyxiation_cast!(cast_chaos_bolt, "chaos bolt");
    mage_asphyxiation_cast!(cast_ether_boundary, "ether boundary");
    mage_asphyxiation_cast!(cast_strangulation, "strangulation");
    mage_asphyxiation_cast!(cast_suffocation, "suffocation");
    mage_asphyxiation_cast!(cast_vacuum_ball, "vacuum ball");
    mage_asphyxiation_cast!(cast_vacuum_globe, "vacuum globe");
    mage_asphyxiation_cast!(cast_vacuumbolt, "vacuumbolt");
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
    fn air_shield_defaults_to_me() {
        assert_eq!(
            MageAsphyxiationGuild::cast_air_shield(&data("cairshield", ""), &mut empty_ctx()),
            Some("@cast air shield at me".to_string())
        );
    }

    #[test]
    fn air_shield_with_target() {
        assert_eq!(
            MageAsphyxiationGuild::cast_air_shield(&data("cairshield", "ally"), &mut empty_ctx()),
            Some("@cast air shield at ally".to_string())
        );
    }

    #[test]
    fn vacuumbolt_cast_spell() {
        assert_eq!(
            MageAsphyxiationGuild::cast_vacuumbolt(&data("cvacb", ""), &mut empty_ctx()),
            Some("@cast 'vacuumbolt'".to_string())
        );
        assert_eq!(
            MageAsphyxiationGuild::cast_vacuumbolt(&data("cvacb", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'vacuumbolt' orc".to_string())
        );
    }
}
