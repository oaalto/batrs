use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{CurateGuild, cast_spell};
use std::collections::HashMap;

impl CurateGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ch".to_string(), Self::cast_hemorrhage as Command),
            ("ca".to_string(), Self::cast_aneurysm as Command),
            ("cmc".to_string(), Self::cast_mobile_cannon as Command),
            ("dmed".to_string(), Self::use_dark_meditation as Command),
        ])
    }

    pub fn cast_hemorrhage(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("hemorrhage", data))
    }

    pub fn cast_aneurysm(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("aneurysm", data))
    }

    pub fn cast_mobile_cannon(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("mobile cannon", data))
    }

    pub fn use_dark_meditation(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let logical = match data.args.trim() {
            "hp" => "use dark meditation at sacrifice health",
            "sp" => "use dark meditation at sacrifice power",
            _ => "use dark meditation at sacrifice endurance",
        };
        Some(abilities::client_send_line(logical))
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
        CommandContext::new(HashMap::new(), true)
    }

    #[test]
    fn hemorrhage_without_target() {
        let result = CurateGuild::cast_hemorrhage(&data("ch", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'hemorrhage'".to_string()));
    }

    #[test]
    fn hemorrhage_with_target() {
        let result = CurateGuild::cast_hemorrhage(&data("ch", "orc"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target orc;cast 'hemorrhage' orc".to_string())
        );
    }

    #[test]
    fn aneurysm_without_target() {
        let result = CurateGuild::cast_aneurysm(&data("ca", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'aneurysm'".to_string()));
    }

    #[test]
    fn aneurysm_with_target() {
        let result = CurateGuild::cast_aneurysm(&data("ca", "troll"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target troll;cast 'aneurysm' troll".to_string())
        );
    }

    #[test]
    fn mobile_cannon_without_target() {
        let result = CurateGuild::cast_mobile_cannon(&data("cmc", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'mobile cannon'".to_string()));
    }

    #[test]
    fn mobile_cannon_with_target() {
        let result = CurateGuild::cast_mobile_cannon(&data("cmc", "orc"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target orc;cast 'mobile cannon' orc".to_string())
        );
    }

    #[test]
    fn dark_meditation_hp() {
        let result = CurateGuild::use_dark_meditation(&data("dmed", "hp"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@use dark meditation at sacrifice health".to_string())
        );
    }

    #[test]
    fn dark_meditation_sp() {
        let result = CurateGuild::use_dark_meditation(&data("dmed", "sp"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@use dark meditation at sacrifice power".to_string())
        );
    }

    #[test]
    fn dark_meditation_default_endurance_when_empty_or_unknown() {
        let empty = CurateGuild::use_dark_meditation(&data("dmed", ""), &mut empty_ctx());
        assert_eq!(
            empty,
            Some("@use dark meditation at sacrifice endurance".to_string())
        );

        let unknown = CurateGuild::use_dark_meditation(&data("dmed", "xy"), &mut empty_ctx());
        assert_eq!(
            unknown,
            Some("@use dark meditation at sacrifice endurance".to_string())
        );
    }
}
