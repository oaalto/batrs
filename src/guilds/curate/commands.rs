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
            ("cmcan".to_string(), Self::cast_mobile_cannon as Command),
            ("dmed".to_string(), Self::use_dark_meditation as Command),
        ])
    }

    pub fn cast_hemorrhage(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("hemorrhage", data))
    }

    pub fn cast_aneurysm(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("aneurysm", data))
    }

    pub fn cast_mobile_cannon(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("mobile cannon", data))
    }

    pub fn use_dark_meditation(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let logical = match data.args.trim() {
            "hp" => abilities::use_quoted_tail("dark meditation", "sacrifice health"),
            "sp" => abilities::use_quoted_tail("dark meditation", "sacrifice power"),
            _ => abilities::use_quoted_tail("dark meditation", "sacrifice endurance"),
        };
        command::send(abilities::client_send_line(&logical))
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
    fn hemorrhage_without_target() {
        let result = CurateGuild::cast_hemorrhage(&data("ch", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'hemorrhage'".to_string()));
    }

    #[test]
    fn hemorrhage_with_target() {
        let result = CurateGuild::cast_hemorrhage(&data("ch", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;cast 'hemorrhage' orc".to_string())
        );
    }

    #[test]
    fn aneurysm_without_target() {
        let result = CurateGuild::cast_aneurysm(&data("ca", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'aneurysm'".to_string()));
    }

    #[test]
    fn aneurysm_with_target() {
        let result = CurateGuild::cast_aneurysm(&data("ca", "troll"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target troll;cast 'aneurysm' troll".to_string())
        );
    }

    #[test]
    fn mobile_cannon_without_target() {
        let result = CurateGuild::cast_mobile_cannon(&data("cmcan", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'mobile cannon'".to_string()));
    }

    #[test]
    fn mobile_cannon_with_target() {
        let result = CurateGuild::cast_mobile_cannon(&data("cmcan", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;cast 'mobile cannon' orc".to_string())
        );
    }

    #[test]
    fn dark_meditation_hp() {
        let result = CurateGuild::use_dark_meditation(&data("dmed", "hp"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@use 'dark meditation' sacrifice health".to_string())
        );
    }

    #[test]
    fn dark_meditation_sp() {
        let result = CurateGuild::use_dark_meditation(&data("dmed", "sp"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@use 'dark meditation' sacrifice power".to_string())
        );
    }

    #[test]
    fn dark_meditation_default_endurance_when_empty_or_unknown() {
        let empty = CurateGuild::use_dark_meditation(&data("dmed", ""), &empty_ctx());
        assert_eq!(
            empty,
            command::send("@use 'dark meditation' sacrifice endurance".to_string())
        );

        let unknown = CurateGuild::use_dark_meditation(&data("dmed", "xy"), &empty_ctx());
        assert_eq!(
            unknown,
            command::send("@use 'dark meditation' sacrifice endurance".to_string())
        );
    }
}
