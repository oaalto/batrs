use crate::command;
use crate::command::Command;
use crate::guilds::riftwalker::RIFTWALKER_HAS_ENTITY_FLAG;
use crate::guilds::{PsionicistGuild, cast_spell};
use std::collections::HashMap;

impl PsionicistGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cms".to_string(), Self::cast_mindseize as Command),
            ("cmb".to_string(), Self::cast_mind_blast as Command),
            ("cp".to_string(), Self::cast_psibolt as Command),
            ("cpb".to_string(), Self::cast_psi_blast as Command),
            ("cmd".to_string(), Self::cast_mind_disruption as Command),
            ("cpc".to_string(), Self::cast_psychic_crush as Command),
            ("cps".to_string(), Self::cast_psychic_storm as Command),
            ("cfs".to_string(), Self::cast_force_shield as Command),
            ("cpshield".to_string(), Self::cast_psionic_shield as Command),
            ("cmdev".to_string(), Self::cast_mind_development as Command),
            ("cgo".to_string(), Self::cast_phaze_shift as Command),
            ("med".to_string(), Self::use_meditation as Command),
            ("ch".to_string(), Self::cast_heal_self as Command),
            ("chf".to_string(), Self::repeat_heal_self as Command),
        ])
    }

    pub fn cast_mindseize(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("mindseize", data))
    }

    pub fn cast_mind_blast(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("mind blast", data))
    }

    pub fn cast_psibolt(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("psibolt", data))
    }

    pub fn cast_psi_blast(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("psi blast", data))
    }

    pub fn cast_mind_disruption(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("mind disruption", data))
    }

    pub fn cast_psychic_crush(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("psychic crush", data))
    }

    pub fn cast_psychic_storm(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("psychic storm", data))
    }

    pub fn cast_force_shield(
        data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if !args.is_empty() {
            return command::send(cast_spell("force shield", data));
        }
        let logical = if ctx.flag(RIFTWALKER_HAS_ENTITY_FLAG) {
            "cast force shield at entity"
        } else {
            "cast force shield at me"
        };
        command::send(crate::abilities::client_send_line(logical))
    }

    pub fn cast_psionic_shield(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(crate::abilities::client_send_line("cast psionic shield"))
    }

    pub fn cast_mind_development(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("mind development", data))
    }

    pub fn cast_phaze_shift(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("phaze shift", data))
    }

    pub fn use_meditation(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(crate::abilities::client_send_line("use meditation"))
    }

    pub fn repeat_heal_self(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(crate::abilities::repeat_inf_cast_heal_self())
    }

    pub fn cast_heal_self(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("heal self", data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abilities;
    use crate::command::CommandEnvironment;
    use crate::guilds::riftwalker::RIFTWALKER_HAS_ENTITY_FLAG;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandEnvironment {
        CommandEnvironment::empty()
    }

    fn ctx_with_entity_flag(has: bool) -> CommandEnvironment {
        CommandEnvironment::new(
            HashMap::from([(RIFTWALKER_HAS_ENTITY_FLAG.to_string(), has)]),
            HashMap::new(),
        )
    }

    #[test]
    fn mindseize_targets_like_cast_spell() {
        assert_eq!(
            PsionicistGuild::cast_mindseize(&data("cms", ""), &empty_ctx()),
            command::send("@cast 'mindseize'".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_mindseize(&data("cms", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'mindseize' orc".to_string())
        );
    }

    #[test]
    fn mind_blast_mobile_cannon_style_spell_names() {
        assert_eq!(
            PsionicistGuild::cast_mind_blast(&data("cmb", "troll"), &empty_ctx()),
            command::send("@target troll;cast 'mind blast' troll".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_phaze_shift(&data("cgo", ""), &empty_ctx()),
            command::send("@cast 'phaze shift'".to_string())
        );
    }

    #[test]
    fn force_shield_branches_match_expected() {
        assert_eq!(
            PsionicistGuild::cast_force_shield(&data("cfs", "self"), &ctx_with_entity_flag(true)),
            command::send(cast_spell("force shield", &data("cfs", "self")))
        );

        assert_eq!(
            abilities::targeted_cast("force shield", "self"),
            "target self;cast 'force shield' self"
        );

        assert_eq!(
            PsionicistGuild::cast_force_shield(&data("cfs", ""), &ctx_with_entity_flag(true)),
            command::send("@cast force shield at entity".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_force_shield(&data("cfs", ""), &ctx_with_entity_flag(false)),
            command::send("@cast force shield at me".to_string())
        );
    }

    #[test]
    fn psionic_shield_unquoted_meditation() {
        assert_eq!(
            PsionicistGuild::cast_psionic_shield(&data("cpshield", ""), &empty_ctx()),
            command::send("@cast psionic shield".to_string())
        );
        assert_eq!(
            PsionicistGuild::use_meditation(&data("med", ""), &empty_ctx()),
            command::send("@use meditation".to_string())
        );
    }

    #[test]
    fn repeat_heal_self_chf() {
        assert_eq!(
            PsionicistGuild::repeat_heal_self(&data("chf", ""), &empty_ctx()),
            command::send("@repeat inf cast heal self".to_string())
        );
    }

    #[test]
    fn cast_heal_self_ch() {
        assert_eq!(
            PsionicistGuild::cast_heal_self(&data("ch", ""), &empty_ctx()),
            command::send("@cast 'heal self'".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_heal_self(&data("ch", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'heal self' orc".to_string())
        );
    }
}
