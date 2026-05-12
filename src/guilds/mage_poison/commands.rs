//! Slash commands for Mage Poison.
//!
//! Aliases diverge from Folklorist (`cpb`, `cvs`, `cts`) and Mage Magical (`csgsp`)
//! where spell names overlap, so multi-guild configs keep distinct shortcuts.

use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::MagePoisonGuild;
use crate::guilds::cast_spell;
use std::collections::HashMap;

macro_rules! mage_poison_targeted_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &mut command::CommandContext,
        ) -> Option<String> {
            Some(cast_spell($spell, data))
        }
    };
}

impl MagePoisonGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ckcloud".to_string(), Self::cast_killing_cloud as Command),
            ("cnoxf".to_string(), Self::cast_noxious_fumes as Command),
            ("cmpb".to_string(), Self::cast_poison_blast as Command),
            ("cpscan".to_string(), Self::cast_poison_scan as Command),
            ("cpspr".to_string(), Self::cast_poison_spray as Command),
            ("cpowb".to_string(), Self::cast_power_blast as Command),
            (
                "cdetoxsh".to_string(),
                Self::cast_shield_of_detoxification as Command,
            ),
            (
                "ccrsp".to_string(),
                Self::cast_summon_carnal_spores as Command,
            ),
            (
                "cpgrsp".to_string(),
                Self::cast_summon_greater_spores as Command,
            ),
            ("cmpthorn".to_string(), Self::cast_thorn_spray as Command),
            ("ctoxdil".to_string(), Self::cast_toxic_dilution as Command),
            ("cmpvs".to_string(), Self::cast_venom_strike as Command),
        ])
    }

    mage_poison_targeted_cast!(cast_killing_cloud, "killing cloud");
    mage_poison_targeted_cast!(cast_noxious_fumes, "noxious fumes");
    mage_poison_targeted_cast!(cast_poison_blast, "poison blast");
    mage_poison_targeted_cast!(cast_poison_scan, "poison scan");
    mage_poison_targeted_cast!(cast_poison_spray, "poison spray");
    mage_poison_targeted_cast!(cast_power_blast, "power blast");
    mage_poison_targeted_cast!(cast_summon_carnal_spores, "summon carnal spores");
    mage_poison_targeted_cast!(cast_summon_greater_spores, "summon greater spores");
    mage_poison_targeted_cast!(cast_thorn_spray, "thorn spray");
    mage_poison_targeted_cast!(cast_toxic_dilution, "toxic dilution");
    mage_poison_targeted_cast!(cast_venom_strike, "venom strike");

    pub fn cast_shield_of_detoxification(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = data.args.trim();
        let at = if target.is_empty() { "me" } else { target };
        Some(abilities::client_send_line(&format!(
            "cast shield of detoxification at {at}"
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
    fn poison_scan_without_target() {
        assert_eq!(
            MagePoisonGuild::cast_poison_scan(&data("cpscan", ""), &mut empty_ctx()),
            Some("@cast 'poison scan'".to_string())
        );
    }

    #[test]
    fn poison_scan_with_target() {
        assert_eq!(
            MagePoisonGuild::cast_poison_scan(&data("cpscan", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'poison scan' orc".to_string())
        );
    }

    #[test]
    fn shield_of_detoxification_defaults_to_me() {
        assert_eq!(
            MagePoisonGuild::cast_shield_of_detoxification(&data("cdetoxsh", ""), &mut empty_ctx()),
            Some("@cast shield of detoxification at me".to_string())
        );
    }

    #[test]
    fn shield_of_detoxification_with_target() {
        assert_eq!(
            MagePoisonGuild::cast_shield_of_detoxification(
                &data("cdetoxsh", "ally"),
                &mut empty_ctx()
            ),
            Some("@cast shield of detoxification at ally".to_string())
        );
    }
}
