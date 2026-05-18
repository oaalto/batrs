//! Slash commands for Mage Electricity (lightning school; extended beyond the minimal core set).

use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::MageElectricityGuild;
use crate::guilds::cast_spell;
use std::collections::HashMap;

macro_rules! mage_electricity_targeted_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &command::CommandEnvironment,
        ) -> Vec<command::CommandEffect> {
            command::send(cast_spell($spell, data))
        }
    };
}

impl MageElectricityGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("csg".to_string(), Self::cast_shocking_grasp as Command),
            ("clb".to_string(), Self::cast_lightning_bolt as Command),
            ("cbl".to_string(), Self::cast_blast_lightning as Command),
            ("cfl".to_string(), Self::cast_forked_lightning as Command),
            ("ce".to_string(), Self::cast_electrocution as Command),
            ("ccl".to_string(), Self::cast_chain_lightning as Command),
            ("cls".to_string(), Self::cast_lightning_storm as Command),
            ("cench".to_string(), Self::cast_energy_channeling as Command),
            (
                "cltshield".to_string(),
                Self::cast_lightning_shield as Command,
            ),
            (
                "cmaglev".to_string(),
                Self::cast_magnetic_levitation as Command,
            ),
        ])
    }

    mage_electricity_targeted_cast!(cast_shocking_grasp, "shocking grasp");
    mage_electricity_targeted_cast!(cast_lightning_bolt, "lightning bolt");
    mage_electricity_targeted_cast!(cast_blast_lightning, "blast lightning");
    mage_electricity_targeted_cast!(cast_forked_lightning, "forked lightning");
    mage_electricity_targeted_cast!(cast_electrocution, "electrocution");
    mage_electricity_targeted_cast!(cast_chain_lightning, "chain lightning");
    mage_electricity_targeted_cast!(cast_lightning_storm, "lightning storm");
    mage_electricity_targeted_cast!(cast_energy_channeling, "energy channeling");
    mage_electricity_targeted_cast!(cast_magnetic_levitation, "magnetic levitation");

    pub fn cast_lightning_shield(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        command::send(abilities::client_send_line(&format!(
            "cast lightning shield at {at}"
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
    fn shocking_grasp_without_target() {
        assert_eq!(
            MageElectricityGuild::cast_shocking_grasp(&data("csg", ""), &empty_ctx()),
            command::send("@cast 'shocking grasp'".to_string())
        );
    }

    #[test]
    fn lightning_bolt_with_target() {
        assert_eq!(
            MageElectricityGuild::cast_lightning_bolt(&data("clb", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'lightning bolt' orc".to_string())
        );
    }

    #[test]
    fn lightning_shield_defaults_to_me() {
        assert_eq!(
            MageElectricityGuild::cast_lightning_shield(&data("cltshield", ""), &empty_ctx()),
            command::send("@cast lightning shield at me".to_string())
        );
    }

    #[test]
    fn lightning_shield_with_target() {
        assert_eq!(
            MageElectricityGuild::cast_lightning_shield(&data("cltshield", "ally"), &empty_ctx()),
            command::send("@cast lightning shield at ally".to_string())
        );
    }

    #[test]
    fn energy_channeling_cast_spell() {
        assert_eq!(
            MageElectricityGuild::cast_energy_channeling(&data("cench", ""), &empty_ctx()),
            command::send("@cast 'energy channeling'".to_string())
        );
    }
}
