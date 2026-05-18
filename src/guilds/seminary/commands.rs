use crate::command;
use crate::command::Command;
use crate::guilds::{SeminaryGuild, cast_spell};
use std::collections::HashMap;

impl SeminaryGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([("chb".to_string(), Self::cast_harm_body as Command)])
    }

    pub fn cast_harm_body(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("harm body", data))
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
    fn harm_body_without_target() {
        let result = SeminaryGuild::cast_harm_body(&data("chb", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'harm body'".to_string()));
    }

    #[test]
    fn harm_body_with_target() {
        let result = SeminaryGuild::cast_harm_body(&data("chb", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;cast 'harm body' orc".to_string())
        );
    }
}
