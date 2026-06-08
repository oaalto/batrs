use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::ChannellersGuild;
use std::collections::HashMap;

impl ChannellersGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([("cdr".to_string(), Self::cast_drain_room as Command)])
    }

    pub fn cast_drain_room(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix("drain room", ""))
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
    fn drain_room_matches_expected_line() {
        let result = ChannellersGuild::cast_drain_room(&data("cdr", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'drain room'".to_string()));
    }
}
