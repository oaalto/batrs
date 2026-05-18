use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::RangerGuild;
use std::collections::HashMap;

impl RangerGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ubf".to_string(), Self::use_bladed_fury as Command),
            ("cs".to_string(), Self::start_combat as Command),
            ("utc".to_string(), Self::use_torch_creation as Command),
        ])
    }

    pub fn use_bladed_fury(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.is_empty() {
            command::send(abilities::client_send_line("use 'bladed fury'"))
        } else {
            command::send(abilities::client_send_line(&abilities::targeted_use(
                "bladed fury",
                &data.args,
            )))
        }
    }

    pub fn start_combat(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.is_empty() {
            vec![command::output(StyledLine::new("No target!"))]
        } else {
            command::send(abilities::client_send_line(&format!(
                "target {};use 'bladed fury' {};@k {}",
                data.args, data.args, data.args
            )))
        }
    }

    pub fn use_torch_creation(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("use 'torch creation'"))
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
    fn bladed_fury_without_target() {
        let result = RangerGuild::use_bladed_fury(&data("ubf", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'bladed fury'".to_string()));
    }

    #[test]
    fn bladed_fury_with_target() {
        let result = RangerGuild::use_bladed_fury(&data("ubf", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;use 'bladed fury' orc".to_string())
        );
    }

    #[test]
    fn start_combat_with_target() {
        let result = RangerGuild::start_combat(&data("cs", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;use 'bladed fury' orc;@k orc".to_string())
        );
    }

    #[test]
    fn start_combat_without_target_shows_message() {
        let result = RangerGuild::start_combat(&data("cs", ""), &empty_ctx());
        assert_eq!(result, vec![command::output(StyledLine::new("No target!"))]);
    }

    #[test]
    fn torch_creation() {
        let result = RangerGuild::use_torch_creation(&data("utc", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'torch creation'".to_string()));
    }
}
