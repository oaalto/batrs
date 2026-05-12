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
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("harm body", data))
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
    fn harm_body_without_target() {
        let result = SeminaryGuild::cast_harm_body(&data("chb", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'harm body'".to_string()));
    }

    #[test]
    fn harm_body_with_target() {
        let result = SeminaryGuild::cast_harm_body(&data("chb", "orc"), &mut empty_ctx());
        assert_eq!(result, Some("@target orc;cast 'harm body' orc".to_string()));
    }
}
