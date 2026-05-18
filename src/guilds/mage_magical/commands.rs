//! Slash commands for Mage Magical, including spells beyond the minimal core set.

use crate::command;
use crate::command::Command;
use crate::guilds::MageMagicalGuild;
use crate::guilds::cast_spell;
use std::collections::HashMap;

macro_rules! mage_magical_targeted_cast {
    ($fn_name:ident, $spell:literal) => {
        pub fn $fn_name(
            data: &command::Data,
            _ctx: &command::CommandEnvironment,
        ) -> Vec<command::CommandEffect> {
            command::send(cast_spell($spell, data))
        }
    };
}

impl MageMagicalGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cmm".to_string(), Self::cast_magic_missile as Command),
            ("cgoa".to_string(), Self::cast_golden_arrow as Command),
            ("clevb".to_string(), Self::cast_levin_bolt as Command),
            ("cmdisp".to_string(), Self::cast_magic_dispersion as Command),
            ("cmerup".to_string(), Self::cast_magic_eruption as Command),
            ("cmwave".to_string(), Self::cast_magic_wave as Command),
            ("cmbeac".to_string(), Self::cast_mana_beacon as Command),
            ("crpars".to_string(), Self::cast_repulsor_aura as Command),
            (
                "csgsp".to_string(),
                Self::cast_summon_greater_spores as Command,
            ),
            (
                "cslsp".to_string(),
                Self::cast_summon_lesser_spores as Command,
            ),
        ])
    }

    mage_magical_targeted_cast!(cast_magic_missile, "magic missile");
    mage_magical_targeted_cast!(cast_golden_arrow, "golden arrow");
    mage_magical_targeted_cast!(cast_levin_bolt, "levin bolt");
    mage_magical_targeted_cast!(cast_magic_dispersion, "magic dispersion");
    mage_magical_targeted_cast!(cast_magic_eruption, "magic eruption");
    mage_magical_targeted_cast!(cast_magic_wave, "magic wave");
    mage_magical_targeted_cast!(cast_mana_beacon, "mana beacon");
    mage_magical_targeted_cast!(cast_repulsor_aura, "repulsor aura");
    mage_magical_targeted_cast!(cast_summon_greater_spores, "summon greater spores");
    mage_magical_targeted_cast!(cast_summon_lesser_spores, "summon lesser spores");
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
    fn magic_missile_without_target() {
        assert_eq!(
            MageMagicalGuild::cast_magic_missile(&data("cmm", ""), &empty_ctx()),
            command::send("@cast 'magic missile'".to_string())
        );
    }

    #[test]
    fn magic_missile_with_target() {
        assert_eq!(
            MageMagicalGuild::cast_magic_missile(&data("cmm", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'magic missile' orc".to_string())
        );
    }

    #[test]
    fn golden_arrow_cast_spell() {
        assert_eq!(
            MageMagicalGuild::cast_golden_arrow(&data("cgoa", ""), &empty_ctx()),
            command::send("@cast 'golden arrow'".to_string())
        );
    }
}
