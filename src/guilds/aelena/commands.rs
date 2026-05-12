use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::AelenaGuild;
use crate::guilds::{cast_spell, use_skill};
use std::collections::HashMap;

impl AelenaGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("csa".to_string(), Self::cast_sting_of_aelena as Command),
            ("csd".to_string(), Self::cast_slow_death_combo as Command),
            ("crb".to_string(), Self::cast_rusted_blade_combo as Command),
            ("cbt".to_string(), Self::cast_black_trance_combo as Command),
            (
                "cb".to_string(),
                Self::cast_bite_of_the_black_widow as Command,
            ),
            ("ccb".to_string(), Self::cast_command_blade as Command),
            ("uw".to_string(), Self::use_wound as Command),
            ("ut".to_string(), Self::use_thrust as Command),
            ("ud".to_string(), Self::use_dissection as Command),
            ("fc".to_string(), Self::familiar_consume as Command),
            (
                "fssd".to_string(),
                Self::familiar_store_slow_death as Command,
            ),
            (
                "fsrb".to_string(),
                Self::familiar_store_rusted_blade as Command,
            ),
            (
                "fsbt".to_string(),
                Self::familiar_store_black_trance as Command,
            ),
            ("rip_consume".to_string(), Self::rip_consume as Command),
            ("rip_dissect".to_string(), Self::rip_dissect as Command),
            ("rip_lung".to_string(), Self::rip_harvest_lung as Command),
            (
                "rip_spleen".to_string(),
                Self::rip_harvest_spleen as Command,
            ),
            ("rip_eye".to_string(), Self::rip_harvest_eye as Command),
        ])
    }

    pub fn cast_sting_of_aelena(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("sting of aelena", data))
    }

    pub fn cast_slow_death_combo(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            "aelena poison slow death",
            abilities::targeted_cast("sting of aelena", &data.args).as_str(),
        ]))
    }

    pub fn cast_rusted_blade_combo(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            "aelena poison rusted blade",
            abilities::targeted_cast("sting of aelena", &data.args).as_str(),
        ]))
    }

    pub fn cast_black_trance_combo(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let sting_target = match data.args.trim() {
            "" => "me",
            trimmed => trimmed,
        };
        Some(abilities::compound_send(&[
            "aelena poison black trance",
            abilities::targeted_cast("sting of aelena", sting_target).as_str(),
        ]))
    }

    pub fn cast_bite_of_the_black_widow(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("bite of the black widow", data))
    }

    pub fn cast_command_blade(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast command blade"))
    }

    pub fn use_wound(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(use_skill("wound", data))
    }

    pub fn use_thrust(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(use_skill("thrust", data))
    }

    pub fn use_dissection(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let parts: Vec<&str> = data.args.split_whitespace().collect();
        if parts.len() < 2 {
            ctx.push_output_line(StyledLine::new(
                "Need two arguments for dissection (see in-game help).",
            ));
            return None;
        }
        Some(abilities::client_send_line(&format!(
            "use dissection at corpse try {} {}",
            parts[0], parts[1]
        )))
    }

    pub fn familiar_consume(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let logical = if tail.is_empty() {
            "familiar consume".to_string()
        } else {
            format!("familiar consume {tail}")
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn familiar_store_slow_death(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("familiar store slow death"))
    }

    pub fn familiar_store_rusted_blade(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("familiar store rusted blade"))
    }

    pub fn familiar_store_black_trance(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("familiar store black trance"))
    }

    pub fn rip_consume(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(
            "@rip_action set get all from corpse;familiar consume corpse;drop zinc;drop mowgles"
                .to_string(),
        )
    }

    pub fn rip_dissect(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some("@rip_action set get all from corpse;drop zinc;drop mowgles".to_string())
    }

    pub fn rip_harvest_lung(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(
            "@rip_action set get all from corpse;familiar harvest lung any;drop zinc;drop mowgles"
                .to_string(),
        )
    }

    pub fn rip_harvest_spleen(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some("@rip_action set get all from corpse;familiar harvest spleen any;drop zinc;drop mowgles".to_string())
    }

    pub fn rip_harvest_eye(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(
            "@rip_action set get all from corpse;familiar harvest eye any;drop zinc;drop mowgles"
                .to_string(),
        )
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
        CommandContext::new(HashMap::new(), true, String::new())
    }

    #[test]
    fn sting_without_target() {
        let result = AelenaGuild::cast_sting_of_aelena(&data("csa", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'sting of aelena'".to_string()));
    }

    #[test]
    fn sting_with_target() {
        let result = AelenaGuild::cast_sting_of_aelena(&data("csa", "orc"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target orc;cast 'sting of aelena' orc".to_string())
        );
    }

    #[test]
    fn slow_death_combo() {
        let result = AelenaGuild::cast_slow_death_combo(&data("csd", "troll"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@aelena poison slow death;target troll;cast 'sting of aelena' troll".to_string())
        );
    }

    #[test]
    fn black_trance_defaults_sting_to_me() {
        let result = AelenaGuild::cast_black_trance_combo(&data("cbt", ""), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@aelena poison black trance;target me;cast 'sting of aelena' me".to_string())
        );
    }

    #[test]
    fn dissection_requires_two_args() {
        let mut ctx = empty_ctx();
        let result = AelenaGuild::use_dissection(&data("ud", "onlyone"), &mut ctx);
        assert!(result.is_none());
        assert_eq!(ctx.output_lines.len(), 1);
    }

    #[test]
    fn dissection_two_args() {
        let result = AelenaGuild::use_dissection(&data("ud", "lung left"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@use dissection at corpse try lung left".to_string())
        );
    }

    #[test]
    fn harvest_spleen_rip_line() {
        let result = AelenaGuild::rip_harvest_spleen(&data("rip_spleen", ""), &mut empty_ctx());
        assert_eq!(
            result,
            Some(
                "@rip_action set get all from corpse;familiar harvest spleen any;drop zinc;drop mowgles"
                    .to_string()
            )
        );
    }
}
