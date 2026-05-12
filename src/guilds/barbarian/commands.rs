use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::BarbarianGuild;
use std::collections::HashMap;

impl BarbarianGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([("barb_rip".to_string(), Self::barb_rip as Command)])
    }

    pub fn barb_rip(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::compound_send(&[
            "rip_action set get all from corpse",
            "light torch",
            "barbburn",
            "extinguish torch",
            "drop zinc",
            "drop mowgles",
        ]))
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
    fn barb_rip_matches_expected_line() {
        let result = BarbarianGuild::barb_rip(&data("barb_rip", ""), &mut empty_ctx());
        assert_eq!(
            result,
            Some(
                "@rip_action set get all from corpse;light torch;barbburn;extinguish torch;drop zinc;drop mowgles"
                    .to_string()
            )
        );
    }
}
