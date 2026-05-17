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
            ("chf".to_string(), Self::repeat_heal_self as Command),
        ])
    }

    pub fn cast_mindseize(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("mindseize", data))
    }

    pub fn cast_mind_blast(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("mind blast", data))
    }

    pub fn cast_psibolt(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("psibolt", data))
    }

    pub fn cast_psi_blast(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("psi blast", data))
    }

    pub fn cast_mind_disruption(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("mind disruption", data))
    }

    pub fn cast_psychic_crush(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("psychic crush", data))
    }

    pub fn cast_psychic_storm(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("psychic storm", data))
    }

    pub fn cast_force_shield(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if !args.is_empty() {
            return Some(cast_spell("force shield", data));
        }
        let logical = if ctx.flag(RIFTWALKER_HAS_ENTITY_FLAG) {
            "cast force shield at entity"
        } else {
            "cast force shield at me"
        };
        Some(crate::abilities::client_send_line(logical))
    }

    pub fn cast_psionic_shield(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(crate::abilities::client_send_line("cast psionic shield"))
    }

    pub fn cast_mind_development(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("mind development", data))
    }

    pub fn cast_phaze_shift(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("phaze shift", data))
    }

    pub fn use_meditation(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(crate::abilities::client_send_line("use meditation"))
    }

    pub fn repeat_heal_self(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(crate::abilities::repeat_inf_cast_heal_self())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abilities;
    use crate::command::CommandContext;
    use crate::guilds::riftwalker::RIFTWALKER_HAS_ENTITY_FLAG;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandContext {
        CommandContext::new(HashMap::new(), true, String::new())
    }

    fn ctx_with_entity_flag(has: bool) -> CommandContext {
        CommandContext::new(
            HashMap::from([(RIFTWALKER_HAS_ENTITY_FLAG.to_string(), has)]),
            true,
            String::new(),
        )
    }

    #[test]
    fn mindseize_targets_like_cast_spell() {
        assert_eq!(
            PsionicistGuild::cast_mindseize(&data("cms", ""), &mut empty_ctx()),
            Some("@cast 'mindseize'".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_mindseize(&data("cms", "orc"), &mut empty_ctx()),
            Some("@target orc;cast 'mindseize' orc".to_string())
        );
    }

    #[test]
    fn mind_blast_mobile_cannon_style_spell_names() {
        assert_eq!(
            PsionicistGuild::cast_mind_blast(&data("cmb", "troll"), &mut empty_ctx()),
            Some("@target troll;cast 'mind blast' troll".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_phaze_shift(&data("cgo", ""), &mut empty_ctx()),
            Some("@cast 'phaze shift'".to_string())
        );
    }

    #[test]
    fn force_shield_branches_match_expected() {
        assert_eq!(
            PsionicistGuild::cast_force_shield(
                &data("cfs", "self"),
                &mut ctx_with_entity_flag(true)
            ),
            Some(cast_spell("force shield", &data("cfs", "self")))
        );

        assert_eq!(
            abilities::targeted_cast("force shield", "self"),
            "target self;cast 'force shield' self"
        );

        assert_eq!(
            PsionicistGuild::cast_force_shield(&data("cfs", ""), &mut ctx_with_entity_flag(true)),
            Some("@cast force shield at entity".to_string())
        );
        assert_eq!(
            PsionicistGuild::cast_force_shield(&data("cfs", ""), &mut ctx_with_entity_flag(false)),
            Some("@cast force shield at me".to_string())
        );
    }

    #[test]
    fn psionic_shield_unquoted_meditation() {
        assert_eq!(
            PsionicistGuild::cast_psionic_shield(&data("cpshield", ""), &mut empty_ctx()),
            Some("@cast psionic shield".to_string())
        );
        assert_eq!(
            PsionicistGuild::use_meditation(&data("med", ""), &mut empty_ctx()),
            Some("@use meditation".to_string())
        );
    }

    #[test]
    fn repeat_heal_self_chf() {
        assert_eq!(
            PsionicistGuild::repeat_heal_self(&data("chf", ""), &mut empty_ctx()),
            Some("@repeat inf cast heal self".to_string())
        );
    }
}
